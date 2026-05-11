//! MCP handlers for the Dream Phase.

use crate::dream::{run_once_all, DreamConfig};
use crate::mcp::handlers::HandlerContext;
use serde_json::{json, Value};

/// Manually trigger a Dream Phase pass across all workspaces.
pub fn dream_run_now(ctx: &HandlerContext, _params: Value) -> Value {
    // For now, use default config. In the future, we could allow overriding
    // consolidation parameters via params.
    let config = DreamConfig::default();

    let report = run_once_all(&ctx.storage, &config);

    json!({
        "status": "success",
        "report": report
    })
}
