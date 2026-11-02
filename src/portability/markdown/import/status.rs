#[derive(Debug, PartialEq)]
pub enum ImportStatus {
    New,
    InSync,
    PendingUpdate,
    Conflict(String),
}

/// Classify the import status of a file given DB state and file metadata.
///
/// - `db_state`: `Some((db_hash, db_version))` if ID exists in DB, `None` if not found.
/// - `current_hash`: dedupe-normalized SHA-256 of the file body.
/// - `file_version`: `engram_version` from frontmatter.
/// - `force_version`: if true, version conflicts are treated as `PendingUpdate`.
pub(super) fn classify_import_status(
    db_state: Option<(&str, i64)>,
    current_hash: &str,
    // Baseline hash to compare for InSync: frontmatter hash (raw, case-sensitive) when
    // available, otherwise the normalized DB content_hash for backward compat.
    sync_baseline: &str,
    file_version: i64,
    force_version: bool,
) -> ImportStatus {
    match db_state {
        None => ImportStatus::New,
        Some((_, db_version)) => {
            if current_hash == sync_baseline {
                // File content hasn't changed since export. But if the DB was updated
                // afterwards (db_version > file_version), the file is stale.
                // force_version lets callers override stale-file conflicts the same way
                // it overrides hash-mismatch conflicts.
                if db_version > file_version && !force_version {
                    return ImportStatus::Conflict(format!(
                        "DB version {} > file version {} (file unchanged, DB updated after export)",
                        db_version, file_version
                    ));
                }
                // force_version=true with stale file falls through to InSync — the
                // caller explicitly asked to not treat version skew as a blocker.
                return ImportStatus::InSync;
            }
            if db_version == file_version || force_version {
                ImportStatus::PendingUpdate
            } else if db_version > file_version {
                // DB is ahead of the file — true conflict (both sides changed)
                ImportStatus::Conflict(format!(
                    "DB version {} > file version {}",
                    db_version, file_version
                ))
            } else {
                // File is ahead of DB — RFC 0004 specifies conflict requiring explicit --force-version
                ImportStatus::Conflict(format!(
                    "File version {} > DB version {} (file version is ahead of DB)",
                    file_version, db_version
                ))
            }
        }
    }
}
