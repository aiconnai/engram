//! Performance benchmarks for MCP dispatch latency (v0.7.0 - T1)
//!
//! Measures end-to-end dispatch latency for common tool calls via the
//! HandlerContext and dispatch() function. This benchmark simulates real
//! MCP request handling without network overhead.
//!
//! Run with: cargo bench --bench mcp_dispatch

use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use parking_lot::Mutex;
use serde_json::json;

use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::handlers::{self, HandlerContext};
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::queries::*;
use engram::storage::Storage;
use engram::types::*;

// ---------------------------------------------------------------------------
// Benchmark Handler Setup
// ---------------------------------------------------------------------------

/// Build a [`HandlerContext`] wired to the given in-memory storage.
///
/// Uses the default TF-IDF embedder (no OpenAI calls), an empty fuzzy engine,
/// and default search/cache configs. Optional features (`meilisearch`,
/// `langfuse`) are conditionally compiled out.
fn create_benchmark_context(storage: Storage) -> HandlerContext {
    let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
    HandlerContext {
        storage,
        embedder: embedder.clone(),
        fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
        hnsw_index: Arc::new(parking_lot::RwLock::new(engram::search::HnswIndex::new(
            engram::search::HnswConfig::new(
                embedder.dimensions(),
                engram::search::VectorMetric::Cosine,
            ),
        ))),
        #[cfg(feature = "meilisearch")]
        meili: None,
        #[cfg(feature = "meilisearch")]
        meili_indexer: None,
        #[cfg(feature = "meilisearch")]
        meili_sync_interval: 300,
        #[cfg(feature = "langfuse")]
        langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
        progress_reporter: None,
        principal: None,
    }
}

/// Pre-populate `storage` with `count` synthetic memories.
///
/// Memories alternate between `Note` and `Todo` types with 10 tag groups and
/// 5 category groups, mimicking a realistic multi-type workspace. Embedding is
/// deferred so seeding cost is pure SQLite write throughput.
fn seed_memories(storage: &Storage, count: usize) {
    for i in 0..count {
        storage
            .with_transaction(|conn| {
                let input = CreateMemoryInput {
                    content: format!(
                        "Benchmark memory #{} - synthetic content for dispatch latency testing",
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
            .expect("seed memory");
    }
}

// ---------------------------------------------------------------------------
// Benchmark: memory_create (write path)
// ---------------------------------------------------------------------------

/// Benchmark end-to-end MCP dispatch latency for the `memory_create` tool.
///
/// Measures the full write path: JSON params → `dispatch()` → handler →
/// `create_memory` SQL insert. Embedding is skipped (no `embedding` field in
/// params), so this isolates MCP routing + serialisation + SQLite write cost.
fn bench_dispatch_memory_create(c: &mut Criterion) {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let ctx = create_benchmark_context(storage);

    let mut group = c.benchmark_group("mcp_dispatch_memory_create");
    group.throughput(Throughput::Elements(1));

    group.bench_function("memory_create", |b| {
        b.iter(|| {
            let params = black_box(json!({
                "content": "Dispatch benchmark memory",
                "type": "note",
                "tags": ["benchmark"],
                "workspace": "default",
            }));
            handlers::dispatch(&ctx, "memory_create", params)
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: memory_search (read + compute path)
// ---------------------------------------------------------------------------

/// Benchmark end-to-end MCP dispatch latency for the `memory_search` tool.
///
/// Pre-seeds 100 memories. Measures the full read + compute path:
/// JSON params → `dispatch()` → handler → TF-IDF embed query →
/// BM25 + vector hybrid search → RRF fusion → JSON response.
/// This is the most compute-intensive common tool call.
fn bench_dispatch_memory_search(c: &mut Criterion) {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    seed_memories(&storage, 100);

    let ctx = create_benchmark_context(storage);

    let mut group = c.benchmark_group("mcp_dispatch_memory_search");
    group.throughput(Throughput::Elements(1));

    group.bench_function("memory_search", |b| {
        b.iter(|| {
            let params = black_box(json!({
                "query": "benchmark memory",
                "limit": 10,
                "workspace": "default",
            }));
            handlers::dispatch(&ctx, "memory_search", params)
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: memory_list (read path)
// ---------------------------------------------------------------------------

/// Benchmark end-to-end MCP dispatch latency for the `memory_list` tool.
///
/// Pre-seeds 100 memories, then lists 20 per call. Exercises the read path
/// without embedding computation: JSON params → `dispatch()` → handler →
/// `list_memories` SQL query → JSON serialisation.
fn bench_dispatch_memory_list(c: &mut Criterion) {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    seed_memories(&storage, 100);

    let ctx = create_benchmark_context(storage);

    let mut group = c.benchmark_group("mcp_dispatch_memory_list");
    group.throughput(Throughput::Elements(1));

    group.bench_function("memory_list", |b| {
        b.iter(|| {
            let params = black_box(json!({
                "limit": 20,
                "workspace": "default",
            }));
            handlers::dispatch(&ctx, "memory_list", params)
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: memory_stats (metadata path)
// ---------------------------------------------------------------------------

/// Benchmark end-to-end MCP dispatch latency for the `memory_stats` tool.
///
/// Pre-seeds 100 memories. Stats aggregate counts, tag cardinality, and
/// storage size via a multi-aggregate SQL query. This is a pure metadata
/// read — no FTS5 or vector index involved — so it measures the cost of
/// SQLite aggregate scans through the MCP dispatch layer.
fn bench_dispatch_memory_stats(c: &mut Criterion) {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    seed_memories(&storage, 100);

    let ctx = create_benchmark_context(storage);

    let mut group = c.benchmark_group("mcp_dispatch_memory_stats");
    group.throughput(Throughput::Elements(1));

    group.bench_function("memory_stats", |b| {
        b.iter(|| {
            let params = black_box(json!({}));
            handlers::dispatch(&ctx, "memory_stats", params)
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Benchmark: unknown tool (error path)
// ---------------------------------------------------------------------------

/// Benchmark the fast-fail error path for an unrecognised tool name.
///
/// `dispatch()` should reject unknown tools in O(1) time via a match arm
/// before any I/O. This benchmark acts as a regression guard: if it regresses
/// significantly (e.g. past 1 µs), the dispatch table likely changed from a
/// static match to a dynamic lookup.
fn bench_dispatch_unknown_tool(c: &mut Criterion) {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let ctx = create_benchmark_context(storage);

    let mut group = c.benchmark_group("mcp_dispatch_error_path");
    group.throughput(Throughput::Elements(1));

    group.bench_function("unknown_tool", |b| {
        b.iter(|| {
            let params = black_box(json!({}));
            handlers::dispatch(&ctx, "unknown_tool_12345", params)
        })
    });

    group.finish();
}

// ---------------------------------------------------------------------------
// Criterion Setup
// ---------------------------------------------------------------------------

criterion_group!(
    benches,
    bench_dispatch_memory_create,
    bench_dispatch_memory_search,
    bench_dispatch_memory_list,
    bench_dispatch_memory_stats,
    bench_dispatch_unknown_tool,
);

criterion_main!(benches);
