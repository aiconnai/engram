//! Graph Filtering (RML-894)

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};

use super::types::{GraphEdge, GraphNode, KnowledgeGraph};
use crate::types::MemoryId;

/// Filter options for graph queries
#[derive(Debug, Clone, Default)]
pub struct GraphFilter {
    /// Filter by memory types
    pub memory_types: Option<Vec<String>>,
    /// Filter by tags (any match)
    pub tags: Option<Vec<String>>,
    /// Filter by edge types
    pub edge_types: Option<Vec<String>>,
    /// Minimum importance threshold
    pub min_importance: Option<f32>,
    /// Maximum importance threshold
    pub max_importance: Option<f32>,
    /// Created after this date
    pub created_after: Option<DateTime<Utc>>,
    /// Created before this date
    pub created_before: Option<DateTime<Utc>>,
    /// Minimum edge confidence
    pub min_confidence: Option<f32>,
    /// Minimum edge score
    pub min_score: Option<f32>,
    /// Maximum number of nodes
    pub limit: Option<usize>,
}

impl GraphFilter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_types(mut self, types: Vec<String>) -> Self {
        self.memory_types = Some(types);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn with_min_importance(mut self, min: f32) -> Self {
        self.min_importance = Some(min);
        self
    }

    pub fn with_min_confidence(mut self, min: f32) -> Self {
        self.min_confidence = Some(min);
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl KnowledgeGraph {
    /// Apply filter to create a subgraph
    pub fn filter(&self, filter: &GraphFilter) -> KnowledgeGraph {
        // Filter nodes
        let mut filtered_nodes: Vec<GraphNode> = self
            .nodes
            .iter()
            .filter(|n| {
                // Type filter
                if let Some(ref types) = filter.memory_types {
                    if !types.contains(&n.memory_type) {
                        return false;
                    }
                }

                // Tag filter (any match)
                if let Some(ref tags) = filter.tags {
                    if !n.tags.iter().any(|t| tags.contains(t)) {
                        return false;
                    }
                }

                // Importance filter
                if let Some(min) = filter.min_importance {
                    if n.importance < min {
                        return false;
                    }
                }
                if let Some(max) = filter.max_importance {
                    if n.importance > max {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        // Apply limit
        if let Some(limit) = filter.limit {
            filtered_nodes.truncate(limit);
        }

        // Get set of valid node IDs
        let valid_ids: HashSet<MemoryId> = filtered_nodes.iter().map(|n| n.id).collect();

        // Filter edges
        let filtered_edges: Vec<GraphEdge> = self
            .edges
            .iter()
            .filter(|e| {
                // Both endpoints must be in filtered nodes
                if !valid_ids.contains(&e.from) || !valid_ids.contains(&e.to) {
                    return false;
                }

                // Edge type filter
                if let Some(ref types) = filter.edge_types {
                    if !types.contains(&e.edge_type) {
                        return false;
                    }
                }

                // Confidence filter
                if let Some(min) = filter.min_confidence {
                    if e.confidence < min {
                        return false;
                    }
                }

                // Score filter
                if let Some(min) = filter.min_score {
                    if e.score < min {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        KnowledgeGraph {
            nodes: filtered_nodes,
            edges: filtered_edges,
        }
    }

    /// Get subgraph centered on a node with given depth
    pub fn neighborhood(&self, center: MemoryId, depth: usize) -> KnowledgeGraph {
        let mut visited: HashSet<MemoryId> = HashSet::new();
        let mut current_level: HashSet<MemoryId> = HashSet::new();
        current_level.insert(center);
        visited.insert(center);

        // Build adjacency
        let mut adj: HashMap<MemoryId, Vec<MemoryId>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.from).or_default().push(edge.to);
            adj.entry(edge.to).or_default().push(edge.from);
        }

        // BFS to depth
        for _ in 0..depth {
            let mut next_level: HashSet<MemoryId> = HashSet::new();
            for &node in &current_level {
                if let Some(neighbors) = adj.get(&node) {
                    for &neighbor in neighbors {
                        if !visited.contains(&neighbor) {
                            visited.insert(neighbor);
                            next_level.insert(neighbor);
                        }
                    }
                }
            }
            current_level = next_level;
        }

        // Filter to visited nodes
        let nodes: Vec<GraphNode> = self
            .nodes
            .iter()
            .filter(|n| visited.contains(&n.id))
            .cloned()
            .collect();

        let edges: Vec<GraphEdge> = self
            .edges
            .iter()
            .filter(|e| visited.contains(&e.from) && visited.contains(&e.to))
            .cloned()
            .collect();

        KnowledgeGraph { nodes, edges }
    }
}
