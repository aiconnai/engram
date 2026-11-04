use crate::mcp::handlers::HandlerContext;
use crate::portability::markdown::{
    export::{export, ExportFailure, ExportRequest},
    MarkdownContext,
};
use serde_json::{json, Value};

pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
    if let Some(denial) = crate::mcp::permission::check_tool_authorization(
        None,
        "memory_export_markdown",
        &params,
        ctx.principal.as_ref(),
    ) {
        return denial;
    }
    let Some(workspace) = params.get("workspace").and_then(Value::as_str) else {
        return json!({"error":"workspace is required"});
    };
    let request = ExportRequest {
        workspace,
        output_dir: params.get("output_dir").and_then(Value::as_str),
        group: params
            .get("group")
            .and_then(Value::as_str)
            .unwrap_or("flat"),
        include_links: params
            .get("include_links")
            .and_then(Value::as_bool)
            .unwrap_or(true),
    };
    let context = MarkdownContext {
        storage: &ctx.storage,
        principal: ctx.principal.as_ref(),
    };
    match export(&context, request) {
        Ok(outcome) => json!(outcome),
        Err(error @ ExportFailure::EmptyWorkspace(_)) => {
            json!({"error":error.to_string(),"files_written":0})
        }
        Err(error) => json!({"error":error.to_string()}),
    }
}
