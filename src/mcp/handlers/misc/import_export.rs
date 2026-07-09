//! Memory export/import handlers.

use serde_json::{json, Value};

use crate::mcp::handlers::HandlerContext;

pub fn memory_export(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::export_memories;

    let workspace = params.get("workspace").and_then(|v| v.as_str());
    if params
        .get("include_embeddings")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return json!({"error": "include_embeddings is not supported yet for memory_export"});
    }

    ctx.storage
        .with_connection(|conn| {
            let data = export_memories(conn, workspace)?;
            Ok(json!(data))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_import(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::{import_memories, ExportData};

    let data: ExportData = match params
        .get("data")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
    {
        Some(d) => d,
        None => return json!({"error": "data object is required"}),
    };

    let skip_duplicates = params
        .get("skip_duplicates")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    ctx.storage
        .with_connection(|conn| {
            let result = import_memories(conn, &data, skip_duplicates)?;
            Ok(json!(result))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}
