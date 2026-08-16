//! Concurrency — Multi-threaded stress and contention benchmark
//!
//! Measures:
//! - read_ops_per_sec: Read operations per second under concurrent multi-thread load
//! - write_ops_per_sec: Ingestion operations per second under concurrent multi-thread load
//! - read_latency_p50_ms, read_latency_p95_ms, read_latency_p99_ms: Read latency distribution
//! - write_latency_p50_ms, write_latency_p95_ms, write_latency_p99_ms: Write latency distribution
//! - cache_hit_rate: Hit rate of semantic / exact query caching under concurrent readers
//! - error_count: Number of concurrency failures, contention deadlocks, or query errors (target: 0)

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use chrono::Utc;
use parking_lot::Mutex;

use super::{Benchmark, BenchmarkResult};
use crate::search::result_cache::CacheFilterParams;
use crate::search::{AdaptiveCacheConfig, SearchResultCache};
use crate::storage::queries::{create_memory, get_memory};
use crate::storage::Storage;
use crate::types::{
    CreateMemoryInput, MatchInfo, MemoryType, SearchResult, StorageConfig, StorageMode,
};

/// Concurrency benchmark configuration
pub struct ConcurrencyBenchmark {
    /// Number of concurrent reader threads
    pub num_readers: usize,
    /// Number of concurrent writer threads
    pub num_writers: usize,
    /// Number of warmup memories preloaded into storage
    pub warmup_memories: usize,
    /// Duration to run the stress test
    pub duration_secs: u64,
}

impl Default for ConcurrencyBenchmark {
    fn default() -> Self {
        Self {
            num_readers: 4,
            num_writers: 2,
            warmup_memories: 200,
            duration_secs: 2,
        }
    }
}

impl ConcurrencyBenchmark {
    fn calculate_percentile(sorted_samples: &[f64], pct: f64) -> f64 {
        if sorted_samples.is_empty() {
            return 0.0;
        }
        let index = ((sorted_samples.len() as f64) * (pct / 100.0)).round() as usize;
        let clamped = index.min(sorted_samples.len().saturating_sub(1));
        sorted_samples[clamped]
    }
}

impl Benchmark for ConcurrencyBenchmark {
    fn name(&self) -> &str {
        "concurrency"
    }

    fn description(&self) -> &str {
        "Multi-threaded stress and contention benchmark (read/write ops/sec, cache contention, latency percentiles)"
    }

    fn run(&self, db_path: &str) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let storage = if db_path == ":memory:" || db_path.is_empty() {
            Storage::open_in_memory()?
        } else {
            let bench_path = format!("{}.concurrency.db", db_path);
            Storage::open(StorageConfig {
                db_path: bench_path,
                storage_mode: StorageMode::Local,
                cloud_uri: None,
                encrypt_cloud: false,
                confidence_half_life_days: 30.0,
                auto_sync: false,
                sync_debounce_ms: 5000,
            })?
        };

        // 1. Warmup: Insert initial corpus of memories
        let mut initial_ids = Vec::with_capacity(self.warmup_memories);
        storage.with_transaction(|conn| {
            for i in 0..self.warmup_memories {
                let input = CreateMemoryInput {
                    content: format!(
                        "Warmup benchmark memory record #{} with architectural facts and caching patterns",
                        i
                    ),
                    memory_type: MemoryType::Fact,
                    tags: vec![
                        "benchmark".to_string(),
                        "warmup".to_string(),
                        format!("batch_{}", i % 5),
                    ],
                    workspace: Some("bench_ws".to_string()),
                    importance: Some(0.8),
                    ..Default::default()
                };
                let mem = create_memory(conn, &input)?;
                initial_ids.push(mem.id);
            }
            Ok(())
        })?;

        let search_cache = Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default()));
        let read_latencies: Arc<Mutex<Vec<f64>>> = Arc::new(Mutex::new(Vec::new()));
        let write_latencies: Arc<Mutex<Vec<f64>>> = Arc::new(Mutex::new(Vec::new()));
        let read_ops = Arc::new(AtomicUsize::new(0));
        let write_ops = Arc::new(AtomicUsize::new(0));
        let cache_hits = Arc::new(AtomicUsize::new(0));
        let total_queries = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));

        let stop_flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut handles = Vec::new();
        let overall_start = Instant::now();

        // 2. Spawn Concurrent Reader Threads
        for reader_idx in 0..self.num_readers {
            let storage_clone = storage.clone();
            let search_cache_clone = Arc::clone(&search_cache);
            let read_latencies_clone = Arc::clone(&read_latencies);
            let read_ops_clone = Arc::clone(&read_ops);
            let cache_hits_clone = Arc::clone(&cache_hits);
            let total_queries_clone = Arc::clone(&total_queries);
            let error_count_clone = Arc::clone(&error_count);
            let stop_clone = Arc::clone(&stop_flag);
            let ids_clone = initial_ids.clone();

            handles.push(thread::spawn(move || {
                let mut local_latencies = Vec::with_capacity(1024);
                let query_patterns = [
                    "architectural facts",
                    "caching patterns",
                    "benchmark memory",
                    "warmup memory record",
                ];

                let filters = CacheFilterParams {
                    workspace: Some("bench_ws".to_string()),
                    tier: None,
                    memory_types: None,
                    include_archived: false,
                    include_transcripts: false,
                    tags: None,
                    global: false,
                    rerank_strategy: None,
                    policy_rerank: false,
                };

                while !stop_clone.load(Ordering::Relaxed) {
                    let start = Instant::now();
                    let q =
                        query_patterns[(reader_idx + local_latencies.len()) % query_patterns.len()];
                    total_queries_clone.fetch_add(1, Ordering::Relaxed);

                    // Check exact cache tier
                    if search_cache_clone.get_exact(q, &filters).is_some() {
                        cache_hits_clone.fetch_add(1, Ordering::Relaxed);
                    } else {
                        // Query storage by ID and perform read
                        let target_id = ids_clone[local_latencies.len() % ids_clone.len()];
                        let read_res =
                            storage_clone.with_connection(|conn| get_memory(conn, target_id));

                        match read_res {
                            Ok(mem) => {
                                search_cache_clone.put(
                                    q,
                                    None,
                                    filters.clone(),
                                    vec![SearchResult {
                                        memory: mem,
                                        score: 1.0,
                                        match_info: MatchInfo {
                                            strategy: crate::types::SearchStrategy::Hybrid,
                                            matched_terms: Vec::new(),
                                            highlights: Vec::new(),
                                            semantic_score: None,
                                            keyword_score: None,
                                        },
                                    }],
                                );
                            }
                            Err(_) => {
                                error_count_clone.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }

                    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                    local_latencies.push(elapsed_ms);
                    read_ops_clone.fetch_add(1, Ordering::Relaxed);
                }

                let mut guard = read_latencies_clone.lock();
                guard.extend(local_latencies);
            }));
        }

        // 3. Spawn Concurrent Writer Threads
        for writer_idx in 0..self.num_writers {
            let storage_clone = storage.clone();
            let search_cache_clone = Arc::clone(&search_cache);
            let write_latencies_clone = Arc::clone(&write_latencies);
            let write_ops_clone = Arc::clone(&write_ops);
            let error_count_clone = Arc::clone(&error_count);
            let stop_clone = Arc::clone(&stop_flag);

            handles.push(thread::spawn(move || {
                let mut local_latencies = Vec::with_capacity(512);
                let mut counter = 0;

                while !stop_clone.load(Ordering::Relaxed) {
                    let start = Instant::now();
                    counter += 1;

                    let input = CreateMemoryInput {
                        content: format!(
                            "Concurrent write memory stream writer={}, seq={}",
                            writer_idx, counter
                        ),
                        memory_type: MemoryType::Note,
                        tags: vec!["stream".to_string(), format!("w_{}", writer_idx)],
                        workspace: Some("bench_ws".to_string()),
                        importance: Some(0.7),
                        ..Default::default()
                    };

                    let res = storage_clone.with_transaction(|conn| create_memory(conn, &input));

                    match res {
                        Ok(_) => {
                            search_cache_clone.invalidate_for_workspace(Some("bench_ws"));
                            let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;
                            local_latencies.push(elapsed_ms);
                            write_ops_clone.fetch_add(1, Ordering::Relaxed);
                        }
                        Err(_) => {
                            error_count_clone.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }

                let mut guard = write_latencies_clone.lock();
                guard.extend(local_latencies);
            }));
        }

        // 4. Run for configured duration
        thread::sleep(Duration::from_secs(self.duration_secs));
        stop_flag.store(true, Ordering::Relaxed);

        for handle in handles {
            let _ = handle.join();
        }

        let total_duration_secs = overall_start.elapsed().as_secs_f64();
        let total_duration_ms = overall_start.elapsed().as_millis() as u64;

        let total_reads = read_ops.load(Ordering::Relaxed);
        let total_writes = write_ops.load(Ordering::Relaxed);
        let total_hits = cache_hits.load(Ordering::Relaxed);
        let total_q = total_queries.load(Ordering::Relaxed);
        let errors = error_count.load(Ordering::Relaxed);

        let mut read_samples = read_latencies.lock().clone();
        read_samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut write_samples = write_latencies.lock().clone();
        write_samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let mut metrics = HashMap::new();
        metrics.insert(
            "read_ops_per_sec".to_string(),
            total_reads as f64 / total_duration_secs,
        );
        metrics.insert(
            "write_ops_per_sec".to_string(),
            total_writes as f64 / total_duration_secs,
        );
        metrics.insert(
            "total_ops_per_sec".to_string(),
            (total_reads + total_writes) as f64 / total_duration_secs,
        );
        metrics.insert(
            "cache_hit_rate".to_string(),
            if total_q > 0 {
                total_hits as f64 / total_q as f64
            } else {
                0.0
            },
        );
        metrics.insert(
            "read_latency_p50_ms".to_string(),
            Self::calculate_percentile(&read_samples, 50.0),
        );
        metrics.insert(
            "read_latency_p95_ms".to_string(),
            Self::calculate_percentile(&read_samples, 95.0),
        );
        metrics.insert(
            "read_latency_p99_ms".to_string(),
            Self::calculate_percentile(&read_samples, 99.0),
        );
        metrics.insert(
            "write_latency_p50_ms".to_string(),
            Self::calculate_percentile(&write_samples, 50.0),
        );
        metrics.insert(
            "write_latency_p95_ms".to_string(),
            Self::calculate_percentile(&write_samples, 95.0),
        );
        metrics.insert(
            "write_latency_p99_ms".to_string(),
            Self::calculate_percentile(&write_samples, 99.0),
        );
        metrics.insert("error_count".to_string(), errors as f64);

        Ok(BenchmarkResult {
            name: "concurrency".to_string(),
            metrics,
            duration_ms: total_duration_ms,
            timestamp: Utc::now().to_rfc3339(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concurrency_benchmark_in_memory() {
        let bench = ConcurrencyBenchmark {
            num_readers: 2,
            num_writers: 1,
            warmup_memories: 20,
            duration_secs: 1,
        };

        let result = bench
            .run(":memory:")
            .expect("benchmark should run successfully");
        assert_eq!(result.name, "concurrency");
        assert!(result.metrics["read_ops_per_sec"] > 0.0);
        assert!(result.metrics["write_ops_per_sec"] > 0.0);
        assert_eq!(result.metrics["error_count"], 0.0);
    }
}
