//! MCP handler for `memory_consolidate_batch`.
//!
//! Thin glue: parses the request, builds a `ConsolidationPolicy` (with
//! conservative defaults applied for any field the caller omits), runs the
//! consolidation pass, and returns the structured report as JSON.

use serde_json::{json, Value};

use super::HandlerContext;
use crate::error::EngramError;
use crate::intelligence::auto_consolidate::{
    list_history, run_consolidation, ConsolidationAction, ConsolidationPolicy,
};
use crate::storage::enrichment_events::{emit_best_effort, EnrichmentEvent};
use crate::storage::Storage;

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
        if let Some(v) = p
            .get("max_access_count_for_archival")
            .and_then(|v| v.as_i64())
        {
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
        if let Some(v) = p.get("hot_ids").and_then(|v| v.as_array()) {
            policy.hot_ids = Some(v.iter().filter_map(|id| id.as_i64()).collect());
        }
        if let Some(v) = p.get("composite_cutoff").and_then(|v| v.as_f64()) {
            policy.composite_cutoff = v;
        }
        if let Some(v) = p
            .get("max_importance_for_archival")
            .and_then(|v| v.as_f64())
        {
            policy.max_importance_for_archival = v as f32;
        }
    }
    // Top-level `dry_run` is allowed as a shortcut — it overrides whatever
    // came in `policy.dry_run` since callers reach for it most often.
    if let Some(v) = params.get("dry_run").and_then(|v| v.as_bool()) {
        policy.dry_run = v;
    }

    match run_consolidation(&ctx.storage, &workspace, &policy) {
        Ok(report) => {
            // Emit enrichment events for non-dry-run consolidation actions.
            if !report.dry_run {
                let operation_id = uuid::Uuid::new_v4().to_string();
                let dry_run_flag = report.dry_run;
                let ws_str = report.workspace.clone();
                let actions_snapshot: Vec<_> = report
                    .actions
                    .iter()
                    .filter_map(|a| match a {
                        ConsolidationAction::DuplicateMerged { kept, merged, .. } => {
                            Some((*kept, *merged))
                        }
                        ConsolidationAction::Summarized {
                            memory_ids,
                            summary_id,
                        } => {
                            // Emit for the first source id, carrying summary_id in outcome.
                            memory_ids.first().map(|&id| (id, summary_id.unwrap_or(id)))
                        }
                        _ => None,
                    })
                    .collect();

                if !actions_snapshot.is_empty() {
                    let _ = ctx.storage.with_connection(|conn| {
                        for (mem_id, _) in &actions_snapshot {
                            emit_best_effort(
                                conn,
                                &EnrichmentEvent {
                                    operation_id: &operation_id,
                                    event_type: "consolidation",
                                    memory_id: Some(*mem_id),
                                    version_id: None,
                                    triggered_by: "memory_consolidate_batch",
                                    agent_id: None,
                                    workspace: Some(ws_str.as_str()),
                                    params: json!({}),
                                    outcome: json!({}),
                                    status: "completed",
                                    dry_run: dry_run_flag,
                                },
                            );
                        }
                        Ok::<_, crate::error::EngramError>(())
                    });
                }
            }

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

// Process-wide auto-consolidator scheduler state. Both fields are atomics so the
// MCP handler can update them safely from any tokio worker thread; the actual
// scheduler loop (when wired by the binary) reads these on each tick.
static AUTO_CONSOLIDATOR_ENABLED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static AUTO_CONSOLIDATOR_INTERVAL_SECS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(3600);
static AUTO_CONSOLIDATOR_SCHEDULER_STARTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Public accessors so the server's scheduler thread can poll these settings
/// without going through the MCP handler.
pub fn auto_consolidator_enabled() -> bool {
    AUTO_CONSOLIDATOR_ENABLED.load(std::sync::atomic::Ordering::SeqCst)
}

pub fn auto_consolidator_interval_secs() -> u64 {
    AUTO_CONSOLIDATOR_INTERVAL_SECS.load(std::sync::atomic::Ordering::SeqCst)
}

/// Process-wide queue of memory IDs that the feedback loop has flagged for
/// consolidation. The scheduler thread (when wired) drains this on each tick.
static PENDING_CONSOLIDATION: std::sync::Mutex<Vec<i64>> = std::sync::Mutex::new(Vec::new());

/// Adapter implementing `AutoConsolidatorTrait` over the process-wide queue.
pub struct QueueingConsolidator;

impl crate::storage::feedback::AutoConsolidatorTrait for QueueingConsolidator {
    fn schedule_consolidation(&self, memory_ids: &[i64]) -> crate::error::Result<()> {
        let mut q = PENDING_CONSOLIDATION.lock().map_err(|_| {
            EngramError::Internal("pending consolidation queue lock poisoned".into())
        })?;
        q.extend_from_slice(memory_ids);
        Ok(())
    }
}

/// Drain (and clear) all currently queued memory IDs.
pub fn drain_pending_consolidation() -> Vec<i64> {
    PENDING_CONSOLIDATION
        .lock()
        .map(|mut q| std::mem::take(&mut *q))
        .unwrap_or_default()
}

/// Number of pending entries — exposed for status/diagnostics.
pub fn pending_consolidation_count() -> usize {
    PENDING_CONSOLIDATION.lock().map(|q| q.len()).unwrap_or(0)
}

pub fn auto_consolidator_status() -> Value {
    json!({
        "enabled": auto_consolidator_enabled(),
        "interval_seconds": auto_consolidator_interval_secs(),
        "pending_consolidation_count": pending_consolidation_count(),
        "scheduler_started": AUTO_CONSOLIDATOR_SCHEDULER_STARTED
            .load(std::sync::atomic::Ordering::SeqCst),
        "note": "Maintenance loop only; runs inside this server process."
    })
}

pub fn run_auto_consolidator_tick(
    storage: &Storage,
    workspace: &str,
) -> crate::error::Result<crate::intelligence::auto_consolidate::ConsolidationReport> {
    let hot_ids = drain_pending_consolidation();
    let policy = ConsolidationPolicy {
        dry_run: false,
        max_actions_per_run: hot_ids.len().max(50),
        hot_ids: if hot_ids.is_empty() {
            None
        } else {
            Some(hot_ids)
        },
        ..Default::default()
    };
    run_consolidation(storage, workspace, &policy)
}

fn ensure_auto_consolidator_scheduler(storage: Storage) -> bool {
    if AUTO_CONSOLIDATOR_SCHEDULER_STARTED
        .compare_exchange(
            false,
            true,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        )
        .is_err()
    {
        return false;
    }

    std::thread::spawn(move || loop {
        let interval = std::time::Duration::from_secs(auto_consolidator_interval_secs());
        std::thread::sleep(interval);

        if !auto_consolidator_enabled() {
            continue;
        }

        match run_auto_consolidator_tick(&storage, "default") {
            Ok(report) => {
                let counts = report.counts();
                if counts.duplicates_merged > 0
                    || counts.conflicts_resolved > 0
                    || counts.summarized > 0
                    || counts.skipped > 0
                {
                    tracing::info!(
                        "Auto-consolidation tick completed: duplicates={}, conflicts={}, summarized={}, skipped={}",
                        counts.duplicates_merged,
                        counts.conflicts_resolved,
                        counts.summarized,
                        counts.skipped,
                    );
                }
            }
            Err(e) => tracing::error!("Auto-consolidation scheduler error: {}", e),
        }
    });

    true
}

pub fn memory_auto_consolidate(ctx: &HandlerContext, params: Value) -> Value {
    let action = match params.get("action").and_then(|v| v.as_str()) {
        Some(a) => a,
        None => return json!({"error": "action is required"}),
    };

    match action {
        "enable" => {
            AUTO_CONSOLIDATOR_ENABLED.store(true, std::sync::atomic::Ordering::SeqCst);
            let scheduler_started_now = ensure_auto_consolidator_scheduler(ctx.storage.clone());
            json!({
                "status": "enabled",
                "scheduler_started_now": scheduler_started_now,
                "scheduler_started": AUTO_CONSOLIDATOR_SCHEDULER_STARTED
                    .load(std::sync::atomic::Ordering::SeqCst),
            })
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
            AUTO_CONSOLIDATOR_INTERVAL_SECS.store(interval, std::sync::atomic::Ordering::SeqCst);
            json!({"status": "interval_set", "interval_seconds": interval})
        }
        "get_status" => auto_consolidator_status(),
        _ => json!({"error": format!("Unknown action: {}", action)}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::feedback::AutoConsolidatorTrait;
    use serde_json::json;

    static QUEUE_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn status_payload_reports_pending_consolidation_count() {
        let _guard = QUEUE_TEST_LOCK.lock().expect("queue test lock");
        let _ = drain_pending_consolidation();
        QueueingConsolidator
            .schedule_consolidation(&[10, 11])
            .expect("queue");

        let status = auto_consolidator_status();
        assert_eq!(status["pending_consolidation_count"], json!(2));

        let _ = drain_pending_consolidation();
    }

    #[test]
    fn auto_consolidator_tick_drains_pending_queue() {
        let _guard = QUEUE_TEST_LOCK.lock().expect("queue test lock");
        let _ = drain_pending_consolidation();
        QueueingConsolidator
            .schedule_consolidation(&[42])
            .expect("queue");

        let storage = Storage::open_in_memory().expect("storage");
        let _ = run_auto_consolidator_tick(&storage, "default").expect("tick");

        assert_eq!(pending_consolidation_count(), 0);
    }

    #[test]
    fn parses_partial_policy_with_defaults() {
        // We only exercise the param-parsing logic here; full e2e is covered
        // by the integration tests in intelligence::auto_consolidate.
        let mut policy = ConsolidationPolicy::default();
        let params = json!({
            "policy": {
                "duplicate_threshold": 0.85,
                "max_actions_per_run": 5,
                "composite_cutoff": 0.4,
                "max_importance_for_archival": 0.45,
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
            if let Some(v) = p.get("composite_cutoff").and_then(|v| v.as_f64()) {
                policy.composite_cutoff = v;
            }
            if let Some(v) = p
                .get("max_importance_for_archival")
                .and_then(|v| v.as_f64())
            {
                policy.max_importance_for_archival = v as f32;
            }
        }
        if let Some(v) = params.get("dry_run").and_then(|v| v.as_bool()) {
            policy.dry_run = v;
        }
        assert_eq!(policy.duplicate_threshold, 0.85);
        assert_eq!(policy.max_actions_per_run, 5);
        assert_eq!(policy.composite_cutoff, 0.4);
        assert_eq!(policy.max_importance_for_archival, 0.45);
        assert!(!policy.dry_run);
        // Untouched fields keep their defaults.
        assert!(!policy.conflict_auto_resolve);
        assert_eq!(policy.summarize_age_days, 90);
    }
}
