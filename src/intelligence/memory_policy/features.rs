//! Explicit feature extraction for deterministic memory policy scoring.

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::storage::queries::PolicyRecord;
use crate::types::{LifecycleState, Memory, MemoryTier, MemoryType};

use super::events::{PolicyEvent, PolicyEventKind};

/// Inputs available to the deterministic policy feature extractor.
pub struct PolicyFeatureInput<'a> {
    pub memory: &'a Memory,
    pub existing_policy: Option<&'a crate::storage::queries::PolicyRecord>,
    pub event: Option<&'a crate::intelligence::memory_policy::PolicyEvent>,
    pub hybrid_search_score: Option<f32>,
    pub session_relevance: Option<f32>,
}

/// Normalized explicit features consumed by `heuristic-v1`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyFeatures {
    pub novelty: f32,
    pub recency: f32,
    pub explicit_importance: f32,
    pub source_confidence: f32,
    pub utility_signal: f32,
    pub contradiction_risk: f32,
    pub reinforcement_strength: f32,
    pub durability_signal: f32,
    pub graph_centrality_proxy: f32,
    pub age_decay: f32,
    pub session_relevance: f32,
    pub hybrid_search_score: f32,
}

impl PolicyFeatures {
    pub fn neutral() -> Self {
        Self {
            novelty: 0.5,
            recency: 0.5,
            explicit_importance: 0.5,
            source_confidence: 0.5,
            utility_signal: 0.5,
            contradiction_risk: 0.0,
            reinforcement_strength: 0.5,
            durability_signal: 0.5,
            graph_centrality_proxy: 0.5,
            age_decay: 0.5,
            session_relevance: 0.5,
            hybrid_search_score: 0.5,
        }
    }
}

/// Extract normalized policy features from explicit memory state and optional signals.
pub fn extract_features(input: PolicyFeatureInput<'_>) -> PolicyFeatures {
    let memory = input.memory;
    let existing = input.existing_policy;
    let event = input.event;

    let recency = recency_from_memory(memory);
    let age_decay = clamp01(1.0 - recency);
    let explicit_importance = explicit_importance(memory, existing, event);
    let contradiction_risk = contradiction_risk(memory, existing, event);
    let reinforcement_strength = reinforcement_strength(memory, existing, event);

    PolicyFeatures {
        novelty: novelty(memory, existing, event),
        recency,
        explicit_importance,
        source_confidence: source_confidence(memory),
        utility_signal: utility_signal(memory, existing, event),
        contradiction_risk,
        reinforcement_strength,
        durability_signal: durability_signal(memory, existing),
        graph_centrality_proxy: graph_centrality_proxy(memory, existing),
        age_decay,
        session_relevance: optional_score(input.session_relevance, 0.5),
        hybrid_search_score: optional_score(input.hybrid_search_score, 0.5),
    }
    .normalized()
}

impl PolicyFeatures {
    fn normalized(mut self) -> Self {
        self.novelty = clamp01(self.novelty);
        self.recency = clamp01(self.recency);
        self.explicit_importance = clamp01(self.explicit_importance);
        self.source_confidence = clamp01(self.source_confidence);
        self.utility_signal = clamp01(self.utility_signal);
        self.contradiction_risk = clamp01(self.contradiction_risk);
        self.reinforcement_strength = clamp01(self.reinforcement_strength);
        self.durability_signal = clamp01(self.durability_signal);
        self.graph_centrality_proxy = clamp01(self.graph_centrality_proxy);
        self.age_decay = clamp01(self.age_decay);
        self.session_relevance = clamp01(self.session_relevance);
        self.hybrid_search_score = clamp01(self.hybrid_search_score);
        self
    }
}

fn novelty(memory: &Memory, existing: Option<&PolicyRecord>, event: Option<&PolicyEvent>) -> f32 {
    let mut score: f32 = if existing.is_some() { 0.4 } else { 0.65 };

    if memory.content.len() >= 240 {
        score += 0.08;
    }
    if matches!(
        memory.memory_type,
        MemoryType::Decision | MemoryType::Learning | MemoryType::Preference | MemoryType::Fact
    ) {
        score += 0.08;
    }
    if existing.is_none() && memory.content_hash.is_some() {
        score += 0.05;
    }
    if has_truthy_metadata(
        memory,
        &["remember_this", "user_correction", "resolved_decision"],
    ) {
        score += 0.2;
    }
    if let Some(event) = event {
        match event.kind {
            PolicyEventKind::RememberThis
            | PolicyEventKind::UserCorrection
            | PolicyEventKind::ResolvedDecision => score = score.max(0.85 * event_strength(event)),
            PolicyEventKind::HighUtility => score = score.max(0.75 * event_strength(event)),
            PolicyEventKind::Contradiction => score = score.max(0.7 * event_strength(event)),
            PolicyEventKind::Promotion => score = score.max(0.65 * event_strength(event)),
            PolicyEventKind::Retrieval | PolicyEventKind::Decay => {}
        }
    }

    score
}

fn recency_from_memory(memory: &Memory) -> f32 {
    let reference = memory.last_accessed_at.unwrap_or(memory.updated_at);
    let age_days = Utc::now()
        .signed_duration_since(reference)
        .num_seconds()
        .max(0) as f32
        / 86_400.0;

    if age_days <= 1.0 {
        1.0
    } else if age_days <= 7.0 {
        0.85
    } else if age_days <= 30.0 {
        0.65
    } else if age_days <= 90.0 {
        0.45
    } else if age_days <= 365.0 {
        0.25
    } else {
        0.1
    }
}

fn explicit_importance(
    memory: &Memory,
    existing: Option<&PolicyRecord>,
    event: Option<&PolicyEvent>,
) -> f32 {
    let mut score = optional_score(Some(memory.importance), 0.5);

    if has_truthy_metadata(memory, &["important", "remember_this"]) {
        score = score.max(0.85);
    }
    if matches!(
        memory.memory_type,
        MemoryType::Decision | MemoryType::Preference
    ) {
        score = score.max(0.7);
    }
    if let Some(existing) = existing {
        score = score.max(clamp01(existing.salience_score) * 0.75);
    }
    if let Some(event) = event {
        match event.kind {
            PolicyEventKind::RememberThis
            | PolicyEventKind::UserCorrection
            | PolicyEventKind::ResolvedDecision => score = score.max(0.9 * event_strength(event)),
            PolicyEventKind::HighUtility | PolicyEventKind::Promotion => {
                score = score.max(0.75 * event_strength(event));
            }
            PolicyEventKind::Contradiction
            | PolicyEventKind::Retrieval
            | PolicyEventKind::Decay => {}
        }
    }

    score
}

fn source_confidence(memory: &Memory) -> f32 {
    if let Some(score) = metadata_score(memory, &["source_confidence", "confidence", "trust_score"])
    {
        return score;
    }

    let mut score: f32 = 0.6;
    if memory.content_hash.is_some() {
        score += 0.05;
    }
    if memory.owner_id.is_some() {
        score += 0.05;
    }
    if matches!(
        memory.memory_type,
        MemoryType::TranscriptChunk | MemoryType::Checkpoint
    ) {
        score -= 0.1;
    }
    if matches!(memory.memory_type, MemoryType::Credential) {
        score -= 0.15;
    }

    score
}

fn utility_signal(
    memory: &Memory,
    existing: Option<&PolicyRecord>,
    event: Option<&PolicyEvent>,
) -> f32 {
    let mut score = (memory.access_count.max(0) as f32 / 10.0).min(0.7);

    if matches!(
        memory.memory_type,
        MemoryType::Decision | MemoryType::Learning | MemoryType::Procedural | MemoryType::Fact
    ) {
        score = score.max(0.55);
    }
    let attempts = memory.procedure_success_count + memory.procedure_failure_count;
    if attempts > 0 {
        let success_rate = memory.procedure_success_count.max(0) as f32 / attempts.max(1) as f32;
        score = score.max(success_rate);
    }
    if let Some(existing) = existing {
        score = score.max(clamp01(existing.retrieval_priority) * 0.8);
    }
    if let Some(event) = event {
        match event.kind {
            PolicyEventKind::HighUtility => score = score.max(0.95 * event_strength(event)),
            PolicyEventKind::Retrieval => score = score.max(0.7 * event_strength(event)),
            PolicyEventKind::Promotion => score = score.max(0.75 * event_strength(event)),
            PolicyEventKind::RememberThis
            | PolicyEventKind::UserCorrection
            | PolicyEventKind::ResolvedDecision => score = score.max(0.65 * event_strength(event)),
            PolicyEventKind::Contradiction | PolicyEventKind::Decay => {}
        }
    }

    score
}

fn contradiction_risk(
    memory: &Memory,
    existing: Option<&PolicyRecord>,
    event: Option<&PolicyEvent>,
) -> f32 {
    let mut score: f32 = 0.0;

    if let Some(existing) = existing {
        let count = existing.contradiction_count.max(0) as f32;
        score = score.max(count / (count + 2.0));
    }
    if has_truthy_metadata(memory, &["contradiction", "conflict", "stale_conflict"]) {
        score = score.max(0.7);
    }
    if memory
        .tags
        .iter()
        .any(|tag| matches!(tag.as_str(), "contradiction" | "conflict" | "stale"))
    {
        score = score.max(0.55);
    }
    if matches!(
        event.map(|event| event.kind),
        Some(PolicyEventKind::Contradiction)
    ) {
        score = score.max(0.9 * event.map(event_strength).unwrap_or(1.0));
    }

    score
}

fn reinforcement_strength(
    memory: &Memory,
    existing: Option<&PolicyRecord>,
    event: Option<&PolicyEvent>,
) -> f32 {
    let mut score = (memory.access_count.max(0) as f32 / 12.0).min(0.4);

    if let Some(existing) = existing {
        let count = existing.reinforcement_count.max(0) as f32;
        score = score.max(count / (count + 4.0));
        if existing.last_reinforced_at.is_some() {
            score = score.max(0.35);
        }
    }
    if let Some(event) = event {
        match event.kind {
            PolicyEventKind::Retrieval => score = score.max(0.55 * event_strength(event)),
            PolicyEventKind::Promotion | PolicyEventKind::HighUtility => {
                score = score.max(0.75 * event_strength(event));
            }
            PolicyEventKind::RememberThis
            | PolicyEventKind::UserCorrection
            | PolicyEventKind::ResolvedDecision => score = score.max(0.65 * event_strength(event)),
            PolicyEventKind::Contradiction | PolicyEventKind::Decay => {}
        }
    }

    score
}

fn durability_signal(memory: &Memory, existing: Option<&PolicyRecord>) -> f32 {
    let mut score: f32 = match memory.tier {
        MemoryTier::Permanent => 0.75,
        MemoryTier::Daily => 0.25,
    };

    if matches!(
        memory.memory_type,
        MemoryType::Decision
            | MemoryType::Preference
            | MemoryType::Learning
            | MemoryType::Procedural
    ) {
        score = score.max(0.8);
    }
    if matches!(memory.memory_type, MemoryType::Summary | MemoryType::Fact) {
        score = score.max(0.65);
    }
    if memory.expires_at.is_some() {
        score = score.min(0.45);
    }
    match memory.lifecycle_state {
        LifecycleState::Active => {}
        LifecycleState::Stale => score = score.min(0.55),
        LifecycleState::Archived => score = score.min(0.35),
    }
    if let Some(existing) = existing {
        score = score.max(clamp01(existing.retention_score) * 0.75);
    }

    score
}

fn graph_centrality_proxy(memory: &Memory, existing: Option<&PolicyRecord>) -> f32 {
    let tag_signal = (memory.tags.len() as f32 / 6.0).min(0.5);
    let access_signal = (memory.access_count.max(0) as f32 / 20.0).min(0.3);
    let summary_signal = if memory.summary_of_id.is_some() {
        0.2
    } else {
        0.0
    };
    let prior_signal = existing
        .map(|policy| clamp01(policy.retrieval_priority) * 0.2)
        .unwrap_or(0.0);

    0.25 + tag_signal + access_signal + summary_signal + prior_signal
}

fn event_strength(event: &PolicyEvent) -> f32 {
    optional_score(event.strength, 1.0)
}

fn optional_score(score: Option<f32>, fallback: f32) -> f32 {
    match score {
        Some(value) if value.is_finite() => clamp01(value),
        _ => clamp01(fallback),
    }
}

fn metadata_score(memory: &Memory, keys: &[&str]) -> Option<f32> {
    keys.iter().find_map(|key| {
        let value = memory.metadata.get(*key)?;
        if let Some(number) = value.as_f64() {
            Some(clamp01(number as f32))
        } else {
            value
                .as_bool()
                .map(|bool_value| if bool_value { 1.0 } else { 0.0 })
        }
    })
}

fn has_truthy_metadata(memory: &Memory, keys: &[&str]) -> bool {
    keys.iter().any(|key| {
        memory
            .metadata
            .get(*key)
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
    })
}

fn clamp01(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.5
    }
}
