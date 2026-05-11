//! Queue of injection payloads bridging SessionEnd → next SessionStart.
//!
//! The two hooks live in different lifecycle phases and cannot communicate
//! via in-memory state — by the time SessionStart fires, the SessionEnd
//! handler is long dead. They synchronise through this small SQL queue.
//!
//! Producer: `SessionEnd` builds an injection payload (relevant memories
//! summarised, topic list, etc.) and calls `enqueue` with the *current*
//! session's workspace.
//!
//! Consumer: `SessionStart` calls `drain_for_workspace`, which atomically
//! returns the rows that match the workspace, removes them from the table,
//! and lets the handler shape them into an injection prompt.

use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// A queued payload waiting to be consumed by the next SessionStart for
/// a given workspace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingInjection {
    pub id: i64,
    pub workspace: String,
    pub payload: String,
    pub source_session_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

/// Default TTL: a queued injection that nobody consumed in a week is
/// almost certainly stale and should be dropped on the next cleanup.
const DEFAULT_TTL_DAYS: i64 = 7;

/// Push a payload onto the queue. Returns the new row id.
///
/// `payload` is expected to be a JSON string the caller already serialised;
/// keeping it opaque at this layer lets the schema stay stable as the
/// payload shape evolves.
pub fn enqueue(
    conn: &Connection,
    workspace: &str,
    payload: &str,
    source_session_id: Option<&str>,
    ttl_days: Option<i64>,
) -> Result<i64> {
    let now = Utc::now();
    let ttl = ttl_days.unwrap_or(DEFAULT_TTL_DAYS).max(1);
    let expires_at = now + Duration::days(ttl);

    conn.execute(
        "INSERT INTO pending_injections (workspace, payload, source_session_id, created_at, expires_at)
         VALUES (?, ?, ?, ?, ?)",
        params![
            workspace,
            payload,
            source_session_id,
            now.to_rfc3339(),
            expires_at.to_rfc3339(),
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Read-and-delete every non-expired row for a workspace, oldest first.
///
/// Atomic via a single transaction: a row returned to the caller is
/// already gone from the table. Two concurrent consumers will each see a
/// disjoint set (SQLite serialises the transaction).
pub fn drain_for_workspace(conn: &Connection, workspace: &str) -> Result<Vec<PendingInjection>> {
    let now = Utc::now().to_rfc3339();
    let mut stmt = conn.prepare(
        "SELECT id, workspace, payload, source_session_id, created_at, expires_at
         FROM pending_injections
         WHERE workspace = ? AND expires_at > ?
         ORDER BY created_at ASC",
    )?;
    let rows: Vec<PendingInjection> = stmt
        .query_map(params![workspace, now], |row| {
            let created_at: String = row.get(4)?;
            let expires_at: String = row.get(5)?;
            Ok(PendingInjection {
                id: row.get(0)?,
                workspace: row.get(1)?,
                payload: row.get(2)?,
                source_session_id: row.get(3)?,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                expires_at: DateTime::parse_from_rfc3339(&expires_at)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    if !rows.is_empty() {
        let ids: Vec<String> = rows.iter().map(|r| r.id.to_string()).collect();
        let sql = format!(
            "DELETE FROM pending_injections WHERE id IN ({})",
            ids.join(",")
        );
        conn.execute(&sql, [])?;
    }
    Ok(rows)
}

/// Delete every row whose `expires_at` has passed. Returns the count
/// removed. Safe to call as often as you like — idempotent.
pub fn cleanup_expired(conn: &Connection) -> Result<usize> {
    let now = Utc::now().to_rfc3339();
    let n = conn.execute(
        "DELETE FROM pending_injections WHERE expires_at <= ?",
        params![now],
    )?;
    Ok(n)
}

/// Count of unexpired rows for a workspace. Mostly useful for tests and
/// for the future MCP read tool.
pub fn pending_count(conn: &Connection, workspace: &str) -> Result<i64> {
    let now = Utc::now().to_rfc3339();
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pending_injections WHERE workspace = ? AND expires_at > ?",
        params![workspace, now],
        |row| row.get(0),
    )?;
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::migrations::run_migrations;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        run_migrations(&c).unwrap();
        c
    }

    #[test]
    fn enqueue_then_drain_returns_payload_and_clears_row() {
        let c = conn();
        let id = enqueue(&c, "default", r#"{"k":"v"}"#, Some("sess-1"), None).unwrap();
        assert!(id > 0);
        assert_eq!(pending_count(&c, "default").unwrap(), 1);

        let drained = drain_for_workspace(&c, "default").unwrap();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].payload, r#"{"k":"v"}"#);
        assert_eq!(drained[0].source_session_id.as_deref(), Some("sess-1"));
        assert_eq!(pending_count(&c, "default").unwrap(), 0);
    }

    #[test]
    fn drain_returns_fifo_within_workspace() {
        let c = conn();
        enqueue(&c, "ws", "first", None, None).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(5));
        enqueue(&c, "ws", "second", None, None).unwrap();

        let drained = drain_for_workspace(&c, "ws").unwrap();
        assert_eq!(drained.len(), 2);
        assert_eq!(drained[0].payload, "first");
        assert_eq!(drained[1].payload, "second");
    }

    #[test]
    fn drain_only_returns_matching_workspace() {
        let c = conn();
        enqueue(&c, "alpha", "a-payload", None, None).unwrap();
        enqueue(&c, "beta", "b-payload", None, None).unwrap();

        let drained = drain_for_workspace(&c, "alpha").unwrap();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].payload, "a-payload");
        // beta row untouched
        assert_eq!(pending_count(&c, "beta").unwrap(), 1);
    }

    #[test]
    fn expired_rows_are_skipped_by_drain() {
        let c = conn();
        // Insert with explicit expires_at in the past
        let past = (Utc::now() - Duration::days(1)).to_rfc3339();
        c.execute(
            "INSERT INTO pending_injections (workspace, payload, created_at, expires_at)
             VALUES ('ws', 'stale', ?, ?)",
            params![past.clone(), past],
        ).unwrap();
        // And a fresh row
        enqueue(&c, "ws", "fresh", None, None).unwrap();

        let drained = drain_for_workspace(&c, "ws").unwrap();
        assert_eq!(drained.len(), 1);
        assert_eq!(drained[0].payload, "fresh");
    }

    #[test]
    fn cleanup_expired_removes_only_expired() {
        let c = conn();
        let past = (Utc::now() - Duration::days(1)).to_rfc3339();
        c.execute(
            "INSERT INTO pending_injections (workspace, payload, created_at, expires_at)
             VALUES ('ws', 'old', ?, ?)",
            params![past.clone(), past],
        ).unwrap();
        enqueue(&c, "ws", "new", None, None).unwrap();

        let removed = cleanup_expired(&c).unwrap();
        assert_eq!(removed, 1);
        assert_eq!(pending_count(&c, "ws").unwrap(), 1);
    }

    #[test]
    fn ttl_override_respected() {
        let c = conn();
        let id = enqueue(&c, "ws", "x", None, Some(0)).unwrap();
        // ttl is clamped to >= 1 day, so this row should still be alive.
        assert!(id > 0);
        assert_eq!(pending_count(&c, "ws").unwrap(), 1);
    }
}
