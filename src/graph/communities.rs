//! Community Detection (RML-894)

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::types::{GraphNode, KnowledgeGraph};
use crate::types::MemoryId;

/// A cluster/community of nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphCluster {
    /// Cluster identifier
    pub id: usize,
    /// Node IDs in this cluster
    pub members: Vec<MemoryId>,
    /// Dominant memory type in cluster
    pub dominant_type: Option<String>,
    /// Common tags across cluster
    pub common_tags: Vec<String>,
    /// Internal edge count
    pub internal_edges: usize,
    /// Cluster cohesion score
    pub cohesion: f32,
}

impl KnowledgeGraph {
    /// Detect communities using label propagation algorithm
    pub fn detect_communities(&self, max_iterations: usize) -> Vec<GraphCluster> {
        if self.nodes.is_empty() {
            return Vec::new();
        }

        // Initialize: each node in its own community
        let mut labels: HashMap<MemoryId, usize> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, n)| (n.id, i))
            .collect();

        // Build adjacency
        let mut adj: HashMap<MemoryId, Vec<(MemoryId, f32)>> = HashMap::new();
        for node in &self.nodes {
            adj.insert(node.id, Vec::new());
        }
        for edge in &self.edges {
            let weight = edge.score * edge.confidence;
            adj.entry(edge.from).or_default().push((edge.to, weight));
            adj.entry(edge.to).or_default().push((edge.from, weight));
        }

        // Label propagation
        let node_ids: Vec<MemoryId> = self.nodes.iter().map(|n| n.id).collect();

        for _ in 0..max_iterations {
            let mut changed = false;

            for &node_id in &node_ids {
                if let Some(neighbors) = adj.get(&node_id) {
                    if neighbors.is_empty() {
                        continue;
                    }

                    // Count weighted votes for each label
                    let mut votes: HashMap<usize, f32> = HashMap::new();
                    for &(neighbor, weight) in neighbors {
                        if let Some(&label) = labels.get(&neighbor) {
                            *votes.entry(label).or_insert(0.0) += weight;
                        }
                    }

                    // Pick label with most votes
                    if let Some((&best_label, _)) = votes.iter().max_by(|a, b| a.1.total_cmp(b.1)) {
                        let current = labels.get(&node_id).copied().unwrap_or(0);
                        if best_label != current {
                            labels.insert(node_id, best_label);
                            changed = true;
                        }
                    }
                }
            }

            if !changed {
                break;
            }
        }

        // Group nodes by label
        let mut clusters_map: HashMap<usize, Vec<MemoryId>> = HashMap::new();
        for (node_id, label) in &labels {
            clusters_map.entry(*label).or_default().push(*node_id);
        }

        // Build cluster objects
        let node_map: HashMap<MemoryId, &GraphNode> =
            self.nodes.iter().map(|n| (n.id, n)).collect();

        let mut clusters: Vec<GraphCluster> = clusters_map
            .into_iter()
            .enumerate()
            .map(|(new_id, (_, members))| {
                // Find dominant type
                let mut type_counts: HashMap<&str, usize> = HashMap::new();
                let mut all_tags: HashMap<&str, usize> = HashMap::new();

                for &member_id in &members {
                    if let Some(node) = node_map.get(&member_id) {
                        *type_counts.entry(node.memory_type.as_str()).or_insert(0) += 1;
                        for tag in &node.tags {
                            *all_tags.entry(tag.as_str()).or_insert(0) += 1;
                        }
                    }
                }

                let dominant_type = type_counts
                    .into_iter()
                    .max_by_key(|(_, count)| *count)
                    .map(|(t, _)| t.to_string());

                // Common tags (present in > 50% of members)
                let threshold = members.len() / 2;
                let common_tags: Vec<String> = all_tags
                    .into_iter()
                    .filter(|(_, count)| *count > threshold)
                    .map(|(tag, _)| tag.to_string())
                    .collect();

                // Count internal edges
                let member_set: HashSet<MemoryId> = members.iter().copied().collect();
                let internal_edges = self
                    .edges
                    .iter()
                    .filter(|e| member_set.contains(&e.from) && member_set.contains(&e.to))
                    .count();

                // Cohesion: internal edges / possible internal edges
                let n = members.len();
                let possible = if n > 1 { n * (n - 1) } else { 1 };
                let cohesion = internal_edges as f32 / possible as f32;

                GraphCluster {
                    id: new_id,
                    members,
                    dominant_type,
                    common_tags,
                    internal_edges,
                    cohesion,
                }
            })
            .collect();

        // Sort by size (largest first)
        clusters.sort_by_key(|b| std::cmp::Reverse(b.members.len()));

        // Renumber IDs
        for (i, cluster) in clusters.iter_mut().enumerate() {
            cluster.id = i;
        }

        clusters
    }
}
