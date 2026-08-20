//! Lifecycle tools (promote, checkpoint, boost, expiration, cleanup).
use serde_json::{json, Value};

use super::super::HandlerContext;

pub fn memory_promote_to_permanent(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::promote_to_permanent;

    let id = match params.get("id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "id is required"}),
    };

    ctx.storage
        .with_connection(|conn| {
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

    let context: HashMap<String, Value> = params
        .get("context")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    let workspace = params.get("workspace").and_then(|v| v.as_str());

    ctx.storage
        .with_connection(|conn| {
            let memory = create_checkpoint(conn, session_id, summary, &context, workspace)?;
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

    let boost_amount = params
        .get("boost_amount")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.2) as f32;
    let duration_seconds = params.get("duration_seconds").and_then(|v| v.as_i64());

    ctx.storage
        .with_connection(|conn| {
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

    let id = params.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
    let ttl_seconds = params.get("ttl_seconds").and_then(|v| v.as_i64());

    if ttl_seconds.is_none() {
        return json!({"error": "ttl_seconds is required"});
    }

    ctx.storage
        .with_transaction(|conn| {
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
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn cleanup_expired(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::{cleanup_expired_memories, count_expired_memories};

    let _ = params;

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
