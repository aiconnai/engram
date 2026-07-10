//! Core graph node/edge/graph types and construction from memories.

use serde::{Deserialize, Serialize};

use crate::types::{CrossReference, Memory, MemoryId};

/// Graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: MemoryId,
    pub label: String,
    pub memory_type: String,
    pub importance: f32,
    pub tags: Vec<String>,
}

/// Graph edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: MemoryId,
    pub to: MemoryId,
    pub edge_type: String,
    pub score: f32,
    pub confidence: f32,
}

/// Knowledge graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

impl KnowledgeGraph {
    /// Create graph from memories and cross-references
    pub fn from_data(memories: &[Memory], crossrefs: &[CrossReference]) -> Self {
        let nodes: Vec<GraphNode> = memories
            .iter()
            .map(|m| GraphNode {
                id: m.id,
                label: truncate_label(&m.content, 50),
                memory_type: m.memory_type.as_str().to_string(),
                importance: m.importance,
                tags: m.tags.clone(),
            })
            .collect();

        let memory_ids: std::collections::HashSet<MemoryId> =
            memories.iter().map(|m| m.id).collect();

        let edges: Vec<GraphEdge> = crossrefs
            .iter()
            .filter(|cr| memory_ids.contains(&cr.from_id) && memory_ids.contains(&cr.to_id))
            .map(|cr| GraphEdge {
                from: cr.from_id,
                to: cr.to_id,
                edge_type: cr.edge_type.as_str().to_string(),
                score: cr.score,
                confidence: cr.confidence,
            })
            .collect();

        Self { nodes, edges }
    }
}

/// Truncate content for display as node label
pub(super) fn truncate_label(content: &str, max_len: usize) -> String {
    let first_line = content.lines().next().unwrap_or(content);
    if first_line.len() <= max_len {
        first_line.to_string()
    } else {
        format!("{}...", &first_line[..max_len - 3])
    }
}
