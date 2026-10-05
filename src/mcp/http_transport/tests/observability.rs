//! O1 / issues #161-#162 follow-ups: the transport tells auth, transport,
//! protocol, tool and handler failures apart, carries a correlation id, counts
//! open SSE streams, and reports latency as a distribution.

use std::sync::Arc;
use std::time::Duration;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use super::support::{json_rpc_request, test_app_with_handler, test_app_with_limits};
use crate::mcp::http_transport::security_config::HttpSecurityConfig;
use crate::mcp::protocol::{McpHandler, McpRequest, McpResponse};
use crate::observability::test_capture::{captured, install};

/// Answers from a closure so a test chooses success, JSON-RPC error or tool error.
struct ScriptedHandler(Box<dyn Fn(McpRequest) -> McpResponse + Send + Sync>);

impl McpHandler for ScriptedHandler {
    fn handle_request(&self, request: McpRequest) -> McpResponse {
        (self.0)(request)
    }
}

fn scripted(f: impl Fn(McpRequest) -> McpResponse + Send + Sync + 'static) -> Arc<ScriptedHandler> {
    Arc::new(ScriptedHandler(Box::new(f)))
}

async fn health(app: &axum::Router) -> Value {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("health");
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&bytes).expect("health json")
}

fn http(health: &Value) -> &Value {
    health.pointer("/transport/http").expect("transport.http")
}

const KEY: &str = "observability-test-key";

/// `mcp_requests_total` is fully accounted for by exactly one of the buckets.
fn assert_balanced(h: &Value) {
    let n = |k: &str| h[k].as_u64().unwrap_or(u64::MAX);
    assert_eq!(
        n("mcp_requests_total"),
        n("mcp_requests_completed")
            + n("mcp_inflight_total")
            + n("mcp_abandoned_total")
            + n("mcp_timeouts_before_handler_total"),
        "unbalanced counters: {h}"
    );
}

fn tool_call(name: &str, request_id: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {KEY}"))
        .header("x-request-id", request_id)
        .body(Body::from(
            json!({
                "jsonrpc": "2.0", "id": 7, "method": "tools/call",
                "params": {"name": name, "arguments": {}}
            })
            .to_string(),
        ))
        .expect("request")
}

#[tokio::test]
async fn parse_errors_outside_the_handler_are_counted_with_unchanged_status() {
    install();
    let app = test_app_with_handler(
        scripted(|r| McpResponse::success(r.id, json!({}))),
        Some(KEY),
        0,
        0,
    );
    let bad = |body: &'static str, content_type: &'static str| {
        Request::builder()
            .method("POST")
            .uri("/mcp")
            .header("content-type", content_type)
            .header("authorization", format!("Bearer {KEY}"))
            .body(Body::from(body))
            .expect("request")
    };

    let malformed = app
        .clone()
        .oneshot(bad("{not json", "application/json"))
        .await
        .expect("response");
    let wrong_type = app
        .clone()
        .oneshot(bad("{}", "text/plain"))
        .await
        .expect("response");

    // Same statuses axum produced before the transport counted them.
    assert_eq!(malformed.status(), StatusCode::BAD_REQUEST);
    assert_eq!(wrong_type.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert!(malformed.headers().contains_key("x-request-id"));

    let report = health(&app).await;
    assert_eq!(http(&report)["mcp_outcomes"]["parse_error"], 2);
    assert_eq!(http(&report)["mcp_requests_total"], 2);
    assert_eq!(http(&report)["mcp_failed_total"], 2);
    assert_eq!(http(&report)["mcp_inflight_total"], 0);
}

#[tokio::test]
async fn mcp_errors_travelling_as_http_200_are_not_counted_as_success() {
    install();
    let app = test_app_with_handler(
        scripted(|r| match r.params["name"].as_str() {
            Some("proto_error") => McpResponse::error(r.id, -32602, "bad params".to_string()),
            Some("tool_error") => {
                McpResponse::success(r.id, json!({"isError": true, "content": []}))
            }
            _ => McpResponse::success(r.id, json!({"content": []})),
        }),
        Some(KEY),
        0,
        0,
    );

    for name in ["proto_error", "tool_error", "fine"] {
        let response = app
            .clone()
            .oneshot(tool_call(name, "corr-protocol-01"))
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK, "{name} stays HTTP 200");
    }

    let report = health(&app).await;
    let outcomes = &http(&report)["mcp_outcomes"];
    assert_eq!(outcomes["protocol_error"], 1);
    assert_eq!(outcomes["tool_error"], 1);
    assert_eq!(outcomes["success"], 1);
    assert_eq!(http(&report)["mcp_success_total"], 1);
    assert_eq!(http(&report)["mcp_failed_total"], 2);
}

#[tokio::test]
async fn correlation_id_and_bounded_method_labels_reach_response_and_log() {
    install();
    let app = test_app_with_handler(
        scripted(|r| McpResponse::success(r.id, json!({}))),
        Some(KEY),
        0,
        0,
    );

    let response = app
        .clone()
        .oneshot(tool_call("memory_search", "corr-test-0001"))
        .await
        .expect("response");
    assert_eq!(
        response
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok()),
        Some("corr-test-0001")
    );
    let unknown = app
        .clone()
        .oneshot(tool_call("tool-name-sentinel-from-caller", "bad id!"))
        .await
        .expect("response");
    let replaced = unknown
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_string();
    assert_eq!(
        replaced.len(),
        32,
        "an invalid id is replaced, got {replaced:?}"
    );

    let logs = captured();
    let line = logs
        .lines()
        .find(|l| l.contains("corr-test-0001"))
        .expect("request log line");
    for field in [
        "operation=\"http.mcp\"",
        "outcome=\"success\"",
        "method=\"tools/call\"",
        "tool=\"memory_search\"",
        "status=200",
        "route=\"/mcp\"",
        "duration_ms=",
    ] {
        assert!(line.contains(field), "missing {field} in {line}");
    }
    assert!(
        !logs.contains("tool-name-sentinel-from-caller"),
        "unknown tool names are bucketed, never logged"
    );
}

#[tokio::test]
async fn rate_limited_requests_log_retry_and_keep_the_429_contract() {
    install();
    let app = test_app_with_handler(
        scripted(|r| McpResponse::success(r.id, json!({}))),
        Some(KEY),
        100,
        1,
    );
    let make = || tool_call("memory_list", "corr-limited-01");

    assert_eq!(
        app.clone().oneshot(make()).await.unwrap().status(),
        StatusCode::OK
    );
    let limited = app.clone().oneshot(make()).await.expect("response");

    assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        limited
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok()),
        Some("1")
    );
    let body: Value = serde_json::from_slice(
        &to_bytes(limited.into_body(), usize::MAX)
            .await
            .expect("body"),
    )
    .expect("json");
    assert_eq!(body["error"]["code"], -32005);

    let logs = captured();
    let line = logs
        .lines()
        .filter(|l| l.contains("corr-limited-01"))
        .find(|l| l.contains("rate_limited"))
        .expect("rate-limited log line");
    assert!(
        line.contains("WARN") && line.contains("retry_after_secs=1"),
        "{line}"
    );
    assert_eq!(http(&health(&app).await)["mcp_outcomes"]["rate_limited"], 1);
}

#[tokio::test]
async fn latency_is_a_distribution_and_no_data_is_not_zero() {
    install();
    let app = test_app_with_handler(
        scripted(|r| McpResponse::success(r.id, json!({}))),
        Some(KEY),
        0,
        0,
    );

    let empty = health(&app).await;
    assert_eq!(http(&empty)["mcp_latency"]["all"]["samples"], 0);
    assert!(http(&empty)["mcp_latency"]["all"]["p95_ms"].is_null());

    for _ in 0..5 {
        app.clone()
            .oneshot(json_rpc_request("/mcp", Some(KEY)))
            .await
            .expect("response");
    }
    let report = health(&app).await;
    let all = &http(&report)["mcp_latency"]["all"];
    assert_eq!(all["samples"], 5);
    assert!(all["p50_ms"].is_u64() && all["p95_ms"].is_u64() && all["p99_ms"].is_u64());
    assert_eq!(all["buckets"].as_array().map(Vec::len), Some(12));
    assert_eq!(
        http(&report)["mcp_latency"]["by_outcome"]["success"]["samples"],
        5
    );
    assert_eq!(
        http(&report)["mcp_latency"]["by_outcome"]["tool_error"]["samples"],
        0
    );
}

#[tokio::test]
async fn body_over_the_limit_and_transport_timeout_are_distinct_outcomes() {
    install();
    let mut security = HttpSecurityConfig::default();
    security.max_body_bytes = 300;
    security.request_timeout = Duration::from_millis(50);
    let slow = scripted(|r| {
        if r.params["name"] == "slow" {
            std::thread::sleep(Duration::from_millis(400));
        }
        McpResponse::success(r.id, json!({}))
    });
    let app = test_app_with_limits(slow, Some(KEY), None, security, 0, 0);

    let too_big = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {KEY}"))
        .body(Body::from(
            json!({"jsonrpc":"2.0","id":1,"method":"x","params":{"pad":"x".repeat(600)}})
                .to_string(),
        ))
        .expect("request");
    let big = app.clone().oneshot(too_big).await.expect("response");
    assert_eq!(big.status(), StatusCode::PAYLOAD_TOO_LARGE);

    let timed_out = app
        .clone()
        .oneshot(tool_call("slow", "corr-timeout-01"))
        .await
        .expect("response");
    assert_eq!(timed_out.status(), StatusCode::REQUEST_TIMEOUT);
    assert!(timed_out.headers().contains_key("x-request-id"));

    let report = health(&app).await;
    let h = http(&report);
    assert_eq!(h["mcp_outcomes"]["body_too_large"], 1);
    assert_eq!(h["mcp_outcomes"]["timeout"], 1);
    assert_eq!(h["mcp_timeouts_total"], 1);
    // The dropped handler releases the in-flight gauge instead of leaking it.
    assert_eq!(h["mcp_inflight_total"], 0);
    assert_eq!(h["mcp_abandoned_total"], 1);
    assert_eq!(h["mcp_timeouts_before_handler_total"], 0);
    assert_balanced(h);
    // The dropped handler is logged, not silent.
    let logs = captured();
    assert!(
        logs.lines()
            .any(|l| l.contains("corr-timeout-01") && l.contains("outcome=\"abandoned\"")),
        "abandoned request must emit an operation event"
    );
}

#[tokio::test]
async fn timeout_before_the_handler_starts_keeps_the_counters_balanced() {
    install();
    let mut security = HttpSecurityConfig::default();
    security.request_timeout = Duration::from_millis(50);
    let app = test_app_with_limits(
        scripted(|r| McpResponse::success(r.id, json!({}))),
        Some(KEY),
        None,
        security,
        0,
        0,
    );
    // A body that never finishes: the extractor is still reading when the timeout fires.
    let stalled = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {KEY}"))
        .header("x-request-id", "corr-stalled-01")
        .body(Body::from_stream(futures::stream::pending::<
            Result<axum::body::Bytes, std::convert::Infallible>,
        >()))
        .expect("request");

    let response = app.clone().oneshot(stalled).await.expect("response");

    assert_eq!(response.status(), StatusCode::REQUEST_TIMEOUT);
    let report = health(&app).await;
    let h = http(&report);
    assert_eq!(h["mcp_timeouts_total"], 1);
    assert_eq!(h["mcp_timeouts_before_handler_total"], 1);
    assert_eq!(h["mcp_requests_total"], 1);
    assert_eq!(h["mcp_abandoned_total"], 0, "the handler never ran");
    assert_balanced(h);
}

#[tokio::test]
async fn active_sse_streams_are_a_gauge_that_returns_to_zero() {
    install();
    let app = test_app_with_limits(
        scripted(|r| McpResponse::success(r.id, json!({}))),
        None,
        Some(crate::realtime::RealtimeManager::new()),
        HttpSecurityConfig::default(),
        0,
        0,
    );
    let open = || {
        app.clone().oneshot(
            Request::builder()
                .uri("/v1/events")
                .body(Body::empty())
                .expect("request"),
        )
    };

    let first = open().await.expect("sse");
    let second = open().await.expect("sse");
    assert_eq!(first.status(), StatusCode::OK);
    assert!(first.headers().contains_key("x-request-id"));
    assert_eq!(http(&health(&app).await)["events_active"], 2);

    drop(first);
    assert_eq!(http(&health(&app).await)["events_active"], 1);
    drop(second);
    let report = health(&app).await;
    assert_eq!(http(&report)["events_active"], 0);
    assert_eq!(http(&report)["events_streams_total"], 2);
}

#[tokio::test]
async fn health_exports_critical_counters_and_what_is_not_instrumented() {
    install();
    let app = test_app_with_handler(
        scripted(|r| McpResponse::success(r.id, json!({}))),
        Some(KEY),
        0,
        0,
    );
    let report = health(&app).await;
    let block = report.get("observability").expect("observability export");
    assert!(block["permission_denied_total"].is_object());
    assert!(block["sqlite_busy_total"].is_u64());
    assert!(block["uptime_seconds"].is_u64());
    assert!(block["not_instrumented"]
        .as_array()
        .is_some_and(|entries| !entries.is_empty()));
}
