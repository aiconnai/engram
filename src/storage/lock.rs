//! Process-level singleton lock per storage path (issue #24).
//!
//! Prevents multiple mutating workers/indexers from operating on the same
//! database at once (the "122 processes pointing at one directory" class of
//! problem). Uses an advisory `flock(2)` on a sidecar lock file: the kernel
//! releases the lock automatically when the holder exits, so a crashed holder
//! never leaves a stale lock behind. Holder metadata (pid, command, created_at)
//! is written to the file for diagnostics.
//!
//! `:memory:` databases and non-Unix platforms are never locked.

use std::fs::{File, OpenOptions};
use std::io::Read;
use std::path::Path;

use crate::error::{EngramError, Result};

/// Diagnostic information about the current lock holder.
#[derive(Debug, Clone)]
pub struct LockInfo {
    pub pid: i32,
    pub command: String,
    pub created_at: String,
}

/// An acquired storage lock. The lock is released when this value is dropped
/// (the file descriptor closes) or when the process exits (kernel releases the
/// flock).
pub struct StorageLock {
    /// Held open to keep the advisory lock for this value's lifetime.
    #[allow(dead_code)]
    file: Option<File>,
}

impl StorageLock {
    /// Acquire the singleton lock for `db_path`, recording `command` as the
    /// owner. `:memory:` and non-Unix platforms return an unlocked guard that
    /// always succeeds. Returns an error if another live process holds the lock.
    pub fn acquire(db_path: &str, command: &str) -> Result<Self> {
        if db_path == ":memory:" {
            return Ok(Self { file: None });
        }
        Self::acquire_path(&format!("{db_path}.lock"), command)
    }

    /// Read the current holder's metadata, if a lock file exists.
    pub fn read_info(db_path: &str) -> Option<LockInfo> {
        if db_path == ":memory:" {
            return None;
        }
        Self::read_info_file(&format!("{db_path}.lock"))
    }

    #[cfg(unix)]
    fn acquire_path(lock_path: &str, command: &str) -> Result<Self> {
        use std::io::Write;
        use std::os::unix::io::AsRawFd;

        if let Some(parent) = Path::new(lock_path).parent() {
            if !parent.as_os_str().is_empty() {
                let _ = std::fs::create_dir_all(parent);
            }
        }

        // Open without truncating: we must not clobber a holder's metadata
        // before we know whether we actually win the lock.
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(lock_path)
            .map_err(|e| {
                EngramError::Storage(format!("cannot open lock file '{lock_path}': {e}"))
            })?;

        // Non-blocking exclusive advisory lock. Fails immediately if held.
        // SAFETY: `file` owns a valid fd; flock with these flags is safe.
        let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
        if rc != 0 {
            let holder = Self::read_info_file(lock_path)
                .map(|i| format!("pid {} ({}), since {}", i.pid, i.command, i.created_at))
                .unwrap_or_else(|| "another process".to_string());
            return Err(EngramError::Storage(format!(
                "storage path is already locked by {holder}; only one mutating worker may \
                 own '{lock_path}'. Stop the other process or use a different database path."
            )));
        }

        // We own the lock — (re)write fresh holder metadata.
        let pid = std::process::id() as i32;
        let created_at = chrono::Utc::now().to_rfc3339();
        let _ = file.set_len(0);
        let _ = write!(
            file,
            "pid={pid}\ncommand={command}\ncreated_at={created_at}\n"
        );
        let _ = file.flush();

        Ok(Self { file: Some(file) })
    }

    #[cfg(not(unix))]
    fn acquire_path(_lock_path: &str, _command: &str) -> Result<Self> {
        // No advisory file locking available; treat as unlocked.
        Ok(Self { file: None })
    }

    fn read_info_file(lock_path: &str) -> Option<LockInfo> {
        let mut contents = String::new();
        File::open(lock_path)
            .ok()?
            .read_to_string(&mut contents)
            .ok()?;
        let field = |key: &str| {
            contents
                .lines()
                .find_map(|line| line.strip_prefix(key).map(str::to_string))
        };
        Some(LockInfo {
            pid: field("pid=").and_then(|v| v.parse().ok()).unwrap_or(0),
            command: field("command=").unwrap_or_default(),
            created_at: field("created_at=").unwrap_or_default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_lock_in_memory_is_noop() {
        // :memory: is never locked; repeated acquisition always succeeds.
        let _a = StorageLock::acquire(":memory:", "a").unwrap();
        let _b = StorageLock::acquire(":memory:", "b").unwrap();
        assert!(StorageLock::read_info(":memory:").is_none());
    }

    #[cfg(unix)]
    #[test]
    fn test_storage_lock_contention_and_release() {
        let db = std::env::temp_dir()
            .join("engram-lock-test-contention.db")
            .to_string_lossy()
            .into_owned();
        let lock_file = format!("{db}.lock");
        let _ = std::fs::remove_file(&lock_file);

        // First holder wins and records its metadata.
        let lock1 = StorageLock::acquire(&db, "worker-1").expect("first acquire succeeds");
        let info = StorageLock::read_info(&db).expect("holder metadata is recorded");
        assert_eq!(info.pid, std::process::id() as i32);
        assert_eq!(info.command, "worker-1");

        // A second acquisition on the same path is rejected (contention).
        assert!(
            StorageLock::acquire(&db, "worker-2").is_err(),
            "second acquire must be blocked while the first is held"
        );

        // Releasing the first lets a new acquisition succeed (no stale lock).
        drop(lock1);
        let lock3 = StorageLock::acquire(&db, "worker-3").expect("re-acquire after release");
        assert_eq!(StorageLock::read_info(&db).unwrap().command, "worker-3");
        drop(lock3);
        let _ = std::fs::remove_file(&lock_file);
    }
}
