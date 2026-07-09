use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use super::support::{json_rpc_request, test_app_with_rate_limits};

#[tokio::test]
async fn test_health_includes_http_transport_metrics() {
    let app = test_app_with_rate_limits(None, 100, 1, None);

    let first = app
        .clone()
        .oneshot(json_rpc_request("/mcp", None))
        .await
        .expect("request should be handled");
    assert_eq!(first.status(), StatusCode::OK);

    let second = app
        .clone()
        .oneshot(json_rpc_request("/mcp", None))
        .await
        .expect("request should be handled");
    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);

    let health_request = Request::builder()
        .method("GET")
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let health = app
        .oneshot(health_request)
        .await
        .expect("health request should be handled");
    assert_eq!(health.status(), StatusCode::OK);

    let health_body = to_bytes(health.into_body(), usize::MAX)
        .await
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_else(|| serde_json::json!({}));

    let transport = health_body
        .pointer("/transport/http")
        .expect("health should expose transport.http metrics");
    assert_eq!(
        transport.get("mcp_requests_total").and_then(|v| v.as_u64()),
        Some(2)
    );
    assert_eq!(
        transport
            .get("mcp_rate_limited_total")
            .and_then(|v| v.as_u64()),
        Some(1)
    );
    assert_eq!(
        transport
            .get("mcp_requests_completed")
            .and_then(|v| v.as_u64()),
        Some(2)
    );
    assert_eq!(
        transport.get("mcp_success_total").and_then(|v| v.as_u64()),
        Some(1)
    );
    assert_eq!(
        transport.get("mcp_inflight_total").and_then(|v| v.as_u64()),
        Some(0)
    );

    let protection = health_body
        .pointer("/protection")
        .expect("health should expose protection status");
    assert_eq!(
        protection.get("enabled").and_then(|v| v.as_bool()),
        Some(true)
    );
}

// ---- Keep-alive configuration test ------------------------------------
