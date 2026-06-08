//! Storage helpers for the dream snapshot review pipeline.
//!
//! Dream jobs and candidates are proposals until an explicit review/apply path
//! records the accepted result. These helpers intentionally do not mutate
//! canonical memories.

use chrono::{DateTime, Utc};
use rusqlite::{params, types::Type, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::error::{EngramError, Result};

const JOB_STATUSES: &[&str] = &[
    "pending",
    "running",
    "completed",
    "failed",
    "canceled",
    "archived",
];
const TERMINAL_JOB_STATUSES: &[&str] = &["completed", "failed", "canceled", "archived"];
const CANDIDATE_KINDS: &[&str] = &[
    "summary",
    "preference",
    "constraint",
    "project_state",
    "stale_fact",
    "contradiction",
    "merge",
    "promotion",
    "decay",
    "temporal_update",
];
const PROPOSED_ACTIONS: &[&str] = &[
    "create",
    "update",
    "merge",
    "supersede",
    "expire",
    "promote",
    "demote",
    "ignore",
];
const REVIEW_STATES: &[&str] = &[
    "pending", "accepted", "edited", "rejected", "applied", "archived",
];
const REVIEW_DECISIONS: &[&str] = &["accepted", "edited", "rejected", "archived"];
const FRESHNESS_STATES: &[&str] = &[
    "current",
    "stale",
    "future_due",
    "expired",
    "conflicted",
    "unknown",
];
const CONTENT_REQUIRED_ACTIONS: &[&str] = &["create", "update", "merge"];

macro_rules! candidate_select {
    ($tail:literal) => {
        concat!(
            "SELECT id, job_id, workspace, kind, proposed_action, review_state,
                    confidence, freshness_state, content_preview, proposed_content,
                    reason_codes, policy_explanation_json, metadata_json,
                    application_result_json, created_at, reviewed_at, applied_at
             FROM dream_candidates ",
            $tail
        )
    };
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DreamJob {
    pub id: String,
    pub workspace: String,
    pub status: String,
    pub instructions: Option<String>,
    pub model_profile: String,
    pub input_summary: serde_json::Value,
    pub output_summary: serde_json::Value,
    pub error: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
}

pub struct NewDreamJob<'a> {
    pub id: Option<&'a str>,
    pub workspace: &'a str,
    pub instructions: Option<&'a str>,
    pub model_profile: Option<&'a str>,
    pub input_summary: &'a serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DreamCandidate {
    pub id: String,
    pub job_id: String,
    pub workspace: String,
    pub kind: String,
    pub proposed_action: String,
    pub review_state: String,
    pub confidence: f64,
    pub freshness_state: String,
    pub content_preview: String,
    pub proposed_content: Option<String>,
    pub reason_codes: serde_json::Value,
    pub policy_explanation: serde_json::Value,
    pub metadata: serde_json::Value,
    pub application_result: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub applied_at: Option<DateTime<Utc>>,
}

pub struct NewDreamCandidate<'a> {
    pub id: Option<&'a str>,
    pub job_id: &'a str,
    pub workspace: &'a str,
    pub kind: &'a str,
    pub proposed_action: &'a str,
    pub confidence: f64,
    pub freshness_state: &'a str,
    pub content_preview: &'a str,
    pub proposed_content: Option<&'a str>,
    pub reason_codes: &'a serde_json::Value,
    pub policy_explanation: &'a serde_json::Value,
    pub metadata: &'a serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DreamCandidateSource {
    pub candidate_id: String,
    pub source_type: String,
    pub source_id: String,
    pub source_ref: Option<String>,
    pub evidence: serde_json::Value,
}

pub struct NewDreamCandidateSource<'a> {
    pub candidate_id: &'a str,
    pub source_type: &'a str,
    pub source_id: &'a str,
    pub source_ref: Option<&'a str>,
    pub evidence: &'a serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DreamCandidateWithSources {
    pub candidate: DreamCandidate,
    pub sources: Vec<DreamCandidateSource>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DreamCandidateApplication {
    pub status: String,
    pub canonical_memory_ids: Vec<i64>,
    pub lifecycle_changes: serde_json::Value,
    pub dry_run: bool,
    pub error: Option<String>,
}

/// Create a dream review job in `pending` status.
pub fn create_dream_job(conn: &Connection, input: &NewDreamJob<'_>) -> Result<DreamJob> {
    validate_non_empty("workspace", input.workspace)?;
    if let Some(id) = input.id {
        validate_non_empty("id", id)?;
    }
    if let Some(profile) = input.model_profile {
        validate_non_empty("model_profile", profile)?;
    }

    let id = input
        .id
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("dream_job:{}", uuid::Uuid::new_v4()));
    let model_profile = input.model_profile.unwrap_or("deterministic-local-v1");
    let input_summary = serde_json::to_string(input.input_summary)?;
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO dream_jobs
            (id, workspace, status, instructions, model_profile,
             input_summary_json, output_summary_json, created_at)
         VALUES
            (?1, ?2, 'pending', ?3, ?4, ?5, '{}', ?6)",
        params![
            id,
            input.workspace,
            input.instructions,
            model_profile,
            input_summary,
            now,
        ],
    )?;

    get_dream_job(conn, &id)?.ok_or_else(|| {
        EngramError::Storage(format!("dream job {} was not readable after insert", id))
    })
}

pub fn get_dream_job(conn: &Connection, id: &str) -> Result<Option<DreamJob>> {
    let mut stmt = conn.prepare(
        "SELECT id, workspace, status, instructions, model_profile,
                input_summary_json, output_summary_json, error_json, created_at,
                started_at, finished_at, archived_at
         FROM dream_jobs
         WHERE id = ?1",
    )?;
    Ok(stmt.query_row(params![id], map_dream_job).optional()?)
}

pub fn list_dream_jobs(
    conn: &Connection,
    workspace: Option<&str>,
    status: Option<&str>,
    limit: Option<i64>,
) -> Result<Vec<DreamJob>> {
    if let Some(status) = status {
        validate_allowed("status", status, JOB_STATUSES)?;
    }
    let limit = limit.unwrap_or(100).clamp(1, 1000);

    let mut sql = String::from(
        "SELECT id, workspace, status, instructions, model_profile,
                input_summary_json, output_summary_json, error_json, created_at,
                started_at, finished_at, archived_at
         FROM dream_jobs",
    );
    let mut clauses = Vec::new();
    if workspace.is_some() {
        clauses.push("workspace = ?1");
    }
    if status.is_some() {
        clauses.push(if workspace.is_some() {
            "status = ?2"
        } else {
            "status = ?1"
        });
    }
    if !clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&clauses.join(" AND "));
    }
    sql.push_str(" ORDER BY created_at DESC, id DESC LIMIT ?");
    let limit_index = 1 + workspace.is_some() as usize + status.is_some() as usize;
    sql.push_str(&limit_index.to_string());

    let mut stmt = conn.prepare(&sql)?;
    let rows = match (workspace, status) {
        (Some(workspace), Some(status)) => {
            stmt.query_map(params![workspace, status, limit], map_dream_job)?
        }
        (Some(workspace), None) => stmt.query_map(params![workspace, limit], map_dream_job)?,
        (None, Some(status)) => stmt.query_map(params![status, limit], map_dream_job)?,
        (None, None) => stmt.query_map(params![limit], map_dream_job)?,
    };
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

/// Move a job through the monotonic lifecycle.
pub fn transition_dream_job(
    conn: &Connection,
    id: &str,
    status: &str,
    output_summary: Option<&serde_json::Value>,
    error: Option<&serde_json::Value>,
) -> Result<DreamJob> {
    validate_allowed("status", status, JOB_STATUSES)?;
    let current = get_dream_job(conn, id)?
        .ok_or_else(|| EngramError::InvalidInput(format!("dream job not found: {}", id)))?;
    validate_job_transition(&current.status, status)?;

    let now = Utc::now().to_rfc3339();
    let output_summary_json = output_summary.map(serde_json::to_string).transpose()?;
    let error_json = error.map(serde_json::to_string).transpose()?;

    conn.execute(
        "UPDATE dream_jobs
         SET status = ?2,
             started_at = CASE WHEN ?2 = 'running' AND started_at IS NULL THEN ?3 ELSE started_at END,
             finished_at = CASE WHEN ?2 IN ('completed', 'failed', 'canceled') THEN ?3 ELSE finished_at END,
             archived_at = CASE WHEN ?2 = 'archived' THEN ?3 ELSE archived_at END,
             output_summary_json = COALESCE(?4, output_summary_json),
             error_json = COALESCE(?5, error_json)
         WHERE id = ?1",
        params![id, status, now, output_summary_json, error_json],
    )?;

    get_dream_job(conn, id)?.ok_or_else(|| {
        EngramError::Storage(format!(
            "dream job {} was not readable after transition",
            id
        ))
    })
}

pub fn create_dream_candidate(
    conn: &Connection,
    input: &NewDreamCandidate<'_>,
) -> Result<DreamCandidate> {
    validate_candidate_input(input)?;

    let id = input
        .id
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("dream_candidate:{}", uuid::Uuid::new_v4()));
    let reason_codes = serde_json::to_string(input.reason_codes)?;
    let policy_explanation = serde_json::to_string(input.policy_explanation)?;
    let metadata = serde_json::to_string(input.metadata)?;
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO dream_candidates
            (id, job_id, workspace, kind, proposed_action, review_state,
             confidence, freshness_state, content_preview, proposed_content,
             reason_codes, policy_explanation_json, metadata_json, created_at)
         VALUES
            (?1, ?2, ?3, ?4, ?5, 'pending', ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            id,
            input.job_id,
            input.workspace,
            input.kind,
            input.proposed_action,
            input.confidence,
            input.freshness_state,
            input.content_preview,
            input.proposed_content,
            reason_codes,
            policy_explanation,
            metadata,
            now,
        ],
    )?;

    get_dream_candidate(conn, &id)?.ok_or_else(|| {
        EngramError::Storage(format!(
            "dream candidate {} was not readable after insert",
            id
        ))
    })
}

pub fn add_dream_candidate_source(
    conn: &Connection,
    source: &NewDreamCandidateSource<'_>,
) -> Result<DreamCandidateSource> {
    validate_non_empty("candidate_id", source.candidate_id)?;
    validate_non_empty("source_type", source.source_type)?;
    validate_non_empty("source_id", source.source_id)?;
    let evidence = serde_json::to_string(source.evidence)?;

    conn.execute(
        "INSERT OR REPLACE INTO dream_candidate_sources
            (candidate_id, source_type, source_id, source_ref, evidence_json)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            source.candidate_id,
            source.source_type,
            source.source_id,
            source.source_ref,
            evidence,
        ],
    )?;

    get_dream_candidate_source(
        conn,
        source.candidate_id,
        source.source_type,
        source.source_id,
    )?
    .ok_or_else(|| {
        EngramError::Storage(format!(
            "dream candidate source {}:{}:{} was not readable after insert",
            source.candidate_id, source.source_type, source.source_id
        ))
    })
}

pub fn get_dream_candidate(conn: &Connection, id: &str) -> Result<Option<DreamCandidate>> {
    let mut stmt = conn.prepare(candidate_select!("WHERE id = ?1"))?;
    Ok(stmt
        .query_row(params![id], map_dream_candidate)
        .optional()?)
}

pub fn get_dream_candidate_with_sources(
    conn: &Connection,
    id: &str,
) -> Result<Option<DreamCandidateWithSources>> {
    let Some(candidate) = get_dream_candidate(conn, id)? else {
        return Ok(None);
    };
    let sources = list_dream_candidate_sources(conn, id)?;
    Ok(Some(DreamCandidateWithSources { candidate, sources }))
}

pub fn list_dream_candidates(
    conn: &Connection,
    workspace: Option<&str>,
    job_id: Option<&str>,
    review_state: Option<&str>,
    limit: Option<i64>,
) -> Result<Vec<DreamCandidate>> {
    if let Some(review_state) = review_state {
        validate_allowed("review_state", review_state, REVIEW_STATES)?;
    }
    let limit = limit.unwrap_or(100).clamp(1, 1000);

    let mut sql = candidate_select!("").to_string();
    let mut clauses = Vec::new();
    if workspace.is_some() {
        clauses.push(format!("workspace = ?{}", clauses.len() + 1));
    }
    if job_id.is_some() {
        clauses.push(format!("job_id = ?{}", clauses.len() + 1));
    }
    if review_state.is_some() {
        clauses.push(format!("review_state = ?{}", clauses.len() + 1));
    }
    if !clauses.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&clauses.join(" AND "));
    }
    sql.push_str(" ORDER BY created_at DESC, id DESC LIMIT ?");
    sql.push_str(&(clauses.len() + 1).to_string());

    let mut stmt = conn.prepare(&sql)?;
    let rows = match (workspace, job_id, review_state) {
        (Some(workspace), Some(job_id), Some(review_state)) => stmt.query_map(
            params![workspace, job_id, review_state, limit],
            map_dream_candidate,
        )?,
        (Some(workspace), Some(job_id), None) => {
            stmt.query_map(params![workspace, job_id, limit], map_dream_candidate)?
        }
        (Some(workspace), None, Some(review_state)) => {
            stmt.query_map(params![workspace, review_state, limit], map_dream_candidate)?
        }
        (Some(workspace), None, None) => {
            stmt.query_map(params![workspace, limit], map_dream_candidate)?
        }
        (None, Some(job_id), Some(review_state)) => {
            stmt.query_map(params![job_id, review_state, limit], map_dream_candidate)?
        }
        (None, Some(job_id), None) => {
            stmt.query_map(params![job_id, limit], map_dream_candidate)?
        }
        (None, None, Some(review_state)) => {
            stmt.query_map(params![review_state, limit], map_dream_candidate)?
        }
        (None, None, None) => stmt.query_map(params![limit], map_dream_candidate)?,
    };
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

pub fn review_dream_candidate(
    conn: &Connection,
    id: &str,
    review_state: &str,
    edited_content: Option<&str>,
    metadata_patch: Option<&serde_json::Value>,
) -> Result<DreamCandidate> {
    validate_allowed("review_state", review_state, REVIEW_DECISIONS)?;
    if matches!(review_state, "edited" | "accepted") {
        if let Some(content) = edited_content {
            validate_non_empty("edited_content", content)?;
        }
    }

    let candidate = get_dream_candidate(conn, id)?
        .ok_or_else(|| EngramError::InvalidInput(format!("dream candidate not found: {}", id)))?;
    if !matches!(
        candidate.review_state.as_str(),
        "pending" | "accepted" | "edited"
    ) {
        return Err(EngramError::Conflict(format!(
            "cannot review candidate {} in state {}",
            id, candidate.review_state
        )));
    }

    let mut metadata = candidate.metadata.clone();
    if let Some(patch) = metadata_patch {
        merge_json_object(&mut metadata, patch);
    }
    let metadata_json = serde_json::to_string(&metadata)?;
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "UPDATE dream_candidates
         SET review_state = ?2,
             proposed_content = COALESCE(?3, proposed_content),
             metadata_json = ?4,
             reviewed_at = ?5
         WHERE id = ?1",
        params![id, review_state, edited_content, metadata_json, now],
    )?;

    get_dream_candidate(conn, id)?.ok_or_else(|| {
        EngramError::Storage(format!(
            "dream candidate {} was not readable after review",
            id
        ))
    })
}

/// Record a successful or failed application result.
///
/// This helper is idempotent: an already-applied candidate is returned as-is.
pub fn record_dream_candidate_application(
    conn: &Connection,
    id: &str,
    application: &DreamCandidateApplication,
) -> Result<DreamCandidate> {
    let candidate = get_dream_candidate(conn, id)?
        .ok_or_else(|| EngramError::InvalidInput(format!("dream candidate not found: {}", id)))?;
    if candidate.review_state == "applied" {
        return Ok(candidate);
    }
    if !matches!(candidate.review_state.as_str(), "accepted" | "edited") {
        return Err(EngramError::Conflict(format!(
            "candidate {} must be accepted or edited before application; current state is {}",
            id, candidate.review_state
        )));
    }
    validate_non_empty("application.status", &application.status)?;

    let result_json = serde_json::to_string(application)?;
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE dream_candidates
         SET review_state = 'applied',
             application_result_json = ?2,
             applied_at = ?3
         WHERE id = ?1",
        params![id, result_json, now],
    )?;

    get_dream_candidate(conn, id)?.ok_or_else(|| {
        EngramError::Storage(format!(
            "dream candidate {} was not readable after application",
            id
        ))
    })
}

pub fn list_dream_candidate_sources(
    conn: &Connection,
    candidate_id: &str,
) -> Result<Vec<DreamCandidateSource>> {
    let mut stmt = conn.prepare(
        "SELECT candidate_id, source_type, source_id, source_ref, evidence_json
         FROM dream_candidate_sources
         WHERE candidate_id = ?1
         ORDER BY source_type ASC, source_id ASC",
    )?;
    let rows = stmt.query_map(params![candidate_id], map_dream_candidate_source)?;
    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

fn get_dream_candidate_source(
    conn: &Connection,
    candidate_id: &str,
    source_type: &str,
    source_id: &str,
) -> Result<Option<DreamCandidateSource>> {
    let mut stmt = conn.prepare(
        "SELECT candidate_id, source_type, source_id, source_ref, evidence_json
         FROM dream_candidate_sources
         WHERE candidate_id = ?1 AND source_type = ?2 AND source_id = ?3",
    )?;
    Ok(stmt
        .query_row(
            params![candidate_id, source_type, source_id],
            map_dream_candidate_source,
        )
        .optional()?)
}

fn validate_candidate_input(input: &NewDreamCandidate<'_>) -> Result<()> {
    validate_non_empty("job_id", input.job_id)?;
    validate_non_empty("workspace", input.workspace)?;
    validate_allowed("kind", input.kind, CANDIDATE_KINDS)?;
    validate_allowed("proposed_action", input.proposed_action, PROPOSED_ACTIONS)?;
    validate_allowed("freshness_state", input.freshness_state, FRESHNESS_STATES)?;
    validate_non_empty("content_preview", input.content_preview)?;
    if !(0.0..=1.0).contains(&input.confidence) {
        return Err(EngramError::InvalidInput(
            "confidence must be between 0.0 and 1.0".to_string(),
        ));
    }
    if CONTENT_REQUIRED_ACTIONS.contains(&input.proposed_action) {
        let has_content = input
            .proposed_content
            .map(|content| !content.trim().is_empty())
            .unwrap_or(false);
        if !has_content {
            return Err(EngramError::InvalidInput(format!(
                "{} candidates require proposed_content",
                input.proposed_action
            )));
        }
    }
    Ok(())
}

fn validate_job_transition(from: &str, to: &str) -> Result<()> {
    if from == to {
        return Ok(());
    }
    if from == "archived" {
        return Err(EngramError::Conflict(
            "archived dream jobs cannot transition".to_string(),
        ));
    }
    if TERMINAL_JOB_STATUSES.contains(&from) {
        if to == "archived" {
            return Ok(());
        }
        return Err(EngramError::Conflict(format!(
            "terminal dream job state {} can only archive",
            from
        )));
    }
    let allowed = match from {
        "pending" => matches!(to, "running" | "canceled" | "failed"),
        "running" => matches!(to, "completed" | "failed" | "canceled"),
        _ => false,
    };
    if allowed {
        Ok(())
    } else {
        Err(EngramError::Conflict(format!(
            "invalid dream job transition {} -> {}",
            from, to
        )))
    }
}

fn validate_allowed(name: &str, value: &str, allowed: &[&str]) -> Result<()> {
    validate_non_empty(name, value)?;
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(EngramError::InvalidInput(format!(
            "{} must be one of {}; got {}",
            name,
            allowed.join(", "),
            value
        )))
    }
}

fn validate_non_empty(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        Err(EngramError::InvalidInput(format!(
            "{} must not be empty",
            name
        )))
    } else {
        Ok(())
    }
}

fn merge_json_object(target: &mut serde_json::Value, patch: &serde_json::Value) {
    let (Some(target), Some(patch)) = (target.as_object_mut(), patch.as_object()) else {
        return;
    };
    for (key, value) in patch {
        target.insert(key.clone(), value.clone());
    }
}

fn map_dream_job(row: &Row<'_>) -> rusqlite::Result<DreamJob> {
    Ok(DreamJob {
        id: row.get(0)?,
        workspace: row.get(1)?,
        status: row.get(2)?,
        instructions: row.get(3)?,
        model_profile: row.get(4)?,
        input_summary: parse_json(row.get(5)?, 5)?,
        output_summary: parse_json(row.get(6)?, 6)?,
        error: parse_optional_json(row.get(7)?, 7)?,
        created_at: parse_datetime(row.get(8)?, 8)?,
        started_at: parse_optional_datetime(row.get(9)?, 9)?,
        finished_at: parse_optional_datetime(row.get(10)?, 10)?,
        archived_at: parse_optional_datetime(row.get(11)?, 11)?,
    })
}

fn map_dream_candidate(row: &Row<'_>) -> rusqlite::Result<DreamCandidate> {
    Ok(DreamCandidate {
        id: row.get(0)?,
        job_id: row.get(1)?,
        workspace: row.get(2)?,
        kind: row.get(3)?,
        proposed_action: row.get(4)?,
        review_state: row.get(5)?,
        confidence: row.get(6)?,
        freshness_state: row.get(7)?,
        content_preview: row.get(8)?,
        proposed_content: row.get(9)?,
        reason_codes: parse_json(row.get(10)?, 10)?,
        policy_explanation: parse_json(row.get(11)?, 11)?,
        metadata: parse_json(row.get(12)?, 12)?,
        application_result: parse_optional_json(row.get(13)?, 13)?,
        created_at: parse_datetime(row.get(14)?, 14)?,
        reviewed_at: parse_optional_datetime(row.get(15)?, 15)?,
        applied_at: parse_optional_datetime(row.get(16)?, 16)?,
    })
}

fn map_dream_candidate_source(row: &Row<'_>) -> rusqlite::Result<DreamCandidateSource> {
    Ok(DreamCandidateSource {
        candidate_id: row.get(0)?,
        source_type: row.get(1)?,
        source_id: row.get(2)?,
        source_ref: row.get(3)?,
        evidence: parse_json(row.get(4)?, 4)?,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::migrations::run_migrations;

    fn test_conn() -> Connection {
        let conn = Connection::open_in_memory().expect("open in-memory db");
        run_migrations(&conn).expect("run migrations");
        conn
    }

    fn create_job(conn: &Connection) -> DreamJob {
        create_dream_job(
            conn,
            &NewDreamJob {
                id: Some("job-1"),
                workspace: "default",
                instructions: Some("Summarize project context."),
                model_profile: None,
                input_summary: &serde_json::json!({"memories": 2}),
            },
        )
        .expect("create dream job")
    }

    fn create_candidate(conn: &Connection) -> DreamCandidate {
        create_job(conn);
        create_dream_candidate(
            conn,
            &NewDreamCandidate {
                id: Some("candidate-1"),
                job_id: "job-1",
                workspace: "default",
                kind: "summary",
                proposed_action: "create",
                confidence: 0.82,
                freshness_state: "current",
                content_preview: "Carry forward the release checklist.",
                proposed_content: Some("Carry forward the release checklist."),
                reason_codes: &serde_json::json!(["durable_project_state"]),
                policy_explanation: &serde_json::json!({"salience": 0.8}),
                metadata: &serde_json::json!({"sensitivity": "normal"}),
            },
        )
        .expect("create candidate")
    }

    #[test]
    fn job_lifecycle_is_monotonic_and_archivable() {
        let conn = test_conn();
        create_job(&conn);

        let running = transition_dream_job(&conn, "job-1", "running", None, None).expect("running");
        assert_eq!(running.status, "running");
        assert!(running.started_at.is_some());

        let completed = transition_dream_job(
            &conn,
            "job-1",
            "completed",
            Some(&serde_json::json!({"candidates": 1})),
            None,
        )
        .expect("completed");
        assert_eq!(completed.status, "completed");
        assert!(completed.finished_at.is_some());

        let invalid = transition_dream_job(&conn, "job-1", "running", None, None);
        assert!(invalid.is_err(), "terminal jobs should not restart");

        let archived =
            transition_dream_job(&conn, "job-1", "archived", None, None).expect("archived");
        assert_eq!(archived.status, "archived");
        assert!(archived.archived_at.is_some());
    }

    #[test]
    fn candidate_round_trips_with_sources() {
        let conn = test_conn();
        create_candidate(&conn);
        add_dream_candidate_source(
            &conn,
            &NewDreamCandidateSource {
                candidate_id: "candidate-1",
                source_type: "memory",
                source_id: "42",
                source_ref: Some("memory:42"),
                evidence: &serde_json::json!({"quote": "release checklist"}),
            },
        )
        .expect("add source");

        let with_sources = get_dream_candidate_with_sources(&conn, "candidate-1")
            .expect("get candidate")
            .expect("candidate exists");
        assert_eq!(with_sources.candidate.kind, "summary");
        assert_eq!(with_sources.sources.len(), 1);
        assert_eq!(with_sources.sources[0].source_type, "memory");
    }

    #[test]
    fn content_required_actions_are_validated_before_insert() {
        let conn = test_conn();
        create_job(&conn);
        let err = create_dream_candidate(
            &conn,
            &NewDreamCandidate {
                id: Some("bad-candidate"),
                job_id: "job-1",
                workspace: "default",
                kind: "summary",
                proposed_action: "create",
                confidence: 0.9,
                freshness_state: "current",
                content_preview: "preview",
                proposed_content: None,
                reason_codes: &serde_json::json!([]),
                policy_explanation: &serde_json::json!({}),
                metadata: &serde_json::json!({}),
            },
        );
        assert!(err.is_err());
    }

    #[test]
    fn review_and_application_are_idempotent() {
        let conn = test_conn();
        create_candidate(&conn);

        let reviewed = review_dream_candidate(
            &conn,
            "candidate-1",
            "accepted",
            None,
            Some(&serde_json::json!({"reviewer": "test"})),
        )
        .expect("review");
        assert_eq!(reviewed.review_state, "accepted");
        assert_eq!(reviewed.metadata["reviewer"], "test");

        let application = DreamCandidateApplication {
            status: "completed".to_string(),
            canonical_memory_ids: vec![101],
            lifecycle_changes: serde_json::json!({}),
            dry_run: false,
            error: None,
        };
        let applied =
            record_dream_candidate_application(&conn, "candidate-1", &application).expect("apply");
        assert_eq!(applied.review_state, "applied");
        assert!(applied.applied_at.is_some());

        let applied_again = record_dream_candidate_application(&conn, "candidate-1", &application)
            .expect("idempotent apply");
        assert_eq!(applied_again.applied_at, applied.applied_at);
        assert_eq!(applied_again.application_result, applied.application_result);
    }

    #[test]
    fn pending_candidate_cannot_be_applied() {
        let conn = test_conn();
        create_candidate(&conn);
        let application = DreamCandidateApplication {
            status: "completed".to_string(),
            canonical_memory_ids: vec![101],
            lifecycle_changes: serde_json::json!({}),
            dry_run: false,
            error: None,
        };
        let result = record_dream_candidate_application(&conn, "candidate-1", &application);
        assert!(result.is_err(), "pending candidate should not apply");
    }
}
