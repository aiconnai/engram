//! Option B viability: pinned parent-directory fd + unix-VFS syscall shim.
//!
//! Each test emulates the attacker winning the race deterministically: the
//! namespace is swapped AFTER Engram resolved/checked the path (here: after
//! `shim::bind`) and BEFORE SQLite opens the files.

use c4spike::shim;
use c4spike::util::*;
use std::path::Path;

const PEER: &str = env!("CARGO_BIN_EXE_peer");

/// Rename the real directory away and put an attacker directory (with an
/// attacker database of the same name) at the original path. No symlinks:
/// SQLITE_OPEN_NOFOLLOW cannot see this. The attacker database is written
/// by a SEPARATE process: inside this process the shim redirects every
/// opener of the bound pathname (any VFS sharing the unix syscall table) to
/// the pinned directory -- itself a finding recorded in the ADR.
fn rename_swap_parent(dir: &Path, db_name: &str) {
    let parked = dir.with_extension("orig");
    std::fs::rename(dir, &parked).expect("park real dir");
    mkdir_private(dir);
    let db = dir.join(db_name);
    assert_eq!(peer(PEER, &["make", db.to_str().unwrap(), "attacker"]), "OK");
}

#[test]
fn shim_open_reads_original_after_parent_directory_rename_swap() {
    let dir = fresh_dir("swap").join("A");
    mkdir_private(&dir);
    let db = dir.join("engram.db");
    make_db(&db, "original", "wal");

    let bound = shim::bind(&db).expect("bind");
    rename_swap_parent(&dir, "engram.db");

    let c = shim::open(&bound, flags()).expect("open through shim");
    assert_eq!(sentinel(&c), "original", "opened the attacker's database");
}

#[test]
fn shim_wal_sidecars_stay_in_pinned_directory_after_swap() {
    let dir = fresh_dir("sidecar").join("A");
    mkdir_private(&dir);
    let db = dir.join("engram.db");
    make_db(&db, "original", "wal");

    let bound = shim::bind(&db).expect("bind");
    rename_swap_parent(&dir, "engram.db");
    let parked = dir.with_extension("orig");

    let c = shim::open(&bound, flags()).expect("open");
    c.execute_batch("PRAGMA journal_mode=WAL; INSERT INTO t(v) VALUES ('w1');")
        .expect("write");
    assert!(parked.join("engram.db-wal").exists(), "WAL not in pinned dir");
    assert!(parked.join("engram.db-shm").exists(), "SHM not in pinned dir");
    // The attacker directory's own (closed) database must be untouched: its
    // WAL was checkpointed+removed when make_db closed it.
    assert!(
        !dir.join("engram.db-wal").exists(),
        "SQLite wrote a WAL into the attacker directory"
    );
    drop(c);
    // Last close checkpoints and removes the WAL in the pinned directory.
    assert!(!parked.join("engram.db-wal").exists(), "WAL not cleaned up");
    let check = rusqlite::Connection::open(parked.join("engram.db")).unwrap();
    assert_eq!(count(&check), 2);
}

#[test]
fn shim_refuses_symlinked_main_file() {
    let dir = fresh_dir("symmain");
    let db = dir.join("engram.db");
    make_db(&db, "original", "wal");
    let bound = shim::bind(&db).expect("bind");

    let other = dir.join("other.db");
    make_db(&other, "attacker", "wal");
    std::fs::remove_file(&db).unwrap();
    std::os::unix::fs::symlink(&other, &db).unwrap();

    assert!(shim::open(&bound, flags())
        .and_then(|c| c.query_row("SELECT 1 FROM t", [], |_| Ok(())))
        .is_err());
}

#[test]
fn shim_refuses_regular_file_swapped_over_main() {
    let dir = fresh_dir("inode");
    let db = dir.join("engram.db");
    make_db(&db, "original", "wal");
    let bound = shim::bind(&db).expect("bind");

    // Same directory, regular-file rename swap (no symlink anywhere).
    let evil = dir.join("evil.db");
    make_db(&evil, "attacker", "wal");
    std::fs::rename(&evil, &db).unwrap();

    let r = shim::open(&bound, flags()).and_then(|c| {
        c.query_row("SELECT v FROM t LIMIT 1", [], |r| r.get::<_, String>(0))
    });
    assert!(r.is_err(), "opened a swapped-in main file: {r:?}");
}

#[test]
fn shim_refuses_directory_writable_by_others() {
    let dir = fresh_dir("perm").join("shared");
    mkdir_private(&dir);
    chmod(&dir, 0o777);
    let r = shim::bind(&dir.join("engram.db"));
    assert!(r.is_err(), "bound a database in a world-writable directory");
}

#[test]
fn shim_pool_and_stock_peer_process_share_wal_locks_and_checkpoint() {
    let dir = fresh_dir("pool");
    let db = dir.join("engram.db");
    make_db(&db, "original", "wal");
    let dbs = db.to_str().unwrap();

    let bound = shim::bind(&db).expect("bind");
    let c1 = shim::open(&bound, flags()).expect("c1");
    let c2 = shim::open(&bound, flags()).expect("c2");
    for c in [&c1, &c2] {
        c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")
            .unwrap();
    }

    // Cross-connection and cross-process visibility.
    c1.execute("INSERT INTO t(v) VALUES ('c1')", []).unwrap();
    assert_eq!(count(&c2), 2);
    assert_eq!(peer(PEER, &["insert", dbs, "peer"]), "OK");
    assert_eq!(count(&c1), 3);
    assert_eq!(peer(PEER, &["count", dbs]), "3");

    // WAL write lock taken through the shim is seen by a stock-VFS process.
    c1.execute_batch("BEGIN IMMEDIATE").unwrap();
    assert_eq!(peer(PEER, &["try-immediate", dbs]), "BUSY");
    c1.execute_batch("COMMIT").unwrap();
    assert_eq!(peer(PEER, &["try-immediate", dbs]), "OK");

    // Checkpoint through the shim truncates the shared WAL.
    c2.query_row("PRAGMA wal_checkpoint(TRUNCATE)", [], |_| Ok(()))
        .unwrap();
    assert_eq!(std::fs::metadata(dir.join("engram.db-wal")).unwrap().len(), 0);

    drop(c1);
    drop(c2);
    assert!(!dir.join("engram.db-wal").exists(), "WAL left behind");
    assert_eq!(peer(PEER, &["count", dbs]), "3");
}

#[test]
fn shim_new_database_is_created_owner_only_in_pinned_dir() {
    use std::os::unix::fs::PermissionsExt;
    let dir = fresh_dir("create").join("A");
    mkdir_private(&dir);
    let db = dir.join("engram.db");
    let bound = shim::bind(&db).expect("bind");
    rename_swap_parent(&dir, "engram.db");
    let parked = dir.with_extension("orig");

    let c = shim::open(&bound, flags()).expect("open new");
    c.execute_batch(
        "PRAGMA journal_mode=WAL; CREATE TABLE t(v TEXT NOT NULL); \
         INSERT INTO t(v) VALUES ('fresh');",
    )
    .unwrap();
    let mode = std::fs::metadata(parked.join("engram.db"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o600);
    let wal_mode = std::fs::metadata(parked.join("engram.db-wal"))
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(wal_mode, 0o600);
    drop(c);
    let attacker = dir.join("engram.db");
    assert_eq!(peer(PEER, &["sentinel", attacker.to_str().unwrap()]), "attacker");
}

#[test]
fn in_process_stock_openers_of_the_bound_pathname_are_redirected_too() {
    let dir = fresh_dir("redirect").join("A");
    mkdir_private(&dir);
    let db = dir.join("engram.db");
    make_db(&db, "original", "wal");
    let _bound = shim::bind(&db).expect("bind");
    rename_swap_parent(&dir, "engram.db");
    // Plain rusqlite open (default "unix" VFS) of the same pathname, in this
    // process: the process-global syscall shim still sends it to the pinned
    // directory, while a separate process sees the attacker's file.
    let c = rusqlite::Connection::open_with_flags(&db, flags()).unwrap();
    assert_eq!(sentinel(&c), "original");
    assert_eq!(peer(PEER, &["sentinel", db.to_str().unwrap()]), "attacker");
}

#[test]
fn unbound_databases_keep_the_stock_path_behaviour() {
    let dir = fresh_dir("unbound");
    let db = dir.join("plain.db");
    make_db(&db, "plain", "wal");
    // Force the shim to be installed in this process first.
    let other = dir.join("bound.db");
    make_db(&other, "bound", "wal");
    let _b = shim::bind(&other).expect("bind");

    let c = rusqlite::Connection::open_with_flags(&db, flags()).expect("stock open");
    c.execute_batch("PRAGMA journal_mode=WAL; INSERT INTO t(v) VALUES ('x');")
        .unwrap();
    assert_eq!(count(&c), 2);
}

/// Engram's local-mode pragmas (src/storage/connection.rs configure_pragmas).
fn engram_local_pragmas(c: &rusqlite::Connection) {
    c.execute_batch(
        "PRAGMA busy_timeout=30000; PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; \
         PRAGMA wal_autocheckpoint=1000; PRAGMA cache_size=-64000; PRAGMA temp_store=MEMORY; \
         PRAGMA mmap_size=268435456; PRAGMA foreign_keys=ON;",
    )
    .unwrap();
}

#[test]
fn shim_concurrent_pool_threads_and_peer_processes_keep_integrity() {
    let dir = fresh_dir("stress");
    let db = dir.join("engram.db");
    make_db(&db, "original", "wal");
    let dbs = db.to_str().unwrap().to_string();
    let bound = std::sync::Arc::new(shim::bind(&db).expect("bind"));

    const THREADS: usize = 4;
    const PER_THREAD: usize = 150;
    const PEERS: usize = 2;
    const PER_PEER: usize = 40;
    let mut handles = Vec::new();
    for t in 0..THREADS {
        let bound = bound.clone();
        handles.push(std::thread::spawn(move || {
            let c = shim::open(&bound, flags()).unwrap();
            engram_local_pragmas(&c);
            for i in 0..PER_THREAD {
                c.execute("INSERT INTO t(v) VALUES (?1)", [format!("t{t}-{i}")])
                    .unwrap();
                if i % 50 == 0 {
                    c.query_row("PRAGMA wal_checkpoint(PASSIVE)", [], |_| Ok(()))
                        .unwrap();
                }
            }
        }));
    }
    for p in 0..PEERS {
        let dbs = dbs.clone();
        handles.push(std::thread::spawn(move || {
            for i in 0..PER_PEER {
                assert_eq!(peer(PEER, &["insert", &dbs, &format!("p{p}-{i}")]), "OK");
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    let c = shim::open(&bound, flags()).unwrap();
    let ok: String = c.query_row("PRAGMA integrity_check", [], |r| r.get(0)).unwrap();
    assert_eq!(ok, "ok");
    assert_eq!(count(&c) as usize, 1 + THREADS * PER_THREAD + PEERS * PER_PEER);
}

#[test]
fn shim_rollback_journal_mode_keeps_journal_in_pinned_dir() {
    // Engram CloudSafe mode uses journal_mode=DELETE + synchronous=FULL.
    let dir = fresh_dir("journal").join("A");
    mkdir_private(&dir);
    let db = dir.join("engram.db");
    make_db(&db, "original", "delete");
    let bound = shim::bind(&db).expect("bind");
    rename_swap_parent(&dir, "engram.db");
    let parked = dir.with_extension("orig");

    let c = shim::open(&bound, flags()).unwrap();
    c.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL;")
        .unwrap();
    c.execute_batch("BEGIN IMMEDIATE; INSERT INTO t(v) VALUES ('j1');")
        .unwrap();
    assert!(parked.join("engram.db-journal").exists(), "journal not in pinned dir");
    assert!(!dir.join("engram.db-journal").exists());
    c.execute_batch("COMMIT").unwrap();
    assert!(!parked.join("engram.db-journal").exists(), "journal not deleted");
    assert_eq!(count(&c), 2);
    assert_eq!(sentinel(&c), "original");
}

#[test]
fn vacuum_into_an_unbound_target_works_from_a_pinned_connection() {
    // Engram's snapshot_copy / vacuum_into / replication recovery / DuckDB
    // graph tools all run `VACUUM INTO <other path>` on the live connection;
    // SQLite opens the target through the SAME VFS.
    let dir = fresh_dir("vacuum");
    let db = dir.join("engram.db");
    make_db(&db, "original", "wal");
    let bound = shim::bind(&db).expect("bind");
    let c = shim::open(&bound, flags()).unwrap();
    let target = dir.join("snapshot.db");
    c.execute("VACUUM INTO ?1", [target.to_str().unwrap()])
        .expect("VACUUM INTO through c4-pinned");
    let snap = rusqlite::Connection::open(&target).unwrap();
    assert_eq!(sentinel(&snap), "original");
}
