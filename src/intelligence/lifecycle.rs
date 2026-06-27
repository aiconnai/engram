use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{LifecycleState, Memory};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LifecycleConfig {
    pub stale_days_base: i64,
    pub archive_days_base: i64,
    pub hard_idle_cap_days: i64,
    pub max_importance_mult: f32,
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        Self {
            stale_days_base: 30,
            archive_days_base: 90,
            hard_idle_cap_days: 365,
            max_importance_mult: 4.0,
        }
    }
}

pub fn normalized_importance(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.5
    }
}

pub fn decide_lifecycle_state(
    memory: &Memory,
    now: DateTime<Utc>,
    cfg: &LifecycleConfig,
) -> LifecycleState {
    if memory.lifecycle_state == LifecycleState::Archived {
        return LifecycleState::Archived;
    }

    let last_access = memory.last_accessed_at.unwrap_or(memory.created_at);
    let idle_days = (now - last_access).num_days();

    if idle_days >= cfg.hard_idle_cap_days {
        return LifecycleState::Archived;
    }

    let importance = normalized_importance(memory.importance);
    let mult = 1.0 + importance * (cfg.max_importance_mult - 1.0);
    let effective_stale = (cfg.stale_days_base as f32 * mult) as i64;
    let effective_archive = (cfg.archive_days_base as f32 * mult) as i64;

    if idle_days >= effective_archive {
        return LifecycleState::Archived;
    }
    if idle_days >= effective_stale {
        return LifecycleState::Stale;
    }

    if memory.lifecycle_state == LifecycleState::Stale {
        return LifecycleState::Stale;
    }

    LifecycleState::Active
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{MemoryId, MemoryScope, MemoryTier, MemoryType, Visibility};
    use chrono::Duration;
    use std::collections::HashMap;

    const NOW_SECS: i64 = 1_700_000_000;

    fn now() -> DateTime<Utc> {
        DateTime::from_timestamp(NOW_SECS, 0).expect("fixed timestamp is valid")
    }

    fn memory_with(
        id: MemoryId,
        importance: f32,
        idle_days: i64,
        lifecycle_state: LifecycleState,
    ) -> Memory {
        let now = now();
        Memory {
            id,
            content: "lifecycle candidate".to_string(),
            memory_type: MemoryType::Note,
            tags: Vec::new(),
            metadata: HashMap::new(),
            importance,
            access_count: 0,
            created_at: now - Duration::days(idle_days + 10),
            updated_at: now,
            last_accessed_at: Some(now - Duration::days(idle_days)),
            owner_id: None,
            visibility: Visibility::Private,
            scope: MemoryScope::Global,
            workspace: "default".to_string(),
            tier: MemoryTier::Permanent,
            version: 1,
            has_embedding: false,
            expires_at: None,
            content_hash: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            procedure_success_count: 0,
            procedure_failure_count: 0,
            summary_of_id: None,
            lifecycle_state,
            media_url: None,
        }
    }

    fn active_memory(id: MemoryId, importance: f32, idle_days: i64) -> Memory {
        memory_with(id, importance, idle_days, LifecycleState::Active)
    }

    #[test]
    fn normalized_importance_clamps_finite_values_and_defaults_non_finite() {
        assert_eq!(normalized_importance(0.25), 0.25);
        assert_eq!(normalized_importance(2.0), 1.0);
        assert_eq!(normalized_importance(-1.0), 0.0);
        assert_eq!(normalized_importance(f32::NAN), 0.5);
        assert_eq!(normalized_importance(f32::INFINITY), 0.5);
    }

    #[test]
    fn lifecycle_predicate_matches_canonical_table() {
        let default_cfg = LifecycleConfig::default();
        let cap_300 = LifecycleConfig {
            hard_idle_cap_days: 300,
            ..LifecycleConfig::default()
        };
        let cases = [
            (
                "fresh default",
                active_memory(1, 0.5, 5),
                default_cfg,
                LifecycleState::Active,
            ),
            (
                "stale by idle",
                active_memory(2, 0.0, 35),
                default_cfg,
                LifecycleState::Stale,
            ),
            (
                "archive by idle",
                active_memory(3, 0.0, 95),
                default_cfg,
                LifecycleState::Archived,
            ),
            (
                "importance protects",
                active_memory(4, 1.0, 200),
                default_cfg,
                LifecycleState::Stale,
            ),
            (
                "boundary 359",
                active_memory(5, 1.0, 359),
                default_cfg,
                LifecycleState::Stale,
            ),
            (
                "boundary 360",
                active_memory(6, 1.0, 360),
                default_cfg,
                LifecycleState::Archived,
            ),
            (
                "forced cap",
                active_memory(7, 1.0, 320),
                cap_300,
                LifecycleState::Archived,
            ),
            (
                "exact stale",
                active_memory(8, 0.0, 30),
                default_cfg,
                LifecycleState::Stale,
            ),
            (
                "exact archive",
                active_memory(9, 0.0, 90),
                default_cfg,
                LifecycleState::Archived,
            ),
            (
                "exact cap",
                active_memory(10, 1.0, 300),
                cap_300,
                LifecycleState::Archived,
            ),
            (
                "one day before",
                active_memory(11, 0.0, 29),
                default_cfg,
                LifecycleState::Active,
            ),
            (
                "importance greater than one",
                active_memory(12, 2.0, 359),
                default_cfg,
                LifecycleState::Stale,
            ),
            (
                "importance below zero",
                active_memory(13, -1.0, 35),
                default_cfg,
                LifecycleState::Stale,
            ),
            (
                "importance nan",
                active_memory(14, f32::NAN, 95),
                default_cfg,
                LifecycleState::Stale,
            ),
            (
                "already archived",
                memory_with(15, 0.0, 5, LifecycleState::Archived),
                default_cfg,
                LifecycleState::Archived,
            ),
            (
                "already stale does not reactivate",
                memory_with(16, 0.0, 5, LifecycleState::Stale),
                default_cfg,
                LifecycleState::Stale,
            ),
        ];

        for (name, memory, cfg, expected) in cases {
            assert_eq!(
                decide_lifecycle_state(&memory, now(), &cfg),
                expected,
                "{name}"
            );
        }
    }

    #[test]
    fn lifecycle_predicate_falls_back_to_created_at_when_last_access_is_missing() {
        let now = now();
        let mut memory = active_memory(17, 0.0, 5);
        memory.created_at = now - Duration::days(35);
        memory.last_accessed_at = None;

        assert_eq!(
            decide_lifecycle_state(&memory, now, &LifecycleConfig::default()),
            LifecycleState::Stale
        );
    }
}
