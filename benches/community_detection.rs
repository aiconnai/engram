//! Performance benchmarks for community detection on the knowledge graph.
//!
//! Generates a synthetic graph with clustered structure (50-node clusters
//! with dense intra-cluster edges and sparse random inter-cluster links)
//! and runs the Louvain-style community detection algorithm.
//!
//! Run with: `cargo bench --bench community_detection`

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use engram::graph::{GraphEdge, GraphNode, KnowledgeGraph};
use engram::types::MemoryId;
use rand::prelude::*;

/// Build a synthetic [`KnowledgeGraph`] with clustered topology.
///
/// Nodes are assigned to clusters of 50. Each node gets 2–5 intra-cluster
/// edges, plus a random cross-cluster edge with probability `edge_density`.
/// Tags are drawn from a 9-element pool (20–50 per node) to create
/// realistic attribute overlap between clusters.
fn generate_graph(node_count: usize, edge_density: f32) -> KnowledgeGraph {
    let mut rng = StdRng::seed_from_u64(42);
    let mut nodes = Vec::with_capacity(node_count);

    let memory_types = [
        "note",
        "todo",
        "issue",
        "decision",
        "preference",
        "learning",
    ];
    let tag_pool = [
        "rust",
        "python",
        "ai",
        "database",
        "web",
        "api",
        "cli",
        "graph",
        "performance",
    ];

    for i in 0..node_count {
        let memory_type = memory_types.choose(&mut rng).unwrap().to_string();
        let num_tags = rng.gen_range(20..50);
        let mut tags = Vec::new();
        for _ in 0..num_tags {
            tags.push(tag_pool.choose(&mut rng).unwrap().to_string());
        }

        nodes.push(GraphNode {
            id: i as MemoryId,
            label: format!("Node {}", i),
            memory_type,
            importance: rng.gen(),
            tags,
        });
    }

    let mut edges = Vec::new();
    let edge_types = ["related_to", "depends_on", "part_of", "contradicts"];

    // Ensure some structure by creating clusters
    let cluster_size = 50;
    for i in 0..node_count {
        // Connect to neighbors in same "cluster"
        let cluster_start = (i / cluster_size) * cluster_size;
        let num_edges = rng.gen_range(2..6);

        for _ in 0..num_edges {
            let target =
                rng.gen_range(cluster_start..(cluster_start + cluster_size).min(node_count));
            if target != i {
                edges.push(GraphEdge {
                    from: i as MemoryId,
                    to: target as MemoryId,
                    edge_type: edge_types.choose(&mut rng).unwrap().to_string(),
                    score: rng.gen(),
                    confidence: rng.gen(),
                });
            }
        }

        // Occasional random link
        if rng.gen::<f32>() < edge_density {
            let target = rng.gen_range(0..node_count);
            if target != i {
                edges.push(GraphEdge {
                    from: i as MemoryId,
                    to: target as MemoryId,
                    edge_type: edge_types.choose(&mut rng).unwrap().to_string(),
                    score: rng.gen(),
                    confidence: rng.gen(),
                });
            }
        }
    }

    KnowledgeGraph { nodes, edges }
}

/// Benchmark community detection on a 500-node clustered graph.
///
/// The graph has ~10 natural clusters of 50 nodes each, connected by
/// sparse random edges (5% density). `detect_communities(10)` requests
/// up to 10 communities. Sample size is reduced to 10 because the
/// algorithm is O(n² · iterations) and each iteration takes ~7 ms.
fn bench_detect_communities(c: &mut Criterion) {
    let graph = generate_graph(500, 0.05);

    let mut group = c.benchmark_group("community_detection");
    group.sample_size(10);
    group.bench_function("detect_communities_500_nodes", |b| {
        b.iter(|| graph.detect_communities(black_box(10)))
    });
    group.finish();
}

criterion_group!(benches, bench_detect_communities);
criterion_main!(benches);
