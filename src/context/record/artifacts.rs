//! Artifact recording path for operational context.

use crate::context::artifact::{
    ArtifactAccessPolicy, ArtifactRedactionStatus, ArtifactRetentionPolicy, NewContextArtifact,
};
use crate::context::metrics::{estimate_tokens, estimated_savings_tokens, metrics_value};
use crate::context::policy::{OperationalContextPolicy, RedactionReport};
use crate::context::record::helpers::*;
use crate::error::{EngramError, Result};
use crate::storage::create_context_artifact;
use rusqlite::Connection;
use serde_json::Value;
use std::str::FromStr;

use super::ContextRecordArtifactRequest;
use super::{ContextRecordArtifactResponse, ContextRecordMetrics, ProvenanceMetadata};

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
    metadata.insert("source".to_string(), serde_json::json!(source));
    insert_opt(&mut metadata, "source_version", source_version.clone());
    insert_opt(&mut metadata, "raw_pointer", raw_pointer.clone());
    metadata.insert("pointer_dereferenced".to_string(), serde_json::json!(false));
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
