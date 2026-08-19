//! Semantic concept clustering over knowledge graph memories.
//!
//! Partitions the memory graph into coherent conceptual clusters, synthesizes
//! high-level labels and key themes, and identifies representative memories.

use std::collections::{HashMap, HashSet};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Configuration options for concept clustering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptClusterOptions {
    /// Optional workspace filter.
    pub workspace: Option<String>,
    /// Minimum number of memories required for a cluster. Default is 2.
    pub min_cluster_size: usize,
    /// Maximum number of concept clusters to return. Default is 10.
    pub max_clusters: usize,
}

impl Default for ConceptClusterOptions {
    fn default() -> Self {
        Self {
            workspace: None,
            min_cluster_size: 2,
            max_clusters: 10,
        }
    }
}

/// A semantic concept cluster summarizing a group of related memories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptCluster {
    pub concept_id: usize,
    pub label: String,
    pub description: String,
    pub size: usize,
    pub member_ids: Vec<i64>,
    pub representative_memory_id: i64,
    pub key_tags: Vec<String>,
    pub cohesion: f32,
}

struct MemoryData {
    id: i64,
    content: String,
    tags: Vec<String>,
}

/// Run semantic concept clustering over the knowledge graph.
pub fn cluster_concepts(
    conn: &Connection,
    options: &ConceptClusterOptions,
) -> Result<Vec<ConceptCluster>> {
    // 1. Fetch memories
    let list_opts = crate::types::ListOptions {
        workspace: options.workspace.clone(),
        limit: Some(5000),
        ..Default::default()
    };
    let raw_memories = crate::storage::queries::list_memories(conn, &list_opts)?;
    let memories: HashMap<i64, MemoryData> = raw_memories
        .into_iter()
        .map(|m| {
            (
                m.id,
                MemoryData {
                    id: m.id,
                    content: m.content,
                    tags: m.tags,
                },
            )
        })
        .collect();

    if memories.is_empty() {
        return Ok(Vec::new());
    }

    // 2. Fetch edges
    let mut adjacency: HashMap<i64, HashSet<i64>> = HashMap::new();
    let mut all_edges: HashSet<(i64, i64)> = HashSet::new();

    {
        let mut stmt = conn.prepare("SELECT from_id, to_id FROM crossrefs")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))?;

        for row in rows.flatten() {
            if memories.contains_key(&row.0) && memories.contains_key(&row.1) {
                adjacency.entry(row.0).or_default().insert(row.1);
                adjacency.entry(row.1).or_default().insert(row.0);
                all_edges.insert((row.0.min(row.1), row.0.max(row.1)));
            }
        }
    }

    // 3. Find connected components or partition by shared tags if graph is sparse
    let mut visited: HashSet<i64> = HashSet::new();
    let mut raw_clusters: Vec<Vec<i64>> = Vec::new();

    for &node_id in memories.keys() {
        if visited.contains(&node_id) {
            continue;
        }

        let mut component: Vec<i64> = Vec::new();
        let mut queue = vec![node_id];
        visited.insert(node_id);

        while let Some(current) = queue.pop() {
            component.push(current);
            if let Some(neighbors) = adjacency.get(&current) {
                for &nbr in neighbors {
                    if !visited.contains(&nbr) {
                        visited.insert(nbr);
                        queue.push(nbr);
                    }
                }
            }
        }

        if component.len() >= options.min_cluster_size {
            raw_clusters.push(component);
        }
    }

    // If no graph components met min_cluster_size, cluster by shared tags
    if raw_clusters.is_empty() {
        let mut tag_to_members: HashMap<String, Vec<i64>> = HashMap::new();
        for mem in memories.values() {
            for tag in &mem.tags {
                tag_to_members.entry(tag.clone()).or_default().push(mem.id);
            }
        }
        for (_, members) in tag_to_members {
            if members.len() >= options.min_cluster_size {
                raw_clusters.push(members);
            }
        }
    }

    // 4. Synthesize concept metadata for each cluster
    let mut result_clusters: Vec<ConceptCluster> = Vec::new();

    for (idx, mut members) in raw_clusters.into_iter().enumerate() {
        members.sort_unstable();
        members.dedup();

        let size = members.len();
        if size < options.min_cluster_size {
            continue;
        }

        // Count tag frequencies
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        for &m_id in &members {
            if let Some(mem) = memories.get(&m_id) {
                for tag in &mem.tags {
                    *tag_counts.entry(tag.clone()).or_default() += 1;
                }
            }
        }

        let mut sorted_tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
        sorted_tags.sort_by_key(|b| std::cmp::Reverse(b.1));
        let key_tags: Vec<String> = sorted_tags.into_iter().take(4).map(|(t, _)| t).collect();

        // Select representative memory (highest degree in cluster or first)
        let mut best_id = members[0];
        let mut max_deg = 0;
        for &m_id in &members {
            let deg = adjacency
                .get(&m_id)
                .map(|nbrs| nbrs.iter().filter(|n| members.contains(n)).count())
                .unwrap_or(0);
            if deg > max_deg {
                max_deg = deg;
                best_id = m_id;
            }
        }

        let rep_mem = memories.get(&best_id);
        let rep_preview = rep_mem
            .map(|m| {
                if m.content.chars().count() > 60 {
                    format!("{}...", m.content.chars().take(60).collect::<String>())
                } else {
                    m.content.clone()
                }
            })
            .unwrap_or_else(|| format!("Memory #{}", best_id));

        // Synthesize label
        let label = if !key_tags.is_empty() {
            format!("Concept: {}", key_tags.join(" • "))
        } else {
            format!("Concept #{}: {}", idx + 1, rep_preview)
        };

        let description = format!(
            "Cluster of {} related memories centered around '{}'.",
            size, rep_preview
        );

        // Compute cohesion
        let mut internal_edges = 0;
        for &u in &members {
            if let Some(nbrs) = adjacency.get(&u) {
                for &v in nbrs {
                    if u < v && members.contains(&v) {
                        internal_edges += 1;
                    }
                }
            }
        }

        let max_possible = (size * (size - 1)) / 2;
        let cohesion = if max_possible > 0 {
            (internal_edges as f32) / (max_possible as f32)
        } else {
            1.0
        };

        result_clusters.push(ConceptCluster {
            concept_id: idx + 1,
            label,
            description,
            size,
            member_ids: members,
            representative_memory_id: best_id,
            key_tags,
            cohesion,
        });
    }

    // Sort descending by size
    result_clusters.sort_by_key(|b| std::cmp::Reverse(b.size));
    result_clusters.truncate(options.max_clusters);

    Ok(result_clusters)
}
