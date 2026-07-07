use super::*;

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
