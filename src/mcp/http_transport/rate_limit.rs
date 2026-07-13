use std::collections::HashMap;
use std::time::{Duration, Instant};

use axum::http::{HeaderMap, StatusCode};

use super::super::protocol::McpResponse;
use super::AppState;

pub(super) const RATE_LIMIT_MAX_BUCKETS: usize = 10_000;
pub(super) const RATE_LIMIT_STALE_AFTER_SECS: u64 = 600;

#[derive(Clone)]
pub(super) struct RateLimiterConfig {
    pub(super) requests_per_second: f64,
    pub(super) burst: f64,
    pub(super) key_header: Option<String>,
    pub(super) max_buckets: usize,
    pub(super) stale_after: Duration,
}

pub(super) struct RateLimitBucket {
    pub(super) last_seen: Instant,
    pub(super) tokens: f64,
    pub(super) last_refill_at: Instant,
}

pub(super) struct RateLimiterState {
    pub(super) config: RateLimiterConfig,
    pub(super) buckets: HashMap<String, RateLimitBucket>,
}

pub(super) struct RateLimitDecision {
    pub(super) allowed: bool,
    pub(super) stale_cleanup: u64,
    pub(super) eviction_cleanup: u64,
}

pub(super) fn rate_limited_response(
    id: Option<serde_json::Value>,
    is_notification: bool,
) -> (StatusCode, serde_json::Value) {
    if is_notification {
        return (StatusCode::ACCEPTED, serde_json::Value::Null);
    }

    (
        StatusCode::TOO_MANY_REQUESTS,
        serde_json::to_value(McpResponse::error(
            id,
            -32005,
            "Too Many Requests".to_string(),
        ))
        .unwrap_or_else(|e| {
            tracing::error!(
                error = %e,
                "failed to serialize error response"
            );
            serde_json::Value::Null
        }),
    )
}

pub(super) async fn is_rate_limit_allowed(
    state: &AppState,
    headers: &HeaderMap,
    peer: Option<std::net::SocketAddr>,
) -> bool {
    let rate_limiter = match &state.rate_limiter {
        Some(rate_limiter) => rate_limiter,
        None => return true,
    };

    let now = Instant::now();
    let mut limiter = rate_limiter.lock().await;
    let config = limiter.config.clone();
    let bucket_key = rate_limit_key(&config, headers, state.security.client_ip(peer, headers));
    let decision = apply_rate_limit(&mut limiter, bucket_key, now);

    state
        .metrics
        .on_rate_limit_cleanup(decision.stale_cleanup, decision.eviction_cleanup);

    decision.allowed
}

pub(super) fn apply_rate_limit(
    limiter: &mut RateLimiterState,
    bucket_key: String,
    now: Instant,
) -> RateLimitDecision {
    let config = limiter.config.clone();
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

    let bucket = limiter
        .buckets
        .entry(bucket_key)
        .or_insert_with(|| RateLimitBucket {
            tokens: config.burst,
            last_seen: now,
            last_refill_at: now,
        });

    let elapsed = now
        .saturating_duration_since(bucket.last_refill_at)
        .as_secs_f64();
    let refill = elapsed * config.requests_per_second;
    bucket.tokens = (bucket.tokens + refill).min(config.burst);
    bucket.last_refill_at = now;
    bucket.last_seen = now;

    if bucket.tokens < 1.0 {
        return RateLimitDecision {
            allowed: false,
            stale_cleanup,
            eviction_cleanup,
        };
    }

    bucket.tokens -= 1.0;
    RateLimitDecision {
        allowed: true,
        stale_cleanup,
        eviction_cleanup,
    }
}

fn rate_limit_key(
    config: &RateLimiterConfig,
    headers: &HeaderMap,
    verified_ip: Option<std::net::IpAddr>,
) -> String {
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

    if let Some(ip) = verified_ip {
        return format!("ip:{ip}");
    }

    "ip:unknown".to_string()
}
