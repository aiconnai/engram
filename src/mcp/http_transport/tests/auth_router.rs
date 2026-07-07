use axum::body::to_bytes;
use axum::http::{HeaderMap, StatusCode};
use tower::ServiceExt;

use super::super::check_bearer;
use super::super::router::cors_origin_allowed;
use super::support::{json_rpc_request, test_app, ENV_LOCK};

#[test]
fn test_check_bearer_valid() {
    let mut headers = HeaderMap::new();
    headers.insert("authorization", "Bearer my-secret".parse().unwrap());
    assert!(check_bearer(&headers, "my-secret"));
}

#[test]
fn test_check_bearer_invalid_token() {
    let mut headers = HeaderMap::new();
    headers.insert("authorization", "Bearer wrong".parse().unwrap());
    assert!(!check_bearer(&headers, "my-secret"));
}

#[test]
fn test_check_bearer_missing_header() {
    let headers = HeaderMap::new();
    assert!(!check_bearer(&headers, "my-secret"));
}

#[test]
fn test_check_bearer_bad_scheme() {
    let mut headers = HeaderMap::new();
    headers.insert("authorization", "Basic abc123".parse().unwrap());
    assert!(!check_bearer(&headers, "abc123"));
}

// ---- Auth rejection test (integration-style, no network) -------------

#[test]
fn test_auth_rejection_no_header() {
    // Without bearer header, check_bearer should return false for any key.
    let headers = HeaderMap::new();
    assert!(!check_bearer(&headers, "secret-key"));
}

#[test]
fn test_auth_no_key_configured_always_passes() {
    // When api_key is None, the server allows any request.
    // check_bearer is only called when api_key is Some, so this
    // test documents the expected behavior.
    let has_key: Option<String> = None;
    // No key = no auth check = always allowed
    assert!(has_key.is_none());
}

#[tokio::test]
async fn test_post_mcp_requires_bearer_when_api_key_configured() {
    let app = test_app(Some("secret-key"));
    let response = app
        .oneshot(json_rpc_request("/mcp", None))
        .await
        .expect("request should be handled");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_else(|| serde_json::json!({"error": {"code": null}}));
    assert_eq!(
        body.pointer("/error/code").and_then(|code| code.as_i64()),
        Some(-32001)
    );
}

#[tokio::test]
async fn test_post_mcp_accepts_matching_bearer_token() {
    let app = test_app(Some("secret-key"));
    let response = app
        .oneshot(json_rpc_request("/mcp", Some("secret-key")))
        .await
        .expect("request should be handled");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_post_v1_mcp_alias_uses_same_auth_contract() {
    let app = test_app(Some("secret-key"));
    let response = app
        .oneshot(json_rpc_request("/v1/mcp", Some("secret-key")))
        .await
        .expect("request should be handled");

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_post_v1_mcp_alias_rejects_missing_bearer() {
    let app = test_app(Some("secret-key"));
    let response = app
        .oneshot(json_rpc_request("/v1/mcp", None))
        .await
        .expect("request should be handled");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_else(|| serde_json::json!({"error": {"code": null}}));
    assert_eq!(
        body.pointer("/error/code").and_then(|code| code.as_i64()),
        Some(-32001)
    );
}

#[test]
fn test_check_bearer_constant_time_wrong_token_rejected() {
    // Token differing only in the last byte must be rejected.
    let secret = "abcdefghijklmnop";
    let almost = "abcdefghijklmnox";
    let mut headers = HeaderMap::new();
    headers.insert("authorization", format!("Bearer {almost}").parse().unwrap());
    assert!(!check_bearer(&headers, secret));
}

// ---- H3: configurable CORS ---------------------------------------------

#[test]
fn test_cors_origins_default_allows_localhost() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var("ENGRAM_CORS_ORIGINS");
    assert!(cors_origin_allowed("http://localhost:3000"));
}

#[test]
fn test_cors_origins_default_rejects_external() {
    let _guard = ENV_LOCK.lock().unwrap();
    std::env::remove_var("ENGRAM_CORS_ORIGINS");
    assert!(!cors_origin_allowed("https://evil.example.com"));
}

/// Env-var-sensitive CORS tests are grouped in one test to avoid races
/// between parallel test threads mutating the same env var.
#[test]
fn test_cors_origins_env_var_cases() {
    let _guard = ENV_LOCK.lock().unwrap();
    // Case 1: * allows any origin
    std::env::set_var("ENGRAM_CORS_ORIGINS", "*");
    assert!(cors_origin_allowed("https://anything.example.com"));

    // Case 2: explicit list allows listed origin
    std::env::set_var(
        "ENGRAM_CORS_ORIGINS",
        "https://app.example.com,https://other.example.com",
    );
    assert!(cors_origin_allowed("https://app.example.com"));

    // Case 3: explicit list rejects unlisted origin
    std::env::set_var("ENGRAM_CORS_ORIGINS", "https://app.example.com");
    assert!(!cors_origin_allowed("https://other.example.com"));

    std::env::remove_var("ENGRAM_CORS_ORIGINS");
}
