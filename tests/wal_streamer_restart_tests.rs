//! WAL streamer behaviour across checkpoints, restarts and failed flushes (C6).
//!
//! Delivery contract under test: `flush_delta` advances the in-memory offset
//! when it *returns* a pack, so the caller owns delivery. A process that
//! restarts (new streamer) starts again at frame 1 and re-emits the live WAL.
//! `reset_offset(frame)` restores only the frame pointer, so it is a safe
//! resume ONLY if the WAL generation did not change while the process was down:
//! the generation identity (checkpoint sequence and the two salts) is not
//! restorable, and a restarted streamer treats the first header it sees as the
//! baseline. A caller that cannot rule out a checkpoint/reset in between must
//! restart from frame 1 and de-duplicate packs itself. This is at-least-once
//! with caller-side de-duplication, not exactly-once.

use std::path::{Path, PathBuf};

use engram::sync::wal_replication::WalReplicationStreamer;
use rusqlite::Connection;

struct TempDb {
    dir: tempfile::TempDir,
    path: PathBuf,
}

impl TempDb {
    fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("stream.db");
        Self { dir, path }
    }

    fn open(&self) -> Connection {
        let conn = Connection::open(&self.path).expect("open");
        conn.execute_batch(
            "PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;
             CREATE TABLE IF NOT EXISTS t (id INTEGER PRIMARY KEY, v TEXT);",
        )
        .expect("setup");
        conn
    }
}

fn insert_rows(conn: &Connection, n: usize, tag: &str) {
    for i in 0..n {
        conn.execute("INSERT INTO t (v) VALUES (?1)", [format!("{tag}-{i}")])
            .expect("insert");
    }
}

fn wal_len(db: &Path) -> u64 {
    std::fs::metadata(format!("{}-wal", db.display()))
        .map(|m| m.len())
        .unwrap_or(0)
}

#[test]
fn frames_written_after_a_first_checkpoint_are_not_skipped() {
    let db = TempDb::new();
    let conn = db.open();
    insert_rows(&conn, 12, "gen1");

    let mut streamer = WalReplicationStreamer::new(&db.path);
    let first = streamer.flush_delta().unwrap().expect("first pack");
    assert!(first.end_frame >= 12, "setup needs a long first generation");

    // Checkpoint + WAL reset: the next write starts a new generation whose
    // frame numbers restart at 1, with a bumped checkpoint sequence.
    conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))
        .unwrap();
    insert_rows(&conn, 3, "gen2");
    assert!(wal_len(&db.path) > 0, "second generation must exist");

    let second = streamer
        .flush_delta()
        .unwrap()
        .expect("frames of the new WAL generation must be replicated, not skipped");
    assert_eq!(second.start_frame, 1, "new generation restarts at frame 1");
    assert!(second.frame_count >= 3);
}

#[test]
fn flush_after_checkpoint_truncate_with_no_new_writes_is_quiet() {
    let db = TempDb::new();
    let conn = db.open();
    insert_rows(&conn, 4, "gen1");
    let mut streamer = WalReplicationStreamer::new(&db.path);
    streamer.flush_delta().unwrap().expect("pack");

    conn.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))
        .unwrap();

    let outcome = streamer.flush_delta();
    assert!(
        matches!(outcome, Ok(None)),
        "an empty WAL has nothing to replicate: {outcome:?}"
    );
    assert!(
        streamer.status().unwrap().last_error.is_none(),
        "a truncated WAL must not leave a replication error behind"
    );
}

#[test]
fn restarted_streamer_re_emits_from_frame_one_and_can_resume_with_reset_offset() {
    let db = TempDb::new();
    let conn = db.open();
    insert_rows(&conn, 5, "a");

    let mut before_restart = WalReplicationStreamer::new(&db.path);
    let delivered = before_restart.flush_delta().unwrap().expect("pack");
    drop(before_restart); // "interrupt"

    // Restart without persisted state: nothing remembers the delivery.
    let mut naive = WalReplicationStreamer::new(&db.path);
    let replay = naive.flush_delta().unwrap().expect("replayed pack");
    assert_eq!(replay.start_frame, 1);
    assert_eq!(replay.end_frame, delivered.end_frame);

    // Restart that persisted the last delivered frame resumes cleanly (valid
    // here because the WAL generation did not change while it was down).
    let mut resumed = WalReplicationStreamer::new(&db.path);
    resumed.reset_offset(delivered.end_frame);
    assert!(resumed.flush_delta().unwrap().is_none(), "no duplicate");
    insert_rows(&conn, 2, "b");
    let next = resumed.flush_delta().unwrap().expect("new frames only");
    assert_eq!(next.start_frame, delivered.end_frame + 1);
}

#[test]
fn failed_flush_does_not_advance_the_offset_so_retry_sends_the_same_frames() {
    let db = TempDb::new();
    let conn = db.open();
    insert_rows(&conn, 3, "x");
    let wal = format!("{}-wal", db.path.display());
    let good = std::fs::read(&wal).unwrap();

    // Corrupt the WAL header magic: the flush must fail and report it.
    let mut bad = good.clone();
    bad[0] ^= 0xFF;
    std::fs::write(&wal, &bad).unwrap();
    let mut streamer = WalReplicationStreamer::new(&db.path);
    assert!(streamer.flush_delta().is_err());
    let status = streamer.status();
    assert!(status.is_err() || status.unwrap().last_error.is_some());

    // Restore and retry: same frames from the start, offset never advanced.
    std::fs::write(&wal, &good).unwrap();
    let pack = streamer.flush_delta().unwrap().expect("retry succeeds");
    assert_eq!(pack.start_frame, 1);
    drop(conn);
    drop(db.dir);
}
