//! Database queries for memory operations

use crate::storage::queries::sync::{record_event, MemoryEventType};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::error::{EngramError, Result};
use crate::storage::filter::{parse_filter, SqlBuilder};
use crate::types::*;

/// Parse a memory from a database row
pub fn memory_from_row(row: &Row) -> rusqlite::Result<Memory> {
    let id: i64 = row.get("id")?;
    let content: String = row.get("content")?;
    let memory_type_str: String = row.get("memory_type")?;
    let importance: f32 = row.get("importance")?;
    let access_count: i32 = row.get("access_count")?;
    let created_at: String = row.get("created_at")?;
    let updated_at: String = row.get("updated_at")?;
    let last_accessed_at: Option<String> = row.get("last_accessed_at")?;
    let owner_id: Option<String> = row.get("owner_id")?;
    let visibility_str: String = row.get("visibility")?;
    let version: i32 = row.get("version")?;
    let has_embedding: i32 = row.get("has_embedding")?;
    let metadata_str: String = row.get("metadata")?;

    // Scope columns (with fallback for backward compatibility)
    let scope_type: String = row
        .get("scope_type")
        .unwrap_or_else(|_| "global".to_string());
    let scope_id: Option<String> = row.get("scope_id").unwrap_or(None);

    // TTL column (with fallback for backward compatibility)
    let expires_at: Option<String> = row.get("expires_at").unwrap_or(None);

    // Content hash column (with fallback for backward compatibility)
    let content_hash: Option<String> = row.get("content_hash").unwrap_or(None);

    let memory_type = memory_type_str.parse().unwrap_or(MemoryType::Note);
    let visibility = match visibility_str.as_str() {
        "shared" => Visibility::Shared,
        "public" => Visibility::Public,
        _ => Visibility::Private,
    };

    // Parse scope from type and id
    let scope = match (scope_type.as_str(), scope_id) {
        ("user", Some(id)) => MemoryScope::User { user_id: id },
        ("session", Some(id)) => MemoryScope::Session { session_id: id },
        ("agent", Some(id)) => MemoryScope::Agent { agent_id: id },
        _ => MemoryScope::Global,
    };

    let metadata: HashMap<String, serde_json::Value> =
        serde_json::from_str(&metadata_str).unwrap_or_default();

    // Workspace column (with fallback for backward compatibility)
    let workspace: String = row
        .get("workspace")
        .unwrap_or_else(|_| "default".to_string());

    // Tier column (with fallback for backward compatibility)
    let tier_str: String = row.get("tier").unwrap_or_else(|_| "permanent".to_string());
    let tier = tier_str.parse().unwrap_or_default();

    let event_time: Option<String> = row.get("event_time").unwrap_or(None);
    let event_duration_seconds: Option<i64> = row.get("event_duration_seconds").unwrap_or(None);
    let trigger_pattern: Option<String> = row.get("trigger_pattern").unwrap_or(None);
    let procedure_success_count: i32 = row.get("procedure_success_count").unwrap_or(0);
    let procedure_failure_count: i32 = row.get("procedure_failure_count").unwrap_or(0);
    let summary_of_id: Option<i64> = row.get("summary_of_id").unwrap_or(None);
    let lifecycle_state_str: Option<String> = row.get("lifecycle_state").unwrap_or(None);

    let lifecycle_state = lifecycle_state_str
        .and_then(|s| s.parse().ok())
        .unwrap_or(crate::types::LifecycleState::Active);

    // media_url column (additive, nullable — with fallback for older schema versions)
    let media_url: Option<String> = row.get("media_url").unwrap_or(None);

    Ok(Memory {
        id,
        content,
        memory_type,
        tags: vec![], // Loaded separately
        metadata,
        importance,
        access_count,
        created_at: DateTime::parse_from_rfc3339(&created_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        updated_at: DateTime::parse_from_rfc3339(&updated_at)
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        last_accessed_at: last_accessed_at.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        }),
        owner_id,
        visibility,
        scope,
        workspace,
        tier,
        version,
        has_embedding: has_embedding != 0,
        expires_at: expires_at.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        }),
        content_hash,
        event_time: event_time.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        }),
        event_duration_seconds,
        trigger_pattern,
        procedure_success_count,
        procedure_failure_count,
        summary_of_id,
        lifecycle_state,
        media_url,
    })
}

pub(crate) fn metadata_value_to_param(
    key: &str,
    value: &serde_json::Value,
    conditions: &mut Vec<String>,
    params: &mut Vec<Box<dyn rusqlite::ToSql>>,
) -> Result<()> {
    match value {
        serde_json::Value::String(s) => {
            conditions.push(format!("json_extract(m.metadata, '$.{}') = ?", key));
            params.push(Box::new(s.clone()));
        }
        serde_json::Value::Number(n) => {
            conditions.push(format!("json_extract(m.metadata, '$.{}') = ?", key));
            if let Some(i) = n.as_i64() {
                params.push(Box::new(i));
            } else if let Some(f) = n.as_f64() {
                params.push(Box::new(f));
            } else {
                return Err(EngramError::InvalidInput("Invalid number".to_string()));
            }
        }
        serde_json::Value::Bool(b) => {
            conditions.push(format!("json_extract(m.metadata, '$.{}') = ?", key));
            params.push(Box::new(*b));
        }
        serde_json::Value::Null => {
            conditions.push(format!("json_extract(m.metadata, '$.{}') IS NULL", key));
        }
        _ => {
            return Err(EngramError::InvalidInput(format!(
                "Unsupported metadata filter value for key: {}",
                key
            )));
        }
    }

    Ok(())
}

fn get_memory_internal(conn: &Connection, id: i64, track_access: bool) -> Result<Memory> {
    let now = Utc::now().to_rfc3339();

    let mut stmt = conn.prepare_cached(
        "SELECT id, content, memory_type, importance, access_count,
                created_at, updated_at, last_accessed_at, owner_id,
                visibility, version, has_embedding, metadata,
                scope_type, scope_id, workspace, tier, expires_at, content_hash,
                event_time, event_duration_seconds, trigger_pattern, procedure_success_count,
                procedure_failure_count, summary_of_id, lifecycle_state, media_url
         FROM memories
         WHERE id = ? AND valid_to IS NULL
           AND (expires_at IS NULL OR expires_at > ?)",
    )?;

    let mut memory = stmt
        .query_row(params![id, now], memory_from_row)
        .map_err(|_| EngramError::NotFound(id))?;

    memory.tags = load_tags(conn, id)?;

    if track_access {
        // Update access tracking
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE memories SET access_count = access_count + 1, last_accessed_at = ?
             WHERE id = ?",
            params![now, id],
        )?;
    }

    Ok(memory)
}

/// Load tags for a memory
pub fn load_tags(conn: &Connection, memory_id: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare_cached(
        "SELECT t.name FROM tags t
         JOIN memory_tags mt ON t.id = mt.tag_id
         WHERE mt.memory_id = ?",
    )?;

    let tags: Vec<String> = stmt
        .query_map([memory_id], |row| row.get(0))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(tags)
}

/// Compute the normalized SHA-256 hash used for deduplication and persistence checks.
///
/// Normalization lowercases and collapses whitespace, so content equivalent under
/// dedup rules maps to the same hash.
pub fn compute_content_hash(content: &str) -> String {
    let normalized = content
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

/// Compute the raw SHA-256 hash of content bytes without any normalization.
///
/// Used for sync detection where case and whitespace differences must be preserved
/// (e.g. detecting case-only edits in markdown import/export).
pub fn compute_content_hash_raw(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

/// Compute a dedupe hash.
///
/// Kept as an explicit semantic alias for existing callers; currently it delegates
/// to [`compute_content_hash`].
pub fn compute_dedup_hash(content: &str) -> String {
    compute_content_hash(content)
}

/// Find a memory by content hash within the same scope and workspace (exact duplicate detection)
///
/// Deduplication respects both scope and workspace isolation:
/// - User-scoped memories only dedupe against other memories with same user_id
/// - Session-scoped memories only dedupe against other memories with same session_id
/// - Global memories only dedupe against other global memories
/// - All deduplication is workspace-scoped (memories in different workspaces are never duplicates)
pub fn find_by_content_hash(
    conn: &Connection,
    content_hash: &str,
    scope: &MemoryScope,
    workspace: Option<&str>,
) -> Result<Option<Memory>> {
    let now = Utc::now().to_rfc3339();
    let scope_type = scope.scope_type();
    let scope_id = scope.scope_id().map(|s| s.to_string());
    let workspace = workspace.unwrap_or("default");

    let mut stmt = conn.prepare_cached(
        "SELECT id, content, memory_type, importance, access_count,
                created_at, updated_at, last_accessed_at, owner_id,
                visibility, version, has_embedding, metadata,
                scope_type, scope_id, workspace, tier, expires_at, content_hash,
                event_time, event_duration_seconds, trigger_pattern, procedure_success_count,
                procedure_failure_count, summary_of_id, lifecycle_state, media_url
         FROM memories
         WHERE content_hash = ? AND valid_to IS NULL
           AND (expires_at IS NULL OR expires_at > ?)
           AND scope_type = ?
           AND (scope_id = ? OR (scope_id IS NULL AND ? IS NULL))
           AND workspace = ?
         LIMIT 1",
    )?;

    let result = stmt
        .query_row(
            params![content_hash, now, scope_type, scope_id, scope_id, workspace],
            memory_from_row,
        )
        .ok();

    if let Some(mut memory) = result {
        memory.tags = load_tags(conn, memory.id)?;
        Ok(Some(memory))
    } else {
        Ok(None)
    }
}

/// Find the most similar memory to given embedding within the same scope AND workspace (semantic duplicate detection)
///
/// Returns the memory with the highest similarity score if it meets the threshold.
/// Only checks memories that have embeddings computed.
pub fn find_similar_by_embedding(
    conn: &Connection,
    query_embedding: &[f32],
    scope: &MemoryScope,
    workspace: Option<&str>,
    threshold: f32,
) -> Result<Option<(Memory, f32)>> {
    use crate::embedding::{cosine_similarity, get_embedding};

    let now = Utc::now().to_rfc3339();
    let scope_type = scope.scope_type();
    let scope_id = scope.scope_id().map(|s| s.to_string());
    let workspace = workspace.unwrap_or("default");

    // Get all memories with embeddings in the same scope AND workspace
    let mut stmt = conn.prepare_cached(
        "SELECT id, content, memory_type, importance, access_count,
                created_at, updated_at, last_accessed_at, owner_id,
                visibility, version, has_embedding, metadata,
                scope_type, scope_id, workspace, tier, expires_at, content_hash,
                event_time, event_duration_seconds, trigger_pattern, procedure_success_count,
                procedure_failure_count, summary_of_id, lifecycle_state, media_url
         FROM memories
         WHERE has_embedding = 1 AND valid_to IS NULL
           AND (expires_at IS NULL OR expires_at > ?)
           AND scope_type = ?
           AND (scope_id = ? OR (scope_id IS NULL AND ? IS NULL))
           AND workspace = ?",
    )?;

    let memories: Vec<Memory> = stmt
        .query_map(
            params![now, scope_type, scope_id, scope_id, workspace],
            memory_from_row,
        )?
        .filter_map(|r| r.ok())
        .collect();

    let mut best_match: Option<(Memory, f32)> = None;

    for memory in memories {
        if let Ok(Some(embedding)) = get_embedding(conn, memory.id) {
            let similarity = cosine_similarity(query_embedding, &embedding);
            if similarity >= threshold {
                match &best_match {
                    None => best_match = Some((memory, similarity)),
                    Some((_, best_score)) if similarity > *best_score => {
                        best_match = Some((memory, similarity));
                    }
                    _ => {}
                }
            }
        }
    }

    // Load tags for the best match
    if let Some((mut memory, score)) = best_match {
        memory.tags = load_tags(conn, memory.id)?;
        Ok(Some((memory, score)))
    } else {
        Ok(None)
    }
}

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

/// Find all potential duplicate memory pairs
///
/// Returns pairs of memories that are either:
/// 1. Exact duplicates (same content hash within same scope)
/// 2. High similarity (crossref score >= threshold within same scope)
///
/// Duplicates are scoped - memories in different scopes are not considered duplicates.
pub fn find_duplicates(conn: &Connection, threshold: f64) -> Result<Vec<DuplicatePair>> {
    find_duplicates_in_workspace(conn, threshold, None)
}

/// Find duplicate memories within a specific workspace (or all if None)
pub fn find_duplicates_in_workspace(
    conn: &Connection,
    threshold: f64,
    workspace: Option<&str>,
) -> Result<Vec<DuplicatePair>> {
    let now = Utc::now().to_rfc3339();
    let mut duplicates = Vec::new();

    // First, find exact hash duplicates (same content_hash within same scope AND workspace)
    let (hash_sql, hash_params): (&str, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(ws) = workspace
    {
        (
            "SELECT content_hash, scope_type, scope_id, GROUP_CONCAT(id) as ids
             FROM memories
             WHERE content_hash IS NOT NULL
               AND valid_to IS NULL
               AND (expires_at IS NULL OR expires_at > ?)
               AND workspace = ?
             GROUP BY content_hash, scope_type, scope_id, workspace
             HAVING COUNT(*) > 1",
            vec![Box::new(now.clone()), Box::new(ws.to_string())],
        )
    } else {
        (
            "SELECT content_hash, scope_type, scope_id, GROUP_CONCAT(id) as ids
             FROM memories
             WHERE content_hash IS NOT NULL
               AND valid_to IS NULL
               AND (expires_at IS NULL OR expires_at > ?)
             GROUP BY content_hash, scope_type, scope_id, workspace
             HAVING COUNT(*) > 1",
            vec![Box::new(now.clone())],
        )
    };

    let mut hash_stmt = conn.prepare_cached(hash_sql)?;
    let hash_rows = hash_stmt.query_map(
        rusqlite::params_from_iter(hash_params.iter().map(|p| p.as_ref())),
        |row| {
            let ids_str: String = row.get(3)?;
            Ok(ids_str)
        },
    )?;

    for ids_result in hash_rows {
        let ids_str = ids_result?;
        let ids: Vec<i64> = ids_str
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();

        // Create pairs from all IDs with same hash
        // Use get_memory_internal with track_access=false to avoid inflating access stats
        for i in 0..ids.len() {
            for j in (i + 1)..ids.len() {
                let memory_a = get_memory_internal(conn, ids[i], false)?;
                let memory_b = get_memory_internal(conn, ids[j], false)?;
                duplicates.push(DuplicatePair {
                    memory_a,
                    memory_b,
                    similarity_score: 1.0, // Exact match
                    match_type: DuplicateMatchType::ExactHash,
                });
            }
        }
    }

    // Second, find high-similarity pairs from crossrefs (within same scope AND workspace)
    let (sim_sql, sim_params): (&str, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(ws) = workspace {
        (
            "SELECT DISTINCT c.from_id, c.to_id, c.score
             FROM crossrefs c
             JOIN memories m1 ON c.from_id = m1.id
             JOIN memories m2 ON c.to_id = m2.id
             WHERE c.score >= ?
               AND m1.valid_to IS NULL
               AND m2.valid_to IS NULL
               AND (m1.expires_at IS NULL OR m1.expires_at > ?)
               AND (m2.expires_at IS NULL OR m2.expires_at > ?)
               AND c.from_id < c.to_id
               AND m1.scope_type = m2.scope_type
               AND (m1.scope_id = m2.scope_id OR (m1.scope_id IS NULL AND m2.scope_id IS NULL))
               AND m1.workspace = ?
               AND m2.workspace = ?
             ORDER BY c.score DESC",
            vec![
                Box::new(threshold),
                Box::new(now.clone()),
                Box::new(now.clone()),
                Box::new(ws.to_string()),
                Box::new(ws.to_string()),
            ],
        )
    } else {
        (
            "SELECT DISTINCT c.from_id, c.to_id, c.score
             FROM crossrefs c
             JOIN memories m1 ON c.from_id = m1.id
             JOIN memories m2 ON c.to_id = m2.id
             WHERE c.score >= ?
               AND m1.valid_to IS NULL
               AND m2.valid_to IS NULL
               AND (m1.expires_at IS NULL OR m1.expires_at > ?)
               AND (m2.expires_at IS NULL OR m2.expires_at > ?)
               AND c.from_id < c.to_id
               AND m1.scope_type = m2.scope_type
               AND (m1.scope_id = m2.scope_id OR (m1.scope_id IS NULL AND m2.scope_id IS NULL))
               AND m1.workspace = m2.workspace
             ORDER BY c.score DESC",
            vec![
                Box::new(threshold),
                Box::new(now.clone()),
                Box::new(now.clone()),
            ],
        )
    };

    let mut sim_stmt = conn.prepare_cached(sim_sql)?;
    let sim_rows = sim_stmt.query_map(
        rusqlite::params_from_iter(sim_params.iter().map(|p| p.as_ref())),
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, f64>(2)?,
            ))
        },
    )?;

    for row_result in sim_rows {
        let (from_id, to_id, score) = row_result?;

        // Skip if this pair was already found as exact hash match
        let already_found = duplicates.iter().any(|d| {
            (d.memory_a.id == from_id && d.memory_b.id == to_id)
                || (d.memory_a.id == to_id && d.memory_b.id == from_id)
        });

        if !already_found {
            // Use get_memory_internal with track_access=false to avoid inflating access stats
            let memory_a = get_memory_internal(conn, from_id, false)?;
            let memory_b = get_memory_internal(conn, to_id, false)?;
            duplicates.push(DuplicatePair {
                memory_a,
                memory_b,
                similarity_score: score,
                match_type: DuplicateMatchType::HighSimilarity,
            });
        }
    }

    Ok(duplicates)
}

/// Find semantically similar memories using embedding cosine similarity.
/// This is "LLM-powered" dedup — goes beyond hash/n-gram matching to detect
/// memories that convey the same information with different wording.
pub fn find_duplicates_by_embedding(
    conn: &Connection,
    threshold: f32,
    workspace: Option<&str>,
    limit: usize,
) -> Result<Vec<DuplicatePair>> {
    use crate::embedding::{cosine_similarity, get_embedding};

    let now = Utc::now().to_rfc3339();

    // Get all memory IDs with embeddings (scoped to workspace if provided)
    let (sql, params_vec): (&str, Vec<Box<dyn rusqlite::ToSql>>) = if let Some(ws) = workspace {
        (
            "SELECT id FROM memories
             WHERE has_embedding = 1 AND valid_to IS NULL
               AND (expires_at IS NULL OR expires_at > ?)
               AND COALESCE(lifecycle_state, 'active') = 'active'
               AND workspace = ?
             ORDER BY id",
            vec![Box::new(now), Box::new(ws.to_string())],
        )
    } else {
        (
            "SELECT id FROM memories
             WHERE has_embedding = 1 AND valid_to IS NULL
               AND (expires_at IS NULL OR expires_at > ?)
               AND COALESCE(lifecycle_state, 'active') = 'active'
             ORDER BY id",
            vec![Box::new(now)],
        )
    };

    let mut stmt = conn.prepare(sql)?;
    let ids: Vec<i64> = stmt
        .query_map(
            rusqlite::params_from_iter(params_vec.iter().map(|p| p.as_ref())),
            |row| row.get(0),
        )?
        .filter_map(|r| r.ok())
        .collect();

    // Load all embeddings into memory for pairwise comparison
    let mut embeddings: Vec<(i64, Vec<f32>)> = Vec::with_capacity(ids.len());
    for &id in &ids {
        if let Ok(Some(emb)) = get_embedding(conn, id) {
            embeddings.push((id, emb));
        }
    }

    let mut duplicates = Vec::new();

    // Pairwise comparison (O(n^2) — bounded by limit)
    for i in 0..embeddings.len() {
        if duplicates.len() >= limit {
            break;
        }
        for j in (i + 1)..embeddings.len() {
            if duplicates.len() >= limit {
                break;
            }
            let sim = cosine_similarity(&embeddings[i].1, &embeddings[j].1);
            if sim >= threshold {
                let memory_a = get_memory_internal(conn, embeddings[i].0, false)?;
                let memory_b = get_memory_internal(conn, embeddings[j].0, false)?;
                duplicates.push(DuplicatePair {
                    memory_a,
                    memory_b,
                    similarity_score: sim as f64,
                    match_type: DuplicateMatchType::EmbeddingSimilarity,
                });
            }
        }
    }

    // Sort by similarity descending
    duplicates.sort_by(|a, b| {
        b.similarity_score
            .partial_cmp(&a.similarity_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(duplicates)
}

/// Create a new memory with deduplication support
pub fn create_memory(conn: &Connection, input: &CreateMemoryInput) -> Result<Memory> {
    let now = Utc::now();
    let now_str = now.to_rfc3339();
    let metadata_json = serde_json::to_string(&input.metadata)?;
    let importance = input.importance.unwrap_or(0.5);

    // Compute dedup hash for duplicate detection (normalized: lowercased, whitespace-collapsed)
    // This is also the stored `content_hash`, so import/export must use the same function.
    let content_hash = compute_content_hash(&input.content);

    // Normalize workspace early for dedup checking
    let workspace = match &input.workspace {
        Some(ws) => crate::types::normalize_workspace(ws)
            .map_err(|e| EngramError::InvalidInput(format!("Invalid workspace: {}", e)))?,
        None => "default".to_string(),
    };

    // Check for duplicates based on dedup_mode (scoped to same scope AND workspace)
    if input.dedup_mode != DedupMode::Allow {
        if let Some(existing) =
            find_by_content_hash(conn, &content_hash, &input.scope, Some(&workspace))?
        {
            match input.dedup_mode {
                DedupMode::Reject => {
                    return Err(EngramError::Duplicate {
                        existing_id: existing.id,
                        message: format!(
                            "Duplicate memory detected (id={}). Content hash: {}",
                            existing.id, content_hash
                        ),
                    });
                }
                DedupMode::Skip => {
                    // Return existing memory without modification
                    return Ok(existing);
                }
                DedupMode::Merge => {
                    // Merge: update existing memory with new tags and metadata
                    let mut merged_tags = existing.tags.clone();
                    for tag in &input.tags {
                        if !merged_tags.contains(tag) {
                            merged_tags.push(tag.clone());
                        }
                    }

                    let mut merged_metadata = existing.metadata.clone();
                    for (key, value) in &input.metadata {
                        merged_metadata.insert(key.clone(), value.clone());
                    }

                    let update_input = UpdateMemoryInput {
                        content: None, // Keep existing content
                        memory_type: None,
                        tags: Some(merged_tags),
                        metadata: Some(merged_metadata),
                        importance: input.importance, // Use new importance if provided
                        scope: None,
                        ttl_seconds: input.ttl_seconds, // Apply new TTL if provided
                        event_time: None,
                        trigger_pattern: None,
                        media_url: input.media_url.clone().map(Some),
                    };

                    return update_memory(conn, existing.id, &update_input);
                }
                DedupMode::Allow => unreachable!(),
            }
        }
    }

    // Extract scope type and id for database storage
    let scope_type = input.scope.scope_type();
    let scope_id = input.scope.scope_id().map(|s| s.to_string());

    // workspace was already normalized above for dedup checking

    // Determine tier and enforce tier invariants
    let tier = input.tier;

    // Calculate expires_at based on tier and ttl_seconds
    // Tier invariants:
    //   - Permanent: expires_at MUST be NULL (cannot expire)
    //   - Daily: expires_at MUST be set (default: created_at + 24h)
    let expires_at = match tier {
        MemoryTier::Permanent => {
            // Permanent memories cannot have an expiration
            if input.ttl_seconds.is_some() && input.ttl_seconds != Some(0) {
                return Err(EngramError::InvalidInput(
                    "Permanent tier memories cannot have a TTL. Use Daily tier for expiring memories.".to_string()
                ));
            }
            None
        }
        MemoryTier::Daily => {
            // Daily memories must have an expiration (default: 24 hours)
            let ttl = input.ttl_seconds.filter(|&t| t > 0).unwrap_or(86400); // 24h default
            Some((now + chrono::Duration::seconds(ttl)).to_rfc3339())
        }
    };

    let event_time = input.event_time.map(|dt| dt.to_rfc3339());

    conn.execute(
        "INSERT INTO memories (content, memory_type, importance, metadata, created_at, updated_at, valid_from, scope_type, scope_id, workspace, tier, expires_at, content_hash, event_time, event_duration_seconds, trigger_pattern, summary_of_id, media_url)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            input.content,
            input.memory_type.as_str(),
            importance,
            metadata_json,
            now_str,
            now_str,
            now_str,
            scope_type,
            scope_id,
            workspace,
            tier.as_str(),
            expires_at,
            content_hash,
            event_time,
            input.event_duration_seconds,
            input.trigger_pattern,
            input.summary_of_id,
            input.media_url,
        ],
    )?;

    let id = conn.last_insert_rowid();

    // Insert tags
    for tag in &input.tags {
        ensure_tag(conn, tag)?;
        conn.execute(
            "INSERT OR IGNORE INTO memory_tags (memory_id, tag_id)
             SELECT ?, id FROM tags WHERE name = ?",
            params![id, tag],
        )?;
    }

    // Queue for embedding if not deferred
    if !input.defer_embedding {
        conn.execute(
            "INSERT INTO embedding_queue (memory_id, status, queued_at)
             VALUES (?, 'pending', ?)",
            params![id, now_str],
        )?;
    }

    // Create initial version
    let tags_json = serde_json::to_string(&input.tags)?;
    conn.execute(
        "INSERT INTO memory_versions (memory_id, version, content, tags, metadata, created_at)
         VALUES (?, 1, ?, ?, ?, ?)",
        params![id, input.content, tags_json, metadata_json, now_str],
    )?;

    // Record event for sync delta tracking
    record_event(
        conn,
        MemoryEventType::Created,
        Some(id),
        None,
        serde_json::json!({
            "workspace": input.workspace.as_deref().unwrap_or("default"),
            "memory_type": input.memory_type.as_str(),
        }),
    )?;

    // Update sync state (version now tracks event count for delta sync)
    conn.execute(
        "UPDATE sync_state SET pending_changes = pending_changes + 1, version = (SELECT MAX(id) FROM memory_events) WHERE id = 1",
        [],
    )?;

    get_memory_internal(conn, id, false)
}

/// Ensure a tag exists and return its ID
fn ensure_tag(conn: &Connection, tag: &str) -> Result<i64> {
    conn.execute("INSERT OR IGNORE INTO tags (name) VALUES (?)", params![tag])?;

    let id: i64 = conn.query_row("SELECT id FROM tags WHERE name = ?", params![tag], |row| {
        row.get(0)
    })?;

    Ok(id)
}

/// Get a memory by ID
pub fn get_memory(conn: &Connection, id: i64) -> Result<Memory> {
    get_memory_internal(conn, id, true)
}

/// Update a memory
pub fn update_memory(conn: &Connection, id: i64, input: &UpdateMemoryInput) -> Result<Memory> {
    // Get current memory for versioning
    let current = get_memory_internal(conn, id, false)?;
    let now = Utc::now().to_rfc3339();

    // Build update query dynamically
    let mut updates = vec!["updated_at = ?".to_string()];
    let mut values: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now.clone())];

    if let Some(ref content) = input.content {
        updates.push("content = ?".to_string());
        values.push(Box::new(content.clone()));
        // Recalculate dedup hash when content changes
        // (dedup uses normalized hash so case/whitespace-only edits don't create duplicates)
        let new_hash = compute_dedup_hash(content);
        updates.push("content_hash = ?".to_string());
        values.push(Box::new(new_hash));
    }

    if let Some(ref memory_type) = input.memory_type {
        updates.push("memory_type = ?".to_string());
        values.push(Box::new(memory_type.as_str().to_string()));
    }

    if let Some(importance) = input.importance {
        updates.push("importance = ?".to_string());
        values.push(Box::new(importance));
    }

    if let Some(ref metadata) = input.metadata {
        let metadata_json = serde_json::to_string(metadata)?;
        updates.push("metadata = ?".to_string());
        values.push(Box::new(metadata_json));
    }

    if let Some(ref scope) = input.scope {
        updates.push("scope_type = ?".to_string());
        values.push(Box::new(scope.scope_type().to_string()));
        updates.push("scope_id = ?".to_string());
        values.push(Box::new(scope.scope_id().map(|s| s.to_string())));
    }

    // Update event_time if provided (Some(None) clears)
    if let Some(event_time) = &input.event_time {
        updates.push("event_time = ?".to_string());
        let value = event_time.as_ref().map(|dt| dt.to_rfc3339());
        values.push(Box::new(value));
    }

    // Update trigger_pattern if provided (Some(None) clears)
    if let Some(trigger_pattern) = &input.trigger_pattern {
        updates.push("trigger_pattern = ?".to_string());
        values.push(Box::new(trigger_pattern.clone()));
    }

    // Update media_url if provided (Some(None) clears, Some(Some(url)) sets)
    if let Some(media_url) = &input.media_url {
        updates.push("media_url = ?".to_string());
        values.push(Box::new(media_url.clone()));
    }

    // Handle TTL update with tier invariant enforcement
    // Normalize: ttl_seconds <= 0 means "no expiration" (consistent with create_memory)
    // Invariants:
    //   - Permanent tier: expires_at MUST be NULL
    //   - Daily tier: expires_at MUST be set
    if let Some(ttl) = input.ttl_seconds {
        if ttl <= 0 {
            // Request to remove expiration
            // Only allowed for Permanent tier; for Daily tier, this is an error
            if current.tier == MemoryTier::Daily {
                return Err(crate::error::EngramError::InvalidInput(
                    "Cannot remove expiration from a Daily tier memory. Use promote_to_permanent first.".to_string()
                ));
            }
            updates.push("expires_at = NULL".to_string());
        } else {
            // Request to set expiration
            // Only allowed for Daily tier; for Permanent tier, this is an error
            if current.tier == MemoryTier::Permanent {
                return Err(crate::error::EngramError::InvalidInput(
                    "Cannot set expiration on a Permanent tier memory. Permanent memories cannot expire.".to_string()
                ));
            }
            let expires_at = (Utc::now() + chrono::Duration::seconds(ttl)).to_rfc3339();
            updates.push("expires_at = ?".to_string());
            values.push(Box::new(expires_at));
        }
    }

    // Increment version
    updates.push("version = version + 1".to_string());

    // Execute update
    let sql = format!("UPDATE memories SET {} WHERE id = ?", updates.join(", "));
    values.push(Box::new(id));

    let params: Vec<&dyn rusqlite::ToSql> = values.iter().map(|b| b.as_ref()).collect();
    conn.execute(&sql, params.as_slice())?;

    // Update tags if provided
    if let Some(ref tags) = input.tags {
        conn.execute("DELETE FROM memory_tags WHERE memory_id = ?", params![id])?;
        for tag in tags {
            ensure_tag(conn, tag)?;
            conn.execute(
                "INSERT OR IGNORE INTO memory_tags (memory_id, tag_id)
                 SELECT ?, id FROM tags WHERE name = ?",
                params![id, tag],
            )?;
        }
    }

    // Create new version
    let new_content = input.content.as_ref().unwrap_or(&current.content);
    let new_tags = input.tags.as_ref().unwrap_or(&current.tags);
    let new_metadata = input.metadata.as_ref().unwrap_or(&current.metadata);
    let tags_json = serde_json::to_string(new_tags)?;
    let metadata_json = serde_json::to_string(new_metadata)?;

    conn.execute(
        "INSERT INTO memory_versions (memory_id, version, content, tags, metadata, created_at)
         VALUES (?, (SELECT version FROM memories WHERE id = ?), ?, ?, ?, ?)",
        params![id, id, new_content, tags_json, metadata_json, now],
    )?;

    // Re-queue for embedding if content changed
    if input.content.is_some() {
        conn.execute(
            "INSERT OR REPLACE INTO embedding_queue (memory_id, status, queued_at)
             VALUES (?, 'pending', ?)",
            params![id, now],
        )?;
        conn.execute(
            "UPDATE memories SET has_embedding = 0 WHERE id = ?",
            params![id],
        )?;
    }

    // Build list of changed fields for event data
    let mut changed_fields = Vec::new();
    if input.content.is_some() {
        changed_fields.push("content");
    }
    if input.tags.is_some() {
        changed_fields.push("tags");
    }
    if input.metadata.is_some() {
        changed_fields.push("metadata");
    }
    if input.importance.is_some() {
        changed_fields.push("importance");
    }
    if input.ttl_seconds.is_some() {
        changed_fields.push("ttl");
    }

    // Record event for sync delta tracking
    record_event(
        conn,
        MemoryEventType::Updated,
        Some(id),
        None,
        serde_json::json!({
            "changed_fields": changed_fields,
        }),
    )?;

    // Update sync state (version now tracks event count for delta sync)
    conn.execute(
        "UPDATE sync_state SET pending_changes = pending_changes + 1, version = (SELECT MAX(id) FROM memory_events) WHERE id = 1",
        [],
    )?;

    get_memory_internal(conn, id, false)
}

/// Promote a memory from Daily tier to Permanent tier.
///
/// This operation:
/// - Changes the tier from Daily to Permanent
/// - Clears the expires_at field (permanent memories cannot expire)
/// - Updates the updated_at timestamp
///
/// # Errors
/// - Returns `NotFound` if memory doesn't exist
/// - Returns `Validation` if memory is already Permanent
pub fn promote_to_permanent(conn: &Connection, id: i64) -> Result<Memory> {
    let memory = get_memory_internal(conn, id, false)?;

    if memory.tier == MemoryTier::Permanent {
        return Err(EngramError::InvalidInput(format!(
            "Memory {} is already in the Permanent tier",
            id
        )));
    }

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE memories SET tier = 'permanent', expires_at = NULL, updated_at = ?, version = version + 1 WHERE id = ?",
        params![now, id],
    )?;

    // Record event for sync delta tracking
    record_event(
        conn,
        MemoryEventType::Updated,
        Some(id),
        None,
        serde_json::json!({
            "changed_fields": ["tier", "expires_at"],
            "action": "promote_to_permanent",
        }),
    )?;

    // Update sync state (version now tracks event count for delta sync)
    conn.execute(
        "UPDATE sync_state SET pending_changes = pending_changes + 1, version = (SELECT MAX(id) FROM memory_events) WHERE id = 1",
        [],
    )?;

    tracing::info!(memory_id = id, "Promoted memory to permanent tier");

    get_memory_internal(conn, id, false)
}

/// Move a memory to a different workspace.
///
/// # Arguments
/// - `id`: Memory ID
/// - `workspace`: New workspace name (will be normalized)
///
/// # Errors
/// - Returns `NotFound` if memory doesn't exist
/// - Returns `Validation` if workspace name is invalid
pub fn move_to_workspace(conn: &Connection, id: i64, workspace: &str) -> Result<Memory> {
    // Validate workspace exists (by checking the memory exists first)
    let _memory = get_memory_internal(conn, id, false)?;

    // Normalize the workspace name
    let normalized = crate::types::normalize_workspace(workspace)
        .map_err(|e| EngramError::InvalidInput(format!("Invalid workspace: {}", e)))?;

    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE memories SET workspace = ?, updated_at = ?, version = version + 1 WHERE id = ?",
        params![normalized, now, id],
    )?;

    // Record event for sync delta tracking
    record_event(
        conn,
        MemoryEventType::Updated,
        Some(id),
        None,
        serde_json::json!({
            "changed_fields": ["workspace"],
            "action": "move_to_workspace",
            "new_workspace": normalized,
        }),
    )?;

    // Update sync state (version now tracks event count for delta sync)
    conn.execute(
        "UPDATE sync_state SET pending_changes = pending_changes + 1, version = (SELECT MAX(id) FROM memory_events) WHERE id = 1",
        [],
    )?;

    tracing::info!(memory_id = id, workspace = %normalized, "Moved memory to workspace");

    get_memory_internal(conn, id, false)
}

/// List all workspaces with their statistics.
///
/// Returns computed stats for each workspace that has at least one memory.
/// Stats are computed on-demand (not cached at the database level).
pub fn list_workspaces(conn: &Connection) -> Result<Vec<WorkspaceStats>> {
    let now = Utc::now().to_rfc3339();

    let mut stmt = conn.prepare(
        r#"
        SELECT
            workspace,
            COUNT(*) as memory_count,
            SUM(CASE WHEN tier = 'permanent' THEN 1 ELSE 0 END) as permanent_count,
            SUM(CASE WHEN tier = 'daily' THEN 1 ELSE 0 END) as daily_count,
            MIN(created_at) as first_memory_at,
            MAX(created_at) as last_memory_at,
            AVG(importance) as avg_importance
        FROM memories
        WHERE valid_to IS NULL AND (expires_at IS NULL OR expires_at > ?)
        GROUP BY workspace
        ORDER BY memory_count DESC
        "#,
    )?;

    let workspaces: Vec<WorkspaceStats> = stmt
        .query_map(params![now], |row| {
            let workspace: String = row.get(0)?;
            let memory_count: i64 = row.get(1)?;
            let permanent_count: i64 = row.get(2)?;
            let daily_count: i64 = row.get(3)?;
            let first_memory_at: Option<String> = row.get(4)?;
            let last_memory_at: Option<String> = row.get(5)?;
            let avg_importance: Option<f64> = row.get(6)?;

            Ok(WorkspaceStats {
                workspace,
                memory_count,
                permanent_count,
                daily_count,
                first_memory_at: first_memory_at.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                }),
                last_memory_at: last_memory_at.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                }),
                top_tags: vec![], // Loaded separately if needed
                avg_importance: avg_importance.map(|v| v as f32),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(workspaces)
}

/// Get statistics for a specific workspace.
pub fn get_workspace_stats(conn: &Connection, workspace: &str) -> Result<WorkspaceStats> {
    let normalized = crate::types::normalize_workspace(workspace)
        .map_err(|e| EngramError::InvalidInput(format!("Invalid workspace: {}", e)))?;

    let now = Utc::now().to_rfc3339();

    let stats = conn
        .query_row(
            r#"
        SELECT
            workspace,
            COUNT(*) as memory_count,
            SUM(CASE WHEN tier = 'permanent' THEN 1 ELSE 0 END) as permanent_count,
            SUM(CASE WHEN tier = 'daily' THEN 1 ELSE 0 END) as daily_count,
            MIN(created_at) as first_memory_at,
            MAX(created_at) as last_memory_at,
            AVG(importance) as avg_importance
        FROM memories
        WHERE workspace = ? AND valid_to IS NULL AND (expires_at IS NULL OR expires_at > ?)
        GROUP BY workspace
        "#,
            params![normalized, now],
            |row| {
                let workspace: String = row.get(0)?;
                let memory_count: i64 = row.get(1)?;
                let permanent_count: i64 = row.get(2)?;
                let daily_count: i64 = row.get(3)?;
                let first_memory_at: Option<String> = row.get(4)?;
                let last_memory_at: Option<String> = row.get(5)?;
                let avg_importance: Option<f64> = row.get(6)?;

                Ok(WorkspaceStats {
                    workspace,
                    memory_count,
                    permanent_count,
                    daily_count,
                    first_memory_at: first_memory_at.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    }),
                    last_memory_at: last_memory_at.and_then(|s| {
                        DateTime::parse_from_rfc3339(&s)
                            .map(|dt| dt.with_timezone(&Utc))
                            .ok()
                    }),
                    top_tags: vec![],
                    avg_importance: avg_importance.map(|v| v as f32),
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                EngramError::NotFound(0) // Workspace doesn't exist
            }
            _ => EngramError::Database(e),
        })?;

    Ok(stats)
}

/// Delete a workspace by moving all its memories to the default workspace or deleting them.
///
/// # Arguments
/// - `workspace`: Workspace to delete
/// - `move_to_default`: If true, moves memories to "default" workspace. If false, deletes them.
///
/// # Returns
/// Number of memories affected.
pub fn delete_workspace(conn: &Connection, workspace: &str, move_to_default: bool) -> Result<i64> {
    let normalized = crate::types::normalize_workspace(workspace)
        .map_err(|e| EngramError::InvalidInput(format!("Invalid workspace: {}", e)))?;

    if normalized == "default" {
        return Err(EngramError::InvalidInput(
            "Cannot delete the default workspace".to_string(),
        ));
    }

    let now = Utc::now().to_rfc3339();

    // First, get the IDs of all affected memories so we can record individual events
    let affected_ids: Vec<i64> = {
        let mut stmt =
            conn.prepare("SELECT id FROM memories WHERE workspace = ? AND valid_to IS NULL")?;
        let rows = stmt.query_map(params![&normalized], |row| row.get(0))?;
        rows.collect::<std::result::Result<Vec<_>, _>>()?
    };

    let affected = affected_ids.len() as i64;

    if affected > 0 {
        if move_to_default {
            // Move all memories to the default workspace
            conn.execute(
                "UPDATE memories SET workspace = 'default', updated_at = ?, version = version + 1 WHERE workspace = ? AND valid_to IS NULL",
                params![&now, &normalized],
            )?;
        } else {
            // Soft delete all memories in the workspace
            conn.execute(
                "UPDATE memories SET valid_to = ? WHERE workspace = ? AND valid_to IS NULL",
                params![&now, &normalized],
            )?;
        }

        // Record individual events for each affected memory (for proper sync delta tracking)
        let event_type = if move_to_default {
            MemoryEventType::Updated
        } else {
            MemoryEventType::Deleted
        };

        for memory_id in &affected_ids {
            record_event(
                conn,
                event_type.clone(),
                Some(*memory_id),
                None,
                serde_json::json!({
                    "action": "delete_workspace",
                    "workspace": normalized,
                    "move_to_default": move_to_default,
                }),
            )?;
        }
    }

    // Update sync state (version now tracks event count for delta sync)
    conn.execute(
        "UPDATE sync_state SET pending_changes = pending_changes + ?, version = (SELECT COALESCE(MAX(id), 0) FROM memory_events) WHERE id = 1",
        params![affected],
    )?;

    tracing::info!(
        workspace = %normalized,
        move_to_default,
        affected,
        "Deleted workspace"
    );

    Ok(affected)
}

/// Delete a memory (soft delete by setting valid_to)
pub fn delete_memory(conn: &Connection, id: i64) -> Result<()> {
    let now = Utc::now().to_rfc3339();

    // Get memory info before deletion for event data
    let memory_info: Option<(String, String)> = conn
        .query_row(
            "SELECT workspace, memory_type FROM memories WHERE id = ? AND valid_to IS NULL",
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    let affected = conn.execute(
        "UPDATE memories SET valid_to = ? WHERE id = ? AND valid_to IS NULL",
        params![now, id],
    )?;

    if affected == 0 {
        return Err(EngramError::NotFound(id));
    }

    // Also invalidate cross-references
    conn.execute(
        "UPDATE crossrefs SET valid_to = ? WHERE (from_id = ? OR to_id = ?) AND valid_to IS NULL",
        params![now, id, id],
    )?;

    // Record event for sync delta tracking
    let (workspace, memory_type) =
        memory_info.unwrap_or(("default".to_string(), "unknown".to_string()));
    record_event(
        conn,
        MemoryEventType::Deleted,
        Some(id),
        None,
        serde_json::json!({
            "workspace": workspace,
            "memory_type": memory_type,
        }),
    )?;

    // Update sync state (version now tracks event count for delta sync)
    conn.execute(
        "UPDATE sync_state SET pending_changes = pending_changes + 1, version = (SELECT MAX(id) FROM memory_events) WHERE id = 1",
        [],
    )?;

    Ok(())
}

/// List memories with filtering and pagination
pub fn list_memories(conn: &Connection, options: &ListOptions) -> Result<Vec<Memory>> {
    let now = Utc::now().to_rfc3339();

    let mut sql = String::from(
        "SELECT DISTINCT m.id, m.content, m.memory_type, m.importance, m.access_count,
                m.created_at, m.updated_at, m.last_accessed_at, m.owner_id,
                m.visibility, m.version, m.has_embedding, m.metadata,
                m.scope_type, m.scope_id, m.workspace, m.tier, m.expires_at, m.content_hash,
                m.event_time, m.event_duration_seconds, m.trigger_pattern, m.procedure_success_count,
                m.procedure_failure_count, m.summary_of_id, m.lifecycle_state, m.media_url
         FROM memories m",
    );

    let mut conditions = vec!["m.valid_to IS NULL".to_string()];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    // Exclude expired memories
    conditions.push("(m.expires_at IS NULL OR m.expires_at > ?)".to_string());
    params.push(Box::new(now));

    // Tag filter (requires join)
    if let Some(ref tags) = options.tags {
        if !tags.is_empty() {
            sql.push_str(
                " JOIN memory_tags mt ON m.id = mt.memory_id
                  JOIN tags t ON mt.tag_id = t.id",
            );
            let placeholders: Vec<String> = tags.iter().map(|_| "?".to_string()).collect();
            conditions.push(format!("t.name IN ({})", placeholders.join(", ")));
            for tag in tags {
                params.push(Box::new(tag.clone()));
            }
        }
    }

    // Type filter
    if let Some(ref memory_type) = options.memory_type {
        conditions.push("m.memory_type = ?".to_string());
        params.push(Box::new(memory_type.as_str().to_string()));
    }

    // Advanced filter (RML-932) - takes precedence over legacy metadata_filter
    if let Some(ref filter_json) = options.filter {
        let filter_expr = parse_filter(filter_json)?;
        let mut builder = SqlBuilder::new();
        let filter_sql = builder.build_filter(&filter_expr)?;
        conditions.push(filter_sql);
        for param in builder.take_params() {
            params.push(param);
        }
    } else if let Some(ref metadata_filter) = options.metadata_filter {
        // Legacy metadata filter (JSON) - deprecated in favor of `filter`
        for (key, value) in metadata_filter {
            metadata_value_to_param(key, value, &mut conditions, &mut params)?;
        }
    }

    // Scope filter
    if let Some(ref scope) = options.scope {
        conditions.push("m.scope_type = ?".to_string());
        params.push(Box::new(scope.scope_type().to_string()));
        if let Some(scope_id) = scope.scope_id() {
            conditions.push("m.scope_id = ?".to_string());
            params.push(Box::new(scope_id.to_string()));
        } else {
            conditions.push("m.scope_id IS NULL".to_string());
        }
    }

    // Workspace filter
    if let Some(ref workspace) = options.workspace {
        conditions.push("m.workspace = ?".to_string());
        params.push(Box::new(workspace.clone()));
    }

    // Tier filter
    if let Some(ref tier) = options.tier {
        conditions.push("m.tier = ?".to_string());
        params.push(Box::new(tier.as_str().to_string()));
    }

    sql.push_str(" WHERE ");
    sql.push_str(&conditions.join(" AND "));

    // Sorting
    let sort_field = match options.sort_by.unwrap_or_default() {
        SortField::CreatedAt => "m.created_at",
        SortField::UpdatedAt => "m.updated_at",
        SortField::LastAccessedAt => "m.last_accessed_at",
        SortField::Importance => "m.importance",
        SortField::AccessCount => "m.access_count",
    };
    let sort_order = match options.sort_order.unwrap_or_default() {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };
    sql.push_str(&format!(" ORDER BY {} {}", sort_field, sort_order));

    // Pagination
    let limit = options.limit.unwrap_or(100);
    let offset = options.offset.unwrap_or(0);
    sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let memories: Vec<Memory> = stmt
        .query_map(param_refs.as_slice(), memory_from_row)?
        .filter_map(|r| r.ok())
        .map(|mut m| {
            m.tags = load_tags(conn, m.id).unwrap_or_default();
            m
        })
        .collect();

    Ok(memories)
}

/// Query episodic memories ordered by event_time within a time range.
pub fn get_episodic_timeline(
    conn: &Connection,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    workspace: Option<&str>,
    tags: Option<&[String]>,
    limit: i64,
) -> Result<Vec<Memory>> {
    let now = Utc::now().to_rfc3339();

    let mut sql = String::from(
        "SELECT DISTINCT m.id, m.content, m.memory_type, m.importance, m.access_count,
                m.created_at, m.updated_at, m.last_accessed_at, m.owner_id,
                m.visibility, m.version, m.has_embedding, m.metadata,
                m.scope_type, m.scope_id, m.workspace, m.tier, m.expires_at, m.content_hash,
                m.event_time, m.event_duration_seconds, m.trigger_pattern, m.procedure_success_count,
                m.procedure_failure_count, m.summary_of_id, m.lifecycle_state, m.media_url,
                m.event_time, m.event_duration_seconds, m.trigger_pattern,
                m.procedure_success_count, m.procedure_failure_count, m.summary_of_id,
                m.lifecycle_state
         FROM memories m",
    );

    let mut conditions = vec![
        "m.valid_to IS NULL".to_string(),
        "(m.expires_at IS NULL OR m.expires_at > ?)".to_string(),
        "m.memory_type = 'episodic'".to_string(),
        "m.event_time IS NOT NULL".to_string(),
    ];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now)];

    if let Some(start) = start_time {
        conditions.push("m.event_time >= ?".to_string());
        params.push(Box::new(start.to_rfc3339()));
    }

    if let Some(end) = end_time {
        conditions.push("m.event_time <= ?".to_string());
        params.push(Box::new(end.to_rfc3339()));
    }

    if let Some(ws) = workspace {
        conditions.push("m.workspace = ?".to_string());
        params.push(Box::new(ws.to_string()));
    }

    if let Some(tag_list) = tags {
        if !tag_list.is_empty() {
            sql.push_str(
                " JOIN memory_tags mt ON m.id = mt.memory_id
                  JOIN tags t ON mt.tag_id = t.id",
            );
            let placeholders: Vec<String> = tag_list.iter().map(|_| "?".to_string()).collect();
            conditions.push(format!("t.name IN ({})", placeholders.join(", ")));
            for tag in tag_list {
                params.push(Box::new(tag.clone()));
            }
        }
    }

    sql.push_str(" WHERE ");
    sql.push_str(&conditions.join(" AND "));
    sql.push_str(" ORDER BY m.event_time ASC");
    sql.push_str(&format!(" LIMIT {}", limit));

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let memories: Vec<Memory> = stmt
        .query_map(param_refs.as_slice(), memory_from_row)?
        .filter_map(|r| r.ok())
        .map(|mut m| {
            m.tags = load_tags(conn, m.id).unwrap_or_default();
            m
        })
        .collect();

    Ok(memories)
}

/// Query procedural memories, optionally filtered by trigger pattern and success rate.
pub fn get_procedural_memories(
    conn: &Connection,
    trigger_pattern: Option<&str>,
    workspace: Option<&str>,
    min_success_rate: Option<f32>,
    limit: i64,
) -> Result<Vec<Memory>> {
    let now = Utc::now().to_rfc3339();

    let sql_base = "SELECT m.id, m.content, m.memory_type, m.importance, m.access_count,
                m.created_at, m.updated_at, m.last_accessed_at, m.owner_id,
                m.visibility, m.version, m.has_embedding, m.metadata,
                m.scope_type, m.scope_id, m.workspace, m.tier, m.expires_at, m.content_hash,
                m.event_time, m.event_duration_seconds, m.trigger_pattern, m.procedure_success_count,
                m.procedure_failure_count, m.summary_of_id, m.lifecycle_state, m.media_url,
                m.event_time, m.event_duration_seconds, m.trigger_pattern,
                m.procedure_success_count, m.procedure_failure_count, m.summary_of_id,
                m.lifecycle_state
         FROM memories m";

    let mut conditions = vec![
        "m.valid_to IS NULL".to_string(),
        "(m.expires_at IS NULL OR m.expires_at > ?)".to_string(),
        "m.memory_type = 'procedural'".to_string(),
    ];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now)];

    if let Some(pattern) = trigger_pattern {
        conditions.push("m.trigger_pattern LIKE ?".to_string());
        params.push(Box::new(format!("%{}%", pattern)));
    }

    if let Some(ws) = workspace {
        conditions.push("m.workspace = ?".to_string());
        params.push(Box::new(ws.to_string()));
    }

    if let Some(min_rate) = min_success_rate {
        // Filter: success / (success + failure) >= min_rate
        // Only apply when there's at least one execution
        conditions.push("(m.procedure_success_count + m.procedure_failure_count) > 0".to_string());
        conditions.push(
            "CAST(m.procedure_success_count AS REAL) / (m.procedure_success_count + m.procedure_failure_count) >= ?"
                .to_string(),
        );
        params.push(Box::new(min_rate as f64));
    }

    let sql = format!(
        "{} WHERE {} ORDER BY m.procedure_success_count DESC LIMIT {}",
        sql_base,
        conditions.join(" AND "),
        limit
    );

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let memories: Vec<Memory> = stmt
        .query_map(param_refs.as_slice(), memory_from_row)?
        .filter_map(|r| r.ok())
        .map(|mut m| {
            m.tags = load_tags(conn, m.id).unwrap_or_default();
            m
        })
        .collect();

    Ok(memories)
}

/// Record a success or failure outcome for a procedural memory.
pub fn record_procedure_outcome(
    conn: &Connection,
    memory_id: i64,
    success: bool,
) -> Result<Memory> {
    let column = if success {
        "procedure_success_count"
    } else {
        "procedure_failure_count"
    };

    let now = Utc::now().to_rfc3339();

    // Verify the memory exists and is procedural
    let memory_type: String = conn
        .query_row(
            "SELECT memory_type FROM memories WHERE id = ? AND valid_to IS NULL",
            params![memory_id],
            |row| row.get(0),
        )
        .map_err(|_| EngramError::NotFound(memory_id))?;

    if memory_type != "procedural" {
        return Err(EngramError::InvalidInput(format!(
            "Memory {} is type '{}', not 'procedural'",
            memory_id, memory_type
        )));
    }

    conn.execute(
        &format!(
            "UPDATE memories SET {} = {} + 1, updated_at = ? WHERE id = ?",
            column, column
        ),
        params![now, memory_id],
    )?;

    get_memory(conn, memory_id)
}

/// Create a cross-reference between memories
pub fn create_crossref(conn: &Connection, input: &CreateCrossRefInput) -> Result<CrossReference> {
    let now = Utc::now().to_rfc3339();

    // Verify both memories exist
    let _ = get_memory_internal(conn, input.from_id, false)?;
    let _ = get_memory_internal(conn, input.to_id, false)?;

    let strength = input.strength.unwrap_or(1.0);

    conn.execute(
        "INSERT INTO crossrefs (from_id, to_id, edge_type, score, strength, source, source_context, pinned, created_at, valid_from)
         VALUES (?, ?, ?, 1.0, ?, 'manual', ?, ?, ?, ?)
         ON CONFLICT(from_id, to_id, edge_type) DO UPDATE SET
            strength = excluded.strength,
            source_context = COALESCE(excluded.source_context, crossrefs.source_context),
            pinned = excluded.pinned",
        params![
            input.from_id,
            input.to_id,
            input.edge_type.as_str(),
            strength,
            input.source_context,
            input.pinned,
            now,
            now,
        ],
    )?;

    get_crossref(conn, input.from_id, input.to_id, input.edge_type)
}

/// Get a cross-reference
pub fn get_crossref(
    conn: &Connection,
    from_id: i64,
    to_id: i64,
    edge_type: EdgeType,
) -> Result<CrossReference> {
    let mut stmt = conn.prepare_cached(
        "SELECT from_id, to_id, edge_type, score, confidence, strength, source,
                source_context, created_at, valid_from, valid_to, pinned, metadata
         FROM crossrefs
         WHERE from_id = ? AND to_id = ? AND edge_type = ? AND valid_to IS NULL",
    )?;

    let crossref = stmt.query_row(params![from_id, to_id, edge_type.as_str()], |row| {
        let edge_type_str: String = row.get("edge_type")?;
        let source_str: String = row.get("source")?;
        let created_at_str: String = row.get("created_at")?;
        let valid_from_str: String = row.get("valid_from")?;
        let valid_to_str: Option<String> = row.get("valid_to")?;
        let metadata_str: String = row.get("metadata")?;

        Ok(CrossReference {
            from_id: row.get("from_id")?,
            to_id: row.get("to_id")?,
            edge_type: edge_type_str.parse().unwrap_or(EdgeType::RelatedTo),
            score: row.get("score")?,
            confidence: row.get("confidence")?,
            strength: row.get("strength")?,
            source: match source_str.as_str() {
                "manual" => RelationSource::Manual,
                "llm" => RelationSource::Llm,
                _ => RelationSource::Auto,
            },
            source_context: row.get("source_context")?,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            valid_from: DateTime::parse_from_rfc3339(&valid_from_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            valid_to: valid_to_str.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            }),
            pinned: row.get::<_, i32>("pinned")? != 0,
            metadata: serde_json::from_str(&metadata_str).unwrap_or_default(),
        })
    })?;

    Ok(crossref)
}

/// Get all cross-references for a memory
pub fn get_related(conn: &Connection, memory_id: i64) -> Result<Vec<CrossReference>> {
    let mut stmt = conn.prepare_cached(
        "SELECT from_id, to_id, edge_type, score, confidence, strength, source,
                source_context, created_at, valid_from, valid_to, pinned, metadata
         FROM crossrefs
         WHERE (from_id = ? OR to_id = ?) AND valid_to IS NULL
         ORDER BY score DESC",
    )?;

    let crossrefs: Vec<CrossReference> = stmt
        .query_map(params![memory_id, memory_id], |row| {
            let edge_type_str: String = row.get("edge_type")?;
            let source_str: String = row.get("source")?;
            let created_at_str: String = row.get("created_at")?;
            let valid_from_str: String = row.get("valid_from")?;
            let valid_to_str: Option<String> = row.get("valid_to")?;
            let metadata_str: String = row.get("metadata")?;

            Ok(CrossReference {
                from_id: row.get("from_id")?,
                to_id: row.get("to_id")?,
                edge_type: edge_type_str.parse().unwrap_or(EdgeType::RelatedTo),
                score: row.get("score")?,
                confidence: row.get("confidence")?,
                strength: row.get("strength")?,
                source: match source_str.as_str() {
                    "manual" => RelationSource::Manual,
                    "llm" => RelationSource::Llm,
                    _ => RelationSource::Auto,
                },
                source_context: row.get("source_context")?,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                valid_from: DateTime::parse_from_rfc3339(&valid_from_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                valid_to: valid_to_str.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                }),
                pinned: row.get::<_, i32>("pinned")? != 0,
                metadata: serde_json::from_str(&metadata_str).unwrap_or_default(),
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(crossrefs)
}

/// Delete a cross-reference (soft delete)
pub fn delete_crossref(
    conn: &Connection,
    from_id: i64,
    to_id: i64,
    edge_type: EdgeType,
) -> Result<()> {
    let now = Utc::now().to_rfc3339();

    let affected = conn.execute(
        "UPDATE crossrefs SET valid_to = ?
         WHERE from_id = ? AND to_id = ? AND edge_type = ? AND valid_to IS NULL",
        params![now, from_id, to_id, edge_type.as_str()],
    )?;

    if affected == 0 {
        return Err(EngramError::NotFound(from_id));
    }

    Ok(())
}

/// Set expiration on an existing memory
///
/// # Arguments
/// * `conn` - Database connection
/// * `id` - Memory ID
/// * `ttl_seconds` - Time-to-live in seconds (0 = remove expiration, None = no change)
pub fn set_memory_expiration(
    conn: &Connection,
    id: i64,
    ttl_seconds: Option<i64>,
) -> Result<Memory> {
    // Verify memory exists and is not expired
    let _ = get_memory_internal(conn, id, false)?;

    match ttl_seconds {
        Some(0) => {
            // Remove expiration
            conn.execute(
                "UPDATE memories SET expires_at = NULL, updated_at = ? WHERE id = ?",
                params![Utc::now().to_rfc3339(), id],
            )?;
        }
        Some(ttl) => {
            // Set new expiration
            let expires_at = (Utc::now() + chrono::Duration::seconds(ttl)).to_rfc3339();
            conn.execute(
                "UPDATE memories SET expires_at = ?, updated_at = ? WHERE id = ?",
                params![expires_at, Utc::now().to_rfc3339(), id],
            )?;
        }
        None => {
            // No change - don't record event or update sync state
            return get_memory_internal(conn, id, false);
        }
    }

    // Record event for sync delta tracking
    record_event(
        conn,
        MemoryEventType::Updated,
        Some(id),
        None,
        serde_json::json!({
            "changed_fields": ["expires_at"],
            "action": "set_expiration",
        }),
    )?;

    // Update sync state (version now tracks event count for delta sync)
    conn.execute(
        "UPDATE sync_state SET pending_changes = pending_changes + 1, version = (SELECT MAX(id) FROM memory_events) WHERE id = 1",
        [],
    )?;

    get_memory_internal(conn, id, false)
}

/// Delete all expired memories (cleanup job)
///
/// Returns the number of memories deleted
pub fn cleanup_expired_memories(conn: &Connection) -> Result<i64> {
    let now = Utc::now().to_rfc3339();

    // Soft delete expired memories by setting valid_to
    let affected = conn.execute(
        "UPDATE memories SET valid_to = ?
         WHERE expires_at IS NOT NULL AND expires_at <= ? AND valid_to IS NULL",
        params![now, now],
    )?;

    if affected > 0 {
        // Also invalidate cross-references involving expired memories
        conn.execute(
            "UPDATE crossrefs SET valid_to = ?
             WHERE valid_to IS NULL AND (
                 from_id IN (SELECT id FROM memories WHERE valid_to IS NOT NULL AND expires_at IS NOT NULL AND expires_at <= ?)
                 OR to_id IN (SELECT id FROM memories WHERE valid_to IS NOT NULL AND expires_at IS NOT NULL AND expires_at <= ?)
             )",
            params![now, now, now],
        )?;

        // Remove memory_entities links for expired memories
        // This ensures expired memories don't appear in entity-based queries
        conn.execute(
            "DELETE FROM memory_entities
             WHERE memory_id IN (
                 SELECT id FROM memories
                 WHERE valid_to IS NOT NULL AND expires_at IS NOT NULL AND expires_at <= ?
             )",
            params![now],
        )?;

        // Remove memory_tags links for expired memories
        conn.execute(
            "DELETE FROM memory_tags
             WHERE memory_id IN (
                 SELECT id FROM memories
                 WHERE valid_to IS NOT NULL AND expires_at IS NOT NULL AND expires_at <= ?
             )",
            params![now],
        )?;

        // Record batch event for sync delta tracking
        record_event(
            conn,
            MemoryEventType::Deleted,
            None, // Batch operation
            None,
            serde_json::json!({
                "action": "cleanup_expired",
                "affected_count": affected,
            }),
        )?;

        // Update sync state (version now tracks event count for delta sync)
        conn.execute(
            "UPDATE sync_state SET pending_changes = pending_changes + ?, version = (SELECT COALESCE(MAX(id), 0) FROM memory_events) WHERE id = 1",
            params![affected as i64],
        )?;
    }

    Ok(affected as i64)
}

/// Get count of expired memories (for monitoring)
pub fn count_expired_memories(conn: &Connection) -> Result<i64> {
    let now = Utc::now().to_rfc3339();

    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memories
         WHERE expires_at IS NOT NULL AND expires_at <= ? AND valid_to IS NULL",
        params![now],
        |row| row.get(0),
    )?;

    Ok(count)
}

/// A compact memory representation for efficient list views.
/// Contains only essential fields and a truncated content preview.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CompactMemoryRow {
    /// Memory ID
    pub id: i64,
    /// Content preview (first line or N chars)
    pub preview: String,
    /// Whether content was truncated
    pub truncated: bool,
    /// Memory type
    pub memory_type: MemoryType,
    /// Tags
    pub tags: Vec<String>,
    /// Importance score
    pub importance: f32,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Workspace name
    pub workspace: String,
    /// Memory tier
    pub tier: MemoryTier,
    /// Original content length in chars
    pub content_length: usize,
    /// Number of lines in original content
    pub line_count: usize,
}

/// List memories in compact format with preview only.
///
/// This is more efficient than `list_memories` when you don't need full content,
/// such as for browsing/listing UIs.
///
/// # Arguments
/// * `conn` - Database connection
/// * `options` - List filtering/pagination options
/// * `preview_chars` - Max chars for preview (default: 100)
pub fn list_memories_compact(
    conn: &Connection,
    options: &ListOptions,
    preview_chars: Option<usize>,
) -> Result<Vec<CompactMemoryRow>> {
    use crate::intelligence::compact_preview;

    let now = Utc::now().to_rfc3339();
    let max_preview = preview_chars.unwrap_or(100);

    let mut sql = String::from(
        "SELECT DISTINCT m.id, m.content, m.memory_type, m.importance,
                m.created_at, m.updated_at, m.workspace, m.tier
         FROM memories m",
    );

    let mut conditions = vec!["m.valid_to IS NULL".to_string()];
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![];

    // Exclude expired memories
    conditions.push("(m.expires_at IS NULL OR m.expires_at > ?)".to_string());
    params.push(Box::new(now));

    // Tag filter (requires join)
    if let Some(ref tags) = options.tags {
        if !tags.is_empty() {
            sql.push_str(
                " JOIN memory_tags mt ON m.id = mt.memory_id
                  JOIN tags t ON mt.tag_id = t.id",
            );
            let placeholders: Vec<String> = tags.iter().map(|_| "?".to_string()).collect();
            conditions.push(format!("t.name IN ({})", placeholders.join(", ")));
            for tag in tags {
                params.push(Box::new(tag.clone()));
            }
        }
    }

    // Type filter
    if let Some(ref memory_type) = options.memory_type {
        conditions.push("m.memory_type = ?".to_string());
        params.push(Box::new(memory_type.as_str().to_string()));
    }

    // Metadata filter (JSON)
    if let Some(ref metadata_filter) = options.metadata_filter {
        for (key, value) in metadata_filter {
            metadata_value_to_param(key, value, &mut conditions, &mut params)?;
        }
    }

    // Scope filter
    if let Some(ref scope) = options.scope {
        conditions.push("m.scope_type = ?".to_string());
        params.push(Box::new(scope.scope_type().to_string()));
        if let Some(scope_id) = scope.scope_id() {
            conditions.push("m.scope_id = ?".to_string());
            params.push(Box::new(scope_id.to_string()));
        } else {
            conditions.push("m.scope_id IS NULL".to_string());
        }
    }

    // Workspace filter
    if let Some(ref workspace) = options.workspace {
        conditions.push("m.workspace = ?".to_string());
        params.push(Box::new(workspace.clone()));
    }

    // Tier filter
    if let Some(ref tier) = options.tier {
        conditions.push("m.tier = ?".to_string());
        params.push(Box::new(tier.as_str().to_string()));
    }

    sql.push_str(" WHERE ");
    sql.push_str(&conditions.join(" AND "));

    // Sorting
    let sort_field = match options.sort_by.unwrap_or_default() {
        SortField::CreatedAt => "m.created_at",
        SortField::UpdatedAt => "m.updated_at",
        SortField::LastAccessedAt => "m.last_accessed_at",
        SortField::Importance => "m.importance",
        SortField::AccessCount => "m.access_count",
    };
    let sort_order = match options.sort_order.unwrap_or_default() {
        SortOrder::Asc => "ASC",
        SortOrder::Desc => "DESC",
    };
    sql.push_str(&format!(" ORDER BY {} {}", sort_field, sort_order));

    // Pagination
    let limit = options.limit.unwrap_or(100);
    let offset = options.offset.unwrap_or(0);
    sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|b| b.as_ref()).collect();
    let mut stmt = conn.prepare(&sql)?;

    let memories: Vec<CompactMemoryRow> = stmt
        .query_map(param_refs.as_slice(), |row| {
            let id: i64 = row.get("id")?;
            let content: String = row.get("content")?;
            let memory_type_str: String = row.get("memory_type")?;
            let importance: f32 = row.get("importance")?;
            let created_at_str: String = row.get("created_at")?;
            let updated_at_str: String = row.get("updated_at")?;
            let workspace: String = row.get("workspace")?;
            let tier_str: String = row.get("tier")?;

            let memory_type = memory_type_str.parse().unwrap_or(MemoryType::Note);
            let tier = tier_str.parse().unwrap_or_default();

            // Generate compact preview
            let (preview, truncated) = compact_preview(&content, max_preview);
            let content_length = content.len();
            let line_count = content.lines().count();

            Ok(CompactMemoryRow {
                id,
                preview,
                truncated,
                memory_type,
                tags: vec![], // Will be loaded separately
                importance,
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                workspace,
                tier,
                content_length,
                line_count,
            })
        })?
        .filter_map(|r| r.ok())
        .map(|mut m| {
            m.tags = load_tags(conn, m.id).unwrap_or_default();
            m
        })
        .collect();

    Ok(memories)
}

/// Get storage statistics
pub fn get_stats(conn: &Connection) -> Result<StorageStats> {
    let total_memories: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memories WHERE valid_to IS NULL",
        [],
        |row| row.get(0),
    )?;

    let total_tags: i64 = conn.query_row("SELECT COUNT(*) FROM tags", [], |row| row.get(0))?;

    let total_crossrefs: i64 = conn.query_row(
        "SELECT COUNT(*) FROM crossrefs WHERE valid_to IS NULL",
        [],
        |row| row.get(0),
    )?;

    let total_versions: i64 =
        conn.query_row("SELECT COUNT(*) FROM memory_versions", [], |row| row.get(0))?;

    let _total_identities: i64 =
        conn.query_row("SELECT COUNT(*) FROM identities", [], |row| row.get(0))?;

    let _total_entities: i64 =
        conn.query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))?;

    let db_size_bytes: i64 = conn.query_row(
        "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
        [],
        |row| row.get(0),
    )?;

    let _schema_version: i32 = conn
        .query_row("SELECT MAX(version) FROM schema_version", [], |row| {
            row.get(0)
        })
        .unwrap_or(0);

    let mut workspace_stmt = conn.prepare(
        "SELECT workspace, COUNT(*) FROM memories WHERE valid_to IS NULL GROUP BY workspace",
    )?;
    let workspaces: HashMap<String, i64> = workspace_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut type_stmt = conn.prepare(
        "SELECT memory_type, COUNT(*) FROM memories WHERE valid_to IS NULL GROUP BY memory_type",
    )?;
    let type_counts: HashMap<String, i64> = type_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let mut tier_stmt = conn.prepare(
        "SELECT COALESCE(tier, 'permanent'), COUNT(*) FROM memories GROUP BY COALESCE(tier, 'permanent')",
    )?;
    let tier_counts: HashMap<String, i64> = tier_stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?
        .filter_map(|r| r.ok())
        .collect();

    let memories_with_embeddings: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memories WHERE has_embedding = 1 AND valid_to IS NULL",
        [],
        |row| row.get(0),
    )?;

    let memories_pending_embedding: i64 = conn.query_row(
        "SELECT COUNT(*) FROM embedding_queue WHERE status = 'pending'",
        [],
        |row| row.get(0),
    )?;

    let (last_sync, sync_pending): (Option<String>, i64) = conn.query_row(
        "SELECT last_sync, pending_changes FROM sync_state WHERE id = 1",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let schema_version: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(StorageStats {
        total_memories,
        total_tags,
        total_crossrefs,
        total_versions,
        total_identities: 0,
        total_entities: 0,
        db_size_bytes,
        memories_with_embeddings,
        memories_pending_embedding,
        last_sync: last_sync.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        }),
        sync_pending: sync_pending > 0,
        storage_mode: "sqlite".to_string(),
        schema_version,
        workspaces,
        type_counts,
        tier_counts,
    })
}

/// Get memory versions
pub fn get_memory_versions(conn: &Connection, memory_id: i64) -> Result<Vec<MemoryVersion>> {
    let mut stmt = conn.prepare_cached(
        "SELECT version, content, tags, metadata, created_at, created_by, change_summary
         FROM memory_versions WHERE memory_id = ? ORDER BY version DESC",
    )?;

    let versions: Vec<MemoryVersion> = stmt
        .query_map([memory_id], |row| {
            let tags_str: String = row.get("tags")?;
            let metadata_str: String = row.get("metadata")?;
            let created_at_str: String = row.get("created_at")?;

            Ok(MemoryVersion {
                version: row.get("version")?,
                content: row.get("content")?,
                tags: serde_json::from_str(&tags_str).unwrap_or_default(),
                metadata: serde_json::from_str(&metadata_str).unwrap_or_default(),
                created_at: DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                created_by: row.get("created_by")?,
                change_summary: row.get("change_summary")?,
            })
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(versions)
}

// =============================================================================
// Search Variants
// =============================================================================

/// Search memories by identity (canonical ID or alias)
pub fn search_by_identity(
    conn: &Connection,
    identity: &str,
    workspace: Option<&str>,
    limit: Option<usize>,
) -> Result<Vec<Memory>> {
    let limit = limit.unwrap_or(50);
    let now = Utc::now().to_rfc3339();

    // Search in content and tags for the identity
    // Tags are in a junction table, so we need to use a subquery or JOIN
    let pattern = format!("%{}%", identity);

    let query = if workspace.is_some() {
        "SELECT DISTINCT m.id, m.content, m.memory_type, m.importance, m.access_count,
                m.created_at, m.updated_at, m.last_accessed_at, m.owner_id,
                m.visibility, m.version, m.has_embedding, m.metadata,
                m.scope_type, m.scope_id, m.workspace, m.tier, m.expires_at, m.content_hash,
                m.event_time, m.event_duration_seconds, m.trigger_pattern, m.procedure_success_count,
                m.procedure_failure_count, m.summary_of_id, m.lifecycle_state, m.media_url
         FROM memories m
         LEFT JOIN memory_tags mt ON m.id = mt.memory_id
         LEFT JOIN tags t ON mt.tag_id = t.id
         WHERE m.workspace = ? AND (m.content LIKE ? OR t.name LIKE ?)
           AND m.valid_to IS NULL
           AND (m.expires_at IS NULL OR m.expires_at > ?)
         ORDER BY m.importance DESC, m.created_at DESC
         LIMIT ?"
    } else {
        "SELECT DISTINCT m.id, m.content, m.memory_type, m.importance, m.access_count,
                m.created_at, m.updated_at, m.last_accessed_at, m.owner_id,
                m.visibility, m.version, m.has_embedding, m.metadata,
                m.scope_type, m.scope_id, m.workspace, m.tier, m.expires_at, m.content_hash,
                m.event_time, m.event_duration_seconds, m.trigger_pattern, m.procedure_success_count,
                m.procedure_failure_count, m.summary_of_id, m.lifecycle_state, m.media_url
         FROM memories m
         LEFT JOIN memory_tags mt ON m.id = mt.memory_id
         LEFT JOIN tags t ON mt.tag_id = t.id
         WHERE (m.content LIKE ? OR t.name LIKE ?)
           AND m.valid_to IS NULL
           AND (m.expires_at IS NULL OR m.expires_at > ?)
         ORDER BY m.importance DESC, m.created_at DESC
         LIMIT ?"
    };

    let mut stmt = conn.prepare(query)?;

    let memories = if let Some(ws) = workspace {
        stmt.query_map(
            params![ws, &pattern, &pattern, &now, limit as i64],
            memory_from_row,
        )?
        .collect::<std::result::Result<Vec<_>, _>>()?
    } else {
        stmt.query_map(
            params![&pattern, &pattern, &now, limit as i64],
            memory_from_row,
        )?
        .collect::<std::result::Result<Vec<_>, _>>()?
    };

    Ok(memories)
}

/// Search within session transcript chunks
pub fn search_sessions(
    conn: &Connection,
    query_text: &str,
    session_id: Option<&str>,
    workspace: Option<&str>,
    limit: Option<usize>,
) -> Result<Vec<Memory>> {
    let limit = limit.unwrap_or(20);
    let now = Utc::now().to_rfc3339();
    let pattern = format!("%{}%", query_text);

    // Build query based on filters
    // Session chunks are stored as TranscriptChunk type (not Context)
    let mut conditions = vec![
        "m.memory_type = 'transcript_chunk'",
        "m.valid_to IS NULL",
        "(m.expires_at IS NULL OR m.expires_at > ?)",
    ];
    let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(now)];

    // Add session filter via tags (tags are in junction table)
    let use_tag_join = session_id.is_some();
    if let Some(sid) = session_id {
        let tag_name = format!("session:{}", sid);
        conditions.push("t.name = ?");
        params_vec.push(Box::new(tag_name));
    }

    // Add workspace filter
    if let Some(ws) = workspace {
        conditions.push("m.workspace = ?");
        params_vec.push(Box::new(ws.to_string()));
    }

    // Add content search
    conditions.push("m.content LIKE ?");
    params_vec.push(Box::new(pattern));

    // Add limit
    params_vec.push(Box::new(limit as i64));

    // Build query with optional tag join
    let join_clause = if use_tag_join {
        "JOIN memory_tags mt ON m.id = mt.memory_id JOIN tags t ON mt.tag_id = t.id"
    } else {
        ""
    };

    let query = format!(
        "SELECT DISTINCT m.id, m.content, m.memory_type, m.importance, m.access_count,
                m.created_at, m.updated_at, m.last_accessed_at, m.owner_id,
                m.visibility, m.version, m.has_embedding, m.metadata,
                m.scope_type, m.scope_id, m.workspace, m.tier, m.expires_at, m.content_hash,
                m.event_time, m.event_duration_seconds, m.trigger_pattern, m.procedure_success_count,
                m.procedure_failure_count, m.summary_of_id, m.lifecycle_state, m.media_url
         FROM memories m {} WHERE {} ORDER BY m.created_at DESC LIMIT ?",
        join_clause,
        conditions.join(" AND ")
    );

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(&query)?;
    let memories = stmt
        .query_map(params_refs.as_slice(), memory_from_row)?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(memories)
}

/// Insert a Dream Phase run report (Phase D - Issue #12)
#[cfg(feature = "dream-phase")]
pub fn insert_dream_run(conn: &Connection, report: &crate::dream::DreamReport) -> Result<i64> {
    let report_json =
        serde_json::to_string(report).map_err(|e| EngramError::Internal(e.to_string()))?;

    conn.execute(
        "INSERT INTO dream_runs (started_at, finished_at, report_json, error_count, workspace_count)
         VALUES (?, ?, ?, ?, ?)",
        params![
            report.started_at.to_rfc3339(),
            report.finished_at.to_rfc3339(),
            report_json,
            report.errors.len() as i32,
            report.workspaces.len() as i32,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Try to acquire an advisory lock (Phase D - Issue #12)
pub fn acquire_dream_lock(
    conn: &Connection,
    lock_id: &str,
    owner_id: &str,
    ttl_secs: u64,
) -> Result<bool> {
    let now = Utc::now();
    let expires_at = now + chrono::Duration::seconds(ttl_secs as i64);

    // Cleanup expired locks first
    conn.execute(
        "DELETE FROM dream_locks WHERE expires_at < ?",
        params![now.to_rfc3339()],
    )?;

    // Try to insert (will fail if lock_id exists and not expired)
    let res = conn.execute(
        "INSERT OR IGNORE INTO dream_locks (lock_id, acquired_at, expires_at, owner_id)
         VALUES (?, ?, ?, ?)",
        params![lock_id, now.to_rfc3339(), expires_at.to_rfc3339(), owner_id],
    )?;

    Ok(res > 0)
}

/// Release an advisory lock (Phase D - Issue #12)
pub fn release_dream_lock(conn: &Connection, lock_id: &str, owner_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM dream_locks WHERE lock_id = ? AND owner_id = ?",
        params![lock_id, owner_id],
    )?;
    Ok(())
}
