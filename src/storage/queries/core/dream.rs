use super::*;

/// Insert a Dream Phase run report (Phase D - Issue #12)
#[cfg(feature = "dream-phase")]
pub fn insert_dream_run(conn: &Connection, report: &crate::dream::DreamReport) -> Result<i64> {
    let report_json =
        serde_json::to_string(report).map_err(|e| EngramError::Internal(e.to_string()))?;

    conn.execute(
        "INSERT INTO dream_runs (started_at, finished_at, report_json, error_count, workspace_count)
         VALUES (?, ?, ?, ?, ?)",
        params![
            report.started_at.to_rfc3339(),
            report.finished_at.to_rfc3339(),
            report_json,
            report.errors.len() as i32,
            report.workspaces.len() as i32,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Try to acquire an advisory lock (Phase D - Issue #12)
pub fn acquire_dream_lock(
    conn: &Connection,
    lock_id: &str,
    owner_id: &str,
    ttl_secs: u64,
) -> Result<bool> {
    let now = Utc::now();
    let expires_at = now + chrono::Duration::seconds(ttl_secs as i64);

    // Cleanup expired locks first
    conn.execute(
        "DELETE FROM dream_locks WHERE expires_at < ?",
        params![now.to_rfc3339()],
    )?;

    // Try to insert (will fail if lock_id exists and not expired)
    let res = conn.execute(
        "INSERT OR IGNORE INTO dream_locks (lock_id, acquired_at, expires_at, owner_id)
         VALUES (?, ?, ?, ?)",
        params![lock_id, now.to_rfc3339(), expires_at.to_rfc3339(), owner_id],
    )?;

    Ok(res > 0)
}

/// Release an advisory lock (Phase D - Issue #12)
pub fn release_dream_lock(conn: &Connection, lock_id: &str, owner_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM dream_locks WHERE lock_id = ? AND owner_id = ?",
        params![lock_id, owner_id],
    )?;
    Ok(())
}
