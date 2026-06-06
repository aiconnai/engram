//! Event inputs for deterministic memory policy scoring.

use serde::{Deserialize, Serialize};

/// Explicit policy event categories understood by `heuristic-v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEventKind {
    RememberThis,
    UserCorrection,
    ResolvedDecision,
    HighUtility,
    Contradiction,
    Retrieval,
    Promotion,
    Decay,
}

/// Optional event context used as an explicit scoring feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyEvent {
    pub kind: PolicyEventKind,
    pub strength: Option<f32>,
    pub reason: Option<String>,
    pub triggered_by: Option<String>,
}

impl PolicyEvent {
    pub fn new(kind: PolicyEventKind) -> Self {
        Self {
            kind,
            strength: None,
            reason: None,
            triggered_by: None,
        }
    }

    pub fn with_strength(kind: PolicyEventKind, strength: f32) -> Self {
        Self {
            kind,
            strength: Some(strength),
            reason: None,
            triggered_by: None,
        }
    }
}
