//! Local alert rules over the `/health` export (task O1).
//!
//! Each rule ties one signal to an owner, a runbook entry in
//! `docs/OPERATIONS.md` and a rollback. Rules are evaluated locally over the
//! JSON the process already exports; nothing is sent anywhere. They describe
//! availability of *this process since it started*, not a deployment SLO: a
//! real SLO needs rate windows and long-lived storage that only the operator's
//! monitoring stack has.
//!
//! A rule without enough samples answers [`AlertStatus::NoData`], never `Ok`:
//! an empty dashboard is not a green one.

use serde::Serialize;
use serde_json::Value;

/// Minimum finished requests before a ratio rule may say anything.
const MIN_RATIO_SAMPLES: u64 = 50;
/// Minimum latency observations before a quantile rule may say anything.
const MIN_LATENCY_SAMPLES: u64 = 20;
/// Auth rejections above this share of requests fire the auth alert
/// (the `Auth failure rate < 2%` planning baseline).
const AUTH_FAILURE_RATE: f64 = 0.02;
/// Rate-limited requests above this share of requests fire the abuse alert.
const RATE_LIMITED_RATE: f64 = 0.05;
/// p95 above this (ms) fires the latency alert (docs: `p95_latency > 500ms`).
const P95_LATENCY_MS: u64 = 500;
/// Permission denials since start above this fire the denial alert.
const PERMISSION_DENIED_BURST: u64 = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertStatus {
    Firing,
    Ok,
    /// Not enough samples, or the signal is not exported. Never read as healthy.
    NoData,
}

/// Static description of one rule; the documented alert catalog.
#[derive(Debug, Clone, Copy)]
pub struct AlertRule {
    pub id: &'static str,
    /// Role and code owner that triages the alert.
    pub owner: &'static str,
    /// Heading of the runbook entry in `docs/OPERATIONS.md`.
    pub runbook: &'static str,
    pub rollback: &'static str,
    evaluate: fn(&Value) -> Signal,
}

enum Signal {
    Firing(String),
    Healthy,
    NoData(&'static str),
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AlertEvaluation {
    pub id: &'static str,
    pub status: AlertStatus,
    pub owner: &'static str,
    pub runbook: &'static str,
    pub rollback: &'static str,
    pub detail: String,
}

fn count(health: &Value, pointer: &str) -> Option<u64> {
    health.pointer(pointer).and_then(Value::as_u64)
}

fn http_total(health: &Value) -> Option<u64> {
    count(health, "/transport/http/mcp_requests_completed")
}

fn outcome(health: &Value, name: &str) -> Option<u64> {
    count(health, &format!("/transport/http/mcp_outcomes/{name}"))
}

/// A failure counter judged against the number of attempts it could have
/// failed in. A non-zero count fires (it is evidence by itself). A zero count is
/// `Healthy` only when the denominator shows there were attempts: zero failures
/// out of zero attempts is no data, not a clean bill of health.
fn gated_signal(
    failures: Option<u64>,
    attempts: Option<u64>,
    what: &str,
    missing: &'static str,
) -> Signal {
    match (failures, attempts) {
        (None, _) => Signal::NoData(missing),
        (Some(n), _) if n > 0 => Signal::Firing(format!("{what} = {n} since process start")),
        (Some(_), Some(a)) if a > 0 => Signal::Healthy,
        (Some(_), _) => Signal::NoData("no attempts since process start (denominator is zero)"),
    }
}

const OBSERVABILITY_OFF: &str = "observability export is off or absent";

fn http_started(health: &Value) -> Option<u64> {
    count(health, "/transport/http/mcp_requests_total")
}

fn ratio_signal(health: &Value, name: &str, limit: f64) -> Signal {
    let Some(total) = http_total(health).filter(|t| *t >= MIN_RATIO_SAMPLES) else {
        return Signal::NoData("fewer than 50 finished requests");
    };
    let Some(hits) = outcome(health, name) else {
        return Signal::NoData("outcome not exported");
    };
    let rate = hits as f64 / total as f64;
    if rate > limit {
        Signal::Firing(format!(
            "{name} rate {rate:.3} over {limit} ({hits}/{total})"
        ))
    } else {
        Signal::Healthy
    }
}

fn handler_panic(health: &Value) -> Signal {
    gated_signal(
        outcome(health, "handler_panic"),
        http_started(health),
        "handler panics",
        "outcome not exported",
    )
}

fn recovery_failed(health: &Value) -> Signal {
    let attempts = count(health, "/observability/recovery_total/succeeded")
        .zip(count(health, "/observability/recovery_total/failed"))
        .map(|(ok, failed)| ok + failed);
    gated_signal(
        count(health, "/observability/recovery_total/failed"),
        attempts,
        "failed recoveries",
        OBSERVABILITY_OFF,
    )
}

fn p95_latency(health: &Value) -> Signal {
    let samples = count(health, "/transport/http/mcp_latency/all/samples").unwrap_or(0);
    if samples < MIN_LATENCY_SAMPLES {
        return Signal::NoData("fewer than 20 latency samples");
    }
    match count(health, "/transport/http/mcp_latency/all/p95_ms") {
        Some(p95) if p95 > P95_LATENCY_MS => {
            Signal::Firing(format!("p95 upper bound {p95} ms over {P95_LATENCY_MS} ms"))
        }
        Some(_) => Signal::Healthy,
        None => Signal::NoData("p95 not exported"),
    }
}

fn permission_denied_burst(health: &Value) -> Signal {
    let Some(map) = health
        .pointer("/observability/permission_denied_total")
        .and_then(Value::as_object)
    else {
        return Signal::NoData(OBSERVABILITY_OFF);
    };
    let total: u64 = map.values().filter_map(Value::as_u64).sum();
    if total > PERMISSION_DENIED_BURST {
        return Signal::Firing(format!("{total} permission denials since process start"));
    }
    match count(health, "/observability/authorization_checks_total") {
        Some(checks) if checks > 0 => Signal::Healthy,
        _ => Signal::NoData("no authorization checks since process start (denominator is zero)"),
    }
}

/// The documented alert catalog. Every `runbook` heading and every `id` must
/// appear in `docs/OPERATIONS.md` (enforced by a test).
pub const RULES: &[AlertRule] = &[
    AlertRule {
        id: "mcp_handler_panic",
        owner: "http-transport (src/mcp/http_transport)",
        runbook: "Runbook: mcp_handler_panic",
        rollback: "Redeploy the previous binary; the failing request is identified by its x-request-id.",
        evaluate: handler_panic,
    },
    AlertRule {
        id: "mcp_timeouts",
        owner: "http-transport (src/mcp/http_transport)",
        runbook: "Runbook: mcp_timeouts",
        rollback: "Restore the previous ENGRAM_HTTP_REQUEST_TIMEOUT_MS and shed the slow tool's callers.",
        evaluate: |h| {
            gated_signal(
                count(h, "/transport/http/mcp_timeouts_total"),
                http_started(h),
                "transport timeouts",
                "timeouts not exported",
            )
        },
    },
    AlertRule {
        id: "mcp_auth_failure_rate",
        owner: "security (src/auth, src/mcp/http_transport)",
        runbook: "Runbook: mcp_auth_failure_rate",
        rollback: "Revert the API key / proxy change that started the rejections; keep the key as is if it is an attack.",
        evaluate: |h| ratio_signal(h, "unauthorized", AUTH_FAILURE_RATE),
    },
    AlertRule {
        id: "mcp_rate_limited_rate",
        owner: "http-transport (src/mcp/http_transport/rate_limit.rs)",
        runbook: "Runbook: mcp_rate_limited_rate",
        rollback: "Restore the previous --http-rate-limit-rps/--http-rate-limit-burst values.",
        evaluate: |h| ratio_signal(h, "rate_limited", RATE_LIMITED_RATE),
    },
    AlertRule {
        id: "mcp_latency_p95",
        owner: "http-transport (src/mcp/http_transport)",
        runbook: "Runbook: mcp_latency_p95",
        rollback: "Roll back the release that moved the p95; tighten the rate limit while investigating.",
        evaluate: p95_latency,
    },
    AlertRule {
        id: "sqlite_busy",
        owner: "storage (src/storage)",
        runbook: "Runbook: sqlite_busy",
        rollback: "Stop the second writer; confirm the singleton storage lock is held by one server.",
        evaluate: |h| {
            gated_signal(
                count(h, "/observability/sqlite_busy_total"),
                count(h, "/observability/storage_operations_total"),
                "SQLITE_BUSY conversions",
                OBSERVABILITY_OFF,
            )
        },
    },
    AlertRule {
        id: "embedding_provider_timeout",
        owner: "embeddings (src/embedding)",
        runbook: "Runbook: embedding_provider_timeout",
        rollback: "Switch ENGRAM_EMBEDDING_MODEL back to the last working provider or tfidf; jobs stay queued.",
        evaluate: |h| {
            gated_signal(
                count(h, "/observability/provider_timeout_total/embedding"),
                count(h, "/observability/provider_calls_total/embedding"),
                "embedding provider timeouts",
                OBSERVABILITY_OFF,
            )
        },
    },
    AlertRule {
        id: "replication_recover_failed",
        owner: "recovery (src/sync/wal_recovery*.rs)",
        runbook: "Runbook: replication_recover_failed",
        rollback: "Keep the active database untouched; discard the staging target and retry from a closed copy.",
        evaluate: recovery_failed,
    },
    AlertRule {
        id: "permission_denied_burst",
        owner: "security (src/mcp/permission.rs, src/mcp/workspace_guard.rs)",
        runbook: "Runbook: permission_denied_burst",
        rollback: "Revert the permission-mode or principal configuration change; denials are fail-closed, so no data rollback is needed.",
        evaluate: permission_denied_burst,
    },
];

/// Evaluate every rule over a `/health` JSON document.
pub fn evaluate_all(health: &Value) -> Vec<AlertEvaluation> {
    RULES
        .iter()
        .map(|rule| {
            let (status, detail) = match (rule.evaluate)(health) {
                Signal::Firing(detail) => (AlertStatus::Firing, detail),
                Signal::Healthy => (AlertStatus::Ok, String::new()),
                Signal::NoData(why) => (AlertStatus::NoData, why.to_string()),
            };
            AlertEvaluation {
                id: rule.id,
                status,
                owner: rule.owner,
                runbook: rule.runbook,
                rollback: rule.rollback,
                detail,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn status_of(evals: &[AlertEvaluation], id: &str) -> AlertStatus {
        evals
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.status)
            .expect("rule exists")
    }

    #[test]
    fn empty_export_is_no_data_never_ok() {
        let evals = evaluate_all(&json!({}));
        assert!(
            evals.iter().all(|e| e.status == AlertStatus::NoData),
            "{evals:?}"
        );
    }

    #[test]
    fn zero_failures_without_attempts_is_no_data_for_every_rule() {
        // Counters exported and zero, but every denominator is zero too.
        let health = json!({
            "transport": {"http": {
                "mcp_requests_total": 0,
                "mcp_requests_completed": 0,
                "mcp_outcomes": {"unauthorized": 0, "rate_limited": 0, "handler_panic": 0},
                "mcp_timeouts_total": 0,
                "mcp_latency": {"all": {"samples": 0, "p95_ms": null}}
            }},
            "observability": {
                "permission_denied_total": {"mode_insufficient": 0},
                "authorization_checks_total": 0,
                "storage_operations_total": 0,
                "sqlite_busy_total": 0,
                "provider_calls_total": {"embedding": 0},
                "provider_timeout_total": {"embedding": 0},
                "recovery_total": {"succeeded": 0, "failed": 0, "rejected": 3}
            }
        });
        let evals = evaluate_all(&health);
        assert!(
            evals.iter().all(|e| e.status == AlertStatus::NoData),
            "{evals:?}"
        );
    }

    #[test]
    fn zero_failures_with_attempts_is_ok_and_few_requests_keep_ratio_rules_no_data() {
        let health = json!({
            "transport": {"http": {
                "mcp_requests_total": 3,
                "mcp_requests_completed": 3,
                "mcp_outcomes": {"unauthorized": 0, "rate_limited": 0, "handler_panic": 0},
                "mcp_timeouts_total": 0,
                "mcp_latency": {"all": {"samples": 3, "p95_ms": 5}}
            }},
            "observability": {
                "permission_denied_total": {"mode_insufficient": 0},
                "authorization_checks_total": 4,
                "storage_operations_total": 9,
                "sqlite_busy_total": 0,
                "provider_calls_total": {"embedding": 2},
                "provider_timeout_total": {"embedding": 0},
                "recovery_total": {"succeeded": 1, "failed": 0, "rejected": 0}
            }
        });
        let evals = evaluate_all(&health);
        for id in [
            "mcp_handler_panic",
            "mcp_timeouts",
            "sqlite_busy",
            "embedding_provider_timeout",
            "replication_recover_failed",
            "permission_denied_burst",
        ] {
            assert_eq!(status_of(&evals, id), AlertStatus::Ok, "{id}");
        }
        for id in [
            "mcp_auth_failure_rate",
            "mcp_rate_limited_rate",
            "mcp_latency_p95",
        ] {
            assert_eq!(status_of(&evals, id), AlertStatus::NoData, "{id}");
        }
    }

    #[test]
    fn thresholds_fire_with_enough_samples() {
        let health = json!({
            "transport": {"http": {
                "mcp_requests_completed": 100,
                "mcp_outcomes": {"unauthorized": 10, "rate_limited": 1, "handler_panic": 1},
                "mcp_timeouts_total": 2,
                "mcp_latency": {"all": {"samples": 100, "p95_ms": 1000}}
            }},
            "observability": {
                "permission_denied_total": {"mode_insufficient": 30},
                "sqlite_busy_total": 1,
                "provider_timeout_total": {"embedding": 1},
                "recovery_total": {"failed": 1, "succeeded": 0}
            }
        });
        let evals = evaluate_all(&health);
        for id in [
            "mcp_handler_panic",
            "mcp_timeouts",
            "mcp_auth_failure_rate",
            "mcp_latency_p95",
            "sqlite_busy",
            "embedding_provider_timeout",
            "replication_recover_failed",
            "permission_denied_burst",
        ] {
            assert_eq!(status_of(&evals, id), AlertStatus::Firing, "{id}");
        }
        assert_eq!(status_of(&evals, "mcp_rate_limited_rate"), AlertStatus::Ok);
    }

    #[test]
    fn every_rule_is_documented_with_owner_runbook_and_rollback() {
        let docs = include_str!("../../docs/OPERATIONS.md");
        for rule in RULES {
            assert!(
                docs.contains(rule.id),
                "docs/OPERATIONS.md must mention {}",
                rule.id
            );
            assert!(
                docs.contains(&format!("#### {}", rule.runbook)),
                "docs/OPERATIONS.md needs the heading `#### {}`",
                rule.runbook
            );
            assert!(!rule.owner.is_empty() && !rule.rollback.is_empty());
        }
    }
}
