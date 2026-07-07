use std::collections::HashMap;

use serde_json::{json, Value};

use crate::mcp::handlers::markdown_export::frontmatter::{frontmatter_tags, import_payload};
use crate::mcp::handlers::HandlerContext;

pub(super) fn create_memory_from_import(
    ctx: &HandlerContext,
    frontmatter: &HashMap<String, String>,
    body: &str,
    workspace_override: Option<&str>,
) -> Result<i64, crate::error::EngramError> {
    let mut obj = import_payload(frontmatter, body, true);
    if let Some(workspace) = workspace_override {
        obj.insert("workspace".into(), json!(workspace));
    }
    let tags = frontmatter_tags(frontmatter);
    if !tags.is_empty() {
        obj.insert("tags".into(), json!(tags));
    }

    serde_json::from_value::<crate::types::CreateMemoryInput>(Value::Object(obj))
        .map_err(|e| crate::error::EngramError::Internal(format!("bad import payload: {e}")))
        .and_then(|input| {
            ctx.storage
                .with_transaction(|conn| crate::storage::queries::create_memory(conn, &input))
        })
        .map(|new_memory| new_memory.id)
}

pub(super) fn update_memory_from_import(
    ctx: &HandlerContext,
    engram_id: i64,
    frontmatter: &HashMap<String, String>,
    body: &str,
    filename: &str,
) -> Result<(), crate::error::EngramError> {
    let mut obj = import_payload(frontmatter, body, false);
    let tags = frontmatter_tags(frontmatter);
    if !tags.is_empty() {
        obj.insert("tags".into(), json!(tags));
    } else if frontmatter.contains_key("engram_tags_list") {
        eprintln!(
            "[markdown_export] import: wiping all tags for memory {} (engram_tags_list is empty in file {})",
            engram_id, filename
        );
        obj.insert("tags".into(), json!([]));
    }

    serde_json::from_value::<crate::types::UpdateMemoryInput>(Value::Object(obj))
        .map_err(|e| crate::error::EngramError::Internal(format!("bad import payload: {e}")))
        .and_then(|input| {
            ctx.storage.with_transaction(|conn| {
                crate::storage::queries::update_memory(conn, engram_id, &input)
            })
        })
        .map(|_| ())
}
