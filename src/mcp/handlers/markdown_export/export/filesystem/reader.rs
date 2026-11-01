//! Bounded, descriptor-relative Markdown input. Never follow input links or
//! transiently open an existing SQLite-bearing inode to inspect it.

use std::collections::BTreeSet;
use std::ffi::{CString, OsString};
use std::fs::File;
use std::io::{self, Read};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use rustix::fs::{AtFlags, Dir, FileType, Mode, OFlags};

use super::unix::{check_file, open_child, open_root, protected_inodes, trusted_directory};
use crate::storage::Storage;

use super::MarkdownSource;

#[derive(Clone, Copy)]
struct Limits {
    depth: usize,
    files: usize,
    entries: usize,
    file_bytes: usize,
    total_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            depth: 32,
            files: 10_000,
            entries: 20_000,
            file_bytes: 4 * 1024 * 1024,
            total_bytes: 64 * 1024 * 1024,
        }
    }
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message)
}

struct Walk<'a> {
    root: &'a Path,
    storage: &'a Storage,
    limits: Limits,
    protected: Vec<(u64, u64)>,
    visited: BTreeSet<(u64, u64)>,
    entries: usize,
    bytes: usize,
    sources: Vec<MarkdownSource>,
}

impl Walk<'_> {
    fn directory(&mut self, dir: &File, relative: &Path, depth: usize) -> io::Result<()> {
        if depth > self.limits.depth {
            return Err(invalid("Markdown input exceeds directory depth limit"));
        }
        trusted_directory(dir)?;
        let metadata = dir.metadata()?;
        if !self.visited.insert((metadata.dev(), metadata.ino())) {
            return Err(invalid("Markdown input contains a repeated directory"));
        }
        let mut names = Vec::new();
        for entry in Dir::read_from(dir)? {
            let entry = entry?;
            let name = entry.file_name();
            if matches!(name.to_bytes(), b"." | b"..") {
                continue;
            }
            self.entries += 1;
            if self.entries > self.limits.entries {
                return Err(invalid("Markdown input exceeds directory entry limit"));
            }
            names.push(CString::from(name));
        }
        names.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
        for name in names {
            let leaf = OsString::from_vec(name.as_bytes().to_vec());
            let path = relative.join(leaf);
            let name_str = path
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(|| invalid("Markdown input names must be UTF-8"))?;
            let stat = rustix::fs::statat(dir, &name, AtFlags::SYMLINK_NOFOLLOW)?;
            match FileType::from_raw_mode(stat.st_mode) {
                FileType::Directory => {
                    let child = open_child(dir, &name, false)?
                        .ok_or_else(|| invalid("Markdown input directory disappeared"))?;
                    self.directory(&child, &path, depth + 1)?;
                }
                FileType::Symlink => return Err(invalid("Markdown input links are not allowed")),
                kind => {
                    let lower = name_str.to_ascii_lowercase();
                    if path.extension().and_then(|ext| ext.to_str()) != Some("md")
                        || name_str.starts_with(['.', '_'])
                        || matches!(lower.as_str(), "index.md" | "readme.md")
                    {
                        continue;
                    }
                    if kind != FileType::RegularFile {
                        return Err(invalid("Markdown input must contain regular files"));
                    }
                    if self.sources.len() >= self.limits.files {
                        return Err(invalid("Markdown input exceeds file count limit"));
                    }
                    // Metadata-only checks precede regular-file open: closing an
                    // alias of SQLite could otherwise release process fcntl locks.
                    self.storage
                        .refuse_active_sqlite_artifact(self.root.join(&path))
                        .map_err(io::Error::other)?;
                    check_file(dir, &name, &self.protected)?;
                    let remaining = self.limits.total_bytes.saturating_sub(self.bytes);
                    let limit = self.limits.file_bytes.min(remaining);
                    if stat.st_size < 0 || stat.st_size as u64 > limit as u64 {
                        return Err(invalid("Markdown input exceeds byte limit"));
                    }
                    let fd = rustix::fs::openat(
                        dir,
                        &name,
                        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
                        Mode::empty(),
                    )?;
                    let file = File::from(fd);
                    let opened = file.metadata()?;
                    if !opened.is_file() || opened.nlink() != 1 {
                        return Err(invalid("Markdown input file changed or has multiple links"));
                    }
                    let mut bytes = Vec::new();
                    file.take(limit as u64 + 1).read_to_end(&mut bytes)?;
                    if bytes.len() > limit {
                        return Err(invalid("Markdown input exceeds byte limit"));
                    }
                    self.bytes += bytes.len();
                    let raw = String::from_utf8(bytes)
                        .map_err(|_| invalid("Markdown input content must be UTF-8"))?;
                    self.sources.push(MarkdownSource {
                        relative: path,
                        raw,
                    });
                }
            }
        }
        Ok(())
    }
}

pub(in crate::mcp::handlers::markdown_export) fn read_markdown(
    root: &Path,
    storage: &Storage,
) -> io::Result<Vec<MarkdownSource>> {
    read_with_limits(root, storage, Limits::default())
}

fn read_with_limits(
    root: &Path,
    storage: &Storage,
    limits: Limits,
) -> io::Result<Vec<MarkdownSource>> {
    storage
        .refuse_active_sqlite_artifact(root)
        .map_err(io::Error::other)?;
    let dir = open_root(root, false)?.ok_or_else(|| invalid("Markdown input root not found"))?;
    let mut walk = Walk {
        root,
        storage,
        limits,
        protected: protected_inodes(storage),
        visited: BTreeSet::new(),
        entries: 0,
        bytes: 0,
        sources: Vec::new(),
    };
    walk.directory(&dir, Path::new(""), 0)?;
    Ok(walk.sources)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_quotas_bound_files_bytes_entries_and_depth() {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().canonicalize().unwrap();
        let storage = Storage::open_in_memory().unwrap();
        std::fs::write(root.join("a.md"), "12345").unwrap();
        std::fs::write(root.join("b.md"), "67890").unwrap();
        for limits in [
            Limits {
                file_bytes: 4,
                ..Limits::default()
            },
            Limits {
                total_bytes: 9,
                ..Limits::default()
            },
            Limits {
                files: 1,
                ..Limits::default()
            },
            Limits {
                entries: 1,
                ..Limits::default()
            },
        ] {
            assert!(read_with_limits(&root, &storage, limits).is_err());
        }
        std::fs::create_dir(root.join("nested")).unwrap();
        assert!(read_with_limits(
            &root,
            &storage,
            Limits {
                depth: 0,
                ..Limits::default()
            }
        )
        .is_err());
        assert_eq!(read_markdown(&root, &storage).unwrap().len(), 2);
    }
}
