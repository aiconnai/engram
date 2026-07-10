use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::http::HeaderValue;
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};

use super::super::protocol::McpHandler;
use super::events::handle_events;
use super::mcp_handler::handle_mcp;
use super::rate_limit::{
    RateLimiterConfig, RateLimiterState, RATE_LIMIT_MAX_BUCKETS, RATE_LIMIT_STALE_AFTER_SECS,
};
use super::{AppState, HttpTransportMetrics};
use crate::realtime::RealtimeManager;

/// `GET /health` -- lightweight liveness / readiness probe.
async fn handle_health(State(state): State<AppState>) -> impl IntoResponse {
    let rate_limit_state = match &state.rate_limiter {
        Some(rate_limiter) => {
            let limiter = rate_limiter.lock().await;
            json!({
                "enabled": true,
                "bucket_count": limiter.buckets.len(),
                "max_buckets": limiter.config.max_buckets,
                "requests_per_second": limiter.config.requests_per_second,
                "burst": limiter.config.burst,
                "stale_after_seconds": limiter.config.stale_after.as_secs(),
            })
        }
        None => json!({
            "enabled": false,
            "bucket_count": 0,
            "max_buckets": 0,
            "requests_per_second": 0.0,
            "burst": 0.0,
            "stale_after_seconds": 0,
        }),
    };

    let transport_metrics = state.metrics.snapshot();
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": "2025-11-25",
        "protection": rate_limit_state,
        "transport": {
            "http": transport_metrics
        }
    }))
}

// ---------------------------------------------------------------------------
// CORS helpers
// ---------------------------------------------------------------------------

/// Return `true` if `origin` is permitted according to `ENGRAM_CORS_ORIGINS`.
///
/// - Unset → only `http://localhost` and `http://127.0.0.1` prefixes allowed.
/// - `*`   → all origins allowed (opt-in).
/// - Comma-separated list → exact match required.
#[allow(dead_code)]
pub(crate) fn cors_origin_allowed(origin: &str) -> bool {
    match env::var("ENGRAM_CORS_ORIGINS") {
        Err(_) => origin.starts_with("http://localhost") || origin.starts_with("http://127.0.0.1"),
        Ok(val) if val.trim() == "*" => true,
        Ok(val) => val.split(',').any(|s| s.trim() == origin),
    }
}

/// Build a `CorsLayer` honouring `ENGRAM_CORS_ORIGINS`.
fn build_cors_layer() -> CorsLayer {
    match env::var("ENGRAM_CORS_ORIGINS") {
        Ok(val) if val.trim() == "*" => CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
        Ok(val) => {
            let origins: Vec<HeaderValue> = val
                .split(',')
                .filter_map(|s| s.trim().parse::<HeaderValue>().ok())
                .collect();
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods(Any)
                .allow_headers(Any)
        }
        Err(_) => {
            let origins: Vec<HeaderValue> = ["http://localhost", "http://127.0.0.1"]
                .iter()
                .filter_map(|s| s.parse::<HeaderValue>().ok())
                .collect();
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods(Any)
                .allow_headers(Any)
        }
    }
}

/// Build the Axum router for the MCP HTTP transport.
///
/// Module-private by design: callers must go through [`serve_http`] or the
/// test helper `test_app`. Direct router composition from outside the module
/// is not a supported use-case; if it becomes necessary, promote this to `pub`.
pub(super) fn build_router(
    handler: Arc<dyn McpHandler>,
    api_key: Option<String>,
    realtime: Option<RealtimeManager>,
    http_rate_limit_rps: u64,
    http_rate_limit_burst: u64,
    http_rate_limit_key: Option<String>,
) -> Router {
    let rate_limiter = if http_rate_limit_rps > 0 && http_rate_limit_burst > 0 {
        let key_header = http_rate_limit_key.and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_lowercase())
            }
        });
        Some(Arc::new(tokio::sync::Mutex::new(RateLimiterState {
            config: RateLimiterConfig {
                requests_per_second: http_rate_limit_rps as f64,
                burst: http_rate_limit_burst as f64,
                key_header,
                max_buckets: RATE_LIMIT_MAX_BUCKETS,
                stale_after: Duration::from_secs(RATE_LIMIT_STALE_AFTER_SECS),
            },
            buckets: HashMap::new(),
        })))
    } else {
        None
    };

    let state = AppState {
        handler,
        api_key,
        realtime,
        rate_limiter,
        metrics: Arc::new(HttpTransportMetrics::default()),
    };

    let cors = build_cors_layer();

    Router::new()
        .route("/mcp", post(handle_mcp))
        .route("/v1/mcp", post(handle_mcp))
        .route("/health", get(handle_health))
        .route("/v1/events", get(handle_events))
        .layer(cors)
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Public entry-point
// ---------------------------------------------------------------------------

/// Start the axum HTTP server on `addr`.
///
/// The server will run until the process is terminated.
///
/// - `realtime` — optional `RealtimeManager` for SSE streaming (`GET /v1/events`).
///   When `None`, the `/v1/events` endpoint returns `503 Service Unavailable`.
pub async fn serve_http(
    handler: Arc<dyn McpHandler>,
    addr: SocketAddr,
    api_key: Option<String>,
    realtime: Option<RealtimeManager>,
    http_rate_limit_rps: u64,
    http_rate_limit_burst: u64,
    http_rate_limit_key: Option<String>,
) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let app = build_router(
        handler,
        api_key,
        realtime,
        http_rate_limit_rps,
        http_rate_limit_burst,
        http_rate_limit_key,
    );

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("HTTP transport listening on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
