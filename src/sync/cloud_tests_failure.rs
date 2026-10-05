//! Interrupt, restart, retry and concurrent-writer behaviour of cloud sync (C6).
//!
//! Offline: an in-memory object store stands in for S3/R2. Delivery contract:
//! uploads replace the whole object (a repeated upload is idempotent), a
//! download either replaces the local file completely or leaves it untouched,
//! and the background worker retries a failed push a bounded number of times.
use super::*;
use crate::storage::migrations::run_migrations;
use crate::sync::worker::{
    DirtyTracker, RetryDecision, SyncWorker, MAX_BACKOFF, MAX_PUSH_ATTEMPTS,
};
use crate::sync::{get_sync_status, SyncDirection};
use parking_lot::Mutex as PlMutex;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::Instant;

fn write(path: &std::path::Path, bytes: &[u8]) {
    std::fs::write(path, bytes).expect("test file is written");
}

#[tokio::test]
async fn download_never_mutates_the_existing_file_in_place() {
    use std::io::Read;
    let store = InMemoryCloudStore::default();
    let cloud = CloudStorage::test_fixture_plaintext("bucket", "path.db", store.clone());
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.db");
    let local = dir.path().join("local.db");
    write(&source, b"remote generation two");
    cloud.upload(&source).await.unwrap();
    write(&local, b"local generation one");

    // A reader that has the old file open (as an interrupted/concurrent
    // consumer would) must keep seeing the old bytes: the file is replaced by
    // rename, never truncated and rewritten.
    let mut held_open = std::fs::File::open(&local).unwrap();
    cloud.download(&local).await.unwrap();

    let mut seen = String::new();
    held_open.read_to_string(&mut seen).unwrap();
    assert_eq!(seen, "local generation one", "file was rewritten in place");
    assert_eq!(std::fs::read(&local).unwrap(), b"remote generation two");
}

#[tokio::test]
async fn failed_download_leaves_no_temp_files_and_keeps_the_target() {
    let store = InMemoryCloudStore::default();
    let cloud = CloudStorage::test_fixture_plaintext("bucket", "path.db", store.clone());
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.db");
    write(&source, b"remote bytes");
    cloud.upload(&source).await.unwrap();

    // The target path is a directory: replacing it must fail cleanly.
    let target = dir.path().join("target");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("keep.txt"), b"keep").unwrap();

    assert!(cloud.download(&target).await.is_err());

    assert!(target.join("keep.txt").exists());
    let leftovers: Vec<_> = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
        .filter(|n| n.contains(".download-"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "temp files left behind: {leftovers:?}"
    );
}

#[tokio::test]
async fn download_refuses_a_target_with_hot_sqlite_side_files() {
    let store = InMemoryCloudStore::default();
    let cloud = CloudStorage::test_fixture_plaintext("bucket", "path.db", store.clone());
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.db");
    write(&source, b"remote bytes");
    cloud.upload(&source).await.unwrap();

    for suffix in ["-wal", "-shm", "-journal"] {
        let target = dir.path().join(format!("live{suffix}.db"));
        write(&target, b"local database");
        let side = dir.path().join(format!("live{suffix}.db{suffix}"));
        write(&side, b"hot state");

        let error = cloud.download(&target).await.unwrap_err();

        assert!(error.to_string().contains("hot"), "{suffix}: {error}");
        assert_eq!(std::fs::read(&target).unwrap(), b"local database");
    }

    // An empty -wal (clean checkpoint) is not hot.
    let clean = dir.path().join("clean.db");
    write(&clean, b"old");
    write(&dir.path().join("clean.db-wal"), b"");
    cloud.download(&clean).await.unwrap();
    assert_eq!(std::fs::read(&clean).unwrap(), b"remote bytes");
}

#[tokio::test]
async fn download_checked_refuses_the_live_database() {
    let storage_dir = tempfile::tempdir().unwrap();
    let live = storage_dir.path().join("live.db");
    let storage = crate::storage::Storage::open(crate::types::StorageConfig {
        db_path: live.to_string_lossy().into_owned(),
        storage_mode: crate::types::StorageMode::Local,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    })
    .unwrap();
    let store = InMemoryCloudStore::default();
    let cloud = CloudStorage::test_fixture_plaintext("bucket", "path.db", store);
    let source = storage_dir.path().join("source.db");
    write(&source, b"remote bytes");
    cloud.upload(&source).await.unwrap();

    let error = cloud.download_checked(&live, &storage).await.unwrap_err();

    assert!(
        error.to_string().to_lowercase().contains("active"),
        "{error}"
    );
    let other = storage_dir.path().join("elsewhere.db");
    cloud.download_checked(&other, &storage).await.unwrap();
    assert_eq!(std::fs::read(&other).unwrap(), b"remote bytes");
}

#[tokio::test]
async fn worker_pull_refuses_to_overwrite_its_own_live_database() {
    let dir = tempfile::tempdir().unwrap();
    let live = dir.path().join("live.db");
    let conn = rusqlite::Connection::open(&live).unwrap();
    run_migrations(&conn).unwrap();
    let conn = Arc::new(PlMutex::new(conn));
    let store = InMemoryCloudStore::default();
    let seed = CloudStorage::test_fixture_plaintext("bucket", "p.db", store.clone());
    let src = dir.path().join("src.db");
    write(&src, b"remote bytes");
    seed.upload(&src).await.unwrap();

    let cloud = CloudStorage::test_fixture_plaintext("bucket", "p.db", store);
    let worker = SyncWorker::start_with_cloud(live.clone(), cloud, 50, conn.clone());
    worker.sync(SyncDirection::Pull, false).await.unwrap();
    wait_until(|| get_sync_status(&conn.lock()).unwrap().last_error.is_some()).await;

    let error = get_sync_status(&conn.lock()).unwrap().last_error.unwrap();
    assert!(error.contains("live database"), "{error}");
    assert_ne!(std::fs::read(&live).unwrap(), b"remote bytes");
    worker.stop().await.unwrap();
}

#[tokio::test]
async fn repeated_upload_is_idempotent_and_the_object_stays_readable() {
    let store = InMemoryCloudStore::default();
    let cloud =
        CloudStorage::test_fixture("bucket", "path.db", provider_from_byte(5), store.clone());
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("source.db");
    let restored = dir.path().join("restored.db");
    write(&source, b"payload that is retried");

    let first = cloud.upload(&source).await.unwrap();
    let retry = cloud.upload(&source).await.unwrap();
    cloud.download(&restored).await.unwrap();

    assert_eq!(first, retry);
    assert_eq!(
        std::fs::read(&restored).unwrap(),
        b"payload that is retried"
    );
}

#[tokio::test]
async fn concurrent_writers_never_corrupt_the_remote_object() {
    let store = InMemoryCloudStore::default();
    let dir = tempfile::tempdir().unwrap();
    let a_src = dir.path().join("a.db");
    let b_src = dir.path().join("b.db");
    write(&a_src, b"writer A payload");
    write(&b_src, b"writer B payload");
    let a = CloudStorage::test_fixture("bucket", "p.db", provider_from_byte(9), store.clone());
    let b = CloudStorage::test_fixture("bucket", "p.db", provider_from_byte(9), store.clone());

    let (ra, rb) = tokio::join!(a.upload(&a_src), b.upload(&b_src));

    assert!(
        ra.is_ok() || rb.is_ok(),
        "someone must win: {ra:?} / {rb:?}"
    );
    let restored = dir.path().join("restored.db");
    a.download(&restored)
        .await
        .expect("remote stays decryptable");
    let bytes = std::fs::read(&restored).unwrap();
    assert!(
        bytes == b"writer A payload" || bytes == b"writer B payload",
        "remote object is a torn mix: {bytes:?}"
    );
}

// ── worker: restart + retry ─────────────────────────────────────────────

fn state_conn() -> Arc<PlMutex<rusqlite::Connection>> {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    run_migrations(&conn).unwrap();
    Arc::new(PlMutex::new(conn))
}

#[tokio::test]
async fn worker_start_clears_is_syncing_left_by_an_interrupted_process() {
    let conn = state_conn();
    conn.lock()
        .execute("UPDATE sync_state SET is_syncing = 1 WHERE id = 1", [])
        .unwrap();
    assert!(get_sync_status(&conn.lock()).unwrap().is_syncing);

    let dir = tempfile::tempdir().unwrap();
    let cloud =
        CloudStorage::test_fixture_plaintext("bucket", "path.db", InMemoryCloudStore::default());
    let worker = SyncWorker::start_with_cloud(dir.path().join("db"), cloud, 50, conn.clone());

    let status = get_sync_status(&conn.lock()).unwrap();
    assert!(!status.is_syncing, "stale is_syncing survived a restart");
    assert!(
        status
            .last_error
            .as_deref()
            .is_some_and(|e| e.contains("interrupted")),
        "the interruption must stay visible: {:?}",
        status.last_error
    );
    worker.stop().await.unwrap();
}

#[tokio::test]
async fn restart_records_the_interruption_even_after_an_earlier_error() {
    let conn = state_conn();
    conn.lock()
        .execute(
            "UPDATE sync_state SET is_syncing = 1, last_error = 'earlier network failure' WHERE id = 1",
            [],
        )
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let cloud =
        CloudStorage::test_fixture_plaintext("bucket", "path.db", InMemoryCloudStore::default());
    let worker = SyncWorker::start_with_cloud(dir.path().join("db"), cloud, 50, conn.clone());

    let status = get_sync_status(&conn.lock()).unwrap();
    let error = status.last_error.expect("error recorded");
    assert!(error.contains("earlier network failure"), "{error}");
    assert!(error.contains("interrupted"), "interruption lost: {error}");
    assert!(!status.is_syncing);
    worker.stop().await.unwrap();
}

#[test]
fn explicit_sync_outcome_resets_or_clears_the_retry_state() {
    let debounce = Duration::from_millis(100);
    let t0 = Instant::now();
    let mut tracker = DirtyTracker::new(debounce);
    for _ in 0..3 {
        tracker.mark_dirty(t0);
        tracker.on_failure(t0);
    }
    // A successful explicit Sync proves the remote works: backoff resets.
    tracker.on_explicit_sync(true);
    tracker.mark_dirty(t0);
    assert!(
        tracker.is_due(t0 + debounce),
        "backoff survived a successful sync"
    );

    // A failed explicit Sync drops the pending marker without scheduling a retry.
    tracker.mark_dirty(t0);
    tracker.on_explicit_sync(false);
    assert!(!tracker.is_due(t0 + Duration::from_secs(3600)));
}

#[tokio::test]
async fn failed_debounced_push_is_retried_and_clears_the_error_on_success() {
    // The remote holds an object this client may not overwrite, so the first
    // push is refused; once it is gone the retry must succeed on its own.
    let store = InMemoryCloudStore::default();
    let foreign =
        CloudStorage::test_fixture("bucket", "p.db", provider_from_byte(1), store.clone());
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("local.db");
    write(&db, b"local database bytes");
    foreign.upload(&db).await.unwrap();

    let conn = state_conn();
    let cloud = CloudStorage::test_fixture_plaintext("bucket", "p.db", store.clone());
    let worker = SyncWorker::start_with_cloud(db.clone(), cloud, 40, conn.clone());
    worker.mark_dirty().await.unwrap();

    wait_until(|| get_sync_status(&conn.lock()).unwrap().last_error.is_some()).await;
    assert!(get_sync_status(&conn.lock()).unwrap().last_sync.is_none());

    store.delete();
    wait_until(|| get_sync_status(&conn.lock()).unwrap().last_sync.is_some()).await;
    let status = get_sync_status(&conn.lock()).unwrap();
    assert!(status.last_error.is_none(), "{:?}", status.last_error);
    worker.stop().await.unwrap();
}

async fn wait_until(mut condition: impl FnMut() -> bool) {
    for _ in 0..200 {
        if condition() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("condition not reached within 5s");
}

#[test]
fn retry_policy_backs_off_exponentially_and_gives_up_after_the_cap() {
    let debounce = Duration::from_millis(100);
    let mut tracker = DirtyTracker::new(debounce);
    let t0 = Instant::now();
    tracker.mark_dirty(t0);
    assert!(!tracker.is_due(t0 + Duration::from_millis(99)));
    assert!(tracker.is_due(t0 + debounce));

    // 1st failure: next attempt after 2x debounce, measured from the failure.
    assert_eq!(tracker.on_failure(t0), RetryDecision::Retry);
    assert!(!tracker.is_due(t0 + Duration::from_millis(199)));
    assert!(tracker.is_due(t0 + Duration::from_millis(200)));
    // 2nd failure: 4x.
    assert_eq!(tracker.on_failure(t0), RetryDecision::Retry);
    assert!(!tracker.is_due(t0 + Duration::from_millis(399)));
    assert!(tracker.is_due(t0 + Duration::from_millis(400)));

    // Attempts are bounded: after MAX_PUSH_ATTEMPTS the tracker stops asking.
    let mut last = RetryDecision::Retry;
    for _ in 2..MAX_PUSH_ATTEMPTS {
        last = tracker.on_failure(t0);
    }
    assert_eq!(last, RetryDecision::GiveUp);
    assert!(
        !tracker.is_due(t0 + Duration::from_secs(3600)),
        "no retry without a new change"
    );

    // A new change re-arms it, still with the capped backoff, and success resets.
    tracker.mark_dirty(t0);
    assert!(tracker.is_due(t0 + MAX_BACKOFF));
    tracker.on_success();
    tracker.mark_dirty(t0);
    assert!(tracker.is_due(t0 + debounce));
}

#[tokio::test]
async fn explicit_sync_is_not_retried_automatically() {
    // One-shot Sync against a refused remote records the failure once and the
    // worker then stays idle (no hidden retry loop).
    let store = InMemoryCloudStore::default();
    let foreign =
        CloudStorage::test_fixture("bucket", "p.db", provider_from_byte(1), store.clone());
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("local.db");
    write(&db, b"local");
    foreign.upload(&db).await.unwrap();
    let before = store.snapshot().unwrap();

    let conn = state_conn();
    let cloud = CloudStorage::test_fixture_plaintext("bucket", "p.db", store.clone());
    let worker = SyncWorker::start_with_cloud(db, cloud, 20, conn.clone());
    worker.sync(SyncDirection::Push, false).await.unwrap();
    wait_until(|| get_sync_status(&conn.lock()).unwrap().last_error.is_some()).await;

    store.delete();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(
        get_sync_status(&conn.lock()).unwrap().last_sync.is_none(),
        "an explicit Sync must not be retried behind the caller's back"
    );
    assert!(store.snapshot().is_none());
    assert!(before.etag.is_some());
    worker.stop().await.unwrap();
}
