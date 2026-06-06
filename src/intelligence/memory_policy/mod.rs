//! Deterministic memory policy engine.

pub mod events;
pub mod explain;
pub mod features;
pub mod scoring;

pub use events::{PolicyEvent, PolicyEventKind};
pub use explain::{explain_policy_score, PolicyExplanation};
pub use features::{extract_features, PolicyFeatureInput, PolicyFeatures};
pub use scoring::{blend_retrieval_priority, score_policy, PolicyScore, POLICY_VERSION};

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use std::collections::HashMap;

    use super::*;
    use crate::storage::queries::PolicyRecord;
    use crate::types::{LifecycleState, Memory, MemoryId, MemoryTier, MemoryType};

    #[test]
    fn novelty_and_explicit_remember_raise_salience() {
        let memory = test_memory(1, "Remember the deployment decision", MemoryType::Decision);
        let neutral_score = score_policy(&PolicyFeatures::neutral());
        let event = PolicyEvent::with_strength(PolicyEventKind::RememberThis, 1.0);
        let features = extract_features(PolicyFeatureInput {
            memory: &memory,
            existing_policy: None,
            event: Some(&event),
            hybrid_search_score: Some(0.5),
            session_relevance: Some(0.5),
        });
        let score = score_policy(&features);

        assert!(features.novelty >= 0.85);
        assert!(features.explicit_importance >= 0.9);
        assert!(score.salience_score > neutral_score.salience_score);
    }

    #[test]
    fn contradictions_reduce_all_policy_scores() {
        let memory = test_memory(2, "Prior auth decision may be wrong", MemoryType::Decision);
        let base_features = extract_features(PolicyFeatureInput {
            memory: &memory,
            existing_policy: None,
            event: None,
            hybrid_search_score: Some(0.8),
            session_relevance: Some(0.8),
        });
        let base_score = score_policy(&base_features);

        let existing = test_policy_record(2, 4);
        let event = PolicyEvent::with_strength(PolicyEventKind::Contradiction, 1.0);
        let contradiction_features = extract_features(PolicyFeatureInput {
            memory: &memory,
            existing_policy: Some(&existing),
            event: Some(&event),
            hybrid_search_score: Some(0.8),
            session_relevance: Some(0.8),
        });
        let contradiction_score = score_policy(&contradiction_features);

        assert!(contradiction_features.contradiction_risk >= 0.9);
        assert!(contradiction_score.salience_score < base_score.salience_score);
        assert!(contradiction_score.retention_score < base_score.retention_score);
        assert!(contradiction_score.retrieval_priority < base_score.retrieval_priority);
    }

    #[test]
    fn blend_retrieval_priority_is_bounded() {
        assert_eq!(blend_retrieval_priority(-10.0, -10.0), 0.0);
        assert_eq!(blend_retrieval_priority(10.0, 10.0), 1.0);
        assert!((blend_retrieval_priority(0.5, 1.0) - 0.575).abs() < f32::EPSILON);
        assert_eq!(blend_retrieval_priority(f32::NAN, 0.5), 0.5);
        assert_eq!(blend_retrieval_priority(f32::INFINITY, 0.5), 0.5);
        assert_eq!(blend_retrieval_priority(0.5, f32::NEG_INFINITY), 0.5);
    }

    #[test]
    fn explanation_contains_stable_reason_codes() {
        let features = PolicyFeatures {
            novelty: 0.9,
            recency: 0.9,
            explicit_importance: 0.8,
            source_confidence: 0.7,
            utility_signal: 0.8,
            contradiction_risk: 0.0,
            reinforcement_strength: 0.6,
            durability_signal: 0.8,
            graph_centrality_proxy: 0.5,
            age_decay: 0.1,
            session_relevance: 0.5,
            hybrid_search_score: 0.5,
        };
        let score = score_policy(&features);
        let explanation = explain_policy_score(&features, &score);

        assert!(explanation
            .reason_codes
            .contains(&"novelty:high".to_string()));
        assert!(explanation
            .reason_codes
            .contains(&"reinforcement:present".to_string()));
        assert!(explanation
            .reason_codes
            .contains(&"contradiction:none".to_string()));
        assert!(explanation
            .reason_codes
            .contains(&"freshness:recent".to_string()));
        assert!(explanation.policy_reason.contains("novelty:high"));
    }

    fn test_memory(id: MemoryId, content: &str, memory_type: MemoryType) -> Memory {
        let now = Utc::now();
        Memory {
            id,
            content: content.to_string(),
            memory_type,
            tags: vec!["project".to_string()],
            metadata: HashMap::new(),
            importance: 0.5,
            access_count: 0,
            created_at: now,
            updated_at: now,
            last_accessed_at: None,
            owner_id: Some("test-owner".to_string()),
            visibility: Default::default(),
            scope: Default::default(),
            workspace: "default".to_string(),
            tier: MemoryTier::Permanent,
            version: 1,
            has_embedding: true,
            expires_at: None,
            content_hash: Some(format!("hash-{id}")),
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            procedure_success_count: 0,
            procedure_failure_count: 0,
            summary_of_id: None,
            lifecycle_state: LifecycleState::Active,
            media_url: None,
        }
    }

    fn test_policy_record(memory_id: MemoryId, contradiction_count: i64) -> PolicyRecord {
        PolicyRecord {
            memory_id,
            salience_score: 0.5,
            retention_score: 0.5,
            retrieval_priority: 0.5,
            last_reinforced_at: None,
            reinforcement_count: 0,
            contradiction_count,
            policy_version: POLICY_VERSION.to_string(),
            policy_reason: "test".to_string(),
            updated_at: Utc::now().to_rfc3339(),
        }
    }
}
