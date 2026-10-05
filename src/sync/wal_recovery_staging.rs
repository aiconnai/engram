//! Staged replay with atomic replacement of the recovery target (C2).
//!
//! Frames are written to a staging copy in the target's directory, fsynced,
//! integrity-checked and only then renamed over the target. Late I/O errors
//! and integrity failures therefore leave a pre-existing target untouched.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use rusqlite::Connection;

use super::wal_recovery::{RecoveryOptions, RecoveryReport};
use super::wal_replay_guard::{page_offset, MAX_ALLOWED_DB_PAGES};
use super::wal_replication::{WalFrame, WalReplicationError};

/// Where the staged database starts from before frames are applied.
#[derive(Clone, Copy)]
pub(crate) enum StageSeed<'a> {
    /// Copy of this file (base snapshot or PITR source).
    File(&'a Path),
    /// Copy of the current target if it exists, else empty (in-place replay).
    ExistingTarget,
    /// Consistent copy of a database this process has open, written by
    /// SQLite (`VACUUM INTO`), so no raw descriptor of it is opened (G1).
    ActiveStorage(&'a crate::storage::Storage),
}

impl std::fmt::Debug for StageSeed<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(path) => f.debug_tuple("File").field(path).finish(),
            Self::ExistingTarget => f.write_str("ExistingTarget"),
            Self::ActiveStorage(storage) => f
                .debug_tuple("ActiveStorage")
                .field(&storage.db_path())
                .finish(),
        }
    }
}

/// Page writer abstraction so tests can inject late I/O failures.
pub(crate) trait PageSink {
    fn write_at(&mut self, offset: u64, data: &[u8]) -> io::Result<()>;
    fn set_len(&mut self, len: u64) -> io::Result<()>;
    fn sync(&mut self) -> io::Result<()>;
}

impl PageSink for File {
    fn write_at(&mut self, offset: u64, data: &[u8]) -> io::Result<()> {
        self.seek(SeekFrom::Start(offset))?;
        self.write_all(data)
    }

    fn set_len(&mut self, len: u64) -> io::Result<()> {
        File::set_len(self, len)
    }

    fn sync(&mut self) -> io::Result<()> {
        self.flush()?;
        self.sync_all()
    }
}

/// Open the staged file read/write for frame application.
pub(crate) fn open_file_sink(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).write(true).open(path)
}

/// Marker embedded in every staging file name (used for cleanup checks).
pub(crate) const STAGING_MARKER: &str = ".engram-replay-";

/// SQLite side files whose presence next to a database changes what SQLite
/// reads on open (a hot journal or WAL would be applied over a replaced file).
const HOT_SIDE_SUFFIXES: [&str; 2] = ["-wal", "-journal"];

/// All SQLite side files that may appear next to a staged database.
const ALL_SIDE_SUFFIXES: [&str; 3] = ["-wal", "-journal", "-shm"];

static STAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Caller-owned staging file in the target's directory.
///
/// Replay happens on the staging copy; the target is replaced only by
/// [`StagedTarget::commit`] (fsync + rename + parent-dir fsync on Unix).
/// Dropping an uncommitted stage removes the staging file and its side files,
/// so any failure before the rename leaves the target untouched.
///
/// Limits: the rename is atomic on POSIX filesystems when staging and target
/// share a filesystem (guaranteed by staging in the same directory). On
/// Windows the replacement uses `MoveFileExW(REPLACE_EXISTING)`, which is not
/// documented as atomic. A crash before the rename leaves the old target plus
/// an orphan `.<name>.engram-replay-*.tmp` file that is safe to delete. The
/// target must not be open by another process during recovery.
///
/// G1 (INVARIANTS #27): `File` and `ExistingTarget` seeds copy through a
/// plain `File` that is closed afterwards. If that file is a database this
/// process has open through rusqlite, the close drops SQLite's POSIX locks
/// and another process can later delete its live WAL. Callers holding a
/// `Storage` must seed its database with `ActiveStorage` (MCP
/// `replication_recover` does) and refuse it as a target.
pub(crate) struct StagedTarget {
    target: PathBuf,
    staging: PathBuf,
    committed: bool,
}

impl StagedTarget {
    pub(crate) fn begin(target: &Path, seed: StageSeed<'_>) -> Result<Self, WalReplicationError> {
        refuse_hot_side_files(target)?;
        if target.is_dir() {
            return Err(recovery_error(target, "target is a directory"));
        }
        let name = target
            .file_name()
            .ok_or_else(|| recovery_error(target, "target has no file name"))?;
        let parent = match target.parent() {
            Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
            _ => PathBuf::from("."),
        };
        fs::create_dir_all(&parent)?;

        let staging = parent.join(format!(
            ".{}{}{}-{}-{}.tmp",
            name.to_string_lossy(),
            STAGING_MARKER,
            std::process::id(),
            unique_nanos(),
            STAGE_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = staging_open_options().open(&staging)?;
        // From here on, Drop cleans up the staging file on any failure.
        let stage = Self {
            target: target.to_path_buf(),
            staging,
            committed: false,
        };

        let source = match seed {
            StageSeed::File(src) => Some(src),
            StageSeed::ExistingTarget if target.exists() => Some(target),
            StageSeed::ExistingTarget => None,
            StageSeed::ActiveStorage(storage) => {
                // The staging file is empty, which VACUUM INTO accepts.
                drop(file);
                storage
                    .vacuum_into(&stage.staging)
                    .map_err(|err| recovery_error(target, &format!("snapshot failed: {err}")))?;
                File::open(&stage.staging)?.sync_all()?;
                return Ok(stage);
            }
        };
        if let Some(src) = source {
            let mut reader = File::open(src)?;
            io::copy(&mut reader, &mut file)?;
        }
        file.sync_all()?;
        Ok(stage)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.staging
    }

    /// Replace the target with the staged file.
    pub(crate) fn commit(mut self) -> Result<(), WalReplicationError> {
        remove_side_files(&self.staging);
        refuse_hot_side_files(&self.target)?;
        File::open(&self.staging)?.sync_all()?;
        fs::rename(&self.staging, &self.target)?;
        self.committed = true;
        sync_parent_dir(&self.target);
        Ok(())
    }
}

impl Drop for StagedTarget {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        remove_quietly(&self.staging);
        remove_side_files(&self.staging);
    }
}

/// Owner-only mode for staged databases, matching `Storage`'s SQLite files.
#[cfg(unix)]
const STAGING_FILE_MODE: u32 = 0o600;

/// `create_new` options for the staging file; owner-only on Unix so the
/// renamed target never widens permissions (it may hold memory content).
fn staging_open_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(STAGING_FILE_MODE);
    }
    options
}

fn side_file(db: &Path, suffix: &str) -> PathBuf {
    let mut os = db.as_os_str().to_os_string();
    os.push(suffix);
    PathBuf::from(os)
}

/// Refuse to replace a database that has a non-empty WAL or rollback journal:
/// SQLite would apply it over the replacement, or another process holds it.
fn refuse_hot_side_files(target: &Path) -> Result<(), WalReplicationError> {
    for suffix in HOT_SIDE_SUFFIXES {
        let side = side_file(target, suffix);
        match fs::metadata(&side) {
            Ok(meta) if meta.len() == 0 => {}
            Ok(_) => {
                return Err(recovery_error(
                    target,
                    &format!(
                        "{} exists and is non-empty; close/checkpoint the target first",
                        side.display()
                    ),
                ));
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            // Fail closed: if we cannot tell, do not replace the target.
            Err(err) => {
                return Err(recovery_error(
                    target,
                    &format!("cannot inspect {}: {}", side.display(), err),
                ));
            }
        }
    }
    Ok(())
}

fn remove_side_files(db: &Path) {
    for suffix in ALL_SIDE_SUFFIXES {
        remove_quietly(&side_file(db, suffix));
    }
}

fn remove_quietly(path: &Path) {
    if let Err(err) = fs::remove_file(path) {
        if err.kind() != io::ErrorKind::NotFound {
            tracing::warn!(
                path = %crate::observability::redact::path_label(path),
                error_class = %crate::observability::redact::io_class(&err),
                "failed to remove staging file"
            );
        }
    }
}

#[cfg(unix)]
fn sync_parent_dir(target: &Path) {
    let parent = match target.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    if let Err(err) = File::open(parent).and_then(|dir| dir.sync_all()) {
        // The rename already happened; report reduced crash durability.
        tracing::warn!(
            dir = %crate::observability::redact::path_label(parent),
            error_class = %crate::observability::redact::io_class(&err),
            "parent dir fsync failed after replace"
        );
    }
}

#[cfg(not(unix))]
fn sync_parent_dir(_target: &Path) {}

fn unique_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

fn recovery_error(target: &Path, reason: &str) -> WalReplicationError {
    WalReplicationError::RecoveryError(format!("{}: {}", target.display(), reason))
}

/// Stage, write frames, verify integrity, then atomically replace the target.
pub(crate) fn replay_staged<S, F>(
    target: &Path,
    seed: StageSeed<'_>,
    page_size: u32,
    frames: &[&WalFrame],
    options: &RecoveryOptions,
    start_time: Instant,
    open_sink: F,
) -> Result<RecoveryReport, WalReplicationError>
where
    S: PageSink,
    F: FnOnce(&Path) -> io::Result<S>,
{
    let staged = StagedTarget::begin(target, seed)?;
    let mut sink = open_sink(staged.path())?;
    let commits_applied = write_frames(&mut sink, page_size, frames)?;
    sink.sync()?;
    drop(sink);

    let final_db_size = fs::metadata(staged.path())?.len();
    let integrity_check = verify_integrity(staged.path(), options, final_db_size)?;
    staged.commit()?;

    Ok(RecoveryReport {
        success: true,
        target_db_path: target.to_string_lossy().to_string(),
        frames_replayed: frames.len(),
        commits_applied,
        last_frame_applied: frames.last().map(|f| f.frame_index),
        final_db_size_bytes: final_db_size,
        integrity_check,
        duration_ms: start_time.elapsed().as_millis() as u64,
        error: None,
    })
}

/// Apply validated frames; returns the number of commit frames applied.
fn write_frames<S: PageSink>(
    sink: &mut S,
    page_size: u32,
    frames: &[&WalFrame],
) -> Result<usize, WalReplicationError> {
    let mut commits_applied = 0;
    for frame in frames {
        // Defense in depth: preflight already refused these.
        let offset = page_offset(frame.page_number, page_size)
            .filter(|_| frame.page_number <= MAX_ALLOWED_DB_PAGES)
            .ok_or_else(|| WalReplicationError::InvalidFrame {
                index: frame.frame_index,
                reason: format!("invalid frame page_number: {}", frame.page_number),
            })?;
        sink.write_at(offset, &frame.data)?;
        if frame.is_commit() {
            commits_applied += 1;
            sink.set_len(frame.db_size_pages as u64 * page_size as u64)?;
        }
    }
    Ok(commits_applied)
}

/// Run `PRAGMA integrity_check` on the staged file when requested.
fn verify_integrity(
    path: &Path,
    options: &RecoveryOptions,
    size: u64,
) -> Result<String, WalReplicationError> {
    if !options.verify_integrity || size == 0 {
        return Ok("skipped".to_string());
    }
    let conn = Connection::open(path)?;
    let res: String = conn.query_row("PRAGMA integrity_check;", [], |row| row.get(0))?;
    if res != "ok" {
        return Err(WalReplicationError::IntegrityCheckFailed(res));
    }
    Ok("ok".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: u32 = 512;
    const SENTINEL: &[u8] = b"SENTINEL-TARGET-MUST-SURVIVE";

    /// Page writer that fails on the Nth write or on sync.
    struct FailingSink {
        inner: File,
        fail_on_write: usize,
        writes: usize,
        fail_sync: bool,
    }

    impl PageSink for FailingSink {
        fn write_at(&mut self, offset: u64, data: &[u8]) -> io::Result<()> {
            self.writes += 1;
            if self.writes == self.fail_on_write {
                return Err(io::Error::other("injected late write failure"));
            }
            self.inner.write_at(offset, data)
        }

        fn set_len(&mut self, len: u64) -> io::Result<()> {
            PageSink::set_len(&mut self.inner, len)
        }

        fn sync(&mut self) -> io::Result<()> {
            if self.fail_sync {
                return Err(io::Error::other("injected late sync failure"));
            }
            self.inner.sync()
        }
    }

    fn frames() -> Vec<WalFrame> {
        (1..=3)
            .map(|i| WalFrame {
                frame_index: i,
                page_number: i,
                db_size_pages: i,
                salt1: 0,
                salt2: 0,
                checksum1: 0,
                checksum2: 0,
                data: vec![i as u8; PAGE as usize],
            })
            .collect()
    }

    fn no_integrity() -> RecoveryOptions {
        RecoveryOptions {
            verify_integrity: false,
            ..Default::default()
        }
    }

    fn staging_leftovers(dir: &Path) -> Vec<String> {
        fs::read_dir(dir)
            .expect("read dir")
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .filter(|n| n.contains(".engram-replay-"))
            .collect()
    }

    fn run_with(
        target: &Path,
        fail_on_write: usize,
        fail_sync: bool,
    ) -> Result<RecoveryReport, WalReplicationError> {
        let frames = frames();
        let refs: Vec<&WalFrame> = frames.iter().collect();
        replay_staged(
            target,
            StageSeed::ExistingTarget,
            PAGE,
            &refs,
            &no_integrity(),
            Instant::now(),
            |path| {
                Ok(FailingSink {
                    inner: open_file_sink(path)?,
                    fail_on_write,
                    writes: 0,
                    fail_sync,
                })
            },
        )
    }

    #[test]
    fn test_late_write_failure_preserves_target() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("target.db");
        fs::write(&target, SENTINEL).expect("sentinel");

        let result = run_with(&target, 3, false);
        assert!(matches!(result, Err(WalReplicationError::Io(_))));
        assert_eq!(fs::read(&target).expect("read"), SENTINEL);
        assert!(staging_leftovers(dir.path()).is_empty());
    }

    #[test]
    fn test_late_sync_failure_preserves_target() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("target.db");
        fs::write(&target, SENTINEL).expect("sentinel");

        let result = run_with(&target, usize::MAX, true);
        assert!(matches!(result, Err(WalReplicationError::Io(_))));
        assert_eq!(fs::read(&target).expect("read"), SENTINEL);
        assert!(staging_leftovers(dir.path()).is_empty());
    }

    #[test]
    fn test_success_replaces_target_atomically() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("target.db");
        fs::write(&target, SENTINEL).expect("sentinel");

        let report = run_with(&target, usize::MAX, false).expect("replay");
        assert_eq!(report.commits_applied, 3);
        let bytes = fs::read(&target).expect("read");
        assert_eq!(bytes.len(), 3 * PAGE as usize);
        assert_eq!(bytes[PAGE as usize * 2], 3);
        assert!(staging_leftovers(dir.path()).is_empty());
    }

    #[test]
    fn test_side_file_check_fails_closed_on_unreadable_metadata() {
        // `<file>/target.db-wal`: the parent component is a regular file, so
        // stat fails with NotADirectory instead of NotFound.
        let dir = tempfile::tempdir().expect("tempdir");
        let not_a_dir = dir.path().join("plain-file");
        fs::write(&not_a_dir, b"x").expect("plain file");
        let target = not_a_dir.join("target.db");
        let err = refuse_hot_side_files(&target).expect_err("must fail closed");
        assert!(
            matches!(err, WalReplicationError::RecoveryError(_)),
            "unexpected error {err:?}"
        );
    }

    #[test]
    fn test_abandoned_stage_leaves_seed_and_target_untouched() {
        let dir = tempfile::tempdir().expect("tempdir");
        let seed = dir.path().join("seed.db");
        fs::write(&seed, b"seed-bytes").expect("seed");
        let target = dir.path().join("target.db");

        let staged = StagedTarget::begin(&target, StageSeed::File(&seed)).expect("stage");
        assert_ne!(staged.path(), target.as_path(), "must stage beside target");
        assert_eq!(fs::read(staged.path()).expect("staged"), b"seed-bytes");
        drop(staged);

        assert!(!target.exists());
        assert_eq!(fs::read(&seed).expect("seed"), b"seed-bytes");
        assert!(staging_leftovers(dir.path()).is_empty());
    }
}

#[cfg(test)]
mod redaction_tests {
    use super::*;
    use crate::observability::test_capture::{assert_logs_exclude, captured, install};

    #[test]
    fn staging_cleanup_and_fsync_warnings_do_not_log_paths() {
        install();
        let dir = tempfile::tempdir().expect("tempdir");

        // remove_file on a directory fails with something other than NotFound.
        let undeletable = dir.path().join("staging-path-sentinel-3c4d");
        fs::create_dir(&undeletable).expect("mkdir");
        remove_quietly(&undeletable);

        // The parent directory of the target does not exist: the fsync open fails.
        let orphan_target = dir.path().join("fsync-dir-sentinel-5e6f").join("target.db");
        sync_parent_dir(&orphan_target);

        let logs = captured();
        assert!(logs.contains("failed to remove staging file"), "{logs}");
        assert!(logs.contains("parent dir fsync failed after replace"));
        assert_logs_exclude(&["staging-path-sentinel-3c4d", "fsync-dir-sentinel-5e6f"]);
    }
}
