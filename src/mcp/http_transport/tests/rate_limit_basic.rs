use axum::body::to_bytes;
use axum::http::StatusCode;
use tower::ServiceExt;

use super::support::{
    json_rpc_notification_request_with_headers, json_rpc_request, json_rpc_request_with_headers,
    test_app_with_rate_limits,
};

#[tokio::test]
async fn test_post_mcp_rate_limit_rejects_after_burst() {
    let app = test_app_with_rate_limits(Some("secret-key"), 100, 1, None);
    let first = app
        .clone()
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[],
        ))
        .await
        .expect("request should be handled");

    assert_eq!(first.status(), StatusCode::OK);

    let second = app
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[],
        ))
        .await
        .expect("request should be handled");

    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after = second
        .headers()
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let second_body = to_bytes(second.into_body(), usize::MAX)
        .await
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_else(|| serde_json::json!({"error": {"code": null}}));
    assert_eq!(
        second_body
            .pointer("/error/code")
            .and_then(|code| code.as_i64()),
        Some(-32005)
    );
    assert_eq!(retry_after.as_deref(), Some("1"));
}

#[tokio::test]
async fn test_post_mcp_auth_failure_does_not_consume_rate_limit_bucket() {
    let app = test_app_with_rate_limits(Some("secret-key"), 100, 1, None);

    let unauthorized = app
        .clone()
        .oneshot(json_rpc_request("/mcp", None))
        .await
        .expect("request should be handled");
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);

    let authorized = app
        .clone()
        .oneshot(json_rpc_request("/mcp", Some("secret-key")))
        .await
        .expect("request should be handled");
    assert_eq!(authorized.status(), StatusCode::OK);

    let exhausted = app
        .oneshot(json_rpc_request("/mcp", Some("secret-key")))
        .await
        .expect("request should be handled");
    assert_eq!(exhausted.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_post_mcp_auth_failure_stays_unauthorized_when_bucket_exhausted() {
    let app = test_app_with_rate_limits(Some("secret-key"), 100, 1, None);

    let authorized = app
        .clone()
        .oneshot(json_rpc_request("/mcp", Some("secret-key")))
        .await
        .expect("request should be handled");
    assert_eq!(authorized.status(), StatusCode::OK);

    let unauthorized = app
        .oneshot(json_rpc_request("/mcp", None))
        .await
        .expect("request should be handled");
    assert_eq!(unauthorized.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_post_mcp_notification_rate_limit_returns_accepted_without_payload() {
    let app = test_app_with_rate_limits(Some("secret-key"), 100, 1, None);
    let first = app
        .clone()
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[],
        ))
        .await
        .expect("request should be handled");
    assert_eq!(first.status(), StatusCode::OK);

    let second = app
        .oneshot(json_rpc_notification_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[],
        ))
        .await
        .expect("request should be handled");

    assert_eq!(second.status(), StatusCode::ACCEPTED);
    let body = to_bytes(second.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    if !body.is_empty() {
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&body)
                .expect("response body should be valid JSON"),
            serde_json::Value::Null
        );
    }
}

#[tokio::test]
async fn test_post_v1_mcp_rate_limit_rejects_after_burst() {
    let app = test_app_with_rate_limits(Some("secret-key"), 100, 1, None);
    let first = app
        .clone()
        .oneshot(json_rpc_request_with_headers(
            "/v1/mcp",
            Some("secret-key"),
            &[],
        ))
        .await
        .expect("request should be handled");

    assert_eq!(first.status(), StatusCode::OK);

    let second = app
        .oneshot(json_rpc_request_with_headers(
            "/v1/mcp",
            Some("secret-key"),
            &[],
        ))
        .await
        .expect("request should be handled");

    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
    let retry_after = second
        .headers()
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let second_body = to_bytes(second.into_body(), usize::MAX)
        .await
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_else(|| serde_json::json!({"error": {"code": null}}));
    assert_eq!(
        second_body
            .pointer("/error/code")
            .and_then(|code| code.as_i64()),
        Some(-32005)
    );
    assert_eq!(retry_after.as_deref(), Some("1"));
}
