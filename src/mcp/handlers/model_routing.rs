//! MCP handler for Model Routing Status (RFC 0011).

use serde_json::{json, Value};

use crate::mcp::handlers::HandlerContext;
use crate::routing::inspect_model_routing;
use crate::types::EmbeddingConfig;

/// Return the comprehensive model routing and capability matrix report.
pub fn model_routing_status(_ctx: &HandlerContext, params: Value) -> Value {
    let mut config = EmbeddingConfig::default();

    if let Some(model) = params.get("model").and_then(|v| v.as_str()) {
        config.model = model.to_string();
    }
    if let Some(emb_model) = params.get("embedding_model").and_then(|v| v.as_str()) {
        config.embedding_model = Some(emb_model.to_string());
    }
    if let Some(dims) = params.get("dimensions").and_then(|v| v.as_u64()) {
        config.dimensions = dims as usize;
    }

    let report = inspect_model_routing(&config);

    json!({
        "status": "success",
        "routing": report
    })
}
