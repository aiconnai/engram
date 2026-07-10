//! Graph Statistics (RML-894)

use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};

use super::types::KnowledgeGraph;
use crate::types::MemoryId;

/// Graph statistics and metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    /// Total number of nodes
    pub node_count: usize,
    /// Total number of edges
    pub edge_count: usize,
    /// Average degree (edges per node)
    pub avg_degree: f32,
    /// Graph density (actual edges / possible edges)
    pub density: f32,
    /// Number of connected components
    pub component_count: usize,
    /// Size of largest component
    pub largest_component_size: usize,
    /// Nodes by memory type
    pub nodes_by_type: HashMap<String, usize>,
    /// Edges by type
    pub edges_by_type: HashMap<String, usize>,
    /// Most connected nodes (top 10 by degree)
    pub hub_nodes: Vec<(MemoryId, usize)>,
    /// Isolated nodes (degree 0)
    pub isolated_count: usize,
}

impl KnowledgeGraph {
    /// Calculate graph statistics
    pub fn stats(&self) -> GraphStats {
        let node_count = self.nodes.len();
        let edge_count = self.edges.len();

        // Build adjacency for degree calculation
        let mut degree: HashMap<MemoryId, usize> = HashMap::new();
        for node in &self.nodes {
            degree.insert(node.id, 0);
        }
        for edge in &self.edges {
            *degree.entry(edge.from).or_insert(0) += 1;
            *degree.entry(edge.to).or_insert(0) += 1;
        }

        let avg_degree = if node_count > 0 {
            degree.values().sum::<usize>() as f32 / node_count as f32
        } else {
            0.0
        };

        // Density: edges / (n * (n-1) / 2) for undirected, edges / (n * (n-1)) for directed
        let density = if node_count > 1 {
            edge_count as f32 / (node_count * (node_count - 1)) as f32
        } else {
            0.0
        };

        // Count by type
        let mut nodes_by_type: HashMap<String, usize> = HashMap::new();
        for node in &self.nodes {
            *nodes_by_type.entry(node.memory_type.clone()).or_insert(0) += 1;
        }

        let mut edges_by_type: HashMap<String, usize> = HashMap::new();
        for edge in &self.edges {
            *edges_by_type.entry(edge.edge_type.clone()).or_insert(0) += 1;
        }

        // Find hub nodes (top 10 by degree)
        let mut degree_list: Vec<(MemoryId, usize)> =
            degree.iter().map(|(&k, &v)| (k, v)).collect();
        degree_list.sort_by_key(|b| std::cmp::Reverse(b.1));
        let hub_nodes: Vec<(MemoryId, usize)> = degree_list.into_iter().take(10).collect();

        // Count isolated nodes
        let isolated_count = degree.values().filter(|&&d| d == 0).count();

        // Find connected components using BFS
        let components = self.find_connected_components();
        let component_count = components.len();
        let largest_component_size = components.iter().map(|c| c.len()).max().unwrap_or(0);

        GraphStats {
            node_count,
            edge_count,
            avg_degree,
            density,
            component_count,
            largest_component_size,
            nodes_by_type,
            edges_by_type,
            hub_nodes,
            isolated_count,
        }
    }

    /// Find connected components using BFS
    fn find_connected_components(&self) -> Vec<Vec<MemoryId>> {
        let node_ids: HashSet<MemoryId> = self.nodes.iter().map(|n| n.id).collect();

        // Build adjacency list (undirected)
        let mut adj: HashMap<MemoryId, Vec<MemoryId>> = HashMap::new();
        for id in &node_ids {
            adj.insert(*id, Vec::new());
        }
        for edge in &self.edges {
            if let Some(list) = adj.get_mut(&edge.from) {
                list.push(edge.to);
            }
            if let Some(list) = adj.get_mut(&edge.to) {
                list.push(edge.from);
            }
        }

        let mut visited: HashSet<MemoryId> = HashSet::new();
        let mut components = Vec::new();

        for &start in &node_ids {
            if visited.contains(&start) {
                continue;
            }

            let mut component = Vec::new();
            let mut queue = VecDeque::new();
            queue.push_back(start);
            visited.insert(start);

            while let Some(node) = queue.pop_front() {
                component.push(node);
                if let Some(neighbors) = adj.get(&node) {
                    for &neighbor in neighbors {
                        if !visited.contains(&neighbor) {
                            visited.insert(neighbor);
                            queue.push_back(neighbor);
                        }
                    }
                }
            }

            components.push(component);
        }

        components
    }

    /// Calculate centrality scores for nodes
    pub fn centrality(&self) -> HashMap<MemoryId, CentralityScores> {
        let mut results: HashMap<MemoryId, CentralityScores> = HashMap::new();

        // Build adjacency
        let mut in_degree: HashMap<MemoryId, usize> = HashMap::new();
        let mut out_degree: HashMap<MemoryId, usize> = HashMap::new();

        for node in &self.nodes {
            in_degree.insert(node.id, 0);
            out_degree.insert(node.id, 0);
        }

        for edge in &self.edges {
            *out_degree.entry(edge.from).or_insert(0) += 1;
            *in_degree.entry(edge.to).or_insert(0) += 1;
        }

        let max_degree = self.nodes.len().saturating_sub(1).max(1) as f32;

        for node in &self.nodes {
            let in_d = *in_degree.get(&node.id).unwrap_or(&0) as f32;
            let out_d = *out_degree.get(&node.id).unwrap_or(&0) as f32;

            results.insert(
                node.id,
                CentralityScores {
                    in_degree: in_d / max_degree,
                    out_degree: out_d / max_degree,
                    degree: (in_d + out_d) / (2.0 * max_degree),
                    // Simplified closeness based on direct connections
                    closeness: (in_d + out_d) / (2.0 * max_degree),
                },
            );
        }

        results
    }
}

/// Centrality scores for a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralityScores {
    /// Normalized in-degree centrality
    pub in_degree: f32,
    /// Normalized out-degree centrality
    pub out_degree: f32,
    /// Combined degree centrality
    pub degree: f32,
    /// Closeness centrality (simplified)
    pub closeness: f32,
}
