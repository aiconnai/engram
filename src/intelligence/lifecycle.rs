//! Canonical memory lifecycle predicate
//!
//! Provides the single authoritative pure predicate for deciding memory decay
//! state transitions (`Active` -> `Stale` -> `Archived`).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{LifecycleState, Memory};

/// Configuration for memory lifecycle evaluation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LifecycleConfig {
    /// Base idle days before an active memory transitions to stale (at importance 0.0)
    pub stale_days_base: i64,
    /// Base idle days before an active/stale memory transitions to archived (at importance 0.0)
    pub archive_days_base: i64,
    /// Hard upper bound on idle days regardless of importance
    pub hard_idle_cap_days: i64,
    /// Multiplier scaling effective threshold at max importance (1.0)
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

/// Normalize an importance score to the range `[0.0, 1.0]`, falling back to `0.5` if non-finite.
pub fn normalized_importance(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.5
    }
}

/// Pure canonical predicate deciding a memory's lifecycle state based on idle time and importance.
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
    let effective_arch = (cfg.archive_days_base as f32 * mult) as i64;

    if idle_days >= effective_arch {
        return LifecycleState::Archived;
    }
    if idle_days >= effective_stale {
        return LifecycleState::Stale;
    }

    LifecycleState::Active
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use std::collections::HashMap;

    fn make_test_memory(
        importance: f32,
        created_at: DateTime<Utc>,
        last_accessed_at: Option<DateTime<Utc>>,
        lifecycle_state: LifecycleState,
    ) -> Memory {
        Memory {
            id: 1,
            content: "test memory content".to_string(),
            memory_type: crate::types::MemoryType::Fact,
            tags: vec![],
            metadata: HashMap::new(),
            importance,
            access_count: 0,
            last_accessed_at,
            created_at,
            updated_at: created_at,
            owner_id: None,
            visibility: crate::types::Visibility::Private,
            scope: crate::types::MemoryScope::Session {
                session_id: "test".to_string(),
            },
            tier: crate::types::MemoryTier::Permanent,
            workspace: "default".to_string(),
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

    #[test]
    fn test_fresh_default() {
        let now = Utc::now();
        let mem = make_test_memory(
            0.5,
            now - Duration::days(5),
            Some(now - Duration::days(5)),
            LifecycleState::Active,
        );
        let state = decide_lifecycle_state(&mem, now, &LifecycleConfig::default());
        assert_eq!(state, LifecycleState::Active);
    }

    #[test]
    fn test_stale_by_idle() {
        let now = Utc::now();
        let mem = make_test_memory(
            0.0,
            now - Duration::days(35),
            Some(now - Duration::days(35)),
            LifecycleState::Active,
        );
        let state = decide_lifecycle_state(&mem, now, &LifecycleConfig::default());
        assert_eq!(state, LifecycleState::Stale);
    }

    #[test]
    fn test_archive_by_idle() {
        let now = Utc::now();
        let mem = make_test_memory(
            0.0,
            now - Duration::days(95),
            Some(now - Duration::days(95)),
            LifecycleState::Active,
        );
        let state = decide_lifecycle_state(&mem, now, &LifecycleConfig::default());
        assert_eq!(state, LifecycleState::Archived);
    }

    #[test]
    fn test_importance_protects() {
        let now = Utc::now();
        let mem = make_test_memory(
            1.0,
            now - Duration::days(200),
            Some(now - Duration::days(200)),
            LifecycleState::Active,
        );
        let state = decide_lifecycle_state(&mem, now, &LifecycleConfig::default());
        // At importance 1.0, mult = 4.0: stale at 120d, archive at 360d. Idle 200d -> Stale.
        assert_eq!(state, LifecycleState::Stale);
    }

    #[test]
    fn test_boundary_359() {
        let now = Utc::now();
        let mem = make_test_memory(
            1.0,
            now - Duration::days(359),
            Some(now - Duration::days(359)),
            LifecycleState::Active,
        );
        let state = decide_lifecycle_state(&mem, now, &LifecycleConfig::default());
        assert_eq!(state, LifecycleState::Stale);
    }

    #[test]
    fn test_boundary_360() {
        let now = Utc::now();
        let mem = make_test_memory(
            1.0,
            now - Duration::days(360),
            Some(now - Duration::days(360)),
            LifecycleState::Active,
        );
        let state = decide_lifecycle_state(&mem, now, &LifecycleConfig::default());
        assert_eq!(state, LifecycleState::Archived);
    }

    #[test]
    fn test_forced_cap() {
        let now = Utc::now();
        let mem = make_test_memory(
            1.0,
            now - Duration::days(320),
            Some(now - Duration::days(320)),
            LifecycleState::Active,
        );
        let cfg = LifecycleConfig {
            hard_idle_cap_days: 300,
            ..Default::default()
        };
        let state = decide_lifecycle_state(&mem, now, &cfg);
        assert_eq!(state, LifecycleState::Archived);
    }

    #[test]
    fn test_exact_stale() {
        let now = Utc::now();
        let mem = make_test_memory(
            0.0,
            now - Duration::days(30),
            Some(now - Duration::days(30)),
            LifecycleState::Active,
        );
        let state = decide_lifecycle_state(&mem, now, &LifecycleConfig::default());
        assert_eq!(state, LifecycleState::Stale);
    }

    #[test]
    fn test_exact_archive() {
        let now = Utc::now();
        let mem = make_test_memory(
            0.0,
            now - Duration::days(90),
            Some(now - Duration::days(90)),
            LifecycleState::Active,
        );
        let state = decide_lifecycle_state(&mem, now, &LifecycleConfig::default());
        assert_eq!(state, LifecycleState::Archived);
    }

    #[test]
    fn test_exact_cap() {
        let now = Utc::now();
        let mem = make_test_memory(
            1.0,
            now - Duration::days(300),
            Some(now - Duration::days(300)),
            LifecycleState::Active,
        );
        let cfg = LifecycleConfig {
            hard_idle_cap_days: 300,
            ..Default::default()
        };
        let state = decide_lifecycle_state(&mem, now, &cfg);
        assert_eq!(state, LifecycleState::Archived);
    }

    #[test]
    fn test_one_day_before() {
        let now = Utc::now();
        let mem = make_test_memory(
            0.0,
            now - Duration::days(29),
            Some(now - Duration::days(29)),
            LifecycleState::Active,
        );
        let state = decide_lifecycle_state(&mem, now, &LifecycleConfig::default());
        assert_eq!(state, LifecycleState::Active);
    }

    #[test]
    fn test_importance_greater_than_one() {
        let now = Utc::now();
        let mem = make_test_memory(
            2.0,
            now - Duration::days(359),
            Some(now - Duration::days(359)),
            LifecycleState::Active,
        );
        let state = decide_lifecycle_state(&mem, now, &LifecycleConfig::default());
        assert_eq!(state, LifecycleState::Stale);
    }

    #[test]
    fn test_importance_less_than_zero() {
        let now = Utc::now();
        let mem = make_test_memory(
            -1.0,
            now - Duration::days(35),
            Some(now - Duration::days(35)),
            LifecycleState::Active,
        );
        let state = decide_lifecycle_state(&mem, now, &LifecycleConfig::default());
        assert_eq!(state, LifecycleState::Stale);
    }

    #[test]
    fn test_importance_nan() {
        let now = Utc::now();
        let mem = make_test_memory(
            f32::NAN,
            now - Duration::days(95),
            Some(now - Duration::days(95)),
            LifecycleState::Active,
        );
        // importance NaN -> normalized to 0.5. mult = 1.0 + 0.5 * 3.0 = 2.5.
        // effective_stale = 30 * 2.5 = 75d; effective_arch = 90 * 2.5 = 225d.
        // idle 95d -> Stale.
        let state = decide_lifecycle_state(&mem, now, &LifecycleConfig::default());
        assert_eq!(state, LifecycleState::Stale);
    }

    #[test]
    fn test_already_archived() {
        let now = Utc::now();
        let mem = make_test_memory(
            1.0,
            now - Duration::days(1),
            Some(now - Duration::days(1)),
            LifecycleState::Archived,
        );
        let state = decide_lifecycle_state(&mem, now, &LifecycleConfig::default());
        assert_eq!(state, LifecycleState::Archived);
    }

    #[test]
    fn test_missing_last_accessed_at() {
        let now = Utc::now();
        let mem = make_test_memory(0.0, now - Duration::days(35), None, LifecycleState::Active);
        let state = decide_lifecycle_state(&mem, now, &LifecycleConfig::default());
        assert_eq!(state, LifecycleState::Stale);
    }
}
