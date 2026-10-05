//! Streamable HTTP transport for MCP (Model Context Protocol)
//!
//! Provides an axum-based HTTP server that accepts JSON-RPC requests at `POST /mcp`
//! and forwards them to the same `McpHandler` used by the stdio transport.
//!
//! Also provides a `GET /v1/events` SSE endpoint for real-time event streaming.

use std::sync::Arc;

use axum::http::HeaderMap;

use super::protocol::McpHandler;
use crate::auth::{Permission, ResourceType, TransportPrincipal, TransportPrincipalError};
use crate::realtime::RealtimeManager;

mod events;
mod mcp_handler;
mod metrics;
mod rate_limit;
mod request_id;
mod router;
mod security_config;

pub use router::serve_http;

use metrics::HttpTransportMetrics;
use rate_limit::RateLimiterState;

/// Shared application state for all axum handlers.
#[derive(Clone)]
struct AppState {
    handler: Arc<dyn McpHandler>,
    api_key: Option<String>,
    realtime: Option<RealtimeManager>,
    rate_limiter: Option<Arc<tokio::sync::Mutex<RateLimiterState>>>,
    metrics: Arc<HttpTransportMetrics>,
    security: security_config::HttpSecurityConfig,
}

// ---------------------------------------------------------------------------
// Auth helpers
// ---------------------------------------------------------------------------

#[cfg(test)]
fn check_bearer(headers: &HeaderMap, expected: &str) -> bool {
    authenticate_transport_principal(&Some(expected.to_string()), headers).is_ok()
}

fn normalize_api_key(api_key: Option<String>) -> Option<String> {
    api_key.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn authenticate_transport_principal(
    api_key: &Option<String>,
    headers: &HeaderMap,
) -> Result<TransportPrincipal, TransportPrincipalError> {
    match api_key {
        Some(expected) => TransportPrincipal::from_process_bearer(
            headers.get("authorization").and_then(|v| v.to_str().ok()),
            expected,
        ),
        None => Ok(TransportPrincipal::anonymous_loopback()),
    }
}

fn principal_can_read_workspace(
    principal: &TransportPrincipal,
    requested_workspace: Option<&str>,
) -> bool {
    principal.has_permission(Permission::Read, ResourceType::Memory)
        && principal.allows_workspace(requested_workspace)
}

#[cfg(test)]
mod tests;
