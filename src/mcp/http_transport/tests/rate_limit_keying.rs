use axum::http::StatusCode;
use tower::ServiceExt;

use super::support::{json_rpc_request_with_headers, test_app_with_rate_limits};

#[tokio::test]
async fn test_post_mcp_rate_limit_uses_custom_key_header() {
    let app = test_app_with_rate_limits(Some("secret-key"), 100, 1, Some("x-tenant-id"));
    let tenant_a = app
        .clone()
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[("x-tenant-id", "tenant-a")],
        ))
        .await
        .expect("request should be handled");
    assert_eq!(tenant_a.status(), StatusCode::OK);

    let tenant_a_second = app
        .clone()
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[("x-tenant-id", "tenant-a")],
        ))
        .await
        .expect("request should be handled");
    assert_eq!(tenant_a_second.status(), StatusCode::TOO_MANY_REQUESTS);

    let tenant_b = app
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[("x-tenant-id", "tenant-b")],
        ))
        .await
        .expect("request should be handled");
    assert_eq!(tenant_b.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_post_mcp_rate_limit_ignores_unverified_x_real_ip() {
    let app = test_app_with_rate_limits(Some("secret-key"), 100, 1, None);

    let first_ip = app
        .clone()
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[("x-real-ip", "198.51.100.10")],
        ))
        .await
        .expect("request should be handled");
    assert_eq!(first_ip.status(), StatusCode::OK);

    let first_ip_again = app
        .clone()
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[("x-real-ip", "198.51.100.10")],
        ))
        .await
        .expect("request should be handled");
    assert_eq!(first_ip_again.status(), StatusCode::TOO_MANY_REQUESTS);

    let second_ip = app
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[("x-real-ip", "198.51.100.11")],
        ))
        .await
        .expect("request should be handled");
    assert_eq!(second_ip.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_post_mcp_rate_limit_ignores_unverified_forwarding_headers() {
    let app = test_app_with_rate_limits(Some("secret-key"), 100, 1, None);

    let xff_first = app
        .clone()
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[
                ("x-forwarded-for", "198.51.100.10"),
                ("x-real-ip", "198.51.100.20"),
            ],
        ))
        .await
        .expect("request should be handled");
    assert_eq!(xff_first.status(), StatusCode::OK);

    let same_xff_different_real_ip = app
        .clone()
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[
                ("x-forwarded-for", "198.51.100.10"),
                ("x-real-ip", "198.51.100.21"),
            ],
        ))
        .await
        .expect("request should be handled");
    assert_eq!(
        same_xff_different_real_ip.status(),
        StatusCode::TOO_MANY_REQUESTS
    );

    let different_xff_same_real_ip = app
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[
                ("x-forwarded-for", "198.51.100.11"),
                ("x-real-ip", "198.51.100.20"),
            ],
        ))
        .await
        .expect("request should be handled");
    assert_eq!(
        different_xff_same_real_ip.status(),
        StatusCode::TOO_MANY_REQUESTS
    );
}

#[tokio::test]
async fn test_post_mcp_rate_limit_empty_key_disables_header_keying() {
    let app = test_app_with_rate_limits(Some("secret-key"), 100, 1, Some(""));

    let tenant_a = app
        .clone()
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[
                ("x-tenant-id", "tenant-a"),
                ("x-forwarded-for", "198.51.100.10"),
            ],
        ))
        .await
        .expect("request should be handled");
    assert_eq!(tenant_a.status(), StatusCode::OK);

    let tenant_a_second = app
        .clone()
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[
                ("x-tenant-id", "tenant-a"),
                ("x-forwarded-for", "198.51.100.10"),
            ],
        ))
        .await
        .expect("request should be handled");
    assert_eq!(tenant_a_second.status(), StatusCode::TOO_MANY_REQUESTS);

    let tenant_b = app
        .oneshot(json_rpc_request_with_headers(
            "/mcp",
            Some("secret-key"),
            &[
                ("x-tenant-id", "tenant-b"),
                ("x-forwarded-for", "198.51.100.11"),
            ],
        ))
        .await
        .expect("request should be handled");
    assert_eq!(tenant_b.status(), StatusCode::TOO_MANY_REQUESTS);
}
