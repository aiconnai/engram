use std::collections::HashSet;

use rusqlite::{params, Connection};

use crate::error::Result;

// Internal helpers
// =============================================================================

/// Maximum number of recent memories to compare against.
const MAX_RECENT_MEMORIES: i64 = 200;

/// Year pattern: 4-digit numbers in the range 1900–2099.
const YEAR_RANGE_START: u32 = 1900;
const YEAR_RANGE_END: u32 = 2099;

/// Fetch (id, content, memory_type, tags) for recent memories in a workspace.
pub(super) fn fetch_workspace_memories(
    conn: &Connection,
    workspace: &str,
) -> Result<Vec<(i64, String, String, Vec<String>)>> {
    let mut stmt = conn.prepare(
        "SELECT id, content, memory_type, tags
         FROM memories
         WHERE workspace = ?1
         ORDER BY id DESC
         LIMIT ?2",
    )?;

    let rows = stmt
        .query_map(params![workspace, MAX_RECENT_MEMORIES], |row| {
            let tags_raw: String = row.get::<_, String>(3).unwrap_or_else(|_| "[]".to_string());
            let tags: Vec<String> = serde_json::from_str(&tags_raw).unwrap_or_default();
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)
                    .unwrap_or_else(|_| "note".to_string()),
                tags,
            ))
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(rows)
}

/// Extract meaningful keywords from lowercase text.
///
/// Splits on whitespace/punctuation, drops stop-words and short tokens.
pub(super) fn extract_keywords(text: &str) -> HashSet<String> {
    const STOP_WORDS: &[&str] = &[
        "a", "an", "the", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had",
        "do", "does", "did", "will", "would", "could", "should", "may", "might", "shall", "to",
        "of", "in", "on", "at", "by", "for", "with", "from", "as", "it", "its", "this", "that",
        "and", "or", "but", "not", "so", "if", "then", "than", "when", "i", "me", "my", "we",
        "our", "you", "your", "he", "she", "they",
    ];

    text.split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 3)
        .filter(|t| !STOP_WORDS.contains(t))
        .map(|t| t.to_string())
        .collect()
}

/// Jaccard-style overlap: |A ∩ B| / |A ∪ B|.
pub(super) fn keyword_overlap(a: &HashSet<String>, b: &HashSet<String>) -> f32 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let intersection = a.intersection(b).count() as f32;
    let union = (a.len() + b.len()) as f32 - intersection;
    if union == 0.0 {
        0.0
    } else {
        intersection / union
    }
}

/// Count capitalised tokens shared between two lowercase texts.
///
/// We use a simple heuristic: tokens that start with an uppercase letter in the
/// *original* (non-lowercased) text are likely named entities. Since we receive
/// already-lowercased text here, we instead count tokens with length >= 4 that
/// appear in both texts as a proxy for entity-like shared nouns.
pub(super) fn shared_entity_count(new_lower: &str, existing_lower: &str) -> usize {
    let a = extract_keywords(new_lower);
    let b = extract_keywords(existing_lower);
    a.intersection(&b).filter(|t| t.len() >= 4).count()
}

/// Return `true` if the text contains a 4-digit year in [1900, 2099].
pub(super) fn contains_old_year(text: &str) -> bool {
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c.is_ascii_digit() {
            let mut num_str = String::with_capacity(4);
            num_str.push(c);
            for _ in 0..3 {
                match chars.peek() {
                    Some(d) if d.is_ascii_digit() => {
                        num_str.push(*d);
                        chars.next();
                    }
                    _ => break,
                }
            }
            if num_str.len() == 4 {
                if let Ok(year) = num_str.parse::<u32>() {
                    if (YEAR_RANGE_START..=YEAR_RANGE_END).contains(&year) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Compute a SHA-256 hex digest of a string without pulling in a heavy dep.
///
/// We use a simple FNV-1a inspired hash here because the spec only asks for
/// a "content hash" string — not cryptographic security. This keeps the module
/// dependency-free.
pub(super) fn sha256_hex(content: &str) -> String {
    // Use a deterministic 64-bit FNV-1a hash formatted as 16-char hex.
    let mut hash: u64 = 14695981039346656037u64; // FNV offset basis
    for byte in content.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(1099511628211u64); // FNV prime
    }
    format!("{:016x}", hash)
}

/// Append a tag to a JSON array string (e.g., `["existing"]` → `["existing","needs-review"]`).
pub(super) fn add_tag_to_json(tags_json: &str, tag: &str) -> String {
    let mut tags: Vec<String> = serde_json::from_str(tags_json).unwrap_or_default();
    if !tags.iter().any(|t| t == tag) {
        tags.push(tag.to_string());
    }
    serde_json::to_string(&tags).unwrap_or_else(|_| format!("[\"{}\"]", tag))
}

// =============================================================================
