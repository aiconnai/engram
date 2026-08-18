//! Hybrid search combining BM25 and semantic search
//!
//! Uses Reciprocal Rank Fusion (RRF) to combine results from
//! keyword and vector search.

use std::collections::HashMap;

use chrono::Utc;
use rusqlite::Connection;

use super::bm25::bm25_search_complete_with_scope_path;
use super::{select_search_strategy, SearchConfig};
use crate::embedding::{cosine_similarity, get_embedding};
use crate::error::Result;
use crate::storage::filter::{parse_filter, SqlBuilder};
use crate::storage::queries::{load_tags, memory_from_row};
use crate::types::{MatchInfo, Memory, MemoryId, SearchOptions, SearchResult, SearchStrategy};

fn apply_project_context_boost(memory: &Memory, score: f32, config: &SearchConfig) -> f32 {
    if let Some(ref project_path) = config.project_context_path {
        // Check if memory is a project context memory matching the current path
        if memory.tags.contains(&"project-context".to_string()) {
            if let Some(memory_path) = memory.metadata.get("project_path") {
                if memory_path.as_str() == Some(project_path.as_str()) {
                    return score * (1.0 + config.project_context_boost);
                }
            }
        }
    }
    score
}

/// Perform hybrid search with automatic strategy selection and optional HNSW acceleration
pub fn hybrid_search_with_hnsw(
    conn: &Connection,
    query: &str,
    query_embedding: Option<&[f32]>,
    options: &SearchOptions,
    config: &SearchConfig,
    hnsw_index: Option<&crate::search::hnsw::HnswIndex<i64>>,
) -> Result<Vec<SearchResult>> {
    let strategy = options
        .strategy
        .unwrap_or_else(|| select_search_strategy(query));
    let limit = options.limit.unwrap_or(20);
    let min_score = options.min_score.unwrap_or(config.min_score);

    match strategy {
        SearchStrategy::KeywordOnly => {
            keyword_only_search(conn, query, limit, min_score, options, config)
        }
        SearchStrategy::SemanticOnly => {
            if let Some(embedding) = query_embedding {
                semantic_only_search_with_hnsw(
                    conn, embedding, limit, min_score, options, config, hnsw_index,
                )
            } else {
                // Fallback to keyword if no embedding
                keyword_only_search(conn, query, limit, min_score, options, config)
            }
        }
        SearchStrategy::Hybrid => {
            if let Some(embedding) = query_embedding {
                rrf_hybrid_search_with_hnsw(
                    conn, query, embedding, limit, min_score, options, config, hnsw_index,
                )
            } else {
                keyword_only_search(conn, query, limit, min_score, options, config)
            }
        }
    }
}

/// Perform hybrid search with automatic strategy selection (legacy signature)
pub fn hybrid_search(
    conn: &Connection,
    query: &str,
    query_embedding: Option<&[f32]>,
    options: &SearchOptions,
    config: &SearchConfig,
) -> Result<Vec<SearchResult>> {
    hybrid_search_with_hnsw(conn, query, query_embedding, options, config, None)
}

/// Keyword-only search using BM25
fn keyword_only_search(
    conn: &Connection,
    query: &str,
    limit: i64,
    min_score: f32,
    options: &SearchOptions,
    config: &SearchConfig,
) -> Result<Vec<SearchResult>> {
    // When global=true, skip all workspace filters to search across all workspaces.
    let workspace_filter = if options.global {
        None
    } else {
        options.workspace.as_deref()
    };
    let workspaces_filter: Option<&[String]> = if options.global {
        None
    } else {
        options.workspaces.as_deref()
    };

    let bm25_results = bm25_search_complete_with_scope_path(
        conn,
        query,
        limit * 2,
        options.explain,
        options.scope.as_ref(),
        options.filter.as_ref(),
        options.include_transcripts,
        options.include_archived,
        workspace_filter,
        workspaces_filter,
        options.tier.as_ref(),
        options.scope_path.as_deref(),
    )?;

    let mut results: Vec<SearchResult> = bm25_results
        .into_iter()
        .filter(|r| r.score >= min_score)
        .map(|r| {
            let boosted_score = apply_project_context_boost(&r.memory, r.score, config);
            SearchResult {
                memory: r.memory,
                score: boosted_score,
                match_info: MatchInfo {
                    strategy: SearchStrategy::KeywordOnly,
                    matched_terms: r.matched_terms,
                    highlights: r.highlights,
                    semantic_score: None,
                    keyword_score: Some(r.score),
                },
            }
        })
        .collect();

    // Re-sort after applying boost
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit as usize);

    Ok(results)
}

/// Semantic-only search using HNSW vector index or falling back to SQLite linear scan
fn semantic_only_search_with_hnsw(
    conn: &Connection,
    query_embedding: &[f32],
    limit: i64,
    min_score: f32,
    options: &SearchOptions,
    config: &SearchConfig,
    hnsw_index: Option<&crate::search::hnsw::HnswIndex<i64>>,
) -> Result<Vec<SearchResult>> {
    if config.hnsw_enabled {
        if let Some(hnsw) = hnsw_index {
            if !hnsw.is_empty() && query_embedding.len() == hnsw.config().dim {
                let candidate_k = (limit * 3).max(16) as usize;
                let candidates =
                    hnsw.search(query_embedding, candidate_k, Some(config.hnsw_ef_search));
                if !candidates.is_empty() {
                    let hydrated = semantic_hydrate_hnsw_candidates(
                        conn, candidates, limit, min_score, options, config,
                    )?;
                    if !hydrated.is_empty() {
                        return Ok(hydrated);
                    }
                }
            }
        }
    }

    // Fallback to SQLite linear scan
    semantic_only_search(conn, query_embedding, limit, min_score, options, config)
}

/// Hydrate candidate records returned by HNSW index from SQLite and apply metadata/workspace filters
fn semantic_hydrate_hnsw_candidates(
    conn: &Connection,
    candidates: Vec<crate::search::hnsw::HnswCandidate<i64>>,
    limit: i64,
    min_score: f32,
    options: &SearchOptions,
    config: &SearchConfig,
) -> Result<Vec<SearchResult>> {
    let now = Utc::now().to_rfc3339();
    let candidate_ids: Vec<i64> = candidates.iter().map(|c| c.id).collect();
    let candidate_sim_map: HashMap<i64, f32> = candidates
        .into_iter()
        .map(|c| (c.id, c.similarity))
        .collect();

    if candidate_ids.is_empty() {
        return Ok(Vec::new());
    }

    let placeholders: Vec<String> = candidate_ids.iter().map(|_| "?".to_string()).collect();
    let mut sql = format!(
        "SELECT m.id, m.content, m.memory_type, m.importance, m.access_count,
                m.created_at, m.updated_at, m.last_accessed_at, m.owner_id,
                m.visibility, m.version, m.has_embedding, m.metadata,
                m.scope_type, m.scope_id, m.expires_at,
                m.workspace, m.tier, m.lifecycle_state
         FROM memories m
         WHERE m.id IN ({}) AND m.valid_to IS NULL
           AND (m.expires_at IS NULL OR m.expires_at > ?)",
        placeholders.join(", ")
    );

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = candidate_ids
        .iter()
        .map(|&id| Box::new(id) as Box<dyn rusqlite::ToSql>)
        .collect();
    params.push(Box::new(now));

    if !options.include_transcripts {
        sql.push_str(" AND m.memory_type != 'transcript_chunk'");
    }

    if !options.include_archived {
        sql.push_str(" AND (m.lifecycle_state IS NULL OR m.lifecycle_state != 'archived')");
    }

    if let Some(ref filter_json) = options.filter {
        let filter_expr = parse_filter(filter_json)?;
        let mut builder = SqlBuilder::new();
        let filter_sql = builder.build_filter(&filter_expr)?;
        sql.push_str(" AND ");
        sql.push_str(&filter_sql);
        for param in builder.take_params() {
            params.push(param);
        }
    } else {
        if let Some(ref tags) = options.tags {
            if !tags.is_empty() {
                sql.push_str(
                    " AND m.id IN (
                        SELECT mt.memory_id FROM memory_tags mt
                        JOIN tags t ON mt.tag_id = t.id
                        WHERE t.name IN (",
                );
                let tag_placeholders: Vec<&str> = tags.iter().map(|_| "?").collect();
                sql.push_str(&tag_placeholders.join(", "));
                sql.push_str("))");
                for tag in tags {
                    params.push(Box::new(tag.clone()));
                }
            }
        }

        if let Some(ref memory_type) = options.memory_type {
            sql.push_str(" AND m.memory_type = ?");
            params.push(Box::new(memory_type.as_str().to_string()));
        }
    }

    if let Some(ref scope) = options.scope {
        sql.push_str(" AND m.scope_type = ?");
        params.push(Box::new(scope.scope_type().to_string()));
        if let Some(scope_id) = scope.scope_id() {
            sql.push_str(" AND m.scope_id = ?");
            params.push(Box::new(scope_id.to_string()));
        } else {
            sql.push_str(" AND m.scope_id IS NULL");
        }
    }

    if options.global {
        // Search across all workspaces
    } else if let Some(ref workspace) = options.workspace {
        sql.push_str(" AND m.workspace = ?");
        params.push(Box::new(workspace.clone()));
    } else if let Some(ref workspaces) = options.workspaces {
        if !workspaces.is_empty() {
            let ws_placeholders: Vec<&str> = workspaces.iter().map(|_| "?").collect();
            sql.push_str(&format!(
                " AND m.workspace IN ({})",
                ws_placeholders.join(", ")
            ));
            for ws in workspaces {
                params.push(Box::new(ws.clone()));
            }
        }
    }

    if let Some(ref tier) = options.tier {
        sql.push_str(&format!(" AND m.tier = '{}'", tier.as_str()));
    }

    if let Some(ref sp) = options.scope_path {
        let escaped = sp.replace('%', "\\%").replace('_', "\\_");
        sql.push_str(" AND (m.scope_path = ? OR m.scope_path LIKE ? ESCAPE '\\')");
        params.push(Box::new(sp.clone()));
        params.push(Box::new(format!("{}/", escaped) + "%"));
    }

    let mut stmt = conn.prepare(&sql)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();

    let memories: Vec<Memory> = stmt
        .query_map(param_refs.as_slice(), memory_from_row)?
        .filter_map(|r| r.ok())
        .map(|mut m| {
            m.tags = load_tags(conn, m.id).unwrap_or_default();
            m
        })
        .collect();

    let mut scored: Vec<(Memory, f32, f32)> = Vec::new();
    for memory in memories {
        if let Some(&similarity) = candidate_sim_map.get(&memory.id) {
            if similarity >= min_score {
                let boosted_score = apply_project_context_boost(&memory, similarity, config);
                scored.push((memory, boosted_score, similarity));
            }
        }
    }

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let results: Vec<SearchResult> = scored
        .into_iter()
        .take(limit as usize)
        .map(|(memory, boosted_score, original_score)| SearchResult {
            memory,
            score: boosted_score,
            match_info: MatchInfo {
                strategy: SearchStrategy::SemanticOnly,
                matched_terms: vec![],
                highlights: vec![],
                semantic_score: Some(original_score),
                keyword_score: None,
            },
        })
        .collect();

    Ok(results)
}

/// Semantic-only search using vector similarity
fn semantic_only_search(
    conn: &Connection,
    query_embedding: &[f32],
    limit: i64,
    min_score: f32,
    options: &SearchOptions,
    config: &SearchConfig,
) -> Result<Vec<SearchResult>> {
    let now = Utc::now().to_rfc3339();

    // NOTE: Columns must match what memory_from_row() reads. Missing columns
    // here trigger silent defaults (e.g. workspace -> "default", tier ->
    // "permanent", lifecycle_state -> Active) instead of panicking. See #10.
    let mut sql = String::from(
        "SELECT m.id, m.content, m.memory_type, m.importance, m.access_count,
                m.created_at, m.updated_at, m.last_accessed_at, m.owner_id,
                m.visibility, m.version, m.has_embedding, m.metadata,
                m.scope_type, m.scope_id, m.expires_at,
                m.workspace, m.tier, m.lifecycle_state
         FROM memories m
         WHERE m.has_embedding = 1 AND m.valid_to IS NULL
           AND (m.expires_at IS NULL OR m.expires_at > ?)",
    );

    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now)];

    // Exclude transcript chunks by default (unless include_transcripts is true)
    if !options.include_transcripts {
        sql.push_str(" AND m.memory_type != 'transcript_chunk'");
    }

    // Exclude archived memories unless include_archived is true
    if !options.include_archived {
        sql.push_str(" AND (m.lifecycle_state IS NULL OR m.lifecycle_state != 'archived')");
    }

    // Advanced filter (RML-932) - takes precedence over legacy tags/memory_type
    if let Some(ref filter_json) = options.filter {
        let filter_expr = parse_filter(filter_json)?;
        let mut builder = SqlBuilder::new();
        let filter_sql = builder.build_filter(&filter_expr)?;
        sql.push_str(" AND ");
        sql.push_str(&filter_sql);
        for param in builder.take_params() {
            params.push(param);
        }
    } else {
        // Legacy filters (deprecated, use `filter` instead)
        // Add tag filter if specified
        if let Some(ref tags) = options.tags {
            if !tags.is_empty() {
                sql.push_str(
                    " AND m.id IN (
                        SELECT mt.memory_id FROM memory_tags mt
                        JOIN tags t ON mt.tag_id = t.id
                        WHERE t.name IN (",
                );
                let placeholders: Vec<&str> = tags.iter().map(|_| "?").collect();
                sql.push_str(&placeholders.join(", "));
                sql.push_str("))");
                for tag in tags {
                    params.push(Box::new(tag.clone()));
                }
            }
        }

        // Add type filter
        if let Some(ref memory_type) = options.memory_type {
            sql.push_str(" AND m.memory_type = ?");
            params.push(Box::new(memory_type.as_str().to_string()));
        }
    }

    // Add scope filter (always applies, regardless of filter mode)
    if let Some(ref scope) = options.scope {
        sql.push_str(" AND m.scope_type = ?");
        params.push(Box::new(scope.scope_type().to_string()));
        if let Some(scope_id) = scope.scope_id() {
            sql.push_str(" AND m.scope_id = ?");
            params.push(Box::new(scope_id.to_string()));
        } else {
            sql.push_str(" AND m.scope_id IS NULL");
        }
    }

    // Add workspace filter (single or multiple).
    // When `global` is true, skip all workspace filters to search across all workspaces.
    if options.global {
        // No workspace filter applied — results span all workspaces.
    } else if let Some(ref workspace) = options.workspace {
        sql.push_str(" AND m.workspace = ?");
        params.push(Box::new(workspace.clone()));
    } else if let Some(ref workspaces) = options.workspaces {
        if !workspaces.is_empty() {
            let placeholders: Vec<&str> = workspaces.iter().map(|_| "?").collect();
            sql.push_str(&format!(
                " AND m.workspace IN ({})",
                placeholders.join(", ")
            ));
            for ws in workspaces {
                params.push(Box::new(ws.clone()));
            }
        }
    }

    // Add tier filter
    if let Some(ref tier) = options.tier {
        sql.push_str(&format!(" AND m.tier = '{}'", tier.as_str()));
    }

    // Add scope_path prefix filter for hierarchical scoping
    if let Some(ref sp) = options.scope_path {
        let escaped = sp.replace('%', "\\%").replace('_', "\\_");
        sql.push_str(" AND (m.scope_path = ? OR m.scope_path LIKE ? ESCAPE '\\')");
        params.push(Box::new(sp.clone()));
        params.push(Box::new(format!("{}/", escaped) + "%"));
    }

    let mut stmt = conn.prepare(&sql)?;

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();

    let memories: Vec<Memory> = stmt
        .query_map(param_refs.as_slice(), memory_from_row)?
        .filter_map(|r| r.ok())
        .map(|mut m| {
            m.tags = load_tags(conn, m.id).unwrap_or_default();
            m
        })
        .collect();

    // Calculate similarity scores with project context boost
    let mut scored: Vec<(Memory, f32, f32)> = Vec::new(); // (memory, boosted_score, original_score)
    for memory in memories {
        if let Ok(Some(embedding)) = get_embedding(conn, memory.id) {
            let similarity = cosine_similarity(query_embedding, &embedding);
            if similarity >= min_score {
                let boosted_score = apply_project_context_boost(&memory, similarity, config);
                scored.push((memory, boosted_score, similarity));
            }
        }
    }

    // Sort by boosted score descending
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let results: Vec<SearchResult> = scored
        .into_iter()
        .take(limit as usize)
        .map(|(memory, boosted_score, original_score)| SearchResult {
            memory,
            score: boosted_score,
            match_info: MatchInfo {
                strategy: SearchStrategy::SemanticOnly,
                matched_terms: vec![],
                highlights: vec![],
                semantic_score: Some(original_score),
                keyword_score: None,
            },
        })
        .collect();

    Ok(results)
}

/// Hybrid search using Reciprocal Rank Fusion with optional HNSW acceleration
fn rrf_hybrid_search_with_hnsw(
    conn: &Connection,
    query: &str,
    query_embedding: &[f32],
    limit: i64,
    min_score: f32,
    options: &SearchOptions,
    config: &SearchConfig,
    hnsw_index: Option<&crate::search::hnsw::HnswIndex<i64>>,
) -> Result<Vec<SearchResult>> {
    // When global=true, skip all workspace filters to search across all workspaces.
    let workspace_filter = if options.global {
        None
    } else {
        options.workspace.as_deref()
    };
    let workspaces_filter: Option<&[String]> = if options.global {
        None
    } else {
        options.workspaces.as_deref()
    };

    // Get keyword results (with all filters applied)
    let keyword_results = bm25_search_complete_with_scope_path(
        conn,
        query,
        limit * 2,
        options.explain,
        options.scope.as_ref(),
        options.filter.as_ref(),
        options.include_transcripts,
        options.include_archived,
        workspace_filter,
        workspaces_filter,
        options.tier.as_ref(),
        options.scope_path.as_deref(),
    )?;

    // Get semantic results (without boost - we'll apply it to the final RRF score)
    let semantic_options = SearchOptions {
        limit: Some(limit * 2),
        min_score: Some(0.0), // We'll filter after fusion
        scope: options.scope.clone(),
        filter: options.filter.clone(),
        include_transcripts: options.include_transcripts,
        include_archived: options.include_archived,
        workspace: options.workspace.clone(),
        workspaces: options.workspaces.clone(),
        tier: options.tier,
        scope_path: options.scope_path.clone(),
        global: options.global,
        ..Default::default()
    };
    // Create a config without project boost for sub-search (we'll apply boost to final RRF)
    let no_boost_config = SearchConfig {
        project_context_path: None,
        ..*config
    };
    let semantic_results = semantic_only_search_with_hnsw(
        conn,
        query_embedding,
        limit * 2,
        0.0,
        &semantic_options,
        &no_boost_config,
        hnsw_index,
    )?;

    // Build rank maps
    let mut keyword_ranks: HashMap<MemoryId, usize> = HashMap::new();
    let mut keyword_scores: HashMap<MemoryId, f32> = HashMap::new();
    for (rank, result) in keyword_results.iter().enumerate() {
        keyword_ranks.insert(result.memory.id, rank + 1);
        keyword_scores.insert(result.memory.id, result.score);
    }

    let mut semantic_ranks: HashMap<MemoryId, usize> = HashMap::new();
    let mut semantic_scores: HashMap<MemoryId, f32> = HashMap::new();
    for (rank, result) in semantic_results.iter().enumerate() {
        semantic_ranks.insert(result.memory.id, rank + 1);
        semantic_scores.insert(result.memory.id, result.score);
    }

    // Collect all unique memory IDs
    let mut all_ids: Vec<MemoryId> = keyword_ranks
        .keys()
        .chain(semantic_ranks.keys())
        .cloned()
        .collect();
    all_ids.sort();
    all_ids.dedup();

    // Calculate RRF scores
    let k = config.rrf_k;
    let mut rrf_scores: Vec<(MemoryId, f32, Option<f32>, Option<f32>)> = Vec::new();

    for id in all_ids {
        let keyword_contribution = keyword_ranks
            .get(&id)
            .map(|&rank| config.keyword_weight / (k + rank as f32))
            .unwrap_or(0.0);

        let semantic_contribution = semantic_ranks
            .get(&id)
            .map(|&rank| config.semantic_weight / (k + rank as f32))
            .unwrap_or(0.0);

        let rrf_score = keyword_contribution + semantic_contribution;

        if rrf_score >= min_score * 0.01 {
            // Adjusted threshold for RRF
            rrf_scores.push((
                id,
                rrf_score,
                keyword_scores.get(&id).copied(),
                semantic_scores.get(&id).copied(),
            ));
        }
    }

    // Sort by RRF score descending
    rrf_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Build final results with project context boost
    let mut results: Vec<SearchResult> = Vec::new();

    for (id, rrf_score, keyword_score, semantic_score) in rrf_scores.into_iter() {
        // Find memory from either result set
        let memory = keyword_results
            .iter()
            .find(|r| r.memory.id == id)
            .map(|r| r.memory.clone())
            .or_else(|| {
                semantic_results
                    .iter()
                    .find(|r| r.memory.id == id)
                    .map(|r| r.memory.clone())
            });

        if let Some(memory) = memory {
            // Apply project context boost to final RRF score
            let boosted_score = apply_project_context_boost(&memory, rrf_score, config);

            let matched_terms = if options.explain {
                keyword_results
                    .iter()
                    .find(|r| r.memory.id == id)
                    .map(|r| r.matched_terms.clone())
                    .unwrap_or_default()
            } else {
                vec![]
            };

            let highlights = if options.explain {
                keyword_results
                    .iter()
                    .find(|r| r.memory.id == id)
                    .map(|r| r.highlights.clone())
                    .unwrap_or_default()
            } else {
                vec![]
            };

            results.push(SearchResult {
                memory,
                score: boosted_score,
                match_info: MatchInfo {
                    strategy: SearchStrategy::Hybrid,
                    matched_terms,
                    highlights,
                    semantic_score,
                    keyword_score,
                },
            });
        }
    }

    // Re-sort after applying project context boost and truncate to requested limit
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    results.truncate(limit as usize);

    Ok(results)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_rrf_formula() {
        // RRF score = sum of 1/(k + rank) for each system
        let k = 60.0;
        let rank1 = 1;
        let rank2 = 5;

        let score1 = 1.0 / (k + rank1 as f32);
        let score2 = 1.0 / (k + rank2 as f32);

        // First rank should have higher score
        assert!(score1 > score2);
    }

    #[test]
    fn test_project_context_boost_multiplicative() {
        use super::*;
        let memory = Memory {
            id: 1,
            content: "Some content".to_string(),
            memory_type: crate::types::MemoryType::Note,
            importance: 0.5,
            tags: vec!["project-context".to_string()],
            access_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed_at: None,
            owner_id: None,
            visibility: crate::types::Visibility::Private,
            version: 1,
            has_embedding: false,
            metadata: {
                let mut m = HashMap::new();
                m.insert(
                    "project_path".to_string(),
                    serde_json::json!("/path/to/repo"),
                );
                m
            },
            scope: crate::types::MemoryScope::Global,
            workspace: "default".to_string(),
            tier: crate::types::MemoryTier::Permanent,
            expires_at: None,
            content_hash: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            procedure_success_count: 0,
            procedure_failure_count: 0,
            summary_of_id: None,
            lifecycle_state: crate::types::LifecycleState::Active,
            stability: 1.0,
            media_url: None,
        };

        let config = SearchConfig {
            project_context_path: Some("/path/to/repo".to_string()),
            project_context_boost: 0.2, // 20% boost
            ..Default::default()
        };

        let base_score = 0.0164;
        let boosted = apply_project_context_boost(&memory, base_score, &config);

        // Boosted score should be base_score * 1.2
        let expected = base_score * 1.2;
        assert!((boosted - expected).abs() < 1e-6);
        assert!(boosted > base_score);
        assert!(boosted < base_score * 1.5);
    }
}
