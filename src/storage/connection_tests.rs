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
    let journal_mode: String = conn.query_scalar_0("PRAGMA journal_mode").unwrap();
    let sentinel: String = conn.query_scalar_0("SELECT value FROM sentinel").unwrap();
    assert_eq!(journal_mode, "delete");
    assert_eq!(sentinel, "unchanged");
}

#[cfg(unix)]
#[test]
fn unix_symlink_swapped_in_before_chmod_is_not_followed() {
    let temp = tempfile::tempdir().unwrap();
    let artifact_path = temp.path().join("memory.db");
    let moved_inode_path = temp.path().join("moved-memory.db");
    let symlink_target = temp.path().join("unrelated-target");

    std::fs::write(&artifact_path, b"checked inode").unwrap();
    std::fs::set_permissions(&artifact_path, std::fs::Permissions::from_mode(0o666)).unwrap();
    std::fs::write(&symlink_target, b"unrelated target").unwrap();
    std::fs::set_permissions(&symlink_target, std::fs::Permissions::from_mode(0o644)).unwrap();

    // Swap the checked path for a symlink between the lstat and the chmod.
    let result = restrict_sqlite_artifact_mode(&artifact_path, || {
        std::fs::rename(&artifact_path, &moved_inode_path).unwrap();
        std::os::unix::fs::symlink(&symlink_target, &artifact_path).unwrap();
    });

    // macOS changes the link's own mode (lchmod); Linux refuses the link.
    if let Err(err) = result {
        assert!(
            err.to_string().contains("refusing to chmod symlink"),
            "unexpected error: {err}"
        );
    }
    let target_mode = std::fs::metadata(&symlink_target)
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(target_mode, 0o644, "chmod followed the swapped-in symlink");
}

#[cfg(unix)]
#[test]
fn unix_recheck_fallback_chmods_only_the_checked_inode() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("memory.db-wal");
    let other = temp.path().join("other");
    let target = temp.path().join("target");
    std::fs::write(&path, b"wal").unwrap();
    std::fs::write(&other, b"other").unwrap();
    std::fs::write(&target, b"target").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o644)).unwrap();
    let mode_of = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;

    // Same inode: narrowed.
    let seen = std::fs::symlink_metadata(&path).unwrap();
    chmod_artifact_after_recheck(&path, 0o600, &seen).unwrap();
    assert_eq!(mode_of(&path), 0o600);

    // Replaced by another regular file: refused, replacement untouched.
    std::fs::set_permissions(&other, std::fs::Permissions::from_mode(0o400)).unwrap();
    std::fs::rename(&other, &path).unwrap();
    let err = chmod_artifact_after_recheck(&path, 0o600, &seen).unwrap_err();
    assert!(
        err.to_string().contains("changed while restricting"),
        "{err}"
    );
    assert_eq!(mode_of(&path), 0o400);

    // Replaced by a symlink: refused, target untouched.
    std::fs::remove_file(&path).unwrap();
    std::os::unix::fs::symlink(&target, &path).unwrap();
    let err = chmod_artifact_after_recheck(&path, 0o600, &seen).unwrap_err();
    assert!(
        err.to_string().contains("refusing to chmod symlink"),
        "{err}"
    );
    assert_eq!(mode_of(&target), 0o644);

    // Vanished: nothing to do.
    std::fs::remove_file(&path).unwrap();
    chmod_artifact_after_recheck(&path, 0o600, &seen).unwrap();
}

#[cfg(unix)]
#[test]
fn unix_permissive_artifact_is_narrowed_by_path() {
    let temp = tempfile::tempdir().unwrap();
    let db_path = temp.path().join("memory.db");
    let wal_path = temp.path().join("memory.db-wal");
    std::fs::write(&db_path, b"db").unwrap();
    std::fs::write(&wal_path, b"wal").unwrap();
    std::fs::set_permissions(&db_path, std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::set_permissions(&wal_path, std::fs::Permissions::from_mode(0o660)).unwrap();

    restrict_sqlite_artifact_permissions(&db_path).unwrap();

    assert_sqlite_artifact_modes(&db_path, 0o600);
}

#[cfg(unix)]
#[test]
fn unix_non_regular_artifact_is_left_alone() {
    let temp = tempfile::tempdir().unwrap();
    let db_path = temp.path().join("memory.db");
    let shm_dir = temp.path().join("memory.db-shm");
    std::fs::write(&db_path, b"db").unwrap();
    std::fs::create_dir(&shm_dir).unwrap();
    std::fs::set_permissions(&shm_dir, std::fs::Permissions::from_mode(0o755)).unwrap();

    restrict_sqlite_artifact_permissions(&db_path).unwrap();

    let mode = std::fs::metadata(&shm_dir).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o755);
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

#[test]
fn test_shm_readonly_sqlite_error_classification_and_retry() {
    let make_sqlite_err = |code: i32| -> EngramError {
        EngramError::Database(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(code),
            Some("simulated sqlite error".to_string()),
        ))
    };

    let readonly_err = make_sqlite_err(rusqlite::ffi::SQLITE_READONLY);
    let cantopen_err = make_sqlite_err(rusqlite::ffi::SQLITE_CANTOPEN);
    let busy_err = make_sqlite_err(rusqlite::ffi::SQLITE_BUSY);
    let locked_err = make_sqlite_err(rusqlite::ffi::SQLITE_LOCKED);

    assert!(is_readonly_sqlite_error(&readonly_err));
    assert!(!is_readonly_sqlite_error(&cantopen_err));
    assert!(!is_readonly_sqlite_error(&busy_err));
    assert!(!is_readonly_sqlite_error(&locked_err));

    assert!(is_retryable_shm_open_error(&readonly_err));
    assert!(is_retryable_shm_open_error(&cantopen_err));
    assert!(!is_retryable_shm_open_error(&busy_err));
    assert!(!is_retryable_shm_open_error(&locked_err));

    let db_path = Path::new("/tmp/engram_shm_retry_unit.db");

    // Transient SQLITE_READONLY on initial attempts recovers via drop+reopen retry.
    let mut attempts = 0usize;
    let recovered = open_with_shm_readonly_retry(db_path, |attempt| {
        attempts += 1;
        if attempt < 2 {
            Err(make_sqlite_err(rusqlite::ffi::SQLITE_READONLY))
        } else {
            Ok("writable-connection")
        }
    })
    .expect("transient SQLITE_READONLY should recover within retry budget");
    assert_eq!(recovered, "writable-connection");
    assert_eq!(attempts, 3);

    // SQLITE_BUSY is NOT retried by the -shm read-only retry loop.
    let mut busy_attempts = 0usize;
    let err = open_with_shm_readonly_retry::<(), _>(db_path, |_| {
        busy_attempts += 1;
        Err(make_sqlite_err(rusqlite::ffi::SQLITE_BUSY))
    })
    .unwrap_err();
    assert_eq!(busy_attempts, 1);
    assert!(matches!(err, EngramError::Database(_)));

    // Persistent SQLITE_READONLY exhausts MAX_SHM_READONLY_OPEN_RETRIES and fails loudly.
    let mut exhausted_attempts = 0usize;
    let err = open_with_shm_readonly_retry::<(), _>(db_path, |_| {
        exhausted_attempts += 1;
        Err(make_sqlite_err(rusqlite::ffi::SQLITE_READONLY))
    })
    .unwrap_err();
    assert_eq!(exhausted_attempts, MAX_SHM_READONLY_OPEN_RETRIES + 1);
    assert!(
        err.to_string()
            .contains("SQLite WAL shared-memory (-shm) state is read-only"),
        "unexpected error message: {err}"
    );
}

#[cfg(unix)]
#[test]
fn test_concurrent_wal_open_write_close_churn() {
    const THREADS: usize = 6;
    const ITERS_PER_THREAD: usize = 60;

    let temp = tempfile::tempdir().unwrap();
    let db_path = temp.path().join("wal_churn.db");

    // Seed schema and close so the initial -wal/-shm are unlinked before churn begins.
    {
        let storage = Storage::open(file_config(&db_path)).unwrap();
        storage
            .with_connection(|conn| {
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS wal_churn_probe (
                        id INTEGER PRIMARY KEY AUTOINCREMENT,
                        thread_id INTEGER NOT NULL,
                        iter INTEGER NOT NULL
                    );",
                )?;
                Ok(())
            })
            .unwrap();
    }

    let barrier = std::sync::Arc::new(std::sync::Barrier::new(THREADS));
    let mut handles = Vec::with_capacity(THREADS);

    for thread_id in 0..THREADS {
        let barrier = barrier.clone();
        let db_path = db_path.clone();
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            for iter in 0..ITERS_PER_THREAD {
                let storage = Storage::open(file_config(&db_path)).unwrap_or_else(|err| {
                    panic!("thread {thread_id} iter {iter}: Storage::open failed: {err}")
                });
                storage
                    .with_transaction(|conn| {
                        conn.execute(
                            "INSERT INTO wal_churn_probe (thread_id, iter) VALUES (?1, ?2)",
                            rusqlite::params![thread_id as i64, iter as i64],
                        )?;
                        Ok(())
                    })
                    .unwrap_or_else(|err| {
                        panic!("thread {thread_id} iter {iter}: write transaction failed: {err}")
                    });
                drop(storage);
            }
        }));
    }

    for handle in handles {
        handle.join().expect("churn worker thread panicked");
    }

    let final_storage = Storage::open(file_config(&db_path)).unwrap();
    let total_rows: i64 = final_storage
        .with_connection(|conn| Ok(conn.query_scalar_0("SELECT COUNT(*) FROM wal_churn_probe")?))
        .unwrap();
    assert_eq!(total_rows, (THREADS * ITERS_PER_THREAD) as i64);
}

#[cfg(unix)]
#[test]
fn unix_read_only_with_connection_preserves_and_narrows_artifact_permissions() {
    use std::os::unix::fs::MetadataExt;

    let temp = tempfile::tempdir().unwrap();
    let db_path = temp.path().join("read_only_perm.db");
    let storage = Storage::open(file_config(&db_path)).unwrap();

    // Create a table and insert a row so WAL mode artifacts are active.
    storage
        .with_transaction(|conn| {
            conn.execute_batch(
                "CREATE TABLE test_data (id INTEGER PRIMARY KEY, val TEXT);
                 INSERT INTO test_data (val) VALUES ('item');",
            )?;
            Ok(())
        })
        .unwrap();

    assert_sqlite_artifact_modes(&db_path, 0o600);

    let wal_path = Path::new(&format!("{}-wal", db_path.display())).to_path_buf();
    let shm_path = Path::new(&format!("{}-shm", db_path.display())).to_path_buf();
    assert!(db_path.is_file());
    assert!(wal_path.is_file());
    assert!(shm_path.is_file());

    // 1. Read-only with_connection preserves 0o600 and does NOT mutate inode ctime
    // (no unnecessary fchmodat writes when already 0o600).
    let db_ctime_before = std::fs::symlink_metadata(&db_path).unwrap().ctime_nsec();
    let wal_ctime_before = std::fs::symlink_metadata(&wal_path).unwrap().ctime_nsec();
    let shm_ctime_before = std::fs::symlink_metadata(&shm_path).unwrap().ctime_nsec();

    let count: i64 = storage
        .with_connection(|conn| {
            Ok(conn.query_row("SELECT COUNT(*) FROM test_data", [], |r| r.get(0))?)
        })
        .unwrap();
    assert_eq!(count, 1);

    assert_sqlite_artifact_modes(&db_path, 0o600);

    let db_ctime_after = std::fs::symlink_metadata(&db_path).unwrap().ctime_nsec();
    let wal_ctime_after = std::fs::symlink_metadata(&wal_path).unwrap().ctime_nsec();
    let shm_ctime_after = std::fs::symlink_metadata(&shm_path).unwrap().ctime_nsec();

    assert_eq!(
        db_ctime_before, db_ctime_after,
        "db inode was mutated on pure read"
    );
    assert_eq!(
        wal_ctime_before, wal_ctime_after,
        "wal inode was mutated on pure read"
    );
    assert_eq!(
        shm_ctime_before, shm_ctime_after,
        "shm inode was mutated on pure read"
    );

    // 2. Broadening artifacts to 0o644 triggers re-narrowing back to 0o600 on read-only with_connection.
    std::fs::set_permissions(&db_path, std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::set_permissions(&wal_path, std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::set_permissions(&shm_path, std::fs::Permissions::from_mode(0o644)).unwrap();

    let val: String = storage
        .with_connection(|conn| {
            Ok(conn.query_row("SELECT val FROM test_data WHERE id = 1", [], |r| r.get(0))?)
        })
        .unwrap();
    assert_eq!(val, "item");

    assert_sqlite_artifact_modes(&db_path, 0o600);

    // 3. Broadening only wal_path followed by a write inside with_connection narrows it back to 0o600.
    std::fs::set_permissions(&wal_path, std::fs::Permissions::from_mode(0o644)).unwrap();
    storage
        .with_connection(|conn| {
            conn.execute("INSERT INTO test_data (val) VALUES ('item2')", [])?;
            Ok(())
        })
        .unwrap();
    assert_sqlite_artifact_modes(&db_path, 0o600);

    // 4. StoragePool also reasserts and narrows 0o644 artifacts back to 0o600.
    let pool = StoragePool::new(file_config(&db_path), 2).unwrap();
    assert_sqlite_artifact_modes(&db_path, 0o600);

    std::fs::set_permissions(&db_path, std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::set_permissions(&wal_path, std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::set_permissions(&shm_path, std::fs::Permissions::from_mode(0o644)).unwrap();

    let pool_count: i64 = pool
        .with_connection(|conn| {
            Ok(conn.query_row("SELECT COUNT(*) FROM test_data", [], |r| r.get(0))?)
        })
        .unwrap();
    assert_eq!(pool_count, 2);

    assert_sqlite_artifact_modes(&db_path, 0o600);
}

#[test]
fn in_memory_storage_pool_clamps_to_single_shared_database() {
    let config = StorageConfig {
        db_path: ":memory:".to_string(),
        storage_mode: StorageMode::Local,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    };
    let pool = StoragePool::new(config, 4).unwrap();
    pool.with_connection(|conn| {
        conn.execute(
            "INSERT INTO memories (content, memory_type, workspace) VALUES ('hello', 'note', 'default')",
            [],
        )?;
        Ok(())
    })
    .unwrap();

    for _ in 0..5 {
        let count: i64 = pool
            .with_connection(|conn| {
                Ok(conn.query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))?)
            })
            .unwrap();
        assert_eq!(
            count, 1,
            "all round-robin calls must observe the single migrated in-memory DB"
        );
    }
}

#[test]
fn in_memory_compact_runs_vacuum_and_bare_relative_disk_probe_checks_current_dir() {
    let storage = Storage::open_in_memory().unwrap();
    let report = storage.compact(true).unwrap();
    assert!(report.applied);
    assert!(
        report.vacuum_safe,
        ":memory: compaction must treat VACUUM as safe"
    );
    assert_eq!(report.free_space_bytes, -1);
    let vacuum_op = report
        .operations
        .iter()
        .find(|op| op.name == "vacuum")
        .expect("vacuum operation");
    assert!(vacuum_op.applied);

    assert_eq!(available_disk_bytes(":memory:"), None);
    #[cfg(unix)]
    assert!(
        available_disk_bytes("nonexistent_bare_relative_engram.db").is_some(),
        "bare relative filename must stat '.' instead of failing on non-existent file"
    );
}
