use super::status::*;

#[test]
fn test_classify_import_status_in_sync() {
    // same hash and same version → in_sync
    let status = classify_import_status(
        Some(("sha256:abc", 3)),
        "sha256:abc",
        "sha256:abc",
        3,
        false,
    );
    assert_eq!(status, ImportStatus::InSync);
}

#[test]
fn test_classify_import_status_db_updated_after_export() {
    // File unchanged since export (hash matches baseline) but DB version is newer →
    // Conflict so the caller knows the file is stale.
    let status = classify_import_status(
        Some(("sha256:abc", 5)),
        "sha256:abc",
        "sha256:abc",
        3,
        false,
    );
    assert!(
        matches!(status, ImportStatus::Conflict(_)),
        "expected Conflict, got {:?}",
        status
    );
}

#[test]
fn test_classify_import_status_force_overrides_stale_db() {
    // force_version=true: stale-file conflict becomes InSync (caller opted in).
    let status =
        classify_import_status(Some(("sha256:abc", 5)), "sha256:abc", "sha256:abc", 3, true);
    assert_eq!(status, ImportStatus::InSync);
}

#[test]
fn test_classify_import_status_new() {
    // ID not in DB → new
    let status = classify_import_status(None, "sha256:abc", "sha256:abc", 1, false);
    assert_eq!(status, ImportStatus::New);
}

#[test]
fn test_classify_import_status_pending_update() {
    // different hash, same version → pending_update
    let status = classify_import_status(
        Some(("sha256:old", 3)),
        "sha256:new",
        "sha256:old",
        3,
        false,
    );
    assert_eq!(status, ImportStatus::PendingUpdate);
}

#[test]
fn test_classify_import_status_conflict_blocked() {
    // version mismatch, no force → conflict
    let status = classify_import_status(
        Some(("sha256:old", 5)),
        "sha256:new",
        "sha256:old",
        3,
        false,
    );
    assert_eq!(
        status,
        ImportStatus::Conflict("DB version 5 > file version 3".to_string())
    );
}

#[test]
fn test_classify_import_status_force_version_applies() {
    // version mismatch + force → pending_update
    let status =
        classify_import_status(Some(("sha256:old", 5)), "sha256:new", "sha256:old", 3, true);
    assert_eq!(status, ImportStatus::PendingUpdate);
}

#[test]
fn test_classify_import_status_case_only_edit_detected() {
    // Frontmatter hash = raw hash of "Hello World" (uppercase H)
    // File body was changed to "hello world" (lowercase) → different raw hashes → PendingUpdate
    use crate::storage::queries::compute_content_hash_raw;
    let original_hash = compute_content_hash_raw("Hello World");
    let edited_hash = compute_content_hash_raw("hello world");
    // They must differ (this is the whole point of the fix)
    assert_ne!(original_hash, edited_hash);
    // The DB has the normalized hash (simulates old behavior), frontmatter has raw hash
    let db_normalized = crate::storage::queries::compute_content_hash("Hello World");
    let status = classify_import_status(
        Some((db_normalized.as_str(), 1)),
        &edited_hash,
        &original_hash, // frontmatter hash (what was exported)
        1,
        false,
    );
    assert_eq!(status, ImportStatus::PendingUpdate);
}
