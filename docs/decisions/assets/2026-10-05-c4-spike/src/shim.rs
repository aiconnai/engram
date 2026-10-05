//! Option B: pinned parent-directory descriptor + stock unix-VFS syscall shim.
//!
//! * `bind` resolves the parent ONCE, opens it `O_DIRECTORY|O_NOFOLLOW`,
//!   verifies the DIRECTORY FD (owner == euid, not group/other-writable) and
//!   records the main file's (dev, ino). Everything later is relative to that
//!   fd, so swapping any ancestor afterwards is irrelevant.
//! * A VFS "c4-pinned" (a copy of SQLite's own "unix" VFS) returns bound
//!   names verbatim from xFullPathname (no lstat/readlink walk); other names
//!   (VACUUM INTO / ATTACH targets) get the stock resolution.
//! * The unix VFS's overridable syscalls (`open`, `stat`, `access`, `unlink`,
//!   `openDirectory`) are redirected for the bound names (`db`, `db-wal`,
//!   `db-shm`, `db-journal`) to `openat`/`fstatat`/`faccessat`/`unlinkat`
//!   relative to the pinned fd. ALL locking, inode sharing, WAL-index and
//!   checkpoint logic stays SQLite's own: the fds SQLite gets are ordinary
//!   fresh opens of the same inodes, so POSIX lock semantics are unchanged.
//! * The shim never closes a descriptor of a database file it handed out
//!   or rejected (invariant #27): a rejected main-file fd is parked forever.
//!
//! CAVEAT (decisive for the ADR): `xSetSystemCall` is documented by SQLite
//! as a testing interface whose set of calls "varies ... from one version of
//! the same VFS to the next"; the table is process-global and unsynchronised.

use rusqlite::{ffi, Connection, OpenFlags};
use std::ffi::{c_char, c_int, CStr, CString};
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

pub const VFS_NAME: &str = "c4-pinned";
const SIDECARS: [&[u8]; 4] = [b"", b"-wal", b"-shm", b"-journal"];

pub struct BoundDb {
    pub open_path: String,
    _binding: Arc<Binding>,
}

struct Binding {
    dir: Vec<u8>,
    base: Vec<u8>,
    dirfd: OwnedFd,
    main_id: Mutex<Option<(u64, u64)>>,
}

type OpenFn = unsafe extern "C" fn(*const c_char, c_int, c_int) -> c_int;
type StatFn = unsafe extern "C" fn(*const c_char, *mut libc::stat) -> c_int;
type AccessFn = unsafe extern "C" fn(*const c_char, c_int) -> c_int;
type UnlinkFn = unsafe extern "C" fn(*const c_char) -> c_int;
type OpenDirFn = unsafe extern "C" fn(*const c_char, *mut c_int) -> c_int;

type FullPathnameFn =
    unsafe extern "C" fn(*mut ffi::sqlite3_vfs, *const c_char, c_int, *mut c_char) -> c_int;

struct Real {
    full_pathname: FullPathnameFn,
    open: OpenFn,
    stat: StatFn,
    access: AccessFn,
    unlink: UnlinkFn,
    open_dir: OpenDirFn,
}

static REAL: OnceLock<Real> = OnceLock::new();
static BINDINGS: RwLock<Vec<Arc<Binding>>> = RwLock::new(Vec::new());
/// Descriptors of rejected main files: never closed (closing could drop
/// POSIX locks if the swapped-in inode is live elsewhere in this process).
/// This is a deliberate, unbounded leak of one fd per rejected open.
static PARKED: Mutex<Vec<RawFd>> = Mutex::new(Vec::new());

fn err(msg: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, msg.into())
}

fn set_errno(e: c_int) {
    #[cfg(target_os = "linux")]
    unsafe {
        *libc::__errno_location() = e;
    }
    #[cfg(any(target_os = "macos", target_os = "freebsd"))]
    unsafe {
        *libc::__error() = e;
    }
}

// ---------------------------------------------------------------- install --

unsafe fn get_call(unix: *mut ffi::sqlite3_vfs, name: &CStr) -> unsafe extern "C" fn() {
    let get = (*unix).xGetSystemCall.expect("xGetSystemCall");
    get(unix, name.as_ptr()).unwrap_or_else(|| panic!("syscall {name:?} missing"))
}

unsafe fn set_call(unix: *mut ffi::sqlite3_vfs, name: &CStr, f: unsafe extern "C" fn()) {
    let set = (*unix).xSetSystemCall.expect("xSetSystemCall");
    let rc = set(unix, name.as_ptr(), Some(f));
    assert_eq!(rc, ffi::SQLITE_OK, "override {name:?}");
}

fn install() -> &'static Real {
    REAL.get_or_init(|| unsafe {
        let unix = ffi::sqlite3_vfs_find(c"unix".as_ptr());
        assert!(!unix.is_null(), "unix VFS missing");
        assert!((*unix).iVersion >= 3, "unix VFS lacks syscall overrides");
        let real = Real {
            full_pathname: (*unix).xFullPathname.expect("unix xFullPathname"),
            open: std::mem::transmute::<unsafe extern "C" fn(), OpenFn>(get_call(unix, c"open")),
            stat: std::mem::transmute::<unsafe extern "C" fn(), StatFn>(get_call(unix, c"stat")),
            access: std::mem::transmute::<unsafe extern "C" fn(), AccessFn>(get_call(
                unix, c"access",
            )),
            unlink: std::mem::transmute::<unsafe extern "C" fn(), UnlinkFn>(get_call(
                unix, c"unlink",
            )),
            open_dir: std::mem::transmute::<unsafe extern "C" fn(), OpenDirFn>(get_call(
                unix,
                c"openDirectory",
            )),
        };
        set_call(unix, c"open", std::mem::transmute::<OpenFn, _>(shim_open as OpenFn));
        set_call(unix, c"stat", std::mem::transmute::<StatFn, _>(shim_stat as StatFn));
        set_call(unix, c"access", std::mem::transmute::<AccessFn, _>(shim_access as AccessFn));
        set_call(unix, c"unlink", std::mem::transmute::<UnlinkFn, _>(shim_unlink as UnlinkFn));
        set_call(
            unix,
            c"openDirectory",
            std::mem::transmute::<OpenDirFn, _>(shim_open_dir as OpenDirFn),
        );

        // A copy of the unix VFS (keeps pAppData = locking-style finder, all
        // I/O + locking methods) with a verbatim, bound-only xFullPathname.
        let mut vfs: ffi::sqlite3_vfs = *unix;
        vfs.pNext = std::ptr::null_mut();
        vfs.zName = c"c4-pinned".as_ptr();
        vfs.xFullPathname = Some(pinned_full_pathname);
        let rc = ffi::sqlite3_vfs_register(Box::leak(Box::new(vfs)), 0);
        assert_eq!(rc, ffi::SQLITE_OK, "register c4-pinned");
        real
    })
}

// ------------------------------------------------------------------- bind --

fn fstat_fd(fd: RawFd) -> io::Result<libc::stat> {
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstat(fd, &mut st) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(st)
}

/// Verify the pinned DIRECTORY descriptor, not a pathname: owner must be the
/// effective uid and no group/other write bit may be set. Other uids then
/// cannot add, rename or remove entries in it (POSIX ACLs / macOS ACLs are a
/// documented residual of this spike).
fn verify_dir_fd(fd: RawFd) -> io::Result<()> {
    let st = fstat_fd(fd)?;
    if (st.st_mode & libc::S_IFMT) != libc::S_IFDIR {
        return Err(err("database parent is not a directory"));
    }
    let euid = unsafe { libc::geteuid() };
    if st.st_uid != euid {
        return Err(err(format!("database directory owned by uid {}", st.st_uid)));
    }
    if (st.st_mode as u32) & 0o022 != 0 {
        return Err(err(format!(
            "database directory mode {:o} is writable by group/others",
            st.st_mode as u32 & 0o7777
        )));
    }
    Ok(())
}

pub fn bind(db_path: &Path) -> io::Result<BoundDb> {
    install();
    let parent = db_path.parent().unwrap_or(Path::new("."));
    let base = db_path
        .file_name()
        .ok_or_else(|| err("no file name"))?
        .as_bytes()
        .to_vec();
    let dir = parent.canonicalize()?.as_os_str().as_bytes().to_vec();

    let c_dir = CString::new(dir.clone()).map_err(|_| err("NUL in path"))?;
    // macOS O_NOFOLLOW_ANY / Linux openat2(RESOLVE_NO_SYMLINKS) would also
    // refuse intermediate symlinks here; the fd verification below is what
    // the spike relies on.
    let raw = unsafe {
        libc::open(
            c_dir.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if raw < 0 {
        return Err(io::Error::last_os_error());
    }
    let dirfd = unsafe { OwnedFd::from_raw_fd(raw) };
    verify_dir_fd(dirfd.as_raw_fd())?;

    if let Some(existing) = BINDINGS
        .read()
        .unwrap()
        .iter()
        .find(|b| b.dir == dir && b.base == base)
    {
        // Same database already bound in this process (pool). Reuse it; the
        // new dirfd is closed (a directory fd carries no database locks).
        let open_path = join(&existing.dir, &existing.base);
        return Ok(BoundDb {
            open_path,
            _binding: existing.clone(),
        });
    }

    let main_id = create_or_identify_main(dirfd.as_raw_fd(), &base)?;
    let binding = Arc::new(Binding {
        dir,
        base,
        dirfd,
        main_id: Mutex::new(Some(main_id)),
    });
    BINDINGS.write().unwrap().push(binding.clone());
    Ok(BoundDb {
        open_path: join(&binding.dir, &binding.base),
        _binding: binding,
    })
}

fn join(dir: &[u8], base: &[u8]) -> String {
    let mut p = dir.to_vec();
    if !p.ends_with(b"/") {
        p.push(b'/');
    }
    p.extend_from_slice(base);
    String::from_utf8(p).expect("utf-8 path in spike")
}

/// New file: `O_CREAT|O_EXCL|O_NOFOLLOW` 0600 relative to the pinned dir; the
/// creation fd is closed before ANY SQLite connection can know the inode
/// (same serialization caveat as Engram's DATABASE_OPEN_LOCK). Existing
/// file: identified with `fstatat(AT_SYMLINK_NOFOLLOW)`, never opened.
fn create_or_identify_main(dirfd: RawFd, base: &[u8]) -> io::Result<(u64, u64)> {
    let name = CString::new(base.to_vec()).map_err(|_| err("NUL in name"))?;
    let fd = unsafe {
        libc::openat(
            dirfd,
            name.as_ptr(),
            libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600 as libc::c_uint,
        )
    };
    if fd >= 0 {
        let created = unsafe { OwnedFd::from_raw_fd(fd) };
        let st = fstat_fd(created.as_raw_fd())?;
        return Ok((st.st_dev as u64, st.st_ino as u64));
    }
    let e = io::Error::last_os_error();
    if e.raw_os_error() != Some(libc::EEXIST) {
        return Err(e);
    }
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    if unsafe { libc::fstatat(dirfd, name.as_ptr(), &mut st, libc::AT_SYMLINK_NOFOLLOW) } != 0 {
        return Err(io::Error::last_os_error());
    }
    if (st.st_mode & libc::S_IFMT) != libc::S_IFREG {
        return Err(err("database is not a regular file"));
    }
    Ok((st.st_dev as u64, st.st_ino as u64))
}

pub fn open(bound: &BoundDb, flags: OpenFlags) -> rusqlite::Result<Connection> {
    Connection::open_with_flags_and_vfs(&bound.open_path, flags, VFS_NAME)
}

// --------------------------------------------------------------- dispatch --

struct Hit {
    binding: Arc<Binding>,
    name: CString,
    is_main: bool,
}

fn classify(path: *const c_char) -> Option<Hit> {
    if path.is_null() {
        return None;
    }
    let p = unsafe { CStr::from_ptr(path) }.to_bytes();
    let bindings = BINDINGS.read().ok()?;
    for b in bindings.iter() {
        let Some(rest) = p.strip_prefix(b.dir.as_slice()) else {
            continue;
        };
        let rest = rest.strip_prefix(b"/").unwrap_or(rest);
        let Some(suffix) = rest.strip_prefix(b.base.as_slice()) else {
            continue;
        };
        if SIDECARS.contains(&suffix) {
            return Some(Hit {
                binding: b.clone(),
                name: CString::new(rest.to_vec()).ok()?,
                is_main: suffix.is_empty(),
            });
        }
    }
    None
}

unsafe extern "C" fn shim_open(path: *const c_char, flags: c_int, mode: c_int) -> c_int {
    let real = REAL.get().expect("installed");
    let Some(hit) = classify(path) else {
        return (real.open)(path, flags, mode);
    };
    let fd = openat_create_retrying(&hit, flags, mode);
    if std::env::var_os("C4_TRACE").is_some() {
        let e = if fd < 0 { io::Error::last_os_error().to_string() } else { String::new() };
        let ro = (flags & libc::O_ACCMODE) == libc::O_RDONLY;
        eprintln!("shim_open {:?} flags={flags:#x} rdonly={} -> {fd} {e}", hit.name, ro as u8);
    }
    if fd < 0 || !hit.is_main {
        return fd;
    }
    let ok = match fstat_fd(fd) {
        Ok(st) => {
            let id = (st.st_dev as u64, st.st_ino as u64);
            let mut expected = hit.binding.main_id.lock().unwrap();
            match *expected {
                Some(want) => want == id,
                None => {
                    *expected = Some(id);
                    true
                }
            }
        }
        Err(_) => false,
    };
    if ok {
        fd
    } else {
        PARKED.lock().unwrap().push(fd);
        set_errno(libc::EACCES);
        -1
    }
}

/// macOS finding (spike stress test): `open*(O_CREAT)` can fail with ENOENT
/// while another process unlinks that name (SQLite's last closer deleting
/// -wal/-shm). SQLite then falls back to O_RDONLY and, for -shm, marks the
/// process-wide shm node read-only => every in-process connection fails
/// writes with SQLITE_READONLY. The micro-probe (tests/create_unlink_race.rs)
/// sees the kernel ENOENT for both open(path) and openat(dirfd) on macOS
/// (never on Linux); end-to-end it hit the shim in 2-8 of 40 stress runs and
/// the stock path in none of the runs recorded in the spike logs. Bounded
/// retry fixes it. `C4_DISABLE_CREATE_RETRY=1` turns the retry off so the
/// failure can be reproduced (see README "F7 repro").
unsafe fn openat_create_retrying(hit: &Hit, flags: c_int, mode: c_int) -> c_int {
    const RETRIES: usize = 64;
    let retries = if std::env::var_os("C4_DISABLE_CREATE_RETRY").is_some() {
        0
    } else {
        RETRIES
    };
    let mut attempt = 0;
    loop {
        let fd = libc::openat(
            hit.binding.dirfd.as_raw_fd(),
            hit.name.as_ptr(),
            flags | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            mode as libc::c_uint,
        );
        let enoent = fd < 0 && io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT);
        if !(enoent && flags & libc::O_CREAT != 0 && attempt < retries) {
            if enoent && std::env::var_os("C4_TRACE").is_some() {
                eprintln!("shim_open {:?} ENOENT after {attempt} retries", hit.name);
            }
            if attempt > 0 && std::env::var_os("C4_TRACE").is_some() {
                eprintln!("shim_open {:?} create retried {attempt}x", hit.name);
            }
            if enoent {
                set_errno(libc::ENOENT);
            }
            return fd;
        }
        attempt += 1;
    }
}

unsafe extern "C" fn shim_stat(path: *const c_char, buf: *mut libc::stat) -> c_int {
    let real = REAL.get().expect("installed");
    match classify(path) {
        Some(hit) => {
            let rc = libc::fstatat(
                hit.binding.dirfd.as_raw_fd(),
                hit.name.as_ptr(),
                buf,
                libc::AT_SYMLINK_NOFOLLOW,
            );
            if std::env::var_os("C4_TRACE").is_some() {
                eprintln!("shim_stat {:?} -> {rc}", hit.name);
            }
            rc
        }
        None => (real.stat)(path, buf),
    }
}

unsafe extern "C" fn shim_access(path: *const c_char, mode: c_int) -> c_int {
    let real = REAL.get().expect("installed");
    match classify(path) {
        Some(hit) => {
            let mut st: libc::stat = std::mem::zeroed();
            let fd = hit.binding.dirfd.as_raw_fd();
            if libc::fstatat(fd, hit.name.as_ptr(), &mut st, libc::AT_SYMLINK_NOFOLLOW) != 0 {
                return -1;
            }
            if (st.st_mode & libc::S_IFMT) == libc::S_IFLNK {
                set_errno(libc::ELOOP);
                return -1;
            }
            if mode == libc::F_OK {
                0
            } else {
                libc::faccessat(fd, hit.name.as_ptr(), mode, 0)
            }
        }
        None => (real.access)(path, mode),
    }
}

unsafe extern "C" fn shim_unlink(path: *const c_char) -> c_int {
    let real = REAL.get().expect("installed");
    match classify(path) {
        Some(hit) if !hit.is_main => {
            let rc = libc::unlinkat(hit.binding.dirfd.as_raw_fd(), hit.name.as_ptr(), 0);
            if std::env::var_os("C4_TRACE").is_some() {
                eprintln!("shim_unlink {:?} -> {rc}", hit.name);
            }
            rc
        }
        Some(_) => {
            set_errno(libc::EPERM);
            -1
        }
        None => (real.unlink)(path),
    }
}

/// SQLite fsyncs the directory after creating/deleting a journal: hand it a
/// dup of the pinned directory fd (dirs carry no database locks).
unsafe extern "C" fn shim_open_dir(path: *const c_char, out: *mut c_int) -> c_int {
    let real = REAL.get().expect("installed");
    if !path.is_null() {
        let p = CStr::from_ptr(path).to_bytes();
        let dir = match p.iter().rposition(|&c| c == b'/') {
            Some(i) if i > 0 => &p[..i],
            _ => p,
        };
        let bindings = BINDINGS.read().unwrap();
        if let Some(b) = bindings.iter().find(|b| b.dir.as_slice() == dir) {
            let fd = libc::fcntl(b.dirfd.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 3);
            *out = fd;
            return if fd >= 0 { ffi::SQLITE_OK } else { ffi::SQLITE_CANTOPEN };
        }
    }
    (real.open_dir)(path, out)
}

/// Bound names are returned verbatim (no lstat/readlink walk that a swapped
/// ancestor could redirect). Any OTHER name (VACUUM INTO / ATTACH targets,
/// opened through this same VFS) gets SQLite's stock resolution, i.e. exactly
/// today's behaviour for those files.
unsafe extern "C" fn pinned_full_pathname(
    vfs: *mut ffi::sqlite3_vfs,
    name: *const c_char,
    n_out: c_int,
    out: *mut c_char,
) -> c_int {
    if classify(name).is_none() {
        let real = REAL.get().expect("installed");
        return (real.full_pathname)(vfs, name, n_out, out);
    }
    let bytes = CStr::from_ptr(name).to_bytes_with_nul();
    if bytes.len() > n_out as usize {
        return ffi::SQLITE_CANTOPEN;
    }
    std::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), out, bytes.len());
    ffi::SQLITE_OK
}
