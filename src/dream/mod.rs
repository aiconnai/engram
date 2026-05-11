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
use crate::intelligence::consolidation_offline::{
    ConsolidationConfig, ConsolidationReport, OfflineConsolidator,
};
use crate::storage::Storage;

/// Configuration for the Dream Phase runner.
#[derive(Debug, Clone)]
pub struct DreamConfig {
    /// Interval between scheduled runs. Defaults to 6 hours.
    pub interval: std::time::Duration,
    /// Consolidation tuning passed down to [`OfflineConsolidator`].
    pub consolidation: ConsolidationConfig,
}

impl Default for DreamConfig {
    fn default() -> Self {
        Self {
            interval: std::time::Duration::from_secs(6 * 60 * 60),
            consolidation: ConsolidationConfig::default(),
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
    let report = storage.with_transaction(|conn| consolidator.consolidate(conn, workspace))?;
    Ok((workspace, report).into())
}

/// Run a dream pass across every workspace known to storage.
pub fn run_once_all(storage: &Storage, config: &DreamConfig) -> DreamReport {
    let started_at = Utc::now();
    let mut workspaces = Vec::new();
    let mut errors = Vec::new();

    let workspace_list = storage.with_transaction(crate::storage::queries::list_workspaces);

    match workspace_list {
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

    DreamReport {
        started_at,
        finished_at: Utc::now(),
        workspaces,
        errors,
    }
}

/// Spawn a background task that runs [`run_once_all`] every `config.interval`.
///
/// The returned handle keeps the task alive until dropped or the program exits.
/// For multi-replica deployments, add a DB advisory lock around the inner call;
/// today only one replica should run this.
pub fn spawn_scheduler(storage: Arc<Storage>, config: DreamConfig) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(config.interval);
        // Skip the immediate first tick — wait one full interval before the
        // first run so server startup doesn't block on consolidation.
        ticker.tick().await;
        loop {
            ticker.tick().await;
            let report = tokio::task::block_in_place(|| run_once_all(&storage, &config));
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
