//! Option A viability: hand SQLite's stock "unix" VFS a descriptor alias
//! (`/dev/fd/N`, `/proc/self/fd/N`) for a file Engram already opened and
//! verified. Characterises what the stock VFS does with such names.

use c4spike::util::*;
use rusqlite::{Connection, OpenFlags};
use std::os::unix::io::AsRawFd;
use std::path::Path;

/// Open + verify the real file, then swap the pathname to an attacker file
/// so any by-name re-resolution is observable. Returns the pinned file.
fn pinned_then_swapped(dir: &Path) -> std::fs::File {
    let db = dir.join("engram.db");
    make_db(&db, "original", "delete");
    let pinned = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&db)
        .unwrap();
    std::fs::rename(&db, dir.join("parked.db")).unwrap();
    make_db(&db, "attacker", "delete");
    pinned
}

#[allow(dead_code)] // used by the Linux-only tests
fn sqlite_code(e: &rusqlite::Error) -> Option<i32> {
    match e {
        rusqlite::Error::SqliteFailure(f, _) => Some(f.extended_code),
        _ => None,
    }
}

#[allow(dead_code)]
fn no_nofollow() -> OpenFlags {
    OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX
}

/// macOS/BSD fdesc: `/dev/fd/N` is not a symlink; open(2) on it dup()s N.
#[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "openbsd"))]
#[test]
fn dev_fd_alias_binds_main_file_but_sidecars_are_unusable() {
    let dir = fresh_dir("devfd");
    let pinned = pinned_then_swapped(&dir);
    let alias = format!("/dev/fd/{}", pinned.as_raw_fd());

    let c = Connection::open_with_flags(&alias, flags()).expect("alias open");
    // Main file IS descriptor-bound: we read the pinned inode, not the swap.
    assert_eq!(sentinel(&c), "original");

    // ...but every sidecar name is derived from the alias ("/dev/fd/N-journal",
    // "/dev/fd/N-wal", "/dev/fd/N-shm"), which cannot be created on devfs.
    let write = c.execute("INSERT INTO t(v) VALUES ('x')", []);
    assert!(write.is_err(), "rollback-journal write unexpectedly worked");
    let wal: String = c
        .query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))
        .unwrap_or_else(|e| format!("error: {e}"));
    assert_ne!(wal.to_lowercase(), "wal", "WAL unexpectedly enabled");
    drop(c);
    drop(pinned); // safe here only because no connection is open any more
}

/// Linux: `/proc/self/fd/N` (and `/dev/fd` -> `/proc/self/fd`) are symlinks.
#[cfg(target_os = "linux")]
#[test]
fn proc_fd_alias_is_refused_by_nofollow() {
    let dir = fresh_dir("procfd-nofollow");
    let pinned = pinned_then_swapped(&dir);
    for alias in [
        format!("/proc/self/fd/{}", pinned.as_raw_fd()),
        format!("/dev/fd/{}", pinned.as_raw_fd()),
    ] {
        let err = Connection::open_with_flags(&alias, flags())
            .and_then(|c| c.query_row("SELECT 1 FROM t", [], |_| Ok(())))
            .expect_err("NOFOLLOW must refuse the magic link");
        // rusqlite surfaces only the primary code of a failed open; the
        // without-NOFOLLOW control below shows the link itself is openable.
        assert_eq!(
            sqlite_code(&err).map(|c| c & 0xff),
            Some(rusqlite::ffi::SQLITE_CANTOPEN),
            "{alias}: {err:?}"
        );
    }
}

#[cfg(target_os = "linux")]
#[test]
fn proc_fd_alias_without_nofollow_resolves_back_to_the_mutable_pathname() {
    let dir = fresh_dir("procfd-follow");
    let pinned = pinned_then_swapped(&dir);
    let alias = format!("/proc/self/fd/{}", pinned.as_raw_fd());
    let c = Connection::open_with_flags(&alias, no_nofollow()).expect("open");
    // unixFullPathname readlink()s the magic link to "<dir>/parked.db"
    // (the name the pinned inode has NOW) and opens that by name.
    let name: String = c
        .query_row("SELECT file FROM pragma_database_list WHERE name='main'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(
        name.ends_with("/parked.db"),
        "expected resolution back to a mutable pathname, got {name}"
    );
    assert_eq!(sentinel(&c), "original");
}

#[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "openbsd"))]
#[test]
fn dev_fd_alias_full_pathname_is_kept_verbatim() {
    let dir = fresh_dir("devfd-name");
    let pinned = pinned_then_swapped(&dir);
    let alias = format!("/dev/fd/{}", pinned.as_raw_fd());
    let c = Connection::open_with_flags(&alias, no_nofollow()).expect("open");
    let name: String = c
        .query_row("SELECT file FROM pragma_database_list WHERE name='main'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(name, alias);
}

// --- The reverted candidates' shape: alias kept verbatim, NOFOLLOW stripped --
// `98b8446` stripped SQLITE_OPEN_NOFOLLOW for `/dev/fd/N`; `0700cb9` kept the
// literal `/proc/self/fd/N` on Linux. Their proxy VFS returned the alias from
// xFullPathname verbatim. Reproduce that shape with a verbatim-only VFS copy:
// the stock unixOpen still ORs O_NOFOLLOW into open(2) unconditionally.

unsafe extern "C" fn verbatim_full_pathname(
    _vfs: *mut rusqlite::ffi::sqlite3_vfs,
    name: *const std::ffi::c_char,
    n_out: std::ffi::c_int,
    out: *mut std::ffi::c_char,
) -> std::ffi::c_int {
    let bytes = std::ffi::CStr::from_ptr(name).to_bytes_with_nul();
    if bytes.len() > n_out as usize {
        return rusqlite::ffi::SQLITE_CANTOPEN;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr().cast(), out, bytes.len());
    rusqlite::ffi::SQLITE_OK
}

fn register_verbatim_vfs() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| unsafe {
        let unix = rusqlite::ffi::sqlite3_vfs_find(c"unix".as_ptr());
        let mut v: rusqlite::ffi::sqlite3_vfs = *unix;
        v.pNext = std::ptr::null_mut();
        v.zName = c"c4-verbatim".as_ptr();
        v.xFullPathname = Some(verbatim_full_pathname);
        assert_eq!(
            rusqlite::ffi::sqlite3_vfs_register(Box::leak(Box::new(v)), 0),
            rusqlite::ffi::SQLITE_OK
        );
    });
}

#[cfg(target_os = "linux")]
#[test]
fn verbatim_proc_fd_alias_without_nofollow_hits_unixopen_o_nofollow() {
    register_verbatim_vfs();
    let dir = fresh_dir("procfd-verbatim");
    let pinned = pinned_then_swapped(&dir);
    let alias = format!("/proc/self/fd/{}", pinned.as_raw_fd());
    let err = Connection::open_with_flags_and_vfs(&alias, no_nofollow(), "c4-verbatim")
        .and_then(|c| c.query_row("SELECT 1 FROM t", [], |_| Ok(())))
        .expect_err("open(2) with O_NOFOLLOW on a magic link must fail (ELOOP)");
    assert_eq!(
        sqlite_code(&err).map(|c| c & 0xff),
        Some(rusqlite::ffi::SQLITE_CANTOPEN),
        "{err:?}"
    );
}

#[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "openbsd"))]
#[test]
fn verbatim_dev_fd_alias_without_nofollow_opens_the_pinned_main_file() {
    register_verbatim_vfs();
    let dir = fresh_dir("devfd-verbatim");
    let pinned = pinned_then_swapped(&dir);
    let alias = format!("/dev/fd/{}", pinned.as_raw_fd());
    let c = Connection::open_with_flags_and_vfs(&alias, no_nofollow(), "c4-verbatim")
        .expect("open");
    assert_eq!(sentinel(&c), "original");
}
