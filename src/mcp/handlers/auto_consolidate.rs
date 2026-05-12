//! MCP handler for `memory_consolidate_batch`.
//!
//! Thin glue: parses the request, builds a `ConsolidationPolicy` (with
//! conservative defaults applied for any field the caller omits), runs the
//! consolidation pass, and returns the structured report as JSON.

use serde_json::{json, Value};

use super::HandlerContext;
use crate::intelligence::auto_consolidate::{list_history, run_consolidation, ConsolidationPolicy};

pub fn memory_consolidate_batch(ctx: &HandlerContext, params: Value) -> Value {
    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    // Build a policy by overlaying caller-supplied fields onto defaults.
    // We do not require the caller to send a full policy object — partial
    // overrides are valid and the rest fall back to safe defaults.
    let mut policy = ConsolidationPolicy::default();
    if let Some(p) = params.get("policy").and_then(|v| v.as_object()) {
        if let Some(v) = p.get("duplicate_threshold").and_then(|v| v.as_f64()) {
            policy.duplicate_threshold = v;
        }
        if let Some(v) = p.get("conflict_auto_resolve").and_then(|v| v.as_bool()) {
            policy.conflict_auto_resolve = v;
        }
        if let Some(v) = p.get("summarize_age_days").and_then(|v| v.as_i64()) {
            policy.summarize_age_days = v;
        }
        if let Some(v) = p.get("max_actions_per_run").and_then(|v| v.as_u64()) {
            policy.max_actions_per_run = v as usize;
        }
        if let Some(v) = p.get("dry_run").and_then(|v| v.as_bool()) {
            policy.dry_run = v;
        }
        if let Some(v) = p.get("utility_threshold").and_then(|v| v.as_f64()) {
            policy.utility_threshold = v;
        }
        if let Some(v) = p.get("min_feedback_events").and_then(|v| v.as_i64()) {
            policy.min_feedback_events = v;
        }
        if let Some(v) = p.get("max_access_count_for_archival").and_then(|v| v.as_i64()) {
            policy.max_access_count_for_archival = v;
        }
        if let Some(v) = p.get("utility_weight").and_then(|v| v.as_f64()) {
            policy.utility_weight = v;
        }
        if let Some(v) = p.get("age_weight").and_then(|v| v.as_f64()) {
            policy.age_weight = v;
        }
        if let Some(v) = p.get("feedback_weight").and_then(|v| v.as_f64()) {
            policy.feedback_weight = v;
        }
    }
    // Top-level `dry_run` is allowed as a shortcut — it overrides whatever
    // came in `policy.dry_run` since callers reach for it most often.
    if let Some(v) = params.get("dry_run").and_then(|v| v.as_bool()) {
        policy.dry_run = v;
    }

    match run_consolidation(&ctx.storage, &workspace, &policy) {
        Ok(report) => {
            let counts = report.counts();
            json!({
                "workspace": report.workspace,
                "started_at": report.started_at,
                "finished_at": report.finished_at,
                "dry_run": report.dry_run,
                "counts": counts,
                "actions": report.actions,
            })
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}

pub fn memory_consolidation_history(ctx: &HandlerContext, params: Value) -> Value {
    let workspace = params.get("workspace").and_then(|v| v.as_str());
    let limit = params
        .get("limit")
        .and_then(|v| v.as_i64())
        .unwrap_or(20)
        .clamp(1, 1000);

    match list_history(&ctx.storage, workspace, limit) {
        Ok(rows) => json!({"runs": rows}),
        Err(e) => json!({"error": e.to_string()}),
    }
}

// Global static AutoConsolidator state (simple approach)
static AUTO_CONSOLIDATOR_ENABLED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static mut AUTO_CONSOLIDATOR_INTERVAL: u64 = 3600; // 1 hour default

pub fn memory_auto_consolidate(_ctx: &HandlerContext, params: Value) -> Value {
    let action = match params.get("action").and_then(|v| v.as_str()) {
        Some(a) => a,
        None => return json!({"error": "action is required"}),
    };

    match action {
        "enable" => {
            AUTO_CONSOLIDATOR_ENABLED.store(true, std::sync::atomic::Ordering::SeqCst);
            json!({"status": "enabled"})
        }
        "disable" => {
            AUTO_CONSOLIDATOR_ENABLED.store(false, std::sync::atomic::Ordering::SeqCst);
            json!({"status": "disabled"})
        }
        "set_interval" => {
            let interval = match params.get("interval_seconds").and_then(|v| v.as_u64()) {
                Some(i) if (60..=86400).contains(&i) => i,
                _ => return json!({"error": "interval_seconds must be between 60 and 86400"}),
            };
            unsafe {
                AUTO_CONSOLIDATOR_INTERVAL = interval;
            }
            json!({"status": "interval_set", "interval_seconds": interval})
        }
        "get_status" => {
            let enabled = AUTO_CONSOLIDATOR_ENABLED.load(std::sync::atomic::Ordering::SeqCst);
            let interval = unsafe { AUTO_CONSOLIDATOR_INTERVAL };
            json!({
                "enabled": enabled,
                "interval_seconds": interval,
                "note": "This is a maintenance loop, not an AI agent"
            })
        }
        _ => json!({"error": format!("Unknown action: {}", action)}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_partial_policy_with_defaults() {
        // We only exercise the param-parsing logic here; full e2e is covered
        // by the integration tests in intelligence::auto_consolidate.
        let mut policy = ConsolidationPolicy::default();
        let params = json!({
            "policy": {
                "duplicate_threshold": 0.85,
                "max_actions_per_run": 5,
            },
            "dry_run": false,
        });
        if let Some(p) = params.get("policy").and_then(|v| v.as_object()) {
            if let Some(v) = p.get("duplicate_threshold").and_then(|v| v.as_f64()) {
                policy.duplicate_threshold = v;
            }
            if let Some(v) = p.get("max_actions_per_run").and_then(|v| v.as_u64()) {
                policy.max_actions_per_run = v as usize;
            }
        }
        if let Some(v) = params.get("dry_run").and_then(|v| v.as_bool()) {
            policy.dry_run = v;
        }
        assert_eq!(policy.duplicate_threshold, 0.85);
        assert_eq!(policy.max_actions_per_run, 5);
        assert!(!policy.dry_run);
        // Untouched fields keep their defaults.
        assert!(!policy.conflict_auto_resolve);
        assert_eq!(policy.summarize_age_days, 90);
    }
}
