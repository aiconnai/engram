//! Refuse extended/default ACLs rather than infer effective access from mode
//! bits. In particular, macOS ACL grants are independent of group mode bits.

use std::fs::File;
use std::io;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::os::fd::AsRawFd;

#[cfg(target_os = "macos")]
pub(super) fn check_directory(dir: &File) -> io::Result<()> {
    // Declarations/constants match the macOS SDK's sys/acl.h. ACL objects are
    // opaque and allocated/freed by libSystem; no Rust layout is assumed.
    unsafe extern "C" {
        fn acl_get_fd_np(fd: libc::c_int, kind: libc::c_int) -> *mut libc::c_void;
        fn acl_get_entry(
            acl: *mut libc::c_void,
            entry: libc::c_int,
            out: *mut *mut libc::c_void,
        ) -> libc::c_int;
        fn acl_free(object: *mut libc::c_void) -> libc::c_int;
    }
    const ACL_TYPE_EXTENDED: libc::c_int = 0x100;
    const ACL_FIRST_ENTRY: libc::c_int = 0;
    // SAFETY: dir is a live directory fd; the SDK defines this ACL type.
    let acl = unsafe { acl_get_fd_np(dir.as_raw_fd(), ACL_TYPE_EXTENDED) };
    if acl.is_null() {
        let error = io::Error::last_os_error();
        // macOS reports ENOENT when the vnode has no extended ACL. Other
        // errors, including unavailable inspection, do not grant trust.
        return if error.raw_os_error() == Some(libc::ENOENT) {
            Ok(())
        } else {
            Err(error)
        };
    }
    let mut entry = std::ptr::null_mut();
    // SAFETY: acl was returned by libSystem; entry points to writable pointer storage.
    let first = unsafe { acl_get_entry(acl, ACL_FIRST_ENTRY, &mut entry) };
    let result = if first == 0 {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "export directories must not have ACLs (including deny-only entries); use a private subdirectory without an ACL",
        ))
    } else {
        let error = io::Error::last_os_error();
        // On macOS an out-of-range first-entry index in an empty ACL is EINVAL
        // (acl_get_entry(3)); this differs from the POSIX/Linux return convention.
        if error.raw_os_error() == Some(libc::EINVAL) {
            Ok(())
        } else {
            Err(error)
        }
    };
    // SAFETY: this is the owned ACL pointer returned above, released exactly once.
    if unsafe { acl_free(acl) } < 0 {
        return Err(io::Error::last_os_error());
    }
    result
}

#[cfg(target_os = "linux")]
pub(super) fn check_directory(dir: &File) -> io::Result<()> {
    // Linux exposes POSIX access/default ACLs and extensions such as NFSv4
    // ACLs as extended attributes. Inspection errors are explicit, not a fallback.
    // listxattr may omit inaccessible names, so also probe known ACL namespaces
    // directly; permission errors must not be mistaken for absence.
    for name in [
        c"system.posix_acl_access",
        c"system.posix_acl_default",
        c"system.nfs4_acl",
        c"system.richacl",
        c"system.cifs_acl",
        c"security.NTACL",
    ] {
        // SAFETY: live directory fd, NUL-terminated name, size-only query.
        let size =
            unsafe { libc::fgetxattr(dir.as_raw_fd(), name.as_ptr(), std::ptr::null_mut(), 0) };
        if size >= 0 {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "export directory has an ACL attribute",
            ));
        }
        let error = io::Error::last_os_error();
        if !matches!(error.raw_os_error(), Some(libc::ENODATA | libc::ENOTSUP)) {
            return Err(error);
        }
    }
    // SAFETY: size-only query of a live directory; no destination buffer is supplied.
    let size = unsafe { libc::flistxattr(dir.as_raw_fd(), std::ptr::null_mut(), 0) };
    if size < 0 {
        return Err(io::Error::last_os_error());
    }
    if size == 0 {
        return Ok(());
    }
    if size > 65_536 {
        return Err(io::Error::other(
            "directory attribute list exceeds inspection limit",
        ));
    }
    let mut names = vec![0u8; size as usize];
    // SAFETY: names has size writable bytes and dir remains a live descriptor.
    let read = unsafe { libc::flistxattr(dir.as_raw_fd(), names.as_mut_ptr().cast(), names.len()) };
    if read < 0 {
        return Err(io::Error::last_os_error());
    }
    if read as usize > names.len() {
        return Err(io::Error::other("invalid directory attribute list size"));
    }
    names.truncate(read as usize);
    if !names.is_empty() && names.last() != Some(&0) {
        return Err(io::Error::other("unterminated directory attribute list"));
    }
    if names.split(|&b| b == 0).any(|name| {
        name.windows(3)
            .any(|part| part.eq_ignore_ascii_case(b"acl"))
    }) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "export directories must not have access, default or extended ACL attributes",
        ));
    }
    Ok(())
}
