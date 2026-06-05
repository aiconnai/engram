//! Conservative Operational Context efficiency metrics.
//!
//! These helpers report estimates only. They are intended to make storage,
//! retrieval, and bundle construction auditable without claiming first-pass
//! shell-output compression or guaranteed savings.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const CHARS_PER_TOKEN_ESTIMATE: usize = 4;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEstimateMetadata {
    pub tokens_est: i64,
    pub bytes: usize,
    pub method: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextBundleMetrics {
    pub metric_type: String,
    pub estimated: bool,
    pub method: String,
    pub retrieved_item_count: usize,
    pub included_section_entry_count: usize,
    pub excluded_item_count: usize,
    pub artifact_pointer_count: usize,
    pub summarized_artifact_ref_count: usize,
    pub raw_artifact_ref_count: usize,
    pub raw_artifact_return_count: usize,
    pub retrieved_context_tokens_est: i64,
    pub bundle_tokens_est: i64,
    pub excluded_tokens_est: Option<i64>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactRetrievalCounters {
    pub summarized_artifact_refs: usize,
    pub raw_artifact_refs: usize,
    pub raw_artifacts_returned: usize,
}

pub fn estimate_tokens(text: &str) -> i64 {
    if text.is_empty() {
        return 0;
    }
    text.len().div_ceil(CHARS_PER_TOKEN_ESTIMATE) as i64
}

pub fn estimate_json_tokens(value: &Value) -> i64 {
    estimate_tokens(&value.to_string())
}

pub fn token_estimate_metadata(text: &str) -> TokenEstimateMetadata {
    TokenEstimateMetadata {
        tokens_est: estimate_tokens(text),
        bytes: text.len(),
        method: "chars_div_4_estimate".to_string(),
    }
}

pub fn estimated_savings_tokens(
    raw_tokens: Option<i64>,
    compact_tokens: Option<i64>,
) -> Option<i64> {
    match (raw_tokens, compact_tokens) {
        (Some(raw), Some(compact)) if raw >= compact => Some(raw - compact),
        _ => None,
    }
}

pub fn metrics_value(
    observed_input_tokens_est: Option<i64>,
    stored_artifact_tokens_est: Option<i64>,
    summary_tokens_est: Option<i64>,
) -> Value {
    json!({
        "estimated": true,
        "method": "chars_div_4_estimate_or_caller_supplied",
        "observed_input_tokens_est": observed_input_tokens_est,
        "stored_artifact_tokens_est": stored_artifact_tokens_est,
        "summary_tokens_est": summary_tokens_est,
        "estimated_savings_tokens": estimated_savings_tokens(
            observed_input_tokens_est,
            summary_tokens_est.or(stored_artifact_tokens_est),
        ),
        "claim": "cross_session_retrieval_estimate_only"
    })
}
