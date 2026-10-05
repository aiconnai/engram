//! Option C characterization: SQLite's alternative unix VFSes
//! ("unix-dotfile", "unix-none") avoid fcntl locks entirely, so a pinned
//! descriptor could never drop them -- but they do not interoperate with the
//! stock "unix" VFS every other Engram process uses.

use c4spike::util::*;
use rusqlite::{Connection, OpenFlags};

const PEER: &str = env!("CARGO_BIN_EXE_peer");

fn open_vfs(db: &std::path::Path, vfs: &str) -> Connection {
    let f = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    Connection::open_with_flags_and_vfs(db, f, vfs).expect("open alt vfs")
}

#[test]
fn dotfile_and_none_writers_are_invisible_to_a_stock_unix_peer() {
    for vfs in ["unix-dotfile", "unix-none"] {
        let dir = fresh_dir(vfs);
        let db = dir.join("engram.db");
        make_db(&db, "original", "delete");
        let c = open_vfs(&db, vfs);
        c.execute_batch("BEGIN IMMEDIATE; INSERT INTO t(v) VALUES ('w');")
            .unwrap();
        // A stock-VFS process can start a concurrent write: the lock
        // protocols do not see each other (corruption hazard).
        assert_eq!(
            peer(PEER, &["try-immediate", db.to_str().unwrap()]),
            "OK",
            "{vfs}: peer was blocked"
        );
        c.execute_batch("ROLLBACK").unwrap();
    }
}

#[test]
fn dotfile_vfs_cannot_enter_shared_memory_wal_mode() {
    let dir = fresh_dir("dotfile-wal");
    let db = dir.join("engram.db");
    make_db(&db, "original", "delete");
    let c = open_vfs(&db, "unix-dotfile");
    let mode: String = c
        .query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))
        .unwrap();
    // Without xShm* methods SQLite keeps the rollback journal unless
    // locking_mode=EXCLUSIVE (single connection) is used.
    assert_ne!(mode.to_lowercase(), "wal");
}
