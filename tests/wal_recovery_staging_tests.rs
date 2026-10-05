//! C2: recovery must preserve a pre-existing target (and the source/base)
//! when replay fails late — integrity-check failure or I/O failure after
//! frames were written. Replay goes to a caller-owned staging file in the
//! target's directory and only an fsynced, verified file replaces the target.
//!
//! Injected late *I/O* failures are covered by unit tests in
//! `src/sync/wal_recovery_staging.rs` (they need a failing page writer).

use std::fs;
use std::path::{Path, PathBuf};

use engram::sync::wal_replication::{
    compute_wal_checksum, wal_frame_checksum, RecoveryOptions, RecoveryReport, WalFrame, WalHeader,
    WalRecoveryEngine, WalReplicationError, WAL_MAGIC_BE,
};
use rusqlite::Connection;

const SENTINEL: &[u8] = b"SENTINEL-TARGET-MUST-SURVIVE";
const STAGING_MARKER: &str = ".engram-replay-";

fn write_sentinel(path: &Path) {
    fs::write(path, SENTINEL).expect("write sentinel target");
}

fn assert_sentinel(path: &Path) {
    let bytes = fs::read(path).expect("read sentinel target");
    assert_eq!(bytes, SENTINEL, "pre-existing target must be untouched");
}

fn assert_no_staging_leftovers(dir: &Path) {
    let leftovers: Vec<String> = fs::read_dir(dir)
        .expect("read dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|name| name.contains(STAGING_MARKER))
        .collect();
    assert!(leftovers.is_empty(), "staging files left: {leftovers:?}");
}

/// Late failure from the staged integrity check: SQLite rejects page 1.
fn assert_not_a_database(result: Result<RecoveryReport, WalReplicationError>, case: &str) {
    match result {
        Err(WalReplicationError::Sqlite(rusqlite::Error::SqliteFailure(e, _)))
            if e.code == rusqlite::ErrorCode::NotADatabase => {}
        other => panic!("{case}: expected SQLITE_NOTADB from integrity check, got {other:?}"),
    }
}

/// A closed, rollback-journal SQLite database with one table.
fn closed_source_db(dir: &Path) -> PathBuf {
    let db = dir.join("source.db");
    let conn = Connection::open(&db).expect("open source");
    conn.execute_batch(
        "PRAGMA journal_mode=DELETE;
         CREATE TABLE t (id INTEGER PRIMARY KEY, body TEXT);
         INSERT INTO t (body) VALUES ('one'), ('two');",
    )
    .expect("init source");
    drop(conn);
    db
}

fn page_size_of(db: &Path) -> u32 {
    let conn = Connection::open(db).expect("open");
    conn.query_row("PRAGMA page_size", [], |r| r.get::<_, u32>(0))
        .expect("page_size")
}

/// Garbage page 1 as a single commit frame, with a SQLite-valid chain
/// (magic 0x377f0682 => little-endian checksum words).
fn garbage_commit_frame(page_size: u32, seed: (u32, u32)) -> WalFrame {
    let data = vec![0xAB; page_size as usize];
    let (c1, c2) = wal_frame_checksum(1, 1, &data, true, seed);
    WalFrame {
        frame_index: 1,
        page_number: 1,
        db_size_pages: 1,
        salt1: 7,
        salt2: 9,
        checksum1: c1,
        checksum2: c2,
        data,
    }
}

/// Raw WAL file whose header and frame chain verify under SQLite semantics,
/// but whose only frame overwrites page 1 with garbage.
fn write_corrupting_wal(path: &Path, page_size: u32) {
    let mut header = WalHeader::new(page_size, 1, 7, 9).serialize();
    assert_eq!(&header[0..4], &WAL_MAGIC_BE.to_be_bytes());
    let (h1, h2) = compute_wal_checksum(&header[0..24], true, (0, 0));
    header[24..28].copy_from_slice(&h1.to_be_bytes());
    header[28..32].copy_from_slice(&h2.to_be_bytes());
    let frame = garbage_commit_frame(page_size, (h1, h2));
    let mut bytes = header.to_vec();
    bytes.extend_from_slice(&frame.serialize(false));
    fs::write(path, bytes).expect("write crafted wal");
}

#[test]
fn test_recovery_preserves_target_on_late_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);

    // Frames are structurally valid; integrity_check fails after writing.
    let frames = vec![garbage_commit_frame(4096, (1, 2))];
    let result =
        WalRecoveryEngine::replay_frames_to_db(&target, 4096, &frames, &RecoveryOptions::default());
    assert_not_a_database(result, "replay with garbage page 1");
    assert_sentinel(&target);
    assert_no_staging_leftovers(dir.path());
}

#[test]
fn test_recover_from_packs_late_failure_preserves_target_and_base() {
    use engram::sync::wal_replication::{WalDelta, WalDeltaPack};
    let dir = tempfile::tempdir().expect("tempdir");
    let base = closed_source_db(dir.path());
    let base_before = fs::read(&base).expect("read base");
    let page_size = page_size_of(&base);
    let target = dir.path().join("target.db");
    write_sentinel(&target);

    let header = WalHeader::new(page_size, 1, 7, 9);
    let seed = (header.checksum1, header.checksum2);
    let frame = garbage_commit_frame(page_size, seed);
    let delta = WalDelta {
        header,
        start_frame: 1,
        end_frame: 1,
        total_wal_frames: 1,
        checkpoint_seq: 1,
        frames: vec![frame],
        chain_seed: Some(seed),
    };
    let pack = WalDeltaPack::pack(&delta, true, None).expect("pack");
    let result = WalRecoveryEngine::recover_from_delta_packs(
        Some(&base),
        &target,
        &[pack],
        &RecoveryOptions::default(),
    );
    assert_not_a_database(result, "pack recovery with garbage page 1");
    assert_sentinel(&target);
    assert_eq!(fs::read(&base).expect("read base"), base_before);
    assert_no_staging_leftovers(dir.path());
}

#[test]
fn test_pitr_late_failure_preserves_target_and_source() {
    let dir = tempfile::tempdir().expect("tempdir");
    let source = closed_source_db(dir.path());
    let source_before = fs::read(&source).expect("read source");
    let wal = dir.path().join("crafted.db-wal");
    write_corrupting_wal(&wal, page_size_of(&source));
    let wal_before = fs::read(&wal).expect("read wal");
    let target = dir.path().join("target.db");
    write_sentinel(&target);

    let result = WalRecoveryEngine::point_in_time_recovery(
        &source,
        &wal,
        &target,
        &RecoveryOptions::default(),
    );
    assert_not_a_database(result, "PITR with garbage page 1");
    assert_sentinel(&target);
    assert_eq!(fs::read(&source).expect("read source"), source_before);
    assert_eq!(fs::read(&wal).expect("read wal"), wal_before);
    assert_no_staging_leftovers(dir.path());
}

#[test]
fn test_pitr_without_wal_verifies_before_replacing_target() {
    let dir = tempfile::tempdir().expect("tempdir");
    let source = dir.path().join("corrupt-source.db");
    fs::write(&source, vec![0xCD; 4096]).expect("write corrupt source");
    let target = dir.path().join("target.db");
    write_sentinel(&target);

    let result = WalRecoveryEngine::point_in_time_recovery(
        &source,
        &dir.path().join("missing.db-wal"),
        &target,
        &RecoveryOptions::default(),
    );
    assert_not_a_database(result, "no-WAL PITR of corrupt source");
    assert_sentinel(&target);
    assert_no_staging_leftovers(dir.path());
}

#[test]
fn test_pitr_without_wal_replaces_target_on_success() {
    let dir = tempfile::tempdir().expect("tempdir");
    let source = closed_source_db(dir.path());
    let source_before = fs::read(&source).expect("read source");
    let target = dir.path().join("target.db");
    write_sentinel(&target);

    let report = WalRecoveryEngine::point_in_time_recovery(
        &source,
        &dir.path().join("missing.db-wal"),
        &target,
        &RecoveryOptions::default(),
    )
    .expect("no-WAL recovery");
    assert_eq!(report.integrity_check, "ok");
    assert_eq!(fs::read(&target).expect("read target"), source_before);
    assert_eq!(fs::read(&source).expect("read source"), source_before);
    assert_no_staging_leftovers(dir.path());
}

#[test]
fn test_recovery_refuses_target_with_hot_wal_or_journal() {
    let dir = tempfile::tempdir().expect("tempdir");
    let source = closed_source_db(dir.path());
    for suffix in ["-wal", "-journal"] {
        let target = dir.path().join(format!("target{suffix}-case.db"));
        write_sentinel(&target);
        let side = PathBuf::from(format!("{}{suffix}", target.display()));
        fs::write(&side, b"live side file").expect("write side file");

        let result = WalRecoveryEngine::point_in_time_recovery(
            &source,
            &dir.path().join("missing.db-wal"),
            &target,
            &RecoveryOptions::default(),
        );
        assert!(
            matches!(&result, Err(WalReplicationError::RecoveryError(msg)) if msg.contains(suffix)),
            "{suffix}: hot side file must block replace, got {result:?}"
        );
        assert_sentinel(&target);
        assert_eq!(fs::read(&side).expect("side"), b"live side file");
    }
    assert_no_staging_leftovers(dir.path());
}

#[test]
fn test_successful_replay_leaves_no_staging_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let source = dir.path().join("live.db");
    let conn = Connection::open(&source).expect("open live");
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA wal_autocheckpoint=0;
         CREATE TABLE t (id INTEGER PRIMARY KEY, body TEXT);
         INSERT INTO t (body) VALUES ('one');",
    )
    .expect("init live");
    let wal = PathBuf::from(format!("{}-wal", source.display()));
    let target = dir.path().join("target.db");
    write_sentinel(&target);

    let report = WalRecoveryEngine::point_in_time_recovery(
        &source,
        &wal,
        &target,
        &RecoveryOptions::default(),
    )
    .expect("PITR");
    assert_eq!(report.integrity_check, "ok");
    let restored = Connection::open(&target).expect("open target");
    let count: i64 = restored
        .query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0))
        .expect("count");
    assert_eq!(count, 1);
    drop(restored);
    drop(conn);
    assert_no_staging_leftovers(dir.path());
}

// ── Review fix 1: replaced target keeps owner-only permissions ──────────────

#[cfg(unix)]
fn mode_of(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).expect("metadata").permissions().mode() & 0o777
}

#[cfg(unix)]
#[test]
fn test_pitr_replaced_target_is_owner_only() {
    let dir = tempfile::tempdir().expect("tempdir");
    let source = closed_source_db(dir.path());
    let target = dir.path().join("target.db");
    WalRecoveryEngine::point_in_time_recovery(
        &source,
        &dir.path().join("missing.db-wal"),
        &target,
        &RecoveryOptions::default(),
    )
    .expect("PITR");
    assert_eq!(mode_of(&target), 0o600, "recovered DB must be 0600");
}

#[cfg(unix)]
#[test]
fn test_pack_recovery_replaced_target_is_owner_only() {
    use engram::sync::wal_replication::{WalDelta, WalDeltaPack};
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);
    let header = WalHeader::new(512, 1, 7, 9);
    let seed = (header.checksum1, header.checksum2);
    let mut frame = garbage_commit_frame(512, seed);
    frame.data = vec![0; 512];
    let (c1, c2) = wal_frame_checksum(1, 1, &frame.data, true, seed);
    frame.checksum1 = c1;
    frame.checksum2 = c2;
    let delta = WalDelta {
        header,
        start_frame: 1,
        end_frame: 1,
        total_wal_frames: 1,
        checkpoint_seq: 1,
        frames: vec![frame],
        chain_seed: Some(seed),
    };
    let pack = WalDeltaPack::pack(&delta, false, None).expect("pack");
    WalRecoveryEngine::recover_from_delta_packs(
        None,
        &target,
        &[pack],
        &RecoveryOptions {
            verify_integrity: false,
            ..Default::default()
        },
    )
    .expect("pack recovery");
    assert_eq!(mode_of(&target), 0o600, "recovered DB must be 0600");
}
