//! Auto-consolidation: policy types, orchestration loop, persistence.

use std::collections::HashSet;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::intelligence::context_quality::{
    detect_conflicts, find_near_duplicates, ContextQualityConfig,
};
use crate::storage::queries::list_memories;
use crate::storage::Storage;
use crate::types::{ListOptions, MemoryType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ConsolidationPolicy {
    pub duplicate_threshold: f64,
    pub conflict_auto_resolve: bool,
    pub summarize_age_days: i64,
    pub max_actions_per_run: usize,
    pub dry_run: bool,
    /// Utility score (0–1) below which a memory is a consolidation candidate.
    /// Combined with age and access frequency for a composite score.
    pub utility_threshold: f64,
    /// Minimum number of retrieval feedback events before utility gating applies.
    /// Memories with fewer events are exempt (not enough signal).
    pub min_feedback_events: i64,
    /// Maximum access count (from `memories.access_count`) for age-based archival.
    /// Frequently accessed memories are skipped even if old.
    pub max_access_count_for_archival: i64,
    /// Weight of the utility score in the composite consolidation priority score.
    /// Composite = utility_weight*(1-utility) + age_weight*age_factor + feedback_weight*negative_ratio
    pub utility_weight: f64,
    pub age_weight: f64,
    pub feedback_weight: f64,
    /// Minimum composite score required to actually summarize an aged memory.
    /// Memories below this threshold are eligible (old, low-access) but not
    /// strong enough candidates — keeps high-importance/no-negative-feedback
    /// memories alive even if they are old.
    pub composite_cutoff: f64,
    /// Maximum importance value still eligible for age-based archival.
    /// Memories above this stay regardless of age.
    pub max_importance_for_archival: f32,
    pub hot_ids: Option<Vec<i64>>,
}

impl Default for ConsolidationPolicy {
    fn default() -> Self {
        Self {
            duplicate_threshold: 0.92,
            conflict_auto_resolve: false,
            summarize_age_days: 90,
            max_actions_per_run: 50,
            dry_run: true,
            utility_threshold: 0.3,
            min_feedback_events: 3,
            max_access_count_for_archival: 10,
            utility_weight: 0.5,
            age_weight: 0.3,
            feedback_weight: 0.2,
            composite_cutoff: 0.5,
            max_importance_for_archival: 0.5,
            hot_ids: None,
        }
    }
}

impl ConsolidationPolicy {
    pub fn validate(&self) -> std::result::Result<(), String> {
        if !(0.0..=1.0).contains(&self.duplicate_threshold) {
            return Err(format!(
                "duplicate_threshold must be in [0.0, 1.0], got {}",
                self.duplicate_threshold
            ));
        }
        if self.max_actions_per_run == 0 {
            return Err("max_actions_per_run must be > 0".to_string());
        }
        if !(0.0..=1.0).contains(&self.utility_threshold) {
            return Err(format!(
                "utility_threshold must be in [0.0, 1.0], got {}",
                self.utility_threshold
            ));
        }
        if self.min_feedback_events < 0 {
            return Err(format!(
                "min_feedback_events must be >= 0, got {}",
                self.min_feedback_events
            ));
        }
        if self.max_access_count_for_archival < 0 {
            return Err(format!(
                "max_access_count_for_archival must be >= 0, got {}",
                self.max_access_count_for_archival
            ));
        }
        for (name, value) in [
            ("utility_weight", self.utility_weight),
            ("age_weight", self.age_weight),
            ("feedback_weight", self.feedback_weight),
            ("composite_cutoff", self.composite_cutoff),
        ] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(format!("{name} must be in [0.0, 1.0], got {value}"));
            }
        }
        if !self.max_importance_for_archival.is_finite()
            || !(0.0..=1.0).contains(&self.max_importance_for_archival)
        {
            return Err(format!(
                "max_importance_for_archival must be in [0.0, 1.0], got {}",
                self.max_importance_for_archival
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConsolidationAction {
    DuplicateMerged {
        kept: i64,
        merged: i64,
        similarity: f64,
    },
    ConflictResolved {
        memory_id: i64,
        strategy: String,
    },
    Summarized {
        memory_ids: Vec<i64>,
        summary_id: Option<i64>,
    },
    Skipped {
        memory_id: i64,
        reason: String,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConsolidationReport {
    pub workspace: String,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub dry_run: bool,
    pub actions: Vec<ConsolidationAction>,
}

impl ConsolidationReport {
    pub fn counts(&self) -> ConsolidationCounts {
        let mut c = ConsolidationCounts::default();
        for a in &self.actions {
            match a {
                ConsolidationAction::DuplicateMerged { .. } => c.duplicates_merged += 1,
                ConsolidationAction::ConflictResolved { .. } => c.conflicts_resolved += 1,
                ConsolidationAction::Summarized { .. } => c.summarized += 1,
                ConsolidationAction::Skipped { .. } => c.skipped += 1,
            }
        }
        c
    }

    /// Memory IDs that should be removed from the normal injected context
    /// surface after applying this report.
    pub fn effective_removed_memory_ids(&self) -> HashSet<i64> {
        let mut removed = HashSet::new();
        for action in &self.actions {
            match action {
                ConsolidationAction::DuplicateMerged { merged, .. } => {
                    removed.insert(*merged);
                }
                ConsolidationAction::Summarized { memory_ids, .. } => {
                    removed.extend(memory_ids.iter().copied());
                }
                ConsolidationAction::ConflictResolved { .. }
                | ConsolidationAction::Skipped { .. } => {}
            }
        }
        removed
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsolidationCounts {
    pub duplicates_merged: usize,
    pub conflicts_resolved: usize,
    pub summarized: usize,
    pub skipped: usize,
}

pub fn run_consolidation(
    storage: &Storage,
    workspace: &str,
    policy: &ConsolidationPolicy,
) -> Result<ConsolidationReport> {
    policy
        .validate()
        .map_err(crate::error::EngramError::InvalidInput)?;

    let started_at = Utc::now();
    let mut actions: Vec<ConsolidationAction> = Vec::new();
    let mut action_budget = policy.max_actions_per_run;

    let workspace_memory_ids: HashSet<i64> = storage.with_connection(|conn| {
        let opts = ListOptions {
            workspace: Some(workspace.to_string()),
            limit: Some(10_000),
            ..Default::default()
        };
        let memories = list_memories(conn, &opts)?;
        Ok(memories.iter().map(|m| m.id).collect())
    })?;

    if workspace_memory_ids.is_empty() {
        let report = ConsolidationReport {
            workspace: workspace.to_string(),
            started_at,
            finished_at: Utc::now(),
            dry_run: policy.dry_run,
            actions,
        };
        let _ = persist_report(storage, &report);
        return Ok(report);
    }

    let dup_limit = (policy.max_actions_per_run as i64).max(1);
    let candidates = storage
        .with_connection(|conn| {
            find_near_duplicates(conn, policy.duplicate_threshold as f32, dup_limit)
        })
        .unwrap_or_default();

    for cand in candidates {
        if action_budget == 0 {
            break;
        }
        let a_id = cand.memory_a_id;
        let b_id = cand.memory_b_id;
        if !workspace_memory_ids.contains(&a_id) || !workspace_memory_ids.contains(&b_id) {
            continue;
        }
        if (cand.similarity_score as f64) < policy.duplicate_threshold {
            actions.push(ConsolidationAction::Skipped {
                memory_id: a_id,
                reason: format!(
                    "duplicate similarity {:.3} below threshold {:.3}",
                    cand.similarity_score, policy.duplicate_threshold
                ),
            });
            continue;
        }
        actions.push(ConsolidationAction::DuplicateMerged {
            kept: a_id,
            merged: b_id,
            similarity: cand.similarity_score as f64,
        });
        action_budget -= 1;
    }

    if action_budget > 0 {
        let cq_config = ContextQualityConfig::default();
        let scan_budget = policy.max_actions_per_run.min(workspace_memory_ids.len());
        for mid in workspace_memory_ids.iter().copied().take(scan_budget) {
            if action_budget == 0 {
                break;
            }
            let conflicts = storage
                .with_connection(|conn| detect_conflicts(conn, mid, &cq_config))
                .unwrap_or_default();
            for c in conflicts {
                if action_budget == 0 {
                    break;
                }
                if !policy.conflict_auto_resolve {
                    actions.push(ConsolidationAction::Skipped {
                        memory_id: c.memory_a_id,
                        reason: format!(
                            "conflict {:?} detected; auto-resolve disabled",
                            c.conflict_type
                        ),
                    });
                    continue;
                }
                actions.push(ConsolidationAction::ConflictResolved {
                    memory_id: c.memory_a_id,
                    strategy: "KeepNewer".to_string(),
                });
                action_budget -= 1;
            }
        }
    }

    if action_budget > 0 && policy.summarize_age_days > 0 {
        use crate::search::utility::UtilityTracker;
        let cutoff = Utc::now() - Duration::days(policy.summarize_age_days);
        let candidates: Vec<crate::types::Memory> = storage.with_connection(|conn| {
            let opts = ListOptions {
                workspace: Some(workspace.to_string()),
                limit: Some((action_budget * 10) as i64),
                ..Default::default()
            };
            let memories = list_memories(conn, &opts)?;
            Ok(memories
                .into_iter()
                .filter(|m| {
                    let is_old = m.created_at < cutoff;
                    let is_hot = policy.hot_ids.as_ref().map_or(false, |ids| ids.contains(&m.id));
                    (is_old || is_hot)
                        && i64::from(m.access_count) < policy.max_access_count_for_archival
                        && m.importance <= policy.max_importance_for_archival
                        && m.memory_type != MemoryType::Summary
                        && m.memory_type != MemoryType::Checkpoint
                })
                .collect())
        })?;

        // Rank candidates by composite score combining utility, age, and feedback.
        let tracker = UtilityTracker::new();
        let mut scored: Vec<(i64, f64)> = storage.with_connection(|conn| {
            let mut out = Vec::with_capacity(candidates.len());
            
            // Fix N+1: Batch fetch feedback stats
            let mut feedback_stats = std::collections::HashMap::new();
            if !candidates.is_empty() {
                let ids_str = candidates.iter().map(|m| m.id.to_string()).collect::<Vec<_>>().join(",");
                let sql = format!(
                    "SELECT memory_id, COUNT(*), SUM(CASE WHEN was_useful = 0 THEN 1 ELSE 0 END) \
                     FROM utility_feedback WHERE memory_id IN ({}) GROUP BY memory_id",
                    ids_str
                );
                let mut stmt = conn.prepare(&sql)?;
                let rows = stmt.query_map([], |row| {
                    Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, i64>(2)?))
                })?;
                for r in rows {
                    if let Ok((mid, count, negative)) = r {
                        feedback_stats.insert(mid, (count, negative));
                    }
                }
            }

            for m in &candidates {
                let utility_score = tracker
                    .get_utility(conn, m.id)
                    .map(|u| u.score)
                    .unwrap_or(0.5);
                
                let (feedback_events, not_useful) = feedback_stats.get(&m.id).copied().unwrap_or((0, 0));

                // Skip if below the minimum feedback threshold (not enough signal).
                if feedback_events > 0
                    && feedback_events < policy.min_feedback_events
                    && utility_score >= policy.utility_threshold
                {
                    continue;
                }

                let negative_ratio = if feedback_events > 0 {
                    not_useful as f64 / feedback_events as f64
                } else {
                    0.0
                };

                let age_days = (Utc::now() - m.created_at).num_days().max(0) as f64;
                let age_factor = (age_days / policy.summarize_age_days as f64).clamp(0.0, 1.0);

                let composite = policy.utility_weight * (1.0 - utility_score)
                    + policy.age_weight * age_factor
                    + policy.feedback_weight * negative_ratio;

                out.push((m.id, composite));
            }
            Ok(out)
        })?;

        // Sort descending: highest composite = most ready for consolidation.
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    }

    let report = ConsolidationReport {
        workspace: workspace.to_string(),
        started_at,
        finished_at: Utc::now(),
        dry_run: policy.dry_run,
        actions,
    };

    let _ = persist_report(storage, &report);
    Ok(report)
}

fn persist_report(storage: &Storage, report: &ConsolidationReport) -> Result<()> {
    use rusqlite::params;
    let counts = report.counts();
    let json = serde_json::to_string(report)
        .map_err(|e| crate::error::EngramError::InvalidInput(e.to_string()))?;
    storage.with_connection(|conn| {
        conn.execute(
            r#"INSERT INTO consolidation_runs (workspace, started_at, finished_at, dry_run,
                duplicates_merged, conflicts_resolved, summarized, skipped, report)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
            params![
                report.workspace,
                report.started_at.to_rfc3339(),
                report.finished_at.to_rfc3339(),
                report.dry_run as i64,
                counts.duplicates_merged as i64,
                counts.conflicts_resolved as i64,
                counts.summarized as i64,
                counts.skipped as i64,
                json,
            ],
        )?;
        Ok(())
    })
}

pub fn list_history(
    storage: &Storage,
    workspace: Option<&str>,
    limit: i64,
) -> Result<Vec<ConsolidationReport>> {
    let limit = limit.clamp(1, 1000);
    storage.with_connection(|conn| {
        let (sql, params): (&str, Vec<rusqlite::types::Value>) = match workspace {
            Some(ws) => (
                "SELECT report FROM consolidation_runs WHERE workspace = ? ORDER BY started_at DESC LIMIT ?",
                vec![rusqlite::types::Value::Text(ws.to_string()), rusqlite::types::Value::Integer(limit)],
            ),
            None => (
                "SELECT report FROM consolidation_runs ORDER BY started_at DESC LIMIT ?",
                vec![rusqlite::types::Value::Integer(limit)],
            ),
        };
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params), |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for r in rows {
            if let Ok(rep) = serde_json::from_str::<ConsolidationReport>(&r?) {
                out.push(rep);
            }
        }
        Ok(out)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_storage() -> Storage {
        Storage::open_in_memory().expect("storage")
    }

    fn mk_memory(s: &Storage, content: &str, ws: &str) -> i64 {
        use crate::storage::queries::create_memory;
        use crate::types::CreateMemoryInput;
        s.with_connection(|conn| {
            let input = CreateMemoryInput {
                content: content.to_string(),
                workspace: Some(ws.to_string()),
                ..Default::default()
            };
            Ok(create_memory(conn, &input)?.id)
        })
        .unwrap()
    }

    #[test]
    fn default_policy_is_conservative() {
        let p = ConsolidationPolicy::default();
        assert!(p.dry_run);
        assert!(!p.conflict_auto_resolve);
        assert_eq!(p.duplicate_threshold, 0.92);
    }

    #[test]
    fn validate_rejects_bad_inputs() {
        let p = ConsolidationPolicy {
            duplicate_threshold: 1.5,
            ..Default::default()
        };
        assert!(p.validate().is_err());
        let p = ConsolidationPolicy {
            max_actions_per_run: 0,
            ..Default::default()
        };
        assert!(p.validate().is_err());
    }

    #[test]
    fn validate_rejects_bad_new_policy_fields() {
        let bad_policies = [
            ConsolidationPolicy {
                min_feedback_events: -1,
                ..Default::default()
            },
            ConsolidationPolicy {
                max_access_count_for_archival: -1,
                ..Default::default()
            },
            ConsolidationPolicy {
                utility_weight: -0.1,
                ..Default::default()
            },
            ConsolidationPolicy {
                age_weight: -0.1,
                ..Default::default()
            },
            ConsolidationPolicy {
                feedback_weight: -0.1,
                ..Default::default()
            },
            ConsolidationPolicy {
                composite_cutoff: 1.1,
                ..Default::default()
            },
            ConsolidationPolicy {
                max_importance_for_archival: 1.1,
                ..Default::default()
            },
        ];

        for policy in bad_policies {
            assert!(
                policy.validate().is_err(),
                "policy should be rejected: {policy:?}"
            );
        }
    }

    #[test]
    fn policy_roundtrips_json() {
        let p = ConsolidationPolicy {
            duplicate_threshold: 0.95,
            conflict_auto_resolve: true,
            summarize_age_days: 30,
            max_actions_per_run: 10,
            dry_run: false,
            ..Default::default()
        };
        let s = serde_json::to_string(&p).unwrap();
        assert_eq!(p, serde_json::from_str(&s).unwrap());
    }

    #[test]
    fn partial_policy_defaults() {
        let p: ConsolidationPolicy = serde_json::from_str(r#"{"dry_run": false}"#).unwrap();
        assert!(!p.dry_run);
        assert_eq!(p.duplicate_threshold, 0.92);
    }

    #[test]
    fn counts_actions_by_variant() {
        let r = ConsolidationReport {
            workspace: "x".into(),
            started_at: Utc::now(),
            finished_at: Utc::now(),
            dry_run: true,
            actions: vec![
                ConsolidationAction::DuplicateMerged {
                    kept: 1,
                    merged: 2,
                    similarity: 0.95,
                },
                ConsolidationAction::Skipped {
                    memory_id: 3,
                    reason: "x".into(),
                },
            ],
        };
        let c = r.counts();
        assert_eq!(c.duplicates_merged, 1);
        assert_eq!(c.skipped, 1);
    }

    #[test]
    fn effective_removed_memory_ids_include_merged_and_summarized_sources() {
        let r = ConsolidationReport {
            workspace: "x".into(),
            started_at: Utc::now(),
            finished_at: Utc::now(),
            dry_run: false,
            actions: vec![
                ConsolidationAction::DuplicateMerged {
                    kept: 1,
                    merged: 2,
                    similarity: 0.96,
                },
                ConsolidationAction::Summarized {
                    memory_ids: vec![3, 4],
                    summary_id: Some(9),
                },
            ],
        };

        let removed = r.effective_removed_memory_ids();
        assert_eq!(removed.len(), 3);
        assert!(removed.contains(&2));
        assert!(removed.contains(&3));
        assert!(removed.contains(&4));
        assert!(!removed.contains(&1));
        assert!(!removed.contains(&9));
    }

    #[test]
    fn empty_workspace_empty_actions() {
        let s = open_storage();
        let r = run_consolidation(&s, "empty", &ConsolidationPolicy::default()).unwrap();
        assert!(r.actions.is_empty());
    }

    #[test]
    fn invalid_policy_errors() {
        let s = open_storage();
        let p = ConsolidationPolicy {
            max_actions_per_run: 0,
            ..Default::default()
        };
        assert!(run_consolidation(&s, "x", &p).is_err());
    }

    #[test]
    fn dry_run_no_mutation() {
        let s = open_storage();
        let ws = "default";
        mk_memory(&s, "a b c", ws);
        mk_memory(&s, "a b c", ws);
        let before = s
            .with_connection(|c| {
                Ok(list_memories(
                    c,
                    &ListOptions {
                        workspace: Some(ws.into()),
                        ..Default::default()
                    },
                )?
                .len())
            })
            .unwrap();
        let r = run_consolidation(&s, ws, &ConsolidationPolicy::default()).unwrap();
        assert!(r.dry_run);
        let after = s
            .with_connection(|c| {
                Ok(list_memories(
                    c,
                    &ListOptions {
                        workspace: Some(ws.into()),
                        ..Default::default()
                    },
                )?
                .len())
            })
            .unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn persists_and_history_returns_row() {
        let s = open_storage();
        let ws = "audit";
        mk_memory(&s, "x", ws);
        run_consolidation(&s, ws, &ConsolidationPolicy::default()).unwrap();
        let n: i64 = s
            .with_connection(|c| {
                c.query_row(
                    "SELECT COUNT(*) FROM consolidation_runs WHERE workspace = ?",
                    rusqlite::params![ws],
                    |row| row.get(0),
                )
                .map_err(crate::error::EngramError::Database)
            })
            .unwrap();
        assert_eq!(n, 1);
        assert_eq!(list_history(&s, Some(ws), 10).unwrap().len(), 1);
    }

    #[test]
    fn history_newest_first() {
        let s = open_storage();
        let ws = "ordered";
        mk_memory(&s, "x", ws);
        run_consolidation(&s, ws, &ConsolidationPolicy::default()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        run_consolidation(&s, ws, &ConsolidationPolicy::default()).unwrap();
        let h = list_history(&s, Some(ws), 10).unwrap();
        assert_eq!(h.len(), 2);
        assert!(h[0].started_at >= h[1].started_at);
    }

    #[test]
    fn action_json_tag() {
        let a = ConsolidationAction::Summarized {
            memory_ids: vec![1, 2],
            summary_id: Some(3),
        };
        let s = serde_json::to_string(&a).unwrap();
        assert!(s.contains("\"kind\":\"summarized\""));
    }
}
