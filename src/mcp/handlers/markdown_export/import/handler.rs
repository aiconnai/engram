use std::fs;

use serde_json::{json, Value};

use super::apply::{create_memory_from_import, update_memory_from_import};
use super::files::collect_md_files;
use super::lookup::db_state_for_memory;
use super::status::{classify_import_status, ImportStatus};
use crate::mcp::handlers::markdown_export::frontmatter::{extract_body, parse_frontmatter};
use crate::mcp::handlers::markdown_export::validate_export_dir;
use crate::mcp::handlers::HandlerContext;
use crate::storage::queries::{compute_content_hash, compute_content_hash_raw};

pub fn memory_import_markdown(ctx: &HandlerContext, params: Value) -> Value {
    let input_dir_raw = match params.get("input_dir").and_then(|v| v.as_str()) {
        Some(d) => d.to_string(),
        None => return json!({"error": "input_dir is required"}),
    };

    let validated_input = match validate_export_dir(&input_dir_raw) {
        Ok(p) => p,
        Err(e) => return json!({"error": format!("Invalid input_dir: {}", e)}),
    };

    let input_dir = if validated_input.is_dir() {
        validated_input.to_string_lossy().to_string()
    } else {
        return json!({"error": "input_dir is not a directory"});
    };

    let workspace_override = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let confirm = params
        .get("confirm")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let force_version = params
        .get("force_version")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // Walk input_dir recursively for *.md files
    let md_files = match collect_md_files(&input_dir) {
        Ok(f) => f,
        Err(e) => return json!({"error": format!("Failed to walk input_dir: {}", e)}),
    };

    let scanned = md_files.len();
    let mut count_in_sync: usize = 0;
    let mut count_new: usize = 0;
    let mut count_pending: usize = 0;
    let mut count_conflict: usize = 0;
    let mut applied: usize = 0;
    let mut files_detail: Vec<Value> = Vec::new();

    for file_path in &md_files {
        if file_path.to_str().is_none() {
            continue;
        }
        let filename = file_path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("")
            .to_string();

        let raw = match fs::read_to_string(file_path) {
            Ok(s) => s,
            Err(e) => {
                files_detail.push(json!({
                    "file": filename,
                    "status": "error",
                    "reason": format!("read error: {}", e)
                }));
                continue;
            }
        };

        let fm = parse_frontmatter(&raw);
        let body = extract_body(&raw).to_string();

        let engram_id: i64 = fm
            .get("engram_id")
            .and_then(|s| s.parse().ok())
            .unwrap_or(-1);

        if engram_id == -1 {
            count_new += 1;
            if confirm {
                match create_memory_from_import(ctx, &fm, &body, workspace_override.as_deref()) {
                    Ok(inserted_id) => {
                        applied += 1;
                        files_detail.push(json!({
                            "file": filename,
                            "engram_id": inserted_id,
                            "status": "new",
                            "applied": true
                        }));
                    }
                    Err(e) => {
                        files_detail.push(json!({
                            "file": filename,
                            "engram_id": null,
                            "status": "error",
                            "reason": format!("insert error: {}", e)
                        }));
                    }
                }
            } else {
                files_detail.push(json!({
                    "file": filename,
                    "engram_id": null,
                    "status": "new"
                }));
            }
            continue;
        }

        let file_version: i64 = fm
            .get("engram_version")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        let db_state = match db_state_for_memory(ctx, engram_id) {
            Ok(s) => s,
            Err(e) => {
                files_detail.push(json!({
                    "file": filename,
                    "engram_id": engram_id,
                    "status": "error",
                    "reason": format!("DB lookup error: {}", e)
                }));
                continue;
            }
        };

        // When the file has an engram_content_hash frontmatter field (written by
        // memory_export_markdown since this fix), use case-sensitive (raw) hashes so
        // case-only edits are detected as PendingUpdate.
        // For files exported before this fix (no frontmatter hash), fall back to
        // normalized hash comparison against the DB hash for backward compat.
        let (current_hash, sync_baseline) = match fm.get("engram_content_hash") {
            Some(fm_hash) => (compute_content_hash_raw(body.trim()), fm_hash.clone()),
            None => {
                let norm = compute_content_hash(body.trim());
                let db_hash = db_state
                    .as_ref()
                    .map(|(h, _)| h.clone())
                    .unwrap_or_default();
                (norm, db_hash)
            }
        };

        let status = classify_import_status(
            db_state.as_ref().map(|(h, v)| (h.as_str(), *v)),
            &current_hash,
            &sync_baseline,
            file_version,
            force_version,
        );

        match &status {
            ImportStatus::InSync => {
                count_in_sync += 1;
                files_detail.push(json!({
                    "file": filename,
                    "engram_id": engram_id,
                    "status": "in_sync"
                }));
            }
            ImportStatus::New => {
                count_new += 1;
                if confirm {
                    match create_memory_from_import(ctx, &fm, &body, workspace_override.as_deref())
                    {
                        Ok(inserted_id) => {
                            applied += 1;
                            files_detail.push(json!({
                                "file": filename,
                                "engram_id": inserted_id,
                                "status": "new",
                                "applied": true
                            }));
                        }
                        Err(e) => {
                            files_detail.push(json!({
                                "file": filename,
                                "engram_id": engram_id,
                                "status": "error",
                                "reason": format!("insert error: {}", e)
                            }));
                        }
                    }
                } else {
                    files_detail.push(json!({
                        "file": filename,
                        "engram_id": engram_id,
                        "status": "new"
                    }));
                }
            }
            ImportStatus::PendingUpdate => {
                count_pending += 1;
                if confirm {
                    match update_memory_from_import(ctx, engram_id, &fm, &body, &filename) {
                        Ok(_) => {
                            applied += 1;
                            files_detail.push(json!({
                                "file": filename,
                                "engram_id": engram_id,
                                "status": "pending_update",
                                "applied": true
                            }));
                        }
                        Err(e) => {
                            files_detail.push(json!({
                                "file": filename,
                                "engram_id": engram_id,
                                "status": "error",
                                "reason": format!("update error: {}", e)
                            }));
                        }
                    }
                } else {
                    files_detail.push(json!({
                        "file": filename,
                        "engram_id": engram_id,
                        "status": "pending_update"
                    }));
                }
            }
            ImportStatus::Conflict(reason) => {
                count_conflict += 1;
                files_detail.push(json!({
                    "file": filename,
                    "engram_id": engram_id,
                    "status": "conflict",
                    "reason": reason
                }));
            }
        }
    }

    json!({
        "scanned": scanned,
        "in_sync": count_in_sync,
        "new": count_new,
        "pending": count_pending,
        "pending_updates": count_pending,
        "conflict": count_conflict,
        "conflicts": count_conflict,
        "applied": applied,
        "files": files_detail
    })
}
