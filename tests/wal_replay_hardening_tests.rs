//! C2 hardening tests for WAL replay: preflight limits, integrity chain and
//! target preservation.
//!
//! Boundary cases at `MAX_ALLOWED_DB_PAGES`/`u32::MAX` are exercised only
//! through the pure `preflight_replay` validator. Tests that touch the
//! filesystem inject tiny byte budgets, so no large or sparse file is ever
//! created, even if a guard regresses.

use std::fs;
use std::path::{Path, PathBuf};

use engram::sync::wal_replication::{
    preflight_replay, wal_frame_checksum, RecoveryOptions, ReplayLimits, WalDelta, WalDeltaPack,
    WalDeltaReader, WalFrame, WalHeader, WalRecoveryEngine, WalReplicationError,
    WalReplicationStreamer, DEFAULT_MAX_REPLAY_TARGET_BYTES, MAX_ALLOWED_DB_PAGES,
    WAL_FRAME_HEADER_SIZE, WAL_HEADER_SIZE,
};
use rusqlite::Connection;
use sha2::{Digest, Sha256};

const SMALL_PAGE: u32 = 512;
const SENTINEL: &[u8] = b"SENTINEL-TARGET-MUST-SURVIVE";

fn frame(frame_index: u32, page_number: u32, db_size_pages: u32, page_size: u32) -> WalFrame {
    WalFrame {
        frame_index,
        page_number,
        db_size_pages,
        salt1: 7,
        salt2: 9,
        checksum1: 0,
        checksum2: 0,
        data: vec![0xAB; page_size as usize],
    }
}

fn tiny_limits(pages: u64) -> ReplayLimits {
    ReplayLimits::with_max_target_bytes(pages * SMALL_PAGE as u64)
}

fn write_sentinel(path: &Path) {
    fs::write(path, SENTINEL).expect("write sentinel target");
}

fn assert_sentinel(path: &Path) {
    let bytes = fs::read(path).expect("read sentinel target");
    assert_eq!(bytes, SENTINEL, "pre-existing target must be untouched");
}

fn file_bytes(path: &Path) -> Vec<u8> {
    fs::read(path).expect("read file")
}

fn options_no_integrity() -> RecoveryOptions {
    RecoveryOptions {
        verify_integrity: false,
        ..Default::default()
    }
}

/// Fill SQLite-correct chained checksums (magic 0x377f0682: little-endian
/// words) starting from `seed`; returns the last running checksum.
fn seal(frames: &mut [WalFrame], seed: (u32, u32)) -> (u32, u32) {
    let mut running = seed;
    for f in frames.iter_mut() {
        running = wal_frame_checksum(f.page_number, f.db_size_pages, &f.data, true, running);
        f.checksum1 = running.0;
        f.checksum2 = running.1;
    }
    running
}

/// Build a chained pack for generation (`checkpoint_seq`, salts 7/9 + seq).
fn chained_pack(
    mut frames: Vec<WalFrame>,
    page_size: u32,
    checkpoint_seq: u32,
    seed: Option<(u32, u32)>,
) -> (WalDeltaPack, (u32, u32)) {
    let header = WalHeader::new(page_size, checkpoint_seq, 7 + checkpoint_seq, 9);
    for f in frames.iter_mut() {
        f.salt1 = header.salt1;
        f.salt2 = header.salt2;
    }
    let seed = seed.unwrap_or((header.checksum1, header.checksum2));
    let last = seal(&mut frames, seed);
    let start = frames.first().map(|f| f.frame_index).unwrap_or(0);
    let end = frames.last().map(|f| f.frame_index).unwrap_or(0);
    let delta = WalDelta {
        header,
        start_frame: start,
        end_frame: end,
        total_wal_frames: end,
        checkpoint_seq,
        frames,
        chain_seed: Some(seed),
    };
    (WalDeltaPack::pack(&delta, false, None).expect("pack"), last)
}

fn single_pack(frames: Vec<WalFrame>, page_size: u32) -> WalDeltaPack {
    chained_pack(frames, page_size, 1, None).0
}

/// Replace a pack's frames and recompute the external SHA-256, as an
/// attacker or a corrupting relay could.
fn repack_with_sha(pack: &WalDeltaPack, frames: &[WalFrame]) -> WalDeltaPack {
    let payload = serde_json::to_vec(frames).expect("serialize frames");
    let checksum_sha256 = hex::encode(Sha256::digest(&payload));
    WalDeltaPack {
        compressed: false,
        frame_count: frames.len(),
        payload,
        checksum_sha256,
        ..pack.clone()
    }
}

fn recover_packs(target: &Path, packs: &[WalDeltaPack]) -> Result<(), WalReplicationError> {
    WalRecoveryEngine::recover_from_delta_packs_with_limits(
        None,
        target,
        packs,
        &options_no_integrity(),
        &tiny_limits(64),
    )
    .map(|_| ())
}

fn commit_frames(range: std::ops::RangeInclusive<u32>) -> Vec<WalFrame> {
    let last = *range.end();
    range
        .map(|i| frame(i, 1, if i == last { 1 } else { 0 }, SMALL_PAGE))
        .collect()
}

/// Real SQLite database in WAL mode with uncheckpointed frames.
fn live_wal_db(dir: &Path) -> (PathBuf, PathBuf, Connection) {
    let db = dir.join("source.db");
    let conn = Connection::open(&db).expect("open source");
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA wal_autocheckpoint=0;
         CREATE TABLE t (id INTEGER PRIMARY KEY, body TEXT);
         INSERT INTO t (body) VALUES ('one'), ('two');",
    )
    .expect("init source");
    let wal = PathBuf::from(format!("{}-wal", db.display()));
    (db, wal, conn)
}

// ── 1. Limits contract ─────────────────────────────────────────────────────

#[test]
fn test_replay_limits_default_is_approved_64_gib() {
    assert_eq!(DEFAULT_MAX_REPLAY_TARGET_BYTES, 64 * 1024 * 1024 * 1024);
    let limits = ReplayLimits::default();
    assert_eq!(limits.max_target_bytes, DEFAULT_MAX_REPLAY_TARGET_BYTES);
    assert!(limits.validate().is_ok());
    assert!(ReplayLimits::with_max_target_bytes(1).validate().is_ok());
}

#[test]
fn test_replay_limits_refuse_zero_and_above_ceiling() {
    for bad in [0, DEFAULT_MAX_REPLAY_TARGET_BYTES + 1, u64::MAX] {
        let err = ReplayLimits::with_max_target_bytes(bad)
            .validate()
            .expect_err("budget must be refused");
        assert!(
            matches!(err, WalReplicationError::ReplayBudgetExceeded(_)),
            "budget {bad}: unexpected error {err:?}"
        );
    }
}

#[test]
fn test_preflight_refuses_invalid_limits() {
    let f = frame(1, 1, 1, SMALL_PAGE);
    let limits = ReplayLimits::with_max_target_bytes(0);
    assert!(preflight_replay(SMALL_PAGE, &[&f], &limits).is_err());
}

// ── 2. Pure preflight: page_number / db_size_pages boundaries ────────────────

#[test]
fn test_preflight_page_number_boundaries() {
    let limits = ReplayLimits::default();
    let ok = frame(1, MAX_ALLOWED_DB_PAGES, 0, SMALL_PAGE);
    let pre = preflight_replay(SMALL_PAGE, &[&ok], &limits).expect("MAX page accepted");
    assert_eq!(
        pre.max_write_end_bytes,
        MAX_ALLOWED_DB_PAGES as u64 * SMALL_PAGE as u64
    );

    for bad_page in [0, MAX_ALLOWED_DB_PAGES + 1, u32::MAX] {
        let f = frame(1, bad_page, 0, SMALL_PAGE);
        let err = preflight_replay(SMALL_PAGE, &[&f], &limits)
            .expect_err("out-of-range page_number refused");
        assert!(
            matches!(err, WalReplicationError::InvalidFrame { index: 1, .. }),
            "page {bad_page}: unexpected error {err:?}"
        );
    }
}

#[test]
fn test_preflight_db_size_pages_boundaries() {
    let limits = ReplayLimits::default();
    let ok = frame(1, 1, MAX_ALLOWED_DB_PAGES, SMALL_PAGE);
    let pre = preflight_replay(SMALL_PAGE, &[&ok], &limits).expect("MAX db_size accepted");
    assert_eq!(
        pre.max_commit_len_bytes,
        MAX_ALLOWED_DB_PAGES as u64 * SMALL_PAGE as u64
    );

    for bad_size in [MAX_ALLOWED_DB_PAGES + 1, u32::MAX] {
        let f = frame(1, 1, bad_size, SMALL_PAGE);
        let err = preflight_replay(SMALL_PAGE, &[&f], &limits)
            .expect_err("out-of-range db_size_pages refused");
        assert!(
            matches!(err, WalReplicationError::InvalidFrame { index: 1, .. }),
            "db_size {bad_size}: unexpected error {err:?}"
        );
    }
}

#[test]
fn test_preflight_page_cap_alone_is_not_enough_for_large_pages() {
    // 100M pages * 64 KiB ~= 6.55 TB: inside the page cap, outside the budget.
    let page = 65536;
    let limits = ReplayLimits::default();
    let write = frame(1, MAX_ALLOWED_DB_PAGES, 0, page);
    let err = preflight_replay(page, &[&write], &limits).expect_err("write beyond budget");
    assert!(matches!(err, WalReplicationError::ReplayBudgetExceeded(_)));

    let commit = frame(1, 1, MAX_ALLOWED_DB_PAGES, page);
    let err = preflight_replay(page, &[&commit], &limits).expect_err("commit beyond budget");
    assert!(matches!(err, WalReplicationError::ReplayBudgetExceeded(_)));
}

#[test]
fn test_preflight_tiny_budget_boundaries() {
    let limits = tiny_limits(4);
    let at_write = frame(1, 4, 0, SMALL_PAGE);
    let at_commit = frame(2, 1, 4, SMALL_PAGE);
    let pre = preflight_replay(SMALL_PAGE, &[&at_write, &at_commit], &limits)
        .expect("exactly at budget accepted");
    assert_eq!(pre.max_write_end_bytes, 4 * SMALL_PAGE as u64);
    assert_eq!(pre.max_commit_len_bytes, 4 * SMALL_PAGE as u64);
    assert_eq!(pre.total_frame_bytes, 2 * SMALL_PAGE as u64);

    let over_write = frame(1, 5, 0, SMALL_PAGE);
    assert!(matches!(
        preflight_replay(SMALL_PAGE, &[&over_write], &limits),
        Err(WalReplicationError::ReplayBudgetExceeded(_))
    ));
    let over_commit = frame(1, 1, 5, SMALL_PAGE);
    assert!(matches!(
        preflight_replay(SMALL_PAGE, &[&over_commit], &limits),
        Err(WalReplicationError::ReplayBudgetExceeded(_))
    ));
}

#[test]
fn test_preflight_total_frame_bytes_obeys_budget() {
    // Each write stays inside 2 pages, but 3 frames carry 3 pages of payload.
    let limits = tiny_limits(2);
    let frames: Vec<WalFrame> = (1..=3).map(|i| frame(i, 1, 0, SMALL_PAGE)).collect();
    let refs: Vec<&WalFrame> = frames.iter().collect();
    assert!(matches!(
        preflight_replay(SMALL_PAGE, &refs, &limits),
        Err(WalReplicationError::ReplayBudgetExceeded(_))
    ));
}

#[test]
fn test_preflight_rejects_unsupported_page_size_and_frame_size() {
    let limits = ReplayLimits::default();
    for bad_page_size in [0, 256, 1000, 4097, 131072] {
        let f = frame(1, 1, 1, SMALL_PAGE);
        assert!(
            preflight_replay(bad_page_size, &[&f], &limits).is_err(),
            "page_size {bad_page_size} must be refused"
        );
    }
    let mut short = frame(1, 1, 1, SMALL_PAGE);
    short.data.truncate(100);
    assert!(matches!(
        preflight_replay(SMALL_PAGE, &[&short], &limits),
        Err(WalReplicationError::InvalidFrame { index: 1, .. })
    ));
    let long = frame(1, 1, 1, 1024);
    assert!(matches!(
        preflight_replay(SMALL_PAGE, &[&long], &limits),
        Err(WalReplicationError::InvalidFrame { index: 1, .. })
    ));
}

// ── 3. Replay API refuses before touching the target ─────────────────────────

#[test]
fn test_replay_refuses_oversized_commit_before_touching_target() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);

    let frames = vec![frame(1, 1, 5, SMALL_PAGE)];
    let result = WalRecoveryEngine::replay_frames_to_db_with_limits(
        &target,
        SMALL_PAGE,
        &frames,
        &options_no_integrity(),
        &tiny_limits(4),
    );
    assert!(matches!(
        result,
        Err(WalReplicationError::ReplayBudgetExceeded(_))
    ));
    assert_sentinel(&target);
}

#[test]
fn test_replay_refusal_does_not_create_target_or_parent() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("not-yet").join("target.db");
    let frames = vec![frame(1, 1, 5, SMALL_PAGE)];
    let result = WalRecoveryEngine::replay_frames_to_db_with_limits(
        &target,
        SMALL_PAGE,
        &frames,
        &options_no_integrity(),
        &tiny_limits(4),
    );
    assert!(result.is_err());
    assert!(!target.exists(), "target must not be created on refusal");
    assert!(
        !dir.path().join("not-yet").exists(),
        "parent dir must not be created on refusal"
    );
}

#[test]
fn test_replay_invalid_frame_after_valid_frame_preserves_target() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);

    let valid = frame(1, 1, 0, SMALL_PAGE);
    let mut invalid = frame(2, 2, 2, SMALL_PAGE);
    invalid.data.truncate(100);
    let result = WalRecoveryEngine::replay_frames_to_db_with_limits(
        &target,
        SMALL_PAGE,
        &[valid, invalid],
        &options_no_integrity(),
        &ReplayLimits::default(),
    );
    assert!(matches!(
        result,
        Err(WalReplicationError::InvalidFrame { index: 2, .. })
    ));
    assert_sentinel(&target);
}

#[test]
fn test_replay_default_api_applies_default_limits() {
    // The legacy entry point must apply the approved limits, never "unlimited".
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);
    let mut bad = frame(1, 1, 1, SMALL_PAGE);
    bad.data.truncate(10);
    let result = WalRecoveryEngine::replay_frames_to_db(
        &target,
        SMALL_PAGE,
        &[bad],
        &options_no_integrity(),
    );
    assert!(result.is_err());
    assert_sentinel(&target);
}

#[test]
fn test_recover_from_packs_refuses_before_copying_base() {
    let dir = tempfile::tempdir().expect("tempdir");
    let base = dir.path().join("base.db");
    fs::write(&base, vec![0x11; SMALL_PAGE as usize]).expect("write base");
    let base_before = file_bytes(&base);
    let target = dir.path().join("target.db");
    write_sentinel(&target);

    let pack = single_pack(vec![frame(1, 1, 5, SMALL_PAGE)], SMALL_PAGE);
    let result = WalRecoveryEngine::recover_from_delta_packs_with_limits(
        Some(&base),
        &target,
        &[pack],
        &options_no_integrity(),
        &tiny_limits(4),
    );
    assert!(result.is_err(), "oversized pack must be refused");
    assert_sentinel(&target);
    assert_eq!(file_bytes(&base), base_before, "base must be untouched");
}

#[test]
fn test_point_in_time_recovery_validates_before_copying_source() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (source_db, source_wal, conn) = live_wal_db(dir.path());
    let target = dir.path().join("target.db");
    write_sentinel(&target);
    let source_before = file_bytes(&source_db);

    // Budget of one 4 KiB page cannot hold the multi-page WAL replay.
    let result = WalRecoveryEngine::point_in_time_recovery_with_limits(
        &source_db,
        &source_wal,
        &target,
        &options_no_integrity(),
        &ReplayLimits::with_max_target_bytes(4096),
    );
    assert!(matches!(
        result,
        Err(WalReplicationError::ReplayBudgetExceeded(_))
    ));
    assert_sentinel(&target);
    assert_eq!(file_bytes(&source_db), source_before, "source untouched");
    drop(conn);
}

// ── 4. Checksum chain, salts, ordering, page_size consistency ───────────────
// Integrity only: these checks do not authenticate the pack emitter.

#[test]
fn test_valid_chained_packs_replay_ok() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    let (p1, last) = chained_pack(commit_frames(1..=2), SMALL_PAGE, 1, None);
    let (p2, _) = chained_pack(commit_frames(3..=4), SMALL_PAGE, 1, Some(last));
    recover_packs(&target, &[p2, p1]).expect("contiguous chained packs replay");
    assert_eq!(file_bytes(&target).len(), SMALL_PAGE as usize);
}

#[test]
fn test_pack_tampered_frame_checksum_with_recomputed_sha_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);

    let (pack, _) = chained_pack(commit_frames(1..=3), SMALL_PAGE, 1, None);
    let mut frames = pack.unpack_frames().expect("unpack");
    frames[1].checksum1 ^= 1;
    let tampered = repack_with_sha(&pack, &frames);
    tampered
        .verify_checksum()
        .expect("external SHA recomputed, so it passes");

    let err = recover_packs(&target, &[tampered]).expect_err("chain break refused");
    assert!(
        matches!(err, WalReplicationError::InvalidFrame { index: 2, .. }),
        "unexpected error {err:?}"
    );
    assert_sentinel(&target);
}

#[test]
fn test_pack_tampered_page_data_with_recomputed_sha_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);

    let (pack, _) = chained_pack(commit_frames(1..=3), SMALL_PAGE, 1, None);
    let mut frames = pack.unpack_frames().expect("unpack");
    frames[2].data[10] ^= 0xFF;
    let tampered = repack_with_sha(&pack, &frames);
    assert!(recover_packs(&target, &[tampered]).is_err());
    assert_sentinel(&target);
}

#[test]
fn test_pack_without_chain_context_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);
    let (mut pack, _) = chained_pack(commit_frames(1..=2), SMALL_PAGE, 1, None);
    pack.chain = None;
    assert!(recover_packs(&target, &[pack]).is_err());
    assert_sentinel(&target);
}

#[test]
fn test_pack_wrong_chain_seed_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);
    let (pack, _) = chained_pack(commit_frames(1..=2), SMALL_PAGE, 1, Some((1, 2)));
    let mut forged = pack.clone();
    if let Some(chain) = forged.chain.as_mut() {
        chain.seed = (3, 4);
    }
    assert!(recover_packs(&target, &[forged]).is_err());
    assert_sentinel(&target);
}

#[test]
fn test_packs_page_size_mismatch_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);
    let (p1, last) = chained_pack(commit_frames(1..=1), SMALL_PAGE, 1, None);
    let p2_frames = vec![frame(2, 1, 1, 1024)];
    let (p2, _) = chained_pack(p2_frames, 1024, 1, Some(last));
    assert!(recover_packs(&target, &[p1, p2]).is_err());
    assert_sentinel(&target);
}

#[test]
fn test_pack_frame_salt_mismatch_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);
    let (pack, _) = chained_pack(commit_frames(1..=2), SMALL_PAGE, 1, None);
    let mut frames = pack.unpack_frames().expect("unpack");
    frames[1].salt2 ^= 0xFFFF;
    let tampered = repack_with_sha(&pack, &frames);
    assert!(recover_packs(&target, &[tampered]).is_err());
    assert_sentinel(&target);
}

#[test]
fn test_pack_frame_index_mismatch_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);
    let (pack, _) = chained_pack(commit_frames(1..=3), SMALL_PAGE, 1, None);
    let mut frames = pack.unpack_frames().expect("unpack");
    frames.swap(0, 1);
    let reordered = repack_with_sha(&pack, &frames);
    assert!(recover_packs(&target, &[reordered]).is_err());
    let mut lying = pack.clone();
    lying.frame_count = 99;
    assert!(recover_packs(&target, &[lying]).is_err());
    assert_sentinel(&target);
}

#[test]
fn test_packs_gap_or_duplicate_refused() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);
    let (p1, last) = chained_pack(commit_frames(1..=2), SMALL_PAGE, 1, None);
    let (gap, _) = chained_pack(commit_frames(4..=5), SMALL_PAGE, 1, Some(last));
    assert!(recover_packs(&target, &[p1.clone(), gap]).is_err());
    assert!(recover_packs(&target, &[p1.clone(), p1]).is_err());
    assert_sentinel(&target);
}

#[test]
fn test_packs_new_generation_must_restart_at_frame_one() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (g1, _) = chained_pack(commit_frames(1..=2), SMALL_PAGE, 1, None);
    let (g2_ok, _) = chained_pack(commit_frames(1..=2), SMALL_PAGE, 2, None);
    recover_packs(&dir.path().join("ok.db"), &[g1.clone(), g2_ok]).expect("new generation");

    let target = dir.path().join("target.db");
    write_sentinel(&target);
    let (g2_late, _) = chained_pack(commit_frames(3..=4), SMALL_PAGE, 2, Some((5, 6)));
    assert!(recover_packs(&target, &[g1, g2_late]).is_err());
    assert_sentinel(&target);
}

#[test]
fn test_direct_frames_must_be_strictly_ordered() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);
    let frames = vec![frame(2, 1, 0, SMALL_PAGE), frame(1, 1, 1, SMALL_PAGE)];
    let result = WalRecoveryEngine::replay_frames_to_db_with_limits(
        &target,
        SMALL_PAGE,
        &frames,
        &options_no_integrity(),
        &tiny_limits(4),
    );
    assert!(result.is_err());
    let mut mixed = vec![frame(1, 1, 0, SMALL_PAGE), frame(2, 1, 1, SMALL_PAGE)];
    mixed[1].salt1 ^= 1;
    let result = WalRecoveryEngine::replay_frames_to_db_with_limits(
        &target,
        SMALL_PAGE,
        &mixed,
        &options_no_integrity(),
        &tiny_limits(4),
    );
    assert!(result.is_err());
    assert_sentinel(&target);
}

// ── 5. Raw WAL reader follows SQLite chain semantics ─────────────────────────

fn frame_data_offset(frame_index: usize, page_size: usize) -> usize {
    WAL_HEADER_SIZE
        + (frame_index - 1) * (WAL_FRAME_HEADER_SIZE + page_size)
        + WAL_FRAME_HEADER_SIZE
}

#[test]
fn test_reader_verifies_real_sqlite_chain() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (_db, wal, conn) = live_wal_db(dir.path());
    let bytes = file_bytes(&wal);
    let delta = WalDeltaReader::extract_delta_frames_from_bytes(&bytes, 0, None).expect("extract");
    assert!(delta.frames.len() >= 2);
    assert_eq!(delta.frames.len() as u32, delta.total_wal_frames);
    assert!(delta.chain_seed.is_some(), "reader must report chain seed");
    drop(conn);
}

#[test]
fn test_reader_stops_at_first_bad_frame_checksum() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (_db, wal, conn) = live_wal_db(dir.path());
    let mut bytes = file_bytes(&wal);
    let full = WalDeltaReader::extract_delta_frames_from_bytes(&bytes, 0, None).expect("extract");
    let page = full.header.page_size as usize;
    assert!(full.frames.len() >= 2);

    // Corrupt frame 2's page data: SQLite treats frame 2 onwards as invalid.
    bytes[frame_data_offset(2, page) + 100] ^= 0xFF;
    let cut = WalDeltaReader::extract_delta_frames_from_bytes(&bytes, 0, None).expect("extract");
    assert_eq!(
        cut.frames.len(),
        1,
        "frames after a chain break are excluded"
    );
    drop(conn);
}

#[test]
fn test_reader_rejects_bad_header_checksum() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (_db, wal, conn) = live_wal_db(dir.path());
    let mut bytes = file_bytes(&wal);
    bytes[WAL_HEADER_SIZE - 1] ^= 0xFF;
    assert!(matches!(
        WalDeltaReader::extract_delta_frames_from_bytes(&bytes, 0, None),
        Err(WalReplicationError::InvalidHeader(_))
    ));
    drop(conn);
}

#[test]
fn test_streamer_packs_carry_linked_chain_context() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (db, _wal, conn) = live_wal_db(dir.path());
    let mut streamer = WalReplicationStreamer::new(&db).with_compression(true);
    let p1 = streamer.flush_delta().expect("flush 1").expect("pack 1");
    conn.execute("INSERT INTO t (body) VALUES ('three')", [])
        .expect("insert");
    let p2 = streamer.flush_delta().expect("flush 2").expect("pack 2");

    let c1 = p1.chain.expect("pack 1 chain context");
    let c2 = p2.chain.expect("pack 2 chain context");
    assert_eq!(c1.magic, c2.magic);
    let last = p1.unpack_frames().expect("unpack").pop().expect("frame");
    let decoded_le = c1.magic == engram::sync::wal_replication::WAL_MAGIC_LE;
    let stored = |v: u32| if decoded_le { v.swap_bytes() } else { v };
    assert_eq!(c2.seed, (stored(last.checksum1), stored(last.checksum2)));

    let target = dir.path().join("restored.db");
    let base = dir.path().join("empty-base.db");
    let report = WalRecoveryEngine::recover_from_delta_packs(
        Some(&base),
        &target,
        &[p1, p2],
        &RecoveryOptions::default(),
    )
    .expect("recover from streamer packs");
    assert_eq!(report.integrity_check, "ok");
    let restored = Connection::open(&target).expect("open restored");
    let count: i64 = restored
        .query_row("SELECT COUNT(*) FROM t", [], |r| r.get(0))
        .expect("count");
    assert_eq!(count, 3);
    drop(conn);
}

// ── Review fix 3: cumulative decompressed bytes obey the budget ─────────────

fn three_small_packs() -> Vec<WalDeltaPack> {
    // Each pack: one 512-byte commit frame (~2.2 KB of JSON once decoded).
    let (p1, s1) = chained_pack(commit_frames(1..=1), SMALL_PAGE, 1, None);
    let (p2, s2) = chained_pack(commit_frames(2..=2), SMALL_PAGE, 1, Some(s1));
    let (p3, _) = chained_pack(commit_frames(3..=3), SMALL_PAGE, 1, Some(s2));
    vec![p1, p2, p3]
}

#[test]
fn test_recover_refuses_cumulative_decompressed_bytes_over_budget() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    write_sentinel(&target);
    let packs = three_small_packs();
    let decoded: usize = packs.iter().map(|p| p.payload.len()).sum();
    let budget = tiny_limits(8); // 4096 bytes
    assert!(
        decoded as u64 > budget.max_target_bytes,
        "fixture exceeds budget"
    );

    // Frame bytes (3 * 512) and write extents fit; decoded payloads do not.
    let err = WalRecoveryEngine::recover_from_delta_packs_with_limits(
        None,
        &target,
        &packs,
        &options_no_integrity(),
        &budget,
    )
    .expect_err("cumulative decompressed size must be refused");
    assert!(
        matches!(err, WalReplicationError::ReplayBudgetExceeded(_)),
        "unexpected error {err:?}"
    );
    assert_sentinel(&target);
}

#[test]
fn test_recover_accepts_cumulative_decompressed_bytes_within_budget() {
    let dir = tempfile::tempdir().expect("tempdir");
    let target = dir.path().join("target.db");
    let packs = three_small_packs();
    let decoded: usize = packs.iter().map(|p| p.payload.len()).sum();
    let budget = tiny_limits(32); // 16 KiB
    assert!((decoded as u64) <= budget.max_target_bytes);
    WalRecoveryEngine::recover_from_delta_packs_with_limits(
        None,
        &target,
        &packs,
        &options_no_integrity(),
        &budget,
    )
    .expect("within budget");
}
