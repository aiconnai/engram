use crate::error::EngramError;
use crate::mcp::{error::ToolError, handlers::HandlerContext};
use crate::portability::markdown::{
    import::{import, ImportRequest},
    MarkdownContext,
};
use serde_json::{json, Value};

pub fn memory_import_markdown(ctx: &HandlerContext, params: Value) -> Value {
    if let Some(denial) = crate::mcp::permission::check_tool_authorization(
        None,
        "memory_import_markdown",
        &params,
        ctx.principal.as_ref(),
    ) {
        return denial;
    }
    let Some(input_dir) = params.get("input_dir").and_then(Value::as_str) else {
        let mut value =
            ToolError::from(EngramError::InvalidInput("input_dir is required".into())).into_value();
        value["applied"] = json!(0);
        value["outcome"] = json!("not_applied");
        return value;
    };
    let request = ImportRequest {
        input_dir,
        workspace: params.get("workspace").and_then(Value::as_str),
        force_version: params
            .get("force_version")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        confirm: params
            .get("confirm")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    };
    let context = MarkdownContext {
        storage: &ctx.storage,
        principal: ctx.principal.as_ref(),
    };
    match import(&context, request) {
        Ok(outcome) => {
            let pending = outcome.pending;
            let conflicts = outcome.conflict;
            let mut value = json!(outcome);
            value["pending_updates"] = json!(pending);
            value["conflicts"] = json!(conflicts);
            value
        }
        Err(error) => {
            let (error, attempted) = error.into_parts();
            if let Some(attempted) = attempted {
                let mut value = ToolError::internal(
                    "Import completion is uncertain; inspect the canonical database before retrying",
                ).into_value();
                value["applied"] = Value::Null;
                value["attempted"] = json!(attempted);
                value["outcome"] = json!("unknown");
                value
            } else {
                let mut value = ToolError::from(error).into_value();
                value["applied"] = json!(0);
                value["outcome"] = json!("not_applied");
                value
            }
        }
    }
}
