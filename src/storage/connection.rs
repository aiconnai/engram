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
    #[cfg(unix)]
    _sqlite_vfs: Option<Arc<descriptor_vfs::DescriptorBoundVfs>>,
}

/// Connection pool for concurrent access
pub struct StoragePool {
    config: StorageConfig,
    pool: Vec<Arc<Mutex<Connection>>>,
    #[cfg(unix)]
    _sqlite_vfs_guards: Vec<Option<Arc<descriptor_vfs::DescriptorBoundVfs>>>,
    next: std::sync::atomic::AtomicUsize,
}

struct CreatedConnection {
    conn: Connection,
    #[cfg(unix)]
    sqlite_vfs: Option<Arc<descriptor_vfs::DescriptorBoundVfs>>,
}

impl Storage {
    /// Open or create a database with the given configuration
    pub fn open(config: StorageConfig) -> Result<Self> {
        let created = Self::create_connection(&config)?;

        // Run migrations
        run_migrations(&created.conn)?;
        #[cfg(unix)]
        Self::reassert_sqlite_artifact_permissions_for_created(
            &config,
            created.sqlite_vfs.as_deref(),
        )?;
        #[cfg(not(unix))]
        Self::reassert_sqlite_artifact_permissions_for_config(&config)?;

        Ok(Self {
            config,
            conn: Arc::new(Mutex::new(created.conn)),
            #[cfg(unix)]
            _sqlite_vfs: created.sqlite_vfs,
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
    fn create_connection(config: &StorageConfig) -> Result<CreatedConnection> {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX;
        #[cfg(unix)]
        let flags = flags | OpenFlags::SQLITE_OPEN_NOFOLLOW;

        let (conn, sqlite_vfs) = if config.db_path == ":memory:" {
            (
                Connection::open_in_memory()?,
                #[cfg(unix)]
                None,
            )
        } else {
            ensure_filesystem_database_supported()?;
            let db_path = Path::new(&config.db_path);
            prepare_database_path(db_path)?;
            #[cfg(unix)]
            let mut sqlite_vfs = descriptor_vfs::DescriptorBoundVfs::new(db_path)?;
            #[cfg(unix)]
            run_sqlite_open_test_hook(db_path);
            #[cfg(unix)]
            sqlite_vfs.refresh_open_path_from_fd()?;
            #[cfg(unix)]
            let conn = Connection::open_with_flags_and_vfs(
                sqlite_vfs.open_path(),
                flags,
                sqlite_vfs.name(),
            )?;
            #[cfg(not(unix))]
            let conn = Connection::open_with_flags(db_path, flags)?;
            (
                conn,
                #[cfg(unix)]
                Some(Arc::new(sqlite_vfs)),
            )
        };

        // Configure based on storage mode (RML-874, RML-900)
        Self::configure_pragmas(&conn, config.storage_mode)?;
        #[cfg(unix)]
        Self::reassert_sqlite_artifact_permissions_for_created(config, sqlite_vfs.as_deref())?;
        #[cfg(not(unix))]
        Self::reassert_sqlite_artifact_permissions_for_config(config)?;

        Ok(CreatedConnection {
            conn,
            #[cfg(unix)]
            sqlite_vfs,
        })
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
        #[cfg(unix)]
        if let Some(sqlite_vfs) = &self._sqlite_vfs {
            return sqlite_vfs.reassert_artifact_permissions();
        }
        Self::reassert_sqlite_artifact_permissions_for_config(&self.config)
    }

    #[cfg(unix)]
    fn reassert_sqlite_artifact_permissions_for_created(
        config: &StorageConfig,
        sqlite_vfs: Option<&descriptor_vfs::DescriptorBoundVfs>,
    ) -> Result<()> {
        if let Some(sqlite_vfs) = sqlite_vfs {
            return sqlite_vfs.reassert_artifact_permissions();
        }
        Self::reassert_sqlite_artifact_permissions_for_config(config)
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

#[cfg(all(test, unix))]
type SqliteOpenTestHook = (PathBuf, Box<dyn FnOnce() + Send>);

#[cfg(all(test, unix))]
static SQLITE_OPEN_TEST_HOOK: std::sync::Mutex<Option<SqliteOpenTestHook>> =
    std::sync::Mutex::new(None);

#[cfg(all(test, unix))]
fn set_sqlite_open_test_hook(db_path: PathBuf, hook: impl FnOnce() + Send + 'static) {
    *SQLITE_OPEN_TEST_HOOK
        .lock()
        .expect("SQLite test hook poisoned") = Some((db_path, Box::new(hook)));
}

#[cfg(all(test, unix))]
fn run_sqlite_open_test_hook(db_path: &Path) {
    let hook = {
        let mut guard = SQLITE_OPEN_TEST_HOOK
            .lock()
            .expect("SQLite test hook poisoned");
        if guard
            .as_ref()
            .is_some_and(|(hook_path, _)| hook_path == db_path)
        {
            guard.take().map(|(_, hook)| hook)
        } else {
            None
        }
    };
    if let Some(hook) = hook {
        hook();
    }
}

#[cfg(all(unix, not(test)))]
fn run_sqlite_open_test_hook(_db_path: &Path) {}

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
        #[cfg(unix)]
        let mut sqlite_vfs_guards = Vec::with_capacity(pool_size);

        for _ in 0..pool_size {
            let created = Storage::create_connection(&config)?;
            #[cfg(unix)]
            sqlite_vfs_guards.push(created.sqlite_vfs);
            pool.push(Arc::new(Mutex::new(created.conn)));
        }

        // Run migrations on first connection
        if let Some(first) = pool.first() {
            let conn = first.lock();
            run_migrations(&conn)?;
        }
        #[cfg(unix)]
        {
            if let Some(Some(sqlite_vfs)) = sqlite_vfs_guards.first() {
                sqlite_vfs.reassert_artifact_permissions()?;
            } else {
                Storage::reassert_sqlite_artifact_permissions_for_config(&config)?;
            }
        }
        #[cfg(not(unix))]
        Storage::reassert_sqlite_artifact_permissions_for_config(&config)?;

        Ok(Self {
            config,
            pool,
            #[cfg(unix)]
            _sqlite_vfs_guards: sqlite_vfs_guards,
            next: std::sync::atomic::AtomicUsize::new(0),
        })
    }

    /// Get a connection from the pool (round-robin)
    pub fn get(&self) -> Arc<Mutex<Connection>> {
        let (_, conn) = self.get_entry();
        conn
    }

    fn get_entry(&self) -> (usize, Arc<Mutex<Connection>>) {
        let idx = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % self.pool.len();
        (idx, self.pool[idx].clone())
    }

    fn reassert_sqlite_artifact_permissions_after_pool_use(&self, idx: usize) -> Result<()> {
        #[cfg(unix)]
        if let Some(Some(sqlite_vfs)) = self._sqlite_vfs_guards.get(idx) {
            return sqlite_vfs.reassert_artifact_permissions();
        }
        Storage::reassert_sqlite_artifact_permissions_for_config(&self.config)
    }

    /// Execute a function with a connection from the pool
    pub fn with_connection<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let (idx, conn_arc) = self.get_entry();
        let result = {
            let conn = conn_arc.lock();
            f(&conn)
        };
        match result {
            Ok(value) => {
                self.reassert_sqlite_artifact_permissions_after_pool_use(idx)?;
                Ok(value)
            }
            Err(err) => {
                let _ = self.reassert_sqlite_artifact_permissions_after_pool_use(idx);
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
            #[cfg(unix)]
            _sqlite_vfs: self._sqlite_vfs.clone(),
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

#[cfg(not(unix))]
fn prepare_database_parent(parent: &Path) -> Result<()> {
    std::fs::create_dir_all(parent)?;
    Ok(())
}

#[cfg(unix)]
mod descriptor_vfs {
    use super::{restrict_open_regular_file_permissions, SQLITE_FILE_MODE};
    use crate::error::{EngramError, Result};
    use rusqlite::ffi;
    #[cfg(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "tvos",
        target_os = "watchos"
    ))]
    use std::ffi::OsString;
    use std::ffi::{CStr, CString};
    use std::fs::{File, OpenOptions};
    use std::os::raw::{c_char, c_int, c_void};
    use std::os::unix::ffi::OsStrExt;
    #[cfg(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "tvos",
        target_os = "watchos"
    ))]
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
    use std::path::{Path, PathBuf};
    use std::ptr;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_VFS_ID: AtomicU64 = AtomicU64::new(1);

    pub(super) struct DescriptorBoundVfs {
        name: String,
        _name_c: CString,
        open_path: PathBuf,
        vfs: Box<ffi::sqlite3_vfs>,
        context: Box<DescriptorVfsContext>,
    }

    // SAFETY: The VFS guard is immutable after registration. Its callbacks use
    // only owned descriptors and an immutable context, and `Storage`/
    // `StoragePool` drop rusqlite connections before unregistering the VFS
    // guard. Sharing the guard does not expose mutable raw-pointer access.
    unsafe impl Send for DescriptorBoundVfs {}
    unsafe impl Sync for DescriptorBoundVfs {}

    struct DescriptorVfsContext {
        default_vfs: *mut ffi::sqlite3_vfs,
        parent_dir: File,
        main_file: File,
        main_path: Vec<u8>,
        main_file_name: Vec<u8>,
        main_dev: u64,
        main_ino: u64,
        main_uid: u32,
    }

    enum BoundPath {
        Main,
        Sidecar(Vec<u8>),
    }

    impl DescriptorBoundVfs {
        pub(super) fn new(db_path: &Path) -> Result<Self> {
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
            let parent_dir = open_parent_dir(parent)?;
            let main_file = open_main_database_file(parent_dir.as_raw_fd(), file_name, db_path)?;
            let main_metadata = main_file.metadata()?;
            validate_main_metadata(&main_metadata, db_path)?;
            restrict_open_regular_file_permissions(&main_file)?;
            let open_path = current_path_for_fd(main_file.as_raw_fd())?;
            let main_path = path_to_cstring(&open_path)?.into_bytes();

            let default_vfs = unsafe {
                // SAFETY: `sqlite3_vfs_find(NULL)` returns SQLite's registered
                // default VFS after SQLite initialization. rusqlite initializes
                // SQLite before opening connections.
                ffi::sqlite3_vfs_find(ptr::null())
            };
            if default_vfs.is_null() {
                return Err(EngramError::Storage(
                    "SQLite default VFS is unavailable".to_string(),
                ));
            }

            let name = format!(
                "engram_fd_bound_{}_{}",
                std::process::id(),
                NEXT_VFS_ID.fetch_add(1, Ordering::Relaxed)
            );
            let name_c = CString::new(name.as_bytes()).map_err(|_| {
                EngramError::Storage("SQLite VFS name contained an interior NUL".to_string())
            })?;

            let mut context = Box::new(DescriptorVfsContext {
                default_vfs,
                parent_dir,
                main_file,
                main_path,
                main_file_name: file_name.as_bytes().to_vec(),
                main_dev: main_metadata.dev(),
                main_ino: main_metadata.ino(),
                main_uid: main_metadata.uid(),
            });

            let mut vfs = unsafe {
                // SAFETY: `default_vfs` is non-null and points to a live SQLite
                // VFS registration. Copying the function table lets this proxy
                // override path-sensitive methods while delegating all file I/O.
                *default_vfs
            };
            vfs.pNext = ptr::null_mut();
            vfs.zName = name_c.as_ptr();
            vfs.pAppData = (&mut *context as *mut DescriptorVfsContext).cast::<c_void>();
            vfs.xOpen = Some(x_open);
            vfs.xDelete = Some(x_delete);
            vfs.xAccess = Some(x_access);
            vfs.xFullPathname = Some(x_full_pathname);

            let mut vfs = Box::new(vfs);
            let rc = unsafe {
                // SAFETY: `vfs` is heap-allocated and remains pinned inside
                // `DescriptorBoundVfs` until Drop unregisters it after the
                // owning rusqlite connection is dropped.
                ffi::sqlite3_vfs_register(&mut *vfs, 0)
            };
            if rc != ffi::SQLITE_OK {
                return Err(EngramError::Storage(format!(
                    "failed to register descriptor-bound SQLite VFS: sqlite rc {rc}"
                )));
            }

            Ok(Self {
                name,
                _name_c: name_c,
                open_path,
                vfs,
                context,
            })
        }

        pub(super) fn name(&self) -> &str {
            &self.name
        }

        pub(super) fn open_path(&self) -> &Path {
            &self.open_path
        }

        pub(super) fn refresh_open_path_from_fd(&mut self) -> Result<()> {
            let open_path = current_path_for_fd(self.context.main_file.as_raw_fd())?;
            self.context.main_path = path_to_cstring(&open_path)?.into_bytes();
            self.open_path = open_path;
            Ok(())
        }

        pub(super) fn reassert_artifact_permissions(&self) -> Result<()> {
            restrict_open_regular_file_permissions(&self.context.main_file)?;
            for suffix in [
                b"-wal".as_slice(),
                b"-shm".as_slice(),
                b"-journal".as_slice(),
            ] {
                let mut name = self.context.main_file_name.clone();
                name.extend_from_slice(suffix);
                match openat_file(
                    self.context.parent_dir.as_raw_fd(),
                    &name,
                    libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                    0,
                ) {
                    Ok(file) => {
                        validate_sidecar_file(&self.context, &file)?;
                        restrict_open_regular_file_permissions(&file)?;
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                    Err(err) if err.raw_os_error() == Some(libc::ELOOP) => {
                        return Err(EngramError::Storage(
                            "refusing to chmod symlink SQLite sidecar".to_string(),
                        ));
                    }
                    Err(err) => return Err(err.into()),
                }
            }
            Ok(())
        }
    }

    impl Drop for DescriptorBoundVfs {
        fn drop(&mut self) {
            unsafe {
                // SAFETY: The owning `Storage` declares the rusqlite connection
                // field before this guard, so the connection closes before the
                // VFS unregisters during normal drops.
                ffi::sqlite3_vfs_unregister(&mut *self.vfs);
            }
        }
    }

    unsafe extern "C" fn x_open(
        vfs: *mut ffi::sqlite3_vfs,
        z_name: ffi::sqlite3_filename,
        file: *mut ffi::sqlite3_file,
        flags: c_int,
        out_flags: *mut c_int,
    ) -> c_int {
        let Some(context) = context(vfs) else {
            return ffi::SQLITE_CANTOPEN;
        };
        let Some(path) = c_path_bytes(z_name) else {
            return delegate_x_open(context, z_name, file, flags, out_flags);
        };

        match context.bound_path(path) {
            Some(BoundPath::Main) => match validate_current_main_path(context, path) {
                Ok(()) => {
                    let delegated_flags = if is_dev_fd_path(path) {
                        flags & !ffi::SQLITE_OPEN_NOFOLLOW
                    } else {
                        flags
                    };
                    delegate_x_open(context, z_name, file, delegated_flags, out_flags)
                }
                Err(_) => ffi::SQLITE_CANTOPEN,
            },
            Some(BoundPath::Sidecar(name)) => {
                match open_sidecar_fd(context, &name, flags).and_then(|fd| {
                    match validate_sidecar_fd(context, fd) {
                        Ok(()) => Ok(fd),
                        Err(err) => {
                            unsafe {
                                // SAFETY: `fd` was returned by `openat` and has not
                                // been transferred to SQLite because validation failed.
                                libc::close(fd);
                            }
                            Err(err)
                        }
                    }
                }) {
                    Ok(fd) => delegate_x_open_fd(context, fd, file, flags, out_flags),
                    Err(_) => ffi::SQLITE_CANTOPEN,
                }
            }
            None => delegate_x_open(context, z_name, file, flags, out_flags),
        }
    }

    unsafe extern "C" fn x_delete(
        vfs: *mut ffi::sqlite3_vfs,
        z_name: *const c_char,
        sync_dir: c_int,
    ) -> c_int {
        let Some(context) = context(vfs) else {
            return ffi::SQLITE_IOERR_DELETE;
        };
        let Some(path) = c_path_bytes(z_name) else {
            return delegate_x_delete(context, z_name, sync_dir);
        };

        match context.bound_path(path) {
            Some(BoundPath::Sidecar(name)) => {
                let Ok(name) = CString::new(name) else {
                    return ffi::SQLITE_IOERR_DELETE;
                };
                let rc = unsafe {
                    // SAFETY: `name` is a NUL-terminated path relative to the
                    // held parent directory fd.
                    libc::unlinkat(context.parent_dir.as_raw_fd(), name.as_ptr(), 0)
                };
                if rc != 0 {
                    let err = std::io::Error::last_os_error();
                    if err.raw_os_error() != Some(libc::ENOENT) {
                        return ffi::SQLITE_IOERR_DELETE;
                    }
                }
                if sync_dir != 0 {
                    let sync_rc = unsafe {
                        // SAFETY: `parent_dir` is an open directory fd owned by
                        // the VFS context.
                        libc::fsync(context.parent_dir.as_raw_fd())
                    };
                    if sync_rc != 0 {
                        return ffi::SQLITE_IOERR_DIR_FSYNC;
                    }
                }
                ffi::SQLITE_OK
            }
            Some(BoundPath::Main) => ffi::SQLITE_IOERR_DELETE,
            None => delegate_x_delete(context, z_name, sync_dir),
        }
    }

    unsafe extern "C" fn x_access(
        vfs: *mut ffi::sqlite3_vfs,
        z_name: *const c_char,
        flags: c_int,
        result: *mut c_int,
    ) -> c_int {
        let Some(context) = context(vfs) else {
            return ffi::SQLITE_IOERR_ACCESS;
        };
        if result.is_null() {
            return ffi::SQLITE_IOERR_ACCESS;
        }
        let Some(path) = c_path_bytes(z_name) else {
            return delegate_x_access(context, z_name, flags, result);
        };

        match context.bound_path(path) {
            Some(BoundPath::Main) => {
                unsafe {
                    // SAFETY: SQLite provided a non-null result pointer.
                    *result = 1;
                }
                ffi::SQLITE_OK
            }
            Some(BoundPath::Sidecar(name)) => {
                let exists = sidecar_access(context, &name, flags);
                match exists {
                    Ok(value) => {
                        unsafe {
                            // SAFETY: SQLite provided a non-null result pointer.
                            *result = i32::from(value);
                        }
                        ffi::SQLITE_OK
                    }
                    Err(_) => ffi::SQLITE_IOERR_ACCESS,
                }
            }
            None => delegate_x_access(context, z_name, flags, result),
        }
    }

    unsafe extern "C" fn x_full_pathname(
        vfs: *mut ffi::sqlite3_vfs,
        z_name: *const c_char,
        output_len: c_int,
        output: *mut c_char,
    ) -> c_int {
        let Some(context) = context(vfs) else {
            return ffi::SQLITE_CANTOPEN;
        };
        let Some(path) = c_path_bytes(z_name) else {
            return delegate_x_full_pathname(context, z_name, output_len, output);
        };
        if context.bound_path(path).is_some() {
            return copy_path(&context.main_path, output_len, output);
        }
        delegate_x_full_pathname(context, z_name, output_len, output)
    }

    impl DescriptorVfsContext {
        fn bound_path(&self, path: &[u8]) -> Option<BoundPath> {
            if path == self.main_path.as_slice() {
                return Some(BoundPath::Main);
            }
            for suffix in [
                b"-wal".as_slice(),
                b"-shm".as_slice(),
                b"-journal".as_slice(),
            ] {
                if self.is_main_path_sidecar(path, suffix) || is_dev_fd_sidecar(path, suffix) {
                    let mut name = self.main_file_name.clone();
                    name.extend_from_slice(suffix);
                    return Some(BoundPath::Sidecar(name));
                }
            }
            None
        }

        fn is_main_path_sidecar(&self, path: &[u8], suffix: &[u8]) -> bool {
            path.len() == self.main_path.len() + suffix.len()
                && path.starts_with(&self.main_path)
                && path.ends_with(suffix)
        }
    }

    fn is_dev_fd_sidecar(path: &[u8], suffix: &[u8]) -> bool {
        let Some(stem) = path
            .strip_prefix(b"/dev/fd/")
            .and_then(|rest| rest.strip_suffix(suffix))
        else {
            return false;
        };
        !stem.is_empty() && stem.iter().all(u8::is_ascii_digit)
    }

    fn is_dev_fd_path(path: &[u8]) -> bool {
        let Some(fd) = path.strip_prefix(b"/dev/fd/") else {
            return false;
        };
        !fd.is_empty() && fd.iter().all(u8::is_ascii_digit)
    }

    unsafe fn context(vfs: *mut ffi::sqlite3_vfs) -> Option<&'static DescriptorVfsContext> {
        if vfs.is_null() {
            return None;
        }
        let ptr = unsafe {
            // SAFETY: caller supplied a SQLite VFS pointer. The proxy stores a
            // DescriptorVfsContext pointer in pAppData for every registered VFS.
            (*vfs).pAppData.cast::<DescriptorVfsContext>()
        };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe {
                // SAFETY: The context lives inside the registered
                // DescriptorBoundVfs for at least as long as SQLite can call the
                // proxy methods.
                &*ptr
            })
        }
    }

    fn c_path_bytes<'a>(path: *const c_char) -> Option<&'a [u8]> {
        if path.is_null() {
            return None;
        }
        Some(unsafe {
            // SAFETY: SQLite VFS paths are NUL-terminated strings for non-temp
            // files. Null is handled above.
            CStr::from_ptr(path).to_bytes()
        })
    }

    unsafe fn delegate_x_open(
        context: &DescriptorVfsContext,
        z_name: ffi::sqlite3_filename,
        file: *mut ffi::sqlite3_file,
        flags: c_int,
        out_flags: *mut c_int,
    ) -> c_int {
        let default = unsafe { &*context.default_vfs };
        match default.xOpen {
            Some(open) => unsafe { open(context.default_vfs, z_name, file, flags, out_flags) },
            None => ffi::SQLITE_CANTOPEN,
        }
    }

    fn delegate_x_open_fd(
        context: &DescriptorVfsContext,
        fd: RawFd,
        file: *mut ffi::sqlite3_file,
        flags: c_int,
        out_flags: *mut c_int,
    ) -> c_int {
        let path = match CString::new(format!("/dev/fd/{fd}")) {
            Ok(path) => path,
            Err(_) => {
                unsafe {
                    // SAFETY: `fd` is owned by this helper until it is either
                    // closed here or duplicated by the delegated VFS open.
                    libc::close(fd);
                }
                return ffi::SQLITE_CANTOPEN;
            }
        };
        let delegated_flags = flags & !ffi::SQLITE_OPEN_NOFOLLOW;
        let rc =
            unsafe { delegate_x_open(context, path.as_ptr(), file, delegated_flags, out_flags) };
        unsafe {
            // SAFETY: The default VFS opens `/dev/fd/<fd>` synchronously during
            // the call above. This descriptor is no longer needed afterwards.
            libc::close(fd);
        }
        rc
    }

    unsafe fn delegate_x_delete(
        context: &DescriptorVfsContext,
        z_name: *const c_char,
        sync_dir: c_int,
    ) -> c_int {
        let default = unsafe { &*context.default_vfs };
        match default.xDelete {
            Some(delete) => unsafe { delete(context.default_vfs, z_name, sync_dir) },
            None => ffi::SQLITE_IOERR_DELETE,
        }
    }

    unsafe fn delegate_x_access(
        context: &DescriptorVfsContext,
        z_name: *const c_char,
        flags: c_int,
        result: *mut c_int,
    ) -> c_int {
        let default = unsafe { &*context.default_vfs };
        match default.xAccess {
            Some(access) => unsafe { access(context.default_vfs, z_name, flags, result) },
            None => ffi::SQLITE_IOERR_ACCESS,
        }
    }

    unsafe fn delegate_x_full_pathname(
        context: &DescriptorVfsContext,
        z_name: *const c_char,
        output_len: c_int,
        output: *mut c_char,
    ) -> c_int {
        let default = unsafe { &*context.default_vfs };
        match default.xFullPathname {
            Some(full_pathname) => unsafe {
                full_pathname(context.default_vfs, z_name, output_len, output)
            },
            None => ffi::SQLITE_CANTOPEN,
        }
    }

    fn copy_path(path: &[u8], output_len: c_int, output: *mut c_char) -> c_int {
        if output.is_null() || output_len <= 0 {
            return ffi::SQLITE_CANTOPEN;
        }
        let needed = path.len().saturating_add(1);
        if needed > output_len as usize {
            return ffi::SQLITE_CANTOPEN;
        }
        unsafe {
            // SAFETY: Bounds were checked above and SQLite provided the output
            // buffer. We also write the trailing NUL expected by SQLite.
            ptr::copy_nonoverlapping(path.as_ptr(), output.cast::<u8>(), path.len());
            *output.add(path.len()) = 0;
        }
        ffi::SQLITE_OK
    }

    fn open_parent_dir(parent: &Path) -> Result<File> {
        OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_CLOEXEC | libc::O_DIRECTORY | libc::O_NOFOLLOW)
            .open(parent)
            .map_err(Into::into)
    }

    fn open_main_database_file(
        parent_fd: RawFd,
        file_name: &std::ffi::OsStr,
        db_path: &Path,
    ) -> Result<File> {
        let flags = libc::O_RDWR | libc::O_CREAT | libc::O_CLOEXEC | libc::O_NOFOLLOW;
        match openat_file(parent_fd, file_name.as_bytes(), flags, SQLITE_FILE_MODE) {
            Ok(file) => Ok(file),
            Err(err) if err.raw_os_error() == Some(libc::ELOOP) => {
                Err(EngramError::Storage(format!(
                    "refusing to open symlink SQLite database '{}'",
                    db_path.display()
                )))
            }
            Err(err) => Err(err.into()),
        }
    }

    fn validate_main_metadata(metadata: &std::fs::Metadata, db_path: &Path) -> Result<()> {
        if !metadata.is_file() {
            return Err(EngramError::Storage(format!(
                "refusing to open non-regular SQLite database '{}'",
                db_path.display()
            )));
        }
        if metadata.nlink() != 1 {
            return Err(EngramError::Storage(format!(
                "refusing to open multiply linked SQLite database '{}'",
                db_path.display()
            )));
        }
        Ok(())
    }

    fn open_sidecar_fd(
        context: &DescriptorVfsContext,
        name: &[u8],
        sqlite_flags: c_int,
    ) -> std::io::Result<RawFd> {
        let read_only = sqlite_flags & ffi::SQLITE_OPEN_READONLY != 0
            && sqlite_flags & ffi::SQLITE_OPEN_READWRITE == 0;
        let mut open_flags = if read_only {
            libc::O_RDONLY
        } else {
            libc::O_RDWR
        };
        if sqlite_flags & ffi::SQLITE_OPEN_CREATE != 0 {
            open_flags |= libc::O_CREAT;
        }
        if sqlite_flags & ffi::SQLITE_OPEN_EXCLUSIVE != 0 {
            open_flags |= libc::O_EXCL;
        }
        open_flags |= libc::O_CLOEXEC | libc::O_NOFOLLOW;

        openat_raw(
            context.parent_dir.as_raw_fd(),
            name,
            open_flags,
            SQLITE_FILE_MODE,
        )
    }

    fn sidecar_access(
        context: &DescriptorVfsContext,
        name: &[u8],
        sqlite_flags: c_int,
    ) -> std::io::Result<bool> {
        let read_write = sqlite_flags == ffi::SQLITE_ACCESS_READWRITE;
        let flags = if read_write {
            libc::O_RDWR
        } else {
            libc::O_RDONLY
        };
        match openat_raw(
            context.parent_dir.as_raw_fd(),
            name,
            flags | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            0,
        ) {
            Ok(fd) => {
                unsafe {
                    // SAFETY: `fd` is owned by this function.
                    libc::close(fd);
                }
                Ok(true)
            }
            Err(err) if err.raw_os_error() == Some(libc::ENOENT) => Ok(false),
            Err(err) => Err(err),
        }
    }

    fn validate_sidecar_fd(context: &DescriptorVfsContext, fd: RawFd) -> std::io::Result<()> {
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        let rc = unsafe {
            // SAFETY: `stat` points to valid uninitialized memory and `fd` is
            // an open file descriptor.
            libc::fstat(fd, stat.as_mut_ptr())
        };
        if rc != 0 {
            return Err(std::io::Error::last_os_error());
        }
        let stat = unsafe {
            // SAFETY: fstat returned success, so `stat` was initialized.
            stat.assume_init()
        };
        if (stat.st_mode & libc::S_IFMT) != libc::S_IFREG {
            return Err(std::io::Error::from_raw_os_error(libc::EINVAL));
        }
        if stat.st_uid != context.main_uid {
            return Err(std::io::Error::from_raw_os_error(libc::EACCES));
        }
        if stat.st_nlink != 1 {
            return Err(std::io::Error::from_raw_os_error(libc::EACCES));
        }
        Ok(())
    }

    fn validate_sidecar_file(context: &DescriptorVfsContext, file: &File) -> Result<()> {
        validate_sidecar_fd(context, file.as_raw_fd()).map_err(Into::into)
    }

    fn validate_current_main_path(
        context: &DescriptorVfsContext,
        path: &[u8],
    ) -> std::io::Result<()> {
        let path = CString::new(path).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "path contained an interior NUL",
            )
        })?;
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        let rc = unsafe {
            // SAFETY: `path` is NUL-terminated and `stat` points to valid
            // uninitialized memory. Descriptor aliases must be followed so
            // the identity check applies to the held regular file; ordinary
            // paths use lstat(2) to keep rejecting symlinks.
            if is_dev_fd_path(path.to_bytes()) {
                libc::stat(path.as_ptr(), stat.as_mut_ptr())
            } else {
                libc::lstat(path.as_ptr(), stat.as_mut_ptr())
            }
        };
        if rc != 0 {
            return Err(std::io::Error::last_os_error());
        }
        let stat = unsafe {
            // SAFETY: lstat returned success, so `stat` was initialized.
            stat.assume_init()
        };
        if (stat.st_mode & libc::S_IFMT) != libc::S_IFREG {
            return Err(std::io::Error::from_raw_os_error(libc::EINVAL));
        }
        if !device_id_matches(stat.st_dev, context.main_dev) || stat.st_ino != context.main_ino {
            return Err(std::io::Error::from_raw_os_error(libc::EACCES));
        }
        Ok(())
    }

    fn device_id_matches<T>(actual: T, expected: u64) -> bool
    where
        u64: TryFrom<T>,
    {
        u64::try_from(actual).is_ok_and(|actual| actual == expected)
    }

    #[cfg(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "tvos",
        target_os = "watchos"
    ))]
    fn current_path_for_fd(fd: RawFd) -> Result<PathBuf> {
        let mut path = vec![0 as libc::c_char; libc::PATH_MAX as usize];
        let rc = unsafe {
            // SAFETY: `path` is a writable buffer large enough for F_GETPATH
            // and `fd` is an open file descriptor owned by the VFS context.
            libc::fcntl(fd, libc::F_GETPATH, path.as_mut_ptr())
        };
        if rc != 0 {
            return Err(std::io::Error::last_os_error().into());
        }
        let path = unsafe {
            // SAFETY: F_GETPATH succeeded and wrote a NUL-terminated path.
            CStr::from_ptr(path.as_ptr())
        };
        Ok(PathBuf::from(OsString::from_vec(path.to_bytes().to_vec())))
    }

    #[cfg(target_os = "linux")]
    fn current_path_for_fd(fd: RawFd) -> Result<PathBuf> {
        std::fs::read_link(format!("/proc/self/fd/{fd}")).map_err(Into::into)
    }

    #[cfg(all(
        unix,
        not(target_os = "linux"),
        not(target_os = "macos"),
        not(target_os = "ios"),
        not(target_os = "tvos"),
        not(target_os = "watchos")
    ))]
    fn current_path_for_fd(fd: RawFd) -> Result<PathBuf> {
        Ok(PathBuf::from(format!("/dev/fd/{fd}")))
    }

    fn openat_file(
        parent_fd: RawFd,
        name: &[u8],
        flags: c_int,
        mode: u32,
    ) -> std::io::Result<File> {
        let fd = openat_raw(parent_fd, name, flags, mode)?;
        Ok(unsafe {
            // SAFETY: `openat_raw` returned a newly owned file descriptor.
            File::from_raw_fd(fd)
        })
    }

    fn openat_raw(
        parent_fd: RawFd,
        name: &[u8],
        flags: c_int,
        mode: u32,
    ) -> std::io::Result<RawFd> {
        let name = CString::new(name).map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "path contained an interior NUL",
            )
        })?;
        let fd = unsafe {
            // SAFETY: `name` is NUL-terminated and `parent_fd` is an open
            // directory descriptor.
            libc::openat(parent_fd, name.as_ptr(), flags, mode as libc::c_uint)
        };
        if fd < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(fd)
        }
    }

    fn path_to_cstring(path: &Path) -> Result<CString> {
        CString::new(path.as_os_str().as_bytes()).map_err(|_| {
            EngramError::Storage(format!(
                "SQLite database path '{}' contains an interior NUL",
                path.display()
            ))
        })
    }

    #[cfg(test)]
    mod tests {
        use super::is_dev_fd_path;

        #[test]
        fn recognizes_only_numeric_dev_fd_aliases() {
            assert!(is_dev_fd_path(b"/dev/fd/7"));
            assert!(is_dev_fd_path(b"/dev/fd/123"));
            assert!(!is_dev_fd_path(b"/dev/fd/"));
            assert!(!is_dev_fd_path(b"/dev/fd/7-wal"));
            assert!(!is_dev_fd_path(b"/dev/fd/not-a-fd"));
            assert!(!is_dev_fd_path(b"/tmp/database.db"));
        }
    }
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
    let mut paths = Vec::with_capacity(4);
    paths.push(db_path.to_path_buf());
    for suffix in ["-wal", "-shm", "-journal"] {
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
    fn unix_parent_replacement_after_verified_open_cannot_redirect_sqlite_or_chmod() {
        use sha2::{Digest, Sha256};

        fn digest_file(path: &Path) -> Vec<u8> {
            Sha256::digest(std::fs::read(path).unwrap()).to_vec()
        }

        let temp = tempfile::tempdir().unwrap();
        let parent = temp.path().join("db-parent");
        let verified_parent = temp.path().join("verified-parent");
        let db_path = parent.join("memory.db");
        std::fs::create_dir(&parent).unwrap();
        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700)).unwrap();

        let original = Storage::open(file_config(&db_path)).unwrap();
        original
            .with_connection(|conn| {
                conn.execute_batch(
                    "CREATE TABLE sentinel (value TEXT NOT NULL);
                     INSERT INTO sentinel VALUES ('original');",
                )?;
                Ok(())
            })
            .unwrap();
        drop(original);

        let replacement_digest = std::sync::Arc::new(std::sync::Mutex::new(None::<Vec<u8>>));
        let replacement_digest_for_hook = replacement_digest.clone();
        let parent_for_hook = parent.clone();
        let verified_parent_for_hook = verified_parent.clone();
        let db_path_for_hook = db_path.clone();
        set_sqlite_open_test_hook(db_path.clone(), move || {
            std::fs::rename(&parent_for_hook, &verified_parent_for_hook).unwrap();
            std::fs::create_dir(&parent_for_hook).unwrap();
            std::fs::set_permissions(&parent_for_hook, std::fs::Permissions::from_mode(0o700))
                .unwrap();

            let replacement = Connection::open(&db_path_for_hook).unwrap();
            replacement
                .execute_batch(
                    "PRAGMA journal_mode=DELETE;
                     CREATE TABLE sentinel (value TEXT NOT NULL);
                     INSERT INTO sentinel VALUES ('replacement');",
                )
                .unwrap();
            drop(replacement);
            std::fs::set_permissions(&db_path_for_hook, std::fs::Permissions::from_mode(0o644))
                .unwrap();
            *replacement_digest_for_hook.lock().unwrap() = Some(digest_file(&db_path_for_hook));
        });

        let storage = Storage::open(file_config(&db_path)).unwrap();
        let opened_sentinel: String = storage
            .with_connection(|conn| {
                Ok(conn.query_row("SELECT value FROM sentinel", [], |row| row.get(0))?)
            })
            .unwrap();
        assert_eq!(opened_sentinel, "original");
        drop(storage);

        let replacement_digest_before = replacement_digest.lock().unwrap().clone().unwrap();
        assert_eq!(digest_file(&db_path), replacement_digest_before);
        let replacement_mode = std::fs::metadata(&db_path).unwrap().permissions().mode() & 0o777;
        assert_eq!(replacement_mode, 0o644);

        let replacement = Connection::open(&db_path).unwrap();
        let replacement_sentinel: String = replacement
            .query_row("SELECT value FROM sentinel", [], |row| row.get(0))
            .unwrap();
        let replacement_journal_mode: String = replacement
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        assert_eq!(replacement_sentinel, "replacement");
        assert_eq!(replacement_journal_mode, "delete");

        let verified_db = verified_parent.join("memory.db");
        let verified = Connection::open(&verified_db).unwrap();
        let verified_sentinel: String = verified
            .query_row("SELECT value FROM sentinel", [], |row| row.get(0))
            .unwrap();
        assert_eq!(verified_sentinel, "original");
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
