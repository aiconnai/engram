use super::*;
use std::collections::HashMap;

/// Queue all memories for re-embedding
pub fn rebuild_embeddings(conn: &Connection) -> Result<i64> {
    let now = Utc::now().to_rfc3339();

    // Clear existing queue
    conn.execute("DELETE FROM embedding_queue", [])?;

    // Queue all memories
    let count = conn.execute(
        "INSERT INTO embedding_queue (memory_id, status, queued_at)
         SELECT id, 'pending', ? FROM memories WHERE valid_to IS NULL",
        params![now],
    )?;

    // Reset has_embedding flag
    conn.execute(
        "UPDATE memories SET has_embedding = 0 WHERE valid_to IS NULL",
        [],
    )?;

    Ok(count as i64)
}

/// Rebuild all cross-references based on embeddings
pub fn rebuild_crossrefs(conn: &Connection) -> Result<i64> {
    let now = Utc::now().to_rfc3339();

    // Clear existing auto-generated crossrefs (keep manual ones)
    let deleted = conn.execute(
        "UPDATE crossrefs SET valid_to = ? WHERE source = 'auto' AND valid_to IS NULL",
        params![now],
    )?;

    // Note: Actual crossref generation requires embeddings and is done by the embedding worker
    // This just clears the old ones so they can be regenerated

    Ok(deleted as i64)
}

// ============================================================================
// Special Memory Types
// ============================================================================

/// Create a section memory (for document structure)
pub fn create_section_memory(
    conn: &Connection,
    title: &str,
    content: &str,
    parent_id: Option<i64>,
    level: i32,
    workspace: Option<&str>,
) -> Result<Memory> {
    let mut metadata = HashMap::new();
    metadata.insert("section_title".to_string(), serde_json::json!(title));
    metadata.insert("section_level".to_string(), serde_json::json!(level));
    if let Some(pid) = parent_id {
        metadata.insert("parent_memory_id".to_string(), serde_json::json!(pid));
    }

    let input = CreateMemoryInput {
        content: format!("# {}\n\n{}", title, content),
        memory_type: MemoryType::Context,
        tags: vec!["section".to_string()],
        metadata,
        importance: Some(0.6),
        scope: MemoryScope::Global,
        workspace: workspace.map(String::from),
        tier: MemoryTier::Permanent,
        defer_embedding: false,
        ttl_seconds: None,
        dedup_mode: DedupMode::Skip,
        dedup_threshold: None,
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern: None,
        summary_of_id: None,
        media_url: None,
    };

    create_memory(conn, &input)
}

/// Create a checkpoint memory for session state
pub fn create_checkpoint(
    conn: &Connection,
    session_id: &str,
    summary: &str,
    context: &HashMap<String, serde_json::Value>,
    workspace: Option<&str>,
) -> Result<Memory> {
    let mut metadata = context.clone();
    metadata.insert(
        "checkpoint_session".to_string(),
        serde_json::json!(session_id),
    );
    metadata.insert(
        "checkpoint_time".to_string(),
        serde_json::json!(Utc::now().to_rfc3339()),
    );

    let input = CreateMemoryInput {
        content: format!("Session Checkpoint: {}\n\n{}", session_id, summary),
        memory_type: MemoryType::Context,
        tags: vec!["checkpoint".to_string(), format!("session:{}", session_id)],
        metadata,
        importance: Some(0.7),
        scope: MemoryScope::Global,
        workspace: workspace.map(String::from),
        tier: MemoryTier::Permanent,
        defer_embedding: false,
        ttl_seconds: None,
        dedup_mode: DedupMode::Allow,
        dedup_threshold: None,
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern: None,
        summary_of_id: None,
        media_url: None,
    };

    create_memory(conn, &input)
}

/// Temporarily boost a memory's importance
pub fn boost_memory(
    conn: &Connection,
    id: i64,
    boost_amount: f32,
    duration_seconds: Option<i64>,
) -> Result<Memory> {
    let memory = get_memory(conn, id)?;
    let new_importance = (memory.importance + boost_amount).min(1.0);
    let now = Utc::now();

    // Update importance
    conn.execute(
        "UPDATE memories SET importance = ?, updated_at = ? WHERE id = ?",
        params![new_importance, now.to_rfc3339(), id],
    )?;

    // If duration specified, store boost info in metadata for later decay
    if let Some(duration) = duration_seconds {
        let expires = now + chrono::Duration::seconds(duration);
        let mut metadata = memory.metadata.clone();
        metadata.insert(
            "boost_expires".to_string(),
            serde_json::json!(expires.to_rfc3339()),
        );
        metadata.insert(
            "boost_original_importance".to_string(),
            serde_json::json!(memory.importance),
        );

        let metadata_json = serde_json::to_string(&metadata)?;
        conn.execute(
            "UPDATE memories SET metadata = ? WHERE id = ?",
            params![metadata_json, id],
        )?;
    }

    get_memory(conn, id)
}
/// Rebuild derived indexes (issue #23): the FTS5 index and/or the embedding
/// queue for memories missing embeddings.
///
/// Derived indexes are disposable — this never mutates canonical `memories` or
/// their versions. With `apply = false` (dry-run) it only reports counts and
/// drift. The same shape is intended to host future external-backend rebuilds.
pub fn rebuild_derived_indexes(
    conn: &Connection,
    rebuild_fts: bool,
    requeue_embeddings: bool,
    apply: bool,
) -> Result<RebuildReport> {
    let memories: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM memories WHERE valid_to IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let memories_total: i64 = conn
        .query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))
        .unwrap_or(0);

    let fts_count = || -> i64 {
        conn.query_row("SELECT COUNT(*) FROM memories_fts_docsize", [], |r| {
            r.get(0)
        })
        .or_else(|_| conn.query_row("SELECT COUNT(*) FROM memories_fts", [], |r| r.get(0)))
        .unwrap_or(0)
    };
    let fts_indexed_before = fts_count();
    let fts_drift_before = (memories_total - fts_indexed_before).abs();

    let embeddings_present: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM memories WHERE has_embedding = 1 AND valid_to IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);
    let embeddings_missing: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM memories WHERE has_embedding = 0 AND valid_to IS NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let mut fts_rebuilt = false;
    let mut embeddings_requeued = 0i64;

    if apply {
        if rebuild_fts {
            // FTS5 'rebuild' reads columns straight from the content table, but
            // this external-content index is fed by triggers (tags is always '',
            // metadata comes from the memories row), so 'rebuild' fails with
            // "no such column". Reset the index and re-populate with the same
            // expression the insert trigger uses.
            conn.execute_batch(
                "INSERT INTO memories_fts(memories_fts) VALUES('delete-all'); \
                 INSERT INTO memories_fts(rowid, content, tags, metadata) \
                     SELECT id, content, '', metadata FROM memories;",
            )?;
            fts_rebuilt = true;
        }
        if requeue_embeddings {
            embeddings_requeued = conn.execute(
                "INSERT INTO embedding_queue (memory_id, status) \
                 SELECT id, 'pending' FROM memories WHERE has_embedding = 0 AND valid_to IS NULL \
                 ON CONFLICT(memory_id) DO UPDATE SET \
                   status = 'pending', error = NULL, retry_count = 0, started_at = NULL, completed_at = NULL",
                [],
            )? as i64;
        }
    }

    let fts_indexed_after = fts_count();
    let fts_drift_after = (memories_total - fts_indexed_after).abs();

    Ok(RebuildReport {
        applied: apply,
        memories,
        fts_targeted: rebuild_fts,
        fts_indexed_before,
        fts_indexed_after,
        fts_drift_before,
        fts_drift_after,
        fts_rebuilt,
        embeddings_targeted: requeue_embeddings,
        embeddings_present,
        embeddings_missing,
        embeddings_requeued,
    })
}
