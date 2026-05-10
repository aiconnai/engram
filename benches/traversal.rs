//! Performance benchmarks for multi-hop graph traversal (BFS).
//!
//! Builds a balanced tree in SQLite (branching factor × depth) and measures
//! `get_related_multi_hop` — the BFS used by the `memory_traverse` MCP tool.
//! The tree structure gives predictable node counts per depth level, making
//! regressions easy to attribute to a specific traversal layer.
//!
//! Run with: `cargo bench --bench traversal`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engram::storage::graph_queries::{get_related_multi_hop, TraversalOptions};
use engram::storage::queries::{create_crossref, create_memory};
use engram::storage::Storage;
use engram::types::{
    CreateCrossRefInput, CreateMemoryInput, DedupMode, EdgeType, MemoryId, MemoryScope, MemoryTier,
    MemoryType,
};

/// Insert a single memory into `storage`, returning its ID.
fn create_test_memory(storage: &Storage, i: usize) -> MemoryId {
    storage
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
        .unwrap()
        .id
}

/// Build a balanced tree of memories linked by `RelatedTo` edges.
///
/// Returns the root memory ID. Total nodes = Σ(branching_factor^d) for
/// d = 0..max_depth. With branching_factor=5, depth=3: 1 + 5 + 25 + 125 = 156 nodes.
fn create_graph(storage: &Storage, branching_factor: usize, max_depth: usize) -> MemoryId {
    let root_id = create_test_memory(storage, 0);
    let mut current_level = vec![root_id];
    let mut memory_counter = 1;

    for _depth in 0..max_depth {
        let mut next_level = Vec::new();
        for &parent_id in &current_level {
            for _ in 0..branching_factor {
                let child_id = create_test_memory(storage, memory_counter);
                memory_counter += 1;

                storage
                    .with_transaction(|conn| {
                        let input = CreateCrossRefInput {
                            from_id: parent_id,
                            to_id: child_id,
                            edge_type: EdgeType::RelatedTo,
                            strength: None,
                            source_context: None,
                            pinned: false,
                        };
                        create_crossref(conn, &input)
                    })
                    .unwrap();

                next_level.push(child_id);
            }
        }
        current_level = next_level;
    }
    root_id
}

/// Benchmark BFS traversal to depth 3 on a 156-node balanced tree.
///
/// The tree has branching factor 5 and depth 3. BFS visits all reachable
/// nodes via the `crossrefs` table, exercising the recursive query path
/// in `get_related_multi_hop`. Entity inclusion is disabled to isolate
/// pure graph-walk cost from the entity join.
fn bench_traversal(c: &mut Criterion) {
    let storage = Storage::open_in_memory().unwrap();
    // Create a graph: depth 3, branching factor 5 (~156 nodes)
    let root_id = create_graph(&storage, 5, 3);

    let mut group = c.benchmark_group("traversal");

    group.bench_function("bfs_depth_3", |b| {
        b.iter(|| {
            let options = TraversalOptions {
                depth: 3,
                include_entities: false,
                ..Default::default()
            };
            storage
                .with_connection(|conn| get_related_multi_hop(conn, black_box(root_id), &options))
                .unwrap()
        })
    });

    group.finish();
}

criterion_group!(benches, bench_traversal);
criterion_main!(benches);
