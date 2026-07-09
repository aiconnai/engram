use chrono::Utc;
use rusqlite::{params, Connection};

use crate::error::Result;

use super::helpers::{add_tag_to_json, sha256_hex};
use super::{UpdateAction, UpdateCandidate, UpdateResult};

// Apply update
// =============================================================================

/// Apply `action` to an existing memory and return the result.
///
/// The caller is responsible for passing the `new_content` that triggered
/// the update; it is used for `Replace` and `Merge` actions.
///
/// **Note:** this function does NOT write to `update_log` itself. Call
/// `create_update_log` separately so the caller controls reason text.
pub fn apply_update(
    conn: &Connection,
    candidate: &UpdateCandidate,
    action: UpdateAction,
    new_content: &str,
) -> Result<UpdateResult> {
    // Fetch current content.
    let (old_content, tags_json): (String, String) = conn.query_row(
        "SELECT content, tags FROM memories WHERE id = ?1",
        params![candidate.existing_id],
        |row| Ok((row.get(0)?, row.get(1).unwrap_or_else(|_| "[]".to_string()))),
    )?;

    let old_hash = sha256_hex(&old_content);

    let new_stored_content = match action {
        UpdateAction::Replace => new_content.to_string(),
        UpdateAction::Merge => format!("{}\n\n{}", old_content.trim(), new_content.trim()),
        UpdateAction::Archive => old_content.clone(),
        UpdateAction::Flag => old_content.clone(),
    };

    let new_hash = sha256_hex(&new_stored_content);

    match action {
        UpdateAction::Replace => {
            conn.execute(
                "UPDATE memories SET content = ?1, updated_at = ?2 WHERE id = ?3",
                params![
                    new_stored_content,
                    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    candidate.existing_id
                ],
            )?;
        }
        UpdateAction::Merge => {
            conn.execute(
                "UPDATE memories SET content = ?1, updated_at = ?2 WHERE id = ?3",
                params![
                    new_stored_content,
                    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    candidate.existing_id
                ],
            )?;
        }
        UpdateAction::Archive => {
            conn.execute(
                "UPDATE memories SET memory_type = 'archived', updated_at = ?1 WHERE id = ?2",
                params![
                    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    candidate.existing_id
                ],
            )?;
        }
        UpdateAction::Flag => {
            // Add 'needs-review' to the JSON tag array.
            let updated_tags = add_tag_to_json(&tags_json, "needs-review");
            conn.execute(
                "UPDATE memories SET tags = ?1, updated_at = ?2 WHERE id = ?3",
                params![
                    updated_tags,
                    Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string(),
                    candidate.existing_id
                ],
            )?;
        }
    }

    Ok(UpdateResult {
        memory_id: candidate.existing_id,
        action_taken: action,
        old_content_hash: old_hash,
        new_content_hash: new_hash,
    })
}

// =============================================================================
