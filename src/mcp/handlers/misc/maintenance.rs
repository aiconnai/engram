//! Maintenance handlers: rebuild embeddings and cross-references.

use serde_json::{json, Value};

use crate::mcp::handlers::HandlerContext;

pub fn memory_rebuild_embeddings(ctx: &HandlerContext, _params: Value) -> Value {
    use crate::storage::rebuild_embeddings;

    ctx.storage
        .with_connection(|conn| {
            let count = rebuild_embeddings(conn)?;
            Ok(json!({"rebuilt": count}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_rebuild_crossrefs(ctx: &HandlerContext, _params: Value) -> Value {
    use crate::storage::rebuild_crossrefs;

    ctx.storage
        .with_connection(|conn| {
            let count = rebuild_crossrefs(conn)?;
            Ok(json!({"rebuilt": count}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}
