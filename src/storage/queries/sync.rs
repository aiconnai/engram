use super::*;
use serde::{Deserialize, Serialize};

// =============================================================================
// Event System
// =============================================================================

/// Event types for the memory system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryEventType {
    Created,
    Updated,
    Deleted,
    Linked,
    Unlinked,
    Shared,
    Synced,
}

impl std::fmt::Display for MemoryEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryEventType::Created => write!(f, "created"),
            MemoryEventType::Updated => write!(f, "updated"),
            MemoryEventType::Deleted => write!(f, "deleted"),
            MemoryEventType::Linked => write!(f, "linked"),
            MemoryEventType::Unlinked => write!(f, "unlinked"),
            MemoryEventType::Shared => write!(f, "shared"),
            MemoryEventType::Synced => write!(f, "synced"),
        }
    }
}

impl std::str::FromStr for MemoryEventType {
    type Err = EngramError;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "created" => Ok(MemoryEventType::Created),
            "updated" => Ok(MemoryEventType::Updated),
            "deleted" => Ok(MemoryEventType::Deleted),
            "linked" => Ok(MemoryEventType::Linked),
            "unlinked" => Ok(MemoryEventType::Unlinked),
            "shared" => Ok(MemoryEventType::Shared),
            "synced" => Ok(MemoryEventType::Synced),
            _ => Err(EngramError::InvalidInput(format!(
                "Invalid event type: {}",
                s
            ))),
        }
    }
}

/// A memory event for tracking changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEvent {
    pub id: i64,
    pub event_type: String,
    pub memory_id: Option<i64>,
    pub agent_id: Option<String>,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Record an event in the event system
pub fn record_event(
    conn: &Connection,
    event_type: MemoryEventType,
    memory_id: Option<i64>,
    agent_id: Option<&str>,
    data: serde_json::Value,
) -> Result<i64> {
    let now = Utc::now();
    let data_json = serde_json::to_string(&data)?;

    conn.execute(
        "INSERT INTO memory_events (event_type, memory_id, agent_id, data, created_at)
         VALUES (?, ?, ?, ?, ?)",
        params![
            event_type.to_string(),
            memory_id,
            agent_id,
            data_json,
            now.to_rfc3339()
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Poll for events since a given timestamp or event ID
pub fn poll_events(
    conn: &Connection,
    since_id: Option<i64>,
    since_time: Option<DateTime<Utc>>,
    agent_id: Option<&str>,
    limit: Option<usize>,
) -> Result<Vec<MemoryEvent>> {
    let limit = limit.unwrap_or(100);

    let (query, params): (&str, Vec<Box<dyn rusqlite::ToSql>>) =
        match (since_id, since_time, agent_id) {
            (Some(id), _, Some(agent)) => (
                "SELECT id, event_type, memory_id, agent_id, data, created_at
             FROM memory_events WHERE id > ? AND (agent_id = ? OR agent_id IS NULL)
             ORDER BY id ASC LIMIT ?",
                vec![
                    Box::new(id),
                    Box::new(agent.to_string()),
                    Box::new(limit as i64),
                ],
            ),
            (Some(id), _, None) => (
                "SELECT id, event_type, memory_id, agent_id, data, created_at
             FROM memory_events WHERE id > ?
             ORDER BY id ASC LIMIT ?",
                vec![Box::new(id), Box::new(limit as i64)],
            ),
            (None, Some(time), Some(agent)) => (
                "SELECT id, event_type, memory_id, agent_id, data, created_at
             FROM memory_events WHERE created_at > ? AND (agent_id = ? OR agent_id IS NULL)
             ORDER BY id ASC LIMIT ?",
                vec![
                    Box::new(time.to_rfc3339()),
                    Box::new(agent.to_string()),
                    Box::new(limit as i64),
                ],
            ),
            (None, Some(time), None) => (
                "SELECT id, event_type, memory_id, agent_id, data, created_at
             FROM memory_events WHERE created_at > ?
             ORDER BY id ASC LIMIT ?",
                vec![Box::new(time.to_rfc3339()), Box::new(limit as i64)],
            ),
            (None, None, Some(agent)) => (
                "SELECT id, event_type, memory_id, agent_id, data, created_at
             FROM memory_events WHERE agent_id = ? OR agent_id IS NULL
             ORDER BY id DESC LIMIT ?",
                vec![Box::new(agent.to_string()), Box::new(limit as i64)],
            ),
            (None, None, None) => (
                "SELECT id, event_type, memory_id, agent_id, data, created_at
             FROM memory_events ORDER BY id DESC LIMIT ?",
                vec![Box::new(limit as i64)],
            ),
        };

    let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();
    let mut stmt = conn.prepare(query)?;
    let events = stmt
        .query_map(params_refs.as_slice(), |row| {
            let data_str: String = row.get(4)?;
            let created_str: String = row.get(5)?;
            Ok(MemoryEvent {
                id: row.get(0)?,
                event_type: row.get(1)?,
                memory_id: row.get(2)?,
                agent_id: row.get(3)?,
                data: serde_json::from_str(&data_str).unwrap_or(serde_json::json!({})),
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(events)
}

/// Clear old events (cleanup)
pub fn clear_events(
    conn: &Connection,
    before_id: Option<i64>,
    before_time: Option<DateTime<Utc>>,
    keep_recent: Option<usize>,
) -> Result<i64> {
    let deleted = if let Some(id) = before_id {
        conn.execute("DELETE FROM memory_events WHERE id < ?", params![id])?
    } else if let Some(time) = before_time {
        conn.execute(
            "DELETE FROM memory_events WHERE created_at < ?",
            params![time.to_rfc3339()],
        )?
    } else if let Some(keep) = keep_recent {
        // Keep only the most recent N events
        conn.execute(
            "DELETE FROM memory_events WHERE id NOT IN (
                SELECT id FROM memory_events ORDER BY id DESC LIMIT ?
            )",
            params![keep as i64],
        )?
    } else {
        // Clear all events
        conn.execute("DELETE FROM memory_events", [])?
    };

    Ok(deleted as i64)
}

// =============================================================================
// Advanced Sync
// =============================================================================

/// Sync version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncVersion {
    pub version: i64,
    pub last_modified: DateTime<Utc>,
    pub memory_count: i64,
    pub checksum: String,
}

/// Sync task status record (Phase 3 - Langfuse integration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTask {
    pub task_id: String,
    pub task_type: String,
    pub status: String,
    pub progress_percent: i32,
    pub traces_processed: i64,
    pub memories_created: i64,
    pub error_message: Option<String>,
    pub started_at: String,
    pub completed_at: Option<String>,
}

/// Get the current sync version
pub fn get_sync_version(conn: &Connection) -> Result<SyncVersion> {
    let memory_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))?;

    let last_modified: Option<String> = conn
        .query_row("SELECT MAX(updated_at) FROM memories", [], |row| row.get(0))
        .ok();

    let version: i64 = conn
        .query_row("SELECT MAX(version) FROM sync_state", [], |row| row.get(0))
        .unwrap_or(0);

    // Simple checksum based on count and last modified
    let checksum = format!(
        "{}-{}-{}",
        memory_count,
        version,
        last_modified.as_deref().unwrap_or("none")
    );

    Ok(SyncVersion {
        version,
        last_modified: last_modified
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now),
        memory_count,
        checksum,
    })
}

/// Insert or update a sync task record
pub fn upsert_sync_task(conn: &Connection, task: &SyncTask) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO sync_tasks (
            task_id, task_type, status, progress_percent, traces_processed, memories_created,
            error_message, started_at, completed_at
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(task_id) DO UPDATE SET
            task_type = excluded.task_type,
            status = excluded.status,
            progress_percent = excluded.progress_percent,
            traces_processed = excluded.traces_processed,
            memories_created = excluded.memories_created,
            error_message = excluded.error_message,
            started_at = excluded.started_at,
            completed_at = excluded.completed_at
        "#,
        params![
            task.task_id,
            task.task_type,
            task.status,
            task.progress_percent,
            task.traces_processed,
            task.memories_created,
            task.error_message,
            task.started_at,
            task.completed_at
        ],
    )?;

    Ok(())
}

/// Get a sync task by ID
pub fn get_sync_task(conn: &Connection, task_id: &str) -> Result<Option<SyncTask>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT task_id, task_type, status, progress_percent, traces_processed, memories_created,
               error_message, started_at, completed_at
        FROM sync_tasks
        WHERE task_id = ?
        "#,
    )?;

    let mut rows = stmt.query(params![task_id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(SyncTask {
            task_id: row.get("task_id")?,
            task_type: row.get("task_type")?,
            status: row.get("status")?,
            progress_percent: row.get("progress_percent")?,
            traces_processed: row.get("traces_processed")?,
            memories_created: row.get("memories_created")?,
            error_message: row.get("error_message")?,
            started_at: row.get("started_at")?,
            completed_at: row.get("completed_at")?,
        }))
    } else {
        Ok(None)
    }
}

/// Delta entry for sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDelta {
    pub created: Vec<Memory>,
    pub updated: Vec<Memory>,
    pub deleted: Vec<i64>,
    pub from_version: i64,
    pub to_version: i64,
}

/// Get changes since a specific version
pub fn get_sync_delta(conn: &Connection, since_version: i64) -> Result<SyncDelta> {
    let current_version = get_sync_version(conn)?.version;

    // Get events since that version to determine what changed
    let events = poll_events(conn, Some(since_version), None, None, Some(10000))?;

    let mut created_ids = std::collections::HashSet::new();
    let mut updated_ids = std::collections::HashSet::new();
    let mut deleted_ids = std::collections::HashSet::new();

    for event in events {
        if let Some(memory_id) = event.memory_id {
            match event.event_type.as_str() {
                "created" => {
                    created_ids.insert(memory_id);
                }
                "updated" if !created_ids.contains(&memory_id) => {
                    updated_ids.insert(memory_id);
                }
                "deleted" => {
                    created_ids.remove(&memory_id);
                    updated_ids.remove(&memory_id);
                    deleted_ids.insert(memory_id);
                }
                _ => {}
            }
        }
    }

    let created: Vec<Memory> = created_ids
        .iter()
        .filter_map(|id| get_memory(conn, *id).ok())
        .collect();

    let updated: Vec<Memory> = updated_ids
        .iter()
        .filter_map(|id| get_memory(conn, *id).ok())
        .collect();

    Ok(SyncDelta {
        created,
        updated,
        deleted: deleted_ids.into_iter().collect(),
        from_version: since_version,
        to_version: current_version,
    })
}

/// Agent sync state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSyncState {
    pub agent_id: String,
    pub last_sync_version: i64,
    pub last_sync_time: DateTime<Utc>,
    pub pending_changes: i64,
}

/// Get sync state for a specific agent
pub fn get_agent_sync_state(conn: &Connection, agent_id: &str) -> Result<AgentSyncState> {
    let result: std::result::Result<(i64, String), rusqlite::Error> = conn.query_row(
        "SELECT last_sync_version, last_sync_time FROM agent_sync_state WHERE agent_id = ?",
        params![agent_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    );

    match result {
        Ok((version, time_str)) => {
            let current_version = get_sync_version(conn)?.version;
            let pending = (current_version - version).max(0);

            Ok(AgentSyncState {
                agent_id: agent_id.to_string(),
                last_sync_version: version,
                last_sync_time: DateTime::parse_from_rfc3339(&time_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                pending_changes: pending,
            })
        }
        Err(_) => {
            // No sync state yet for this agent
            Ok(AgentSyncState {
                agent_id: agent_id.to_string(),
                last_sync_version: 0,
                last_sync_time: Utc::now(),
                pending_changes: get_sync_version(conn)?.version,
            })
        }
    }
}

/// Update sync state for an agent
pub fn update_agent_sync_state(conn: &Connection, agent_id: &str, version: i64) -> Result<()> {
    let now = Utc::now();
    conn.execute(
        "INSERT INTO agent_sync_state (agent_id, last_sync_version, last_sync_time)
         VALUES (?, ?, ?)
         ON CONFLICT(agent_id) DO UPDATE SET
            last_sync_version = excluded.last_sync_version,
            last_sync_time = excluded.last_sync_time",
        params![agent_id, version, now.to_rfc3339()],
    )?;
    Ok(())
}

/// Cleanup old sync data
pub fn cleanup_sync_data(conn: &Connection, older_than_days: i64) -> Result<i64> {
    let cutoff = Utc::now() - chrono::Duration::days(older_than_days);
    let deleted = conn.execute(
        "DELETE FROM memory_events WHERE created_at < ?",
        params![cutoff.to_rfc3339()],
    )?;
    Ok(deleted as i64)
}

// =============================================================================
// Multi-Agent Sharing
// =============================================================================

/// A shared memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMemory {
    pub id: i64,
    pub memory_id: i64,
    pub from_agent: String,
    pub to_agent: String,
    pub message: Option<String>,
    pub acknowledged: bool,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Share a memory with another agent
pub fn share_memory(
    conn: &Connection,
    memory_id: i64,
    from_agent: &str,
    to_agent: &str,
    message: Option<&str>,
) -> Result<i64> {
    let now = Utc::now();

    // Verify memory exists
    let _ = get_memory(conn, memory_id)?;

    conn.execute(
        "INSERT INTO shared_memories (memory_id, from_agent, to_agent, message, acknowledged, created_at)
         VALUES (?, ?, ?, ?, 0, ?)",
        params![memory_id, from_agent, to_agent, message, now.to_rfc3339()],
    )?;

    let share_id = conn.last_insert_rowid();

    // Record event
    record_event(
        conn,
        MemoryEventType::Shared,
        Some(memory_id),
        Some(from_agent),
        serde_json::json!({
            "to_agent": to_agent,
            "share_id": share_id,
            "message": message
        }),
    )?;

    Ok(share_id)
}

/// Poll for shared memories sent to this agent
pub fn poll_shared_memories(
    conn: &Connection,
    to_agent: &str,
    include_acknowledged: bool,
) -> Result<Vec<SharedMemory>> {
    let query = if include_acknowledged {
        "SELECT id, memory_id, from_agent, to_agent, message, acknowledged, acknowledged_at, created_at
         FROM shared_memories WHERE to_agent = ?
         ORDER BY created_at DESC"
    } else {
        "SELECT id, memory_id, from_agent, to_agent, message, acknowledged, acknowledged_at, created_at
         FROM shared_memories WHERE to_agent = ? AND acknowledged = 0
         ORDER BY created_at DESC"
    };

    let mut stmt = conn.prepare(query)?;
    let shares = stmt
        .query_map(params![to_agent], |row| {
            let created_str: String = row.get(7)?;
            let ack_str: Option<String> = row.get(6)?;
            Ok(SharedMemory {
                id: row.get(0)?,
                memory_id: row.get(1)?,
                from_agent: row.get(2)?,
                to_agent: row.get(3)?,
                message: row.get(4)?,
                acknowledged: row.get(5)?,
                acknowledged_at: ack_str.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                }),
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(shares)
}

/// Acknowledge a shared memory
pub fn acknowledge_share(conn: &Connection, share_id: i64, agent_id: &str) -> Result<()> {
    let now = Utc::now();
    let affected = conn.execute(
        "UPDATE shared_memories SET acknowledged = 1, acknowledged_at = ?
         WHERE id = ? AND to_agent = ?",
        params![now.to_rfc3339(), share_id, agent_id],
    )?;

    if affected == 0 {
        return Err(EngramError::NotFound(share_id));
    }

    Ok(())
}
