//! Search functionality for Engram
//!
//! Implements:
//! - BM25 full-text search (RML-876)
//! - Fuzzy/typo-tolerant search (RML-877)
//! - Search result explanation (RML-878)
//! - Adaptive search strategy (RML-898)
//! - Hybrid search with RRF
//! - Aggregation queries (RML-880)
//! - Search result reranking (RML-927)
//! - Search result caching with adaptive thresholds (ENG-36)

mod aggregation;
mod bm25;
pub mod explain;
pub mod feedback;
mod fuzzy;
pub mod hnsw;
mod hybrid;
mod metadata;
pub mod mmr;
mod rerank;
pub mod result_cache;
pub mod semantic_cache;
pub mod utility;
pub mod vector;

#[cfg(feature = "neural-rerank")]
pub mod neural_rerank;

pub use aggregation::*;
pub use bm25::*;
pub use explain::*;
pub use fuzzy::*;
pub use hnsw::*;
pub use hybrid::*;
pub use metadata::*;
pub use mmr::*;
pub use rerank::*;
pub use result_cache::*;
pub use vector::*;

use crate::types::SearchStrategy;

/// Analyze query to determine optimal search strategy (RML-898)
pub fn select_search_strategy(query: &str) -> SearchStrategy {
    let query = query.trim();
    let word_count = query.split_whitespace().count();
    let has_quotes = query.contains('"');
    let has_operators = query.contains(':')
        || query.contains(" AND ")
        || query.contains(" OR ")
        || query.contains(" NOT ");
    let has_special = query.contains('*') || query.contains('?');

    // Explicit search syntax → keyword only
    if has_quotes || has_operators || has_special {
        return SearchStrategy::KeywordOnly;
    }

    // Very short queries → keyword (faster, usually precise enough)
    if word_count <= 2 {
        return SearchStrategy::KeywordOnly;
    }

    // Long conceptual queries → semantic
    if word_count >= 8 {
        return SearchStrategy::SemanticOnly;
    }

    // Default → hybrid
    SearchStrategy::Hybrid
}

/// Strategy for deduplicating search results across result sets
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DedupeStrategy {
    /// Deduplicate by memory ID (default, fastest)
    #[default]
    ById,
    /// Deduplicate by content hash (catches duplicates with different IDs)
    ByContentHash,
}

/// Configuration for search thresholds
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Word count threshold for short queries (keyword-only)
    pub short_threshold: usize,
    /// Word count threshold for long queries (semantic-only)
    pub long_threshold: usize,
    /// Minimum score to include in results
    pub min_score: f32,
    /// Weight for keyword score in hybrid search
    pub keyword_weight: f32,
    /// Weight for semantic score in hybrid search
    pub semantic_weight: f32,
    /// RRF constant (k parameter, default: 60)
    /// Higher values favor lower-ranked results, lower values favor top results
    pub rrf_k: f32,
    /// Boost factor for project context memories when metadata.project_path matches cwd
    pub project_context_boost: f32,
    /// Current working directory for project context matching
    pub project_context_path: Option<String>,
    /// Deduplication strategy for hybrid search
    pub dedupe_strategy: DedupeStrategy,
    /// Whether in-memory HNSW approximate nearest neighbor search is enabled
    pub hnsw_enabled: bool,
    /// Number of candidates to evaluate during HNSW search phase (ef_search)
    pub hnsw_ef_search: usize,
    /// Number of candidates to evaluate during HNSW construction phase (ef_construction)
    pub hnsw_ef_construction: usize,
    /// Max connections per node at higher layers in HNSW
    pub hnsw_m: usize,
    /// Max connections per node at layer 0 in HNSW
    pub hnsw_m_max0: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            short_threshold: 2,
            long_threshold: 8,
            min_score: 0.1,
            keyword_weight: 0.4,
            semantic_weight: 0.6,
            rrf_k: 60.0,
            project_context_boost: 0.2,
            project_context_path: None,
            dedupe_strategy: DedupeStrategy::default(),
            hnsw_enabled: true,
            hnsw_ef_search: 64,
            hnsw_ef_construction: 128,
            hnsw_m: 16,
            hnsw_m_max0: 32,
        }
    }
}

/// Warm up and populate an HNSW index from SQLite checkpoints and stored embedding BLOBs.
///
/// If a valid HNSW checkpoint exists for the model/dimension, it is restored in $O(1)$ time,
/// and only embeddings created or updated *after* the checkpoint timestamp are incrementally ingested.
/// Otherwise, all valid embeddings in the database are replayed into the index.
///
/// Returns the number of vectors in the warmed up index.
pub fn warmup_hnsw_from_db(
    conn: &rusqlite::Connection,
    hnsw: &mut HnswIndex<i64>,
) -> crate::error::Result<usize> {
    let dim = hnsw.config().dim;
    let model = "default";

    // 1. Try restoring from latest checkpoint
    if let Ok(Some(ckpt)) = crate::storage::queries::get_latest_hnsw_checkpoint(conn, model, dim) {
        if let Ok(loaded_index) = HnswIndex::load_from_bytes(&ckpt.checkpoint_blob) {
            if loaded_index.config().dim == dim {
                *hnsw = loaded_index;
                let checkpoint_time = ckpt.created_at;

                // Incremental catch-up for embeddings newer than checkpoint
                let mut stmt = conn.prepare(
                    "SELECT e.memory_id, e.embedding, e.dimensions
                     FROM embeddings e
                     INNER JOIN memories m ON e.memory_id = m.id
                     WHERE m.valid_to IS NULL AND (e.created_at > ?1 OR m.updated_at > ?1)",
                )?;

                let mut count = 0;
                let mut rows = stmt.query([&checkpoint_time])?;

                while let Some(row) = rows.next()? {
                    let memory_id: i64 = row.get(0)?;
                    let bytes: Vec<u8> = row.get(1)?;
                    let dimensions: usize = row.get(2)?;

                    if dimensions != dim || bytes.len() != dimensions * 4 {
                        continue;
                    }

                    let mut vector = Vec::with_capacity(dimensions);
                    for chunk in bytes.chunks_exact(4) {
                        let arr: [u8; 4] = match chunk.try_into() {
                            Ok(a) => a,
                            Err(_) => break,
                        };
                        vector.push(f32::from_le_bytes(arr));
                    }

                    if vector.len() == dimensions {
                        hnsw.insert(memory_id, &vector);
                        count += 1;
                    }
                }

                tracing::debug!(
                    "HNSW checkpoint restored ({} vectors); caught up {} incremental updates",
                    hnsw.len(),
                    count
                );

                return Ok(hnsw.len());
            }
        }
    }

    // 2. Full replay fallback when no checkpoint exists or dimensions mismatch
    let mut stmt = conn.prepare(
        "SELECT e.memory_id, e.embedding, e.dimensions
         FROM embeddings e
         INNER JOIN memories m ON e.memory_id = m.id
         WHERE m.valid_to IS NULL",
    )?;

    let mut count = 0;
    let mut rows = stmt.query([])?;

    while let Some(row) = rows.next()? {
        let memory_id: i64 = row.get(0)?;
        let bytes: Vec<u8> = row.get(1)?;
        let dimensions: usize = row.get(2)?;

        if dimensions != hnsw.config().dim {
            continue;
        }

        let expected_bytes = dimensions * 4;
        if bytes.len() != expected_bytes {
            continue;
        }

        let mut vector = Vec::with_capacity(dimensions);
        for chunk in bytes.chunks_exact(4) {
            let arr: [u8; 4] = match chunk.try_into() {
                Ok(a) => a,
                Err(_) => break,
            };
            vector.push(f32::from_le_bytes(arr));
        }

        if vector.len() == dimensions {
            hnsw.insert(memory_id, &vector);
            count += 1;
        }
    }

    Ok(count)
}

/// Save a snapshot checkpoint of the HNSW index to SQLite.
pub fn checkpoint_hnsw_to_db(
    conn: &rusqlite::Connection,
    hnsw: &HnswIndex<i64>,
    model: &str,
) -> crate::error::Result<i64> {
    let blob = hnsw.save_to_bytes()?;
    let metric_str = match hnsw.config().metric {
        VectorMetric::Cosine => "cosine",
        VectorMetric::DotProduct => "dot_product",
        VectorMetric::Euclidean => "euclidean",
    };
    let checkpoint_id = crate::storage::queries::save_hnsw_checkpoint(
        conn,
        model,
        hnsw.config().dim,
        metric_str,
        hnsw.len(),
        &blob,
    )?;
    let _ = crate::storage::queries::prune_old_hnsw_checkpoints(conn, model, hnsw.config().dim, 2);
    Ok(checkpoint_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy_selection() {
        // Short queries → keyword
        assert_eq!(select_search_strategy("auth"), SearchStrategy::KeywordOnly);
        assert_eq!(
            select_search_strategy("jwt token"),
            SearchStrategy::KeywordOnly
        );

        // Quoted → keyword
        assert_eq!(
            select_search_strategy("\"exact phrase\""),
            SearchStrategy::KeywordOnly
        );

        // Operators → keyword
        assert_eq!(
            select_search_strategy("auth AND jwt"),
            SearchStrategy::KeywordOnly
        );

        // Medium → hybrid
        assert_eq!(
            select_search_strategy("how does authentication work"),
            SearchStrategy::Hybrid
        );

        // Long → semantic
        assert_eq!(
            select_search_strategy(
                "explain the authentication flow with jwt tokens and refresh mechanism"
            ),
            SearchStrategy::SemanticOnly
        );
    }
}
