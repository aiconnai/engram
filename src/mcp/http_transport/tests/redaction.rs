//! O1: HTTP transport logs never carry credentials, request content or panic
//! payloads; wire responses are unchanged.

use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use serde_json::{json, Value};
use tower::ServiceExt;

use super::support::{json_rpc_request, test_app_with_handler, test_app_with_rate_limits};
use crate::mcp::protocol::{McpHandler, McpRequest, McpResponse};
use crate::observability::test_capture::{assert_logs_exclude, install};

struct PanickingHandler {
    payload: &'static str,
}

impl McpHandler for PanickingHandler {
    fn handle_request(&self, _request: McpRequest) -> McpResponse {
        panic!("handler failed while processing {}", self.payload);
    }
}

async fn body_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

#[tokio::test]
async fn credentials_are_never_logged_for_rejected_or_accepted_requests() {
    install();
    const WRONG: &str = "bearer-sentinel-wrong-1a2b";
    const RIGHT: &str = "bearer-sentinel-right-3c4d";
    let app = test_app_with_rate_limits(Some(RIGHT), 0, 0, None);

    let rejected = app
        .clone()
        .oneshot(json_rpc_request("/mcp", Some(WRONG)))
        .await
        .expect("response");
    assert_eq!(rejected.status(), StatusCode::UNAUTHORIZED);
    assert_eq!(body_json(rejected).await["error"]["code"], -32001);

    let accepted = app
        .oneshot(json_rpc_request("/mcp", Some(RIGHT)))
        .await
        .expect("response");
    assert_eq!(accepted.status(), StatusCode::OK);

    assert_logs_exclude(&[WRONG, RIGHT]);
}

#[tokio::test]
async fn panicking_handler_payload_is_not_logged_but_response_is_unchanged() {
    install();
    const PAYLOAD: &str = "panic-payload-sentinel-5e6f";
    let app = test_app_with_handler(Arc::new(PanickingHandler { payload: PAYLOAD }), None, 0, 0);

    let response = app
        .oneshot(json_rpc_request("/mcp", None))
        .await
        .expect("response");

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response).await;
    assert_eq!(body["error"]["code"], -32603);
    assert_eq!(body["error"]["message"], "Internal server error");
    assert_logs_exclude(&[PAYLOAD]);
}

#[tokio::test]
async fn malformed_body_is_not_logged_but_rejection_status_is_unchanged() {
    install();
    const PRIVATE: &str = "private-body-sentinel-7a8b";
    let app = test_app_with_rate_limits(None, 0, 0, None);
    let request = Request::builder()
        .method("POST")
        .uri("/mcp")
        .header("content-type", "application/json")
        .body(Body::from(format!(
            r#"{{"jsonrpc":"2.0","note":"{PRIVATE}""#
        )))
        .expect("request");

    let response = app.oneshot(request).await.expect("response");

    assert!(
        response.status().is_client_error(),
        "malformed JSON stays a 4xx rejection, got {}",
        response.status()
    );
    assert_logs_exclude(&[PRIVATE]);
}

#[tokio::test]
async fn rate_limit_key_header_value_is_neither_logged_nor_exported() {
    install();
    const KEY: &str = "client-key-sentinel-9c0d";
    let app = test_app_with_rate_limits(None, 100, 1, Some("x-client-key"));
    let request = || {
        Request::builder()
            .method("POST")
            .uri("/mcp")
            .header("content-type", "application/json")
            .header("x-client-key", KEY)
            .body(Body::from(
                json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}).to_string(),
            ))
            .expect("request")
    };

    let first = app.clone().oneshot(request()).await.expect("response");
    assert_eq!(first.status(), StatusCode::OK);
    let limited = app.clone().oneshot(request()).await.expect("response");
    assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        limited
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok()),
        Some("1")
    );

    let health = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    let exported = body_json(health).await.to_string();
    assert!(!exported.contains(KEY), "health export leaked the key");
    assert_logs_exclude(&[KEY]);
}
