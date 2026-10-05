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

use crate::error::{EngramError, Result};

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

/// Upper bound for a caller-supplied TTL. Larger values are clamped (not
/// rejected) so an out-of-range request cannot overflow the date arithmetic.
pub const MAX_TTL_DAYS: i64 = 365;

/// Largest payload (bytes) `enqueue` accepts. Oversize payloads are rejected
/// with `InvalidInput`; nothing is truncated here because the payload is an
/// opaque, already-serialised string and cutting it would corrupt it.
pub const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

/// Most rows a single `drain_for_workspace` call returns. The remainder stays
/// queued (oldest first) for the next SessionStart, so an unbounded backlog
/// can never turn into an unbounded injection.
pub const MAX_DRAIN_ROWS: usize = 50;

fn expiry_from_ttl(ttl_days: Option<i64>) -> (DateTime<Utc>, DateTime<Utc>) {
    let now = Utc::now();
    let ttl = ttl_days.unwrap_or(DEFAULT_TTL_DAYS).clamp(1, MAX_TTL_DAYS);
    (now, now + Duration::days(ttl))
}

fn check_payload_size(payload: &str) -> Result<()> {
    if payload.len() > MAX_PAYLOAD_BYTES {
        return Err(EngramError::InvalidInput(format!(
            "pending injection payload is {} bytes; limit is {} bytes",
            payload.len(),
            MAX_PAYLOAD_BYTES
        )));
    }
    Ok(())
}

/// Push a payload onto the queue. Returns the new row id.
///
/// `payload` is expected to be a JSON string the caller already serialised;
/// keeping it opaque at this layer lets the schema stay stable as the
/// payload shape evolves. Payloads above [`MAX_PAYLOAD_BYTES`] are rejected
/// and `ttl_days` is clamped to `1..=MAX_TTL_DAYS`.
pub fn enqueue(
    conn: &Connection,
    workspace: &str,
    payload: &str,
    source_session_id: Option<&str>,
    ttl_days: Option<i64>,
) -> Result<i64> {
    check_payload_size(payload)?;
    let (now, expires_at) = expiry_from_ttl(ttl_days);

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

/// Like [`enqueue`], but a no-op (`Ok(None)`) while a non-expired row for the
/// same `(workspace, source_session_id)` is still queued. This makes a
/// replayed or retried SessionEnd idempotent. The check and the insert are one
/// SQL statement, so concurrent producers cannot both insert.
///
/// Only *queued* rows are remembered: once a row has been drained the same
/// session id may enqueue again. Delivery is at-most-once, not exactly-once.
pub fn enqueue_once_for_session(
    conn: &Connection,
    workspace: &str,
    payload: &str,
    source_session_id: &str,
    ttl_days: Option<i64>,
) -> Result<Option<i64>> {
    check_payload_size(payload)?;
    let (now, expires_at) = expiry_from_ttl(ttl_days);
    let now_text = now.to_rfc3339();

    let inserted = conn.execute(
        "INSERT INTO pending_injections (workspace, payload, source_session_id, created_at, expires_at)
         SELECT ?1, ?2, ?3, ?4, ?5
         WHERE NOT EXISTS (
             SELECT 1 FROM pending_injections
             WHERE workspace = ?1 AND source_session_id = ?3 AND expires_at > ?4
         )",
        params![
            workspace,
            payload,
            source_session_id,
            now_text,
            expires_at.to_rfc3339(),
        ],
    )?;
    Ok((inserted > 0).then(|| conn.last_insert_rowid()))
}

/// Read-and-delete up to [`MAX_DRAIN_ROWS`] non-expired rows for a workspace,
/// oldest first.
///
/// The selection and the delete are a single `DELETE ... RETURNING`
/// statement, so a row handed to the caller is already gone and two
/// concurrent consumers (even on separate connections or processes) receive
/// disjoint sets. This also works when the caller already holds a
/// transaction. If the caller drops the returned rows (crash, abort by a later
/// hook) they are lost: delivery is at-most-once.
pub fn drain_for_workspace(conn: &Connection, workspace: &str) -> Result<Vec<PendingInjection>> {
    let now = Utc::now().to_rfc3339();
    let mut stmt = conn.prepare(
        "DELETE FROM pending_injections
         WHERE id IN (
             SELECT id FROM pending_injections
             WHERE workspace = ?1 AND expires_at > ?2
             ORDER BY created_at ASC, id ASC
             LIMIT ?3
         )
         RETURNING id, workspace, payload, source_session_id, created_at, expires_at",
    )?;
    let mut rows: Vec<PendingInjection> = stmt
        .query_map(params![workspace, now, MAX_DRAIN_ROWS as i64], |row| {
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

    // RETURNING does not promise an order.
    rows.sort_by(|a, b| a.created_at.cmp(&b.created_at).then(a.id.cmp(&b.id)));
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
        )
        .unwrap();
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
        )
        .unwrap();
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

    // ---- C6 failure / bounds tests ------------------------------------

    #[test]
    fn huge_ttl_is_clamped_instead_of_panicking() {
        let c = conn();
        let id = enqueue(&c, "ws", "x", None, Some(i64::MAX)).unwrap();
        assert!(id > 0);
        let expires: String = c
            .query_row(
                "SELECT expires_at FROM pending_injections WHERE id = ?",
                params![id],
                |row| row.get(0),
            )
            .unwrap();
        let expires = DateTime::parse_from_rfc3339(&expires).unwrap();
        let horizon = Utc::now() + Duration::days(MAX_TTL_DAYS + 1);
        assert!(expires.with_timezone(&Utc) < horizon);
    }

    #[test]
    fn oversize_payload_is_rejected_without_enqueue() {
        let c = conn();
        let big = "x".repeat(MAX_PAYLOAD_BYTES + 1);
        let err = enqueue(&c, "ws", &big, Some("s"), None).unwrap_err();
        assert!(matches!(err, crate::error::EngramError::InvalidInput(_)));
        assert_eq!(pending_count(&c, "ws").unwrap(), 0);
        // The boundary itself is accepted.
        let edge = "x".repeat(MAX_PAYLOAD_BYTES);
        enqueue(&c, "ws", &edge, None, None).unwrap();
        assert_eq!(pending_count(&c, "ws").unwrap(), 1);
    }

    #[test]
    fn drain_is_bounded_and_keeps_the_rest_in_fifo_order() {
        let c = conn();
        let total = MAX_DRAIN_ROWS + 5;
        for i in 0..total {
            enqueue(&c, "ws", &format!("p{i:03}"), None, None).unwrap();
        }
        let first = drain_for_workspace(&c, "ws").unwrap();
        assert_eq!(first.len(), MAX_DRAIN_ROWS);
        assert_eq!(first[0].payload, "p000");
        assert_eq!(pending_count(&c, "ws").unwrap(), 5);

        let second = drain_for_workspace(&c, "ws").unwrap();
        assert_eq!(second.len(), 5);
        assert_eq!(second[0].payload, format!("p{:03}", MAX_DRAIN_ROWS));
        assert_eq!(pending_count(&c, "ws").unwrap(), 0);
    }

    #[test]
    fn enqueue_for_session_is_idempotent_per_workspace_and_session() {
        let c = conn();
        let first = enqueue_once_for_session(&c, "ws", "a", "sess-1", None).unwrap();
        assert!(first.is_some());
        let replay = enqueue_once_for_session(&c, "ws", "a-again", "sess-1", None).unwrap();
        assert_eq!(replay, None, "a replayed SessionEnd must not duplicate");
        assert_eq!(pending_count(&c, "ws").unwrap(), 1);

        // Other session / other workspace are independent.
        assert!(enqueue_once_for_session(&c, "ws", "b", "sess-2", None)
            .unwrap()
            .is_some());
        assert!(enqueue_once_for_session(&c, "other", "c", "sess-1", None)
            .unwrap()
            .is_some());

        // After the row is consumed the same session may enqueue again
        // (consumed rows are not remembered; delivery is at-most-once).
        drain_for_workspace(&c, "ws").unwrap();
        assert!(enqueue_once_for_session(&c, "ws", "d", "sess-1", None)
            .unwrap()
            .is_some());
    }

    #[test]
    fn concurrent_consumers_never_receive_the_same_row() {
        use std::sync::{Arc, Barrier};
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("pi.db");
        let seed = Connection::open(&path).unwrap();
        seed.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
        run_migrations(&seed).unwrap();
        let total = 400usize;
        for i in 0..total {
            enqueue(&seed, "ws", &format!("p{i}"), None, None).unwrap();
        }

        let barrier = Arc::new(Barrier::new(4));
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let path = path.clone();
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    let c = Connection::open(&path).unwrap();
                    c.busy_timeout(std::time::Duration::from_secs(30)).unwrap();
                    barrier.wait();
                    let mut got = Vec::new();
                    loop {
                        let rows = drain_for_workspace(&c, "ws").unwrap();
                        if rows.is_empty() {
                            break;
                        }
                        got.extend(rows.into_iter().map(|r| r.id));
                    }
                    got
                })
            })
            .collect();
        let mut all: Vec<i64> = handles
            .into_iter()
            .flat_map(|h| h.join().unwrap())
            .collect();
        all.sort_unstable();
        let before = all.len();
        all.dedup();
        assert_eq!(before, all.len(), "a row was delivered to two consumers");
        assert_eq!(all.len(), total, "a row was lost");
    }
}
