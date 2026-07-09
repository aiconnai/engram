use chrono::Utc;
use rusqlite::{params, Connection};

use crate::error::Result;

use super::{UpdateAction, UpdateLogEntry, UpdateResult};

// DDL
// =============================================================================

/// DDL for the `update_log` table.
///
/// Call once during schema setup (e.g., alongside `CREATE_FACTS_TABLE`).
pub const CREATE_UPDATE_LOG_TABLE: &str = r#"
    CREATE TABLE IF NOT EXISTS update_log (
        id         INTEGER PRIMARY KEY AUTOINCREMENT,
        memory_id  INTEGER NOT NULL,
        action     TEXT    NOT NULL,
        old_hash   TEXT    NOT NULL,
        new_hash   TEXT    NOT NULL,
        reason     TEXT    NOT NULL DEFAULT '',
        timestamp  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
    );
    CREATE INDEX IF NOT EXISTS idx_update_log_memory ON update_log(memory_id);
"#;

// =============================================================================
// Storage helpers
// =============================================================================

/// Insert one row into `update_log` and return the stored entry.
pub fn create_update_log(
    conn: &Connection,
    result: &UpdateResult,
    reason: &str,
) -> Result<UpdateLogEntry> {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

    conn.execute(
        "INSERT INTO update_log (memory_id, action, old_hash, new_hash, reason, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            result.memory_id,
            result.action_taken.as_str(),
            result.old_content_hash,
            result.new_content_hash,
            reason,
            now,
        ],
    )?;

    let id = conn.last_insert_rowid();

    Ok(UpdateLogEntry {
        id,
        memory_id: result.memory_id,
        action: result.action_taken,
        old_hash: result.old_content_hash.clone(),
        new_hash: result.new_content_hash.clone(),
        reason: reason.to_string(),
        timestamp: now,
    })
}

/// List update log entries, optionally filtered to a specific memory.
///
/// `limit = 0` means unlimited.
pub fn list_update_logs(
    conn: &Connection,
    memory_id: Option<i64>,
    limit: usize,
) -> Result<Vec<UpdateLogEntry>> {
    let effective_limit: i64 = if limit == 0 { i64::MAX } else { limit as i64 };

    let rows = match memory_id {
        Some(mid) => {
            let mut stmt = conn.prepare(
                "SELECT id, memory_id, action, old_hash, new_hash, reason, timestamp
                 FROM update_log
                 WHERE memory_id = ?1
                 ORDER BY id ASC
                 LIMIT ?2",
            )?;
            let x = stmt
                .query_map(params![mid, effective_limit], map_log_row)?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            x
        }
        None => {
            let mut stmt = conn.prepare(
                "SELECT id, memory_id, action, old_hash, new_hash, reason, timestamp
                 FROM update_log
                 ORDER BY id ASC
                 LIMIT ?1",
            )?;
            let x = stmt
                .query_map(params![effective_limit], map_log_row)?
                .collect::<std::result::Result<Vec<_>, _>>()?;
            x
        }
    };

    Ok(rows)
}

fn map_log_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<UpdateLogEntry> {
    let action_str: String = row.get(2)?;
    let action = action_str
        .parse::<UpdateAction>()
        .unwrap_or(UpdateAction::Flag);
    Ok(UpdateLogEntry {
        id: row.get(0)?,
        memory_id: row.get(1)?,
        action,
        old_hash: row.get(3)?,
        new_hash: row.get(4)?,
        reason: row.get(5)?,
        timestamp: row.get(6)?,
    })
}

// =============================================================================
