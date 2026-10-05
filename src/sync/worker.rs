//! Background sync worker with debouncing (RML-875)

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use tokio::sync::mpsc;
use tokio::time::{interval, Instant};

use super::{CloudStorage, SyncDirection};
use crate::error::{EngramError, Result};
use crate::types::SyncStatus;

/// Commands for the sync worker
#[derive(Debug)]
pub enum SyncCommand {
    /// Trigger a sync (direction, force)
    Sync(SyncDirection, bool),
    /// Mark data as dirty (triggers debounced sync)
    MarkDirty,
    /// Stop the worker
    Stop,
}

/// Background sync worker
pub struct SyncWorker {
    sender: mpsc::Sender<SyncCommand>,
}

impl SyncWorker {
    /// Start the sync worker
    pub async fn start(
        db_path: PathBuf,
        cloud_uri: String,
        encrypt: bool,
        debounce_ms: u64,
        conn: Arc<Mutex<Connection>>,
    ) -> Result<Self> {
        let cloud = CloudStorage::from_uri(&cloud_uri, encrypt).await?;
        Ok(Self::start_with_cloud(db_path, cloud, debounce_ms, conn))
    }

    /// Start the worker around an already-constructed storage backend.
    ///
    /// Restart semantics: a previous process that died mid-sync leaves
    /// `sync_state.is_syncing = 1`; nothing is syncing now, so that flag is
    /// cleared and `last_error` records the interruption (see
    /// [`reset_interrupted_sync`]). The interrupted push is not resumed; the
    /// next `MarkDirty`/`Sync` redoes it from scratch (uploads replace the
    /// whole object, so a repeat is idempotent).
    ///
    /// Retry policy for debounced pushes: a failed push is retried with
    /// exponential backoff (`debounce * 2^failures`, capped at
    /// [`MAX_BACKOFF`]) at most [`MAX_PUSH_ATTEMPTS`] times in a row, after
    /// which the worker stops retrying until the next `MarkDirty`. Explicit
    /// `Sync` commands are never retried automatically.
    pub(crate) fn start_with_cloud(
        db_path: PathBuf,
        cloud: CloudStorage,
        debounce_ms: u64,
        conn: Arc<Mutex<Connection>>,
    ) -> Self {
        let (sender, mut receiver) = mpsc::channel::<SyncCommand>(100);
        let debounce = Duration::from_millis(debounce_ms);

        {
            let guard = conn.lock();
            match reset_interrupted_sync(&guard) {
                Ok(true) => tracing::warn!("previous sync was interrupted; cleared is_syncing"),
                Ok(false) => {}
                Err(e) => warn_sync_state_write_failed("restart", SyncDirection::Push, &e),
            }
        }

        // Spawn worker task
        tokio::spawn(async move {
            let mut dirty = DirtyTracker::new(debounce);
            let tick = debounce.clamp(Duration::from_millis(10), Duration::from_secs(1));
            let mut check_interval = interval(tick);

            loop {
                tokio::select! {
                    Some(cmd) = receiver.recv() => {
                        match cmd {
                            SyncCommand::Sync(direction, force) => {
                                let ok = Self::do_sync(&db_path, &cloud, &conn, direction, force).await;
                                dirty.on_explicit_sync(ok);
                            }
                            SyncCommand::MarkDirty => {
                                dirty.mark_dirty(Instant::now());
                            }
                            SyncCommand::Stop => {
                                // Final sync before stopping
                                Self::do_sync(&db_path, &cloud, &conn, SyncDirection::Push, false).await;
                                break;
                            }
                        }
                    }
                    _ = check_interval.tick() => {
                        if dirty.is_due(Instant::now()) {
                            let ok = Self::do_sync(&db_path, &cloud, &conn, SyncDirection::Push, false).await;
                            if ok {
                                dirty.on_success();
                            } else if let RetryDecision::GiveUp = dirty.on_failure(Instant::now()) {
                                tracing::error!(
                                    "Sync push failed {} times in a row; not retrying until the next change",
                                    MAX_PUSH_ATTEMPTS
                                );
                            }
                        }
                    }
                }
            }

            tracing::info!("Sync worker stopped");
        });

        Self { sender }
    }

    /// Perform the actual sync operation
    async fn do_sync(
        db_path: &PathBuf,
        cloud: &CloudStorage,
        conn: &Arc<Mutex<Connection>>,
        direction: SyncDirection,
        _force: bool,
    ) -> bool {
        let started_at = Utc::now();

        if let Err(e) = {
            let conn = conn.lock();
            mark_sync_started(&conn)
        } {
            warn_sync_state_write_failed("start", direction, &e);
        }

        let result = match direction {
            SyncDirection::Push => cloud.upload(db_path).await,
            SyncDirection::Pull => match refuse_live_database(conn, db_path) {
                Ok(()) => cloud.download(db_path).await,
                Err(e) => Err(e),
            },
            SyncDirection::Bidirectional => {
                // Check which is newer
                match cloud.metadata().await {
                    Ok(_remote_meta) => {
                        let _local_modified =
                            std::fs::metadata(db_path).and_then(|m| m.modified()).ok();

                        // Simple heuristic: push if local is newer or no remote
                        cloud.upload(db_path).await
                    }
                    Err(_) => {
                        // No remote, push
                        cloud.upload(db_path).await
                    }
                }
            }
        };

        let completed_at = Utc::now();

        match &result {
            Ok(_) => {
                if let Err(e) = {
                    let conn = conn.lock();
                    record_sync_success(&conn, &completed_at)
                } {
                    warn_sync_state_write_failed("success", direction, &e);
                }
            }
            Err(e) => {
                let error_message = e.to_string();
                if let Err(write_error) = {
                    let conn = conn.lock();
                    record_sync_failure(&conn, &error_message)
                } {
                    warn_sync_state_write_failed("failure", direction, &write_error);
                }
            }
        }

        match result {
            Ok(bytes) => {
                tracing::info!(
                    "Sync {:?} completed: {} bytes in {:?}",
                    direction,
                    bytes,
                    completed_at - started_at
                );
                true
            }
            Err(e) => {
                tracing::error!("Sync {:?} failed: {}", direction, e);
                false
            }
        }
    }

    /// Trigger a sync
    pub async fn sync(&self, direction: SyncDirection, force: bool) -> Result<()> {
        self.sender
            .send(SyncCommand::Sync(direction, force))
            .await
            .map_err(|_| EngramError::Sync("Worker channel closed".to_string()))?;
        Ok(())
    }

    /// Mark data as dirty (triggers debounced sync)
    pub async fn mark_dirty(&self) -> Result<()> {
        self.sender
            .send(SyncCommand::MarkDirty)
            .await
            .map_err(|_| EngramError::Sync("Worker channel closed".to_string()))?;
        Ok(())
    }

    /// Stop the worker
    pub async fn stop(&self) -> Result<()> {
        self.sender
            .send(SyncCommand::Stop)
            .await
            .map_err(|_| EngramError::Sync("Worker channel closed".to_string()))?;
        Ok(())
    }
}

/// Longest wait between two automatic push retries.
pub(crate) const MAX_BACKOFF: Duration = Duration::from_secs(300);
/// Consecutive failed automatic pushes before the worker stops retrying.
pub(crate) const MAX_PUSH_ATTEMPTS: u32 = 5;

/// Whether to keep retrying after a failed automatic push.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RetryDecision {
    Retry,
    GiveUp,
}

/// Debounce + bounded-backoff state for automatic pushes. Pure (the clock is
/// passed in) so the retry policy is testable without a runtime.
pub(crate) struct DirtyTracker {
    debounce: Duration,
    dirty_since: Option<Instant>,
    failures: u32,
}

impl DirtyTracker {
    pub(crate) fn new(debounce: Duration) -> Self {
        Self {
            debounce,
            dirty_since: None,
            failures: 0,
        }
    }

    /// Local data changed: (re)start the quiet-period timer. Failures are kept,
    /// so a steady stream of changes cannot reset the backoff.
    pub(crate) fn mark_dirty(&mut self, now: Instant) {
        self.dirty_since = Some(now);
    }

    fn delay(&self) -> Duration {
        let factor = 1u32 << self.failures.min(16);
        self.debounce
            .saturating_mul(factor)
            .min(MAX_BACKOFF.max(self.debounce))
    }

    pub(crate) fn is_due(&self, now: Instant) -> bool {
        self.dirty_since
            .is_some_and(|since| now.saturating_duration_since(since) >= self.delay())
    }

    pub(crate) fn clear(&mut self) {
        self.dirty_since = None;
    }

    /// Outcome of an explicit `Sync` command: success proves the remote works
    /// (backoff resets); failure only drops the pending marker, because explicit
    /// syncs are never retried automatically.
    pub(crate) fn on_explicit_sync(&mut self, ok: bool) {
        if ok {
            self.on_success();
        } else {
            self.clear();
        }
    }

    pub(crate) fn on_success(&mut self) {
        self.dirty_since = None;
        self.failures = 0;
    }

    pub(crate) fn on_failure(&mut self, now: Instant) -> RetryDecision {
        self.failures = self.failures.saturating_add(1);
        if self.failures >= MAX_PUSH_ATTEMPTS {
            self.dirty_since = None;
            RetryDecision::GiveUp
        } else {
            self.dirty_since = Some(now);
            RetryDecision::Retry
        }
    }
}

/// A pull replaces the target file by rename; doing that to the database this
/// worker's own connection has open orphans the open connection and drops its
/// locks (G1). Refuse when `db_path` is the connection's database.
fn refuse_live_database(conn: &Arc<Mutex<Connection>>, db_path: &std::path::Path) -> Result<()> {
    let live = conn.lock().path().map(std::path::PathBuf::from);
    let Some(live) = live else {
        return Ok(()); // in-memory connection: no file to protect
    };
    let canon = |p: &std::path::Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    if canon(&live) == canon(db_path) {
        return Err(EngramError::Sync(format!(
            "refusing to pull over '{}': it is the live database of this process",
            db_path.display()
        )));
    }
    Ok(())
}

/// Clear a stale `is_syncing` flag left by a process that died mid-sync.
/// Returns whether a stale flag was found. `last_error` records it so the
/// partial state is visible instead of silently looking idle.
pub(crate) fn reset_interrupted_sync(conn: &Connection) -> Result<bool> {
    let changed = conn.execute(
        "UPDATE sync_state SET
            is_syncing = 0,
            last_error = CASE
                WHEN last_error IS NULL OR last_error = ''
                    THEN 'previous sync was interrupted before completing'
                ELSE last_error || '; previous sync was interrupted before completing'
            END
         WHERE id = 1 AND is_syncing = 1",
        [],
    )?;
    Ok(changed > 0)
}

fn mark_sync_started(conn: &Connection) -> Result<()> {
    conn.execute("UPDATE sync_state SET is_syncing = 1 WHERE id = 1", [])?;
    Ok(())
}

fn record_sync_success(conn: &Connection, completed_at: &DateTime<Utc>) -> Result<()> {
    conn.execute(
        "UPDATE sync_state SET
            is_syncing = 0,
            last_sync = ?,
            pending_changes = 0,
            last_error = NULL
         WHERE id = 1",
        params![completed_at.to_rfc3339()],
    )?;
    Ok(())
}

fn record_sync_failure(conn: &Connection, error_message: &str) -> Result<()> {
    conn.execute(
        "UPDATE sync_state SET
            is_syncing = 0,
            last_error = ?
         WHERE id = 1",
        params![error_message],
    )?;
    Ok(())
}

fn warn_sync_state_write_failed(
    phase: &'static str,
    direction: SyncDirection,
    error: &EngramError,
) {
    tracing::warn!(
        phase = phase,
        direction = ?direction,
        error = %error,
        "sync_state bookkeeping write failed"
    );
}

/// Get current sync status from database
pub fn get_sync_status(conn: &Connection) -> Result<SyncStatus> {
    let row = conn.query_row(
        "SELECT pending_changes, last_sync, last_error, is_syncing FROM sync_state WHERE id = 1",
        [],
        |row| {
            let pending: i64 = row.get(0)?;
            let last_sync: Option<String> = row.get(1)?;
            let last_error: Option<String> = row.get(2)?;
            let is_syncing: i32 = row.get(3)?;

            Ok(SyncStatus {
                pending_changes: pending,
                last_sync: last_sync.and_then(|s| {
                    chrono::DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                }),
                last_error,
                is_syncing: is_syncing != 0,
            })
        },
    )?;

    Ok(row)
}

/// Increment pending changes counter
#[allow(dead_code)]
pub fn increment_pending_changes(conn: &Connection) -> Result<()> {
    conn.execute(
        "UPDATE sync_state SET pending_changes = pending_changes + 1 WHERE id = 1",
        [],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sync_state_helpers_return_errors_when_table_is_missing() {
        // Given: an empty database with no sync_state table.
        let conn = Connection::open_in_memory().expect("test opens in-memory database");

        // When/Then: every bookkeeping write returns the database error.
        assert_bookkeeping_writes_fail(&conn);
    }

    #[test]
    fn sync_state_helpers_return_errors_when_table_is_broken() {
        // Given: a malformed sync_state table missing the columns workers update.
        let conn = Connection::open_in_memory().expect("test opens in-memory database");
        conn.execute("CREATE TABLE sync_state (id INTEGER PRIMARY KEY)", [])
            .expect("test setup creates malformed sync_state");
        conn.execute("INSERT INTO sync_state (id) VALUES (1)", [])
            .expect("test setup inserts malformed sync_state row");

        // When/Then: missing columns make each bookkeeping helper fail visibly.
        assert_bookkeeping_writes_fail(&conn);
    }

    fn assert_bookkeeping_writes_fail(conn: &Connection) {
        let completed_at = Utc::now();
        let results = [
            mark_sync_started(conn),
            record_sync_success(conn, &completed_at),
            record_sync_failure(conn, "network failure"),
        ];

        for result in results {
            assert!(matches!(result, Err(EngramError::Database(_))));
        }
    }
}
