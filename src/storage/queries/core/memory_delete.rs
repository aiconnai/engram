use super::*;

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
