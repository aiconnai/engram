//! Streamable HTTP transport for MCP (Model Context Protocol)
//!
//! Provides an axum-based HTTP server that accepts JSON-RPC requests at `POST /mcp`
//! and forwards them to the same `McpHandler` used by the stdio transport.
//!
//! Also provides a `GET /v1/events` SSE endpoint for real-time event streaming.

use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Duration;

use axum::http::HeaderMap;

use super::protocol::McpHandler;
use crate::auth::{Permission, ResourceType, TransportPrincipal, TransportPrincipalError};
use crate::realtime::RealtimeManager;

mod events;
mod mcp_handler;
mod rate_limit;
mod router;

pub use router::serve_http;

use rate_limit::RateLimiterState;

const MICROSECONDS_PER_MILLISECOND: u64 = 1_000;

#[derive(Default)]
struct HttpTransportMetrics {
    mcp_requests_total: AtomicU64,
    mcp_requests_completed: AtomicU64,
    mcp_notifications_total: AtomicU64,
    mcp_rate_limited_total: AtomicU64,
    mcp_unauthorized_total: AtomicU64,
    mcp_failed_total: AtomicU64,
    mcp_success_total: AtomicU64,
    mcp_inflight_total: AtomicU64,
    mcp_latency_nanos: AtomicU64,

    events_requests_total: AtomicU64,
    events_requests_unauthorized_total: AtomicU64,
    events_requests_no_realtime_total: AtomicU64,

    rate_limit_buckets_stale_cleanups: AtomicU64,
    rate_limit_bucket_evictions: AtomicU64,
}

#[derive(serde::Serialize)]
struct HttpTransportMetricsSnapshot {
    mcp_requests_total: u64,
    mcp_requests_completed: u64,
    mcp_notifications_total: u64,
    mcp_rate_limited_total: u64,
    mcp_unauthorized_total: u64,
    mcp_failed_total: u64,
    mcp_success_total: u64,
    mcp_inflight_total: u64,
    mcp_avg_latency_ms: f64,
    events_requests_total: u64,
    events_requests_unauthorized_total: u64,
    events_requests_no_realtime_total: u64,
    rate_limit_buckets_stale_cleanups: u64,
    rate_limit_bucket_evictions: u64,
}

impl HttpTransportMetrics {
    fn on_mcp_request_start(&self, is_notification: bool) {
        self.mcp_requests_total.fetch_add(1, Ordering::Relaxed);
        if is_notification {
            self.mcp_notifications_total.fetch_add(1, Ordering::Relaxed);
        }
        self.mcp_inflight_total.fetch_add(1, Ordering::Relaxed);
    }

    fn on_mcp_request_complete(
        &self,
        is_success: bool,
        is_unauthorized: bool,
        is_rate_limited: bool,
        latency: Duration,
    ) {
        self.mcp_requests_completed.fetch_add(1, Ordering::Relaxed);
        self.mcp_inflight_total.fetch_sub(1, Ordering::Relaxed);

        let latency_nanos = u64::try_from(latency.as_nanos()).unwrap_or(u64::MAX);
        self.mcp_latency_nanos
            .fetch_add(latency_nanos, Ordering::Relaxed);

        if is_rate_limited {
            self.mcp_rate_limited_total.fetch_add(1, Ordering::Relaxed);
            self.mcp_failed_total.fetch_add(1, Ordering::Relaxed);
            return;
        }

        if is_unauthorized {
            self.mcp_unauthorized_total.fetch_add(1, Ordering::Relaxed);
            self.mcp_failed_total.fetch_add(1, Ordering::Relaxed);
            return;
        }

        if is_success {
            self.mcp_success_total.fetch_add(1, Ordering::Relaxed);
        } else {
            self.mcp_failed_total.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn on_events_request(&self, is_unauthorized: bool, is_no_realtime: bool) {
        self.events_requests_total.fetch_add(1, Ordering::Relaxed);
        if is_unauthorized {
            self.events_requests_unauthorized_total
                .fetch_add(1, Ordering::Relaxed);
        }
        if is_no_realtime {
            self.events_requests_no_realtime_total
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    fn on_rate_limit_cleanup(&self, stale: u64, evictions: u64) {
        if stale > 0 {
            self.rate_limit_buckets_stale_cleanups
                .fetch_add(stale, Ordering::Relaxed);
        }
        if evictions > 0 {
            self.rate_limit_bucket_evictions
                .fetch_add(evictions, Ordering::Relaxed);
        }
    }

    fn snapshot(&self) -> HttpTransportMetricsSnapshot {
        let completed = self.mcp_requests_completed.load(Ordering::Relaxed);
        let latency_nanos = self.mcp_latency_nanos.load(Ordering::Relaxed);
        let avg_latency_ms = if completed == 0 {
            0.0
        } else {
            (latency_nanos as f64 / completed as f64) / 1_000_000.0
        };

        HttpTransportMetricsSnapshot {
            mcp_requests_total: self.mcp_requests_total.load(Ordering::Relaxed),
            mcp_requests_completed: completed,
            mcp_notifications_total: self.mcp_notifications_total.load(Ordering::Relaxed),
            mcp_rate_limited_total: self.mcp_rate_limited_total.load(Ordering::Relaxed),
            mcp_unauthorized_total: self.mcp_unauthorized_total.load(Ordering::Relaxed),
            mcp_failed_total: self.mcp_failed_total.load(Ordering::Relaxed),
            mcp_success_total: self.mcp_success_total.load(Ordering::Relaxed),
            mcp_inflight_total: self.mcp_inflight_total.load(Ordering::Relaxed),
            mcp_avg_latency_ms: avg_latency_ms,
            events_requests_total: self.events_requests_total.load(Ordering::Relaxed),
            events_requests_unauthorized_total: self
                .events_requests_unauthorized_total
                .load(Ordering::Relaxed),
            events_requests_no_realtime_total: self
                .events_requests_no_realtime_total
                .load(Ordering::Relaxed),
            rate_limit_buckets_stale_cleanups: self
                .rate_limit_buckets_stale_cleanups
                .load(Ordering::Relaxed),
            rate_limit_bucket_evictions: self.rate_limit_bucket_evictions.load(Ordering::Relaxed),
        }
    }
}

/// Shared application state for all axum handlers.
#[derive(Clone)]
struct AppState {
    handler: Arc<dyn McpHandler>,
    api_key: Option<String>,
    realtime: Option<RealtimeManager>,
    rate_limiter: Option<Arc<tokio::sync::Mutex<RateLimiterState>>>,
    metrics: Arc<HttpTransportMetrics>,
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
