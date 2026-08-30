//! Model routing MCP handlers (RFC 0011).

use serde_json::{json, Value};
use std::str::FromStr;

use super::HandlerContext;
use crate::routing::{ModelPurpose, ModelRouter};

/// Resolve the active or preferred model route for a given AI capability / purpose.
pub fn model_route_resolve(_ctx: &HandlerContext, params: Value) -> Value {
    let purpose_str = match params.get("purpose").and_then(|v| v.as_str()) {
        Some(p) => p,
        None => return json!({"error": "purpose is required"}),
    };

    let purpose = match ModelPurpose::from_str(purpose_str) {
        Ok(p) => p,
        Err(e) => return json!({"error": e.to_string()}),
    };

    let preferred_provider = params.get("preferred_provider").and_then(|v| v.as_str());

    let router = ModelRouter::new();
    let resolution = router.resolve(purpose, preferred_provider);

    match serde_json::to_value(&resolution) {
        Ok(val) => val,
        Err(e) => json!({"error": format!("Failed to serialize route resolution: {}", e)}),
    }
}

/// List all declared model routes and their metadata.
pub fn model_routes_list(_ctx: &HandlerContext, params: Value) -> Value {
    let filter_purpose = if let Some(p_val) = params.get("purpose").and_then(|v| v.as_str()) {
        match ModelPurpose::from_str(p_val) {
            Ok(p) => Some(p),
            Err(e) => return json!({ "error": format!("Invalid purpose '{}': {}", p_val, e) }),
        }
    } else {
        None
    };

    let router = ModelRouter::new();
    let mut routes = router.list_routes();

    if let Some(target_purpose) = filter_purpose {
        routes.retain(|r| r.purpose == target_purpose);
    }

    json!({
        "routes_count": routes.len(),
        "routes": routes
    })
}

/// Inspect active model provider availability, embedding dimensions, and routing status.
pub fn model_routing_status(_ctx: &HandlerContext, params: Value) -> Value {
    let model = params
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("tfidf")
        .to_string();
    let embedding_model = params
        .get("embedding_model")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let dimensions = params
        .get("dimensions")
        .and_then(|v| v.as_u64())
        .unwrap_or(128) as usize;

    let config = crate::types::EmbeddingConfig {
        model,
        embedding_model,
        dimensions,
        api_key: std::env::var("OPENAI_API_KEY").ok(),
        base_url: None,
        model_path: None,
        batch_size: 100,
    };

    let report = crate::routing::inspect_model_routing(&config);
    json!({
        "status": "success",
        "routing": report,
    })
}
