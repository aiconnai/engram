//! Invariant #27 / G1, reproduced in isolation: closing ANY extra descriptor
//! of a live SQLite file drops this process's POSIX locks on it. Every
//! "pre-opened fd" design must therefore never close its pinned fd while
//! any connection in the process may hold locks on that inode.

use c4spike::util::*;
use rusqlite::Connection;

const PEER: &str = env!("CARGO_BIN_EXE_peer");

#[test]
fn closing_an_extra_descriptor_drops_the_sqlite_reserved_lock() {
    let dir = fresh_dir("lockdrop");
    let db = dir.join("engram.db");
    make_db(&db, "original", "delete");
    let dbs = db.to_str().unwrap();

    let c = Connection::open_with_flags(&db, flags()).unwrap();
    c.execute_batch("BEGIN IMMEDIATE; INSERT INTO t(v) VALUES ('pending');")
        .unwrap();
    assert_eq!(peer(PEER, &["probe-reserved", dbs]), "HELD");
    assert_eq!(peer(PEER, &["try-immediate", dbs]), "BUSY");

    // e.g. a pinned/verification fd, a /dev/fd dup, a hardlink alias fd...
    drop(std::fs::File::open(&db).unwrap());

    assert_eq!(
        peer(PEER, &["probe-reserved", dbs]),
        "FREE",
        "expected the extra close to drop SQLite's lock"
    );
    assert_eq!(
        peer(PEER, &["try-immediate", dbs]),
        "OK",
        "another process can now start a write while we are mid-transaction"
    );
    c.execute_batch("ROLLBACK").unwrap();
}

#[test]
fn closing_a_directory_descriptor_keeps_the_sqlite_reserved_lock() {
    // The shim only ever holds the PARENT DIRECTORY fd (plus short-lived
    // dup()s of it for SQLite's directory fsync). Directory fds carry no
    // locks on the database inode.
    let dir = fresh_dir("dirfd");
    let db = dir.join("engram.db");
    make_db(&db, "original", "delete");
    let dbs = db.to_str().unwrap();

    let c = Connection::open_with_flags(&db, flags()).unwrap();
    c.execute_batch("BEGIN IMMEDIATE; INSERT INTO t(v) VALUES ('pending');")
        .unwrap();
    drop(std::fs::File::open(&dir).unwrap());
    assert_eq!(peer(PEER, &["probe-reserved", dbs]), "HELD");
    c.execute_batch("ROLLBACK").unwrap();
}
