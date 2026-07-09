//! Tag utility handlers: list, hierarchy, and validation.

use serde_json::{json, Value};

use crate::mcp::handlers::HandlerContext;

pub fn memory_tags(ctx: &HandlerContext, _params: Value) -> Value {
    use crate::storage::list_tags;

    ctx.storage
        .with_connection(|conn| {
            let tags = list_tags(conn)?;
            Ok(json!({"tags": tags}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_tag_hierarchy(ctx: &HandlerContext, _params: Value) -> Value {
    use crate::storage::get_tag_hierarchy;

    ctx.storage
        .with_connection(|conn| {
            let hierarchy = get_tag_hierarchy(conn)?;
            Ok(json!({"hierarchy": hierarchy}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_validate_tags(ctx: &HandlerContext, _params: Value) -> Value {
    use crate::storage::validate_tags;

    ctx.storage
        .with_connection(|conn| {
            let result = validate_tags(conn)?;
            Ok(json!(result))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}
