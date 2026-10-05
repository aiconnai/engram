//! Control for the shim stress test: identical workload through the STOCK
//! unix VFS (no shim installed in this test binary).

use c4spike::util::*;

const PEER: &str = env!("CARGO_BIN_EXE_peer");

// Passthrough "open" hook: only observes (C4_TRACE) sidecar open failures.
type OpenFn = unsafe extern "C" fn(*const std::ffi::c_char, i32, i32) -> i32;
static REAL_OPEN: std::sync::OnceLock<OpenFn> = std::sync::OnceLock::new();
unsafe extern "C" fn observe_open(p: *const std::ffi::c_char, f: i32, m: i32) -> i32 {
    let fd = (REAL_OPEN.get().unwrap())(p, f, m);
    if std::env::var_os("C4_TRACE").is_some() {
        let name = std::ffi::CStr::from_ptr(p).to_string_lossy();
        if name.ends_with("-shm") || name.ends_with("-wal") {
            let e = if fd < 0 { std::io::Error::last_os_error().to_string() } else { String::new() };
            let ro = (f & libc::O_ACCMODE) == libc::O_RDONLY;
            eprintln!("stock_open {} flags={f:#x} rdonly={} -> {fd} {e}", &name[name.len() - 4..], ro as u8);
        }
    }
    fd
}
fn install_observer() {
    REAL_OPEN.get_or_init(|| unsafe {
        let unix = rusqlite::ffi::sqlite3_vfs_find(c"unix".as_ptr());
        let cur = ((*unix).xGetSystemCall.unwrap())(unix, c"open".as_ptr()).unwrap();
        let real = std::mem::transmute::<unsafe extern "C" fn(), OpenFn>(cur);
        ((*unix).xSetSystemCall.unwrap())(
            unix,
            c"open".as_ptr(),
            Some(std::mem::transmute::<OpenFn, unsafe extern "C" fn()>(observe_open)),
        );
        real
    });
}

fn engram_local_pragmas(c: &rusqlite::Connection) {
    c.execute_batch(
        "PRAGMA busy_timeout=30000; PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; \
         PRAGMA wal_autocheckpoint=1000; PRAGMA cache_size=-64000; PRAGMA temp_store=MEMORY; \
         PRAGMA mmap_size=268435456; PRAGMA foreign_keys=ON;",
    )
    .unwrap();
}

#[test]
fn stock_concurrent_pool_threads_and_peer_processes_keep_integrity() {
    install_observer();
    let dir = fresh_dir("stress-stock");
    let db = dir.join("engram.db");
    make_db(&db, "original", "wal");
    let dbs = db.to_str().unwrap().to_string();

    const THREADS: usize = 4;
    const PER_THREAD: usize = 150;
    const PEERS: usize = 2;
    const PER_PEER: usize = 40;
    let mut handles = Vec::new();
    for t in 0..THREADS {
        let db = db.clone();
        handles.push(std::thread::spawn(move || {
            let c = rusqlite::Connection::open_with_flags(&db, flags()).unwrap();
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
    let c = rusqlite::Connection::open_with_flags(&db, flags()).unwrap();
    let ok: String = c.query_row("PRAGMA integrity_check", [], |r| r.get(0)).unwrap();
    assert_eq!(ok, "ok");
    assert_eq!(count(&c) as usize, 1 + THREADS * PER_THREAD + PEERS * PER_PEER);
}
