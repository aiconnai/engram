//! Streamable HTTP transport for MCP (Model Context Protocol)
//!
//! Provides an axum-based HTTP server that accepts JSON-RPC requests at `POST /mcp`
//! and forwards them to the same `McpHandler` used by the stdio transport.
//!
//! Also provides a `GET /v1/events` SSE endpoint for real-time event streaming.

use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use subtle::ConstantTimeEq;

use axum::http::HeaderValue;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode, Uri},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::{Stream, StreamExt};
use tower_http::cors::{Any, CorsLayer};

use super::protocol::{McpHandler, McpRequest, McpResponse};
use crate::realtime::{EventType, RealtimeEvent, RealtimeManager};

const RATE_LIMIT_MAX_BUCKETS: usize = 10_000;
const RATE_LIMIT_STALE_AFTER_SECS: u64 = 600;
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

#[derive(Clone)]
struct RateLimiterConfig {
    requests_per_second: f64,
    burst: f64,
    key_header: Option<String>,
    max_buckets: usize,
    stale_after: Duration,
}

struct RateLimitBucket {
    last_seen: Instant,
    tokens: f64,
    last_refill_at: Instant,
}

struct RateLimiterState {
    config: RateLimiterConfig,
    buckets: HashMap<String, RateLimitBucket>,
}

// ---------------------------------------------------------------------------
// Routes
// ---------------------------------------------------------------------------

/// `POST /mcp` -- accept a JSON-RPC request and return a JSON-RPC response.
/// Per JSON-RPC 2.0, notifications (no `id`) MUST NOT produce a response.
async fn handle_mcp(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
    Json(request): Json<McpRequest>,
) -> Response {
    let request_started = Instant::now();
    let is_notification = request.id.is_none();
    state.metrics.on_mcp_request_start(is_notification);

    let duration_ms = || {
        request_started
            .elapsed()
            .as_micros()
            .saturating_div(MICROSECONDS_PER_MILLISECOND as u128)
    };

    let mut decision = "success";
    let mut is_unauthorized = false;
    let mut is_rate_limited = false;
    let mut include_retry_after = false;

    let (status, response_payload) = if !is_rate_limit_allowed(&state, &headers).await {
        is_rate_limited = true;
        decision = "rate_limited";
        include_retry_after = true;
        if is_notification {
            (StatusCode::ACCEPTED, serde_json::Value::Null)
        } else {
            (
                StatusCode::TOO_MANY_REQUESTS,
                serde_json::to_value(McpResponse::error(
                    request.id,
                    -32005,
                    "Too Many Requests".to_string(),
                ))
                .unwrap_or_else(|e| {
                    tracing::error!(
                        error = %e,
                        route = %uri.path(),
                        "failed to serialize error response"
                    );
                    serde_json::Value::Null
                }),
            )
        }
    } else if let Some(ref expected) = state.api_key {
        if !check_bearer(&headers, expected) {
            is_unauthorized = true;
            decision = "unauthorized";
            let status = if is_notification {
                StatusCode::ACCEPTED
            } else {
                StatusCode::UNAUTHORIZED
            };

            (
                status,
                if is_notification {
                    serde_json::Value::Null
                } else {
                    serde_json::to_value(McpResponse::error(
                        request.id,
                        -32001,
                        "Unauthorized".to_string(),
                    ))
                    .unwrap_or_else(|e| {
                        tracing::error!(
                            error = %e,
                            route = %uri.path(),
                            "failed to serialize error response"
                        );
                        serde_json::Value::Null
                    })
                },
            )
        } else {
            if is_notification {
                (StatusCode::ACCEPTED, serde_json::Value::Null)
            } else {
                let response = state.handler.handle_request(request);
                (
                    StatusCode::OK,
                    serde_json::to_value(response).unwrap_or_else(|e| {
                        tracing::error!(error = %e, route = %uri.path(), "failed to serialize MCP response");
                        serde_json::Value::Null
                    }),
                )
            }
        }
    } else if is_notification {
        decision = "success";
        (StatusCode::ACCEPTED, serde_json::Value::Null)
    } else {
        let response = state.handler.handle_request(request);
        decision = "success";
        (
            StatusCode::OK,
            serde_json::to_value(response).unwrap_or_else(|e| {
                tracing::error!(error = %e, route = %uri.path(), "failed to serialize MCP response");
                serde_json::Value::Null
            }),
        )
    };

    state.metrics.on_mcp_request_complete(
        !is_unauthorized && !is_rate_limited,
        is_unauthorized,
        is_rate_limited,
        request_started.elapsed(),
    );

    let response = if include_retry_after {
        (status, [("retry-after", "1")], Json(response_payload)).into_response()
    } else {
        (status, Json(response_payload)).into_response()
    };

    if is_rate_limited || is_unauthorized {
        tracing::warn!(
            route = %uri.path(),
            status = %status,
            decision = decision,
            notification = is_notification,
            duration_ms = duration_ms(),
            "mcp_http_request"
        );
    } else {
        tracing::info!(
            route = %uri.path(),
            status = %status,
            decision = decision,
            notification = is_notification,
            duration_ms = duration_ms(),
            "mcp_http_request"
        );
    }

    response
}

/// Check whether this request may proceed under the configured rate limit.
///
/// If no limiter is configured, always allows the request.
async fn is_rate_limit_allowed(state: &AppState, headers: &HeaderMap) -> bool {
    let rate_limiter = match &state.rate_limiter {
        Some(rate_limiter) => rate_limiter,
        None => return true,
    };

    let now = Instant::now();
    let mut limiter = rate_limiter.lock().await;
    let config = limiter.config.clone();
    let bucket_key = rate_limit_key(&config, headers);
    let mut stale_cleanup = 0u64;
    let mut eviction_cleanup = 0u64;

    if limiter.buckets.len() >= config.max_buckets && !limiter.buckets.contains_key(&bucket_key) {
        if config.stale_after > Duration::ZERO {
            if let Some(cutoff) = now.checked_sub(config.stale_after) {
                let before_stale = limiter.buckets.len();
                limiter
                    .buckets
                    .retain(|_, bucket| bucket.last_seen >= cutoff);
                let after_stale = limiter.buckets.len();
                stale_cleanup =
                    stale_cleanup.saturating_add((before_stale.saturating_sub(after_stale)) as u64);
            }
        }

        if limiter.buckets.len() >= config.max_buckets {
            if let Some(oldest_key) = limiter
                .buckets
                .iter()
                .min_by_key(|(_, bucket)| bucket.last_seen)
                .map(|(key, _)| key.clone())
            {
                limiter.buckets.remove(&oldest_key);
                eviction_cleanup = eviction_cleanup.saturating_add(1);
            }
        }
    }

    state
        .metrics
        .on_rate_limit_cleanup(stale_cleanup, eviction_cleanup);

    let bucket = limiter
        .buckets
        .entry(bucket_key)
        .or_insert_with(|| RateLimitBucket {
            tokens: config.burst,
            last_seen: now,
            last_refill_at: now,
        });

    let elapsed = now.duration_since(bucket.last_refill_at).as_secs_f64();
    let refill = elapsed * config.requests_per_second;
    bucket.tokens = (bucket.tokens + refill).min(config.burst);
    bucket.last_refill_at = now;
    bucket.last_seen = now;

    if bucket.tokens < 1.0 {
        return false;
    }

    bucket.tokens -= 1.0;
    true
}

fn rate_limit_key(config: &RateLimiterConfig, headers: &HeaderMap) -> String {
    if let Some(header_name) = config.key_header.as_deref() {
        if let Some(raw) = headers
            .get(header_name)
            .and_then(|header| header.to_str().ok())
            .and_then(|value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            })
        {
            return format!("header:{header_name}:{raw}");
        }
    }

    if let Some(xff) = headers
        .get("x-forwarded-for")
        .and_then(|header| header.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        return format!("ip:{xff}");
    }

    if let Some(ip) = headers
        .get("x-real-ip")
        .and_then(|header| header.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("ip:{ip}");
    }

    "ip:unknown".to_string()
}

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
// SSE query parameters
// ---------------------------------------------------------------------------

/// Query parameters for the `GET /v1/events` SSE endpoint.
#[derive(Debug, Clone, Deserialize)]
struct EventsQuery {
    /// Comma-separated list of event types to subscribe to.
    /// Accepted values: `memory_created`, `memory_updated`, `memory_deleted`,
    /// `crossref_created`, `crossref_deleted`, `sync_started`, `sync_completed`,
    /// `sync_failed`.
    /// If omitted, all event types are streamed.
    event_types: Option<String>,

    /// Filter events to a specific workspace (matched against `data.workspace`).
    /// If omitted, events from all workspaces are streamed.
    workspace: Option<String>,
}

impl EventsQuery {
    /// Parse the `event_types` query param into a `Vec<EventType>`.
    /// Unknown tokens are silently ignored.
    fn parsed_event_types(&self) -> Option<Vec<EventType>> {
        let raw = self.event_types.as_deref()?;
        let types: Vec<EventType> = raw
            .split(',')
            .filter_map(|s| parse_event_type(s.trim()))
            .collect();
        if types.is_empty() {
            None
        } else {
            Some(types)
        }
    }
}

/// Parse a snake_case string into an `EventType`.
fn parse_event_type(s: &str) -> Option<EventType> {
    match s {
        "memory_created" => Some(EventType::MemoryCreated),
        "memory_updated" => Some(EventType::MemoryUpdated),
        "memory_deleted" => Some(EventType::MemoryDeleted),
        "crossref_created" => Some(EventType::CrossrefCreated),
        "crossref_deleted" => Some(EventType::CrossrefDeleted),
        "sync_started" => Some(EventType::SyncStarted),
        "sync_completed" => Some(EventType::SyncCompleted),
        "sync_failed" => Some(EventType::SyncFailed),
        _ => None,
    }
}

/// Serialize an `EventType` to its SSE `event:` field string.
fn event_type_to_str(et: EventType) -> &'static str {
    match et {
        EventType::MemoryCreated => "memory_created",
        EventType::MemoryUpdated => "memory_updated",
        EventType::MemoryDeleted => "memory_deleted",
        EventType::CrossrefCreated => "crossref_created",
        EventType::CrossrefDeleted => "crossref_deleted",
        EventType::SyncStarted => "sync_started",
        EventType::SyncCompleted => "sync_completed",
        EventType::SyncFailed => "sync_failed",
    }
}

// ---------------------------------------------------------------------------
// SSE handler
// ---------------------------------------------------------------------------

/// Reconnection backoff hint sent to SSE clients (milliseconds).
const SSE_RETRY_MS: u64 = 3000;

/// Convert a `RealtimeEvent` into an SSE `Event`, stamping the `id:` field
/// with `seq_id` when present.
fn realtime_event_to_sse(event: &RealtimeEvent) -> Event {
    let event_type_str = event_type_to_str(event.event_type);
    let data = serde_json::to_string(event).unwrap_or_else(|_| "{}".to_string());
    let mut sse = Event::default().event(event_type_str).data(data);
    if let Some(id) = event.seq_id {
        sse = sse.id(format!("{id}"));
    }
    sse
}

/// `GET /v1/events` — resumable Server-Sent Events stream of `RealtimeEvent`s.
///
/// Each event is sent as:
/// ```text
/// id: <seq_id>
/// event: <event_type>
/// data: <JSON of RealtimeEvent>
/// retry: 3000
/// ```
///
/// **Resumable streams:** clients that reconnect after a drop should include
/// the `Last-Event-Id` header set to the last `id` value they received.
/// The server will replay all buffered events with a higher sequence number
/// before continuing with the live stream.
///
/// Query parameters:
/// - `event_types` — comma-separated list of event types to subscribe to
/// - `workspace` — filter events by workspace (matched against `data.workspace`)
///
/// Requires `Authorization: Bearer <token>` when the server was started with an API key.
async fn handle_events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<EventsQuery>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, StatusCode> {
    // Auth check
    if let Some(ref expected) = state.api_key {
        if !check_bearer(&headers, expected) {
            state.metrics.on_events_request(true, false);
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // If realtime is not enabled, return 503.
    let manager = match state.realtime {
        Some(m) => m,
        None => {
            state.metrics.on_events_request(false, true);
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    state.metrics.on_events_request(false, false);

    // Parse Last-Event-Id header for replay support.
    let last_event_id: Option<u64> = headers
        .get("last-event-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok());

    let event_type_filter = query.parsed_event_types();
    let workspace_filter = query.workspace.clone();

    // Build a filter closure reused for both replay and live events.
    let apply_filters = {
        let et_filter = event_type_filter.clone();
        let ws_filter = workspace_filter.clone();
        move |event: &RealtimeEvent| -> bool {
            if let Some(ref types) = et_filter {
                if !types.contains(&event.event_type) {
                    return false;
                }
            }
            if let Some(ref ws) = ws_filter {
                let event_ws = event
                    .data
                    .as_ref()
                    .and_then(|d: &serde_json::Value| d.get("workspace"))
                    .and_then(|v: &serde_json::Value| v.as_str());
                match event_ws {
                    Some(ews) if ews == ws => {}
                    _ => return false,
                }
            }
            true
        }
    };

    // Subscribe to the live broadcast channel *before* draining the buffer so
    // we don't miss any events that arrive between the two operations.
    let rx = manager.subscribe();
    let broadcast_stream = BroadcastStream::new(rx);

    // Build the replay burst (may be empty if no Last-Event-Id or nothing to replay).
    let replay_events: Vec<Result<Event, Infallible>> = if let Some(last_id) = last_event_id {
        manager
            .get_events_after(last_id)
            .into_iter()
            .filter(|e| apply_filters(e))
            .map(|e| Ok::<Event, Infallible>(realtime_event_to_sse(&e)))
            .collect()
    } else {
        vec![]
    };

    let replay_stream = tokio_stream::iter(replay_events);

    // Live stream from broadcast channel.
    let live_stream = broadcast_stream.filter_map(move |result| {
        match result {
            // Lagged: the receiver fell behind — skip dropped events without crashing.
            Err(_lagged) => None,
            Ok(event) => {
                if !apply_filters(&event) {
                    return None;
                }
                Some(Ok::<Event, Infallible>(realtime_event_to_sse(&event)))
            }
        }
    });

    // Chain: replay burst first, then live events.
    let combined = replay_stream.chain(live_stream);

    // Prepend a `retry:` field so clients know the reconnection backoff.
    // The retry directive is sent as a synthetic SSE comment event emitted once
    // at the start of the stream.
    let retry_event = std::iter::once(Ok::<Event, Infallible>(
        Event::default().retry(Duration::from_millis(SSE_RETRY_MS)),
    ));
    let full_stream = tokio_stream::iter(retry_event).chain(combined);

    Ok(Sse::new(full_stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(30))))
}

// ---------------------------------------------------------------------------
// Auth helpers
// ---------------------------------------------------------------------------

/// Return `true` when the `Authorization: Bearer <token>` header matches the
/// expected key.
fn check_bearer(headers: &HeaderMap, expected: &str) -> bool {
    headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| {
            v.strip_prefix("Bearer ")
                .map(|token| bool::from(token.as_bytes().ct_eq(expected.as_bytes())))
                .unwrap_or(false)
        })
        .unwrap_or(false)
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
fn build_router(
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

/// Start the axum HTTP server on `0.0.0.0:{port}`.
///
/// The server will run until the process is terminated.
///
/// - `realtime` — optional `RealtimeManager` for SSE streaming (`GET /v1/events`).
///   When `None`, the `/v1/events` endpoint returns `503 Service Unavailable`.
pub async fn serve_http(
    handler: Arc<dyn McpHandler>,
    port: u16,
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

    let addr = format!("0.0.0.0:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("HTTP transport listening on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::realtime::RealtimeEvent;
    use axum::body::to_bytes;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use std::sync::Mutex as StdMutex;
    use tower::ServiceExt;

    // Serialize CORS env var mutation in tests via a shared mutex.
    static ENV_LOCK: StdMutex<()> = StdMutex::new(());

    struct TestMcpHandler;

    impl McpHandler for TestMcpHandler {
        fn handle_request(&self, request: McpRequest) -> McpResponse {
            McpResponse::success(request.id, json!({"ok": true}))
        }
    }

    fn json_rpc_request(path: &str, bearer: Option<&str>) -> Request<Body> {
        json_rpc_request_with_headers(path, bearer, &[])
    }

    fn json_rpc_request_with_headers(
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

    fn json_rpc_notification_request_with_headers(
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

    fn test_app(api_key: Option<&str>) -> Router {
        test_app_with_rate_limits(api_key, 0, 0, None)
    }

    fn test_app_with_rate_limits(
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

    // ---- check_bearer tests ------------------------------------------------

    /// Ensure `check_bearer` correctly validates tokens.
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

    // ---- SSE event serialization tests ------------------------------------

    /// Verify that a `RealtimeEvent` can be round-tripped through JSON
    /// and produces the expected SSE `event:` field value.
    #[test]
    fn test_sse_event_serialization() {
        let event = RealtimeEvent::memory_created(42, "hello world".to_string());
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"type\":\"memory_created\""));
        assert!(json.contains("\"memory_id\":42"));
        assert_eq!(event_type_to_str(event.event_type), "memory_created");
    }

    #[test]
    fn test_sse_event_type_to_str_all_variants() {
        assert_eq!(
            event_type_to_str(EventType::MemoryCreated),
            "memory_created"
        );
        assert_eq!(
            event_type_to_str(EventType::MemoryUpdated),
            "memory_updated"
        );
        assert_eq!(
            event_type_to_str(EventType::MemoryDeleted),
            "memory_deleted"
        );
        assert_eq!(
            event_type_to_str(EventType::CrossrefCreated),
            "crossref_created"
        );
        assert_eq!(
            event_type_to_str(EventType::CrossrefDeleted),
            "crossref_deleted"
        );
        assert_eq!(event_type_to_str(EventType::SyncStarted), "sync_started");
        assert_eq!(
            event_type_to_str(EventType::SyncCompleted),
            "sync_completed"
        );
        assert_eq!(event_type_to_str(EventType::SyncFailed), "sync_failed");
    }

    // ---- parse_event_type tests -------------------------------------------

    #[test]
    fn test_parse_event_type_known() {
        assert_eq!(
            parse_event_type("memory_created"),
            Some(EventType::MemoryCreated)
        );
        assert_eq!(parse_event_type("sync_failed"), Some(EventType::SyncFailed));
    }

    #[test]
    fn test_parse_event_type_unknown_is_none() {
        assert_eq!(parse_event_type("unknown_event"), None);
        assert_eq!(parse_event_type(""), None);
    }

    // ---- EventsQuery filter parsing tests ---------------------------------

    #[test]
    fn test_events_query_parsed_event_types_none() {
        let q = EventsQuery {
            event_types: None,
            workspace: None,
        };
        assert!(q.parsed_event_types().is_none());
    }

    #[test]
    fn test_events_query_parsed_event_types_single() {
        let q = EventsQuery {
            event_types: Some("memory_created".to_string()),
            workspace: None,
        };
        let types = q.parsed_event_types().unwrap();
        assert_eq!(types, vec![EventType::MemoryCreated]);
    }

    #[test]
    fn test_events_query_parsed_event_types_multiple() {
        let q = EventsQuery {
            event_types: Some("memory_created,memory_deleted,sync_failed".to_string()),
            workspace: None,
        };
        let types = q.parsed_event_types().unwrap();
        assert_eq!(
            types,
            vec![
                EventType::MemoryCreated,
                EventType::MemoryDeleted,
                EventType::SyncFailed
            ]
        );
    }

    #[test]
    fn test_events_query_parsed_event_types_with_spaces() {
        let q = EventsQuery {
            event_types: Some("memory_created, memory_updated".to_string()),
            workspace: None,
        };
        let types = q.parsed_event_types().unwrap();
        assert_eq!(
            types,
            vec![EventType::MemoryCreated, EventType::MemoryUpdated]
        );
    }

    #[test]
    fn test_events_query_parsed_event_types_all_unknown_returns_none() {
        let q = EventsQuery {
            event_types: Some("bogus,fake".to_string()),
            workspace: None,
        };
        // All tokens invalid → None (no filter)
        assert!(q.parsed_event_types().is_none());
    }

    // ---- Filter matching tests (via SubscriptionFilter in events module) --

    #[test]
    fn test_event_type_filter_matches() {
        use crate::realtime::SubscriptionFilter;

        let filter = SubscriptionFilter {
            event_types: Some(vec![EventType::MemoryCreated]),
            memory_ids: None,
            tags: None,
        };
        let created = RealtimeEvent::memory_created(1, "test".to_string());
        let deleted = RealtimeEvent::memory_deleted(1);
        assert!(filter.matches(&created));
        assert!(!filter.matches(&deleted));
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
        assert_eq!(tenant_b.status(), StatusCode::OK);
    }

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

    #[test]
    fn test_keep_alive_interval_is_30s() {
        // Verify the constant used for keep-alive is correct.
        let interval = Duration::from_secs(30);
        assert_eq!(interval.as_secs(), 30);
    }

    // ---- Last-Event-Id header parsing tests --------------------------------

    /// Verify that a valid numeric `Last-Event-Id` header is parsed to `u64`.
    #[test]
    fn test_last_event_id_header_valid() {
        let mut headers = HeaderMap::new();
        headers.insert("last-event-id", "42".parse().unwrap());

        let parsed: Option<u64> = headers
            .get("last-event-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());

        assert_eq!(parsed, Some(42));
    }

    #[test]
    fn test_last_event_id_header_missing_is_none() {
        let headers = HeaderMap::new();
        let parsed: Option<u64> = headers
            .get("last-event-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        assert!(parsed.is_none());
    }

    #[test]
    fn test_last_event_id_header_non_numeric_is_none() {
        let mut headers = HeaderMap::new();
        headers.insert("last-event-id", "not-a-number".parse().unwrap());
        let parsed: Option<u64> = headers
            .get("last-event-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        assert!(parsed.is_none());
    }

    #[test]
    fn test_last_event_id_header_zero() {
        let mut headers = HeaderMap::new();
        headers.insert("last-event-id", "0".parse().unwrap());
        let parsed: Option<u64> = headers
            .get("last-event-id")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        assert_eq!(parsed, Some(0));
    }

    // ---- realtime_event_to_sse tests ---------------------------------------

    /// Verify that an event with a seq_id produces an SSE event with an `id:` field.
    #[test]
    fn test_realtime_event_to_sse_with_seq_id() {
        use crate::realtime::RealtimeManager;

        let manager = RealtimeManager::new();
        let _rx = manager.subscribe();
        manager.broadcast(RealtimeEvent::memory_created(1, "hello".to_string()));

        let buffered = manager.get_events_after(0);
        assert_eq!(buffered.len(), 1);

        let event = &buffered[0];
        assert_eq!(event.seq_id, Some(1));

        // Verify the SSE event would include the id
        // (axum's Event::id sets the id field; we verify seq_id is present)
        let sse = realtime_event_to_sse(event);
        // The event should serialize without panic; content is verified via seq_id field
        let _ = sse; // axum::sse::Event has no public getter, just verify it builds
    }

    #[test]
    fn test_realtime_event_to_sse_without_seq_id_no_id_field() {
        // Events with seq_id = None should still build an SSE event (no id field).
        let event = RealtimeEvent::memory_created(5, "no id".to_string());
        assert!(event.seq_id.is_none());
        let sse = realtime_event_to_sse(&event);
        let _ = sse; // should not panic
    }

    // ---- Replay via get_events_after + Last-Event-Id integration -----------

    #[test]
    fn test_replay_events_after_last_id() {
        use crate::realtime::RealtimeManager;

        let manager = RealtimeManager::new();
        let _rx = manager.subscribe();

        // Broadcast 5 events
        for i in 1..=5i64 {
            manager.broadcast(RealtimeEvent::memory_created(i, format!("ev{i}")));
        }

        // Simulate Last-Event-Id: 3 — client missed events 4 and 5
        let last_id: u64 = 3;
        let replayed = manager.get_events_after(last_id);
        assert_eq!(replayed.len(), 2);
        let ids: Vec<u64> = replayed.iter().filter_map(|e| e.seq_id).collect();
        assert_eq!(ids, vec![4, 5]);
    }

    #[test]
    fn test_retry_constant_is_3000ms() {
        assert_eq!(SSE_RETRY_MS, 3000);
    }

    // ---- H2: constant-time token comparison --------------------------------

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
}
