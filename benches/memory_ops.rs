//! Performance benchmarks for core memory operations (RML-902).
//!
//! Covers the hot paths that run on every memory interaction:
//! create, get, list, cross-reference management, and aggregate stats.
//! The original groups use an in-memory SQLite database to isolate CPU cost
//! from disk I/O. Q7 adds groups that keep the storage mode separate so one
//! number is never read as another:
//!
//! - `storage_modes/{create,get,search}/{in_memory,disk_wal,disk_cloud_safe}`:
//!   same operation on `:memory:`, an on-disk WAL database (`StorageMode::Local`,
//!   synchronous=NORMAL) and an on-disk rollback-journal database
//!   (`StorageMode::CloudSafe`, synchronous=FULL).
//! - `storage_concurrency/{readers_N,readers_N_one_writer}`: N threads sharing a
//!   WAL `StoragePool` (one connection per thread slot). One iteration is a
//!   fixed batch per thread, so the time is wall time for the whole batch, not
//!   per operation.
//! - `latency_percentiles`: opt-in (`ENGRAM_BENCH_PERCENTILES=1`) manual timing
//!   that prints p50/p95/p99 per mode and operation after an explicit warmup.
//!   Criterion itself reports mean/median and confidence intervals, not
//!   percentiles.
//!
//! Disk groups use caller-owned temporary directories that are removed when the
//! group finishes. Numbers are local engineering evidence for the machine they
//! ran on; they are not hosted SLOs.
//!
//! Run with: `cargo bench --bench memory_ops`
//!
//! ## Performance targets
//! | Operation                  | Target  |
//! |----------------------------|---------|
//! | `memory_create/no_embedding` | < 200 µs |
//! | `memory_get/by_id`           | < 100 µs |

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use engram::search::bm25_search;
use engram::storage::queries::*;
use engram::storage::{Storage, StoragePool};
use engram::types::*;
use tempfile::{tempdir, TempDir};

/// Benchmark raw write throughput for `create_memory` without TF-IDF embedding.
///
/// Uses `defer_embedding: true` to isolate the SQLite insert cost from
/// embedding computation. This is the baseline for the write path.
fn bench_memory_create(c: &mut Criterion) {
    let storage = Storage::open_in_memory().unwrap();

    let mut group = c.benchmark_group("memory_create");
    group.throughput(Throughput::Elements(1));

    // Benchmark without embedding
    group.bench_function("no_embedding", |b| {
        b.iter(|| {
            storage
                .with_transaction(|conn| {
                    let id = rand::random::<u32>() % 1000;
                    let input = CreateMemoryInput {
                        content: format!("Test content for benchmarking purposes {}", id),
                        memory_type: MemoryType::Note,
                        tags: vec!["benchmark".to_string()],
                        metadata: Default::default(),
                        importance: Some(0.5),
                        defer_embedding: true,
                        scope: MemoryScope::Global,
                        ttl_seconds: None,
                        dedup_mode: DedupMode::Allow,
                        dedup_threshold: None,
                        workspace: Some("default".to_string()),
                        tier: MemoryTier::Permanent,
                        event_time: None,
                        event_duration_seconds: None,
                        trigger_pattern: None,
                        summary_of_id: None,
                        media_url: None,
                    };
                    create_memory(conn, black_box(&input))
                })
                .unwrap()
        })
    });

    group.finish();
}

/// Benchmark single-row read latency for `get_memory` by primary key.
///
/// Pre-seeds 1 000 memories, then cycles through their IDs to avoid
/// cache effects. Throughput is expressed per element (one fetch = one element).
fn bench_memory_get(c: &mut Criterion) {
    let storage = Storage::open_in_memory().unwrap();

    // Create some memories first
    let mut ids = Vec::new();
    for i in 0..1000 {
        let idx = i;
        let memory = storage
            .with_transaction(|conn| {
                let input = CreateMemoryInput {
                    content: format!("Memory content number {}", idx),
                    memory_type: MemoryType::Note,
                    tags: vec![format!("tag{}", idx % 10)],
                    metadata: Default::default(),
                    importance: Some(0.5),
                    defer_embedding: true,
                    scope: MemoryScope::Global,
                    ttl_seconds: None,
                    dedup_mode: DedupMode::Allow,
                    dedup_threshold: None,
                    workspace: Some("default".to_string()),
                    tier: MemoryTier::Permanent,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                };
                create_memory(conn, &input)
            })
            .unwrap();
        ids.push(memory.id);
    }

    let mut group = c.benchmark_group("memory_get");
    group.throughput(Throughput::Elements(1));

    group.bench_function("by_id", |b| {
        let mut i = 0;
        b.iter(|| {
            let id = ids[i % ids.len()];
            i += 1;
            storage
                .with_connection(|conn| get_memory(conn, black_box(id)))
                .unwrap()
        })
    });

    group.finish();
}

/// Benchmark paginated list latency at three page sizes (10, 50, 100).
///
/// Two variants per page size:
/// - **`limit/{n}`** — no filter, full-scan with `LIMIT n`.
/// - **`with_tag_filter/{n}`** — filtered by a single tag (`tag5`), exercising
///   the FTS-backed tag index.
///
/// Pre-seeds 1 000 memories with 10 tag groups and 5 category groups so the
/// tag filter returns roughly 100 results before pagination.
fn bench_memory_list(c: &mut Criterion) {
    let storage = Storage::open_in_memory().unwrap();

    // Create memories with various tags
    for i in 0..1000 {
        storage
            .with_transaction(|conn| {
                let input = CreateMemoryInput {
                    content: format!(
                        "Memory content number {} with some longer text to simulate real usage",
                        i
                    ),
                    memory_type: if i % 3 == 0 {
                        MemoryType::Todo
                    } else {
                        MemoryType::Note
                    },
                    tags: vec![format!("tag{}", i % 10), format!("category{}", i % 5)],
                    metadata: Default::default(),
                    importance: Some((i % 10) as f32 / 10.0),
                    defer_embedding: true,
                    scope: MemoryScope::Global,
                    ttl_seconds: None,
                    dedup_mode: DedupMode::Allow,
                    dedup_threshold: None,
                    workspace: Some("default".to_string()),
                    tier: MemoryTier::Permanent,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                };
                create_memory(conn, &input)
            })
            .unwrap();
    }

    let mut group = c.benchmark_group("memory_list");

    for limit in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*limit as u64));

        group.bench_with_input(BenchmarkId::new("limit", limit), limit, |b, &limit| {
            b.iter(|| {
                let options = ListOptions {
                    limit: Some(limit),
                    ..Default::default()
                };
                storage
                    .with_connection(|conn| list_memories(conn, black_box(&options)))
                    .unwrap()
            })
        });

        group.bench_with_input(
            BenchmarkId::new("with_tag_filter", limit),
            limit,
            |b, &limit| {
                b.iter(|| {
                    let options = ListOptions {
                        limit: Some(limit),
                        tags: Some(vec!["tag5".to_string()]),
                        ..Default::default()
                    };
                    storage
                        .with_connection(|conn| list_memories(conn, black_box(&options)))
                        .unwrap()
                })
            },
        );
    }

    group.finish();
}

/// Benchmark knowledge-graph edge operations: create and read.
///
/// Pre-seeds 100 memories and 50 `RelatedTo` edges. Then measures:
/// - **`crossref/create`** — inserting a new `References` edge between two
///   existing memories (write path, avoids duplicate IDs).
/// - **`crossref/get_related`** — fetching all neighbors of a node via
///   `get_related` (read path, exercises the cross-reference index).
fn bench_crossref_operations(c: &mut Criterion) {
    let storage = Storage::open_in_memory().unwrap();

    // Create memories
    let mut ids = Vec::new();
    for i in 0..100 {
        let idx = i;
        let memory = storage
            .with_transaction(|conn| {
                let input = CreateMemoryInput {
                    content: format!("Memory {}", idx),
                    memory_type: MemoryType::Note,
                    tags: vec![],
                    metadata: Default::default(),
                    importance: None,
                    defer_embedding: true,
                    scope: MemoryScope::Global,
                    ttl_seconds: None,
                    dedup_mode: DedupMode::Allow,
                    dedup_threshold: None,
                    workspace: Some("default".to_string()),
                    tier: MemoryTier::Permanent,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                };
                create_memory(conn, &input)
            })
            .unwrap();
        ids.push(memory.id);
    }

    // Create some cross-references
    for i in 0..50 {
        storage
            .with_transaction(|conn| {
                let input = CreateCrossRefInput {
                    from_id: ids[i],
                    to_id: ids[i + 1],
                    edge_type: EdgeType::RelatedTo,
                    strength: None,
                    source_context: None,
                    pinned: false,
                };
                create_crossref(conn, &input)
            })
            .unwrap();
    }

    let mut group = c.benchmark_group("crossref");

    group.bench_function("create", |b| {
        let mut i = 60;
        b.iter(|| {
            let from = ids[i % 40];
            let to = ids[(i + 50) % 100];
            i += 1;

            storage
                .with_transaction(|conn| {
                    let input = CreateCrossRefInput {
                        from_id: from,
                        to_id: to,
                        edge_type: EdgeType::References,
                        strength: None,
                        source_context: None,
                        pinned: false,
                    };
                    create_crossref(conn, black_box(&input))
                })
                .unwrap()
        })
    });

    group.bench_function("get_related", |b| {
        let mut i = 0;
        b.iter(|| {
            let id = ids[i % 50];
            i += 1;
            storage
                .with_connection(|conn| get_related(conn, black_box(id)))
                .unwrap()
        })
    });

    group.finish();
}

/// Benchmark the `get_stats` aggregate query over 500 memories.
///
/// Stats are used by the `memory_stats` MCP tool and the CLI. The query
/// aggregates counts, tag cardinality, and storage size — a good proxy for
/// overall metadata index health.
fn bench_stats(c: &mut Criterion) {
    let storage = Storage::open_in_memory().unwrap();

    // Populate with data
    for i in 0..500 {
        let idx = i;
        storage
            .with_transaction(|conn| {
                let input = CreateMemoryInput {
                    content: format!("Memory {}", idx),
                    memory_type: MemoryType::Note,
                    tags: vec![format!("tag{}", idx % 20)],
                    metadata: Default::default(),
                    importance: None,
                    defer_embedding: true,
                    scope: MemoryScope::Global,
                    ttl_seconds: None,
                    dedup_mode: DedupMode::Allow,
                    dedup_threshold: None,
                    workspace: Some("default".to_string()),
                    tier: MemoryTier::Permanent,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                };
                create_memory(conn, &input)
            })
            .unwrap();
    }

    c.bench_function("get_stats", |b| {
        b.iter(|| storage.with_connection(get_stats).unwrap())
    });
}

// ---------------------------------------------------------------------------
// Storage-mode separated groups (Q7)
// ---------------------------------------------------------------------------

/// Storage backends measured separately. The label is the Criterion id.
#[derive(Clone, Copy)]
enum BenchMode {
    InMemory,
    DiskWal,
    DiskCloudSafe,
}

impl BenchMode {
    const ALL: [BenchMode; 3] = [
        BenchMode::InMemory,
        BenchMode::DiskWal,
        BenchMode::DiskCloudSafe,
    ];

    fn label(self) -> &'static str {
        match self {
            BenchMode::InMemory => "in_memory",
            BenchMode::DiskWal => "disk_wal",
            BenchMode::DiskCloudSafe => "disk_cloud_safe",
        }
    }
}

/// A storage handle plus the temp directory that owns its files (if any).
struct ModeStorage {
    storage: Storage,
    _dir: Option<TempDir>,
}

fn storage_config(path: &std::path::Path, mode: StorageMode) -> StorageConfig {
    StorageConfig {
        db_path: path.to_string_lossy().to_string(),
        storage_mode: mode,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    }
}

fn open_mode(mode: BenchMode) -> ModeStorage {
    match mode {
        BenchMode::InMemory => ModeStorage {
            storage: Storage::open_in_memory().expect("in-memory storage"),
            _dir: None,
        },
        BenchMode::DiskWal | BenchMode::DiskCloudSafe => {
            let dir = tempdir().expect("bench tempdir");
            let storage_mode = if matches!(mode, BenchMode::DiskWal) {
                StorageMode::Local
            } else {
                StorageMode::CloudSafe
            };
            let storage = Storage::open(storage_config(
                &dir.path().join("bench.sqlite"),
                storage_mode,
            ))
            .expect("disk storage");
            ModeStorage {
                storage,
                _dir: Some(dir),
            }
        }
    }
}

fn bench_input(content: String) -> CreateMemoryInput {
    CreateMemoryInput {
        content,
        memory_type: MemoryType::Note,
        tags: vec!["benchmark".to_string()],
        importance: Some(0.5),
        defer_embedding: true,
        workspace: Some("default".to_string()),
        ..Default::default()
    }
}

/// Seed `count` memories in one transaction and return their ids.
fn seed_storage(storage: &Storage, count: usize) -> Vec<MemoryId> {
    storage
        .with_transaction(|conn| {
            let mut ids = Vec::with_capacity(count);
            for i in 0..count {
                let memory = create_memory(
                    conn,
                    &bench_input(format!(
                        "Storage mode benchmark memory {i} about authentication and caching"
                    )),
                )?;
                ids.push(memory.id);
            }
            Ok(ids)
        })
        .expect("seed storage")
}

/// Rows written to one database before `storage_modes/create` starts a fresh one.
const CREATE_CHUNK_ROWS: u64 = 1000;

/// `create_memory` per mode. Disk modes pay a commit (WAL: NORMAL sync,
/// cloud-safe: FULL sync) on every iteration, which is the point.
fn bench_storage_modes_create(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_modes/create");
    group.throughput(Throughput::Elements(1));
    for mode in BenchMode::ALL {
        group.bench_function(mode.label(), |b| {
            // A fresh database per chunk keeps the file (and so the cost of each
            // commit) bounded no matter how many iterations Criterion asks for;
            // opening it is outside the timed section.
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                let mut remaining = iters;
                while remaining > 0 {
                    let chunk = remaining.min(CREATE_CHUNK_ROWS);
                    let handle = open_mode(mode);
                    let started = Instant::now();
                    for i in 0..chunk {
                        handle
                            .storage
                            .with_transaction(|conn| {
                                create_memory(
                                    conn,
                                    &bench_input(format!("Mode create benchmark {i}")),
                                )
                            })
                            .expect("create");
                    }
                    total += started.elapsed();
                    remaining -= chunk;
                }
                total
            })
        });
    }
    group.finish();
}

/// `get_memory` by id per mode over 1 000 seeded rows.
fn bench_storage_modes_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_modes/get");
    group.throughput(Throughput::Elements(1));
    for mode in BenchMode::ALL {
        let handle = open_mode(mode);
        let ids = seed_storage(&handle.storage, 1000);
        let mut i = 0_usize;
        group.bench_function(mode.label(), |b| {
            b.iter(|| {
                let id = ids[i % ids.len()];
                i += 1;
                handle
                    .storage
                    .with_connection(|conn| get_memory(conn, black_box(id)))
                    .expect("get")
            })
        });
    }
    group.finish();
}

/// FTS5/BM25 search per mode over 1 000 seeded rows.
fn bench_storage_modes_search(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_modes/search");
    group.throughput(Throughput::Elements(1));
    for mode in BenchMode::ALL {
        let handle = open_mode(mode);
        seed_storage(&handle.storage, 1000);
        group.bench_function(mode.label(), |b| {
            b.iter(|| {
                handle
                    .storage
                    .with_connection(|conn| {
                        bm25_search(conn, black_box("authentication caching"), 10, false)
                    })
                    .expect("search")
            })
        });
    }
    group.finish();
}

const CONCURRENT_OPS_PER_THREAD: usize = 200;
const WRITER_OPS: usize = 20;

/// Run one batch: `readers` threads each doing `CONCURRENT_OPS_PER_THREAD`
/// `get_memory` calls on the shared pool, plus an optional writer thread doing
/// `WRITER_OPS` creates. Any storage error panics the batch, so a contention
/// failure fails the benchmark instead of being averaged away.
fn run_concurrent_batch(
    pool: &Arc<StoragePool>,
    ids: &Arc<Vec<MemoryId>>,
    readers: usize,
    writer: bool,
) {
    let mut handles = Vec::new();
    for t in 0..readers {
        let pool = Arc::clone(pool);
        let ids = Arc::clone(ids);
        handles.push(thread::spawn(move || {
            for i in 0..CONCURRENT_OPS_PER_THREAD {
                let id = ids[(t * 31 + i) % ids.len()];
                pool.with_connection(|conn| get_memory(conn, id))
                    .expect("concurrent read");
            }
        }));
    }
    if writer {
        let pool = Arc::clone(pool);
        handles.push(thread::spawn(move || {
            for i in 0..WRITER_OPS {
                pool.with_connection(|conn| {
                    create_memory(conn, &bench_input(format!("concurrent writer {i}")))
                })
                .expect("concurrent write");
            }
        }));
    }
    for handle in handles {
        handle.join().expect("bench thread must not panic");
    }
}

/// WAL concurrency: reader scaling and readers with one writer.
fn bench_storage_concurrency(c: &mut Criterion) {
    let dir = tempdir().expect("bench tempdir");
    let config = storage_config(&dir.path().join("concurrent.sqlite"), StorageMode::Local);
    let pool = Arc::new(StoragePool::new(config, 8).expect("storage pool"));
    let ids = Arc::new(
        pool.with_connection(|conn| {
            let mut ids = Vec::new();
            for i in 0..1000 {
                ids.push(create_memory(conn, &bench_input(format!("Concurrent seed {i}")))?.id);
            }
            Ok(ids)
        })
        .expect("seed pool"),
    );

    let mut group = c.benchmark_group("storage_concurrency");
    group.sample_size(20);
    for readers in [1_usize, 4, 8] {
        group.throughput(Throughput::Elements(
            (readers * CONCURRENT_OPS_PER_THREAD) as u64,
        ));
        group.bench_function(format!("readers_{readers}"), |b| {
            b.iter(|| run_concurrent_batch(&pool, &ids, readers, false))
        });
        group.bench_function(format!("readers_{readers}_one_writer"), |b| {
            b.iter(|| run_concurrent_batch(&pool, &ids, readers, true))
        });
    }
    group.finish();
}

fn percentile(sorted: &[Duration], pct: f64) -> Duration {
    let index = ((sorted.len() as f64 * pct / 100.0).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len() - 1);
    sorted[index]
}

/// Opt-in percentile report (`ENGRAM_BENCH_PERCENTILES=1`). Prints one
/// `percentile_report {json}` line per mode and operation after a warmup of
/// `ENGRAM_BENCH_PERCENTILE_WARMUP` (default 200) discarded iterations and
/// `ENGRAM_BENCH_PERCENTILE_SAMPLES` (default 2000) timed iterations. Without the
/// env var this registers nothing (no placeholder benchmark is emitted).
fn bench_latency_percentiles(_c: &mut Criterion) {
    if std::env::var("ENGRAM_BENCH_PERCENTILES").ok().as_deref() != Some("1") {
        return;
    }
    let env_usize = |name: &str, default: usize| {
        std::env::var(name)
            .ok()
            .and_then(|raw| raw.parse().ok())
            .unwrap_or(default)
    };
    let warmup = env_usize("ENGRAM_BENCH_PERCENTILE_WARMUP", 200);
    let samples = env_usize("ENGRAM_BENCH_PERCENTILE_SAMPLES", 2000).max(100);

    for mode in BenchMode::ALL {
        let handle = open_mode(mode);
        let ids = seed_storage(&handle.storage, 1000);
        for (operation, run) in [("create", 0_u8), ("get", 1_u8), ("search", 2_u8)] {
            let mut timings = Vec::with_capacity(samples);
            for i in 0..(warmup + samples) {
                let started = Instant::now();
                match run {
                    0 => {
                        handle
                            .storage
                            .with_transaction(|conn| {
                                create_memory(conn, &bench_input(format!("percentile create {i}")))
                            })
                            .expect("create");
                    }
                    1 => {
                        let id = ids[i % ids.len()];
                        handle
                            .storage
                            .with_connection(|conn| get_memory(conn, id))
                            .expect("get");
                    }
                    _ => {
                        handle
                            .storage
                            .with_connection(|conn| {
                                bm25_search(conn, "authentication caching", 10, false)
                            })
                            .expect("search");
                    }
                }
                let elapsed = started.elapsed();
                if i >= warmup {
                    timings.push(elapsed);
                }
            }
            timings.sort_unstable();
            println!(
                "percentile_report {}",
                serde_json::json!({
                    "bench": "memory_ops/latency_percentiles",
                    "mode": mode.label(),
                    "operation": operation,
                    "warmup_iterations": warmup,
                    "samples": samples,
                    "p50_us": percentile(&timings, 50.0).as_micros(),
                    "p95_us": percentile(&timings, 95.0).as_micros(),
                    "p99_us": percentile(&timings, 99.0).as_micros(),
                    "max_us": timings.last().map(Duration::as_micros),
                })
            );
        }
    }
}

criterion_group!(
    benches,
    bench_memory_create,
    bench_memory_get,
    bench_memory_list,
    bench_crossref_operations,
    bench_stats,
    bench_storage_modes_create,
    bench_storage_modes_get,
    bench_storage_modes_search,
    bench_storage_concurrency,
    bench_latency_percentiles,
);

criterion_main!(benches);
