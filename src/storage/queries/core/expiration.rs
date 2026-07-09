use super::*;

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
