//! Internal Operational Context efficiency metadata.
//!
//! These helpers intentionally store estimates and reuse counts without making
//! hard "tokens saved" or cost-reduction claims. The default tokenizer is a
//! coarse character-count heuristic so callers can safely omit exact token
//! estimates without breaking search or bundle construction.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenEstimateMetadata {
    pub tokenizer: String,
    pub heuristic: String,
    pub source: String,
}

impl TokenEstimateMetadata {
    pub fn char_heuristic(source: impl Into<String>) -> Self {
        Self {
            tokenizer: "heuristic:chars_per_token".to_string(),
            heuristic: "ceil(char_count / 4)".to_string(),
            source: source.into(),
        }
    }

    pub fn byte_heuristic(source: impl Into<String>) -> Self {
        Self {
            tokenizer: "heuristic:bytes_per_token".to_string(),
            heuristic: "ceil(byte_count / 4)".to_string(),
            source: source.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenEstimate {
    pub tokens: i64,
    pub metadata: TokenEstimateMetadata,
}

pub fn estimate_text_tokens(text: &str, source: impl Into<String>) -> Option<TokenEstimate> {
    if text.is_empty() {
        return None;
    }
    let char_count = text.chars().count() as i64;
    Some(TokenEstimate {
        tokens: ceil_div(char_count, 4),
        metadata: TokenEstimateMetadata::char_heuristic(source),
    })
}

pub fn estimate_bytes_tokens(byte_len: i64, source: impl Into<String>) -> Option<TokenEstimate> {
    if byte_len <= 0 {
        return None;
    }
    Some(TokenEstimate {
        tokens: ceil_div(byte_len, 4),
        metadata: TokenEstimateMetadata::byte_heuristic(source),
    })
}

fn ceil_div(value: i64, divisor: i64) -> i64 {
    if value <= 0 {
        0
    } else {
        (value + divisor - 1) / divisor
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextEfficiencyMetrics {
    pub tokens_raw_est: Option<i64>,
    pub tokens_compact_est: Option<i64>,
    pub estimate_sources: Vec<TokenEstimateMetadata>,
    pub summaries_reused: usize,
    pub raw_artifact_retrievals_avoided: usize,
    pub raw_artifact_retrievals: usize,
    pub repeated_failures_found: usize,
    pub repeated_command_context_reused: usize,
    pub artifacts_summarized: usize,
    pub artifacts_returned_as_pointers: usize,
}

impl ContextEfficiencyMetrics {
    pub fn add_raw_estimate(&mut self, estimate: TokenEstimate) {
        self.tokens_raw_est = Some(self.tokens_raw_est.unwrap_or(0) + estimate.tokens);
        self.record_source(estimate.metadata);
    }

    pub fn add_compact_estimate(&mut self, estimate: TokenEstimate) {
        self.tokens_compact_est = Some(self.tokens_compact_est.unwrap_or(0) + estimate.tokens);
        self.record_source(estimate.metadata);
    }

    pub fn add_known_pair(
        &mut self,
        raw_tokens: Option<i64>,
        compact_tokens: Option<i64>,
        metadata: Option<TokenEstimateMetadata>,
    ) {
        if let Some(tokens) = raw_tokens.filter(|tokens| *tokens >= 0) {
            self.tokens_raw_est = Some(self.tokens_raw_est.unwrap_or(0) + tokens);
        }
        if let Some(tokens) = compact_tokens.filter(|tokens| *tokens >= 0) {
            self.tokens_compact_est = Some(self.tokens_compact_est.unwrap_or(0) + tokens);
        }
        if let Some(metadata) = metadata {
            self.record_source(metadata);
        }
    }

    pub fn estimated_tokens_reduced(&self) -> Option<i64> {
        match (self.tokens_raw_est, self.tokens_compact_est) {
            (Some(raw), Some(compact)) if raw >= compact => Some(raw - compact),
            _ => None,
        }
    }

    pub fn to_internal_json(&self) -> Value {
        json!({
            "schema": "engram.context_efficiency.v1",
            "claim_scope": "internal estimates only; no exact tokens saved or guaranteed cost reduction",
            "allowed_language": [
                "estimated tokens reduced",
                "summaries reused",
                "raw artifact retrievals avoided",
                "repeated command context reused"
            ],
            "estimates": {
                "tokens_raw_est": self.tokens_raw_est,
                "tokens_compact_est": self.tokens_compact_est,
                "estimated_tokens_reduced": self.estimated_tokens_reduced(),
                "sources": self.estimate_sources,
            },
            "reuse": {
                "summaries_reused": self.summaries_reused,
                "raw_artifact_retrievals_avoided": self.raw_artifact_retrievals_avoided,
                "raw_artifact_retrievals": self.raw_artifact_retrievals,
                "repeated_failures_found": self.repeated_failures_found,
                "repeated_command_context_reused": self.repeated_command_context_reused,
                "artifacts_summarized": self.artifacts_summarized,
                "artifacts_returned_as_pointers": self.artifacts_returned_as_pointers,
            }
        })
    }

    fn record_source(&mut self, metadata: TokenEstimateMetadata) {
        if !self.estimate_sources.contains(&metadata) {
            self.estimate_sources.push(metadata);
        }
    }
}

pub fn attach_summary_estimate_metadata(
    structured_facts: &Value,
    raw_tokens: Option<i64>,
    compact_tokens: Option<i64>,
    metadata: Option<TokenEstimateMetadata>,
) -> Value {
    if raw_tokens.is_none() && compact_tokens.is_none() && metadata.is_none() {
        return structured_facts.clone();
    }

    let mut root = structured_facts.as_object().cloned().unwrap_or_default();
    let mut internal = root
        .remove("_internal")
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();

    internal.insert(
        "token_estimate".to_string(),
        json!({
            "tokens_raw_est": raw_tokens,
            "tokens_compact_est": compact_tokens,
            "metadata": metadata.unwrap_or_else(|| TokenEstimateMetadata {
                tokenizer: "unknown".to_string(),
                heuristic: "caller_provided_estimate_without_tokenizer_metadata".to_string(),
                source: "context_summary.tokens_*_est".to_string(),
            }),
            "claim_scope": "approximate estimate metadata only"
        }),
    );
    root.insert("_internal".to_string(), Value::Object(internal));
    Value::Object(root)
}

pub fn summary_estimate_metadata(structured_facts: &Value) -> Option<TokenEstimateMetadata> {
    let value = structured_facts
        .get("_internal")?
        .get("token_estimate")?
        .get("metadata")?
        .clone();
    serde_json::from_value(value).ok()
}
