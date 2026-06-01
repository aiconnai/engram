//! Markdown export handler — human-readable memory export.
//!
//! Exports memories as Markdown files with RFC 0004 canonical YAML frontmatter
//! and wiki-style `[[links]]` for browsing and version control.
//! Import handler supports review mode (dry-run) and confirm mode.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use std::str::FromStr;

use serde_json::{json, Value};

use super::HandlerContext;
use crate::storage::queries::compute_content_hash;

/// Extract tags from parsed frontmatter (`engram_tags_list` is a comma-joined
/// string produced by [`parse_frontmatter`]).
fn frontmatter_tags(fm: &HashMap<String, String>) -> Vec<String> {
    fm.get("engram_tags_list")
        .map(|c| {
            c.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Build a deserializable frontmatter payload for `create_memory` /
/// `update_memory`. `scope` is intentionally omitted (the bare `engram_scope`
/// string cannot be losslessly mapped to `MemoryScope`); it defaults on import.
fn import_payload(
    fm: &HashMap<String, String>,
    body: &str,
    include_workspace: bool,
) -> serde_json::Map<String, Value> {
    let mut obj = serde_json::Map::new();
    obj.insert("content".into(), json!(body.trim()));
    if include_workspace {
        let ws = fm
            .get("engram_workspace")
            .cloned()
            .unwrap_or_else(|| "default".to_string());
        obj.insert("workspace".into(), json!(ws));
    }
    if let Some(mt) = fm.get("engram_type") {
        if crate::types::MemoryType::from_str(mt).is_ok() {
            obj.insert("memory_type".into(), json!(mt));
        }
    }
    if let Some(imp) = fm
        .get("engram_importance")
        .and_then(|s| s.parse::<f64>().ok())
    {
        obj.insert("importance".into(), json!(imp));
    }
    obj
}

// ── Frontmatter parsing ───────────────────────────────────────────────────────

/// Parse YAML frontmatter from a Markdown file.
///
/// Returns a map of `engram_*` keys to their string values.
/// Non-`engram_` keys are ignored (Obsidian-safe).
/// Tag sequences (`engram_tags:` followed by `  - item` lines) are stored
/// under the special key `engram_tags_list` as a comma-separated string.
fn parse_frontmatter(content: &str) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    let lines: Vec<&str> = content.lines().collect();

    // Find opening ---
    if lines.is_empty() || lines[0].trim() != "---" {
        return map;
    }

    // Find closing ---
    let close = match lines[1..].iter().position(|l| l.trim() == "---") {
        Some(i) => i + 1, // offset by 1 because we sliced from [1..]
        None => return map,
    };

    let fm_lines = &lines[1..close];
    let mut i = 0;
    while i < fm_lines.len() {
        let line = fm_lines[i];
        if let Some(colon_pos) = line.find(": ") {
            let key = line[..colon_pos].trim();
            let value = line[colon_pos + 2..].trim();

            if !key.starts_with("engram_") {
                i += 1;
                continue;
            }

            if key == "engram_tags" && value.is_empty() {
                // Sequence mode — collect following `  - item` lines
                let mut tags: Vec<String> = Vec::new();
                i += 1;
                while i < fm_lines.len() {
                    let next = fm_lines[i];
                    if next.starts_with("  - ") {
                        tags.push(next[4..].trim().to_string());
                        i += 1;
                    } else {
                        break;
                    }
                }
                if !tags.is_empty() {
                    map.insert("engram_tags_list".to_string(), tags.join(","));
                }
                continue;
            }

            // Strip surrounding quotes
            let cleaned = value.trim_matches('"').to_string();
            map.insert(key.to_string(), cleaned);
        } else {
            // Handle `engram_tags:` without trailing space (value is empty on same line)
            let trimmed = line.trim();
            if trimmed == "engram_tags:" {
                let mut tags: Vec<String> = Vec::new();
                i += 1;
                while i < fm_lines.len() {
                    let next = fm_lines[i];
                    if next.starts_with("  - ") {
                        tags.push(next[4..].trim().to_string());
                        i += 1;
                    } else {
                        break;
                    }
                }
                if !tags.is_empty() {
                    map.insert("engram_tags_list".to_string(), tags.join(","));
                }
                continue;
            }
        }
        i += 1;
    }

    map
}

/// Extract the body (content after the closing `---` of frontmatter).
fn extract_body(content: &str) -> &str {
    if !content.starts_with("---") {
        return content;
    }
    // Skip past first ---\n
    let pos = content.find('\n').map(|p| p + 1).unwrap_or(content.len());
    // Find closing ---
    if let Some(rel) = content[pos..].find("\n---\n") {
        let body_start = pos + rel + 5; // skip \n---\n
        &content[body_start..]
    } else if let Some(rel) = content[pos..].find("\n---") {
        let after = pos + rel + 4;
        // after == content.len() → closing marker is at EOF, no body
        // after < content.len() → body follows immediately after "---"
        &content[after.min(content.len())..]
    } else {
        content
    }
}

// ── Import classification ─────────────────────────────────────────────────────

/// Status of a file during import review.
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
/// - `current_hash`: SHA-256 of the file body.
/// - `file_version`: `engram_version` from frontmatter.
/// - `force_version`: if true, version conflicts are treated as `PendingUpdate`.
fn classify_import_status(
    db_state: Option<(&str, i64)>,
    current_hash: &str,
    file_version: i64,
    force_version: bool,
) -> ImportStatus {
    match db_state {
        None => ImportStatus::New,
        Some((db_hash, db_version)) => {
            if current_hash == db_hash {
                return ImportStatus::InSync;
            }
            if db_version == file_version {
                ImportStatus::PendingUpdate
            } else if force_version {
                ImportStatus::PendingUpdate
            } else if db_version > file_version {
                // DB is ahead of the file — true conflict (file is stale)
                ImportStatus::Conflict(format!(
                    "DB version {} > file version {}",
                    db_version, file_version
                ))
            } else {
                // file is ahead of DB — normal pending update
                ImportStatus::PendingUpdate
            }
        }
    }
}

// ── Grouping helper ───────────────────────────────────────────────────────────

/// Compute the subdirectory for a memory given the grouping mode.
///
/// Returns `None` for flat mode (no subdir).
fn compute_file_subdir(group: &str, mem: &Value) -> Option<String> {
    match group {
        "flat" => None,
        "day" => {
            let created = mem.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
            // Take first 10 chars: YYYY-MM-DD
            Some(created.chars().take(10).collect())
        }
        "workspace" => {
            let ws = mem
                .get("workspace")
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            Some(ws.to_string())
        }
        "type" => {
            let t = mem
                .get("memory_type")
                .and_then(|v| v.as_str())
                .unwrap_or("note");
            Some(t.to_string())
        }
        "entity" => {
            let tags_str = mem.get("tags").and_then(|v| v.as_str()).unwrap_or("");
            let mut tags = parse_tags(tags_str);
            tags.sort(); // deterministic: first alphabetically
            Some(
                tags.into_iter()
                    .next()
                    .unwrap_or_else(|| "untagged".to_string()),
            )
        }
        _ => None,
    }
}

/// Export a workspace as a directory of Markdown files.
///
/// Params:
/// - `workspace` (string, required) — workspace to export
/// - `output_dir` (string, optional) — output directory
///   (default: `./engram-export/{workspace}/`)
/// - `include_links` (bool, optional, default true) — include
///   [[wiki links]] to related memories
/// - `group` (string, optional, default `"flat"`) — grouping mode:
///   `"flat"` | `"day"` | `"workspace"` | `"type"` | `"entity"`
pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
    let workspace = match params.get("workspace").and_then(|v| v.as_str()) {
        Some(w) => w.to_string(),
        None => return json!({"error": "workspace is required"}),
    };

    let default_dir = format!("./engram-export/{}", workspace);
    let output_dir = params
        .get("output_dir")
        .and_then(|v| v.as_str())
        .unwrap_or(&default_dir);
    let output_path = PathBuf::from(output_dir);

    let include_links = params
        .get("include_links")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let group = params
        .get("group")
        .and_then(|v| v.as_str())
        .unwrap_or("flat")
        .to_string();

    // 1. Query all memories in workspace
    let memories = match query_workspace_memories(ctx, &workspace) {
        Ok(m) => m,
        Err(e) => return json!({"error": format!("Failed to query memories: {}", e)}),
    };

    if memories.is_empty() {
        return json!({
            "error": format!("No memories found in workspace '{}'", workspace),
            "files_written": 0
        });
    }

    // 2. If include_links, query cross-references for all memory IDs
    let related_map: HashMap<i64, Vec<(i64, String)>> = if include_links {
        let memory_ids: Vec<i64> = memories
            .iter()
            .filter_map(|m| m.get("id").and_then(|v| v.as_i64()))
            .collect();
        build_related_map(ctx, &memory_ids)
    } else {
        HashMap::new()
    };

    // 3. Create root output directory
    if let Err(e) = fs::create_dir_all(&output_path) {
        return json!({"error": format!("Failed to create output directory: {}", e)});
    }

    // First pass: compute filenames (stem only, no subdir)
    let id_to_filename: HashMap<i64, String> = memories
        .iter()
        .filter_map(|mem| {
            let id = mem.get("id").and_then(|v| v.as_i64())?;
            let content = mem.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let title = content.lines().next().unwrap_or("untitled");
            let sanitized = sanitize_filename(title);
            Some((id, format!("{}-{}", id, sanitized)))
        })
        .collect();

    let mut type_counts: HashMap<String, usize> = HashMap::new();
    for mem in &memories {
        let mem_type = mem
            .get("memory_type")
            .and_then(|v| v.as_str())
            .unwrap_or("note");
        *type_counts.entry(mem_type.to_string()).or_insert(0) += 1;
    }

    // Second pass: write files
    let mut files_written: usize = 0;
    for mem in &memories {
        let id = mem.get("id").and_then(|v| v.as_i64()).unwrap_or(0);

        // Determine target directory based on grouping mode
        let target_dir = match compute_file_subdir(&group, mem) {
            Some(subdir) => {
                let d = output_path.join(&subdir);
                if let Err(e) = fs::create_dir_all(&d) {
                    return json!({
                        "error": format!("Failed to create directory {}: {}", d.display(), e)
                    });
                }
                d
            }
            None => output_path.clone(),
        };

        let filename = id_to_filename
            .get(&id)
            .cloned()
            .unwrap_or_else(|| format!("{}", id));
        let file_path = target_dir.join(format!("{}.md", filename));

        let md = format_memory_markdown(mem, include_links, &related_map, &id_to_filename);

        if let Err(e) = fs::write(&file_path, &md) {
            return json!({"error": format!("Failed to write {}: {}", file_path.display(), e)});
        }
        files_written += 1;
    }

    // 4. Write index.md in root
    let index_path = output_path.join("index.md");
    let index = build_index_markdown(&workspace, &memories, &type_counts, &id_to_filename);

    if let Err(e) = fs::write(&index_path, &index) {
        return json!({"error": format!("Failed to write index: {}", e)});
    }

    json!({
        "files_written": files_written + 1,
        "output_dir": output_path.to_string_lossy(),
        "index_path": index_path.to_string_lossy(),
        "memories_exported": memories.len(),
        "type_breakdown": type_counts,
        "group": group
    })
}

/// Import memories from a directory of Markdown files with `engram_` frontmatter.
///
/// Params:
/// - `input_dir` (string, required) — directory to scan for `.md` files
/// - `workspace` (string, optional) — override workspace (falls back to file frontmatter)
/// - `confirm` (bool, optional, default false) — if false, dry-run only
/// - `force_version` (bool, optional, default false) — bypass version conflict check
pub fn memory_import_markdown(ctx: &HandlerContext, params: Value) -> Value {
    let input_dir_raw = match params.get("input_dir").and_then(|v| v.as_str()) {
        Some(d) => d.to_string(),
        None => return json!({"error": "input_dir is required"}),
    };
    let input_dir = match std::fs::canonicalize(&input_dir_raw) {
        Ok(p) if p.is_dir() => p.to_string_lossy().to_string(),
        Ok(_) => return json!({"error": "input_dir is not a directory"}),
        Err(e) => return json!({"error": format!("input_dir invalid: {}", e)}),
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
            files_detail.push(json!({
                "file": filename,
                "engram_id": null,
                "status": "skipped",
                "reason": "no valid engram_id in frontmatter"
            }));
            continue;
        }

        let file_version: i64 = fm
            .get("engram_version")
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);

        let current_hash = compute_content_hash(body.trim());

        // Look up in DB
        let db_state_result: Result<Option<(String, i64)>, crate::error::EngramError> =
            ctx.storage.with_connection(|conn| {
                let result = conn.query_row(
                    "SELECT content_hash, version FROM memories WHERE id = ?1",
                    rusqlite::params![engram_id],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
                );
                match result {
                    Ok(row) => Ok(Some(row)),
                    Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
                    Err(e) => Err(crate::error::EngramError::Database(e)),
                }
            });

        let db_state = match db_state_result {
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

        let status = classify_import_status(
            db_state.as_ref().map(|(h, v)| (h.as_str(), *v)),
            &current_hash,
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
                    // Build input via the query layer (single source of schema
                    // truth: handles scope_type/scope_id, tier, tags, hash).
                    let mut obj = import_payload(&fm, &body, true);
                    if let Some(ws) = workspace_override.clone() {
                        obj.insert("workspace".into(), json!(ws));
                    }
                    let tags = frontmatter_tags(&fm);
                    if !tags.is_empty() {
                        obj.insert("tags".into(), json!(tags));
                    }
                    let create_result = serde_json::from_value::<crate::types::CreateMemoryInput>(
                        Value::Object(obj),
                    )
                    .map_err(|e| {
                        crate::error::EngramError::Internal(format!("bad import payload: {e}"))
                    })
                    .and_then(|input| {
                        ctx.storage.with_transaction(|conn| {
                            crate::storage::queries::create_memory(conn, &input)
                        })
                    });

                    match create_result {
                        Ok(new_mem) => {
                            applied += 1;
                            let inserted_id = new_mem.id;
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
                    // Restore content, memory_type, importance, and tags from the
                    // file via update_memory (recomputes content_hash, bumps version,
                    // re-syncs tags). scope/workspace left to update_memory defaults.
                    let mut obj = import_payload(&fm, &body, false);
                    let tags = frontmatter_tags(&fm);
                    if !tags.is_empty() {
                        obj.insert("tags".into(), json!(tags));
                    } else if fm.contains_key("engram_tags_list") {
                        // Tag key present but empty — explicit wipe; warn before applying.
                        eprintln!(
                            "[markdown_export] import: wiping all tags for memory {} (engram_tags_list is empty in file {})",
                            engram_id, filename
                        );
                        obj.insert("tags".into(), json!([]));
                    }
                    // If engram_tags_list is absent from frontmatter entirely, tags
                    // field is omitted from the payload so update_memory leaves them
                    // untouched (no-op).
                    let update_result = serde_json::from_value::<crate::types::UpdateMemoryInput>(
                        Value::Object(obj),
                    )
                    .map_err(|e| {
                        crate::error::EngramError::Internal(format!("bad import payload: {e}"))
                    })
                    .and_then(|input| {
                        ctx.storage.with_transaction(|conn| {
                            crate::storage::queries::update_memory(conn, engram_id, &input)
                        })
                    });

                    match update_result {
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
        "pending_updates": count_pending,
        "conflicts": count_conflict,
        "applied": applied,
        "files": files_detail
    })
}

/// Recursively collect all `.md` files under `dir`.
fn collect_md_files(dir: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut result = Vec::new();
    collect_md_files_inner(&PathBuf::from(dir), &mut result)?;
    Ok(result)
}

fn collect_md_files_inner(dir: &PathBuf, out: &mut Vec<PathBuf>) -> Result<(), std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_md_files_inner(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            out.push(path);
        }
    }
    Ok(())
}

/// Query all active memories in a workspace.
fn query_workspace_memories(
    ctx: &HandlerContext,
    workspace: &str,
) -> Result<Vec<Value>, crate::error::EngramError> {
    ctx.storage.with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT m.id, m.content, m.memory_type, m.importance, m.workspace, m.tier,
                    m.created_at, m.updated_at,
                    (SELECT GROUP_CONCAT(t.name, ',')
                     FROM memory_tags mt
                     JOIN tags t ON mt.tag_id = t.id
                     WHERE mt.memory_id = m.id) as tags,
                    m.scope, m.version, m.metadata, m.content_hash
             FROM memories m
             WHERE m.workspace = ?1
               AND COALESCE(m.lifecycle_state, 'active') != 'archived'
               AND m.valid_to IS NULL
             ORDER BY m.memory_type, m.created_at",
        )?;
        let rows = stmt.query_map(rusqlite::params![workspace], |row| {
            let metadata_str: Option<String> = row.get(11)?;
            let metadata: Value = metadata_str
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or(Value::Null);
            Ok(json!({
                "id": row.get::<_, i64>(0)?,
                "content": row.get::<_, String>(1)?,
                "memory_type": row.get::<_, String>(2)?,
                "importance": row.get::<_, Option<f64>>(3)?,
                "workspace": row.get::<_, String>(4)?,
                "tier": row.get::<_, Option<String>>(5)?,
                "created_at": row.get::<_, String>(6)?,
                "updated_at": row.get::<_, Option<String>>(7)?,
                "tags": row.get::<_, Option<String>>(8)?,
                "scope": row.get::<_, Option<String>>(9)?,
                "version": row.get::<_, Option<i64>>(10)?,
                "metadata": metadata,
                "content_hash": row.get::<_, Option<String>>(12)?
            }))
        })?;
        let memories: Vec<Value> = rows.filter_map(|r| r.ok()).collect();
        Ok(memories)
    })
}

/// Build a map of memory_id -> [(related_id, relation_type)].
///
/// Issues a single batched query over all `memory_ids` instead of one per ID.
/// DB errors are logged and result in an empty map rather than being silently
/// discarded.
fn build_related_map(ctx: &HandlerContext, memory_ids: &[i64]) -> HashMap<i64, Vec<(i64, String)>> {
    if memory_ids.is_empty() {
        return HashMap::new();
    }

    // Build a single IN-list query for all IDs at once.
    let placeholders: Vec<String> = (1..=memory_ids.len()).map(|i| format!("?{i}")).collect();
    let in_list = placeholders.join(", ");
    let sql = format!(
        "SELECT from_id, to_id, relation_type FROM cross_references
          WHERE from_id IN ({in_list}) OR to_id IN ({in_list})"
    );

    let result = ctx.storage.with_connection(|conn| {
        let mut stmt = conn.prepare(&sql)?;
        // Build params: each id appears twice (once for from_id IN, once for to_id IN).
        let doubled: Vec<i64> = memory_ids
            .iter()
            .chain(memory_ids.iter())
            .copied()
            .collect();
        let rows: Vec<(i64, i64, String)> = stmt
            .query_map(rusqlite::params_from_iter(doubled.iter()), |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(crate::error::EngramError::Database)?;
        Ok(rows)
    });

    match result {
        Ok(rows) => {
            let id_set: std::collections::HashSet<i64> = memory_ids.iter().copied().collect();
            let mut map: HashMap<i64, Vec<(i64, String)>> = HashMap::new();
            for (from_id, to_id, rel_type) in rows {
                if id_set.contains(&from_id) {
                    map.entry(from_id)
                        .or_default()
                        .push((to_id, rel_type.clone()));
                }
                if id_set.contains(&to_id) && to_id != from_id {
                    map.entry(to_id).or_default().push((from_id, rel_type));
                }
            }
            map
        }
        Err(e) => {
            eprintln!("[markdown_export] build_related_map DB error: {e}");
            HashMap::new()
        }
    }
}

/// Format a single memory as Markdown with YAML frontmatter.
fn format_memory_markdown(
    mem: &Value,
    include_links: bool,
    related_map: &HashMap<i64, Vec<(i64, String)>>,
    id_to_filename: &HashMap<i64, String>,
) -> String {
    let id = mem.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let mem_type = mem
        .get("memory_type")
        .and_then(|v| v.as_str())
        .unwrap_or("note");
    let content = mem.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let tags_str = mem.get("tags").and_then(|v| v.as_str()).unwrap_or("");
    let importance = mem.get("importance").and_then(|v| v.as_f64());
    let tier = mem
        .get("tier")
        .and_then(|v| v.as_str())
        .unwrap_or("permanent");
    let created = mem.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let updated = mem.get("updated_at").and_then(|v| v.as_str());

    let tags_vec = parse_tags(tags_str);
    let scope = mem.get("scope").and_then(|v| v.as_str()).unwrap_or("user");
    let workspace = mem
        .get("workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let version = mem.get("version").and_then(|v| v.as_i64()).unwrap_or(1);
    let metadata = mem.get("metadata");
    let source_session = metadata
        .and_then(|m| m.get("source_session"))
        .and_then(|v| v.as_str());

    let hash = compute_content_hash(content.trim());

    let mut md = String::new();

    // RFC 0004 canonical YAML frontmatter — engram_ prefix on all fields
    md.push_str("---\n");
    md.push_str(&format!("engram_id: {}\n", id));
    md.push_str(&format!("engram_workspace: {}\n", workspace));
    md.push_str(&format!("engram_scope: {}\n", scope));
    md.push_str(&format!("engram_type: {}\n", mem_type));
    md.push_str(&format!("engram_created_at: \"{}\"\n", created));
    if let Some(upd) = updated {
        md.push_str(&format!("engram_updated_at: \"{}\"\n", upd));
    } else {
        md.push_str(&format!("engram_updated_at: \"{}\"\n", created));
    }
    md.push_str(&format!("engram_content_hash: {}\n", hash));
    md.push_str(&format!("engram_version: {}\n", version));
    if let Some(imp) = importance {
        md.push_str(&format!("engram_importance: {}\n", imp));
    } else {
        md.push_str("engram_importance: 0.5\n");
    }
    // Tags: YAML sequence format
    if tags_vec.is_empty() {
        md.push_str("engram_tags: []\n");
    } else {
        md.push_str("engram_tags:\n");
        for tag in &tags_vec {
            md.push_str(&format!("  - {}\n", tag));
        }
    }
    md.push_str(&format!("engram_tier: {}\n", tier));
    // source_session: only emit when present
    if let Some(sess) = source_session {
        md.push_str(&format!("engram_source_session: {}\n", sess));
    }
    md.push_str("---\n\n");

    // Content
    md.push_str(content);
    md.push('\n');

    // Related memories as [[wiki links]]
    if include_links {
        if let Some(related) = related_map.get(&id) {
            if !related.is_empty() {
                md.push_str("\n## Related\n\n");
                for (related_id, relation_type) in related {
                    let linked_name = id_to_filename
                        .get(related_id)
                        .cloned()
                        .unwrap_or_else(|| format!("memory-{}", related_id));
                    md.push_str(&format!("- {} [[{}]]\n", relation_type, linked_name));
                }
            }
        }
    }

    md
}

/// Build an index.md with a summary table of all exported memories.
fn build_index_markdown(
    workspace: &str,
    memories: &[Value],
    type_counts: &HashMap<String, usize>,
    id_to_filename: &HashMap<i64, String>,
) -> String {
    let mut index = String::new();
    index.push_str(&format!("# {} -- Engram Export\n\n", workspace));
    index.push_str(&format!("**Total memories:** {}\n\n", memories.len()));
    index.push_str("## By Type\n\n");

    let mut sorted_types: Vec<_> = type_counts.iter().collect();
    sorted_types.sort_by(|a, b| b.1.cmp(a.1));
    for (mem_type, count) in &sorted_types {
        index.push_str(&format!(
            "- **{}/** -- {} memories\n",
            pluralize_type(mem_type),
            count
        ));
    }

    index.push_str("\n## All Memories\n\n");
    index.push_str("| ID | Type | Title | Tags |\n");
    index.push_str("|-----|------|-------|------|\n");
    for mem in memories {
        let id = mem.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        let mem_type = mem
            .get("memory_type")
            .and_then(|v| v.as_str())
            .unwrap_or("note");
        let content = mem.get("content").and_then(|v| v.as_str()).unwrap_or("");
        let title: String = content
            .chars()
            .take(60)
            .collect::<String>()
            .replace('|', "\\|")
            .replace('\n', " ");
        let tags_str = mem.get("tags").and_then(|v| v.as_str()).unwrap_or("");
        let filename = id_to_filename.get(&id).cloned().unwrap_or_default();
        index.push_str(&format!(
            "| {} | {} | [{}]({}/{}.md) | {} |\n",
            id,
            mem_type,
            title,
            pluralize_type(mem_type),
            filename,
            tags_str
        ));
    }

    index
}

/// Parse tags from either comma-separated or JSON array format.
fn parse_tags(tags_str: &str) -> Vec<String> {
    if tags_str.is_empty() {
        return Vec::new();
    }
    if tags_str.starts_with('[') {
        serde_json::from_str(tags_str).unwrap_or_default()
    } else {
        tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }
}

/// Sanitize a string for use as a filename.
fn sanitize_filename(s: &str) -> String {
    let cleaned: String = s
        .chars()
        .take(40)
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = cleaned.trim_matches('-').to_lowercase();
    if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        trimmed
    }
}

/// Pluralize a memory type for directory naming.
fn pluralize_type(mem_type: &str) -> String {
    match mem_type {
        "summary" => "summaries".to_string(),
        s if s.ends_with('s') => s.to_string(),
        s => format!("{}s", s),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── sanitize_filename ──────────────────────────────────────────────

    #[test]
    fn test_sanitize_filename_basic() {
        assert_eq!(sanitize_filename("Hello World!"), "hello-world");
    }

    #[test]
    fn test_sanitize_filename_preserves_alphanumeric() {
        assert_eq!(sanitize_filename("my-note_123"), "my-note_123");
    }

    #[test]
    fn test_sanitize_filename_empty_input() {
        assert_eq!(sanitize_filename(""), "untitled");
    }

    #[test]
    fn test_sanitize_filename_all_special_chars() {
        assert_eq!(sanitize_filename("!!!@@@"), "untitled");
    }

    #[test]
    fn test_sanitize_filename_truncates_long_input() {
        let long_input = "a".repeat(100);
        let result = sanitize_filename(&long_input);
        assert!(result.len() <= 40);
    }

    #[test]
    fn test_sanitize_filename_trims_dashes() {
        assert_eq!(sanitize_filename("  hello  "), "hello");
    }

    // ── pluralize_type ─────────────────────────────────────────────────

    #[test]
    fn test_pluralize_type_note() {
        assert_eq!(pluralize_type("note"), "notes");
    }

    #[test]
    fn test_pluralize_type_todo() {
        assert_eq!(pluralize_type("todo"), "todos");
    }

    #[test]
    fn test_pluralize_type_summary() {
        assert_eq!(pluralize_type("summary"), "summaries");
    }

    #[test]
    fn test_pluralize_type_already_plural() {
        assert_eq!(pluralize_type("issues"), "issues");
    }

    // ── parse_tags ─────────────────────────────────────────────────────

    #[test]
    fn test_parse_tags_empty() {
        assert!(parse_tags("").is_empty());
    }

    #[test]
    fn test_parse_tags_comma_separated() {
        assert_eq!(
            parse_tags("rust, memory, test"),
            vec!["rust", "memory", "test"]
        );
    }

    #[test]
    fn test_parse_tags_json_array() {
        assert_eq!(parse_tags(r#"["alpha","beta"]"#), vec!["alpha", "beta"]);
    }

    // ── format_memory_markdown ─────────────────────────────────────────

    #[test]
    fn test_format_memory_markdown_basic() {
        let mem = json!({
            "id": 1,
            "content": "Hello world",
            "memory_type": "note",
            "scope": "user",
            "workspace": "default",
            "tags": "rust,test",
            "importance": 0.8,
            "tier": "permanent",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-02T00:00:00Z",
            "version": 1,
            "metadata": null
        });

        let related_map = HashMap::new();
        let id_to_filename = HashMap::new();
        let md = format_memory_markdown(&mem, false, &related_map, &id_to_filename);

        assert!(md.starts_with("---\n"));
        assert!(md.contains("engram_id: 1"));
        assert!(md.contains("engram_type: note"));
        assert!(md.contains("  - rust"));
        assert!(md.contains("  - test"));
        assert!(md.contains("engram_importance: 0.8"));
        assert!(md.contains("engram_tier: permanent"));
        assert!(md.contains("Hello world"));
    }

    #[test]
    fn test_format_memory_markdown_with_links() {
        let mem = json!({
            "id": 1,
            "content": "Memory one",
            "memory_type": "note",
            "scope": "user",
            "workspace": "default",
            "tags": "",
            "importance": null,
            "tier": "permanent",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": null,
            "version": 1,
            "metadata": null
        });

        let mut related_map = HashMap::new();
        related_map.insert(1i64, vec![(2i64, "related".to_string())]);

        let mut id_to_filename = HashMap::new();
        id_to_filename.insert(2i64, "2-memory-two".to_string());

        let md = format_memory_markdown(&mem, true, &related_map, &id_to_filename);

        assert!(md.contains("## Related"));
        assert!(md.contains("- related [[2-memory-two]]"));
    }

    #[test]
    fn test_format_memory_markdown_no_links_section_when_empty() {
        let mem = json!({
            "id": 1,
            "content": "Solo memory",
            "memory_type": "note",
            "scope": "user",
            "workspace": "default",
            "tags": "",
            "importance": null,
            "tier": "permanent",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": null,
            "version": 1,
            "metadata": null
        });

        let related_map = HashMap::new();
        let id_to_filename = HashMap::new();
        let md = format_memory_markdown(&mem, true, &related_map, &id_to_filename);

        assert!(!md.contains("## Related"));
    }

    // ── build_index_markdown ───────────────────────────────────────────

    #[test]
    fn test_build_index_markdown_header() {
        let memories = vec![json!({
            "id": 1,
            "content": "Test note",
            "memory_type": "note",
            "tags": "",
        })];
        let mut type_counts = HashMap::new();
        type_counts.insert("note".to_string(), 1);
        let mut id_to_filename = HashMap::new();
        id_to_filename.insert(1i64, "1-test-note".to_string());

        let index = build_index_markdown("mywork", &memories, &type_counts, &id_to_filename);

        assert!(index.contains("# mywork -- Engram Export"));
        assert!(index.contains("**Total memories:** 1"));
        assert!(index.contains("**notes/**"));
        assert!(index.contains("| 1 | note |"));
    }

    // ── memory_export_markdown (requires HandlerContext — skip here) ──
    // Integration tests are in tests/markdown_export.rs

    // ── RFC 0004 canonical frontmatter ────────────────────────────────

    #[test]
    fn test_format_memory_markdown_canonical_frontmatter() {
        let mem = json!({
            "id": 42,
            "content": "Authentication is required for every request",
            "memory_type": "note",
            "scope": "user",
            "workspace": "default",
            "tags": "rust,architecture",
            "importance": 0.8,
            "tier": "permanent",
            "created_at": "2026-05-31T02:07:00Z",
            "updated_at": "2026-05-31T04:00:00Z",
            "version": 3,
            "metadata": {"source_session": "sess_abc"}
        });
        let related_map = HashMap::new();
        let id_to_filename = HashMap::new();
        let md = format_memory_markdown(&mem, false, &related_map, &id_to_filename);

        // All 12 engram_ keys must be present
        assert!(md.contains("engram_id: 42"), "missing engram_id");
        assert!(
            md.contains("engram_workspace: default"),
            "missing engram_workspace"
        );
        assert!(md.contains("engram_scope: user"), "missing engram_scope");
        assert!(md.contains("engram_type: note"), "missing engram_type");
        assert!(
            md.contains("engram_created_at:"),
            "missing engram_created_at"
        );
        assert!(
            md.contains("engram_updated_at:"),
            "missing engram_updated_at"
        );
        assert!(
            md.contains("engram_content_hash: sha256:"),
            "missing engram_content_hash"
        );
        assert!(md.contains("engram_version: 3"), "missing engram_version");
        assert!(
            md.contains("engram_importance: 0.8"),
            "missing engram_importance"
        );
        assert!(md.contains("engram_tier: permanent"), "missing engram_tier");
        assert!(
            md.contains("engram_source_session: sess_abc"),
            "missing engram_source_session"
        );
        // Tags must be YAML sequence format
        assert!(md.contains("engram_tags:"), "missing engram_tags key");
        assert!(md.contains("  - rust"), "missing tag sequence item rust");
        assert!(
            md.contains("  - architecture"),
            "missing tag sequence item architecture"
        );
        // Old-style keys must be absent
        assert!(!md.contains("\nid: "), "old id: key must be removed");
        assert!(!md.contains("\ntype: "), "old type: key must be removed");
    }

    #[test]
    fn test_format_memory_markdown_no_source_session() {
        let mem = json!({
            "id": 10,
            "content": "No session here",
            "memory_type": "note",
            "scope": "global",
            "workspace": "work",
            "tags": "",
            "importance": 0.5,
            "tier": "daily",
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": null,
            "version": 1,
            "metadata": null
        });
        let related_map = HashMap::new();
        let id_to_filename = HashMap::new();
        let md = format_memory_markdown(&mem, false, &related_map, &id_to_filename);

        assert!(
            !md.contains("engram_source_session"),
            "source_session must be absent when not in metadata"
        );
        assert!(md.contains("engram_id: 10"), "engram_id must be present");
    }

    // ── content_hash helper ────────────────────────────────────────────

    #[test]
    fn test_content_hash_prefix() {
        let h = compute_content_hash("hello");
        assert!(h.starts_with("sha256:"), "hash must have sha256: prefix");
        assert_eq!(h.len(), 7 + 64, "sha256 hex is 64 chars");
    }

    #[test]
    fn test_content_hash_deterministic() {
        assert_eq!(compute_content_hash("abc"), compute_content_hash("abc"));
        assert_ne!(compute_content_hash("abc"), compute_content_hash("xyz"));
    }

    // ── parse_frontmatter helper ───────────────────────────────────────

    #[test]
    fn test_parse_frontmatter_extracts_engram_keys() {
        let content = "---\nengram_id: 42\nengram_type: note\naliases: [foo]\n---\nBody here";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.get("engram_id").map(|s| s.as_str()), Some("42"));
        assert_eq!(fm.get("engram_type").map(|s| s.as_str()), Some("note"));
        // Non-engram_ keys must be ignored
        assert!(
            !fm.contains_key("aliases"),
            "non-engram_ keys must be ignored"
        );
    }

    #[test]
    fn test_parse_frontmatter_tags_sequence() {
        let content = "---\nengram_tags:\n  - rust\n  - arch\n---\nBody";
        let fm = parse_frontmatter(content);
        assert_eq!(
            fm.get("engram_tags_list").map(|s| s.as_str()),
            Some("rust,arch")
        );
    }

    #[test]
    fn test_parse_frontmatter_body_extraction() {
        let content = "---\nengram_id: 1\n---\nHello world\nSecond line";
        let body = extract_body(content);
        assert_eq!(body.trim(), "Hello world\nSecond line");
    }

    // ── classify_import_status helper ─────────────────────────────────

    #[test]
    fn test_classify_import_status_in_sync() {
        // same hash and same version → in_sync
        let status = classify_import_status(Some(("sha256:abc", 3)), "sha256:abc", 3, false);
        assert_eq!(status, ImportStatus::InSync);
    }

    #[test]
    fn test_classify_import_status_new() {
        // ID not in DB → new
        let status = classify_import_status(None, "sha256:abc", 1, false);
        assert_eq!(status, ImportStatus::New);
    }

    #[test]
    fn test_classify_import_status_pending_update() {
        // different hash, same version → pending_update
        let status = classify_import_status(Some(("sha256:old", 3)), "sha256:new", 3, false);
        assert_eq!(status, ImportStatus::PendingUpdate);
    }

    #[test]
    fn test_classify_import_status_conflict_blocked() {
        // version mismatch, no force → conflict
        let status = classify_import_status(Some(("sha256:old", 5)), "sha256:new", 3, false);
        assert_eq!(
            status,
            ImportStatus::Conflict("DB version 5 > file version 3".to_string())
        );
    }

    #[test]
    fn test_classify_import_status_force_version_applies() {
        // version mismatch + force → pending_update
        let status = classify_import_status(Some(("sha256:old", 5)), "sha256:new", 3, true);
        assert_eq!(status, ImportStatus::PendingUpdate);
    }

    // ── compute_file_subdir helper (grouping) ─────────────────────────

    #[test]
    fn test_grouping_flat() {
        let mem = json!({
            "memory_type": "note",
            "created_at": "2026-05-31T02:07:00Z",
            "workspace": "default",
            "tags": "rust,arch"
        });
        let subdir = compute_file_subdir("flat", &mem);
        assert!(subdir.is_none(), "flat mode: no subdir");
    }

    #[test]
    fn test_grouping_by_type() {
        let mem = json!({
            "memory_type": "decision",
            "created_at": "2026-05-31T02:07:00Z",
            "workspace": "default",
            "tags": ""
        });
        let subdir = compute_file_subdir("type", &mem);
        assert_eq!(subdir.as_deref(), Some("decision"));
    }

    #[test]
    fn test_grouping_by_day() {
        let mem = json!({
            "memory_type": "note",
            "created_at": "2026-05-31T02:07:00Z",
            "workspace": "default",
            "tags": ""
        });
        let subdir = compute_file_subdir("day", &mem);
        assert_eq!(subdir.as_deref(), Some("2026-05-31"));
    }

    #[test]
    fn test_grouping_by_workspace() {
        let mem = json!({
            "memory_type": "note",
            "created_at": "2026-05-31T02:07:00Z",
            "workspace": "myproject",
            "tags": ""
        });
        let subdir = compute_file_subdir("workspace", &mem);
        assert_eq!(subdir.as_deref(), Some("myproject"));
    }

    #[test]
    fn test_grouping_by_entity_first_tag() {
        let mem = json!({
            "memory_type": "note",
            "created_at": "2026-05-31T02:07:00Z",
            "workspace": "default",
            "tags": "rust,arch"
        });
        let subdir = compute_file_subdir("entity", &mem);
        // sorted alphabetically: "arch" < "rust"
        assert_eq!(subdir.as_deref(), Some("arch"));
    }

    #[test]
    fn test_grouping_by_entity_no_tags() {
        let mem = json!({
            "memory_type": "note",
            "created_at": "2026-05-31T02:07:00Z",
            "workspace": "default",
            "tags": ""
        });
        let subdir = compute_file_subdir("entity", &mem);
        assert_eq!(subdir.as_deref(), Some("untagged"));
    }

    // ── Import round-trip integration (confirm:true write paths) ───────────
    mod import_integration {
        use super::super::memory_import_markdown;
        use crate::storage::queries::create_memory;
        use crate::storage::Storage;
        use crate::types::{CreateMemoryInput, MemoryType};
        use serde_json::json;
        use std::fs;
        use std::sync::Arc;

        fn ctx() -> crate::mcp::handlers::HandlerContext {
            crate::mcp::handlers::HandlerContext {
                storage: Storage::open_in_memory().expect("in-memory storage"),
                embedder: Arc::new(crate::embedding::TfIdfEmbedder::new(128)),
                fuzzy_engine: Arc::new(parking_lot::Mutex::new(crate::search::FuzzyEngine::new())),
                search_config: crate::search::SearchConfig::default(),
                realtime: None,
                embedding_cache: Arc::new(crate::embedding::EmbeddingCache::default()),
                search_cache: Arc::new(crate::search::SearchResultCache::new(
                    crate::search::AdaptiveCacheConfig::default(),
                )),
                #[cfg(feature = "meilisearch")]
                meili: None,
                #[cfg(feature = "meilisearch")]
                meili_indexer: None,
                #[cfg(feature = "meilisearch")]
                meili_sync_interval: 300,
                #[cfg(feature = "langfuse")]
                langfuse_runtime: Arc::new(
                    tokio::runtime::Builder::new_current_thread()
                        .build()
                        .unwrap(),
                ),
            }
        }

        fn make_memory(
            c: &crate::mcp::handlers::HandlerContext,
            content: &str,
            tags: &[&str],
        ) -> i64 {
            let input = CreateMemoryInput {
                content: content.to_string(),
                memory_type: MemoryType::Note,
                tags: tags.iter().map(|t| t.to_string()).collect(),
                importance: Some(0.5),
                workspace: Some("default".to_string()),
                ..Default::default()
            };
            c.storage
                .with_transaction(|conn| create_memory(conn, &input))
                .expect("create memory")
                .id
        }

        /// Write a markdown file with engram_ frontmatter into `dir`.
        #[allow(clippy::too_many_arguments)]
        fn write_md(
            dir: &std::path::Path,
            fname: &str,
            id: Option<i64>,
            version: i64,
            tags: &[&str],
            importance: f64,
            body: &str,
            extra_keys: &[(&str, &str)],
        ) {
            let mut fm = String::from("---\n");
            if let Some(id) = id {
                fm.push_str(&format!("engram_id: {}\n", id));
            }
            fm.push_str("engram_workspace: default\n");
            fm.push_str("engram_scope: user\n");
            fm.push_str("engram_type: note\n");
            fm.push_str(&format!("engram_version: {}\n", version));
            fm.push_str(&format!("engram_importance: {}\n", importance));
            for (k, v) in extra_keys {
                fm.push_str(&format!("{}: {}\n", k, v));
            }
            if !tags.is_empty() {
                fm.push_str("engram_tags:\n");
                for t in tags {
                    fm.push_str(&format!("  - {}\n", t));
                }
            }
            fm.push_str("---\n");
            fm.push_str(body);
            fm.push('\n');
            fs::write(dir.join(fname), fm).expect("write md");
        }

        fn db_row(c: &crate::mcp::handlers::HandlerContext, id: i64) -> (String, i64, f64) {
            c.storage
                .with_connection(|conn| {
                    conn.query_row(
                        "SELECT content, version, importance FROM memories WHERE id = ?1",
                        rusqlite::params![id],
                        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                    )
                    .map_err(crate::error::EngramError::Database)
                })
                .expect("db row")
        }

        fn db_tags(c: &crate::mcp::handlers::HandlerContext, id: i64) -> Vec<String> {
            c.storage
                .with_connection(|conn| {
                    let mut stmt = conn.prepare(
                        "SELECT t.name FROM memory_tags mt JOIN tags t ON mt.tag_id = t.id \
                         WHERE mt.memory_id = ?1 ORDER BY t.name",
                    )?;
                    let v = stmt
                        .query_map(rusqlite::params![id], |r| r.get::<_, String>(0))?
                        .filter_map(|r| r.ok())
                        .collect();
                    Ok(v)
                })
                .expect("db tags")
        }

        fn status_of<'a>(result: &'a serde_json::Value, id: i64) -> Option<&'a str> {
            result["files"].as_array()?.iter().find_map(|f| {
                if f["engram_id"].as_i64() == Some(id) {
                    f["status"].as_str()
                } else {
                    None
                }
            })
        }

        #[test]
        fn test_import_in_sync_when_body_matches() {
            let c = ctx();
            let id = make_memory(&c, "hello world", &["alpha"]);
            let dir = tempfile::tempdir().unwrap();
            write_md(
                dir.path(),
                "m.md",
                Some(id),
                1,
                &["alpha"],
                0.5,
                "hello world",
                &[],
            );

            let r = memory_import_markdown(&c, json!({"input_dir": dir.path().to_str().unwrap()}));
            assert_eq!(status_of(&r, id), Some("in_sync"), "result={}", r);
            assert_eq!(r["applied"].as_i64(), Some(0));
        }

        #[test]
        fn test_import_confirm_applies_update() {
            let c = ctx();
            let id = make_memory(&c, "original content", &["alpha"]);
            let dir = tempfile::tempdir().unwrap();
            // same version (1) as DB, changed body → pending_update
            write_md(
                dir.path(),
                "m.md",
                Some(id),
                1,
                &["alpha"],
                0.5,
                "edited content",
                &[],
            );

            // review mode: staged, no write
            let review =
                memory_import_markdown(&c, json!({"input_dir": dir.path().to_str().unwrap()}));
            assert_eq!(status_of(&review, id), Some("pending_update"));
            assert_eq!(review["applied"].as_i64(), Some(0));
            assert_eq!(
                db_row(&c, id).0,
                "original content",
                "review must not write"
            );

            // confirm: applies
            let applied = memory_import_markdown(
                &c,
                json!({"input_dir": dir.path().to_str().unwrap(), "confirm": true}),
            );
            assert_eq!(applied["applied"].as_i64(), Some(1));
            let (content, version, _) = db_row(&c, id);
            assert_eq!(content, "edited content");
            assert_eq!(version, 2, "version must increment");
        }

        #[test]
        fn test_import_confirm_resyncs_tags_and_importance() {
            let c = ctx();
            let id = make_memory(&c, "base", &["old"]);
            let dir = tempfile::tempdir().unwrap();
            write_md(
                dir.path(),
                "m.md",
                Some(id),
                1,
                &["new1", "new2"],
                0.9,
                "base edited",
                &[],
            );

            let r = memory_import_markdown(
                &c,
                json!({"input_dir": dir.path().to_str().unwrap(), "confirm": true}),
            );
            assert_eq!(r["applied"].as_i64(), Some(1));
            let tags = db_tags(&c, id);
            assert_eq!(
                tags,
                vec!["new1".to_string(), "new2".to_string()],
                "tags resynced"
            );
            let (_, _, importance) = db_row(&c, id);
            assert!(
                (importance - 0.9).abs() < 1e-6,
                "importance resynced: {}",
                importance
            );
        }

        #[test]
        fn test_import_conflict_blocks_then_force_applies() {
            let c = ctx();
            let id = make_memory(&c, "c", &[]);
            // bump DB version to 3 so the file (version 1) is stale
            c.storage
                .with_connection(|conn| {
                    conn.execute(
                        "UPDATE memories SET version = 3 WHERE id = ?1",
                        rusqlite::params![id],
                    )
                    .map_err(crate::error::EngramError::Database)
                })
                .unwrap();
            let dir = tempfile::tempdir().unwrap();
            write_md(dir.path(), "m.md", Some(id), 1, &[], 0.5, "stale edit", &[]);

            // no force → conflict, no write
            let blocked = memory_import_markdown(
                &c,
                json!({"input_dir": dir.path().to_str().unwrap(), "confirm": true}),
            );
            assert_eq!(status_of(&blocked, id), Some("conflict"));
            assert_eq!(blocked["applied"].as_i64(), Some(0));
            assert_eq!(db_row(&c, id).0, "c", "conflict must not write");

            // force → applies
            let forced = memory_import_markdown(
                &c,
                json!({"input_dir": dir.path().to_str().unwrap(), "confirm": true, "force_version": true}),
            );
            assert_eq!(forced["applied"].as_i64(), Some(1));
            assert_eq!(db_row(&c, id).0, "stale edit");
        }

        #[test]
        fn test_import_new_inserts_memory() {
            let c = ctx();
            let dir = tempfile::tempdir().unwrap();
            // engram_id not present in DB
            write_md(
                dir.path(),
                "new.md",
                Some(999_999),
                1,
                &["x"],
                0.5,
                "brand new memory",
                &[],
            );

            let review =
                memory_import_markdown(&c, json!({"input_dir": dir.path().to_str().unwrap()}));
            assert_eq!(status_of(&review, 999_999), Some("new"));
            assert_eq!(review["applied"].as_i64(), Some(0));

            let applied = memory_import_markdown(
                &c,
                json!({"input_dir": dir.path().to_str().unwrap(), "confirm": true}),
            );
            assert_eq!(applied["applied"].as_i64(), Some(1));
            // a memory with that content now exists (under a fresh autoincrement id)
            let count: i64 = c
                .storage
                .with_connection(|conn| {
                    conn.query_row(
                        "SELECT COUNT(*) FROM memories WHERE content = ?1",
                        rusqlite::params!["brand new memory"],
                        |r| r.get(0),
                    )
                    .map_err(crate::error::EngramError::Database)
                })
                .unwrap();
            assert_eq!(count, 1, "new memory inserted");
        }

        #[test]
        fn test_import_ignores_obsidian_keys() {
            let c = ctx();
            let id = make_memory(&c, "hello world", &["alpha"]);
            let dir = tempfile::tempdir().unwrap();
            // extra non-engram_ frontmatter keys must be ignored, not error
            write_md(
                dir.path(),
                "m.md",
                Some(id),
                1,
                &["alpha"],
                0.5,
                "hello world",
                &[("aliases", "[foo, bar]"), ("cssclasses", "note-card")],
            );

            let r = memory_import_markdown(&c, json!({"input_dir": dir.path().to_str().unwrap()}));
            assert_eq!(
                status_of(&r, id),
                Some("in_sync"),
                "obsidian keys ignored; result={}",
                r
            );
        }

        #[test]
        fn test_import_skips_file_without_engram_id() {
            let c = ctx();
            let dir = tempfile::tempdir().unwrap();
            write_md(
                dir.path(),
                "plain.md",
                None,
                1,
                &[],
                0.5,
                "just an obsidian note",
                &[],
            );

            let r = memory_import_markdown(&c, json!({"input_dir": dir.path().to_str().unwrap()}));
            let skipped = r["files"]
                .as_array()
                .unwrap()
                .iter()
                .any(|f| f["status"] == "skipped");
            assert!(
                skipped,
                "file without engram_id must be skipped; result={}",
                r
            );
            assert_eq!(r["applied"].as_i64(), Some(0));
        }
    }
}
