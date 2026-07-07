use super::*;

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
