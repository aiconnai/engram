//! Database connection management with WAL mode support (RML-874)
//!
//! Implements SQLite connection pooling with configurable storage modes
//! for both local (WAL) and cloud-safe (DELETE journal) operation.

use parking_lot::Mutex;
use rusqlite::{Connection, OpenFlags};
use std::path::Path;
#[cfg(unix)]
use std::path::PathBuf;
use std::sync::Arc;

use super::migrations::run_migrations;
#[cfg(unix)]
use crate::error::EngramError;
use crate::error::Result;
use crate::types::{CompactOp, CompactReport, StorageConfig, StorageMode};

#[cfg(unix)]
const SQLITE_FILE_MODE: u32 = 0o600;
#[cfg(unix)]
const ENGRAM_OWNED_DIR_MODE: u32 = 0o700;

/// Storage engine wrapping SQLite with connection pooling
pub struct Storage {
    config: StorageConfig,
    conn: Arc<Mutex<Connection>>,
}

/// Connection pool for concurrent access
pub struct StoragePool {
    config: StorageConfig,
    pool: Vec<Arc<Mutex<Connection>>>,
    next: std::sync::atomic::AtomicUsize,
}

impl Storage {
    /// Open or create a database with the given configuration
    pub fn open(config: StorageConfig) -> Result<Self> {
        let conn = Self::create_connection(&config)?;

        // Run migrations
        run_migrations(&conn)?;
        Self::reassert_sqlite_artifact_permissions_for_config(&config)?;

        Ok(Self {
            config,
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Open with default configuration (in-memory for testing)
    pub fn open_in_memory() -> Result<Self> {
        let config = StorageConfig {
            db_path: ":memory:".to_string(),
            storage_mode: StorageMode::Local,
            cloud_uri: None,
            encrypt_cloud: false,
            confidence_half_life_days: 30.0,
            auto_sync: false,
            sync_debounce_ms: 5000,
        };
        Self::open(config)
    }

    /// Create a new connection with appropriate pragmas
    fn create_connection(config: &StorageConfig) -> Result<Connection> {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        #[cfg(unix)]
        let flags = flags | OpenFlags::SQLITE_OPEN_NOFOLLOW;

        let conn = if config.db_path == ":memory:" {
            Connection::open_in_memory()?
        } else {
            ensure_filesystem_database_supported()?;
            let db_path = Path::new(&config.db_path);
            prepare_database_path(db_path)?;
            #[cfg(unix)]
            prepare_database_file(db_path)?;
            #[cfg(unix)]
            let db_path = database_open_path(db_path)?;
            Connection::open_with_flags(db_path, flags)?
        };

        // Configure based on storage mode (RML-874, RML-900)
        Self::configure_pragmas(&conn, config.storage_mode)?;
        Self::reassert_sqlite_artifact_permissions_for_config(config)?;

        Ok(conn)
    }

    /// Configure SQLite pragmas based on storage mode
    ///
    /// Local mode (RML-874): WAL for performance and crash recovery
    /// Cloud-safe mode (RML-900): DELETE journal for cloud sync compatibility
    fn configure_pragmas(conn: &Connection, mode: StorageMode) -> Result<()> {
        match mode {
            StorageMode::Local => {
                // WAL mode for better concurrency and crash recovery
                conn.execute_batch(
                    r#"
                    PRAGMA journal_mode=WAL;
                    PRAGMA synchronous=NORMAL;
                    PRAGMA wal_autocheckpoint=1000;
                    PRAGMA busy_timeout=30000;
                    PRAGMA cache_size=-64000;
                    PRAGMA temp_store=MEMORY;
                    PRAGMA mmap_size=268435456;
                    PRAGMA foreign_keys=ON;
                    "#,
                )?;
            }
            StorageMode::CloudSafe => {
                // Single-file mode for cloud sync (Dropbox, OneDrive, iCloud)
                conn.execute_batch(
                    r#"
                    PRAGMA journal_mode=DELETE;
                    PRAGMA synchronous=FULL;
                    PRAGMA busy_timeout=30000;
                    PRAGMA cache_size=-32000;
                    PRAGMA temp_store=MEMORY;
                    PRAGMA foreign_keys=ON;
                    "#,
                )?;
            }
        }
        Ok(())
    }

    /// Get a reference to the connection (for single-threaded use)
    pub fn connection(&self) -> parking_lot::MutexGuard<'_, Connection> {
        self.conn.lock()
    }

    /// Execute a function with the connection
    pub fn with_connection<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let result = {
            let conn = self.conn.lock();
            f(&conn)
        };
        self.reassert_sqlite_artifact_permissions_after(result)
    }

    /// Execute a function with a transaction
    pub fn with_transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let result = {
            let mut conn = self.conn.lock();
            let tx = conn.transaction()?;
            match f(&tx) {
                Ok(result) => tx.commit().map(|_| result).map_err(Into::into),
                Err(err) => Err(err),
            }
        };
        self.reassert_sqlite_artifact_permissions_after(result)
    }

    /// Get current storage mode
    pub fn storage_mode(&self) -> StorageMode {
        self.config.storage_mode
    }

    /// Get database path
    pub fn db_path(&self) -> &str {
        &self.config.db_path
    }

    /// Check if database is in a cloud-synced folder
    pub fn is_in_cloud_folder(&self) -> bool {
        let path = self.config.db_path.to_lowercase();
        path.contains("dropbox")
            || path.contains("onedrive")
            || path.contains("icloud")
            || path.contains("google drive")
    }

    /// Get warning if storage mode doesn't match folder type
    pub fn storage_mode_warning(&self) -> Option<String> {
        if self.is_in_cloud_folder() && self.config.storage_mode == StorageMode::Local {
            Some(format!(
                "WARNING: Database '{}' appears to be in a cloud-synced folder. \
                WAL mode may cause corruption. Consider:\n\
                1. Set ENGRAM_STORAGE_MODE=cloud-safe\n\
                2. Move database to a local folder with backup sync",
                self.config.db_path
            ))
        } else {
            None
        }
    }

    /// Checkpoint WAL file (for local mode)
    pub fn checkpoint(&self) -> Result<()> {
        if self.config.storage_mode == StorageMode::Local {
            let conn = self.conn.lock();
            conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        }
        self.reassert_sqlite_artifact_permissions()?;
        Ok(())
    }

    /// Get database size in bytes
    pub fn db_size(&self) -> Result<i64> {
        let conn = self.conn.lock();
        let size: i64 = conn.query_row(
            "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
            [],
            |row| row.get(0),
        )?;
        Ok(size)
    }

    /// Vacuum the database to reclaim space
    pub fn vacuum(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute_batch("VACUUM;")?;
        drop(conn);
        self.reassert_sqlite_artifact_permissions()?;
        Ok(())
    }

    /// Build a storage compaction report (issue #22).
    ///
    /// With `apply = false` (dry-run) nothing is mutated. With `apply = true`,
    /// completed/failed embedding-queue rows are pruned, the WAL is
    /// checkpointed, and the database is VACUUMed — but VACUUM runs only when
    /// there is enough free disk space for the rewrite.
    pub fn compact(&self, apply: bool) -> Result<CompactReport> {
        #[cfg(unix)]
        #[allow(clippy::unnecessary_cast)]
        fn available_disk_bytes(path: &str) -> Option<i64> {
            use std::ffi::CString;
            use std::os::unix::ffi::OsStrExt;
            // Stat the parent dir so this works even before the file exists.
            let p = Path::new(path);
            let target = match p.parent() {
                Some(parent) if !parent.as_os_str().is_empty() => parent,
                _ => p,
            };
            let cpath = CString::new(target.as_os_str().as_bytes()).ok()?;
            // SAFETY: `cpath` is a valid NUL-terminated path; `stat` is zeroed
            // before being filled by statvfs(3).
            unsafe {
                let mut stat: libc::statvfs = std::mem::zeroed();
                if libc::statvfs(cpath.as_ptr(), &mut stat) == 0 {
                    let avail = (stat.f_bavail as u64).saturating_mul(stat.f_frsize as u64);
                    Some(avail.min(i64::MAX as u64) as i64)
                } else {
                    None
                }
            }
        }
        #[cfg(not(unix))]
        fn available_disk_bytes(_path: &str) -> Option<i64> {
            None
        }

        let conn = self.conn.lock();

        let page_size: i64 = conn
            .query_row("PRAGMA page_size", [], |r| r.get(0))
            .unwrap_or(0);
        let page_count: i64 = conn
            .query_row("PRAGMA page_count", [], |r| r.get(0))
            .unwrap_or(0);
        let freelist_count: i64 = conn
            .query_row("PRAGMA freelist_count", [], |r| r.get(0))
            .unwrap_or(0);
        let db_size_bytes = page_size * page_count;
        let reclaimable_bytes = page_size * freelist_count;

        let queue_complete_prunable: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM embedding_queue WHERE status = 'complete'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let queue_failed_prunable: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM embedding_queue WHERE status = 'failed'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let orphan_embeddings: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM embeddings WHERE memory_id NOT IN (SELECT id FROM memories)",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let sidecar = |suffix: &str| -> i64 {
            if self.config.db_path == ":memory:" {
                return 0;
            }
            std::fs::metadata(format!("{}{}", self.config.db_path, suffix))
                .map(|m| m.len() as i64)
                .unwrap_or(0)
        };
        let wal_bytes = sidecar("-wal");
        let shm_bytes = sidecar("-shm");

        let free_space = available_disk_bytes(&self.config.db_path);
        let free_space_bytes = free_space.unwrap_or(-1);
        // VACUUM rewrites the DB into a temp file, needing ~db_size extra bytes.
        let vacuum_safe = matches!(free_space, Some(free) if free >= db_size_bytes);

        let mut operations = Vec::new();

        let mut prune_complete = CompactOp {
            name: "prune_complete_queue".to_string(),
            candidates: queue_complete_prunable,
            applied: false,
            skipped_reason: None,
        };
        if apply {
            // DELETE of zero rows is a harmless no-op; running it unconditionally
            // in apply mode lets the operation report as applied rather than dry-run.
            conn.execute("DELETE FROM embedding_queue WHERE status = 'complete'", [])?;
            prune_complete.applied = true;
        }
        operations.push(prune_complete);

        let mut prune_failed = CompactOp {
            name: "prune_failed_queue".to_string(),
            candidates: queue_failed_prunable,
            applied: false,
            skipped_reason: None,
        };
        if apply {
            conn.execute("DELETE FROM embedding_queue WHERE status = 'failed'", [])?;
            prune_failed.applied = true;
        }
        operations.push(prune_failed);

        let mut checkpoint = CompactOp {
            name: "checkpoint_wal".to_string(),
            candidates: wal_bytes,
            applied: false,
            skipped_reason: None,
        };
        if apply {
            if self.config.storage_mode == StorageMode::Local {
                conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
                checkpoint.applied = true;
            } else {
                checkpoint.skipped_reason = Some("not in local/WAL mode".to_string());
            }
        }
        operations.push(checkpoint);

        let mut vacuum = CompactOp {
            name: "vacuum".to_string(),
            candidates: reclaimable_bytes,
            applied: false,
            skipped_reason: None,
        };
        if apply {
            if vacuum_safe {
                conn.execute_batch("VACUUM;")?;
                vacuum.applied = true;
            } else {
                vacuum.skipped_reason = Some(match free_space {
                    Some(free) => {
                        format!(
                            "insufficient free space: {free} available, need >= {db_size_bytes}"
                        )
                    }
                    None => "free space could not be determined".to_string(),
                });
            }
        }
        operations.push(vacuum);

        if apply {
            drop(conn);
            self.reassert_sqlite_artifact_permissions()?;
        }

        Ok(CompactReport {
            applied: apply,
            db_size_bytes,
            wal_bytes,
            shm_bytes,
            freelist_count,
            reclaimable_bytes,
            queue_complete_prunable,
            queue_failed_prunable,
            orphan_embeddings,
            free_space_bytes,
            vacuum_safe,
            operations,
        })
    }

    /// Get configuration
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }

    fn reassert_sqlite_artifact_permissions_after<T>(&self, result: Result<T>) -> Result<T> {
        match result {
            Ok(value) => {
                self.reassert_sqlite_artifact_permissions()?;
                Ok(value)
            }
            Err(err) => {
                let _ = self.reassert_sqlite_artifact_permissions();
                Err(err)
            }
        }
    }

    fn reassert_sqlite_artifact_permissions(&self) -> Result<()> {
        Self::reassert_sqlite_artifact_permissions_for_config(&self.config)
    }

    fn reassert_sqlite_artifact_permissions_for_config(config: &StorageConfig) -> Result<()> {
        if config.db_path == ":memory:" {
            return Ok(());
        }
        restrict_sqlite_artifact_permissions(Path::new(&config.db_path))
    }
}

#[cfg(unix)]
fn ensure_filesystem_database_supported() -> Result<()> {
    Ok(())
}

#[cfg(not(unix))]
fn ensure_filesystem_database_supported() -> Result<()> {
    Err(crate::error::EngramError::Storage(
        "filesystem SQLite databases require atomic no-follow support; use ':memory:' on this platform"
            .to_string(),
    ))
}

impl StoragePool {
    /// Create a connection pool with the specified size
    pub fn new(config: StorageConfig, pool_size: usize) -> Result<Self> {
        let mut pool = Vec::with_capacity(pool_size);

        for _ in 0..pool_size {
            let conn = Storage::create_connection(&config)?;
            pool.push(Arc::new(Mutex::new(conn)));
        }

        // Run migrations on first connection
        if let Some(first) = pool.first() {
            let conn = first.lock();
            run_migrations(&conn)?;
        }
        Storage::reassert_sqlite_artifact_permissions_for_config(&config)?;

        Ok(Self {
            config,
            pool,
            next: std::sync::atomic::AtomicUsize::new(0),
        })
    }

    /// Get a connection from the pool (round-robin)
    pub fn get(&self) -> Arc<Mutex<Connection>> {
        let idx = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % self.pool.len();
        self.pool[idx].clone()
    }

    /// Execute a function with a connection from the pool
    pub fn with_connection<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let result = {
            let conn_arc = self.get();
            let conn = conn_arc.lock();
            f(&conn)
        };
        match result {
            Ok(value) => {
                Storage::reassert_sqlite_artifact_permissions_for_config(&self.config)?;
                Ok(value)
            }
            Err(err) => {
                let _ = Storage::reassert_sqlite_artifact_permissions_for_config(&self.config);
                Err(err)
            }
        }
    }

    /// Get configuration
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }
}

impl Clone for Storage {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            conn: self.conn.clone(),
        }
    }
}

/// Prepare the database path before SQLite opens it.
///
/// On Unix, missing Engram-owned parent directories are created as `0700`.
/// Existing parent directories are never recursively chmodded; if the direct
/// parent is permissive, Engram emits an actionable warning and leaves it
/// unchanged. On non-Unix platforms, Rust's standard library does not expose
/// POSIX mode bits, so this only creates missing directories and file-mode
/// hardening below is a no-op.
fn prepare_database_path(db_path: &Path) -> Result<()> {
    if let Some(parent) = db_path.parent().filter(|path| !path.as_os_str().is_empty()) {
        prepare_database_parent(parent)?;
    }
    Ok(())
}

#[cfg(unix)]
fn prepare_database_parent(parent: &Path) -> Result<()> {
    let parent_existed = parent.exists();
    if !parent_existed {
        create_dir_all_restrictive(parent)?;
    }
    if parent_existed {
        warn_if_parent_is_permissive(parent)?;
    }
    Ok(())
}

#[cfg(unix)]
fn prepare_database_file(db_path: &Path) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;

    let opened = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(SQLITE_FILE_MODE)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(db_path);

    let file = match opened {
        Ok(file) => file,
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => {
            open_sqlite_artifact_no_follow(db_path).map_err(|open_err| {
                if open_err.raw_os_error() == Some(libc::ELOOP) {
                    EngramError::Storage(format!(
                        "refusing to open symlink SQLite database '{}'",
                        db_path.display()
                    ))
                } else {
                    open_err.into()
                }
            })?
        }
        Err(err) => return Err(err.into()),
    };

    if !file.metadata()?.is_file() {
        return Err(EngramError::Storage(format!(
            "refusing to open non-regular SQLite database '{}'",
            db_path.display()
        )));
    }
    restrict_open_regular_file_permissions(&file)
}

#[cfg(unix)]
fn database_open_path(db_path: &Path) -> Result<PathBuf> {
    let parent = db_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = db_path.file_name().ok_or_else(|| {
        EngramError::Storage(format!(
            "SQLite database path '{}' has no file name",
            db_path.display()
        ))
    })?;
    Ok(parent.canonicalize()?.join(file_name))
}

#[cfg(not(unix))]
fn prepare_database_parent(parent: &Path) -> Result<()> {
    std::fs::create_dir_all(parent)?;
    Ok(())
}

#[cfg(unix)]
fn create_dir_all_restrictive(dir: &Path) -> Result<()> {
    use std::os::unix::fs::DirBuilderExt;

    if dir.exists() {
        return Ok(());
    }

    if let Some(parent) = dir.parent().filter(|path| !path.as_os_str().is_empty()) {
        create_dir_all_restrictive(parent)?;
    }

    match std::fs::DirBuilder::new()
        .mode(ENGRAM_OWNED_DIR_MODE)
        .create(dir)
    {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => Ok(()),
        Err(err) => Err(err.into()),
    }
}

#[cfg(unix)]
fn warn_if_parent_is_permissive(parent: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mode = std::fs::metadata(parent)?.permissions().mode() & 0o777;
    if mode & 0o077 != 0 {
        tracing::warn!(
            path = %parent.display(),
            mode = %format_args!("{mode:03o}"),
            "Database parent directory is accessible by group or others; Engram will not chmod \
             pre-existing directories recursively. Move the database under an Engram-owned \
             private directory or run `chmod 700` on the parent directory."
        );
    }
    Ok(())
}

#[cfg(unix)]
fn restrict_sqlite_artifact_permissions(db_path: &Path) -> Result<()> {
    for path in sqlite_artifact_paths(db_path) {
        match open_sqlite_artifact_no_follow(&path) {
            Ok(file) => restrict_open_regular_file_permissions(&file)?,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) if err.raw_os_error() == Some(libc::ELOOP) => {
                return Err(EngramError::Storage(format!(
                    "refusing to chmod symlink SQLite artifact '{}'",
                    path.display()
                )));
            }
            Err(err) => return Err(err.into()),
        }
    }
    Ok(())
}

#[cfg(unix)]
fn open_sqlite_artifact_no_follow(path: &Path) -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;

    std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
}

#[cfg(unix)]
fn restrict_open_regular_file_permissions(file: &std::fs::File) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = file.metadata()?;
    if !metadata.is_file() {
        return Ok(());
    }

    let current = metadata.permissions().mode() & 0o777;
    let restricted = current & SQLITE_FILE_MODE;
    if restricted != current {
        file.set_permissions(std::fs::Permissions::from_mode(restricted))?;
    }

    Ok(())
}

#[cfg(not(unix))]
fn restrict_sqlite_artifact_permissions(_db_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(unix)]
fn sqlite_artifact_paths(db_path: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::with_capacity(3);
    paths.push(db_path.to_path_buf());
    for suffix in ["-wal", "-shm"] {
        let mut path = db_path.as_os_str().to_os_string();
        path.push(suffix);
        paths.push(PathBuf::from(path));
    }
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    #[cfg(unix)]
    fn file_config(db_path: &Path) -> StorageConfig {
        StorageConfig {
            db_path: db_path.to_string_lossy().into_owned(),
            storage_mode: StorageMode::Local,
            cloud_uri: None,
            encrypt_cloud: false,
            confidence_half_life_days: 30.0,
            auto_sync: false,
            sync_debounce_ms: 5000,
        }
    }

    #[test]
    fn test_open_in_memory() {
        let storage = Storage::open_in_memory().unwrap();
        assert_eq!(storage.db_path(), ":memory:");
    }

    #[test]
    fn test_storage_modes() {
        // Test local mode
        let config = StorageConfig {
            db_path: ":memory:".to_string(),
            storage_mode: StorageMode::Local,
            cloud_uri: None,
            encrypt_cloud: false,
            confidence_half_life_days: 30.0,
            auto_sync: false,
            sync_debounce_ms: 5000,
        };
        let storage = Storage::open(config).unwrap();
        assert_eq!(storage.storage_mode(), StorageMode::Local);

        // Test cloud-safe mode
        let config = StorageConfig {
            db_path: ":memory:".to_string(),
            storage_mode: StorageMode::CloudSafe,
            cloud_uri: None,
            encrypt_cloud: false,
            confidence_half_life_days: 30.0,
            auto_sync: false,
            sync_debounce_ms: 5000,
        };
        let storage = Storage::open(config).unwrap();
        assert_eq!(storage.storage_mode(), StorageMode::CloudSafe);
    }

    #[test]
    fn test_cloud_folder_detection() {
        let config = StorageConfig {
            db_path: "/Users/test/Dropbox/memories.db".to_string(),
            storage_mode: StorageMode::Local,
            cloud_uri: None,
            encrypt_cloud: false,
            confidence_half_life_days: 30.0,
            auto_sync: false,
            sync_debounce_ms: 5000,
        };
        // Can't actually open this path in tests, but we can test detection
        let path = config.db_path.to_lowercase();
        assert!(path.contains("dropbox"));
    }

    #[cfg(not(unix))]
    #[test]
    fn non_unix_filesystem_database_fails_closed_without_no_follow_support() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("engram-owned").join("memory.db");
        let err = match Storage::open(StorageConfig {
            db_path: db_path.to_string_lossy().into_owned(),
            storage_mode: StorageMode::Local,
            cloud_uri: None,
            encrypt_cloud: false,
            confidence_half_life_days: 30.0,
            auto_sync: false,
            sync_debounce_ms: 5000,
        }) {
            Ok(_) => panic!("filesystem database unexpectedly opened"),
            Err(err) => err,
        };

        assert!(err.to_string().contains("atomic no-follow support"));
        assert!(!db_path.parent().unwrap().exists());
    }

    #[cfg(unix)]
    #[test]
    fn unix_new_database_parent_and_sqlite_artifacts_are_restrictive() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("engram-owned").join("memory.db");
        let storage = Storage::open(file_config(&db_path)).unwrap();

        let parent_mode = std::fs::metadata(db_path.parent().unwrap())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(parent_mode, 0o700);

        storage
            .with_connection(|conn| {
                conn.execute_batch(
                    "CREATE TABLE permission_probe (id INTEGER PRIMARY KEY, value TEXT);
                     INSERT INTO permission_probe (value) VALUES ('ok');",
                )?;
                Ok(())
            })
            .unwrap();

        assert_sqlite_artifact_modes(&db_path, 0o600);

        storage.checkpoint().unwrap();
        assert_sqlite_artifact_modes(&db_path, 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn unix_existing_stricter_file_mode_is_preserved() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("strict.db");
        std::fs::write(&db_path, b"not a sqlite db").unwrap();
        std::fs::set_permissions(&db_path, std::fs::Permissions::from_mode(0o400)).unwrap();

        restrict_sqlite_artifact_permissions(&db_path).unwrap();

        let mode = std::fs::metadata(&db_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o400);
    }

    #[cfg(unix)]
    #[test]
    fn unix_symlink_artifact_is_rejected_without_chmodding_target() {
        let temp = tempfile::tempdir().unwrap();
        let db_path = temp.path().join("memory.db");
        let target = temp.path().join("target");
        let wal_path = Path::new(&format!("{}-wal", db_path.display())).to_path_buf();

        std::fs::write(&db_path, b"not a sqlite db").unwrap();
        std::fs::write(&target, b"target").unwrap();
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o644)).unwrap();
        std::os::unix::fs::symlink(&target, &wal_path).unwrap();

        let err = restrict_sqlite_artifact_permissions(&db_path).unwrap_err();
        assert!(err.to_string().contains("refusing to chmod symlink"));

        let target_mode = std::fs::metadata(&target).unwrap().permissions().mode() & 0o777;
        assert_eq!(target_mode, 0o644);
    }

    #[cfg(unix)]
    #[test]
    fn unix_database_symlink_is_rejected_before_sqlite_mutates_target() {
        use sha2::{Digest, Sha256};

        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target.db");
        let db_path = temp.path().join("memory.db");
        let conn = Connection::open(&target).unwrap();
        conn.execute_batch(
            "PRAGMA journal_mode=DELETE;
             CREATE TABLE sentinel (value TEXT NOT NULL);
             INSERT INTO sentinel VALUES ('unchanged');",
        )
        .unwrap();
        drop(conn);
        let target_sha_before = Sha256::digest(std::fs::read(&target).unwrap());
        std::os::unix::fs::symlink(&target, &db_path).unwrap();

        let err = match Storage::open(file_config(&db_path)) {
            Ok(_) => panic!("database symlink unexpectedly opened"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("refusing to open symlink SQLite database"),
            "unexpected error: {err}"
        );
        assert_eq!(
            Sha256::digest(std::fs::read(&target).unwrap()),
            target_sha_before
        );
        let conn = Connection::open(&target).unwrap();
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        let sentinel: String = conn
            .query_row("SELECT value FROM sentinel", [], |row| row.get(0))
            .unwrap();
        assert_eq!(journal_mode, "delete");
        assert_eq!(sentinel, "unchanged");
    }

    #[cfg(unix)]
    #[test]
    fn unix_path_replacement_after_open_cannot_redirect_chmod() {
        let temp = tempfile::tempdir().unwrap();
        let artifact_path = temp.path().join("memory.db");
        let opened_inode_path = temp.path().join("opened-memory.db");
        let symlink_target = temp.path().join("unrelated-target");

        std::fs::write(&artifact_path, b"opened inode").unwrap();
        std::fs::set_permissions(&artifact_path, std::fs::Permissions::from_mode(0o666)).unwrap();
        std::fs::write(&symlink_target, b"unrelated target").unwrap();
        std::fs::set_permissions(&symlink_target, std::fs::Permissions::from_mode(0o644)).unwrap();

        let opened_artifact = open_sqlite_artifact_no_follow(&artifact_path).unwrap();
        std::fs::rename(&artifact_path, &opened_inode_path).unwrap();
        std::os::unix::fs::symlink(&symlink_target, &artifact_path).unwrap();

        restrict_open_regular_file_permissions(&opened_artifact).unwrap();

        let opened_inode_mode = std::fs::metadata(&opened_inode_path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(opened_inode_mode, 0o600);

        let target_mode = std::fs::metadata(&symlink_target)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(target_mode, 0o644);
    }

    #[cfg(unix)]
    #[test]
    fn unix_existing_permissive_parent_stays_unchanged_and_warns() {
        #[derive(Clone)]
        struct CapturedWriter(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

        impl std::io::Write for CapturedWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        let temp = tempfile::tempdir().unwrap();
        let shared_parent = temp.path().join("shared-parent");
        std::fs::create_dir(&shared_parent).unwrap();
        std::fs::set_permissions(&shared_parent, std::fs::Permissions::from_mode(0o777)).unwrap();

        let logs = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let writer_logs = logs.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::WARN)
            .with_writer(move || CapturedWriter(writer_logs.clone()))
            .without_time()
            .finish();

        let db_path = shared_parent.join("memory.db");
        tracing::subscriber::with_default(subscriber, || {
            Storage::open(file_config(&db_path)).unwrap();
        });

        let parent_mode = std::fs::metadata(&shared_parent)
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(parent_mode, 0o777);

        let captured = String::from_utf8(logs.lock().unwrap().clone()).unwrap();
        assert!(captured.contains("Database parent directory"));
        assert!(captured.contains("chmod 700"));
    }

    #[cfg(unix)]
    fn assert_sqlite_artifact_modes(db_path: &Path, expected_mode: u32) {
        for path in [
            db_path.to_path_buf(),
            Path::new(&format!("{}-wal", db_path.display())).to_path_buf(),
            Path::new(&format!("{}-shm", db_path.display())).to_path_buf(),
        ] {
            if path.exists() {
                let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
                assert_eq!(mode, expected_mode, "{} has wrong mode", path.display());
            }
        }
    }
}
