//! O1: storage warnings never print the database location.

use crate::observability::test_capture::{assert_logs_exclude, captured, install};
use crate::storage::Storage;
use crate::types::{StorageConfig, StorageMode};

#[cfg(unix)]
#[test]
fn permissive_parent_directory_warning_does_not_log_the_path() {
    use std::os::unix::fs::PermissionsExt;

    install();
    let dir = tempfile::tempdir().expect("tempdir");
    let parent = dir.path().join("db-parent-sentinel-7a8b");
    std::fs::create_dir(&parent).expect("mkdir");
    std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o755)).expect("chmod");

    let storage = Storage::open(StorageConfig {
        db_path: parent.join("m.db").to_string_lossy().to_string(),
        storage_mode: StorageMode::Local,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    })
    .expect("open still succeeds");

    // The operator still learns that the directory is too open, and what to do.
    assert!(captured().contains("Database parent directory is accessible by group or others"));
    assert_logs_exclude(&["db-parent-sentinel-7a8b"]);
    // The warning text returned to the caller (server startup) keeps the path.
    assert_eq!(storage.db_path(), parent.join("m.db").to_string_lossy());
}
