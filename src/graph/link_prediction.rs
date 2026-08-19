//! Semantic and topological link prediction for the knowledge graph.
//!
//! Predicts missing or implicit relationships between memories using:
//! - Topological metrics (Common Neighbors, Jaccard, Adamic-Adar).
//! - Transitive 2-hop path reasoning (`A -> B -> C` => `A -> C`).
//! - Semantic embedding cosine similarity and tag overlap heuristics.

use std::collections::{HashMap, HashSet};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::embedding::Embedder;
use crate::error::Result;
use crate::search::cosine_similarity;
use crate::storage::queries::create_crossref;
use crate::types::{CreateCrossRefInput, EdgeType};

/// Configuration options for link prediction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictLinksOptions {
    /// Optional source memory ID. If provided, predicts links specifically for this memory.
    pub memory_id: Option<i64>,
    /// Optional workspace filter.
    pub workspace: Option<String>,
    /// Minimum confidence threshold (0.0 to 1.0). Default is 0.6.
    pub min_confidence: f32,
    /// Maximum number of predicted links to return. Default is 10.
    pub top_k: usize,
    /// Prediction algorithm: "hybrid", "topological", "semantic", or "transitive".
    pub algorithm: String,
    /// If true, automatically creates the predicted links in the database.
    pub auto_apply: bool,
}

impl Default for PredictLinksOptions {
    fn default() -> Self {
        Self {
            memory_id: None,
            workspace: None,
            min_confidence: 0.6,
            top_k: 10,
            algorithm: "hybrid".to_string(),
            auto_apply: false,
        }
    }
}

/// A predicted link between two memories with confidence and reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictedLink {
    pub from_id: i64,
    pub to_id: i64,
    pub from_preview: String,
    pub to_preview: String,
    pub predicted_relation: String,
    pub confidence: f32,
    pub reasons: Vec<String>,
}

/// Result envelope containing predicted links and execution stats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictLinksResult {
    pub predictions: Vec<PredictedLink>,
    pub count: usize,
    pub applied_count: usize,
}

#[derive(Debug, Clone)]
struct MemoryNode {
    id: i64,
    content: String,
    tags: Vec<String>,
}

/// Predict missing links in the knowledge graph.
pub fn predict_links(
    conn: &Connection,
    embedder: Option<&dyn Embedder>,
    options: &PredictLinksOptions,
) -> Result<PredictLinksResult> {
    // 1. Fetch memories using storage queries
    let list_opts = crate::types::ListOptions {
        workspace: options.workspace.clone(),
        limit: Some(5000),
        ..Default::default()
    };
    let raw_memories = crate::storage::queries::list_memories(conn, &list_opts)?;
    let memories: Vec<MemoryNode> = raw_memories
        .into_iter()
        .map(|m| MemoryNode {
            id: m.id,
            content: m.content,
            tags: m.tags,
        })
        .collect();

    if memories.len() < 2 {
        return Ok(PredictLinksResult {
            predictions: Vec::new(),
            count: 0,
            applied_count: 0,
        });
    }

    // 2. Fetch embeddings if available or compute on the fly for small candidate sets
    let mem_map: HashMap<i64, MemoryNode> = memories.into_iter().map(|m| (m.id, m)).collect();

    // 3. Load existing edges
    let mut existing_edges: HashSet<(i64, i64)> = HashSet::new();
    let mut adjacency: HashMap<i64, HashSet<i64>> = HashMap::new();
    let mut edge_types: HashMap<(i64, i64), String> = HashMap::new();

    {
        let mut stmt = conn.prepare("SELECT from_id, to_id, edge_type FROM crossrefs")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?;

        for row in rows.flatten() {
            existing_edges.insert((row.0, row.1));
            existing_edges.insert((row.1, row.0)); // Treat undirected for topology
            edge_types.insert((row.0, row.1), row.2.clone());

            adjacency.entry(row.0).or_default().insert(row.1);
            adjacency.entry(row.1).or_default().insert(row.0);
        }
    }

    // 4. Candidate pair generation
    let mut candidate_pairs: HashSet<(i64, i64)> = HashSet::new();

    if let Some(target_id) = options.memory_id {
        if let Some(neighbors) = adjacency.get(&target_id) {
            for &mid in neighbors {
                if let Some(second_hop) = adjacency.get(&mid) {
                    for &candidate in second_hop {
                        if candidate != target_id
                            && !existing_edges.contains(&(target_id, candidate))
                        {
                            candidate_pairs
                                .insert((target_id.min(candidate), target_id.max(candidate)));
                        }
                    }
                }
            }
        }
        // Also pair with all other memories if candidate count is small
        for &other_id in mem_map.keys() {
            if other_id != target_id && !existing_edges.contains(&(target_id, other_id)) {
                candidate_pairs.insert((target_id.min(other_id), target_id.max(other_id)));
            }
        }
    } else {
        // Global / workspace scan
        for (&u, neighbors) in &adjacency {
            for &w in neighbors {
                if let Some(second_hop) = adjacency.get(&w) {
                    for &v in second_hop {
                        if u < v && !existing_edges.contains(&(u, v)) {
                            candidate_pairs.insert((u, v));
                        }
                    }
                }
            }
        }

        // If very few graph edges exist, fall back to pairwise comparison
        if candidate_pairs.len() < 5 {
            let ids: Vec<i64> = mem_map.keys().copied().collect();
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    let u = ids[i].min(ids[j]);
                    let v = ids[i].max(ids[j]);
                    if !existing_edges.contains(&(u, v)) {
                        candidate_pairs.insert((u, v));
                    }
                }
            }
        }
    }

    // 5. Score candidates
    let mut scored_predictions: Vec<PredictedLink> = Vec::new();

    for &(u, v) in &candidate_pairs {
        let node_u = match mem_map.get(&u) {
            Some(n) => n,
            None => continue,
        };
        let node_v = match mem_map.get(&v) {
            Some(n) => n,
            None => continue,
        };

        let empty_set = HashSet::new();
        let adj_u = adjacency.get(&u).unwrap_or(&empty_set);
        let adj_v = adjacency.get(&v).unwrap_or(&empty_set);

        // Common neighbors
        let common: Vec<i64> = adj_u.intersection(adj_v).copied().collect();
        let common_count = common.len();

        // Adamic-Adar
        let mut adamic_adar = 0.0f32;
        for &w in &common {
            let deg = adjacency.get(&w).map(|s| s.len()).unwrap_or(1);
            if deg > 1 {
                adamic_adar += 1.0 / (deg as f32).ln().max(0.1);
            } else {
                adamic_adar += 1.0;
            }
        }

        // Jaccard index
        let union_count = adj_u.union(adj_v).count();
        let jaccard = if union_count > 0 {
            (common_count as f32) / (union_count as f32)
        } else {
            0.0
        };

        let topo_score = ((adamic_adar / 2.0).min(1.0) + jaccard) / 2.0;

        // Tag overlap
        let tags_u: HashSet<&String> = node_u.tags.iter().collect();
        let tags_v: HashSet<&String> = node_v.tags.iter().collect();
        let shared_tags: Vec<String> = tags_u.intersection(&tags_v).map(|s| (*s).clone()).collect();
        let tag_score = if !tags_u.is_empty() && !tags_v.is_empty() {
            (shared_tags.len() as f32) / (tags_u.union(&tags_v).count() as f32)
        } else {
            0.0
        };

        // Semantic embedding similarity
        let mut semantic_score = 0.0f32;
        let mut reasons = Vec::new();

        if let Some(emb) = embedder {
            if let (Ok(vec_u), Ok(vec_v)) = (emb.embed(&node_u.content), emb.embed(&node_v.content))
            {
                semantic_score = cosine_similarity(&vec_u, &vec_v).max(0.0);
                if semantic_score > 0.6 {
                    reasons.push(format!("Semantic cosine similarity: {:.2}", semantic_score));
                }
            }
        }

        if common_count > 0 {
            reasons.push(format!(
                "{} common neighbor(s) (Adamic-Adar: {:.2})",
                common_count, adamic_adar
            ));
        }

        if !shared_tags.is_empty() {
            reasons.push(format!("Shared tags: [{}]", shared_tags.join(", ")));
        }

        // Composite confidence calculation
        let confidence = match options.algorithm.as_str() {
            "topological" => topo_score,
            "semantic" => {
                if semantic_score > 0.0 {
                    semantic_score
                } else {
                    tag_score
                }
            }
            "transitive" => {
                if common_count > 0 {
                    0.7 + (adamic_adar * 0.1).min(0.25)
                } else {
                    0.0
                }
            }
            _ => {
                // "hybrid"
                let mut conf = 0.0f32;
                let mut weight_sum = 0.0f32;

                if semantic_score > 0.0 {
                    conf += semantic_score * 0.50;
                    weight_sum += 0.50;
                }
                if topo_score > 0.0 {
                    conf += topo_score * 0.35;
                    weight_sum += 0.35;
                }
                if tag_score > 0.0 {
                    conf += tag_score * 0.25;
                    weight_sum += 0.25;
                }

                if weight_sum > 0.0 {
                    conf / weight_sum
                } else {
                    0.0
                }
            }
        };

        if confidence >= options.min_confidence {
            let from_preview = if node_u.content.chars().count() > 80 {
                format!("{}...", node_u.content.chars().take(80).collect::<String>())
            } else {
                node_u.content.clone()
            };
            let to_preview = if node_v.content.chars().count() > 80 {
                format!("{}...", node_v.content.chars().take(80).collect::<String>())
            } else {
                node_v.content.clone()
            };

            scored_predictions.push(PredictedLink {
                from_id: u,
                to_id: v,
                from_preview,
                to_preview,
                predicted_relation: "related_to".to_string(),
                confidence,
                reasons,
            });
        }
    }

    // Sort descending by confidence
    scored_predictions.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored_predictions.truncate(options.top_k);

    // 6. Auto-apply if requested
    let mut applied_count = 0;
    if options.auto_apply {
        for pred in &scored_predictions {
            let edge_type: EdgeType = pred
                .predicted_relation
                .parse()
                .unwrap_or(EdgeType::RelatedTo);
            let input = CreateCrossRefInput {
                from_id: pred.from_id,
                to_id: pred.to_id,
                edge_type,
                strength: Some(pred.confidence),
                pinned: false,
                source_context: Some(format!("auto_predicted ({})", options.algorithm)),
            };
            if create_crossref(conn, &input).is_ok() {
                applied_count += 1;
            }
        }
    }

    let count = scored_predictions.len();
    Ok(PredictLinksResult {
        predictions: scored_predictions,
        count,
        applied_count,
    })
}
