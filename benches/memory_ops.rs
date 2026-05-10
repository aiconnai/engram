//! Performance benchmarks for core memory operations (RML-902).
//!
//! Covers the hot paths that run on every memory interaction:
//! create, get, list, cross-reference management, and aggregate stats.
//! All benchmarks use an in-memory SQLite database to isolate CPU cost
//! from disk I/O.
//!
//! Run with: `cargo bench --bench memory_ops`
//!
//! ## Performance targets
//! | Operation                  | Target  |
//! |----------------------------|---------|
//! | `memory_create/no_embedding` | < 200 µs |
//! | `memory_get/by_id`           | < 100 µs |

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use engram::storage::queries::*;
use engram::storage::Storage;
use engram::types::*;

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
        let memory = storage
            .with_transaction(|conn| {
                let input = CreateMemoryInput {
                    content: format!("Memory content number {}", i),
                    memory_type: MemoryType::Note,
                    tags: vec![format!("tag{}", i % 10)],
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
        let memory = storage
            .with_transaction(|conn| {
                let input = CreateMemoryInput {
                    content: format!("Memory {}", i),
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
        storage
            .with_transaction(|conn| {
                let input = CreateMemoryInput {
                    content: format!("Memory {}", i),
                    memory_type: MemoryType::Note,
                    tags: vec![format!("tag{}", i % 20)],
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

criterion_group!(
    benches,
    bench_memory_create,
    bench_memory_get,
    bench_memory_list,
    bench_crossref_operations,
    bench_stats,
);

criterion_main!(benches);
