//! Operational-context event/artifact tools (context_record, artifacts, search, bundle).
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{json, Value};

use super::super::HandlerContext;

pub fn context_record(ctx: &HandlerContext, params: Value) -> Value {
    let policy = crate::context::policy::OperationalContextPolicy::from_params(&params);
    let request: crate::context::ContextRecordRequest = match serde_json::from_value(params) {
        Ok(request) => request,
        Err(e) => return json!({"error": e.to_string()}),
    };

    ctx.storage
        .with_connection(|conn| crate::context::record_context(conn, &policy, request))
        .map(|response| json!(response))
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

/// Record an Operational Context artifact pointer or policy-approved raw blob.
pub fn context_record_artifact(ctx: &HandlerContext, params: Value) -> Value {
    let policy = crate::context::policy::OperationalContextPolicy::from_params(&params);
    let request: crate::context::ContextRecordArtifactRequest = match serde_json::from_value(params)
    {
        Ok(request) => request,
        Err(e) => return json!({"error": e.to_string()}),
    };

    ctx.storage
        .with_connection(|conn| crate::context::record_context_artifact(conn, &policy, request))
        .map(|response| json!(response))
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

#[derive(Debug, Deserialize)]
struct ContextGetArtifactRequest {
    artifact_id: String,
    requester_agent_id: Option<String>,
    session_id: Option<String>,
    task_id: Option<String>,
    repo_id: Option<String>,
    workspace_path_hash: Option<String>,
    #[serde(default)]
    workspace: Option<String>,
    max_bytes: Option<usize>,
    #[serde(default)]
    allow_stale: bool,
    #[serde(default = "default_require_redacted")]
    require_redacted: bool,
    reason: String,
}

fn default_require_redacted() -> bool {
    true
}

/// Explicitly retrieve retained Operational Context artifact content.
pub fn context_get_artifact(ctx: &HandlerContext, params: Value) -> Value {
    let request: ContextGetArtifactRequest = match serde_json::from_value(params) {
        Ok(request) => request,
        Err(e) => return json!({"error": e.to_string()}),
    };
    if request.reason.trim().is_empty() {
        return json!({"error": "context_get_artifact requires a non-empty reason"});
    }

    let require_redacted = request.require_redacted;
    let storage_request = crate::storage::ArtifactRetrievalRequest {
        artifact_id: request.artifact_id,
        requester_agent_id: request.requester_agent_id,
        session_id: request.session_id,
        task_id: request.task_id,
        repo_id: request.repo_id,
        workspace_path_hash: request.workspace_path_hash.or(request.workspace),
        max_bytes: request.max_bytes,
        allow_stale: request.allow_stale,
        reason: Some(request.reason),
    };

    ctx.storage
        .with_connection(|conn| {
            crate::storage::retrieve_context_artifact_raw(conn, storage_request)
        })
        .map(|retrieved| {
            if require_redacted && !retrieved.artifact.redaction_status.allows_raw_storage() {
                return json!({
                    "error": "artifact redaction status does not permit raw retrieval",
                    "redaction_status": retrieved.artifact.redaction_status.as_str()
                });
            }

            let now = Utc::now();
            let stale = retrieved.artifact.is_stale_at(now);
            let expired = retrieved.artifact.is_expired_at(now);
            let returned_bytes = retrieved.returned_bytes;
            let original_bytes = retrieved.original_bytes;
            let truncated = retrieved.truncated;
            let artifact = retrieved.artifact;
            let (encoding, content) = match String::from_utf8(retrieved.content) {
                Ok(content) => ("utf8", content),
                Err(err) => ("base64", BASE64.encode(err.into_bytes())),
            };

            json!({
                "artifact": artifact,
                "content": content,
                "encoding": encoding,
                "returned_bytes": returned_bytes,
                "original_bytes": original_bytes,
                "truncated": truncated,
                "stale": stale,
                "expired": expired
            })
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

/// Search scoped Operational Context events and derived summaries.
pub fn context_search(ctx: &HandlerContext, params: Value) -> Value {
    let query = match params.get("query").and_then(|v| v.as_str()) {
        Some(query) if !query.trim().is_empty() => query,
        _ => return json!({"error": "query is required"}),
    };
    let mut request: crate::context::ContextSearchRequest =
        match serde_json::from_value(params.clone()) {
            Ok(request) => request,
            Err(e) => return json!({"error": e.to_string()}),
        };
    request.query = Some(query.to_string());

    ctx.storage
        .with_connection(|conn| crate::context::search_context(conn, &request))
        .map(|response| json!(response))
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

/// Build a compact Operational Context bundle for resuming work.
pub fn context_build_bundle(ctx: &HandlerContext, params: Value) -> Value {
    let request: crate::context::ContextBundleRequest = match serde_json::from_value(params) {
        Ok(request) => request,
        Err(e) => return json!({"error": e.to_string()}),
    };

    ctx.storage
        .with_connection(|conn| crate::context::build_context_bundle(conn, &request))
        .map(|bundle| json!(bundle))
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

// ── Fact extraction ───────────────────────────────────────────────────────────
