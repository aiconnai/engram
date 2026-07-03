use std::collections::HashSet;

use serde::Deserialize;
use serde_json::{json, Map, Value};

use crate::dream::candidates::preview as dream_content_preview;
use crate::error::{EngramError, Result};

pub(crate) const AGENT_WRITEBACK_KIND: &str = "agent_writeback";
pub(crate) const AGENT_WRITEBACK_ACTION: &str = "create";
pub(crate) const AGENT_WRITEBACK_MODEL_PROFILE: &str = "agent-writeback-v1";

const DEFAULT_WORKSPACE: &str = "default";
const DEFAULT_CONFIDENCE: f64 = 0.5;
const PREVIEW_MAX_CHARS: usize = 180;
const RESERVED_METADATA_KEYS: &[&str] = &[
    "origin",
    "status",
    "generated_by_ai",
    "evidence_only_until_review",
    "review_required",
    "source_memory_ids",
    "evidence_source_count",
];

#[derive(Debug, Deserialize)]
struct AgentWritebackRequest {
    proposed_content: String,
    #[serde(default)]
    workspace: Option<String>,
    #[serde(default)]
    job_id: Option<String>,
    #[serde(default)]
    candidate_id: Option<String>,
    #[serde(default)]
    confidence: Option<f64>,
    #[serde(default)]
    reason_codes: Option<Vec<String>>,
    #[serde(default)]
    metadata: Option<Value>,
    #[serde(default)]
    source_memory_ids: Option<Vec<i64>>,
    #[serde(default)]
    evidence: Option<Vec<AgentWritebackEvidenceInput>>,
    #[serde(default)]
    dry_run: Option<bool>,
    #[serde(default)]
    confirm: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct AgentWritebackEvidenceInput {
    source_type: String,
    source_id: String,
    #[serde(default)]
    source_ref: Option<String>,
    #[serde(default)]
    evidence: Option<Value>,
}

#[derive(Debug)]
pub(crate) struct AgentWritebackPlan {
    pub(crate) workspace: String,
    pub(crate) job_id: String,
    pub(crate) candidate_id: Option<String>,
    pub(crate) proposed_content: String,
    pub(crate) content_preview: String,
    pub(crate) confidence: f64,
    pub(crate) reason_codes: Value,
    pub(crate) metadata: Value,
    pub(crate) policy_explanation: Value,
    pub(crate) sources: Vec<AgentWritebackSourcePlan>,
    pub(crate) dry_run: bool,
    pub(crate) confirm: bool,
}

#[derive(Debug)]
pub(crate) struct AgentWritebackSourcePlan {
    pub(crate) source_type: String,
    pub(crate) source_id: String,
    pub(crate) source_ref: Option<String>,
    pub(crate) evidence: Value,
}

pub(crate) fn parse_agent_writeback_plan(params: Value) -> Result<AgentWritebackPlan> {
    let request = serde_json::from_value::<AgentWritebackRequest>(params).map_err(|error| {
        EngramError::InvalidInput(format!("invalid memory_agent_writeback request: {error}"))
    })?;
    build_agent_writeback_plan(request)
}

fn build_agent_writeback_plan(request: AgentWritebackRequest) -> Result<AgentWritebackPlan> {
    let proposed_content = required_trimmed(request.proposed_content, "proposed_content")?;
    let workspace = normalize_workspace(request.workspace)?;
    let job_id = optional_trimmed(request.job_id, "job_id")?
        .unwrap_or_else(|| format!("agent_writeback:{}", uuid::Uuid::new_v4()));
    let candidate_id = optional_trimmed(request.candidate_id, "candidate_id")?;
    let confidence = validated_confidence(request.confidence)?;
    let reason_codes = normalize_reason_codes(request.reason_codes)?;
    let source_memory_ids = normalize_source_memory_ids(request.source_memory_ids)?;
    let metadata = normalize_metadata(request.metadata)?;
    let sources = collect_sources(&source_memory_ids, request.evidence)?;

    if sources.is_empty() {
        return Err(EngramError::InvalidInput(
            "memory_agent_writeback requires at least one source_memory_ids entry or evidence source"
                .to_string(),
        ));
    }

    Ok(AgentWritebackPlan {
        workspace,
        job_id,
        candidate_id,
        content_preview: dream_content_preview(&proposed_content, PREVIEW_MAX_CHARS),
        proposed_content,
        confidence,
        reason_codes: json!(reason_codes),
        metadata: agent_writeback_metadata(metadata, &source_memory_ids, sources.len()),
        policy_explanation: json!({
            "policy": "agent-generated memory remains pending until dream_candidate_review and dream_candidate_apply",
            "canonical_memory_mutated": false,
            "requires_review": true
        }),
        sources,
        dry_run: request.dry_run.unwrap_or(true),
        confirm: request.confirm.unwrap_or(false),
    })
}

fn normalize_workspace(workspace: Option<String>) -> Result<String> {
    match workspace {
        Some(workspace) => required_trimmed(workspace, "workspace"),
        None => Ok(DEFAULT_WORKSPACE.to_string()),
    }
}

fn validated_confidence(confidence: Option<f64>) -> Result<f64> {
    let confidence = confidence.unwrap_or(DEFAULT_CONFIDENCE);
    if (0.0..=1.0).contains(&confidence) {
        Ok(confidence)
    } else {
        Err(EngramError::InvalidInput(
            "confidence must be between 0.0 and 1.0".to_string(),
        ))
    }
}

fn required_trimmed(value: String, field: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(EngramError::InvalidInput(format!("{field} is required")))
    } else {
        Ok(trimmed.to_string())
    }
}

fn optional_trimmed(value: Option<String>, field: &str) -> Result<Option<String>> {
    value
        .map(|value| required_trimmed(value, field))
        .transpose()
}

fn normalize_reason_codes(reason_codes: Option<Vec<String>>) -> Result<Vec<String>> {
    let raw_codes = reason_codes.unwrap_or_else(|| vec![AGENT_WRITEBACK_KIND.to_string()]);
    let mut normalized = Vec::with_capacity(raw_codes.len());
    for code in raw_codes {
        normalized.push(required_trimmed(code, "reason_codes")?);
    }
    if normalized.is_empty() {
        Ok(vec![AGENT_WRITEBACK_KIND.to_string()])
    } else {
        Ok(normalized)
    }
}

fn normalize_source_memory_ids(source_memory_ids: Option<Vec<i64>>) -> Result<Vec<i64>> {
    let mut normalized = Vec::new();
    let mut seen = HashSet::new();
    for memory_id in source_memory_ids.unwrap_or_default() {
        if memory_id <= 0 {
            return Err(EngramError::InvalidInput(
                "source_memory_ids must contain positive memory ids".to_string(),
            ));
        }
        if !seen.insert(memory_id) {
            return Err(EngramError::InvalidInput(
                "source_memory_ids must not contain duplicate ids".to_string(),
            ));
        }
        normalized.push(memory_id);
    }
    Ok(normalized)
}

fn normalize_metadata(metadata: Option<Value>) -> Result<Map<String, Value>> {
    match metadata {
        Some(Value::Object(metadata)) => reject_reserved_metadata(metadata),
        Some(Value::Null) | None => Ok(Map::new()),
        Some(_) => Err(EngramError::InvalidInput(
            "metadata must be a JSON object".to_string(),
        )),
    }
}

fn reject_reserved_metadata(metadata: Map<String, Value>) -> Result<Map<String, Value>> {
    for key in metadata.keys() {
        let normalized = key.to_ascii_lowercase();
        if RESERVED_METADATA_KEYS.contains(&normalized.as_str()) {
            return Err(EngramError::InvalidInput(format!(
                "reserved metadata key `{key}` is managed by memory_agent_writeback"
            )));
        }
    }
    Ok(metadata)
}

fn collect_sources(
    source_memory_ids: &[i64],
    evidence: Option<Vec<AgentWritebackEvidenceInput>>,
) -> Result<Vec<AgentWritebackSourcePlan>> {
    let mut sources = Vec::new();
    let mut source_keys = HashSet::new();

    for memory_id in source_memory_ids {
        push_source(
            &mut sources,
            &mut source_keys,
            AgentWritebackSourcePlan {
                source_type: "memory".to_string(),
                source_id: memory_id.to_string(),
                source_ref: Some(format!("memory:{memory_id}")),
                evidence: json!({"role": "source_memory", "memory_id": memory_id}),
            },
        )?;
    }

    for evidence in evidence.unwrap_or_default() {
        push_source(
            &mut sources,
            &mut source_keys,
            AgentWritebackSourcePlan {
                source_type: required_trimmed(evidence.source_type, "evidence.source_type")?,
                source_id: required_trimmed(evidence.source_id, "evidence.source_id")?,
                source_ref: optional_trimmed(evidence.source_ref, "evidence.source_ref")?,
                evidence: evidence.evidence.unwrap_or_else(|| json!({})),
            },
        )?;
    }

    Ok(sources)
}

fn agent_writeback_metadata(
    mut metadata: Map<String, Value>,
    source_memory_ids: &[i64],
    evidence_source_count: usize,
) -> Value {
    metadata.insert("origin".to_string(), json!(AGENT_WRITEBACK_KIND));
    metadata.insert("status".to_string(), json!("pending_review"));
    metadata.insert("generated_by_ai".to_string(), json!(true));
    metadata.insert("evidence_only_until_review".to_string(), json!(true));
    metadata.insert("review_required".to_string(), json!(true));
    metadata.insert("source_memory_ids".to_string(), json!(source_memory_ids));
    metadata.insert(
        "evidence_source_count".to_string(),
        json!(evidence_source_count),
    );
    Value::Object(metadata)
}

fn push_source(
    sources: &mut Vec<AgentWritebackSourcePlan>,
    source_keys: &mut HashSet<(String, String)>,
    source: AgentWritebackSourcePlan,
) -> Result<()> {
    let key = (source.source_type.clone(), source.source_id.clone());
    if !source_keys.insert(key.clone()) {
        return Err(EngramError::InvalidInput(format!(
            "duplicate evidence source {}:{}",
            key.0, key.1
        )));
    }
    sources.push(source);
    Ok(())
}
