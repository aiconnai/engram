//! Dream Phase — periodic background consolidation of memories.
//!
//! Inspired by the "Unified Agentic Memory Across Harnesses Using Hooks"
//! pattern (Towards Data Science, May 2026): online hooks capture raw events,
//! an offline batch phase distills them, and the next session reads only
//! the distillate.
//!
//! This module is a thin orchestrator around the existing
//! [`OfflineConsolidator`](crate::intelligence::consolidation_offline) — it
//! enumerates workspaces and runs the consolidation pass on a schedule.
//!
//! ## Scope of this MVP
//!
//! - Single-instance scheduler (no multi-replica advisory locks yet).
//! - One pipeline stage (consolidation). Adding summarize/decay/digest
//!   emission is the natural follow-up.
//! - Manual trigger via the `dream_run_now` MCP tool.
//!
//! See issue #12 for the full vision.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::intelligence::auto_consolidate::{
    run_consolidation, ConsolidationCounts, ConsolidationPolicy,
};
use crate::intelligence::consolidation_offline::{
    ConsolidationConfig, ConsolidationReport, OfflineConsolidator,
};
use crate::storage::Storage;

pub mod candidates;

/// Configuration for the Dream Phase runner.
#[derive(Debug, Clone)]
pub struct DreamConfig {
    /// Interval between scheduled runs. Defaults to 6 hours.
    pub interval: std::time::Duration,
    /// Consolidation tuning passed down to [`OfflineConsolidator`].
    pub consolidation: ConsolidationConfig,
    /// Unique ID for this runner instance (used for advisory locks).
    pub owner_id: String,
    /// Advisory lock TTL in seconds. Acts as a crash-recovery bound: if the
    /// holder dies without releasing, the next acquirer can take over after
    /// this many seconds. Should be a small multiple of the expected pass
    /// duration. Defaults to 300s (5 min).
    pub lock_ttl_seconds: u64,
    /// Policy for the post-consolidation auto-consolidation pass that runs
    /// inside each workspace's dream pass. The default is conservative
    /// (`dry_run = true`) so dream-phase consumers get observability without
    /// new destructive behavior. Set `dry_run = false` to opt into auto-merge
    /// and archive-eligible decisions.
    pub auto_consolidate: ConsolidationPolicy,
}

impl Default for DreamConfig {
    fn default() -> Self {
        Self {
            interval: std::time::Duration::from_secs(6 * 60 * 60),
            consolidation: ConsolidationConfig::default(),
            owner_id: uuid::Uuid::new_v4().to_string(),
            lock_ttl_seconds: 300,
            auto_consolidate: ConsolidationPolicy::default(),
        }
    }
}

/// Identifier of the system-wide Dream Phase advisory lock.
const DREAM_LOCK_ID: &str = "dream_phase_all";

/// RAII guard that releases the dream-phase advisory lock on drop —
/// including on panic-unwind. This is the only way to guarantee the lock
/// doesn't leak when `consolidate` or any downstream code panics.
struct DreamLockGuard<'a> {
    storage: &'a Storage,
    owner_id: String,
}

impl<'a> Drop for DreamLockGuard<'a> {
    fn drop(&mut self) {
        let res = self.storage.with_transaction(|conn| {
            crate::storage::queries::release_dream_lock(conn, DREAM_LOCK_ID, &self.owner_id)
        });
        if let Err(e) = res {
            tracing::error!(
                target = "engram::dream",
                error = %e,
                owner_id = %self.owner_id,
                "Failed to release dream-phase advisory lock — lock may persist until TTL expires"
            );
        }
    }
}

/// Per-workspace outcome of a single dream pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamWorkspaceReport {
    pub workspace: String,
    pub groups_found: usize,
    pub memories_merged: usize,
    pub memories_archived: usize,
    pub tokens_before: usize,
    pub tokens_after: usize,
    pub tokens_saved: usize,
    /// Counts from the auto-consolidation phase that runs after the offline
    /// consolidator. `None` when the phase was disabled or the call failed —
    /// auto-consolidate is best-effort and never aborts the dream pass.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub auto_consolidate: Option<ConsolidationCounts>,
}

impl From<(&str, ConsolidationReport)> for DreamWorkspaceReport {
    fn from((workspace, r): (&str, ConsolidationReport)) -> Self {
        Self {
            workspace: workspace.to_string(),
            groups_found: r.groups_found,
            memories_merged: r.memories_merged,
            memories_archived: r.memories_archived,
            tokens_before: r.tokens_before,
            tokens_after: r.tokens_after,
            tokens_saved: r.tokens_saved,
            auto_consolidate: None,
        }
    }
}

/// Aggregate report across all workspaces processed in one dream pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamReport {
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub workspaces: Vec<DreamWorkspaceReport>,
    pub errors: Vec<String>,
}

/// Run a single consolidation pass for one workspace.
pub fn run_once_workspace(
    storage: &Storage,
    workspace: &str,
    config: &DreamConfig,
) -> Result<DreamWorkspaceReport> {
    let consolidator = OfflineConsolidator::new(config.consolidation.clone());
    let report = storage.with_transaction(|conn| {
        let r = consolidator.consolidate(conn, workspace)?;

        if r.groups_found > 0 {
            let digest_content = format!(
                "Dream Phase Digest: Consolidated {} memories into {} groups, saving {} tokens.",
                r.memories_merged, r.groups_found, r.tokens_saved
            );

            let digest_input = crate::types::CreateMemoryInput {
                content: digest_content,
                memory_type: crate::types::MemoryType::Summary,
                workspace: Some(workspace.to_string()),
                tags: vec!["dream-digest".to_string(), "distillate".to_string()],
                importance: Some(0.8),
                ..Default::default()
            };

            // Digest emission is best-effort: a failure here should not undo
            // the consolidation itself. Log so it's observable.
            if let Err(e) = crate::storage::queries::create_memory(conn, &digest_input) {
                tracing::warn!(
                    target = "engram::dream",
                    workspace = %workspace,
                    error = %e,
                    "Failed to emit dream-phase digest memory"
                );
            }
        }

        Ok(r)
    })?;

    let mut workspace_report: DreamWorkspaceReport = (workspace, report).into();

    // ── Auto-consolidation phase (best-effort) ──────────────────────────
    // The offline consolidator above handles content-overlap / tag / temporal
    // grouping. `run_consolidation` is a second, complementary pass that
    // surfaces near-duplicate pairs and conflicts detected by the
    // `context_quality` heuristics. A failure here must not abort the dream
    // pass — log and continue with `auto_consolidate = None`.
    match run_consolidation(storage, workspace, &config.auto_consolidate) {
        Ok(report) => {
            let counts = report.counts();
            tracing::debug!(
                target = "engram::dream",
                workspace = %workspace,
                dry_run = report.dry_run,
                duplicates_merged = counts.duplicates_merged,
                conflicts_resolved = counts.conflicts_resolved,
                summarized = counts.summarized,
                skipped = counts.skipped,
                "auto-consolidate phase finished"
            );
            workspace_report.auto_consolidate = Some(counts);
        }
        Err(e) => {
            tracing::warn!(
                target = "engram::dream",
                workspace = %workspace,
                error = %e,
                "auto-consolidate phase failed; continuing"
            );
        }
    }

    Ok(workspace_report)
}

/// Run a dream pass across every workspace known to storage.
///
/// Acquires the system-wide advisory lock first; if another instance holds
/// it, returns a `DreamReport` with an error entry and no workspaces
/// processed. The lock is released via [`DreamLockGuard`] on every exit
/// path, including panic-unwind.
pub fn run_once_all(storage: &Storage, config: &DreamConfig) -> DreamReport {
    let started_at = Utc::now();
    let mut workspaces = Vec::new();
    let mut errors = Vec::new();

    let lock_attempt = storage.with_transaction(|conn| {
        crate::storage::queries::acquire_dream_lock(
            conn,
            DREAM_LOCK_ID,
            &config.owner_id,
            config.lock_ttl_seconds,
        )
    });

    let lock_acquired = match lock_attempt {
        Ok(acquired) => acquired,
        Err(e) => {
            tracing::error!(
                target = "engram::dream",
                error = %e,
                "acquire_dream_lock failed"
            );
            return DreamReport {
                started_at,
                finished_at: Utc::now(),
                workspaces: Vec::new(),
                errors: vec![format!("acquire_dream_lock: {}", e)],
            };
        }
    };

    if !lock_acquired {
        return DreamReport {
            started_at,
            finished_at: Utc::now(),
            workspaces: Vec::new(),
            errors: vec![
                "Could not acquire advisory lock (another instance may be running)".to_string(),
            ],
        };
    }

    // From here on, the guard guarantees the lock is released even on panic.
    let _guard = DreamLockGuard {
        storage,
        owner_id: config.owner_id.clone(),
    };

    match storage.with_transaction(crate::storage::queries::list_workspaces) {
        Ok(list) => {
            for stats in list {
                match run_once_workspace(storage, &stats.workspace, config) {
                    Ok(report) => workspaces.push(report),
                    Err(e) => errors.push(format!("{}: {}", stats.workspace, e)),
                }
            }
        }
        Err(e) => errors.push(format!("list_workspaces: {}", e)),
    }

    let report = DreamReport {
        started_at,
        finished_at: Utc::now(),
        workspaces,
        errors,
    };

    if let Err(e) =
        storage.with_transaction(|conn| crate::storage::queries::insert_dream_run(conn, &report))
    {
        tracing::error!(
            target = "engram::dream",
            error = %e,
            "Failed to persist dream_runs history row"
        );
    }

    report
}

/// Spawn a background task that runs [`run_once_all`] every `config.interval`.
///
/// The returned handle keeps the task alive until dropped or the program exits.
/// The blocking DB work runs inside `tokio::task::spawn_blocking`, so this
/// works under both the multi-thread and current_thread tokio runtimes.
pub fn spawn_scheduler(storage: Arc<Storage>, config: DreamConfig) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(config.interval);
        // Skip the immediate first tick — wait one full interval before the
        // first run so server startup doesn't block on consolidation.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let storage_for_pass = Arc::clone(&storage);
            let config_for_pass = config.clone();
            let report = match tokio::task::spawn_blocking(move || {
                run_once_all(&storage_for_pass, &config_for_pass)
            })
            .await
            {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(
                        target = "engram::dream",
                        error = %e,
                        "Dream Phase pass panicked"
                    );
                    continue;
                }
            };
            tracing::info!(
                target = "engram::dream",
                workspaces = report.workspaces.len(),
                errors = report.errors.len(),
                duration_ms = (report.finished_at - report.started_at).num_milliseconds(),
                "Dream Phase pass complete"
            );
            for err in &report.errors {
                tracing::warn!(target = "engram::dream", error = %err, "Dream Phase workspace failed");
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_once_workspace_on_empty_storage() {
        let storage = Storage::open_in_memory().unwrap();
        let cfg = DreamConfig::default();
        let report = run_once_workspace(&storage, "default", &cfg).unwrap();
        assert_eq!(report.workspace, "default");
        assert_eq!(report.groups_found, 0);
        assert_eq!(report.memories_merged, 0);
    }

    #[test]
    fn test_run_once_all_on_empty_storage() {
        let storage = Storage::open_in_memory().unwrap();
        let cfg = DreamConfig::default();
        let report = run_once_all(&storage, &cfg);
        assert!(report.errors.is_empty(), "errors: {:?}", report.errors);
        assert!(report.finished_at >= report.started_at);
    }
}
