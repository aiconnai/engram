//! Procedural/timeline retrieval tools.
use serde_json::{json, Value};

use super::super::HandlerContext;

pub fn memory_get_timeline(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::get_episodic_timeline;

    let start_time = params
        .get("start_time")
        .and_then(|v| v.as_str())
        .and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        });
    let end_time = params
        .get("end_time")
        .and_then(|v| v.as_str())
        .and_then(|s| {
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.with_timezone(&chrono::Utc))
        });
    let workspace = params.get("workspace").and_then(|v| v.as_str());
    let tags: Option<Vec<String>> = params.get("tags").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    });
    let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);

    ctx.storage
        .with_connection(|conn| {
            let memories = get_episodic_timeline(
                conn,
                start_time,
                end_time,
                workspace,
                tags.as_deref(),
                limit,
            )?;
            Ok(json!(memories))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_get_procedures(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::get_procedural_memories;

    let trigger_pattern = params.get("trigger_pattern").and_then(|v| v.as_str());
    let workspace = params.get("workspace").and_then(|v| v.as_str());
    let min_success_rate = params
        .get("min_success_rate")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32);
    let limit = params.get("limit").and_then(|v| v.as_i64()).unwrap_or(50);

    ctx.storage
        .with_connection(|conn| {
            let memories =
                get_procedural_memories(conn, trigger_pattern, workspace, min_success_rate, limit)?;
            Ok(json!(memories))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn record_procedure_outcome(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::record_procedure_outcome;

    let id = match params.get("id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "id is required"}),
    };
    let success = match params.get("success").and_then(|v| v.as_bool()) {
        Some(s) => s,
        None => return json!({"error": "success (boolean) is required"}),
    };

    ctx.storage
        .with_transaction(|conn| {
            let memory = record_procedure_outcome(conn, id, success)?;
            Ok(json!(memory))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}
