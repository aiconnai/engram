//! Option E viability: trusted directory chain (OpenSSH-StrictModes style).
//! If every ancestor of the canonical database directory is owned by root or
//! the effective uid and is not writable by group/others (a sticky
//! world-writable ancestor is tolerated only when its next component is
//! owned by the euid), then no other non-root uid can rename, replace or add
//! entries anywhere on the path: the check-then-open race has no attacker.

use c4spike::trust::verify_trusted_chain;
use c4spike::util::*;

#[test]
fn private_directory_under_the_test_temp_dir_is_trusted() {
    let dir = fresh_dir("trust-ok");
    verify_trusted_chain(&dir).expect("trusted chain");
}

#[test]
fn group_writable_ancestor_is_rejected() {
    let root = fresh_dir("trust-g");
    let mid = root.join("shared");
    mkdir_private(&mid);
    chmod(&mid, 0o770);
    let leaf = mid.join("engram");
    mkdir_private(&leaf);
    assert!(verify_trusted_chain(&leaf).is_err());
}

#[test]
fn world_writable_non_sticky_ancestor_is_rejected() {
    let root = fresh_dir("trust-o");
    let mid = root.join("open");
    mkdir_private(&mid);
    chmod(&mid, 0o777);
    let leaf = mid.join("engram");
    mkdir_private(&leaf);
    assert!(verify_trusted_chain(&leaf).is_err());
}

#[test]
fn sticky_world_writable_ancestor_with_owned_child_is_trusted() {
    let root = fresh_dir("trust-sticky");
    let mid = root.join("tmp");
    mkdir_private(&mid);
    chmod(&mid, 0o1777);
    let leaf = mid.join("engram");
    mkdir_private(&leaf);
    verify_trusted_chain(&leaf).expect("sticky ancestor + owned child");
}

#[test]
fn database_directory_itself_must_not_be_writable_by_others() {
    let root = fresh_dir("trust-leaf");
    let leaf = root.join("engram");
    mkdir_private(&leaf);
    chmod(&leaf, 0o1777);
    assert!(verify_trusted_chain(&leaf).is_err());
}

#[test]
fn symlinked_component_is_rejected() {
    let root = fresh_dir("trust-link");
    let real = root.join("real");
    mkdir_private(&real);
    let link = root.join("link");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    assert!(verify_trusted_chain(&link).is_err());
}

// --- E part 2: the database and its sidecars themselves -------------------
// A trusted chain stops other uids from changing NAMES, but a file that was
// already in the directory (planted before the chain became trusted, or
// left from an earlier permissive configuration) can still be foreign-owned
// or permissive. E therefore also requires db/-wal/-shm/-journal to be
// regular files owned by the euid with mode & 077 == 0, checked by lstat
// before the open and re-checked by fstat on an opened handle.

use c4spike::trust::{verify_db_artifacts, verify_open_artifact};
use std::os::unix::fs::OpenOptionsExt;

fn touch(path: &std::path::Path, mode: u32) {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(mode)
        .open(path)
        .unwrap();
    chmod(path, mode); // defeat umask
}

#[test]
fn owner_only_artifacts_are_accepted_and_missing_sidecars_are_fine() {
    let dir = fresh_dir("art-ok");
    touch(&dir.join("engram.db"), 0o600);
    touch(&dir.join("engram.db-wal"), 0o600);
    verify_db_artifacts(&dir, "engram.db").expect("owner-only artifacts");
}

#[test]
fn a_planted_group_or_world_readable_sidecar_is_rejected() {
    let dir = fresh_dir("art-wal");
    touch(&dir.join("engram.db"), 0o600);
    touch(&dir.join("engram.db-wal"), 0o644);
    assert!(verify_db_artifacts(&dir, "engram.db").is_err());
}

#[test]
fn a_permissive_main_file_is_rejected() {
    let dir = fresh_dir("art-main");
    touch(&dir.join("engram.db"), 0o660);
    assert!(verify_db_artifacts(&dir, "engram.db").is_err());
}

#[test]
fn a_symlinked_or_non_regular_sidecar_is_rejected() {
    let dir = fresh_dir("art-link");
    touch(&dir.join("engram.db"), 0o600);
    touch(&dir.join("elsewhere"), 0o600);
    std::os::unix::fs::symlink(dir.join("elsewhere"), dir.join("engram.db-shm")).unwrap();
    assert!(verify_db_artifacts(&dir, "engram.db").is_err());
    let dir2 = fresh_dir("art-dir");
    touch(&dir2.join("engram.db"), 0o600);
    mkdir_private(&dir2.join("engram.db-journal"));
    assert!(verify_db_artifacts(&dir2, "engram.db").is_err());
}

#[test]
fn the_post_open_fstat_recheck_rejects_a_permissive_handle() {
    let dir = fresh_dir("art-fstat");
    let p = dir.join("engram.db");
    touch(&p, 0o600);
    let f = std::fs::File::open(&p).unwrap();
    verify_open_artifact(&f).expect("owner-only handle");
    chmod(&p, 0o644);
    assert!(verify_open_artifact(&f).is_err());
}
