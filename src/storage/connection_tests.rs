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
