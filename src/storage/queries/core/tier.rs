use super::*;

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

    if let Err(e) = record_reinforcement(conn, id, 0.25, "memory_promote_to_permanent") {
        tracing::warn!(
            memory_id = id,
            error = %e,
            "failed to record policy reinforcement for promotion; continuing"
        );
    }

    get_memory_internal(conn, id, false)
}
