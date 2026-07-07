use super::*;

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
