//! STUB (RED phase): pathname open through the stock "unix" VFS, i.e. what
//! Engram does today. Replaced by the pinned-directory shim for GREEN.

use rusqlite::{Connection, OpenFlags};
use std::io;
use std::path::Path;

pub struct BoundDb {
    pub open_path: String,
}

pub const VFS_NAME: &str = "unix";

pub fn bind(db_path: &Path) -> io::Result<BoundDb> {
    let parent = db_path.parent().unwrap_or(Path::new("."));
    let name = db_path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "no file name"))?;
    let open_path = parent.canonicalize()?.join(name);
    Ok(BoundDb {
        open_path: open_path.to_string_lossy().into_owned(),
    })
}

pub fn open(bound: &BoundDb, flags: OpenFlags) -> rusqlite::Result<Connection> {
    Connection::open_with_flags_and_vfs(&bound.open_path, flags, VFS_NAME)
}
