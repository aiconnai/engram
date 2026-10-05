//! In-process counters, outcome classes and latency distribution for the HTTP
//! transport (issues #161/#162 follow-ups, task O1).
//!
//! Every series has bounded cardinality: outcome classes and latency buckets
//! are fixed enums/arrays. Nothing here records ids, tenants, tool names,
//! queries or error text.

use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

use serde::Serialize;

/// How a request to `/mcp` or `/v1/mcp` ended. Separates auth, transport and
/// handler failures: an MCP-level error still travels as HTTP 200, so the
/// status code alone cannot tell them apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum McpOutcome {
    Success,
    Unauthorized,
    Forbidden,
    RateLimited,
    /// Body rejected before the handler (malformed JSON, wrong content type, bad shape).
    ParseError,
    /// Body over the configured limit (HTTP 413).
    BodyTooLarge,
    /// The transport request timeout fired (HTTP 408).
    Timeout,
    /// HTTP 200 carrying a JSON-RPC `error` object.
    ProtocolError,
    /// HTTP 200 carrying a tool result with `isError: true`.
    ToolError,
    /// The handler task panicked or was cancelled.
    HandlerPanic,
}

impl McpOutcome {
    pub(super) const ALL: [Self; 10] = [
        Self::Success,
        Self::Unauthorized,
        Self::Forbidden,
        Self::RateLimited,
        Self::ParseError,
        Self::BodyTooLarge,
        Self::Timeout,
        Self::ProtocolError,
        Self::ToolError,
        Self::HandlerPanic,
    ];

    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::RateLimited => "rate_limited",
            Self::ParseError => "parse_error",
            Self::BodyTooLarge => "body_too_large",
            Self::Timeout => "timeout",
            Self::ProtocolError => "protocol_error",
            Self::ToolError => "tool_error",
            Self::HandlerPanic => "handler_panic",
        }
    }

    pub(super) fn is_failure(self) -> bool {
        self != Self::Success
    }

    fn index(self) -> usize {
        Self::ALL.iter().position(|o| *o == self).unwrap_or(0)
    }
}

/// Upper bounds (ms) of the latency buckets; one more bucket catches the rest.
pub(super) const LATENCY_BOUNDS_MS: [u64; 11] = [1, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000];
const LATENCY_BUCKETS: usize = LATENCY_BOUNDS_MS.len() + 1;

struct LatencyHistogram {
    buckets: [AtomicU64; LATENCY_BUCKETS],
    count: AtomicU64,
    sum_nanos: AtomicU64,
}

impl LatencyHistogram {
    fn new() -> Self {
        Self {
            buckets: std::array::from_fn(|_| AtomicU64::new(0)),
            count: AtomicU64::new(0),
            sum_nanos: AtomicU64::new(0),
        }
    }

    fn record(&self, latency: Duration) {
        let index = LATENCY_BOUNDS_MS
            .iter()
            .position(|bound| latency <= Duration::from_millis(*bound))
            .unwrap_or(LATENCY_BOUNDS_MS.len());
        self.buckets[index].fetch_add(1, Ordering::Relaxed);
        self.count.fetch_add(1, Ordering::Relaxed);
        let nanos = u64::try_from(latency.as_nanos()).unwrap_or(u64::MAX);
        self.sum_nanos.fetch_add(nanos, Ordering::Relaxed);
    }

    fn snapshot(&self) -> LatencySnapshot {
        let per_bucket: Vec<u64> = self
            .buckets
            .iter()
            .map(|b| b.load(Ordering::Relaxed))
            .collect();
        let samples: u64 = per_bucket.iter().sum();
        let mut cumulative = 0u64;
        let buckets = per_bucket
            .iter()
            .enumerate()
            .map(|(i, n)| {
                cumulative += n;
                BucketSnapshot {
                    le_ms: LATENCY_BOUNDS_MS.get(i).copied(),
                    cumulative_count: cumulative,
                }
            })
            .collect();
        LatencySnapshot {
            samples,
            sum_ms: self.sum_nanos.load(Ordering::Relaxed) as f64 / 1_000_000.0,
            p50_ms: quantile_upper_bound(&per_bucket, samples, 0.50),
            p95_ms: quantile_upper_bound(&per_bucket, samples, 0.95),
            p99_ms: quantile_upper_bound(&per_bucket, samples, 0.99),
            buckets,
        }
    }
}

/// Upper bound (ms) of the bucket holding the `q` quantile. `None` when there
/// are no samples (no data is not zero latency). Samples above the largest
/// bound report that bound, i.e. the true value is at least that.
fn quantile_upper_bound(per_bucket: &[u64], samples: u64, q: f64) -> Option<u64> {
    if samples == 0 {
        return None;
    }
    let target = ((samples as f64) * q).ceil().max(1.0) as u64;
    let mut seen = 0u64;
    for (i, n) in per_bucket.iter().enumerate() {
        seen += n;
        if seen >= target {
            return Some(
                LATENCY_BOUNDS_MS
                    .get(i)
                    .copied()
                    .unwrap_or(LATENCY_BOUNDS_MS[LATENCY_BOUNDS_MS.len() - 1]),
            );
        }
    }
    None
}

#[derive(Serialize)]
pub(super) struct BucketSnapshot {
    /// Inclusive upper bound in ms; `null` is the overflow bucket.
    le_ms: Option<u64>,
    cumulative_count: u64,
}

#[derive(Serialize)]
pub(super) struct LatencySnapshot {
    /// Number of observations. Zero means no data, not fast.
    samples: u64,
    sum_ms: f64,
    p50_ms: Option<u64>,
    p95_ms: Option<u64>,
    p99_ms: Option<u64>,
    buckets: Vec<BucketSnapshot>,
}

#[derive(Serialize)]
pub(super) struct LatencyReport {
    all: LatencySnapshot,
    by_outcome: BTreeMap<&'static str, LatencySnapshot>,
}

pub(super) struct HttpTransportMetrics {
    mcp_requests_total: AtomicU64,
    mcp_requests_completed: AtomicU64,
    mcp_notifications_total: AtomicU64,
    mcp_rate_limited_total: AtomicU64,
    mcp_unauthorized_total: AtomicU64,
    mcp_failed_total: AtomicU64,
    mcp_success_total: AtomicU64,
    mcp_inflight_total: AtomicU64,
    mcp_latency_nanos: AtomicU64,
    mcp_abandoned_total: AtomicU64,
    mcp_timeouts_total: AtomicU64,
    mcp_timeouts_before_handler_total: AtomicU64,
    mcp_outcomes: [AtomicU64; McpOutcome::ALL.len()],
    mcp_latency_all: LatencyHistogram,
    mcp_latency_by_outcome: [LatencyHistogram; McpOutcome::ALL.len()],

    events_requests_total: AtomicU64,
    events_requests_unauthorized_total: AtomicU64,
    events_requests_no_realtime_total: AtomicU64,
    events_streams_total: AtomicU64,
    events_active: AtomicU64,

    rate_limit_buckets_stale_cleanups: AtomicU64,
    rate_limit_bucket_evictions: AtomicU64,
}

impl Default for HttpTransportMetrics {
    fn default() -> Self {
        Self {
            mcp_requests_total: AtomicU64::new(0),
            mcp_requests_completed: AtomicU64::new(0),
            mcp_notifications_total: AtomicU64::new(0),
            mcp_rate_limited_total: AtomicU64::new(0),
            mcp_unauthorized_total: AtomicU64::new(0),
            mcp_failed_total: AtomicU64::new(0),
            mcp_success_total: AtomicU64::new(0),
            mcp_inflight_total: AtomicU64::new(0),
            mcp_latency_nanos: AtomicU64::new(0),
            mcp_abandoned_total: AtomicU64::new(0),
            mcp_timeouts_total: AtomicU64::new(0),
            mcp_timeouts_before_handler_total: AtomicU64::new(0),
            mcp_outcomes: std::array::from_fn(|_| AtomicU64::new(0)),
            mcp_latency_all: LatencyHistogram::new(),
            mcp_latency_by_outcome: std::array::from_fn(|_| LatencyHistogram::new()),
            events_requests_total: AtomicU64::new(0),
            events_requests_unauthorized_total: AtomicU64::new(0),
            events_requests_no_realtime_total: AtomicU64::new(0),
            events_streams_total: AtomicU64::new(0),
            events_active: AtomicU64::new(0),
            rate_limit_buckets_stale_cleanups: AtomicU64::new(0),
            rate_limit_bucket_evictions: AtomicU64::new(0),
        }
    }
}

#[derive(Serialize)]
pub(super) struct HttpTransportMetricsSnapshot {
    mcp_requests_total: u64,
    mcp_requests_completed: u64,
    mcp_notifications_total: u64,
    mcp_rate_limited_total: u64,
    mcp_unauthorized_total: u64,
    mcp_failed_total: u64,
    mcp_success_total: u64,
    mcp_inflight_total: u64,
    mcp_avg_latency_ms: f64,
    /// Requests whose handler was dropped before answering (client gone or timeout).
    mcp_abandoned_total: u64,
    /// Requests answered 408 by the transport timeout.
    mcp_timeouts_total: u64,
    /// Subset of the timeouts that fired before the request was counted by any
    /// stage (body still being read). Keeps the invariant
    /// `mcp_requests_total = completed + inflight + abandoned + this`.
    mcp_timeouts_before_handler_total: u64,
    /// Finished requests by outcome class (auth, transport, protocol, tool, handler).
    mcp_outcomes: BTreeMap<&'static str, u64>,
    mcp_latency: LatencyReport,
    events_requests_total: u64,
    events_requests_unauthorized_total: u64,
    events_requests_no_realtime_total: u64,
    /// SSE streams opened since start.
    events_streams_total: u64,
    /// SSE streams open right now.
    events_active: u64,
    rate_limit_buckets_stale_cleanups: u64,
    rate_limit_bucket_evictions: u64,
}

/// Request extension shared between the timeout middleware and the stages that
/// count a request (auth rejection, body rejection, handler). A stage calls
/// [`RequestCounted::mark`] when it adds the request to `mcp_requests_total`.
#[derive(Clone, Default)]
pub(super) struct RequestCounted(Arc<AtomicBool>);

impl RequestCounted {
    pub(super) fn mark(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub(super) fn is_marked(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// Tracks one request inside the MCP handler. If it is dropped without
/// [`InflightGuard::finish`] (client disconnect, transport timeout) the request
/// is counted as abandoned and the in-flight gauge is released, so the gauge
/// cannot leak.
pub(super) struct InflightGuard {
    metrics: Arc<HttpTransportMetrics>,
    started: Instant,
    finished: bool,
    /// Correlation id and route, so an abandoned request can be logged.
    context: Option<(String, &'static str)>,
}

impl InflightGuard {
    pub(super) fn with_context(mut self, correlation_id: &str, route: &'static str) -> Self {
        self.context = Some((correlation_id.to_string(), route));
        self
    }

    pub(super) fn finish(mut self, outcome: McpOutcome) -> Duration {
        self.finished = true;
        let latency = self.started.elapsed();
        self.metrics.on_mcp_request_complete(outcome, latency);
        latency
    }
}

impl Drop for InflightGuard {
    fn drop(&mut self) {
        if !self.finished {
            self.metrics.on_mcp_abandoned();
            // Client disconnect or transport timeout dropped the handler future.
            if let Some((id, route)) = &self.context {
                crate::observability::OperationEvent::new(
                    "http.mcp",
                    id,
                    "abandoned",
                    self.started.elapsed(),
                )
                .route(route)
                .failed(true)
                .emit();
            }
        }
    }
}

/// Keeps `events_active` accurate for the lifetime of one SSE stream.
pub(super) struct SseGuard {
    metrics: Arc<HttpTransportMetrics>,
}

impl Drop for SseGuard {
    fn drop(&mut self) {
        let _ =
            self.metrics
                .events_active
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1));
    }
}

fn decrement(counter: &AtomicU64) {
    let _ = counter.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_sub(1));
}

impl HttpTransportMetrics {
    fn record_outcome(&self, outcome: McpOutcome, latency: Duration) {
        self.mcp_outcomes[outcome.index()].fetch_add(1, Ordering::Relaxed);
        self.mcp_latency_all.record(latency);
        self.mcp_latency_by_outcome[outcome.index()].record(latency);
    }

    /// Rejected by the auth middleware before the body was read.
    pub(super) fn on_mcp_preparse_unauthorized(&self, latency: Duration) {
        self.mcp_requests_total.fetch_add(1, Ordering::Relaxed);
        self.mcp_requests_completed.fetch_add(1, Ordering::Relaxed);
        self.mcp_unauthorized_total.fetch_add(1, Ordering::Relaxed);
        self.mcp_failed_total.fetch_add(1, Ordering::Relaxed);
        self.record_outcome(McpOutcome::Unauthorized, latency);
    }

    /// Rejected by the body extractor before the handler ran (parse error,
    /// content type, body limit).
    pub(super) fn on_mcp_rejected_before_handler(&self, outcome: McpOutcome, latency: Duration) {
        self.mcp_requests_total.fetch_add(1, Ordering::Relaxed);
        self.mcp_requests_completed.fetch_add(1, Ordering::Relaxed);
        self.mcp_failed_total.fetch_add(1, Ordering::Relaxed);
        self.record_outcome(outcome, latency);
    }

    /// The transport request timeout answered 408. `already_counted` is true when
    /// a later stage had already counted the request in `mcp_requests_total`
    /// (its handler guard then records it as abandoned); otherwise the request is
    /// counted here so the counters stay balanced.
    pub(super) fn on_mcp_timeout(&self, latency: Duration, already_counted: bool) {
        self.mcp_timeouts_total.fetch_add(1, Ordering::Relaxed);
        if !already_counted {
            self.mcp_requests_total.fetch_add(1, Ordering::Relaxed);
            self.mcp_timeouts_before_handler_total
                .fetch_add(1, Ordering::Relaxed);
        }
        self.record_outcome(McpOutcome::Timeout, latency);
    }

    pub(super) fn start_mcp_request(
        self: &Arc<Self>,
        is_notification: bool,
        started: Instant,
    ) -> InflightGuard {
        self.mcp_requests_total.fetch_add(1, Ordering::Relaxed);
        if is_notification {
            self.mcp_notifications_total.fetch_add(1, Ordering::Relaxed);
        }
        self.mcp_inflight_total.fetch_add(1, Ordering::Relaxed);
        InflightGuard {
            metrics: Arc::clone(self),
            started,
            finished: false,
            context: None,
        }
    }

    fn on_mcp_abandoned(&self) {
        decrement(&self.mcp_inflight_total);
        self.mcp_abandoned_total.fetch_add(1, Ordering::Relaxed);
    }

    fn on_mcp_request_complete(&self, outcome: McpOutcome, latency: Duration) {
        self.mcp_requests_completed.fetch_add(1, Ordering::Relaxed);
        decrement(&self.mcp_inflight_total);
        let latency_nanos = u64::try_from(latency.as_nanos()).unwrap_or(u64::MAX);
        self.mcp_latency_nanos
            .fetch_add(latency_nanos, Ordering::Relaxed);
        self.record_outcome(outcome, latency);

        match outcome {
            McpOutcome::Success => {
                self.mcp_success_total.fetch_add(1, Ordering::Relaxed);
            }
            McpOutcome::RateLimited => {
                self.mcp_rate_limited_total.fetch_add(1, Ordering::Relaxed);
                self.mcp_failed_total.fetch_add(1, Ordering::Relaxed);
            }
            McpOutcome::Unauthorized => {
                self.mcp_unauthorized_total.fetch_add(1, Ordering::Relaxed);
                self.mcp_failed_total.fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                self.mcp_failed_total.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub(super) fn on_events_request(&self, is_unauthorized: bool, is_no_realtime: bool) {
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

    /// An SSE stream was accepted; hold the guard for as long as it is open.
    pub(super) fn on_events_stream_opened(self: &Arc<Self>) -> SseGuard {
        self.events_streams_total.fetch_add(1, Ordering::Relaxed);
        self.events_active.fetch_add(1, Ordering::Relaxed);
        SseGuard {
            metrics: Arc::clone(self),
        }
    }

    pub(super) fn on_rate_limit_cleanup(&self, stale: u64, evictions: u64) {
        if stale > 0 {
            self.rate_limit_buckets_stale_cleanups
                .fetch_add(stale, Ordering::Relaxed);
        }
        if evictions > 0 {
            self.rate_limit_bucket_evictions
                .fetch_add(evictions, Ordering::Relaxed);
        }
    }

    pub(super) fn snapshot(&self) -> HttpTransportMetricsSnapshot {
        let completed = self.mcp_requests_completed.load(Ordering::Relaxed);
        let latency_nanos = self.mcp_latency_nanos.load(Ordering::Relaxed);
        let avg_latency_ms = if completed == 0 {
            0.0
        } else {
            (latency_nanos as f64 / completed as f64) / 1_000_000.0
        };
        let load = |a: &AtomicU64| a.load(Ordering::Relaxed);

        HttpTransportMetricsSnapshot {
            mcp_requests_total: load(&self.mcp_requests_total),
            mcp_requests_completed: completed,
            mcp_notifications_total: load(&self.mcp_notifications_total),
            mcp_rate_limited_total: load(&self.mcp_rate_limited_total),
            mcp_unauthorized_total: load(&self.mcp_unauthorized_total),
            mcp_failed_total: load(&self.mcp_failed_total),
            mcp_success_total: load(&self.mcp_success_total),
            mcp_inflight_total: load(&self.mcp_inflight_total),
            mcp_avg_latency_ms: avg_latency_ms,
            mcp_abandoned_total: load(&self.mcp_abandoned_total),
            mcp_timeouts_total: load(&self.mcp_timeouts_total),
            mcp_timeouts_before_handler_total: load(&self.mcp_timeouts_before_handler_total),
            mcp_outcomes: McpOutcome::ALL
                .iter()
                .map(|o| (o.as_str(), load(&self.mcp_outcomes[o.index()])))
                .collect(),
            mcp_latency: LatencyReport {
                all: self.mcp_latency_all.snapshot(),
                by_outcome: McpOutcome::ALL
                    .iter()
                    .map(|o| {
                        (
                            o.as_str(),
                            self.mcp_latency_by_outcome[o.index()].snapshot(),
                        )
                    })
                    .collect(),
            },
            events_requests_total: load(&self.events_requests_total),
            events_requests_unauthorized_total: load(&self.events_requests_unauthorized_total),
            events_requests_no_realtime_total: load(&self.events_requests_no_realtime_total),
            events_streams_total: load(&self.events_streams_total),
            events_active: load(&self.events_active),
            rate_limit_buckets_stale_cleanups: load(&self.rate_limit_buckets_stale_cleanups),
            rate_limit_bucket_evictions: load(&self.rate_limit_bucket_evictions),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn histogram_reports_no_data_as_null_not_zero() {
        let h = LatencyHistogram::new();
        let empty = h.snapshot();
        assert_eq!(empty.samples, 0);
        assert_eq!(empty.p95_ms, None);
    }

    #[test]
    fn histogram_quantiles_are_bucket_upper_bounds() {
        let h = LatencyHistogram::new();
        for _ in 0..90 {
            h.record(Duration::from_millis(3));
        }
        for _ in 0..9 {
            h.record(Duration::from_millis(40));
        }
        h.record(Duration::from_secs(20));
        let s = h.snapshot();
        assert_eq!(s.samples, 100);
        assert_eq!(s.p50_ms, Some(5));
        assert_eq!(s.p95_ms, Some(50));
        assert_eq!(s.p99_ms, Some(50));
        // The 20s sample is only visible in the overflow bucket.
        assert_eq!(s.buckets[LATENCY_BOUNDS_MS.len() - 1].cumulative_count, 99);
        assert_eq!(s.buckets.last().map(|b| b.cumulative_count), Some(100));
        assert_eq!(s.buckets.last().and_then(|b| b.le_ms), None);
    }

    #[test]
    fn inflight_gauge_is_released_when_the_handler_is_dropped() {
        let metrics = Arc::new(HttpTransportMetrics::default());
        let guard = metrics.start_mcp_request(false, Instant::now());
        assert_eq!(metrics.snapshot().mcp_inflight_total, 1);
        drop(guard);
        let snap = metrics.snapshot();
        assert_eq!(snap.mcp_inflight_total, 0);
        assert_eq!(snap.mcp_abandoned_total, 1);
        assert_eq!(snap.mcp_requests_completed, 0);
    }

    #[test]
    fn sse_gauge_tracks_open_streams() {
        let metrics = Arc::new(HttpTransportMetrics::default());
        let a = metrics.on_events_stream_opened();
        let b = metrics.on_events_stream_opened();
        assert_eq!(metrics.snapshot().events_active, 2);
        drop(a);
        assert_eq!(metrics.snapshot().events_active, 1);
        drop(b);
        let snap = metrics.snapshot();
        assert_eq!((snap.events_active, snap.events_streams_total), (0, 2));
    }
}
