use std::time::{Duration, Instant};

use super::super::rate_limit::{apply_rate_limit, RateLimitBucket};
use super::support::test_rate_limiter_state;

#[test]
fn test_rate_limit_cleans_stale_buckets_under_max_bucket_pressure() {
    let now = Instant::now();
    let stale_seen = now
        .checked_sub(Duration::from_secs(601))
        .expect("test instant should support stale offset");
    let fresh_seen = now
        .checked_sub(Duration::from_secs(1))
        .expect("test instant should support fresh offset");
    let mut limiter = test_rate_limiter_state(2, Duration::from_secs(600));
    limiter.buckets.insert(
        "ip:stale".to_string(),
        RateLimitBucket {
            last_seen: stale_seen,
            tokens: 1.0,
            last_refill_at: stale_seen,
        },
    );
    limiter.buckets.insert(
        "ip:fresh".to_string(),
        RateLimitBucket {
            last_seen: fresh_seen,
            tokens: 1.0,
            last_refill_at: fresh_seen,
        },
    );

    let decision = apply_rate_limit(&mut limiter, "ip:new".to_string(), now);

    assert!(decision.allowed);
    assert_eq!(decision.stale_cleanup, 1);
    assert_eq!(decision.eviction_cleanup, 0);
    assert!(!limiter.buckets.contains_key("ip:stale"));
    assert!(limiter.buckets.contains_key("ip:fresh"));
    assert!(limiter.buckets.contains_key("ip:new"));
}

#[test]
fn test_rate_limit_evicts_oldest_bucket_when_no_stale_bucket_exists() {
    let now = Instant::now();
    let older_seen = now
        .checked_sub(Duration::from_secs(10))
        .expect("test instant should support older offset");
    let newer_seen = now
        .checked_sub(Duration::from_secs(1))
        .expect("test instant should support newer offset");
    let mut limiter = test_rate_limiter_state(2, Duration::from_secs(600));
    limiter.buckets.insert(
        "ip:older".to_string(),
        RateLimitBucket {
            last_seen: older_seen,
            tokens: 1.0,
            last_refill_at: older_seen,
        },
    );
    limiter.buckets.insert(
        "ip:newer".to_string(),
        RateLimitBucket {
            last_seen: newer_seen,
            tokens: 1.0,
            last_refill_at: newer_seen,
        },
    );

    let decision = apply_rate_limit(&mut limiter, "ip:new".to_string(), now);

    assert!(decision.allowed);
    assert_eq!(decision.stale_cleanup, 0);
    assert_eq!(decision.eviction_cleanup, 1);
    assert!(!limiter.buckets.contains_key("ip:older"));
    assert!(limiter.buckets.contains_key("ip:newer"));
    assert!(limiter.buckets.contains_key("ip:new"));
}
