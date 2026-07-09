//! Event recording path for operational context.

use crate::context::metrics::{
    estimate_json_tokens, estimate_tokens, estimated_savings_tokens, metrics_value,
};
use crate::context::policy::{OperationalContextPolicy, RedactionReport};
use crate::context::record::helpers::*;
use crate::error::{EngramError, Result};
use crate::storage::{
    create_context_event, create_context_summary, NewContextEvent, NewContextSummary,
};
use rusqlite::Connection;
use serde_json::{json, Value};

use super::{
    ContextRecordCreatedIds, ContextRecordMetrics, ContextRecordRequest, ContextRecordResponse,
    ProvenanceMetadata,
};

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
