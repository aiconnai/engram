//! Export writes are relative to pinned directory descriptors, never a checked
//! path followed by a regular-file open. The output root and descendants must
//! be owned by this process user and not writable by other users/groups.
//! Administrators must protect the root's ancestors from same-user replacement;
//! this is not a sandbox against another process with the server's OS identity.

use std::collections::BTreeSet;
use std::io;
use std::path::{Component, Path, PathBuf};

use crate::storage::Storage;

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod acl;

pub(in crate::mcp::handlers::markdown_export) struct MarkdownSource {
    pub relative: PathBuf,
    pub raw: String,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod reader;
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(in crate::mcp::handlers::markdown_export) use reader::read_markdown;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub(in crate::mcp::handlers::markdown_export) fn read_markdown(
    _root: &Path,
    _storage: &Storage,
) -> io::Result<Vec<MarkdownSource>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "safe filesystem import requires Linux/macOS directory-relative and ACL operations",
    ))
}

pub(super) fn validate_component(name: &str) -> io::Result<()> {
    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && matches!(stem.as_bytes()[3], b'1'..=b'9'));
    if name.is_empty()
        || matches!(name, "." | "..")
        || name.ends_with(['.', ' '])
        || name
            .chars()
            .any(|c| c.is_control() || "/\\:<>\"|?*".contains(c))
        || reserved
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unsafe export path component",
        ));
    }
    Ok(())
}

fn validate_batch(root: &Path, batch: &[(PathBuf, String)], storage: &Storage) -> io::Result<()> {
    storage
        .refuse_active_sqlite_artifact(root)
        .map_err(io::Error::other)?;
    let mut names = BTreeSet::new();
    for (relative, _) in batch {
        if relative.as_os_str().is_empty() || relative.is_absolute() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "export path must be relative",
            ));
        }
        for component in relative.components() {
            let Component::Normal(name) = component else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "invalid export path",
                ));
            };
            validate_component(
                name.to_str()
                    .ok_or_else(|| io::Error::other("non-UTF-8 export name"))?,
            )?;
        }
        if !names.insert(relative.clone()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "duplicate export destination",
            ));
        }
        storage
            .refuse_active_sqlite_artifact(root.join(relative))
            .map_err(io::Error::other)?;
    }
    for relative in &names {
        if relative
            .ancestors()
            .skip(1)
            .any(|parent| names.contains(parent))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "export file/directory collision",
            ));
        }
    }
    Ok(())
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub(super) fn write_batch(
    root: &Path,
    batch: &[(PathBuf, String)],
    storage: &Storage,
) -> io::Result<()> {
    validate_batch(root, batch, storage)?;
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "safe filesystem export requires Linux/macOS directory-relative and ACL operations",
    ))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(super) use unix::write_batch;

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod unix {
    use super::*;
    use std::collections::BTreeMap;
    use std::ffi::{CStr, CString, OsStr};
    use std::fs::{File, OpenOptions};
    use std::io::Write;
    use std::mem::MaybeUninit;
    use std::os::fd::{AsRawFd, FromRawFd};
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

    fn c_name(name: &OsStr) -> io::Result<CString> {
        CString::new(name.as_bytes())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "null byte in export path"))
    }

    fn sync_to_filesystem(file: &File) -> io::Result<()> {
        rustix::fs::fsync(file).map_err(io::Error::from)
    }

    pub(super) fn open_child(parent: &File, name: &CStr, create: bool) -> io::Result<Option<File>> {
        // SAFETY: parent is a live directory fd and name is NUL-terminated.
        // O_DIRECTORY prevents opening a SQLite regular file even transiently.
        let mut fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if fd < 0 && io::Error::last_os_error().kind() == io::ErrorKind::NotFound {
            if !create {
                return Ok(None);
            }
            // SAFETY: same live directory/name; only this one component is created.
            let made = unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) };
            if made < 0 && io::Error::last_os_error().kind() != io::ErrorKind::AlreadyExists {
                return Err(io::Error::last_os_error());
            }
            sync_to_filesystem(parent)?;
            // SAFETY: open the just-created directory without following a replacement symlink.
            fd = unsafe {
                libc::openat(
                    parent.as_raw_fd(),
                    name.as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                )
            };
        }
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: a successful openat returned a new uniquely owned descriptor.
        Ok(Some(unsafe { File::from_raw_fd(fd) }))
    }

    pub(super) fn trusted_directory(dir: &File) -> io::Result<()> {
        let metadata = dir.metadata()?;
        // SAFETY: geteuid has no arguments and does not mutate memory.
        if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o022 != 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "export directories must be owned by the server user and not group/world writable",
            ));
        }
        super::acl::check_directory(dir)
    }

    pub(super) fn open_root(path: &Path, create: bool) -> io::Result<Option<File>> {
        if !path.is_absolute() {
            return Err(io::Error::other("export root is not absolute"));
        }
        let mut dir = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open("/")?;
        for component in path.components() {
            match component {
                Component::RootDir => (),
                Component::Normal(name) => {
                    let Some(child) = open_child(&dir, &c_name(name)?, create)? else {
                        return Ok(None);
                    };
                    dir = child;
                }
                _ => {
                    return Err(io::Error::other(
                        "export root contains non-normal components",
                    ))
                }
            }
        }
        trusted_directory(&dir)?;
        Ok(Some(dir))
    }

    fn parent_dir(root: &File, relative: &Path, create: bool) -> io::Result<Option<File>> {
        let mut dir = root.try_clone()?; // Only duplicates a directory, never SQLite.
        for component in relative
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .components()
        {
            let Component::Normal(name) = component else {
                return Err(io::Error::other("invalid subdirectory"));
            };
            let Some(child) = open_child(&dir, &c_name(name)?, create)? else {
                return Ok(None);
            };
            trusted_directory(&child)?;
            dir = child;
        }
        Ok(Some(dir))
    }

    pub(super) fn protected_inodes(storage: &Storage) -> Vec<(u64, u64)> {
        if storage.db_path() == ":memory:" {
            return Vec::new();
        }
        ["", "-wal", "-shm", "-journal"]
            .iter()
            .filter_map(|suffix| {
                std::fs::metadata(format!("{}{suffix}", storage.db_path()))
                    .ok()
                    .map(|m| (m.dev(), m.ino()))
            })
            .collect()
    }

    #[allow(clippy::unnecessary_cast)] // libc's dev_t/ino_t differ across Unix targets.
    pub(super) fn check_file(
        parent: &File,
        name: &CStr,
        protected: &[(u64, u64)],
    ) -> io::Result<()> {
        let mut stat = MaybeUninit::<libc::stat>::uninit();
        // SAFETY: stat points to enough writable storage, parent is live and name valid.
        // This inspects the entry without opening/closing a SQLite-bearing inode.
        if unsafe {
            libc::fstatat(
                parent.as_raw_fd(),
                name.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } < 0
        {
            let err = io::Error::last_os_error();
            return if err.kind() == io::ErrorKind::NotFound {
                Ok(())
            } else {
                Err(err)
            };
        }
        // SAFETY: successful fstatat initialized the entire stat structure.
        let stat = unsafe { stat.assume_init() };
        if stat.st_mode & libc::S_IFMT != libc::S_IFREG
            || stat.st_nlink > 1
            || protected.contains(&(stat.st_dev as u64, stat.st_ino as u64))
        {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "export destination is a link, non-regular file or active SQLite artifact",
            ));
        }
        Ok(())
    }

    fn atomic_write(
        parent: &File,
        name: &CStr,
        protected: &[(u64, u64)],
        write: impl FnOnce(&mut File) -> io::Result<()>,
    ) -> io::Result<()> {
        atomic_write_with(parent, name, protected, write, |temporary| {
            // SAFETY: both names are single components under the same pinned directory.
            // renameat replaces an entry atomically; it never follows a final symlink.
            if unsafe {
                libc::renameat(
                    parent.as_raw_fd(),
                    temporary.as_ptr(),
                    parent.as_raw_fd(),
                    name.as_ptr(),
                )
            } < 0
            {
                Err(io::Error::last_os_error())
            } else {
                Ok(())
            }
        })
    }

    fn atomic_write_with(
        parent: &File,
        name: &CStr,
        protected: &[(u64, u64)],
        write: impl FnOnce(&mut File) -> io::Result<()>,
        publish: impl FnOnce(&CStr) -> io::Result<()>,
    ) -> io::Result<()> {
        check_file(parent, name, protected)?;
        let temp = CString::new(format!(".engram-export-{}.tmp", uuid::Uuid::new_v4()))
            .map_err(io::Error::other)?;
        // SAFETY: exclusive creation of a new private file relative to the live directory.
        // No existing inode (including an active SQLite alias) can be opened here.
        let fd = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                temp.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600 as libc::c_uint,
            )
        };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: successful exclusive openat returned a new uniquely owned fd.
        let mut file = unsafe { File::from_raw_fd(fd) };
        let result = (|| {
            write(&mut file)?;
            sync_to_filesystem(&file)?;
            check_file(parent, name, protected)?;
            publish(&temp)?;
            Ok(())
        })();
        if let Err(err) = result {
            // SAFETY: removes only our temporary name in the pinned directory, never a link target.
            if unsafe { libc::unlinkat(parent.as_raw_fd(), temp.as_ptr(), 0) } < 0 {
                let cleanup = io::Error::last_os_error();
                if cleanup.kind() != io::ErrorKind::NotFound {
                    return Err(io::Error::other(format!(
                        "{err}; temporary cleanup failed: {cleanup}"
                    )));
                }
            }
            return Err(err);
        }
        Ok(())
    }

    pub(crate) fn write_batch(
        root: &Path,
        batch: &[(PathBuf, String)],
        storage: &Storage,
    ) -> io::Result<()> {
        write_batch_with_barrier(root, batch, storage, |directory| directory.sync_all())
    }

    fn write_batch_with_barrier(
        root: &Path,
        batch: &[(PathBuf, String)],
        storage: &Storage,
        mut barrier: impl FnMut(&File) -> io::Result<()>,
    ) -> io::Result<()> {
        validate_batch(root, batch, storage)?;
        let protected = protected_inodes(storage);
        let existing = open_root(root, false)?;
        if let Some(dir) = &existing {
            for (relative, _) in batch {
                if let Some(parent) = parent_dir(dir, relative, false)? {
                    check_file(
                        &parent,
                        &c_name(
                            relative
                                .file_name()
                                .ok_or_else(|| io::Error::other("missing filename"))?,
                        )?,
                        &protected,
                    )?;
                }
            }
        }
        let dir = match existing {
            Some(dir) => dir,
            None => {
                open_root(root, true)?.ok_or_else(|| io::Error::other("export root unavailable"))?
            }
        };
        // Group by destination directory so its metadata is synchronized once.
        // Keep only one pinned representative per device, not one fd per file
        // or directory: large exports must not exhaust the descriptor limit.
        let mut groups: BTreeMap<PathBuf, Vec<&(PathBuf, String)>> = BTreeMap::new();
        for entry in batch {
            groups
                .entry(entry.0.parent().unwrap_or_else(|| Path::new("")).to_owned())
                .or_default()
                .push(entry);
        }
        let root_files = groups.remove(Path::new(""));
        let mut devices: BTreeMap<u64, File> = BTreeMap::new();
        // The root group contains index.md, which remains last in the batch.
        for (_, files) in groups
            .into_iter()
            .chain(root_files.into_iter().map(|files| (PathBuf::new(), files)))
        {
            let first = files
                .first()
                .ok_or_else(|| io::Error::other("empty export group"))?;
            let parent = parent_dir(&dir, &first.0, true)?
                .ok_or_else(|| io::Error::other("export parent unavailable"))?;
            for (relative, bytes) in files {
                let name = c_name(
                    relative
                        .file_name()
                        .ok_or_else(|| io::Error::other("missing filename"))?,
                )?;
                atomic_write(&parent, &name, &protected, |file| {
                    file.write_all(bytes.as_bytes())
                })?;
            }
            sync_to_filesystem(&parent)?;
            if let std::collections::btree_map::Entry::Vacant(entry) =
                devices.entry(parent.metadata()?.dev())
            {
                entry.insert(parent.try_clone()?);
            }
        }
        // On macOS this performs F_FULLFSYNC: prior fsync'd data and metadata
        // on that device are durable when it succeeds (fcntl(2)). Linux has
        // already fsync'd each file/directory. Errors remain batch failures.
        for directory in devices.values() {
            barrier(directory)?;
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn p0_failed_write_preserves_old_file_and_removes_temporary() {
            let fixture = tempfile::tempdir().unwrap();
            let root = std::fs::canonicalize(fixture.path()).unwrap();
            std::fs::write(root.join("note.md"), b"old").unwrap();
            let dir = open_root(&root, false).unwrap().unwrap();
            let err = atomic_write(&dir, c"note.md", &[], |file| {
                file.write_all(b"partial")?;
                Err(io::Error::other("injected write failure"))
            })
            .unwrap_err();
            assert!(err.to_string().contains("injected"));
            assert_eq!(std::fs::read(root.join("note.md")).unwrap(), b"old");
            assert_eq!(std::fs::read_dir(root).unwrap().count(), 1);
        }

        #[test]
        fn p0_failed_rename_preserves_old_file_and_removes_temporary() {
            let fixture = tempfile::tempdir().unwrap();
            let root = std::fs::canonicalize(fixture.path()).unwrap();
            std::fs::write(root.join("note.md"), b"old").unwrap();
            let dir = open_root(&root, false).unwrap().unwrap();
            let result = atomic_write_with(
                &dir,
                c"note.md",
                &[],
                |file| file.write_all(b"new"),
                |_| Err(io::Error::other("injected rename failure")),
            );
            assert!(result.unwrap_err().to_string().contains("rename failure"));
            assert_eq!(std::fs::read(root.join("note.md")).unwrap(), b"old");
            assert_eq!(std::fs::read_dir(root).unwrap().count(), 1);
        }

        #[test]
        fn followup_final_durability_error_is_not_reported_as_success() {
            let fixture = tempfile::tempdir().unwrap();
            let root = std::fs::canonicalize(fixture.path()).unwrap();
            let storage = Storage::open_in_memory().unwrap();
            let batch = vec![(PathBuf::from("note.md"), "complete file".into())];
            let result = write_batch_with_barrier(&root, &batch, &storage, |_| {
                Err(io::Error::other("injected final durability failure"))
            });
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("durability failure"));
            assert_eq!(
                std::fs::read_to_string(root.join("note.md")).unwrap(),
                "complete file"
            );
        }

        #[test]
        fn p0_directory_swap_to_symlink_cannot_redirect_write() {
            let fixture = tempfile::tempdir().unwrap();
            let root = std::fs::canonicalize(fixture.path()).unwrap();
            std::fs::create_dir(root.join("owned")).unwrap();
            std::fs::create_dir(root.join("outside")).unwrap();
            let dir = open_root(&root.join("owned"), false).unwrap().unwrap();
            std::fs::rename(root.join("owned"), root.join("pinned")).unwrap();
            std::os::unix::fs::symlink(root.join("outside"), root.join("owned")).unwrap();
            atomic_write(&dir, c"note.md", &[], |f| f.write_all(b"safe")).unwrap();
            assert_eq!(std::fs::read_dir(root.join("outside")).unwrap().count(), 0);
            assert_eq!(std::fs::read(root.join("pinned/note.md")).unwrap(), b"safe");
        }

        #[cfg(target_os = "macos")]
        #[test]
        fn p0_review_macos_acl_is_rejected_despite_private_mode_bits() {
            use std::os::unix::fs::PermissionsExt;
            let fixture = tempfile::tempdir().unwrap();
            let root = std::fs::canonicalize(fixture.path()).unwrap();
            let status = std::process::Command::new("/bin/chmod")
                .args([
                    "+a",
                    "everyone allow add_file,delete_child,file_inherit,directory_inherit",
                ])
                .arg(&root)
                .status()
                .unwrap();
            assert!(status.success(), "ACL fixture must be created");
            assert_eq!(
                std::fs::metadata(&root).unwrap().permissions().mode() & 0o022,
                0
            );
            assert!(
                open_root(&root, false).is_err(),
                "mode 0700 does not exclude macOS ACL grants"
            );
        }

        #[cfg(target_os = "linux")]
        #[test]
        fn followup_linux_default_acl_is_rejected_despite_private_mode_bits() {
            let fixture = tempfile::tempdir().unwrap();
            let root = std::fs::canonicalize(fixture.path()).unwrap();
            let dir = open_root(&root, false).unwrap().unwrap();
            // Linux POSIX ACL xattr v2, including a named numeric user. A
            // default ACL does not change the directory's access mode bits.
            let mut value = 2u32.to_le_bytes().to_vec();
            for (tag, permissions, id) in [
                (0x01u16, 7u16, u32::MAX),
                (0x02, 7, 1_000_001),
                (0x04, 0, u32::MAX),
                (0x10, 7, u32::MAX),
                (0x20, 0, u32::MAX),
            ] {
                value.extend(tag.to_le_bytes());
                value.extend(permissions.to_le_bytes());
                value.extend(id.to_le_bytes());
            }
            rustix::fs::fsetxattr(
                &dir,
                "system.posix_acl_default",
                &value,
                rustix::fs::XattrFlags::empty(),
            )
            .expect("Linux test filesystem must support the ACL fixture");
            assert_eq!(dir.metadata().unwrap().mode() & 0o022, 0);
            assert!(open_root(&root, false).is_err());
        }
    }
}

#[cfg(test)]
mod component_tests {
    use super::*;

    #[test]
    fn p0_components_reject_both_platforms_and_control_characters() {
        for name in [
            "",
            ".",
            "..",
            "../../x",
            "/absolute",
            "a/b",
            "a\\b",
            "C:drive",
            "C:\\root",
            "a\0b",
            "a\nb",
            "NUL",
            "COM1.txt",
            "trailing.",
            "trailing ",
        ] {
            assert!(validate_component(name).is_err(), "{name:?}");
        }
        for name in ["São-Paulo", "東京", "tag", "2026-11-01"] {
            assert!(validate_component(name).is_ok(), "{name:?}");
        }
    }
}
