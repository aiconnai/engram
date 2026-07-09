use super::*;
use crate::types::MemoryId;

fn make_node(id: MemoryId, memory_type: &str, tags: Vec<&str>) -> GraphNode {
    GraphNode {
        id,
        label: format!("Node {}", id),
        memory_type: memory_type.to_string(),
        importance: 0.5,
        tags: tags.into_iter().map(String::from).collect(),
    }
}

fn make_edge(from: MemoryId, to: MemoryId, edge_type: &str) -> GraphEdge {
    GraphEdge {
        from,
        to,
        edge_type: edge_type.to_string(),
        score: 0.8,
        confidence: 0.9,
    }
}

#[test]
fn test_truncate_label() {
    assert_eq!(types::truncate_label("short", 50), "short");
    assert_eq!(
        types::truncate_label("this is a very long label that should be truncated", 20),
        "this is a very lo..."
    );
}

#[test]
fn test_graph_stats() {
    let id1: MemoryId = 1;
    let id2: MemoryId = 2;
    let id3: MemoryId = 3;

    let graph = KnowledgeGraph {
        nodes: vec![
            make_node(id1, "note", vec!["rust"]),
            make_node(id2, "note", vec!["rust"]),
            make_node(id3, "todo", vec!["python"]),
        ],
        edges: vec![
            make_edge(id1, id2, "related_to"),
            make_edge(id2, id3, "depends_on"),
        ],
    };

    let stats = graph.stats();
    assert_eq!(stats.node_count, 3);
    assert_eq!(stats.edge_count, 2);
    assert_eq!(stats.nodes_by_type.get("note"), Some(&2));
    assert_eq!(stats.nodes_by_type.get("todo"), Some(&1));
    assert_eq!(stats.isolated_count, 0);
    assert_eq!(stats.component_count, 1);
}

#[test]
fn test_graph_filter() {
    let id1: MemoryId = 1;
    let id2: MemoryId = 2;
    let id3: MemoryId = 3;

    let graph = KnowledgeGraph {
        nodes: vec![
            make_node(id1, "note", vec!["rust"]),
            make_node(id2, "note", vec!["python"]),
            make_node(id3, "todo", vec!["rust"]),
        ],
        edges: vec![
            make_edge(id1, id2, "related_to"),
            make_edge(id2, id3, "depends_on"),
        ],
    };

    // Filter by type
    let filter = GraphFilter::new().with_types(vec!["note".to_string()]);
    let filtered = graph.filter(&filter);
    assert_eq!(filtered.nodes.len(), 2);
    assert_eq!(filtered.edges.len(), 1); // Only edge between notes

    // Filter by tag
    let filter = GraphFilter::new().with_tags(vec!["rust".to_string()]);
    let filtered = graph.filter(&filter);
    assert_eq!(filtered.nodes.len(), 2); // id1 and id3 have "rust"
}

#[test]
fn test_neighborhood() {
    let id1: MemoryId = 1;
    let id2: MemoryId = 2;
    let id3: MemoryId = 3;
    let id4: MemoryId = 4;

    let graph = KnowledgeGraph {
        nodes: vec![
            make_node(id1, "note", vec![]),
            make_node(id2, "note", vec![]),
            make_node(id3, "note", vec![]),
            make_node(id4, "note", vec![]),
        ],
        edges: vec![
            make_edge(id1, id2, "related_to"),
            make_edge(id2, id3, "related_to"),
            make_edge(id3, id4, "related_to"),
        ],
    };

    // Depth 1 from id1 should include id1, id2
    let subgraph = graph.neighborhood(id1, 1);
    assert_eq!(subgraph.nodes.len(), 2);

    // Depth 2 from id1 should include id1, id2, id3
    let subgraph = graph.neighborhood(id1, 2);
    assert_eq!(subgraph.nodes.len(), 3);
}

#[test]
fn test_to_dot() {
    let id1: MemoryId = 1;
    let id2: MemoryId = 2;

    let graph = KnowledgeGraph {
        nodes: vec![
            make_node(id1, "note", vec![]),
            make_node(id2, "todo", vec![]),
        ],
        edges: vec![make_edge(id1, id2, "related_to")],
    };

    let dot = graph.to_dot();
    assert!(dot.contains("digraph knowledge_graph"));
    assert!(dot.contains(&id1.to_string()));
    assert!(dot.contains(&id2.to_string()));
    assert!(dot.contains("related_to"));
}

#[test]
fn test_community_detection() {
    // Create two clusters
    let a1: MemoryId = 1;
    let a2: MemoryId = 2;
    let a3: MemoryId = 3;
    let b1: MemoryId = 4;
    let b2: MemoryId = 5;

    let graph = KnowledgeGraph {
        nodes: vec![
            make_node(a1, "note", vec!["cluster-a"]),
            make_node(a2, "note", vec!["cluster-a"]),
            make_node(a3, "note", vec!["cluster-a"]),
            make_node(b1, "todo", vec!["cluster-b"]),
            make_node(b2, "todo", vec!["cluster-b"]),
        ],
        edges: vec![
            // Cluster A - densely connected
            make_edge(a1, a2, "related_to"),
            make_edge(a2, a3, "related_to"),
            make_edge(a1, a3, "related_to"),
            // Cluster B - connected
            make_edge(b1, b2, "related_to"),
            // Weak link between clusters
            GraphEdge {
                from: a3,
                to: b1,
                edge_type: "related_to".to_string(),
                score: 0.1, // weak
                confidence: 0.1,
            },
        ],
    };

    let communities = graph.detect_communities(10);
    // Should detect at least the general structure
    assert!(!communities.is_empty());
    // Largest community should have at least 2 members
    assert!(communities[0].members.len() >= 2);
}
