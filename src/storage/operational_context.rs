//! Operational context storage model.
//!
//! Context events are observed facts from agents, tools, and commands.
//! Context summaries are derived, lossy or lossless reducer outputs with
//! explicit source provenance back to the event they summarized.

use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, types::Type, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::audit::{log_audit, AuditAction};
use crate::context::artifact::{
    ArtifactRetrievalRequest, ContextArtifact, NewContextArtifact, RetrievedContextArtifact,
};
use crate::error::{EngramError, Result};

macro_rules! artifact_metadata_select {
    ($tail:literal) => {
        concat!(
            "SELECT id, source_event_id, repo_id, workspace_path_hash, session_id,
                    task_id, agent_id, kind, label, uri, media_type, content_sha256,
                    byte_len, redaction_status, retention_policy, access_policy,
                    retain_raw, stale_at, expires_at, metadata, created_at
             FROM context_artifacts ",
            $tail
        )
    };
}

/// Durable observed operational fact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextEvent {
    pub id: i64,
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub git_branch: Option<String>,
    pub worktree_name: Option<String>,
    pub commit_hash: Option<String>,
    pub session_id: String,
    pub task_id: Option<String>,
    pub agent_id: Option<String>,
    pub source: String,
    pub event_type: String,
    pub command_name: Option<String>,
    pub tool_name: Option<String>,
    pub cwd: Option<String>,
    pub exit_code: Option<i32>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub redaction_status: String,
    pub retention_policy: String,
    pub raw_artifact_id: Option<String>,
    pub raw_payload: Option<serde_json::Value>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Input for inserting a context event.
pub struct NewContextEvent<'a> {
    pub repo_id: Option<&'a str>,
    pub workspace_path_hash: Option<&'a str>,
    pub git_branch: Option<&'a str>,
    pub worktree_name: Option<&'a str>,
    pub commit_hash: Option<&'a str>,
    pub session_id: &'a str,
    pub task_id: Option<&'a str>,
    pub agent_id: Option<&'a str>,
    pub source: &'a str,
    pub event_type: &'a str,
    pub command_name: Option<&'a str>,
    pub tool_name: Option<&'a str>,
    pub cwd: Option<&'a str>,
    pub exit_code: Option<i32>,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub redaction_status: &'a str,
    pub retention_policy: &'a str,
    pub raw_artifact_id: Option<&'a str>,
    pub raw_payload: Option<&'a serde_json::Value>,
    pub metadata: &'a serde_json::Value,
}

/// Durable reducer output derived from operational context events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextSummary {
    pub id: i64,
    pub source_event_id: i64,
    pub source_artifact_id: Option<String>,
    pub reducer_name: String,
    pub reducer_version: String,
    pub lossy: bool,
    pub confidence: f64,
    pub summary: String,
    pub structured_facts: serde_json::Value,
    pub warnings: serde_json::Value,
    pub tokens_raw_est: Option<i64>,
    pub tokens_compact_est: Option<i64>,
    pub created_at: DateTime<Utc>,
}

/// Input for inserting a context summary.
pub struct NewContextSummary<'a> {
    pub source_event_id: i64,
    pub source_artifact_id: Option<&'a str>,
    pub reducer_name: &'a str,
    pub reducer_version: &'a str,
    pub lossy: bool,
    pub confidence: f64,
    pub summary: &'a str,
    pub structured_facts: &'a serde_json::Value,
    pub warnings: &'a serde_json::Value,
    pub tokens_raw_est: Option<i64>,
    pub tokens_compact_est: Option<i64>,
}

/// Insert an observed operational context event.
pub fn create_context_event(conn: &Connection, event: &NewContextEvent<'_>) -> Result<i64> {
    validate_event(event)?;

    let raw_payload = event.raw_payload.map(serde_json::to_string).transpose()?;
    let metadata = serde_json::to_string(event.metadata)?;
    let started_at = event.started_at.to_rfc3339();
    let finished_at = event.finished_at.as_ref().map(DateTime::to_rfc3339);
    let created_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO context_events
            (repo_id, workspace_path_hash, git_branch, worktree_name, commit_hash,
             session_id, task_id, agent_id, source, event_type, command_name,
             tool_name, cwd, exit_code, started_at, finished_at, redaction_status,
             retention_policy, raw_artifact_id, raw_payload, metadata, created_at)
         VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11,
             ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
        params![
            event.repo_id,
            event.workspace_path_hash,
            event.git_branch,
            event.worktree_name,
            event.commit_hash,
            event.session_id,
            event.task_id,
            event.agent_id,
            event.source,
            event.event_type,
            event.command_name,
            event.tool_name,
            event.cwd,
            event.exit_code,
            started_at,
            finished_at,
            event.redaction_status,
            event.retention_policy,
            event.raw_artifact_id,
            raw_payload,
            metadata,
            created_at,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Fetch one context event by row id.
pub fn get_context_event(conn: &Connection, id: i64) -> Result<Option<ContextEvent>> {
    let mut stmt = conn.prepare(
        "SELECT id, repo_id, workspace_path_hash, git_branch, worktree_name,
                commit_hash, session_id, task_id, agent_id, source, event_type,
                command_name, tool_name, cwd, exit_code, started_at, finished_at,
                redaction_status, retention_policy, raw_artifact_id, raw_payload,
                metadata, created_at
         FROM context_events
         WHERE id = ?1",
    )?;
    let event = stmt.query_row(params![id], map_context_event).optional()?;
    Ok(event)
}

/// List recent context events for a session, newest first.
pub fn list_context_events_for_session(
    conn: &Connection,
    session_id: &str,
    limit: Option<i64>,
) -> Result<Vec<ContextEvent>> {
    let limit = limit.unwrap_or(100).clamp(1, 1000);
    let mut stmt = conn.prepare(
        "SELECT id, repo_id, workspace_path_hash, git_branch, worktree_name,
                commit_hash, session_id, task_id, agent_id, source, event_type,
                command_name, tool_name, cwd, exit_code, started_at, finished_at,
                redaction_status, retention_policy, raw_artifact_id, raw_payload,
                metadata, created_at
         FROM context_events
         WHERE session_id = ?1
         ORDER BY started_at DESC, id DESC
         LIMIT ?2",
    )?;
    let events = stmt
        .query_map(params![session_id, limit], map_context_event)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(events)
}

/// Insert a derived context summary with source provenance.
pub fn create_context_summary(conn: &Connection, summary: &NewContextSummary<'_>) -> Result<i64> {
    validate_summary(summary)?;

    let structured_facts = serde_json::to_string(summary.structured_facts)?;
    let warnings = serde_json::to_string(summary.warnings)?;
    let created_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO context_summaries
            (source_event_id, source_artifact_id, reducer_name, reducer_version,
             lossy, confidence, summary, structured_facts, warnings,
             tokens_raw_est, tokens_compact_est, created_at)
         VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            summary.source_event_id,
            summary.source_artifact_id,
            summary.reducer_name,
            summary.reducer_version,
            summary.lossy as i32,
            summary.confidence,
            summary.summary,
            structured_facts,
            warnings,
            summary.tokens_raw_est,
            summary.tokens_compact_est,
            created_at,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

/// Fetch one context summary by row id.
pub fn get_context_summary(conn: &Connection, id: i64) -> Result<Option<ContextSummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_event_id, source_artifact_id, reducer_name,
                reducer_version, lossy, confidence, summary, structured_facts,
                warnings, tokens_raw_est, tokens_compact_est, created_at
         FROM context_summaries
         WHERE id = ?1",
    )?;
    let summary = stmt
        .query_row(params![id], map_context_summary)
        .optional()?;
    Ok(summary)
}

/// List summaries derived from one source event, newest first.
pub fn list_context_summaries_for_event(
    conn: &Connection,
    source_event_id: i64,
) -> Result<Vec<ContextSummary>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_event_id, source_artifact_id, reducer_name,
                reducer_version, lossy, confidence, summary, structured_facts,
                warnings, tokens_raw_est, tokens_compact_est, created_at
         FROM context_summaries
         WHERE source_event_id = ?1
         ORDER BY created_at DESC, id DESC",
    )?;
    let summaries = stmt
        .query_map(params![source_event_id], map_context_summary)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(summaries)
}

/// Insert policy-controlled operational context artifact metadata.
///
/// Raw content is discarded unless the caller explicitly sets
/// `retention.retain_raw = true` and supplies an allowed redaction status.
pub fn create_context_artifact(
    conn: &Connection,
    input: NewContextArtifact,
) -> Result<ContextArtifact> {
    validate_new_artifact(&input)?;

    let id = input
        .id
        .filter(|id| !id.trim().is_empty())
        .unwrap_or_else(|| format!("ctx_artifact:{}", uuid::Uuid::new_v4()));
    let raw_present = input.raw_content.is_some();
    input.retention.validate_raw_storage(raw_present)?;

    let raw_to_store = if input.retention.retain_raw {
        input.raw_content
    } else {
        None
    };
    let raw_retained = raw_to_store.is_some();
    let content_sha256 = raw_to_store
        .as_deref()
        .map(sha256_bytes)
        .or(input.content_sha256);
    let byte_len = raw_to_store
        .as_ref()
        .map(|bytes| bytes.len() as i64)
        .or(input.byte_len);
    let now = Utc::now();
    let stale_at = seconds_from_now(now, input.retention.stale_after_seconds);
    let expires_at = seconds_from_now(now, input.retention.ttl_seconds);
    let metadata = serde_json::to_string(&input.metadata)?;

    conn.execute(
        "INSERT INTO context_artifacts
            (id, source_event_id, repo_id, workspace_path_hash, session_id,
             task_id, agent_id, kind, label, uri, media_type, content_sha256,
             byte_len, redaction_status, retention_policy, access_policy,
             retain_raw, raw_content, stale_at, expires_at, metadata, created_at)
         VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
             ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
        params![
            id,
            input.source_event_id,
            input.repo_id,
            input.workspace_path_hash,
            input.session_id,
            input.task_id,
            input.agent_id,
            input.kind,
            input.label,
            input.uri,
            input.media_type,
            content_sha256,
            byte_len,
            input.retention.redaction_status.as_str(),
            input.retention.policy_name,
            input.retention.access_policy.as_str(),
            raw_retained as i32,
            raw_to_store,
            stale_at.map(|ts| ts.to_rfc3339()),
            expires_at.map(|ts| ts.to_rfc3339()),
            metadata,
            now.to_rfc3339(),
        ],
    )?;

    get_context_artifact(conn, &id)?
        .ok_or_else(|| EngramError::Internal("context artifact insert was not readable".into()))
}

/// Fetch public artifact metadata by artifact id. Raw content is intentionally absent.
pub fn get_context_artifact(
    conn: &Connection,
    artifact_id: &str,
) -> Result<Option<ContextArtifact>> {
    conn.query_row(
        artifact_metadata_select!("WHERE id = ?1"),
        params![artifact_id],
        context_artifact_from_row,
    )
    .optional()
    .map_err(Into::into)
}

/// List public artifact metadata for a source event. Raw content is intentionally absent.
pub fn list_context_artifacts_for_event(
    conn: &Connection,
    source_event_id: i64,
    limit: Option<i64>,
) -> Result<Vec<ContextArtifact>> {
    let limit = limit.unwrap_or(100).clamp(1, 1000);
    let mut stmt = conn.prepare(artifact_metadata_select!(
        "WHERE source_event_id = ?1 ORDER BY created_at DESC LIMIT ?2"
    ))?;
    let artifacts = stmt
        .query_map(params![source_event_id, limit], context_artifact_from_row)?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(artifacts)
}

/// Explicitly retrieve retained raw artifact content after policy checks.
pub fn retrieve_context_artifact_raw(
    conn: &Connection,
    request: ArtifactRetrievalRequest,
) -> Result<RetrievedContextArtifact> {
    if request.artifact_id.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "artifact_id is required for raw artifact retrieval".into(),
        ));
    }

    let loaded = load_artifact_with_raw(conn, &request.artifact_id)?;
    let Some((artifact, raw_content)) = loaded else {
        record_artifact_access(
            conn,
            &request,
            "not_found",
            "artifact_not_found",
            None,
            false,
        )?;
        return Err(EngramError::Storage(format!(
            "Context artifact not found: {}",
            request.artifact_id
        )));
    };

    if !artifact.access_allowed(&request) {
        record_artifact_access(
            conn,
            &request,
            "denied",
            "access_policy_denied",
            None,
            false,
        )?;
        return Err(EngramError::Unauthorized(
            "Artifact access policy denied raw retrieval".into(),
        ));
    }

    let now = Utc::now();
    if artifact.is_expired_at(now) {
        record_artifact_access(conn, &request, "denied", "artifact_expired", None, false)?;
        return Err(EngramError::Unauthorized(
            "Artifact raw content is expired by retention policy".into(),
        ));
    }

    if artifact.is_stale_at(now) && !request.allow_stale {
        record_artifact_access(conn, &request, "denied", "artifact_stale", None, false)?;
        return Err(EngramError::Unauthorized(
            "Artifact raw content is stale; retry with allow_stale when appropriate".into(),
        ));
    }

    if !artifact.retain_raw || raw_content.is_none() {
        record_artifact_access(
            conn,
            &request,
            "denied",
            "raw_content_not_retained",
            None,
            false,
        )?;
        return Err(EngramError::Unauthorized(
            "Raw content was not retained for this artifact".into(),
        ));
    }

    let raw_content = raw_content.unwrap_or_default();
    let original_bytes = raw_content.len();
    let max_bytes = request.max_bytes.unwrap_or(original_bytes);
    let truncated = raw_content.len() > max_bytes;
    let content = if truncated {
        raw_content[..max_bytes].to_vec()
    } else {
        raw_content
    };
    let returned_bytes = content.len();

    record_artifact_access(
        conn,
        &request,
        "allowed",
        "raw_retrieved",
        Some(returned_bytes),
        truncated,
    )?;

    Ok(RetrievedContextArtifact {
        artifact,
        content,
        returned_bytes,
        original_bytes,
        truncated,
    })
}

fn validate_event(event: &NewContextEvent<'_>) -> Result<()> {
    if is_blank(event.repo_id) && is_blank(event.workspace_path_hash) {
        return Err(EngramError::InvalidInput(
            "context event requires repo_id or workspace_path_hash".into(),
        ));
    }
    if event.session_id.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "context event session_id must not be empty".into(),
        ));
    }
    if event.source.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "context event source must not be empty".into(),
        ));
    }
    if event.event_type.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "context event event_type must not be empty".into(),
        ));
    }
    if event.event_type.eq_ignore_ascii_case("command") && event.exit_code.is_none() {
        return Err(EngramError::InvalidInput(
            "command context events require exit_code".into(),
        ));
    }
    if event.event_type.eq_ignore_ascii_case("command") && is_blank(event.command_name) {
        return Err(EngramError::InvalidInput(
            "command context events require command_name".into(),
        ));
    }
    if event.event_type.eq_ignore_ascii_case("tool") && is_blank(event.tool_name) {
        return Err(EngramError::InvalidInput(
            "tool context events require tool_name".into(),
        ));
    }
    Ok(())
}

fn is_blank(value: Option<&str>) -> bool {
    value.map(str::trim).is_none_or(str::is_empty)
}

fn validate_summary(summary: &NewContextSummary<'_>) -> Result<()> {
    if summary.reducer_name.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "context summary reducer_name must not be empty".into(),
        ));
    }
    if summary.reducer_version.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "context summary reducer_version must not be empty".into(),
        ));
    }
    if !summary.confidence.is_finite() || !(0.0..=1.0).contains(&summary.confidence) {
        return Err(EngramError::InvalidInput(
            "context summary confidence must be between 0.0 and 1.0".into(),
        ));
    }
    if summary.summary.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "context summary text must not be empty".into(),
        ));
    }
    if summary.tokens_raw_est.is_some_and(|tokens| tokens < 0)
        || summary.tokens_compact_est.is_some_and(|tokens| tokens < 0)
    {
        return Err(EngramError::InvalidInput(
            "context summary token estimates must be non-negative".into(),
        ));
    }
    Ok(())
}

fn validate_new_artifact(input: &NewContextArtifact) -> Result<()> {
    if input.kind.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "context artifacts require kind".into(),
        ));
    }
    if input.source_event_id.is_none()
        && is_blank(input.repo_id.as_deref())
        && is_blank(input.workspace_path_hash.as_deref())
    {
        return Err(EngramError::InvalidInput(
            "context artifacts require source_event_id, repo_id, or workspace_path_hash".into(),
        ));
    }
    if !input.retention.redaction_status.allows_raw_storage() {
        return Err(EngramError::InvalidInput(format!(
            "context artifacts require redaction to pass before storage; status={}",
            input.retention.redaction_status.as_str()
        )));
    }
    input
        .retention
        .validate_raw_storage(input.raw_content.is_some())
}

fn load_artifact_with_raw(
    conn: &Connection,
    artifact_id: &str,
) -> Result<Option<(ContextArtifact, Option<Vec<u8>>)>> {
    conn.query_row(
        "SELECT id, source_event_id, repo_id, workspace_path_hash, session_id,
                task_id, agent_id, kind, label, uri, media_type, content_sha256,
                byte_len, redaction_status, retention_policy, access_policy,
                retain_raw, stale_at, expires_at, metadata, created_at, raw_content
         FROM context_artifacts
         WHERE id = ?1",
        params![artifact_id],
        |row| {
            let artifact = context_artifact_from_row(row)?;
            let raw_content: Option<Vec<u8>> = row.get(21)?;
            Ok((artifact, raw_content))
        },
    )
    .optional()
    .map_err(Into::into)
}

fn record_artifact_access(
    conn: &Connection,
    request: &ArtifactRetrievalRequest,
    access_result: &str,
    reason: &str,
    returned_bytes: Option<usize>,
    truncated: bool,
) -> Result<()> {
    let created_at = Utc::now().to_rfc3339();
    let max_bytes = request.max_bytes.map(|value| value as i64);
    let returned_bytes = returned_bytes.map(|value| value as i64);

    conn.execute(
        "INSERT INTO context_artifact_access_log
            (artifact_id, requester_agent_id, session_id, task_id, repo_id,
             workspace_path_hash, access_result, reason, max_bytes,
             returned_bytes, truncated, created_at)
         VALUES
            (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            request.artifact_id.as_str(),
            request.requester_agent_id.as_deref(),
            request.session_id.as_deref(),
            request.task_id.as_deref(),
            request.repo_id.as_deref(),
            request.workspace_path_hash.as_deref(),
            access_result,
            reason,
            max_bytes,
            returned_bytes,
            truncated as i32,
            created_at,
        ],
    )?;

    let changes = serde_json::json!({
        "artifact_id": request.artifact_id.as_str(),
        "access_result": access_result,
        "reason": reason,
        "max_bytes": max_bytes,
        "returned_bytes": returned_bytes,
        "truncated": truncated,
        "retrieval_reason": request.reason.as_deref(),
    });
    let _ = log_audit(
        conn,
        AuditAction::Export,
        None,
        request.requester_agent_id.as_deref(),
        Some(&changes),
        None,
    )
    .map_err(|err| tracing::warn!("artifact audit_log emit failed: {err}"));

    Ok(())
}

fn map_context_event(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContextEvent> {
    let raw_payload: Option<String> = row.get(20)?;
    let metadata: String = row.get(21)?;
    Ok(ContextEvent {
        id: row.get(0)?,
        repo_id: row.get(1)?,
        workspace_path_hash: row.get(2)?,
        git_branch: row.get(3)?,
        worktree_name: row.get(4)?,
        commit_hash: row.get(5)?,
        session_id: row.get(6)?,
        task_id: row.get(7)?,
        agent_id: row.get(8)?,
        source: row.get(9)?,
        event_type: row.get(10)?,
        command_name: row.get(11)?,
        tool_name: row.get(12)?,
        cwd: row.get(13)?,
        exit_code: row.get(14)?,
        started_at: parse_datetime(row.get(15)?, 15)?,
        finished_at: parse_optional_datetime(row.get(16)?, 16)?,
        redaction_status: row.get(17)?,
        retention_policy: row.get(18)?,
        raw_artifact_id: row.get(19)?,
        raw_payload: parse_optional_json(raw_payload, 20)?,
        metadata: parse_json(metadata, 21)?,
        created_at: parse_datetime(row.get(22)?, 22)?,
    })
}

fn map_context_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContextSummary> {
    let lossy: i32 = row.get(5)?;
    let structured_facts: String = row.get(8)?;
    let warnings: String = row.get(9)?;
    Ok(ContextSummary {
        id: row.get(0)?,
        source_event_id: row.get(1)?,
        source_artifact_id: row.get(2)?,
        reducer_name: row.get(3)?,
        reducer_version: row.get(4)?,
        lossy: lossy != 0,
        confidence: row.get(6)?,
        summary: row.get(7)?,
        structured_facts: parse_json(structured_facts, 8)?,
        warnings: parse_json(warnings, 9)?,
        tokens_raw_est: row.get(10)?,
        tokens_compact_est: row.get(11)?,
        created_at: parse_datetime(row.get(12)?, 12)?,
    })
}

fn context_artifact_from_row(row: &Row<'_>) -> rusqlite::Result<ContextArtifact> {
    let metadata: String = row.get("metadata")?;
    let redaction_status: String = row.get("redaction_status")?;
    let access_policy: String = row.get("access_policy")?;
    let retain_raw: i64 = row.get("retain_raw")?;
    Ok(ContextArtifact {
        id: row.get("id")?,
        source_event_id: row.get("source_event_id")?,
        repo_id: row.get("repo_id")?,
        workspace_path_hash: row.get("workspace_path_hash")?,
        session_id: row.get("session_id")?,
        task_id: row.get("task_id")?,
        agent_id: row.get("agent_id")?,
        kind: row.get("kind")?,
        label: row.get("label")?,
        uri: row.get("uri")?,
        media_type: row.get("media_type")?,
        content_sha256: row.get("content_sha256")?,
        byte_len: row.get("byte_len")?,
        redaction_status: redaction_status
            .parse()
            .map_err(|err: EngramError| artifact_conversion_error(err.to_string()))?,
        retention_policy: row.get("retention_policy")?,
        access_policy: access_policy
            .parse()
            .map_err(|err: EngramError| artifact_conversion_error(err.to_string()))?,
        retain_raw: retain_raw != 0,
        stale_at: parse_artifact_optional_datetime(row.get("stale_at")?)?,
        expires_at: parse_artifact_optional_datetime(row.get("expires_at")?)?,
        metadata: parse_artifact_json_value(&metadata),
        created_at: parse_artifact_datetime(row.get("created_at")?)?,
    })
}

fn parse_datetime(value: String, column: usize) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| rusqlite::Error::FromSqlConversionFailure(column, Type::Text, Box::new(err)))
}

fn parse_optional_datetime(
    value: Option<String>,
    column: usize,
) -> rusqlite::Result<Option<DateTime<Utc>>> {
    value.map(|v| parse_datetime(v, column)).transpose()
}

fn parse_json(value: String, column: usize) -> rusqlite::Result<serde_json::Value> {
    serde_json::from_str(&value)
        .map_err(|err| rusqlite::Error::FromSqlConversionFailure(column, Type::Text, Box::new(err)))
}

fn parse_optional_json(
    value: Option<String>,
    column: usize,
) -> rusqlite::Result<Option<serde_json::Value>> {
    value.map(|v| parse_json(v, column)).transpose()
}

fn parse_artifact_datetime(value: String) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&value)
        .map(|ts| ts.with_timezone(&Utc))
        .map_err(|err| artifact_conversion_error(err.to_string()))
}

fn parse_artifact_optional_datetime(
    value: Option<String>,
) -> rusqlite::Result<Option<DateTime<Utc>>> {
    value.map(parse_artifact_datetime).transpose()
}

fn parse_artifact_json_value(value: &str) -> serde_json::Value {
    serde_json::from_str(value).unwrap_or_else(|_| serde_json::json!({}))
}

fn artifact_conversion_error(message: String) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        0,
        Type::Text,
        Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            message,
        )),
    )
}

fn seconds_from_now(now: DateTime<Utc>, seconds: Option<i64>) -> Option<DateTime<Utc>> {
    seconds.and_then(|seconds| {
        if seconds > 0 {
            Some(now + Duration::seconds(seconds))
        } else {
            None
        }
    })
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::migrations::run_migrations;

    fn conn() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        run_migrations(&c).unwrap();
        c
    }

    fn event<'a>(
        metadata: &'a serde_json::Value,
        raw_payload: Option<&'a serde_json::Value>,
    ) -> NewContextEvent<'a> {
        NewContextEvent {
            repo_id: Some("github:aiconnai/engram"),
            workspace_path_hash: None,
            git_branch: Some("feat/oc-storage-model"),
            worktree_name: Some("oc-storage"),
            commit_hash: Some("abc123"),
            session_id: "sess-1",
            task_id: Some("ENGRA-68"),
            agent_id: Some("codex"),
            source: "codex",
            event_type: "command",
            command_name: Some("cargo"),
            tool_name: None,
            cwd: Some("/repo"),
            exit_code: Some(0),
            started_at: Utc::now(),
            finished_at: Some(Utc::now()),
            redaction_status: "redacted",
            retention_policy: "default",
            raw_artifact_id: Some("artifact-1"),
            raw_payload,
            metadata,
        }
    }

    #[test]
    fn command_events_require_exit_code() {
        let c = conn();
        let metadata = serde_json::json!({});
        let mut input = event(&metadata, None);
        input.exit_code = None;

        let result = create_context_event(&c, &input);
        assert!(
            result.is_err(),
            "command context event must reject missing exit_code"
        );
    }

    #[test]
    fn create_event_and_summary_preserve_provenance() {
        let c = conn();
        let metadata = serde_json::json!({"scope": "storage"});
        let raw_payload = serde_json::json!({"argv": ["cargo", "check"]});
        let event_id = create_context_event(&c, &event(&metadata, Some(&raw_payload))).unwrap();

        let summary_id = create_context_summary(
            &c,
            &NewContextSummary {
                source_event_id: event_id,
                source_artifact_id: Some("artifact-1"),
                reducer_name: "command_savings",
                reducer_version: "1.0.0",
                lossy: true,
                confidence: 0.9,
                summary: "Cargo command completed successfully.",
                structured_facts: &serde_json::json!({"exit_code": 0}),
                warnings: &serde_json::json!([]),
                tokens_raw_est: Some(120),
                tokens_compact_est: Some(12),
            },
        )
        .unwrap();

        let event = get_context_event(&c, event_id).unwrap().unwrap();
        assert_eq!(event.exit_code, Some(0));
        assert_eq!(event.raw_payload, Some(raw_payload));

        let summary = get_context_summary(&c, summary_id).unwrap().unwrap();
        assert_eq!(summary.source_event_id, event_id);
        assert_eq!(summary.source_artifact_id.as_deref(), Some("artifact-1"));
        assert_eq!(summary.reducer_name, "command_savings");
        assert_eq!(summary.reducer_version, "1.0.0");
        assert!(summary.lossy);
    }
}
