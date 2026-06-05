//! Storage helpers for Operational Context events, summaries, and artifacts.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::context::{
    json_array_or_default, json_object_or_default, now_rfc3339, validate_context_scope,
    ArtifactRetrievalRequest, ContextArtifact, ContextEvent, ContextSummary, NewContextArtifact,
    NewContextEvent, NewContextSummary, RetrievedContextArtifact,
};
use crate::context::metrics::attach_summary_estimate_metadata;
use crate::error::{EngramError, Result};

pub fn create_context_event(conn: &Connection, event: &NewContextEvent) -> Result<ContextEvent> {
    validate_context_scope(
        event.repo_id.as_deref(),
        event.workspace_path_hash.as_deref(),
    )?;
    if event.session_id.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "context event requires session_id".to_string(),
        ));
    }
    if event.source.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "context event requires source".to_string(),
        ));
    }
    if event.event_type.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "context event requires event_type".to_string(),
        ));
    }
    if event.event_type.eq_ignore_ascii_case("command")
        && (event.command_name.as_deref().is_none_or(str::is_empty)
            || event.exit_code.is_none())
    {
        return Err(EngramError::InvalidInput(
            "command context events require command_name and exit_code".to_string(),
        ));
    }
    if event.event_type.eq_ignore_ascii_case("tool")
        && event.tool_name.as_deref().is_none_or(str::is_empty)
    {
        return Err(EngramError::InvalidInput(
            "tool context events require tool_name".to_string(),
        ));
    }

    let created_at = now_rfc3339();
    let started_at = event
        .started_at
        .clone()
        .unwrap_or_else(|| created_at.clone());
    let metadata = serde_json::to_string(&json_object_or_default(&event.metadata))?;

    conn.execute(
        "INSERT INTO context_events
             (repo_id, workspace_path_hash, git_branch, worktree_name, commit_hash,
              session_id, task_id, agent_id, source, event_type, command_name,
              tool_name, cwd, exit_code, started_at, finished_at, redaction_status,
              retention_policy, raw_artifact_id, raw_payload, metadata, created_at)
         VALUES
             (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
              ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
        params![
            event.repo_id.as_deref(),
            event.workspace_path_hash.as_deref(),
            event.git_branch.as_deref(),
            event.worktree_name.as_deref(),
            event.commit_hash.as_deref(),
            &event.session_id,
            event.task_id.as_deref(),
            event.agent_id.as_deref(),
            &event.source,
            &event.event_type,
            event.command_name.as_deref(),
            event.tool_name.as_deref(),
            event.cwd.as_deref(),
            event.exit_code,
            &started_at,
            event.finished_at.as_deref(),
            event
                .redaction_status
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            event
                .retention_policy
                .clone()
                .unwrap_or_else(|| "default".to_string()),
            event.raw_artifact_id.as_deref(),
            event.raw_payload.as_deref(),
            &metadata,
            &created_at,
        ],
    )?;

    get_context_event(conn, conn.last_insert_rowid())
}

pub fn get_context_event(conn: &Connection, id: i64) -> Result<ContextEvent> {
    conn.query_row(
        "SELECT
             id, repo_id, workspace_path_hash, git_branch, worktree_name,
             commit_hash, session_id, task_id, agent_id, source, event_type,
             command_name, tool_name, cwd, exit_code, started_at, finished_at,
             redaction_status, retention_policy, raw_artifact_id, raw_payload,
             metadata, created_at
         FROM context_events
         WHERE id = ?1",
        params![id],
        map_context_event,
    )
    .map_err(EngramError::from)
}

pub fn list_context_events_for_session(
    conn: &Connection,
    session_id: &str,
    limit: usize,
) -> Result<Vec<ContextEvent>> {
    let mut stmt = conn.prepare(
        "SELECT
             id, repo_id, workspace_path_hash, git_branch, worktree_name,
             commit_hash, session_id, task_id, agent_id, source, event_type,
             command_name, tool_name, cwd, exit_code, started_at, finished_at,
             redaction_status, retention_policy, raw_artifact_id, raw_payload,
             metadata, created_at
         FROM context_events
         WHERE session_id = ?1
         ORDER BY started_at DESC, id DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![session_id, limit as i64], map_context_event)?;
    let mut events = Vec::new();
    for row in rows {
        events.push(row?);
    }
    Ok(events)
}

pub fn create_context_summary(
    conn: &Connection,
    summary: &NewContextSummary,
) -> Result<ContextSummary> {
    if summary.source_event_id <= 0 {
        return Err(EngramError::InvalidInput(
            "context summary requires source_event_id".to_string(),
        ));
    }
    if summary.reducer_name.trim().is_empty() || summary.reducer_version.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "context summary requires reducer_name and reducer_version".to_string(),
        ));
    }
    if summary.summary.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "context summary requires summary".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&summary.confidence) {
        return Err(EngramError::InvalidInput(
            "context summary confidence must be between 0.0 and 1.0".to_string(),
        ));
    }

    let created_at = now_rfc3339();
    let structured_facts = attach_summary_estimate_metadata(
        &json_object_or_default(&summary.structured_facts),
        summary.tokens_raw_est,
        summary.tokens_compact_est,
        summary.token_estimate_metadata.clone(),
    );
    let structured_facts = serde_json::to_string(&structured_facts)?;
    let warnings = serde_json::to_string(&json_array_or_default(&summary.warnings))?;

    conn.execute(
        "INSERT INTO context_summaries
             (source_event_id, source_artifact_id, reducer_name, reducer_version,
              lossy, confidence, summary, structured_facts, warnings,
              tokens_raw_est, tokens_compact_est, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            summary.source_event_id,
            summary.source_artifact_id.as_deref(),
            &summary.reducer_name,
            &summary.reducer_version,
            summary.lossy as i32,
            summary.confidence,
            &summary.summary,
            &structured_facts,
            &warnings,
            summary.tokens_raw_est,
            summary.tokens_compact_est,
            &created_at,
        ],
    )?;

    get_context_summary(conn, conn.last_insert_rowid())
}

pub fn get_context_summary(conn: &Connection, id: i64) -> Result<ContextSummary> {
    conn.query_row(
        "SELECT
             id, source_event_id, source_artifact_id, reducer_name, reducer_version,
             lossy, confidence, summary, structured_facts, warnings,
             tokens_raw_est, tokens_compact_est, created_at
         FROM context_summaries
         WHERE id = ?1",
        params![id],
        map_context_summary,
    )
    .map_err(EngramError::from)
}

pub fn latest_context_summary_for_event(
    conn: &Connection,
    event_id: i64,
) -> Result<Option<ContextSummary>> {
    conn.query_row(
        "SELECT
             id, source_event_id, source_artifact_id, reducer_name, reducer_version,
             lossy, confidence, summary, structured_facts, warnings,
             tokens_raw_est, tokens_compact_est, created_at
         FROM context_summaries
         WHERE source_event_id = ?1
         ORDER BY created_at DESC, id DESC
         LIMIT 1",
        params![event_id],
        map_context_summary,
    )
    .optional()
    .map_err(EngramError::from)
}

pub fn list_context_summaries_for_event(
    conn: &Connection,
    event_id: i64,
) -> Result<Vec<ContextSummary>> {
    let mut stmt = conn.prepare(
        "SELECT
             id, source_event_id, source_artifact_id, reducer_name, reducer_version,
             lossy, confidence, summary, structured_facts, warnings,
             tokens_raw_est, tokens_compact_est, created_at
         FROM context_summaries
         WHERE source_event_id = ?1
         ORDER BY created_at DESC, id DESC",
    )?;
    let rows = stmt.query_map(params![event_id], map_context_summary)?;
    let mut summaries = Vec::new();
    for row in rows {
        summaries.push(row?);
    }
    Ok(summaries)
}

pub fn create_context_artifact(
    conn: &Connection,
    artifact: &NewContextArtifact,
) -> Result<ContextArtifact> {
    validate_context_scope(
        artifact.repo_id.as_deref(),
        artifact.workspace_path_hash.as_deref(),
    )
    .or_else(|scope_error| {
        if artifact.source_event_id.is_some() {
            Ok(())
        } else {
            Err(scope_error)
        }
    })?;
    if artifact.id.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "context artifact requires id".to_string(),
        ));
    }
    if artifact.kind.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "context artifact requires kind".to_string(),
        ));
    }

    let raw_content = if artifact.retain_raw {
        artifact.raw_content.as_deref()
    } else {
        None
    };
    let content_sha256 = artifact
        .content_sha256
        .clone()
        .or_else(|| raw_content.map(sha256_bytes));
    let byte_len = artifact
        .byte_len
        .or_else(|| raw_content.map(|bytes| bytes.len() as i64));
    let created_at = now_rfc3339();
    let metadata = serde_json::to_string(&json_object_or_default(&artifact.metadata))?;

    conn.execute(
        "INSERT INTO context_artifacts
             (id, source_event_id, repo_id, workspace_path_hash, session_id, task_id,
              agent_id, kind, label, uri, media_type, content_sha256, byte_len,
              redaction_status, retention_policy, access_policy, retain_raw,
              raw_content, stale_at, expires_at, metadata, created_at)
         VALUES
             (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14,
              ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)",
        params![
            &artifact.id,
            artifact.source_event_id,
            artifact.repo_id.as_deref(),
            artifact.workspace_path_hash.as_deref(),
            artifact.session_id.as_deref(),
            artifact.task_id.as_deref(),
            artifact.agent_id.as_deref(),
            &artifact.kind,
            artifact.label.as_deref(),
            artifact.uri.as_deref(),
            artifact.media_type.as_deref(),
            content_sha256.as_deref(),
            byte_len,
            artifact
                .redaction_status
                .clone()
                .unwrap_or_else(|| "not_required".to_string()),
            artifact
                .retention_policy
                .clone()
                .unwrap_or_else(|| "pointer_only".to_string()),
            artifact
                .access_policy
                .clone()
                .unwrap_or_else(|| "same_session".to_string()),
            artifact.retain_raw as i32,
            raw_content,
            artifact.stale_at.as_deref(),
            artifact.expires_at.as_deref(),
            &metadata,
            &created_at,
        ],
    )?;

    get_context_artifact(conn, &artifact.id)
}

pub fn get_context_artifact(conn: &Connection, id: &str) -> Result<ContextArtifact> {
    conn.query_row(
        "SELECT
             id, source_event_id, repo_id, workspace_path_hash, session_id, task_id,
             agent_id, kind, label, uri, media_type, content_sha256, byte_len,
             redaction_status, retention_policy, access_policy, retain_raw,
             stale_at, expires_at, metadata, created_at
         FROM context_artifacts
         WHERE id = ?1",
        params![id],
        map_context_artifact,
    )
    .map_err(EngramError::from)
}

pub fn list_context_artifacts_for_event(
    conn: &Connection,
    event_id: i64,
) -> Result<Vec<ContextArtifact>> {
    let mut stmt = conn.prepare(
        "SELECT
             id, source_event_id, repo_id, workspace_path_hash, session_id, task_id,
             agent_id, kind, label, uri, media_type, content_sha256, byte_len,
             redaction_status, retention_policy, access_policy, retain_raw,
             stale_at, expires_at, metadata, created_at
         FROM context_artifacts
         WHERE source_event_id = ?1
         ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map(params![event_id], map_context_artifact)?;
    let mut artifacts = Vec::new();
    for row in rows {
        artifacts.push(row?);
    }
    Ok(artifacts)
}

pub fn retrieve_context_artifact_raw(
    conn: &Connection,
    request: &ArtifactRetrievalRequest,
) -> Result<RetrievedContextArtifact> {
    if request.artifact_id.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "artifact_id is required".to_string(),
        ));
    }

    let artifact = match get_context_artifact(conn, &request.artifact_id) {
        Ok(artifact) => artifact,
        Err(e) => {
            log_artifact_access(
                conn,
                request,
                "denied",
                "artifact_not_found",
                None,
                0,
                false,
            )?;
            return Err(e);
        }
    };

    let (allowed, reason) = artifact_access_decision(&artifact, request);
    if !allowed {
        log_artifact_access(conn, request, "denied", reason, None, 0, false)?;
        return Err(EngramError::Unauthorized(reason.to_string()));
    }

    let raw: Option<Vec<u8>> = conn
        .query_row(
            "SELECT raw_content FROM context_artifacts WHERE id = ?1",
            params![request.artifact_id],
            |row| row.get(0),
        )
        .optional()?;
    let raw = match raw {
        Some(raw) => raw,
        None => {
            log_artifact_access(
                conn,
                request,
                "denied",
                "raw_content_not_retained",
                None,
                0,
                false,
            )?;
            return Err(EngramError::Unauthorized(
                "raw_content_not_retained".to_string(),
            ));
        }
    };

    let max_bytes = request.max_bytes.filter(|max| *max >= 0);
    let truncated = max_bytes.is_some_and(|max| raw.len() as i64 > max);
    let content = if let Some(max) = max_bytes {
        raw.into_iter().take(max as usize).collect::<Vec<_>>()
    } else {
        raw
    };
    let returned_bytes = content.len() as i64;
    log_artifact_access(
        conn,
        request,
        "allowed",
        "raw_artifact_retrieval",
        max_bytes,
        returned_bytes,
        truncated,
    )?;

    Ok(RetrievedContextArtifact {
        artifact,
        content,
        returned_bytes,
        truncated,
    })
}

pub fn map_context_event(row: &Row<'_>) -> rusqlite::Result<ContextEvent> {
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
        started_at: row.get(15)?,
        finished_at: row.get(16)?,
        redaction_status: row.get(17)?,
        retention_policy: row.get(18)?,
        raw_artifact_id: row.get(19)?,
        raw_payload: row.get(20)?,
        metadata: parse_json_object(&metadata),
        created_at: row.get(22)?,
    })
}

fn map_context_summary(row: &Row<'_>) -> rusqlite::Result<ContextSummary> {
    let structured_facts: String = row.get(8)?;
    let warnings: String = row.get(9)?;
    Ok(ContextSummary {
        id: row.get(0)?,
        source_event_id: row.get(1)?,
        source_artifact_id: row.get(2)?,
        reducer_name: row.get(3)?,
        reducer_version: row.get(4)?,
        lossy: row.get::<_, i64>(5)? != 0,
        confidence: row.get(6)?,
        summary: row.get(7)?,
        structured_facts: parse_json_object(&structured_facts),
        warnings: parse_json_array(&warnings),
        tokens_raw_est: row.get(10)?,
        tokens_compact_est: row.get(11)?,
        created_at: row.get(12)?,
    })
}

fn map_context_artifact(row: &Row<'_>) -> rusqlite::Result<ContextArtifact> {
    let metadata: String = row.get(19)?;
    Ok(ContextArtifact {
        id: row.get(0)?,
        source_event_id: row.get(1)?,
        repo_id: row.get(2)?,
        workspace_path_hash: row.get(3)?,
        session_id: row.get(4)?,
        task_id: row.get(5)?,
        agent_id: row.get(6)?,
        kind: row.get(7)?,
        label: row.get(8)?,
        uri: row.get(9)?,
        media_type: row.get(10)?,
        content_sha256: row.get(11)?,
        byte_len: row.get(12)?,
        redaction_status: row.get(13)?,
        retention_policy: row.get(14)?,
        access_policy: row.get(15)?,
        retain_raw: row.get::<_, i64>(16)? != 0,
        stale_at: row.get(17)?,
        expires_at: row.get(18)?,
        metadata: parse_json_object(&metadata),
        created_at: row.get(20)?,
    })
}

fn artifact_access_decision<'a>(
    artifact: &'a ContextArtifact,
    request: &'a ArtifactRetrievalRequest,
) -> (bool, &'static str) {
    if !artifact.retain_raw {
        return (false, "raw_content_not_retained");
    }
    if is_past(artifact.expires_at.as_deref()) {
        return (false, "artifact_expired");
    }
    if is_past(artifact.stale_at.as_deref()) {
        return (false, "artifact_stale");
    }

    match artifact.access_policy.as_str() {
        "public" => (true, "public"),
        "repo" => same_optional(&artifact.repo_id, &request.repo_id, "repo_scope_mismatch"),
        "same_agent" => same_optional(
            &artifact.agent_id,
            &request.requester_agent_id,
            "agent_scope_mismatch",
        ),
        "same_task" => same_optional(&artifact.task_id, &request.task_id, "task_scope_mismatch"),
        "same_session" => same_optional(
            &artifact.session_id,
            &request.session_id,
            "session_scope_mismatch",
        ),
        _ => (false, "unsupported_access_policy"),
    }
}

fn same_optional(
    artifact_value: &Option<String>,
    request_value: &Option<String>,
    mismatch_reason: &'static str,
) -> (bool, &'static str) {
    match (artifact_value.as_deref(), request_value.as_deref()) {
        (Some(artifact), Some(request)) if artifact == request => (true, "scope_match"),
        _ => (false, mismatch_reason),
    }
}

fn is_past(timestamp: Option<&str>) -> bool {
    timestamp
        .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
        .is_some_and(|time| time.with_timezone(&Utc) < Utc::now())
}

fn log_artifact_access(
    conn: &Connection,
    request: &ArtifactRetrievalRequest,
    access_result: &str,
    reason: &str,
    max_bytes: Option<i64>,
    returned_bytes: i64,
    truncated: bool,
) -> Result<()> {
    conn.execute(
        "INSERT INTO context_artifact_access_log
             (artifact_id, requester_agent_id, session_id, task_id, repo_id,
              workspace_path_hash, access_result, reason, max_bytes,
              returned_bytes, truncated, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            &request.artifact_id,
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
            now_rfc3339(),
        ],
    )?;
    Ok(())
}

fn parse_json_object(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or_else(|_| json!({}))
}

fn parse_json_array(raw: &str) -> Value {
    serde_json::from_str(raw).unwrap_or_else(|_| json!([]))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("sha256:{}", hex::encode(hasher.finalize()))
}
