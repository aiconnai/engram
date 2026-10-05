//! Characterization of TODAY's Engram open path (stock "unix" VFS, pathname,
//! SQLITE_OPEN_NOFOLLOW, canonical parent): which races it does and does not
//! close. These tests assert the CURRENT behaviour, including the residual.

use c4spike::util::*;
use rusqlite::{ffi, Connection};
use std::ffi::{c_char, c_int, CStr, CString};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

fn canonical_open_path(db: &Path) -> PathBuf {
    // Mirrors src/storage/connection.rs::database_open_path.
    db.parent()
        .unwrap()
        .canonicalize()
        .unwrap()
        .join(db.file_name().unwrap())
}

#[test]
fn stock_open_follows_a_directory_rename_swap_after_engram_checked_the_path() {
    let dir = fresh_dir("stock-rename").join("A");
    mkdir_private(&dir);
    let db = dir.join("engram.db");
    make_db(&db, "original", "wal");

    // Engram's pre-open checks happen here (lstat: regular file, not symlink).
    let open_path = canonical_open_path(&db);
    assert!(std::fs::symlink_metadata(&open_path).unwrap().is_file());

    // Attacker wins the race: rename-swap the parent (no symlink involved).
    std::fs::rename(&dir, dir.with_extension("orig")).unwrap();
    mkdir_private(&dir);
    make_db(&dir.join("engram.db"), "attacker", "wal");

    let c = Connection::open_with_flags(&open_path, flags()).unwrap();
    assert_eq!(sentinel(&c), "attacker", "residual: pathname open is not bound");
}

#[test]
fn stock_nofollow_rejects_a_symlinked_parent_seen_by_full_pathname() {
    let root = fresh_dir("stock-symparent");
    let dir = root.join("A");
    mkdir_private(&dir);
    make_db(&dir.join("engram.db"), "original", "wal");
    let open_path = canonical_open_path(&dir.join("engram.db"));

    let evil = root.join("evil");
    mkdir_private(&evil);
    make_db(&evil.join("engram.db"), "attacker", "wal");
    std::fs::rename(&dir, root.join("A.orig")).unwrap();
    std::os::unix::fs::symlink(&evil, &dir).unwrap();

    let err = Connection::open_with_flags(&open_path, flags())
        .and_then(|c| c.query_row("SELECT 1 FROM t", [], |_| Ok(())))
        .expect_err("NOFOLLOW must refuse a symlinked ancestor");
    // rusqlite reports only the primary code for a failed open (the handle
    // carrying SQLITE_CANTOPEN_SYMLINK is already closed).
    let code = match err {
        rusqlite::Error::SqliteFailure(e, _) => e.code,
        other => panic!("unexpected error {other:?}"),
    };
    assert_eq!(code, rusqlite::ErrorCode::CannotOpen);
    // Control: the same open WITHOUT NOFOLLOW follows the link, so the
    // refusal above is NOFOLLOW's doing.
    let plain = rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let c = Connection::open_with_flags(&open_path, plain).unwrap();
    assert_eq!(sentinel(&c), "attacker");
}

// --- Window between unixFullPathname (lstat walk) and open(2) -------------

type OpenFn = unsafe extern "C" fn(*const c_char, c_int, c_int) -> c_int;
static REAL_OPEN: Mutex<Option<OpenFn>> = Mutex::new(None);
/// (path that triggers, swap action) — runs once, right before the real open.
static ARMED: Mutex<Option<(CString, Box<dyn FnOnce() + Send>)>> = Mutex::new(None);

unsafe extern "C" fn hooked_open(path: *const c_char, flags: c_int, mode: c_int) -> c_int {
    let hit = {
        let mut armed = ARMED.lock().unwrap();
        let matches = armed
            .as_ref()
            .is_some_and(|(p, _)| unsafe { CStr::from_ptr(path) } == p.as_c_str());
        if matches {
            armed.take().map(|(_, f)| f)
        } else {
            None
        }
    };
    if let Some(swap) = hit {
        swap();
    }
    let real = REAL_OPEN.lock().unwrap().expect("real open");
    unsafe { real(path, flags, mode) }
}

fn install_open_hook() {
    let mut real = REAL_OPEN.lock().unwrap();
    if real.is_some() {
        return;
    }
    unsafe {
        let unix = ffi::sqlite3_vfs_find(c"unix".as_ptr());
        assert!(!unix.is_null());
        let get = (*unix).xGetSystemCall.unwrap();
        let set = (*unix).xSetSystemCall.unwrap();
        let cur = get(unix, c"open".as_ptr()).expect("open syscall");
        *real = Some(std::mem::transmute::<unsafe extern "C" fn(), OpenFn>(cur));
        let rc = set(
            unix,
            c"open".as_ptr(),
            Some(std::mem::transmute::<OpenFn, unsafe extern "C" fn()>(
                hooked_open,
            )),
        );
        assert_eq!(rc, ffi::SQLITE_OK);
    }
}

#[test]
fn stock_nofollow_does_not_close_the_window_before_open() {
    install_open_hook();
    let root = fresh_dir("stock-window");
    let dir = root.join("A");
    mkdir_private(&dir);
    make_db(&dir.join("engram.db"), "original", "wal");
    let open_path = canonical_open_path(&dir.join("engram.db"));

    let evil = root.join("evil");
    mkdir_private(&evil);
    make_db(&evil.join("engram.db"), "attacker", "wal");

    // Swap the parent for a SYMLINK after SQLite's lstat walk passed (it saw
    // no symlink) and immediately before open(2) of the main file.
    let (d, e, parked) = (dir.clone(), evil.clone(), root.join("A.orig"));
    *ARMED.lock().unwrap() = Some((
        CString::new(open_path.to_str().unwrap()).unwrap(),
        Box::new(move || {
            std::fs::rename(&d, &parked).unwrap();
            std::os::unix::fs::symlink(&e, &d).unwrap();
        }),
    ));

    let c = Connection::open_with_flags(&open_path, flags()).unwrap();
    assert_eq!(
        sentinel(&c),
        "attacker",
        "residual: O_NOFOLLOW guards only the last component"
    );
}
