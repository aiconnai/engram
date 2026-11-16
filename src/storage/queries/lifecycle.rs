use rusqlite::{params, Connection};

use super::{load_tags, memory_from_row, record_event, MemoryEventType};
use crate::error::{EngramError, Result};
use crate::types::{LifecycleState, Memory};

fn get_current_memory_including_expired(conn: &Connection, id: i64) -> Result<Memory> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, content, memory_type, importance, access_count,
                created_at, updated_at, last_accessed_at, owner_id,
                visibility, version, has_embedding, metadata,
                scope_type, scope_id, workspace, tier, expires_at, content_hash,
                event_time, event_duration_seconds, trigger_pattern, procedure_success_count,
                procedure_failure_count, summary_of_id, lifecycle_state, media_url
         FROM memories
         WHERE id = ? AND valid_to IS NULL",
    )?;

    let mut memory = stmt
        .query_row(params![id], memory_from_row)
        .map_err(|_| EngramError::NotFound(id))?;
    memory.tags = load_tags(conn, id)?;

    Ok(memory)
}

/// Update a memory lifecycle state with normal update bookkeeping.
pub fn update_memory_lifecycle_state(
    conn: &Connection,
    id: i64,
    lifecycle_state: LifecycleState,
) -> Result<Memory> {
    let current = get_current_memory_including_expired(conn, id)?;
    let now = chrono::Utc::now().to_rfc3339();
    let lifecycle_state_value = lifecycle_state.to_string();

    let updated = conn.execute(
        "UPDATE memories
         SET lifecycle_state = ?1,
             updated_at = ?2,
             version = version + 1
         WHERE id = ?3
           AND valid_to IS NULL",
        params![&lifecycle_state_value, &now, id],
    )?;

    if updated == 0 {
        return Err(EngramError::NotFound(id));
    }

    let tags_json = serde_json::to_string(&current.tags)?;
    let metadata_json = serde_json::to_string(&current.metadata)?;
    conn.execute(
        "INSERT INTO memory_versions (memory_id, version, content, tags, metadata, created_at)
         VALUES (?, (SELECT version FROM memories WHERE id = ?), ?, ?, ?, ?)",
        params![id, id, &current.content, tags_json, metadata_json, &now],
    )?;

    record_event(
        conn,
        MemoryEventType::Updated,
        Some(id),
        None,
        serde_json::json!({
            "changed_fields": ["lifecycle_state"],
            "lifecycle_state": {
                "from": current.lifecycle_state.to_string(),
                "to": lifecycle_state_value,
            },
        }),
    )?;

    conn.execute(
        "UPDATE sync_state SET pending_changes = pending_changes + 1, version = (SELECT MAX(id) FROM memory_events) WHERE id = 1",
        [],
    )?;

    get_current_memory_including_expired(conn, id)
}

/// Archive a live, not-yet-archived memory through the normal update path.
///
/// Bulk-archival sites (retention, consolidation, conflict resolution) use this
/// instead of a raw `UPDATE memories SET lifecycle_state = 'archived'`, so every
/// archived memory gets the version bump, the `memory_versions` row, the change
/// event and the `sync_state` update of [`update_memory_lifecycle_state`].
///
/// Memories that are missing, soft-deleted, already archived or (when
/// `workspace` is given) in another workspace are left untouched and reported as
/// `Ok(false)`, matching the guards of the former raw statements.
pub(crate) fn archive_memory_if_active(
    conn: &Connection,
    id: i64,
    workspace: Option<&str>,
) -> Result<bool> {
    let eligible: bool = conn.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM memories
             WHERE id = ?1
               AND valid_to IS NULL
               AND COALESCE(lifecycle_state, 'active') != 'archived'
               AND (?2 IS NULL OR workspace = ?2)
         )",
        params![id, workspace],
        |row| row.get(0),
    )?;
    if !eligible {
        return Ok(false);
    }
    update_memory_lifecycle_state(conn, id, LifecycleState::Archived)?;
    Ok(true)
}

/// Test-only snapshot of the bookkeeping a normal update must leave behind:
/// `(version, memory_versions rows, memory_events rows, sync_state.pending_changes)`.
#[cfg(test)]
pub(crate) fn update_footprint(conn: &Connection, id: i64) -> Result<(i64, i64, i64, i64)> {
    let version: i64 = conn.query_row(
        "SELECT version FROM memories WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    let versions: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memory_versions WHERE memory_id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    let events: i64 = conn.query_row(
        "SELECT COUNT(*) FROM memory_events WHERE memory_id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    let pending: i64 = conn.query_row(
        "SELECT pending_changes FROM sync_state WHERE id = 1",
        [],
        |row| row.get(0),
    )?;
    Ok((version, versions, events, pending))
}
