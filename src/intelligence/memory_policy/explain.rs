//! Stable explanations for deterministic memory policy scoring.

use serde::{Deserialize, Serialize};

use super::features::PolicyFeatures;
use super::scoring::PolicyScore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyExplanation {
    pub salience_score: f32,
    pub retention_score: f32,
    pub retrieval_priority: f32,
    pub policy_version: String,
    pub policy_reason: String,
    pub reason_codes: Vec<String>,
}

pub fn explain_policy_score(features: &PolicyFeatures, score: &PolicyScore) -> PolicyExplanation {
    PolicyExplanation {
        salience_score: score.salience_score,
        retention_score: score.retention_score,
        retrieval_priority: score.retrieval_priority,
        policy_version: score.policy_version.clone(),
        policy_reason: score.policy_reason.clone(),
        reason_codes: reason_codes(features),
    }
}

pub(crate) fn concise_policy_reason(features: &PolicyFeatures) -> String {
    reason_codes(features).join(",")
}

pub(crate) fn reason_codes(features: &PolicyFeatures) -> Vec<String> {
    vec![
        bucket_code("novelty", features.novelty, "high", "medium", "low"),
        bucket_code(
            "importance",
            features.explicit_importance,
            "high",
            "normal",
            "low",
        ),
        bucket_code(
            "utility",
            features.utility_signal,
            "high",
            "present",
            "none",
        ),
        if features.reinforcement_strength >= 0.35 {
            "reinforcement:present".to_string()
        } else {
            "reinforcement:none".to_string()
        },
        if features.contradiction_risk >= 0.35 {
            "contradiction:present".to_string()
        } else {
            "contradiction:none".to_string()
        },
        if features.recency >= 0.75 {
            "freshness:recent".to_string()
        } else if features.recency >= 0.35 {
            "freshness:aged".to_string()
        } else {
            "freshness:stale".to_string()
        },
        bucket_code(
            "durability",
            features.durability_signal,
            "durable",
            "mixed",
            "ephemeral",
        ),
        bucket_code(
            "source",
            features.source_confidence,
            "trusted",
            "normal",
            "weak",
        ),
    ]
}

fn bucket_code(prefix: &str, value: f32, high: &str, medium: &str, low: &str) -> String {
    let bucket = if value >= 0.75 {
        high
    } else if value >= 0.35 {
        medium
    } else {
        low
    };
    format!("{prefix}:{bucket}")
}
