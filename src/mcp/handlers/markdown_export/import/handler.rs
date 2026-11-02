use serde_json::Value;

use super::apply::{import_batch, ImportFailure};
use super::files::read_markdown;
use crate::error::EngramError;
use crate::mcp::handlers::markdown_export::validate_export_dir;
use crate::mcp::handlers::HandlerContext;

pub fn memory_import_markdown(ctx: &HandlerContext, params: Value) -> Value {
    if let Some(denial) = crate::mcp::permission::check_tool_authorization(
        None,
        "memory_import_markdown",
        &params,
        ctx.principal.as_ref(),
    ) {
        return denial;
    }
    let outcome: std::result::Result<Value, ImportFailure> = (|| {
        let input = params
            .get("input_dir")
            .and_then(Value::as_str)
            .ok_or_else(|| EngramError::InvalidInput("input_dir is required".into()))?;
        let root = validate_export_dir(input).map_err(EngramError::InvalidInput)?;
        let sources = read_markdown(&root, &ctx.storage).map_err(|error| {
            EngramError::InvalidInput(format!("Markdown input refused: {error}"))
        })?;
        import_batch(
            ctx,
            sources,
            params.get("workspace").and_then(Value::as_str),
            params
                .get("force_version")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            params
                .get("confirm")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        )
    })();
    match outcome {
        Ok(value) => value,
        Err(error) => error.into_value(),
    }
}
