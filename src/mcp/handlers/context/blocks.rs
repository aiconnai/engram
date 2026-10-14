//! Self-editing memory-block tools (Letta/MemGPT-style).
use serde_json::{json, Value};

use super::super::HandlerContext;
use crate::mcp::error::ToolError;

/// Get a memory block by name.
///
/// Params:
/// - `name` (string, required)
pub fn memory_block_get(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::memory_blocks::get_block;

    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return ToolError::missing_argument("name").into_value(),
    };

    ctx.storage
        .with_connection(|conn| match get_block(conn, &name)? {
            Some(block) => Ok(json!(block)),
            None => Ok(ToolError::not_found("block", &name).into_value()),
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

/// Edit (update) a memory block's content.
///
/// Params:
/// - `name` (string, required)
/// - `content` (string, required)
/// - `reason` (string, optional) — description of the edit
pub fn memory_block_edit(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::memory_blocks::update_block;

    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return ToolError::missing_argument("name").into_value(),
    };

    let content = match params.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return ToolError::missing_argument("content").into_value(),
    };

    let reason = params
        .get("reason")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    ctx.storage
        .with_connection(|conn| {
            let block = update_block(conn, &name, &content, &reason)?;
            Ok(json!(block))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

/// List all memory blocks.
pub fn memory_block_list(ctx: &HandlerContext, _params: Value) -> Value {
    use crate::storage::memory_blocks::list_blocks;

    ctx.storage
        .with_connection(|conn| {
            let blocks = list_blocks(conn)?;
            Ok(json!({"blocks": blocks, "count": blocks.len()}))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

/// Create a new memory block.
///
/// Params:
/// - `name` (string, required)
/// - `content` (string, optional, default: "")
/// - `max_tokens` (u64, optional, default: 4096)
pub fn memory_block_create(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::memory_blocks::create_block;

    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return ToolError::missing_argument("name").into_value(),
    };

    let content = params
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let max_tokens = params
        .get("max_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(4096) as usize;

    ctx.storage
        .with_connection(|conn| {
            let block = create_block(conn, &name, &content, max_tokens)?;
            Ok(json!(block))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

/// Archive a memory block (delete it and return the final content).
///
/// Params:
/// - `name` (string, required)
pub fn memory_block_archive(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::memory_blocks::{delete_block, get_block};

    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return ToolError::missing_argument("name").into_value(),
    };

    ctx.storage
        .with_connection(|conn| {
            let block = get_block(conn, &name)?;
            let final_content = block
                .as_ref()
                .map(|b| b.content.clone())
                .unwrap_or_default();
            let final_version = block.as_ref().map(|b| b.version).unwrap_or(0);

            if block.is_none() {
                return Ok(ToolError::not_found("block", &name).into_value());
            }

            delete_block(conn, &name)?;

            Ok(json!({
                "success": true,
                "name": name,
                "final_content": final_content,
                "final_version": final_version
            }))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

// ── Injection prompt ──────────────────────────────────────────────────────────

/// Get the edit history for a memory block.
///
/// Params:
/// - `name` (string, required)
/// - `limit` (u64, optional, default: 20)
pub fn memory_block_history(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::memory_blocks::get_block_history;

    let name = match params.get("name").and_then(|v| v.as_str()) {
        Some(n) => n.to_string(),
        None => return ToolError::missing_argument("name").into_value(),
    };

    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(20) as usize;

    ctx.storage
        .with_connection(|conn| {
            let history = get_block_history(conn, &name, limit)?;
            Ok(json!({"name": name, "history": history, "count": history.len()}))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}
