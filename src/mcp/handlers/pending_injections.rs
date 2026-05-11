//! MCP handlers for the SessionEnd → SessionStart injection queue.
//!
//! Three read/maintenance tools — none of them affect the hook
//! producer/consumer paths directly. They exist so an operator can see
//! what's queued, drop stale rows, and clear a workspace if a payload
//! is misbehaving.

use serde_json::{json, Value};

use super::HandlerContext;
use crate::storage::pending_injections;

/// Read-only: count of non-expired queued payloads for a workspace.
pub fn pending_injections_count(ctx: &HandlerContext, params: Value) -> Value {
    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    match ctx
        .storage
        .with_connection(|conn| pending_injections::pending_count(conn, workspace))
    {
        Ok(n) => json!({"workspace": workspace, "count": n}),
        Err(e) => json!({"error": e.to_string()}),
    }
}

/// Destructive: drop every row whose `expires_at` has passed across all
/// workspaces. Returns the number removed. Idempotent and cheap to call.
pub fn pending_injections_cleanup(ctx: &HandlerContext, _params: Value) -> Value {
    match ctx
        .storage
        .with_connection(|conn| pending_injections::cleanup_expired(conn))
    {
        Ok(n) => json!({"removed": n}),
        Err(e) => json!({"error": e.to_string()}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::pending_injections::enqueue;
    use chrono::Duration as ChronoDuration;
    use chrono::Utc;
    use rusqlite::params;

    fn open_storage() -> crate::storage::Storage {
        crate::storage::Storage::open_in_memory().unwrap()
    }

    #[test]
    fn count_returns_zero_for_empty_workspace() {
        let storage = open_storage();
        let n: i64 = storage
            .with_connection(|c| pending_injections::pending_count(c, "ws"))
            .unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn count_reflects_enqueued_rows() {
        let storage = open_storage();
        storage
            .with_connection(|c| {
                enqueue(c, "ws", "x", None, None)?;
                enqueue(c, "ws", "y", None, None)?;
                Ok(())
            })
            .unwrap();
        let n = storage
            .with_connection(|c| pending_injections::pending_count(c, "ws"))
            .unwrap();
        assert_eq!(n, 2);
    }

    #[test]
    fn cleanup_removes_only_expired() {
        let storage = open_storage();
        let past = (Utc::now() - ChronoDuration::days(1)).to_rfc3339();
        storage
            .with_connection(|c| {
                c.execute(
                    "INSERT INTO pending_injections (workspace, payload, created_at, expires_at)
                     VALUES ('ws', 'stale', ?, ?)",
                    params![past.clone(), past],
                )?;
                enqueue(c, "ws", "fresh", None, None)?;
                Ok(())
            })
            .unwrap();
        let removed = storage
            .with_connection(|c| pending_injections::cleanup_expired(c))
            .unwrap();
        assert_eq!(removed, 1);
        let leftover = storage
            .with_connection(|c| pending_injections::pending_count(c, "ws"))
            .unwrap();
        assert_eq!(leftover, 1);
    }
}
