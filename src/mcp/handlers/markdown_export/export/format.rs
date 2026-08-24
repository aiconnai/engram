use std::collections::HashMap;

use serde_json::Value;

use crate::storage::queries::compute_content_hash_raw;

pub(super) fn compute_file_subdir(group: &str, mem: &Value) -> Option<String> {
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

pub(super) fn format_memory_markdown(
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

    // Raw (case-sensitive) hash written to frontmatter so the import sync check
    // can detect case-only edits as PendingUpdate rather than InSync.
    let hash = compute_content_hash_raw(content.trim());

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
pub(super) fn build_index_markdown(
    workspace: &str,
    memories: &[Value],
    type_counts: &HashMap<String, usize>,
    id_to_filename: &HashMap<i64, String>,
    group: &str,
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
        let rel_link = match compute_file_subdir(group, mem) {
            Some(subdir) => format!("{}/{}.md", subdir, filename),
            None => format!("{}.md", filename),
        };
        index.push_str(&format!(
            "| {} | {} | [{}]({}) | {} |\n",
            id, mem_type, title, rel_link, tags_str
        ));
    }

    index
}

/// Parse tags from either comma-separated or JSON array format.
pub(super) fn parse_tags(tags_str: &str) -> Vec<String> {
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
pub(super) fn sanitize_filename(s: &str) -> String {
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
pub(super) fn pluralize_type(mem_type: &str) -> String {
    match mem_type {
        "summary" => "summaries".to_string(),
        s if s.ends_with('s') => s.to_string(),
        s => format!("{}s", s),
    }
}
