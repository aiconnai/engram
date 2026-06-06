//! Deterministic `heuristic-v1` scoring formulas for memory policy.

use serde::{Deserialize, Serialize};

use super::explain::concise_policy_reason;
use super::features::PolicyFeatures;

pub const POLICY_VERSION: &str = "heuristic-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyScore {
    pub salience_score: f32,
    pub retention_score: f32,
    pub retrieval_priority: f32,
    pub policy_version: String,
    pub policy_reason: String,
}

/// Score a memory policy from already-normalized explicit features.
pub fn score_policy(features: &PolicyFeatures) -> PolicyScore {
    let salience_score = clamp01(
        0.30 * features.novelty
            + 0.20 * features.recency
            + 0.20 * features.explicit_importance
            + 0.15 * features.source_confidence
            + 0.15 * features.utility_signal
            - 0.20 * features.contradiction_risk,
    );

    let retention_score = clamp01(
        0.25 * salience_score
            + 0.25 * features.reinforcement_strength
            + 0.20 * features.durability_signal
            + 0.15 * features.source_confidence
            + 0.15 * features.graph_centrality_proxy
            - 0.25 * features.contradiction_risk
            - 0.15 * features.age_decay,
    );

    let retrieval_priority = clamp01(
        0.45 * features.hybrid_search_score
            + 0.25 * salience_score
            + 0.20 * features.session_relevance
            + 0.10 * retention_score
            - 0.20 * features.contradiction_risk,
    );

    PolicyScore {
        salience_score,
        retention_score,
        retrieval_priority,
        policy_version: POLICY_VERSION.to_string(),
        policy_reason: concise_policy_reason(features),
    }
}

pub fn blend_retrieval_priority(hybrid_score: f32, policy_priority: f32) -> f32 {
    clamp01(0.85 * hybrid_score + 0.15 * policy_priority)
}

fn clamp01(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.5
    }
}
