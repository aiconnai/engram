//! Test helpers: private temp dirs, sentinel databases, peer process.

use rusqlite::{Connection, OpenFlags};
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static SEQ: AtomicU64 = AtomicU64::new(0);

/// A fresh, canonical, owner-only (0700) directory under the temp dir.
pub fn fresh_dir(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().canonicalize().expect("temp dir");
    let n = SEQ.fetch_add(1, Ordering::Relaxed);
    let dir = base.join(format!("c4-{tag}-{}-{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&dir)
        .expect("create dir");
    dir.canonicalize().expect("canonical dir")
}

pub fn mkdir_private(dir: &Path) {
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(dir)
        .expect("mkdir");
}

pub fn chmod(path: &Path, mode: u32) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).expect("chmod");
}

pub fn flags() -> OpenFlags {
    OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_CREATE
        | OpenFlags::SQLITE_OPEN_NO_MUTEX
        | OpenFlags::SQLITE_OPEN_NOFOLLOW
}

/// Create a database holding one sentinel row via the stock VFS, then close.
pub fn make_db(path: &Path, sentinel: &str, journal_mode: &str) {
    let c = Connection::open(path).expect("make_db open");
    c.query_row(&format!("PRAGMA journal_mode={journal_mode}"), [], |r| {
        r.get::<_, String>(0)
    })
    .expect("journal mode");
    c.execute_batch("CREATE TABLE t(v TEXT NOT NULL)").expect("create");
    c.execute("INSERT INTO t(v) VALUES (?1)", [sentinel])
        .expect("insert");
}

/// The first sentinel row (the database's "identity").
pub fn sentinel(c: &Connection) -> String {
    c.query_row("SELECT v FROM t ORDER BY rowid LIMIT 1", [], |r| r.get(0))
        .expect("sentinel")
}

pub fn count(c: &Connection) -> i64 {
    c.query_row("SELECT count(*) FROM t", [], |r| r.get(0))
        .expect("count")
}

/// Run the peer helper (separate process, stock VFS by pathname).
pub fn peer(bin: &str, args: &[&str]) -> String {
    let out = std::process::Command::new(bin)
        .args(args)
        .output()
        .expect("spawn peer");
    assert!(
        out.status.success(),
        "peer {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}
