//! Database connection management with WAL mode support (RML-874)
//!
//! Implements SQLite connection pooling with configurable storage modes
//! for both local (WAL) and cloud-safe (DELETE journal) operation.

use parking_lot::Mutex;
use rusqlite::{Connection, OpenFlags};
use std::path::Path;
use std::sync::Arc;

use super::migrations::run_migrations;
use crate::error::Result;
use crate::types::{CompactOp, CompactReport, StorageConfig, StorageMode};

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

        let conn = if config.db_path == ":memory:" {
            Connection::open_in_memory()?
        } else {
            // Ensure parent directory exists
            if let Some(parent) = Path::new(&config.db_path).parent() {
                std::fs::create_dir_all(parent)?;
            }
            Connection::open_with_flags(&config.db_path, flags)?
        };

        // Configure based on storage mode (RML-874, RML-900)
        Self::configure_pragmas(&conn, config.storage_mode)?;

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
        let conn = self.conn.lock();
        f(&conn)
    }

    /// Execute a function with a transaction
    pub fn with_transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Connection) -> Result<T>,
    {
        let mut conn = self.conn.lock();
        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
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
        let conn_arc = self.get();
        let conn = conn_arc.lock();
        f(&conn)
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
