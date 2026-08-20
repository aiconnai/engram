//! Semantic Query Cache — RML-1229
//!
//! Caches search results and matches queries by embedding similarity rather
//! than exact string match. Complements `result_cache.rs` (which caches by
//! exact query hash). A new query is a cache hit when its embedding is
//! sufficiently close (cosine similarity ≥ threshold) to a previously cached
//! query.

use dashmap::DashMap;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the semantic query cache.
#[derive(Debug, Clone)]
pub struct SemanticCacheConfig {
    /// Minimum cosine similarity to consider a cache hit (default 0.92).
    pub similarity_threshold: f32,
    /// Maximum number of cached entries (default 1000).
    pub max_entries: usize,
    /// Default TTL in seconds (default 300 = 5 minutes).
    pub default_ttl_secs: u64,
}

impl Default for SemanticCacheConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.92,
            max_entries: 1000,
            default_ttl_secs: 300,
        }
    }
}

// ---------------------------------------------------------------------------
// Internal entry
// ---------------------------------------------------------------------------

struct CacheEntry {
    query_embedding: Vec<f32>,
    /// Stored for observability / future logging; not read in hot paths.
    #[allow(dead_code)]
    query_text: String,
    /// Model name used for cache isolation; not read in hot paths.
    #[allow(dead_code)]
    model_name: String,
    results: Value,
    created_at: Instant,
    ttl: Duration,
    hit_count: u64,
}

impl CacheEntry {
    fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.ttl
    }
}

// ---------------------------------------------------------------------------
// Public stats
// ---------------------------------------------------------------------------

/// Snapshot of cache statistics.
#[derive(Debug, Clone, Default)]
pub struct SemanticCacheStats {
    pub entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub invalidations: u64,
}

// ---------------------------------------------------------------------------
// SemanticCache
// ---------------------------------------------------------------------------

/// A cache that matches queries by embedding similarity rather than exact
/// string match.
pub struct SemanticCache {
    /// Live entries. Key = `embedding_hash(embedding)`.
    entries: DashMap<u64, CacheEntry>,
    config: SemanticCacheConfig,
    model_name: String,
    hits: AtomicU64,
    misses: AtomicU64,
    evictions: AtomicU64,
    invalidations: AtomicU64,
}

impl SemanticCache {
    /// Create a new cache with the supplied configuration.
    pub fn new(config: SemanticCacheConfig, model_name: String) -> Self {
        Self {
            entries: DashMap::new(),
            config,
            model_name,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            evictions: AtomicU64::new(0),
            invalidations: AtomicU64::new(0),
        }
    }

    // -----------------------------------------------------------------------
    // Public API
    // -----------------------------------------------------------------------

    /// Try to find a cached result for `query_embedding`.
    ///
    /// Performs a linear scan over all entries comparing cosine similarity.
    /// Returns the result of the best-matching non-expired entry whose
    /// similarity is at or above the configured threshold.
    pub fn get(&self, query_embedding: &[f32]) -> Option<Value> {
        let threshold = self.config.similarity_threshold;

        let mut best_similarity = -1.0_f32;
        let mut best_result: Option<Value> = None;
        let mut best_key: Option<u64> = None;

        for entry_ref in self.entries.iter() {
            if entry_ref.is_expired() {
                continue;
            }

            if entry_ref.query_embedding.len() != query_embedding.len() {
                continue;
            }

            let sim = cosine_similarity(query_embedding, &entry_ref.query_embedding);
            if sim >= threshold && sim > best_similarity {
                best_similarity = sim;
                best_result = Some(entry_ref.results.clone());
                best_key = Some(*entry_ref.key());
            }
        }

        if let Some(key) = best_key {
            if let Some(mut entry) = self.entries.get_mut(&key) {
                entry.hit_count = entry.hit_count.saturating_add(1);
            }
            self.hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
        }

        best_result
    }

    /// Store `results` under `query_embedding`.
    ///
    /// If the cache is at capacity, the oldest entry (by `created_at`) is
    /// evicted first.
    pub fn put(&self, query_embedding: Vec<f32>, query_text: String, results: Value) {
        if self.entries.len() >= self.config.max_entries {
            self.evict_oldest();
        }

        let key = embedding_hash(&query_embedding, &self.model_name);
        let entry = CacheEntry {
            query_embedding,
            query_text,
            model_name: self.model_name.clone(),
            results,
            created_at: Instant::now(),
            ttl: Duration::from_secs(self.config.default_ttl_secs),
            hit_count: 0,
        };

        self.entries.insert(key, entry);
    }

    /// Remove all entries whose result JSON contains `memory_id`.
    ///
    /// The JSON is checked for any occurrence of the integer value at any
    /// position in the document.
    pub fn invalidate_memory(&self, memory_id: i64) {
        let target = Value::Number(memory_id.into());

        self.entries.retain(|_, entry| {
            let contains = json_contains(&entry.results, &target);
            if contains {
                self.invalidations.fetch_add(1, Ordering::Relaxed);
            }
            !contains
        });
    }

    /// Remove all cached entries.
    pub fn clear(&self) {
        self.entries.clear();
    }

    /// Return a snapshot of cache statistics.
    pub fn stats(&self) -> SemanticCacheStats {
        SemanticCacheStats {
            entries: self.entries.len(),
            hits: self.hits.load(Ordering::Relaxed),
            misses: self.misses.load(Ordering::Relaxed),
            evictions: self.evictions.load(Ordering::Relaxed),
            invalidations: self.invalidations.load(Ordering::Relaxed),
        }
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    /// Evict the entry with the smallest `created_at`.
    fn evict_oldest(&self) {
        // Collect (key, created_at) pairs to find the oldest without holding
        // any individual shard lock longer than necessary.
        let mut oldest_key: Option<u64> = None;
        let mut oldest_time: Option<Instant> = None;

        for entry_ref in self.entries.iter() {
            let t = entry_ref.created_at;
            match oldest_time {
                None => {
                    oldest_time = Some(t);
                    oldest_key = Some(*entry_ref.key());
                }
                Some(ot) if t < ot => {
                    oldest_time = Some(t);
                    oldest_key = Some(*entry_ref.key());
                }
                _ => {}
            }
        }

        if let Some(key) = oldest_key {
            self.entries.remove(&key);
            self.evictions.fetch_add(1, Ordering::Relaxed);
        }
    }
}

// ---------------------------------------------------------------------------
// Free-standing helpers (pub for testing / benchmarking)
// ---------------------------------------------------------------------------

pub use super::vector::cosine_similarity;

/// Derive a `u64` bucket key from the entire embedding vector.
///
/// This is used as the DashMap key for `O(1)` insertion. `get` always does a
/// full linear scan for semantic matching.
pub fn embedding_hash(embedding: &[f32], model_name: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325; // FNV-1a offset basis
                                            // Mix model name into hash
    for byte in model_name.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    for &f in embedding {
        let canonical_f = if f == 0.0 { 0.0f32 } else { f };
        let bytes = canonical_f.to_le_bytes();
        for byte in bytes {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3); // FNV-1a prime
        }
    }
    hash
}

/// Recursively check whether `json` contains `target` anywhere in its tree.
fn json_contains(json: &Value, target: &Value) -> bool {
    if json == target {
        return true;
    }
    match json {
        Value::Array(arr) => arr.iter().any(|v| json_contains(v, target)),
        Value::Object(map) => map.values().any(|v| json_contains(v, target)),
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::thread;
    use std::time::Duration;

    fn default_cache() -> SemanticCache {
        SemanticCache::new(SemanticCacheConfig::default(), "test-model".to_string())
    }

    fn unit_vec(dim: usize, hot: usize) -> Vec<f32> {
        let mut v = vec![0.0_f32; dim];
        v[hot] = 1.0;
        v
    }

    // -----------------------------------------------------------------------

    #[test]
    fn test_cache_hit() {
        let cache = default_cache();
        let emb = unit_vec(4, 0);
        let results = json!({"memories": [{"id": 42}]});

        cache.put(emb.clone(), "query".into(), results.clone());
        let got = cache.get(&emb);
        assert_eq!(got, Some(results));
    }

    #[test]
    fn test_cache_miss_below_threshold() {
        let cache = default_cache();
        // Perpendicular vectors: similarity = 0.0
        let emb_stored = unit_vec(4, 0);
        let emb_query = unit_vec(4, 1);

        cache.put(emb_stored, "query A".into(), json!({"memories": []}));
        let got = cache.get(&emb_query);
        assert!(got.is_none(), "perpendicular vectors must not hit");
    }

    #[test]
    fn test_cache_hit_similar() {
        // Slightly perturb the stored embedding; similarity must remain ≥ 0.92.
        let cache = SemanticCache::new(
            SemanticCacheConfig {
                similarity_threshold: 0.92,
                ..Default::default()
            },
            "test-model".to_string(),
        );

        let emb_stored = vec![1.0_f32, 0.0, 0.0, 0.0];
        let results = json!({"memories": [{"id": 7}]});
        cache.put(emb_stored, "original".into(), results.clone());

        // Small perturbation: similarity ≈ 0.9998
        let emb_similar = vec![1.0_f32, 0.01, 0.0, 0.0];
        let got = cache.get(&emb_similar);
        assert!(got.is_some(), "similar embedding must be a cache hit");
    }

    #[test]
    fn test_ttl_expiration() {
        let cache = SemanticCache::new(
            SemanticCacheConfig {
                default_ttl_secs: 0, // expires immediately
                ..Default::default()
            },
            "test-model".to_string(),
        );

        let emb = unit_vec(4, 2);
        cache.put(emb.clone(), "q".into(), json!({"ok": true}));

        // Even 1 ms is enough for `created_at.elapsed() > Duration::ZERO`.
        thread::sleep(Duration::from_millis(5));
        let got = cache.get(&emb);
        assert!(got.is_none(), "entry should have expired");
    }

    #[test]
    fn test_invalidate_memory() {
        let cache = default_cache();
        let emb = unit_vec(4, 0);
        let results = json!([{"id": 99, "content": "hello"}]);

        cache.put(emb.clone(), "q".into(), results);

        // Confirm it's cached.
        assert!(cache.get(&emb).is_some());

        // Invalidate by memory id 99.
        cache.invalidate_memory(99);
        assert!(
            cache.get(&emb).is_none(),
            "entry containing id 99 must be removed"
        );

        let stats = cache.stats();
        assert_eq!(stats.invalidations, 1);
    }

    #[test]
    fn test_invalidate_memory_does_not_remove_unrelated() {
        let cache = default_cache();
        let emb1 = unit_vec(4, 0);
        let emb2 = unit_vec(4, 1);

        cache.put(emb1.clone(), "q1".into(), json!([{"id": 1}]));
        cache.put(emb2.clone(), "q2".into(), json!([{"id": 2}]));

        cache.invalidate_memory(1);

        assert!(cache.get(&emb1).is_none());
        assert!(cache.get(&emb2).is_some());
    }

    #[test]
    fn test_clear() {
        let cache = default_cache();
        cache.put(unit_vec(4, 0), "a".into(), json!(1));
        cache.put(unit_vec(4, 1), "b".into(), json!(2));

        cache.clear();
        assert_eq!(cache.stats().entries, 0);
    }

    #[test]
    fn test_stats() {
        let cache = default_cache();
        let emb = unit_vec(4, 3);

        // Miss
        cache.get(&emb);

        // Put + hit twice
        cache.put(emb.clone(), "q".into(), json!({"x": 1}));
        cache.get(&emb);
        cache.get(&emb);

        let stats = cache.stats();
        assert_eq!(stats.hits, 2, "expected 2 hits");
        assert_eq!(stats.misses, 1, "expected 1 miss");
        assert_eq!(stats.entries, 1);
    }

    #[test]
    fn test_capacity_eviction() {
        let cache = SemanticCache::new(
            SemanticCacheConfig {
                max_entries: 2,
                ..Default::default()
            },
            "test-model".to_string(),
        );

        let emb0 = unit_vec(4, 0);
        let emb1 = unit_vec(4, 1);
        let emb2 = unit_vec(4, 2);

        cache.put(emb0.clone(), "first".into(), json!("first"));
        // Small sleep to ensure distinct Instants (monotonic clock granularity).
        thread::sleep(Duration::from_millis(1));
        cache.put(emb1.clone(), "second".into(), json!("second"));
        thread::sleep(Duration::from_millis(1));
        // Third insert must evict the oldest (emb0 / "first").
        cache.put(emb2.clone(), "third".into(), json!("third"));

        assert_eq!(cache.stats().entries, 2);
        assert_eq!(cache.stats().evictions, 1);

        // "first" should be gone, "second" and "third" should survive.
        assert!(
            cache.get(&emb0).is_none(),
            "oldest entry must have been evicted"
        );
        assert!(cache.get(&emb1).is_some());
        assert!(cache.get(&emb2).is_some());
    }

    // -----------------------------------------------------------------------
    // cosine_similarity corner cases
    // -----------------------------------------------------------------------

    #[test]
    fn test_cosine_similarity_identical() {
        let v = vec![1.0_f32, 2.0, 3.0];
        let sim = cosine_similarity(&v, &v);
        assert!((sim - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0_f32, 0.0];
        let b = vec![0.0_f32, 1.0];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        assert_eq!(cosine_similarity(&[], &[]), 0.0);
    }

    #[test]
    fn test_cosine_similarity_length_mismatch() {
        let a = vec![1.0_f32, 0.0];
        let b = vec![1.0_f32];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }
}
