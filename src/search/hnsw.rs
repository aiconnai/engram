//! Hierarchical Navigable Small World (HNSW) Graph Index (RFC 0005)
//!
//! High-performance in-memory approximate nearest neighbor (ANN) index over dense embedding vectors.
//! Provides $O(\log N)$ search complexity across multi-layer navigable small world graphs.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::vector::VectorMetric;

/// Configuration parameters for HNSW index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HnswConfig {
    /// Dimension of indexed vectors (e.g. 384, 768, 1536).
    pub dim: usize,
    /// Maximum number of bidirectional connections per node at layers > 0.
    pub m: usize,
    /// Maximum number of bidirectional connections per node at layer 0.
    pub m_max0: usize,
    /// Size of dynamic candidate list during graph construction.
    pub ef_construction: usize,
    /// Size of dynamic candidate list during nearest-neighbor search.
    pub ef_search: usize,
    /// Distance metric used for vector comparisons.
    pub metric: VectorMetric,
    /// Normalization factor for level generation (usually `1.0 / ln(M)`).
    pub ml: f32,
}

impl Default for HnswConfig {
    fn default() -> Self {
        let m = 16;
        Self {
            dim: 384,
            m,
            m_max0: m * 2,
            ef_construction: 64,
            ef_search: 32,
            metric: VectorMetric::Cosine,
            ml: 1.0 / (m as f32).ln(),
        }
    }
}

impl HnswConfig {
    /// Create a new HNSW config with the specified vector dimension and metric.
    pub fn new(dim: usize, metric: VectorMetric) -> Self {
        let m = 16;
        Self {
            dim,
            m,
            m_max0: m * 2,
            ef_construction: 64,
            ef_search: 32,
            metric,
            ml: 1.0 / (m as f32).ln(),
        }
    }

    /// Set construction and search beam widths (`ef_construction` and `ef_search`).
    pub fn with_ef(mut self, ef_construction: usize, ef_search: usize) -> Self {
        self.ef_construction = ef_construction;
        self.ef_search = ef_search;
        self
    }
}

/// A search candidate returned by HNSW query search.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HnswCandidate<Id> {
    /// Identifier of the indexed item.
    pub id: Id,
    /// Distance according to the index metric (smaller = closer).
    pub distance: f32,
    /// Similarity score normalized to [0.0, 1.0] (higher = more similar).
    pub similarity: f32,
}

#[derive(Debug, Clone)]
struct Node<Id> {
    id: Id,
    vector: Vec<f32>,
    /// Adjacency lists for each layer from 0 up to `level`.
    neighbors: Vec<Vec<usize>>,
}

/// Internal element for priority queues keyed by distance.
#[derive(Clone, Copy, PartialEq)]
struct DistNode {
    idx: usize,
    dist: f32,
}

impl Eq for DistNode {}

impl Ord for DistNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for max-heap behavior on smallest distances
        self.dist.total_cmp(&other.dist)
    }
}

impl PartialOrd for DistNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Minimum-distance comparator for min-heaps.
#[derive(Clone, Copy, PartialEq)]
struct MinDistNode {
    idx: usize,
    dist: f32,
}

impl Eq for MinDistNode {}

impl Ord for MinDistNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.dist.total_cmp(&self.dist)
    }
}

impl PartialOrd for MinDistNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Fast in-memory Hierarchical Navigable Small World (HNSW) graph index.
#[derive(Debug, Clone)]
pub struct HnswIndex<Id = i64> {
    config: HnswConfig,
    nodes: Vec<Node<Id>>,
    id_to_idx: HashMap<Id, usize>,
    entry_point: Option<usize>,
    max_level: usize,
    rng_state: u64,
}

impl<Id: Clone + std::hash::Hash + Eq> HnswIndex<Id> {
    /// Create a new empty HNSW index with the provided configuration.
    pub fn new(config: HnswConfig) -> Self {
        Self {
            config,
            nodes: Vec::new(),
            id_to_idx: HashMap::new(),
            entry_point: None,
            max_level: 0,
            rng_state: 0x853c49e6748fea9b,
        }
    }

    /// Number of items stored in the index.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the index contains no items.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Check if the index contains a given ID.
    pub fn contains(&self, id: &Id) -> bool {
        self.id_to_idx.contains_key(id)
    }

    /// Get index configuration.
    pub fn config(&self) -> &HnswConfig {
        &self.config
    }

    /// Pseudorandom level generator using a fast linear congruential generator.
    fn random_level(&mut self) -> usize {
        // LCG algorithm
        self.rng_state = self
            .rng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let uniform = ((self.rng_state >> 11) as f32) / ((1u64 << 53) as f32);
        let uniform = uniform.clamp(1e-7, 1.0 - 1e-7);
        (-uniform.ln() * self.config.ml).floor() as usize
    }

    /// Distance between an external query vector and an internal node index.
    #[inline]
    fn dist_to_vec(&self, query: &[f32], node_idx: usize) -> f32 {
        self.config
            .metric
            .distance(query, &self.nodes[node_idx].vector)
    }

    /// Insert a vector with its associated item ID into the HNSW index.
    ///
    /// If the ID already exists, its coordinates are updated and its multi-layer
    /// neighborhood is dynamically re-linked to preserve graph topology.
    pub fn insert(&mut self, id: Id, vector: &[f32]) {
        if vector.len() != self.config.dim {
            return;
        }

        if let Some(&existing_idx) = self.id_to_idx.get(&id) {
            // Update vector and re-link its multi-layer neighborhood
            self.nodes[existing_idx].vector = vector.to_vec();
            let node_level = self.nodes[existing_idx].neighbors.len().saturating_sub(1);
            if let Some(entry) = self.entry_point {
                let mut curr_obj = entry;
                let mut curr_dist = self.dist_to_vec(vector, curr_obj);
                let top_level = self.max_level;

                if top_level > node_level {
                    for level in ((node_level + 1)..=top_level).rev() {
                        let mut changed = true;
                        while changed {
                            changed = false;
                            for &neighbor_idx in &self.nodes[curr_obj].neighbors[level] {
                                if neighbor_idx == existing_idx {
                                    continue;
                                }
                                let d = self.dist_to_vec(vector, neighbor_idx);
                                if d < curr_dist {
                                    curr_dist = d;
                                    curr_obj = neighbor_idx;
                                    changed = true;
                                }
                            }
                        }
                    }
                }

                let start_level = node_level.min(top_level);
                let mut ep_vec = vec![curr_obj];

                for level in (0..=start_level).rev() {
                    let candidates = self.search_layer_internal(
                        vector,
                        &ep_vec,
                        self.config.ef_construction,
                        level,
                    );
                    let m_limit = if level == 0 {
                        self.config.m_max0
                    } else {
                        self.config.m
                    };

                    let selected_neighbors = self.select_neighbors(&candidates, m_limit);
                    let filtered_neighbors: Vec<usize> = selected_neighbors
                        .into_iter()
                        .filter(|&idx| idx != existing_idx)
                        .collect();

                    self.nodes[existing_idx].neighbors[level] = filtered_neighbors.clone();

                    for &neighbor_idx in &filtered_neighbors {
                        if !self.nodes[neighbor_idx].neighbors[level].contains(&existing_idx) {
                            self.nodes[neighbor_idx].neighbors[level].push(existing_idx);
                        }
                    }

                    ep_vec = candidates.into_iter().map(|cn| cn.idx).collect();
                }
            }
            return;
        }

        let node_level = self.random_level();
        let new_idx = self.nodes.len();

        let mut neighbors = Vec::with_capacity(node_level + 1);
        for _ in 0..=node_level {
            neighbors.push(Vec::new());
        }

        let node = Node {
            id: id.clone(),
            vector: vector.to_vec(),
            neighbors,
        };
        self.nodes.push(node);
        self.id_to_idx.insert(id, new_idx);

        let entry = match self.entry_point {
            Some(ep) => ep,
            None => {
                self.entry_point = Some(new_idx);
                self.max_level = node_level;
                return;
            }
        };

        let mut curr_obj = entry;
        let mut curr_dist = self.dist_to_vec(vector, curr_obj);
        let top_level = self.max_level;

        // 1. Greedily descend from top level down to node_level + 1
        if top_level > node_level {
            for level in ((node_level + 1)..=top_level).rev() {
                let mut changed = true;
                while changed {
                    changed = false;
                    for &neighbor_idx in &self.nodes[curr_obj].neighbors[level] {
                        let d = self.dist_to_vec(vector, neighbor_idx);
                        if d < curr_dist {
                            curr_dist = d;
                            curr_obj = neighbor_idx;
                            changed = true;
                        }
                    }
                }
            }
        }

        // 2. From min(node_level, top_level) down to 0, search and connect neighbors
        let start_level = node_level.min(top_level);
        let mut ep_vec = vec![curr_obj];

        for level in (0..=start_level).rev() {
            let candidates =
                self.search_layer_internal(vector, &ep_vec, self.config.ef_construction, level);
            let m_limit = if level == 0 {
                self.config.m_max0
            } else {
                self.config.m
            };

            let selected_neighbors = self.select_neighbors(&candidates, m_limit);

            // Connect new node to selected neighbors
            self.nodes[new_idx].neighbors[level] = selected_neighbors.clone();

            // Connect neighbors back to new node (bidirectional links)
            for &neighbor_idx in &selected_neighbors {
                self.nodes[neighbor_idx].neighbors[level].push(new_idx);

                // Shrink neighbor's link list if it exceeds limit without cloning full vectors
                if self.nodes[neighbor_idx].neighbors[level].len() > m_limit {
                    let metric = self.config.metric;
                    let n_vec = &self.nodes[neighbor_idx].vector;
                    let links = &self.nodes[neighbor_idx].neighbors[level];
                    let mut candidate_dists: Vec<(usize, f32)> = links
                        .iter()
                        .map(|&idx| (idx, metric.distance(n_vec, &self.nodes[idx].vector)))
                        .collect();
                    candidate_dists.sort_unstable_by(|a, b| a.1.total_cmp(&b.1));
                    candidate_dists.truncate(m_limit);
                    self.nodes[neighbor_idx].neighbors[level] =
                        candidate_dists.into_iter().map(|(idx, _)| idx).collect();
                }
            }

            ep_vec = candidates.into_iter().map(|cn| cn.idx).collect();
        }

        // 3. Update entry point if new node has a higher level
        if node_level > self.max_level {
            self.max_level = node_level;
            self.entry_point = Some(new_idx);
        }
    }

    /// Search candidate elements in a specific layer.
    fn search_layer_internal(
        &self,
        query: &[f32],
        entry_points: &[usize],
        ef: usize,
        level: usize,
    ) -> Vec<DistNode> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new(); // Min-heap of closest candidates to explore
        let mut w = BinaryHeap::new(); // Max-heap of furthest elements found so far (size bounded by ef)

        for &ep in entry_points {
            let dist = self.dist_to_vec(query, ep);
            visited.insert(ep);
            candidates.push(MinDistNode { idx: ep, dist });
            w.push(DistNode { idx: ep, dist });
        }

        while let Some(current) = candidates.pop() {
            let furthest_dist = w.peek().map(|dn| dn.dist).unwrap_or(f32::INFINITY);
            if current.dist > furthest_dist {
                break;
            }

            if level < self.nodes[current.idx].neighbors.len() {
                for &neighbor_idx in &self.nodes[current.idx].neighbors[level] {
                    if visited.insert(neighbor_idx) {
                        let furthest_dist = w.peek().map(|dn| dn.dist).unwrap_or(f32::INFINITY);
                        let d = self.dist_to_vec(query, neighbor_idx);

                        if d < furthest_dist || w.len() < ef {
                            candidates.push(MinDistNode {
                                idx: neighbor_idx,
                                dist: d,
                            });
                            w.push(DistNode {
                                idx: neighbor_idx,
                                dist: d,
                            });

                            if w.len() > ef {
                                w.pop();
                            }
                        }
                    }
                }
            }
        }

        w.into_sorted_vec()
    }

    /// Select the best `m` neighbors from candidate list.
    fn select_neighbors(&self, candidates: &[DistNode], m: usize) -> Vec<usize> {
        let mut sorted = candidates.to_vec();
        sorted.sort_unstable_by(|a, b| a.dist.total_cmp(&b.dist));
        sorted.into_iter().take(m).map(|dn| dn.idx).collect()
    }

    /// Search the HNSW index for the `k` nearest neighbors to `query`.
    ///
    /// Returns candidates sorted from nearest to furthest.
    pub fn search(&self, query: &[f32], k: usize, ef: Option<usize>) -> Vec<HnswCandidate<Id>> {
        if self.nodes.is_empty() || k == 0 || query.len() != self.config.dim {
            return Vec::new();
        }

        let entry = match self.entry_point {
            Some(ep) => ep,
            None => return Vec::new(),
        };

        let ef_val = ef.unwrap_or(self.config.ef_search).max(k);
        let mut curr_obj = entry;
        let mut curr_dist = self.dist_to_vec(query, curr_obj);

        // 1. Descend from top layer down to layer 1 greedily with ef = 1
        for level in (1..=self.max_level).rev() {
            let mut changed = true;
            while changed {
                changed = false;
                if level < self.nodes[curr_obj].neighbors.len() {
                    for &neighbor_idx in &self.nodes[curr_obj].neighbors[level] {
                        let d = self.dist_to_vec(query, neighbor_idx);
                        if d < curr_dist {
                            curr_dist = d;
                            curr_obj = neighbor_idx;
                            changed = true;
                        }
                    }
                }
            }
        }

        // 2. Run beam search in layer 0 with ef
        let candidates = self.search_layer_internal(query, &[curr_obj], ef_val, 0);

        // 3. Take top-k and construct results with metric similarity scores
        let mut results = Vec::with_capacity(k.min(candidates.len()));
        for cn in candidates.into_iter().take(k) {
            let node = &self.nodes[cn.idx];
            let similarity = self.config.metric.similarity(query, &node.vector);

            results.push(HnswCandidate {
                id: node.id.clone(),
                distance: cn.dist,
                similarity,
            });
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hnsw_empty_and_single() {
        let config = HnswConfig::new(4, VectorMetric::Cosine);
        let mut index = HnswIndex::new(config);
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);

        let query = [1.0, 0.0, 0.0, 0.0];
        assert!(index.search(&query, 5, None).is_empty());

        index.insert(100, &[1.0, 0.0, 0.0, 0.0]);
        assert_eq!(index.len(), 1);
        assert!(index.contains(&100));

        let res = index.search(&query, 5, None);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].id, 100);
        assert!((res[0].similarity - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_hnsw_top_k_retrieval() {
        let config = HnswConfig::new(3, VectorMetric::Cosine).with_ef(32, 32);
        let mut index = HnswIndex::new(config);

        index.insert(1, &[1.0, 0.0, 0.0]);
        index.insert(2, &[0.8, 0.2, 0.0]);
        index.insert(3, &[0.0, 1.0, 0.0]);
        index.insert(4, &[0.0, 0.0, 1.0]);

        let query = [0.9, 0.1, 0.0];
        let res = index.search(&query, 2, None);
        assert_eq!(res.len(), 2);
        assert_eq!(res[0].id, 1);
        assert_eq!(res[1].id, 2);
    }
}
