use std::collections::HashMap;
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;

use axum::body::Body;
use axum::http::Request;
use axum::Router;
use serde_json::json;

use super::super::rate_limit::{RateLimiterConfig, RateLimiterState};
use super::super::router::build_router;
use crate::mcp::protocol::{McpHandler, McpRequest, McpResponse};

// Serialize CORS env var mutation in tests via a shared mutex.
pub(super) static ENV_LOCK: StdMutex<()> = StdMutex::new(());

struct TestMcpHandler;

impl McpHandler for TestMcpHandler {
    fn handle_request(&self, request: McpRequest) -> McpResponse {
        McpResponse::success(request.id, json!({"ok": true}))
    }
}

pub(super) fn json_rpc_request(path: &str, bearer: Option<&str>) -> Request<Body> {
    json_rpc_request_with_headers(path, bearer, &[])
}

pub(super) fn json_rpc_request_with_headers(
    path: &str,
    bearer: Option<&str>,
    headers: &[(&str, &str)],
) -> Request<Body> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list",
        "params": {}
    })
    .to_string();

    let mut builder = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json");

    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    for &(name, value) in headers {
        builder = builder.header(name, value);
    }

    builder.body(Body::from(body)).unwrap()
}

pub(super) fn json_rpc_notification_request_with_headers(
    path: &str,
    bearer: Option<&str>,
    headers: &[(&str, &str)],
) -> Request<Body> {
    let body = json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "params": {}
    })
    .to_string();

    let mut builder = Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json");

    if let Some(token) = bearer {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    for &(name, value) in headers {
        builder = builder.header(name, value);
    }

    builder.body(Body::from(body)).unwrap()
}

pub(super) fn test_app(api_key: Option<&str>) -> Router {
    test_app_with_rate_limits(api_key, 0, 0, None)
}

pub(super) fn test_app_with_rate_limits(
    api_key: Option<&str>,
    http_rate_limit_rps: u64,
    http_rate_limit_burst: u64,
    http_rate_limit_key: Option<&str>,
) -> Router {
    let _guard = ENV_LOCK.lock().unwrap();
    build_router(
        Arc::new(TestMcpHandler),
        api_key.map(str::to_string),
        None,
        http_rate_limit_rps,
        http_rate_limit_burst,
        http_rate_limit_key.map(str::to_string),
    )
}

pub(super) fn test_rate_limiter_state(
    max_buckets: usize,
    stale_after: Duration,
) -> RateLimiterState {
    RateLimiterState {
        config: RateLimiterConfig {
            requests_per_second: 0.0,
            burst: 1.0,
            key_header: None,
            max_buckets,
            stale_after,
        },
        buckets: HashMap::new(),
    }
}

// ---- check_bearer tests ------------------------------------------------
