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
