//! Option E: trusted directory chain (OpenSSH StrictModes style).
//!
//! Walks the path from "/" WITHOUT following symlinks (`lstat` per
//! component; any symlink is refused, so callers pass a canonical path).
//! Each component must be a directory owned by root or the euid. Group- or
//! other-writable components are refused, except a sticky world-writable
//! ancestor (e.g. /tmp) whose NEXT component is owned by the euid or root (sticky
//! semantics stop other uids from renaming/removing that entry). The final
//! (database) directory must be owned by the euid and not writable by
//! group/others at all.
//!
//! Every property checked can only be changed by root or the euid, so the
//! verdict cannot be invalidated between the check and SQLite's open by any
//! other non-root uid. Spike residuals: ACLs (macOS/NFSv4 ACLs, POSIX ACLs)
//! can grant write without mode bits and are NOT inspected here; network /
//! FUSE filesystems may misreport ownership.

use std::io;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};

fn reject(path: &Path, why: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        format!("untrusted database directory chain at {}: {why}", path.display()),
    )
}

pub fn verify_trusted_chain(dir: &Path) -> io::Result<()> {
    if !dir.is_absolute() {
        return Err(reject(dir, "path must be absolute (canonical)"));
    }
    let euid = unsafe { libc::geteuid() };
    let mut chain: Vec<PathBuf> = Vec::new();
    let mut cur = PathBuf::new();
    for comp in dir.components() {
        match comp {
            Component::RootDir => cur.push("/"),
            Component::Normal(c) => cur.push(c),
            _ => return Err(reject(dir, "path must be canonical (no . or ..)")),
        }
        chain.push(cur.clone());
    }
    let metas = chain
        .iter()
        .map(|p| std::fs::symlink_metadata(p).map(|m| (p, m)))
        .collect::<io::Result<Vec<_>>>()?;

    for (i, (path, m)) in metas.iter().enumerate() {
        if m.file_type().is_symlink() {
            return Err(reject(path, "symlinked component"));
        }
        if !m.is_dir() {
            return Err(reject(path, "not a directory"));
        }
        if m.uid() != 0 && m.uid() != euid {
            return Err(reject(path, &format!("owned by uid {}", m.uid())));
        }
        let mode = m.permissions().mode();
        let is_last = i + 1 == metas.len();
        if is_last {
            if m.uid() != euid {
                return Err(reject(path, "database directory not owned by euid"));
            }
            if mode & 0o022 != 0 {
                return Err(reject(path, &format!("mode {:o}", mode & 0o7777)));
            }
            continue;
        }
        if mode & 0o022 != 0 {
            // Writable by group/others: tolerated only for a sticky directory
            // (/tmp style) whose next component other uids cannot rename.
            let sticky = mode & 0o1000 != 0;
            let next_uid = metas[i + 1].1.uid();
            let next_owned = next_uid == euid || next_uid == 0;
            if !(sticky && next_owned) {
                return Err(reject(
                    path,
                    &format!("writable by group/others, mode {:o}", mode & 0o7777),
                ));
            }
        }
    }
    Ok(())
}

const ARTIFACT_SUFFIXES: [&str; 4] = ["", "-wal", "-shm", "-journal"];

fn check_artifact_metadata(label: &Path, m: &std::fs::Metadata) -> io::Result<()> {
    let euid = unsafe { libc::geteuid() };
    if m.file_type().is_symlink() {
        return Err(reject(label, "symlinked database artifact"));
    }
    if !m.is_file() {
        return Err(reject(label, "database artifact is not a regular file"));
    }
    if m.uid() != euid {
        return Err(reject(label, &format!("artifact owned by uid {}", m.uid())));
    }
    let mode = m.permissions().mode() & 0o7777;
    if mode & 0o077 != 0 {
        return Err(reject(label, &format!("artifact mode {mode:o} is not owner-only")));
    }
    Ok(())
}

/// Pre-open check of `db` and its sidecars inside an already trusted
/// directory: each EXISTING one must be a regular file (lstat, no symlink),
/// owned by the euid, with mode & 077 == 0. Missing sidecars are fine.
/// Residual (not closable by any path or mode check): a process of another
/// uid that opened the file while it was still permissive keeps its fd.
pub fn verify_db_artifacts(dir: &Path, db_name: &str) -> io::Result<()> {
    for suffix in ARTIFACT_SUFFIXES {
        let p = dir.join(format!("{db_name}{suffix}"));
        match std::fs::symlink_metadata(&p) {
            Ok(m) => check_artifact_metadata(&p, &m)?,
            Err(e) if e.kind() == io::ErrorKind::NotFound && !suffix.is_empty() => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}

/// Post-open re-check on a handle (fstat): same rules as above. In Engram the
/// handle would be SQLite's own (e.g. via SQLITE_FCNTL_FILE_POINTER) or, under
/// a trusted chain where names cannot change, an lstat of the same name.
pub fn verify_open_artifact(file: &std::fs::File) -> io::Result<()> {
    let m = file.metadata()?;
    check_artifact_metadata(Path::new("<open handle>"), &m)
}
