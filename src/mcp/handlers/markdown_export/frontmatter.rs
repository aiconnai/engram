use std::collections::HashMap;
use std::str::FromStr;

use serde_json::{json, Value};

pub(super) fn frontmatter_tags(fm: &HashMap<String, String>) -> Vec<String> {
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
pub(super) fn import_payload(
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
pub(super) fn parse_frontmatter(content: &str) -> HashMap<String, String> {
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
                    if let Some(item) = next.strip_prefix("  - ") {
                        tags.push(item.trim().to_string());
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
                    if let Some(item) = next.strip_prefix("  - ") {
                        tags.push(item.trim().to_string());
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
pub(super) fn extract_body(content: &str) -> &str {
    let body = if !content.starts_with("---") {
        content
    } else {
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
    };

    // Strip auto-generated `## Related` or `## Related Memories` footer if present
    if let Some(pos) = body.find("\n## Related\n") {
        &body[..pos]
    } else if let Some(pos) = body.find("\n## Related Memories\n") {
        &body[..pos]
    } else {
        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
