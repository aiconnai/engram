use super::*;

/// A pair of potentially duplicate memories with their similarity score
#[derive(Debug, Clone, serde::Serialize)]
pub struct DuplicatePair {
    pub memory_a: Memory,
    pub memory_b: Memory,
    pub similarity_score: f64,
    pub match_type: DuplicateMatchType,
}

/// How the duplicate was detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateMatchType {
    /// Exact content hash match
    ExactHash,
    /// High similarity score from crossrefs
    HighSimilarity,
    /// Semantic similarity via embedding cosine distance
    EmbeddingSimilarity,
}
