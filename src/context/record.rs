//! Operational Context write API.

use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::str::FromStr;

use crate::context::artifact::{
    ArtifactAccessPolicy, ArtifactRedactionStatus, ArtifactRetentionPolicy, NewContextArtifact,
};
use crate::context::metrics::{
    estimate_json_tokens, estimate_tokens, estimated_savings_tokens, metrics_value,
};
use crate::context::policy::{
    redact_field, redact_optional_field, redact_string_list, OperationalContextPolicy,
    RedactedText, RedactionReport,
};
use crate::error::{EngramError, Result};
use crate::storage::{
    create_context_artifact, create_context_event, create_context_summary, get_context_event,
    get_context_summary, ContextEvent, NewContextEvent, NewContextSummary,
};

#[derive(Debug, Clone, Deserialize)]
pub struct ContextRecordRequest {
    pub source: String,
    #[serde(default)]
    pub source_version: Option<String>,
    #[serde(default)]
    pub repo_id: Option<String>,
    #[serde(default)]
    pub workspace_path_hash: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub git_branch: Option<String>,
    #[serde(default)]
    pub worktree_name: Option<String>,
    #[serde(default)]
    pub commit_hash: Option<String>,
    pub session_id: String,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    pub event_type: String,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub command_name: Option<String>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub exit_code: Option<i64>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub key_errors: Vec<String>,
    #[serde(default)]
    pub touched_files: Vec<String>,
    #[serde(default)]
    pub reducer: Option<ContextReducerInput>,
    #[serde(default)]
    pub external_reducer: Option<String>,
    #[serde(default)]
    pub raw_pointer: Option<String>,
    #[serde(default)]
    pub external_unverified: Option<bool>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub retention_policy: Option<String>,
    #[serde(default)]
    pub raw_artifact_id: Option<String>,
    #[serde(default)]
    pub metadata: Option<Value>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContextReducerInput {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub external_reducer: Option<String>,
    #[serde(default)]
    pub raw_pointer: Option<String>,
    #[serde(default)]
    pub lossy: Option<bool>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub structured_facts: Option<Value>,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub tokens_raw_est: Option<i64>,
    #[serde(default)]
    pub tokens_compact_est: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextRecordCreatedIds {
    pub event_id: i64,
    pub summary_id: Option<i64>,
    pub raw_artifact_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProvenanceMetadata {
    pub source: String,
    pub source_version: Option<String>,
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub agent_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextRecordMetrics {
    pub estimated: bool,
    pub method: String,
    pub observed_input_tokens_est: Option<i64>,
    pub summary_tokens_est: Option<i64>,
    pub stored_artifact_tokens_est: Option<i64>,
    pub estimated_savings_tokens: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextRecordResponse {
    pub created_ids: ContextRecordCreatedIds,
    pub redaction_status: String,
    pub retention_policy: String,
    pub provenance: ProvenanceMetadata,
    pub metrics: ContextRecordMetrics,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContextRecordArtifactRequest {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub source_event_id: Option<i64>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub source_version: Option<String>,
    #[serde(default)]
    pub repo_id: Option<String>,
    #[serde(default)]
    pub workspace_path_hash: Option<String>,
    #[serde(default)]
    pub workspace: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub task_id: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    pub kind: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub raw_pointer: Option<String>,
    #[serde(default)]
    pub media_type: Option<String>,
    #[serde(default)]
    pub raw_content: Option<String>,
    #[serde(default)]
    pub content_sha256: Option<String>,
    #[serde(default)]
    pub byte_len: Option<i64>,
    #[serde(default)]
    pub retention_policy: Option<String>,
    #[serde(default)]
    pub access_policy: Option<String>,
    #[serde(default)]
    pub retain_raw: Option<bool>,
    #[serde(default)]
    pub ttl_seconds: Option<i64>,
    #[serde(default)]
    pub stale_after_seconds: Option<i64>,
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextRecordArtifactResponse {
    pub artifact_id: String,
    pub storage_kind: String,
    pub redaction_status: String,
    pub retention_policy: String,
    pub provenance: ProvenanceMetadata,
    pub metrics: ContextRecordMetrics,
}

pub fn record_context(
    conn: &Connection,
    policy: &OperationalContextPolicy,
    request: ContextRecordRequest,
) -> Result<ContextRecordResponse> {
    let mut report = RedactionReport::new();
    let source_raw = require_non_empty(request.source, "source")?;
    let source = redact_field_result(policy, &mut report, "source", &source_raw)?;
    let source_version = redact_optional_result(
        policy,
        &mut report,
        "source_version",
        clean_optional(request.source_version),
    )?;
    let session_id_raw = require_non_empty(request.session_id, "session_id")?;
    let session_id = redact_field_result(policy, &mut report, "session_id", &session_id_raw)?;
    let event_type_raw = require_non_empty(request.event_type, "event_type")?;
    let event_type = redact_field_result(policy, &mut report, "event_type", &event_type_raw)?;

    let repo_id = redact_optional_result(
        policy,
        &mut report,
        "repo_id",
        clean_optional(request.repo_id),
    )?;
    let workspace_path_hash = redact_optional_result(
        policy,
        &mut report,
        "workspace_path_hash",
        clean_optional(request.workspace_path_hash).or_else(|| clean_optional(request.workspace)),
    )?;
    if repo_id.is_none() && workspace_path_hash.is_none() {
        return Err(EngramError::InvalidInput(
            "context_record requires repo_id or workspace_path_hash/workspace".to_string(),
        ));
    }

    let command_for_analysis = clean_optional(request.command_name.clone())
        .or_else(|| clean_optional(request.command.clone()));
    let sensitive = policy.analyze_command(command_for_analysis.as_deref());
    let command_name = redact_optional_result(
        policy,
        &mut report,
        "command_name",
        clean_optional(request.command_name).or_else(|| clean_optional(request.command)),
    )?;
    let tool_name = redact_optional_result(
        policy,
        &mut report,
        "tool_name",
        clean_optional(request.tool_name).or_else(|| clean_optional(request.tool)),
    )?;

    if event_type.eq_ignore_ascii_case("command") && command_name.is_none() {
        return Err(EngramError::InvalidInput(
            "context_record command events require command or command_name".to_string(),
        ));
    }
    if event_type.eq_ignore_ascii_case("tool") && tool_name.is_none() {
        return Err(EngramError::InvalidInput(
            "context_record tool events require tool or tool_name".to_string(),
        ));
    }

    let exit_code = optional_i32(request.exit_code, "exit_code")?;
    let summary = redact_optional_result(
        policy,
        &mut report,
        "summary",
        clean_optional(request.summary),
    )?;
    let key_errors =
        redact_string_list_result(policy, &mut report, "key_errors", &request.key_errors)?;
    let touched_files =
        redact_string_list_result(policy, &mut report, "touched_files", &request.touched_files)?;
    let raw_artifact_id = redact_optional_result(
        policy,
        &mut report,
        "raw_artifact_id",
        clean_optional(request.raw_artifact_id),
    )?;
    let raw_pointer = redact_optional_result(
        policy,
        &mut report,
        "raw_pointer",
        clean_optional(request.raw_pointer.clone()).or_else(|| {
            request
                .reducer
                .as_ref()
                .and_then(|reducer| clean_optional(reducer.raw_pointer.clone()))
        }),
    )?;

    let started_at = parse_datetime_or_now(request.started_at, "started_at")?;
    let finished_at = parse_optional_datetime(request.finished_at, "finished_at")?;
    let mut retention_policy =
        clean_optional(request.retention_policy).unwrap_or_else(|| "default".to_string());
    if policy.force_ephemeral(&sensitive) {
        retention_policy = "ephemeral_sensitive".to_string();
    }

    let mut metadata = metadata_map(request.metadata);
    let metadata_value =
        redact_json_value(policy, &mut report, "metadata", Value::Object(metadata))?;
    metadata = object_map(metadata_value);
    insert_opt(&mut metadata, "source_version", source_version.clone());
    insert_opt(&mut metadata, "raw_pointer", raw_pointer.clone());
    if !key_errors.is_empty() {
        metadata.insert("key_errors".to_string(), json!(key_errors));
    }
    if !touched_files.is_empty() {
        metadata.insert("touched_files".to_string(), json!(touched_files));
    }

    let reducer = request.reducer.unwrap_or_default();
    let external_reducer = redact_optional_result(
        policy,
        &mut report,
        "external_reducer",
        clean_optional(request.external_reducer)
            .or_else(|| clean_optional(reducer.external_reducer.clone())),
    )?;
    let is_external = source.eq_ignore_ascii_case("rtk")
        || external_reducer.is_some()
        || raw_pointer.is_some()
        || request.external_unverified.unwrap_or(false);
    let external_unverified = request.external_unverified.unwrap_or(is_external);
    let mut labels = normalized_labels(request.labels.into_iter().chain(reducer.labels.clone()));
    if is_external {
        push_label(&mut labels, "derived");
        push_label(&mut labels, "lossy");
        if external_unverified {
            push_label(&mut labels, "external_unverified");
        }
    }
    if !labels.is_empty() {
        metadata.insert("labels".to_string(), json!(labels.clone()));
    }
    if is_external {
        metadata.insert(
            "external_summary".to_string(),
            json!({
                "source": source,
                "source_version": source_version,
                "external_reducer": external_reducer,
                "raw_pointer": raw_pointer,
                "labels": labels,
                "external_unverified": external_unverified,
                "pointer_dereferenced": false
            }),
        );
    }

    let observed_input_tokens_est = reducer
        .tokens_raw_est
        .or_else(|| Some(estimate_json_tokens(&Value::Object(metadata.clone()))));
    let summary_tokens_est = summary.as_deref().map(estimate_tokens);
    metadata.insert(
        "metrics".to_string(),
        metrics_value(observed_input_tokens_est, None, summary_tokens_est),
    );
    metadata.insert(
        "redaction".to_string(),
        report.to_value(policy, &sensitive, "raw_payload_not_accepted"),
    );

    let redaction_status = if report.has_redactions() {
        "redacted"
    } else {
        "passed"
    };
    let event_metadata = Value::Object(metadata);
    let event_id = create_context_event(
        conn,
        &NewContextEvent {
            repo_id: repo_id.as_deref(),
            workspace_path_hash: workspace_path_hash.as_deref(),
            git_branch: clean_optional(request.git_branch).as_deref(),
            worktree_name: clean_optional(request.worktree_name).as_deref(),
            commit_hash: clean_optional(request.commit_hash).as_deref(),
            session_id: &session_id,
            task_id: clean_optional(request.task_id.clone()).as_deref(),
            agent_id: clean_optional(request.agent_id.clone()).as_deref(),
            source: &source,
            event_type: &event_type,
            command_name: command_name.as_deref(),
            tool_name: tool_name.as_deref(),
            cwd: clean_optional(request.cwd).as_deref(),
            exit_code,
            started_at,
            finished_at,
            redaction_status,
            retention_policy: &retention_policy,
            raw_artifact_id: raw_artifact_id.as_deref(),
            raw_payload: None,
            metadata: &event_metadata,
        },
    )?;
    let event = load_event(conn, event_id)?;

    let mut summary_id = None;
    if let Some(summary_text) = summary.filter(|value| !value.trim().is_empty()) {
        let confidence = reducer
            .confidence
            .unwrap_or(if is_external { 0.7 } else { 1.0 });
        if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
            return Err(EngramError::InvalidInput(
                "reducer confidence must be between 0.0 and 1.0".to_string(),
            ));
        }

        let mut structured = object_map(
            reducer
                .structured_facts
                .map(|value| redact_json_value(policy, &mut report, "structured_facts", value))
                .transpose()?
                .unwrap_or_else(|| json!({})),
        );
        if let Some(files) = event.metadata.get("touched_files") {
            structured.insert("touched_files".to_string(), files.clone());
        }
        if let Some(errors) = event.metadata.get("key_errors") {
            structured.insert("key_errors".to_string(), errors.clone());
        }
        if let Some(external) = event.metadata.get("external_summary") {
            structured.insert("external_summary".to_string(), external.clone());
        }
        if let Some(labels) = event.metadata.get("labels") {
            structured.insert("labels".to_string(), labels.clone());
        }

        let mut warnings =
            redact_string_list_result(policy, &mut report, "warnings", &reducer.warnings)?;
        if external_unverified {
            push_unique(&mut warnings, "external_unverified");
        }
        if raw_pointer.is_some() {
            push_unique(&mut warnings, "raw_pointer_not_dereferenced");
        }
        let tokens_compact_est = reducer
            .tokens_compact_est
            .or_else(|| Some(estimate_tokens(&summary_text)));
        let summary_row_id = create_context_summary(
            conn,
            &NewContextSummary {
                source_event_id: event.id,
                source_artifact_id: event.raw_artifact_id.as_deref(),
                reducer_name: reducer_name(
                    &source,
                    external_reducer.as_deref(),
                    reducer.name.as_deref(),
                ),
                reducer_version: reducer_version(
                    source_version.as_deref(),
                    reducer.version.as_deref(),
                ),
                lossy: if is_external {
                    true
                } else {
                    reducer.lossy.unwrap_or(true)
                },
                confidence,
                summary: &summary_text,
                structured_facts: &Value::Object(structured),
                warnings: &json!(warnings),
                tokens_raw_est: reducer.tokens_raw_est,
                tokens_compact_est,
            },
        )?;
        summary_id = Some(load_summary_id(conn, summary_row_id)?);
    }

    Ok(ContextRecordResponse {
        created_ids: ContextRecordCreatedIds {
            event_id: event.id,
            summary_id,
            raw_artifact_id: event.raw_artifact_id.clone(),
        },
        redaction_status: redaction_status.to_string(),
        retention_policy,
        provenance: ProvenanceMetadata {
            source,
            source_version,
            repo_id,
            workspace_path_hash,
            session_id: Some(session_id),
            task_id: clean_optional(request.task_id),
            agent_id: clean_optional(request.agent_id),
            created_at: event.created_at,
        },
        metrics: ContextRecordMetrics {
            estimated: true,
            method: "chars_div_4_estimate_or_caller_supplied".to_string(),
            observed_input_tokens_est,
            summary_tokens_est,
            stored_artifact_tokens_est: None,
            estimated_savings_tokens: estimated_savings_tokens(
                observed_input_tokens_est,
                summary_tokens_est,
            ),
        },
    })
}

pub fn record_context_artifact(
    conn: &Connection,
    policy: &OperationalContextPolicy,
    request: ContextRecordArtifactRequest,
) -> Result<ContextRecordArtifactResponse> {
    let mut report = RedactionReport::new();
    let kind_raw = require_non_empty(request.kind, "kind")?;
    let kind = redact_field_result(policy, &mut report, "kind", &kind_raw)?;
    let repo_id = redact_optional_result(
        policy,
        &mut report,
        "repo_id",
        clean_optional(request.repo_id),
    )?;
    let workspace_path_hash = redact_optional_result(
        policy,
        &mut report,
        "workspace_path_hash",
        clean_optional(request.workspace_path_hash).or_else(|| clean_optional(request.workspace)),
    )?;
    if request.source_event_id.is_none() && repo_id.is_none() && workspace_path_hash.is_none() {
        return Err(EngramError::InvalidInput(
            "context_record_artifact requires source_event_id, repo_id, or workspace_path_hash/workspace"
                .to_string(),
        ));
    }

    let mut metadata = metadata_map(request.metadata);
    let command_hint = metadata
        .get("command")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| {
            metadata
                .get("command_name")
                .and_then(Value::as_str)
                .map(str::to_string)
        });
    let sensitive = policy.analyze_command(command_hint.as_deref());
    let retain_raw_requested = request.retain_raw.unwrap_or(false);
    if retain_raw_requested && request.raw_content.is_none() {
        return Err(EngramError::InvalidInput(
            "context_record_artifact retain_raw=true requires raw_content".to_string(),
        ));
    }
    if request.raw_content.is_some() && !retain_raw_requested {
        return Err(EngramError::InvalidInput(
            "context_record_artifact raw_content requires retain_raw=true; pointer-only is the default"
                .to_string(),
        ));
    }
    if retain_raw_requested && !policy.allow_raw_for(&sensitive) {
        return Err(EngramError::InvalidInput(
            "context_record_artifact raw retention is blocked for sensitive command context"
                .to_string(),
        ));
    }

    let source = redact_optional_result(
        policy,
        &mut report,
        "source",
        clean_optional(request.source),
    )?
    .unwrap_or_else(|| "context_record_artifact".to_string());
    let source_version = redact_optional_result(
        policy,
        &mut report,
        "source_version",
        clean_optional(request.source_version),
    )?;
    let raw_pointer = redact_optional_result(
        policy,
        &mut report,
        "raw_pointer",
        clean_optional(request.raw_pointer),
    )?;
    let uri = redact_optional_result(
        policy,
        &mut report,
        "uri",
        clean_optional(request.uri).or_else(|| raw_pointer.clone()),
    )?;
    let raw_content = request
        .raw_content
        .as_deref()
        .map(|content| redact_field_result(policy, &mut report, "raw_content", content))
        .transpose()?;
    let observed_input_tokens_est = raw_content.as_deref().map(estimate_tokens);
    let raw_bytes = raw_content.map(String::into_bytes);
    let stored_artifact_tokens_est = raw_bytes
        .as_ref()
        .map(|bytes| estimate_tokens(&String::from_utf8_lossy(bytes)));
    let metadata_value =
        redact_json_value(policy, &mut report, "metadata", Value::Object(metadata))?;
    metadata = object_map(metadata_value);
    metadata.insert("source".to_string(), json!(source));
    insert_opt(&mut metadata, "source_version", source_version.clone());
    insert_opt(&mut metadata, "raw_pointer", raw_pointer.clone());
    metadata.insert("pointer_dereferenced".to_string(), json!(false));
    metadata.insert(
        "metrics".to_string(),
        metrics_value(observed_input_tokens_est, stored_artifact_tokens_est, None),
    );
    metadata.insert(
        "redaction".to_string(),
        report.to_value(
            policy,
            &sensitive,
            if retain_raw_requested {
                "raw_retained_after_redaction"
            } else {
                "pointer_only"
            },
        ),
    );

    let redaction_status = if report.has_redactions() {
        ArtifactRedactionStatus::Redacted
    } else if raw_bytes.is_some() {
        ArtifactRedactionStatus::Passed
    } else {
        ArtifactRedactionStatus::NotRequired
    };
    let access_policy = clean_optional(request.access_policy)
        .as_deref()
        .map(ArtifactAccessPolicy::from_str)
        .transpose()?
        .unwrap_or_default();
    let retention_policy_name = clean_optional(request.retention_policy).unwrap_or_else(|| {
        if retain_raw_requested {
            "raw_retained".to_string()
        } else {
            "pointer_only".to_string()
        }
    });
    let artifact = create_context_artifact(
        conn,
        NewContextArtifact {
            id: clean_optional(request.id),
            source_event_id: request.source_event_id,
            repo_id: repo_id.clone(),
            workspace_path_hash: workspace_path_hash.clone(),
            session_id: clean_optional(request.session_id.clone()),
            task_id: clean_optional(request.task_id.clone()),
            agent_id: clean_optional(request.agent_id.clone()),
            kind,
            label: redact_optional_result(
                policy,
                &mut report,
                "label",
                clean_optional(request.label),
            )?,
            uri,
            media_type: redact_optional_result(
                policy,
                &mut report,
                "media_type",
                clean_optional(request.media_type),
            )?,
            content_sha256: clean_optional(request.content_sha256),
            byte_len: request.byte_len,
            raw_content: raw_bytes,
            retention: ArtifactRetentionPolicy {
                policy_name: retention_policy_name.clone(),
                retain_raw: retain_raw_requested,
                redaction_status,
                ttl_seconds: request.ttl_seconds,
                stale_after_seconds: request.stale_after_seconds,
                access_policy,
            },
            metadata: Value::Object(metadata),
        },
    )?;

    Ok(ContextRecordArtifactResponse {
        artifact_id: artifact.id,
        storage_kind: if artifact.retain_raw {
            "raw_retained".to_string()
        } else {
            "pointer_only".to_string()
        },
        redaction_status: artifact.redaction_status.as_str().to_string(),
        retention_policy: retention_policy_name,
        provenance: ProvenanceMetadata {
            source,
            source_version,
            repo_id,
            workspace_path_hash,
            session_id: clean_optional(request.session_id),
            task_id: clean_optional(request.task_id),
            agent_id: clean_optional(request.agent_id),
            created_at: artifact.created_at,
        },
        metrics: ContextRecordMetrics {
            estimated: true,
            method: "chars_div_4_estimate_or_caller_supplied".to_string(),
            observed_input_tokens_est,
            summary_tokens_est: None,
            stored_artifact_tokens_est,
            estimated_savings_tokens: estimated_savings_tokens(
                observed_input_tokens_est,
                stored_artifact_tokens_est,
            ),
        },
    })
}

fn reducer_name<'a>(
    source: &'a str,
    external_reducer: Option<&'a str>,
    reducer_name: Option<&'a str>,
) -> &'a str {
    reducer_name
        .and_then(non_empty)
        .or_else(|| external_reducer.and_then(non_empty))
        .unwrap_or(if source.eq_ignore_ascii_case("rtk") {
            "rtk_external_summary"
        } else {
            "context_record"
        })
}

fn reducer_version<'a>(
    source_version: Option<&'a str>,
    reducer_version: Option<&'a str>,
) -> &'a str {
    reducer_version
        .and_then(non_empty)
        .or_else(|| source_version.and_then(non_empty))
        .unwrap_or("1")
}

fn non_empty(value: &str) -> Option<&str> {
    if value.trim().is_empty() {
        None
    } else {
        Some(value)
    }
}

fn load_event(conn: &Connection, event_id: i64) -> Result<ContextEvent> {
    get_context_event(conn, event_id)?
        .ok_or_else(|| EngramError::Internal("context event insert was not readable".to_string()))
}

fn load_summary_id(conn: &Connection, summary_id: i64) -> Result<i64> {
    get_context_summary(conn, summary_id)?
        .map(|summary| summary.id)
        .ok_or_else(|| EngramError::Internal("context summary insert was not readable".to_string()))
}

fn redact_field_result(
    policy: &OperationalContextPolicy,
    report: &mut RedactionReport,
    field: &str,
    value: &str,
) -> Result<String> {
    redact_field(policy, report, field, value).map_err(redaction_error)
}

fn redact_optional_result(
    policy: &OperationalContextPolicy,
    report: &mut RedactionReport,
    field: &str,
    value: Option<String>,
) -> Result<Option<String>> {
    redact_optional_field(policy, report, field, &value).map_err(redaction_error)
}

fn redact_string_list_result(
    policy: &OperationalContextPolicy,
    report: &mut RedactionReport,
    field: &str,
    values: &[String],
) -> Result<Vec<String>> {
    redact_string_list(policy, report, field, values).map_err(redaction_error)
}

fn redact_json_value(
    policy: &OperationalContextPolicy,
    report: &mut RedactionReport,
    field: &str,
    value: Value,
) -> Result<Value> {
    match value {
        Value::String(value) => {
            redact_field_result(policy, report, field, &value).map(Value::String)
        }
        Value::Array(values) => values
            .into_iter()
            .enumerate()
            .map(|(idx, value)| {
                redact_json_value(policy, report, &format!("{field}[{idx}]"), value)
            })
            .collect::<Result<Vec<_>>>()
            .map(Value::Array),
        Value::Object(values) => {
            let mut output = Map::new();
            for (key, value) in values {
                let nested = format!("{field}.{key}");
                if sensitive_key(&key) {
                    report.record(
                        &nested,
                        &RedactedText {
                            text: String::new(),
                            redacted: true,
                            classes: vec!["metadata_sensitive_key".to_string()],
                        },
                    );
                    output.insert(
                        key,
                        Value::String("[REDACTED:metadata_sensitive_key]".to_string()),
                    );
                } else {
                    output.insert(key, redact_json_value(policy, report, &nested, value)?);
                }
            }
            Ok(Value::Object(output))
        }
        value => Ok(value),
    }
}

fn metadata_map(metadata: Option<Value>) -> Map<String, Value> {
    match metadata {
        Some(Value::Object(map)) => map,
        Some(value) => {
            let mut map = Map::new();
            map.insert("value".to_string(), value);
            map
        }
        None => Map::new(),
    }
}

fn object_map(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        other => {
            let mut map = Map::new();
            map.insert("value".to_string(), other);
            map
        }
    }
}

fn insert_opt(map: &mut Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(value) = value {
        map.insert(key.to_string(), json!(value));
    }
}

fn normalized_labels(values: impl Iterator<Item = String>) -> Vec<String> {
    let mut labels = Vec::new();
    for value in values {
        let label = value.trim().to_ascii_lowercase();
        if !label.is_empty() && !labels.iter().any(|existing| existing == &label) {
            labels.push(label);
        }
    }
    labels
}

fn push_label(labels: &mut Vec<String>, label: &str) {
    if !labels.iter().any(|existing| existing == label) {
        labels.push(label.to_string());
    }
}

fn push_unique(values: &mut Vec<String>, value: &str) {
    if !values.iter().any(|existing| existing == value) {
        values.push(value.to_string());
    }
}

fn require_non_empty(value: String, field: &str) -> Result<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        Err(EngramError::InvalidInput(format!("{field} is required")))
    } else {
        Ok(value)
    }
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn optional_i32(value: Option<i64>, field: &str) -> Result<Option<i32>> {
    value
        .map(|value| {
            i32::try_from(value).map_err(|_| {
                EngramError::InvalidInput(format!("{field} must fit in a 32-bit integer"))
            })
        })
        .transpose()
}

fn parse_datetime_or_now(value: Option<String>, field: &str) -> Result<DateTime<Utc>> {
    parse_optional_datetime(value, field).map(|value| value.unwrap_or_else(Utc::now))
}

fn parse_optional_datetime(value: Option<String>, field: &str) -> Result<Option<DateTime<Utc>>> {
    let Some(value) = clean_optional(value) else {
        return Ok(None);
    };
    DateTime::parse_from_rfc3339(&value)
        .map(|dt| Some(dt.with_timezone(&Utc)))
        .map_err(|err| EngramError::InvalidInput(format!("{field} must be RFC3339: {err}")))
}

fn sensitive_key(key: &str) -> bool {
    let lower = key.to_ascii_lowercase();
    lower.contains("password")
        || lower.contains("token")
        || lower.contains("secret")
        || lower.contains("api_key")
        || lower.contains("apikey")
        || lower.contains("authorization")
        || lower.contains("cookie")
}

fn redaction_error(err: impl std::fmt::Display) -> EngramError {
    EngramError::InvalidInput(format!("operational context redaction failed: {err}"))
}
