//! Lifecycle tools (promote, checkpoint, boost, expiration, cleanup).
use serde_json::{json, Value};

use super::super::HandlerContext;
use crate::mcp::error::ToolError;
use crate::mcp::workspace_guard::ensure_memory_access;

pub fn memory_promote_to_permanent(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::promote_to_permanent;

    let id = match params.get("id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "id is required"}),
    };

    ctx.storage
        .with_transaction(|conn| {
            ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
            let memory = promote_to_permanent(conn, id)?;
            Ok(json!({"success": true, "memory": memory}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_checkpoint(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::create_checkpoint;
    use std::collections::HashMap;

    let session_id = match params.get("session_id").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return json!({"error": "session_id is required"}),
    };

    let summary = match params.get("summary").and_then(|v| v.as_str()) {
        Some(s) => s,
        None => return json!({"error": "summary is required"}),
    };

    let context: HashMap<String, Value> = match params.get("context") {
        None | Some(Value::Null) => HashMap::new(),
        Some(Value::Object(obj)) => obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        Some(_) => return json!({"error": "context must be an object"}),
    };
    let workspace = match params.get("workspace") {
        None | Some(Value::Null) => None,
        Some(Value::String(ws)) => match crate::types::normalize_workspace(ws) {
            Ok(norm) => Some(norm),
            Err(e) => return json!({"error": format!("Invalid workspace: {e}")}),
        },
        Some(_) => return json!({"error": "workspace must be a string"}),
    };
    if let Some(ref p) = ctx.principal {
        let target_ws = workspace.as_deref().unwrap_or("default");
        if let Some(denial) = crate::mcp::permission::permission_denial_for_principal(
            "memory_checkpoint",
            p,
            Some(target_ws),
        ) {
            return denial;
        }
    }

    ctx.storage
        .with_transaction(|conn| {
            let memory =
                create_checkpoint(conn, session_id, summary, &context, workspace.as_deref())?;
            Ok(json!(memory))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_boost(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::boost_memory;

    let id = match params.get("id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "id is required"}),
    };

    let boost_amount = match params.get("boost_amount") {
        None | Some(Value::Null) => 0.2_f32,
        Some(v) => match v.as_f64() {
            Some(f) if f.is_finite() => f as f32,
            _ => return json!({"error": "boost_amount must be a finite number"}),
        },
    };
    let duration_seconds = match params.get("duration_seconds") {
        None | Some(Value::Null) => None,
        Some(v) => match v.as_i64() {
            Some(secs) => Some(secs),
            None => return json!({"error": "duration_seconds must be an integer"}),
        },
    };

    ctx.storage
        .with_transaction(|conn| {
            ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
            let mut memory = boost_memory(conn, id, boost_amount, duration_seconds)?;
            if let Ok(Some(new_stability)) =
                crate::intelligence::stability::record_reinforcement(conn, id, chrono::Utc::now())
            {
                memory.stability = new_stability;
            }
            Ok(json!(memory))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn set_expiration(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::set_memory_expiration;

    let id = match params.get("id") {
        None | Some(Value::Null) => return ToolError::missing_argument("id").into_value(),
        Some(v) => match v.as_i64() {
            Some(id) if id > 0 => id,
            _ => return ToolError::invalid_params("id must be a positive integer").into_value(),
        },
    };
    let ttl_seconds = match params.get("ttl_seconds") {
        None | Some(Value::Null) => {
            return ToolError::missing_argument("ttl_seconds").into_value();
        }
        Some(v) => match v.as_i64() {
            Some(ttl) => Some(ttl),
            None => {
                return ToolError::invalid_params("ttl_seconds must be an integer").into_value()
            }
        },
    };

    ctx.storage
        .with_transaction(|conn| {
            ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
            let memory = set_memory_expiration(conn, id, ttl_seconds)?;
            let message = if ttl_seconds == Some(0) {
                "Expiration removed".to_string()
            } else if let Some(ttl) = ttl_seconds {
                format!("Expiration set to {} seconds from now", ttl)
            } else {
                "Expiration updated".to_string()
            };
            Ok(json!({
                "success": true,
                "memory": memory,
                "message": message
            }))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

pub fn cleanup_expired(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::{cleanup_expired_memories, count_expired_memories};

    if let Some(denial) = crate::mcp::permission::check_tool_authorization(
        None,
        "memory_cleanup_expired",
        &params,
        ctx.principal.as_ref(),
    ) {
        return denial;
    }

    ctx.storage
        .with_transaction(|conn| {
            let _count_before = count_expired_memories(conn)?;
            let deleted = cleanup_expired_memories(conn)?;
            Ok(json!({
                "success": true,
                "deleted": deleted,
                "message": format!("Cleaned up {} expired memories", deleted)
            }))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}
