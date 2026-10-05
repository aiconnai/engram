//! Isolated kernel behaviour behind the shim stress failure: does
//! open(O_CREAT) of a sidecar name return ENOENT while another process/thread
//! unlinks that name (SQLite's last-closer WAL/-shm deletion)? Compares
//! path-based open(2) with dirfd-relative openat(2). Reports counts; asserts
//! only that the probe ran.

use c4spike::util::*;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn race(use_openat: bool) -> (usize, usize) {
    let dir = fresh_dir(if use_openat { "race-at" } else { "race-path" });
    let name = CString::new("engram.db-wal").unwrap();
    let full = CString::new(dir.join("engram.db-wal").as_os_str().as_bytes()).unwrap();
    let cdir = CString::new(dir.as_os_str().as_bytes()).unwrap();
    let dirfd = unsafe { libc::open(cdir.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY) };
    assert!(dirfd >= 0);
    let stop = Arc::new(AtomicBool::new(false));
    let s2 = stop.clone();
    let full2 = full.clone();
    let unlinker = std::thread::spawn(move || {
        while !s2.load(Ordering::Relaxed) {
            unsafe { libc::unlink(full2.as_ptr()) };
        }
    });
    let flags = libc::O_RDWR | libc::O_CREAT | libc::O_NOFOLLOW | libc::O_CLOEXEC;
    let (mut enoent, mut total) = (0, 0);
    for _ in 0..50_000 {
        let fd = unsafe {
            if use_openat {
                libc::openat(dirfd, name.as_ptr(), flags, 0o600 as libc::c_uint)
            } else {
                libc::open(full.as_ptr(), flags, 0o600 as libc::c_uint)
            }
        };
        total += 1;
        if fd < 0 {
            if std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
                enoent += 1;
            }
        } else {
            unsafe { libc::close(fd) };
        }
    }
    stop.store(true, Ordering::Relaxed);
    unlinker.join().unwrap();
    unsafe { libc::close(dirfd) };
    (enoent, total)
}

#[test]
fn create_vs_concurrent_unlink_enoent_rates() {
    let (p_enoent, p_total) = race(false);
    let (a_enoent, a_total) = race(true);
    eprintln!(
        "RACE-RESULT os={} open(path,O_CREAT): {p_enoent}/{p_total} ENOENT; \
         openat(dirfd,O_CREAT): {a_enoent}/{a_total} ENOENT",
        std::env::consts::OS
    );
    assert!(p_total > 0 && a_total > 0);
}
