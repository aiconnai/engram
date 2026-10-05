//! Guards for the database files that this process has open through SQLite (G1).
//!
//! POSIX (fcntl) locks belong to the (process, inode) pair: closing ANY
//! descriptor of the active database or its `-shm` drops every lock this
//! process holds on it, behind the back of SQLite's unix VFS. Another process
//! that then opens and closes the database deletes the live WAL and later
//! commits are lost. Code that opens caller-controlled paths must therefore
//! refuse the active SQLite files, and code that needs the database bytes
//! must go through SQLite (`VACUUM INTO`), never a raw `File`.

use std::path::{Path, PathBuf};

use super::connection::Storage;
use crate::error::{EngramError, Result};

/// SQLite side files that live next to the database.
const SIDE_SUFFIXES: [&str; 3] = ["-wal", "-shm", "-journal"];

/// File name of the copy inside an [`ActiveDbSnapshot`] directory.
const SNAPSHOT_FILE_NAME: &str = "engram-snapshot.db";

impl Storage {
    /// Refuse `path` if it is the active database or one of its SQLite side
    /// files (`-wal`, `-shm`, `-journal`).
    ///
    /// Compares device + inode for existing files (so symlinks and hard links
    /// to the database are caught) and the resolved name for paths that do
    /// not exist yet (a side file SQLite may create later). Call it right
    /// before opening a caller-supplied path.
    ///
    /// Residual: this is a check, then the caller opens by path; a process
    /// that can rename entries in between can still redirect the open.
    pub fn refuse_active_sqlite_artifact(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        refuse_descriptor_alias(path)?;
        if self.is_active_sqlite_artifact(path) {
            return Err(refuse_active(path));
        }
        Ok(())
    }

    /// Like [`Self::refuse_active_sqlite_artifact`], but allows the active
    /// `-wal` (and `-journal`): SQLite takes no POSIX locks on those, so
    /// reading them (WAL replication status / delta extraction) cannot drop
    /// its locks. Refuses the database file and `-shm`, which carry locks.
    pub fn refuse_lock_bearing_sqlite_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        refuse_descriptor_alias(path)?;
        if self.db_path() != ":memory:"
            && matches_any(path, &artifact_paths(Path::new(self.db_path()), &["-shm"]))
        {
            return Err(refuse_active(path));
        }
        Ok(())
    }

    /// True when `path` is the active database or one of its side files.
    pub fn is_active_sqlite_artifact(&self, path: impl AsRef<Path>) -> bool {
        if self.db_path() == ":memory:" {
            return false;
        }
        matches_any(
            path.as_ref(),
            &artifact_paths(Path::new(self.db_path()), &SIDE_SUFFIXES),
        )
    }

    /// Write a consistent copy of the database (including committed WAL
    /// content) to `destination` through SQLite's `VACUUM INTO`.
    ///
    /// `destination` must not exist or must be an empty file. SQLite opens
    /// and closes only `destination`; the active files are never opened
    /// outside SQLite.
    pub fn vacuum_into(&self, destination: impl AsRef<Path>) -> Result<()> {
        let destination = destination.as_ref();
        self.refuse_active_sqlite_artifact(destination)?;
        // The connection was opened with SQLITE_OPEN_NOFOLLOW, which SQLite
        // also applies to VACUUM INTO and which rejects a symlink in ANY
        // path component (macOS: /var -> /private/var), so resolve the parent.
        let destination = resolved_name(destination).ok_or_else(|| {
            EngramError::InvalidInput(format!(
                "snapshot path '{}' has no resolvable parent directory",
                destination.display()
            ))
        })?;
        let target = destination.to_str().ok_or_else(|| {
            EngramError::InvalidInput(format!(
                "snapshot path '{}' is not valid UTF-8",
                destination.display()
            ))
        })?;
        self.with_connection(|conn| {
            conn.execute("VACUUM INTO ?1", [target])?;
            Ok(())
        })
    }

    /// Consistent, owner-only copy of the database in a private temporary
    /// directory, removed on drop. For read-only consumers that must open
    /// the database themselves (for example DuckDB's SQLite scanner, which
    /// links its own SQLite copy and would otherwise drop this process's
    /// locks on the active file).
    pub fn snapshot_copy(&self) -> Result<ActiveDbSnapshot> {
        if self.db_path() == ":memory:" {
            return Err(EngramError::InvalidInput(
                "in-memory databases have no file to snapshot".to_string(),
            ));
        }
        let dir = create_private_temp_dir()?;
        let snapshot = ActiveDbSnapshot {
            path: dir.join(SNAPSHOT_FILE_NAME),
            dir,
        };
        self.vacuum_into(&snapshot.path)?;
        Ok(snapshot)
    }
}

/// A private on-disk copy of the database; the directory is removed on drop.
#[derive(Debug)]
pub struct ActiveDbSnapshot {
    dir: PathBuf,
    path: PathBuf,
}

impl ActiveDbSnapshot {
    /// Path of the snapshot database file.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ActiveDbSnapshot {
    fn drop(&mut self) {
        if let Err(err) = std::fs::remove_dir_all(&self.dir) {
            if err.kind() != std::io::ErrorKind::NotFound {
                tracing::warn!(
                    dir = %crate::observability::redact::path_label(&self.dir),
                    error_class = %crate::observability::redact::io_class(&err),
                    "failed to remove database snapshot directory"
                );
            }
        }
    }
}

fn refuse_active(path: &Path) -> EngramError {
    EngramError::InvalidInput(format!(
        "refusing to use '{}': it is the active Engram database or one of its \
         SQLite side files; use a closed copy or a snapshot instead",
        path.display()
    ))
}

/// Paths under `/dev` and `/proc` can name an already-open descriptor of the
/// database (`/dev/fd/N`, `/proc/self/fd/N`). On macOS, `stat` of
/// `/dev/fd/N` reports the devfs device, so a device + inode comparison
/// misses it, and opening and closing it drops this process's POSIX locks.
/// Such paths are never legitimate inputs here, so they are refused outright.
fn refuse_descriptor_alias(path: &Path) -> Result<()> {
    if is_descriptor_alias_path(path) {
        return Err(EngramError::InvalidInput(format!(
            "refusing to use '{}': paths under /dev and /proc can alias open file \
             descriptors of the active Engram database",
            path.display()
        )));
    }
    Ok(())
}

fn is_descriptor_alias_path(path: &Path) -> bool {
    let special = |p: &Path| p.starts_with("/dev") || p.starts_with("/proc");
    special(path)
        || std::path::absolute(path).is_ok_and(|p| special(&p))
        || resolved_name(path).is_some_and(|p| special(&p))
        || std::fs::canonicalize(path).is_ok_and(|p| special(&p))
}

fn artifact_paths(db_path: &Path, suffixes: &[&str]) -> Vec<PathBuf> {
    let mut paths = vec![db_path.to_path_buf()];
    for suffix in suffixes {
        let mut path = db_path.as_os_str().to_os_string();
        path.push(suffix);
        paths.push(PathBuf::from(path));
    }
    paths
}

fn matches_any(path: &Path, artifacts: &[PathBuf]) -> bool {
    let candidate_name = resolved_name(path);
    artifacts.iter().any(|artifact| {
        same_inode(path, artifact)
            || (candidate_name.is_some() && candidate_name == resolved_name(artifact))
    })
}

#[cfg(unix)]
fn same_inode(a: &Path, b: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (std::fs::metadata(a), std::fs::metadata(b)) {
        (Ok(a), Ok(b)) => {
            a.ino() == b.ino() && (a.dev() == b.dev() || is_descriptor_fs_device(a.dev()))
        }
        _ => false,
    }
}

/// Devices of the descriptor filesystems (macOS devfs/fdesc, Linux procfs),
/// whose `stat` keeps the target's inode but not its device.
#[cfg(unix)]
fn is_descriptor_fs_device(dev: u64) -> bool {
    use std::os::unix::fs::MetadataExt;
    ["/dev", "/dev/fd", "/proc/self/fd"]
        .iter()
        .filter_map(|p| std::fs::metadata(p).ok())
        .any(|m| m.dev() == dev)
}

#[cfg(not(unix))]
fn same_inode(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Canonical parent directory joined with the file name, or `None` when the
/// parent cannot be resolved.
fn resolved_name(path: &Path) -> Option<PathBuf> {
    let name = path.file_name()?;
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    Some(std::fs::canonicalize(parent).ok()?.join(name))
}

fn create_private_temp_dir() -> Result<PathBuf> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "engram-db-snapshot-{}-{}-{}",
        std::process::id(),
        nanos,
        COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let mut builder = std::fs::DirBuilder::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder.create(&dir)?;
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{StorageConfig, StorageMode};

    fn open(path: &Path) -> Storage {
        Storage::open(StorageConfig {
            db_path: path.to_string_lossy().into_owned(),
            storage_mode: StorageMode::Local,
            cloud_uri: None,
            encrypt_cloud: false,
            confidence_half_life_days: 30.0,
            auto_sync: false,
            sync_debounce_ms: 5000,
        })
        .expect("open storage")
    }

    #[test]
    fn active_files_and_aliases_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("memory.db");
        let storage = open(&db);
        storage
            .with_connection(
                |c| Ok(c.execute_batch("CREATE TABLE t(x); INSERT INTO t VALUES(1);")?),
            )
            .unwrap();

        assert!(storage.refuse_active_sqlite_artifact(&db).is_err());
        assert!(storage
            .refuse_active_sqlite_artifact(dir.path().join("memory.db-wal"))
            .is_err());
        // Not created yet, but SQLite would use it.
        assert!(storage
            .refuse_active_sqlite_artifact(dir.path().join("memory.db-journal"))
            .is_err());
        assert!(storage
            .refuse_active_sqlite_artifact(dir.path().join(".").join("memory.db"))
            .is_err());

        #[cfg(unix)]
        {
            let link = dir.path().join("photo.png");
            std::os::unix::fs::symlink(&db, &link).unwrap();
            let err = storage.refuse_active_sqlite_artifact(&link).unwrap_err();
            assert!(err.to_string().contains("active Engram database"), "{err}");
            let hard = dir.path().join("hard.db");
            std::fs::hard_link(&db, &hard).unwrap();
            assert!(storage.refuse_active_sqlite_artifact(&hard).is_err());
        }

        let other = dir.path().join("other.db");
        std::fs::write(&other, b"x").unwrap();
        storage.refuse_active_sqlite_artifact(&other).unwrap();
        storage
            .refuse_active_sqlite_artifact(dir.path().join("missing.md"))
            .unwrap();
    }

    /// macOS `/dev/fd/N` stat reports the devfs device; Linux `/proc/self/fd/N`
    /// is a magic link. Both name an open descriptor of the database.
    #[cfg(unix)]
    #[test]
    fn descriptor_aliases_of_the_active_database_are_refused() {
        use std::os::unix::io::AsRawFd;

        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("memory.db");
        let storage = open(&db);
        // Only to obtain a descriptor number; this test does not rely on locks.
        let file = std::fs::File::open(&db).unwrap();
        let fd = file.as_raw_fd();

        let aliases: Vec<PathBuf> = [
            Some(format!("/dev/fd/{fd}")),
            cfg!(target_os = "linux").then(|| format!("/proc/self/fd/{fd}")),
        ]
        .into_iter()
        .flatten()
        .map(PathBuf::from)
        .collect();
        for alias in &aliases {
            let err = storage.refuse_active_sqlite_artifact(alias).unwrap_err();
            assert!(
                err.to_string().contains("/dev and /proc"),
                "{alias:?}: {err}"
            );
            assert!(storage.refuse_lock_bearing_sqlite_file(alias).is_err());
        }
        // A symlink to the descriptor path resolves to it and is refused too.
        let link = dir.path().join("notes.md");
        std::os::unix::fs::symlink(&aliases[0], &link).unwrap();
        assert!(storage.refuse_active_sqlite_artifact(&link).is_err());
        drop(file);
    }

    #[test]
    fn lock_bearing_guard_allows_the_wal_only() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("memory.db");
        let storage = open(&db);
        storage
            .with_connection(
                |c| Ok(c.execute_batch("CREATE TABLE t(x); INSERT INTO t VALUES(1);")?),
            )
            .unwrap();
        storage
            .refuse_lock_bearing_sqlite_file(dir.path().join("memory.db-wal"))
            .unwrap();
        assert!(storage.refuse_lock_bearing_sqlite_file(&db).is_err());
        assert!(storage
            .refuse_lock_bearing_sqlite_file(dir.path().join("memory.db-shm"))
            .is_err());
    }

    #[test]
    fn in_memory_storage_refuses_nothing() {
        let storage = Storage::open_in_memory().unwrap();
        storage.refuse_active_sqlite_artifact(":memory:").unwrap();
        assert!(storage.snapshot_copy().is_err());
    }

    #[test]
    fn snapshot_copy_holds_committed_rows_and_is_removed_on_drop() {
        let dir = tempfile::tempdir().unwrap();
        let storage = open(&dir.path().join("memory.db"));
        storage
            .with_connection(
                |c| Ok(c.execute_batch("CREATE TABLE t(x); INSERT INTO t VALUES(7);")?),
            )
            .unwrap();

        let snapshot = storage.snapshot_copy().unwrap();
        let conn = rusqlite::Connection::open(snapshot.path()).unwrap();
        let x: i64 = conn.query_row("SELECT x FROM t", [], |r| r.get(0)).unwrap();
        assert_eq!(x, 7);
        drop(conn);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let parent = snapshot.path().parent().unwrap().to_path_buf();
            let mode = std::fs::metadata(&parent).unwrap().permissions().mode() & 0o777;
            assert_eq!(mode, 0o700);
        }
        let parent = snapshot.path().parent().unwrap().to_path_buf();
        drop(snapshot);
        assert!(!parent.exists());
    }

    #[test]
    fn vacuum_into_refuses_the_active_database() {
        let dir = tempfile::tempdir().unwrap();
        let db = dir.path().join("memory.db");
        let storage = open(&db);
        assert!(storage.vacuum_into(&db).is_err());
    }
}
