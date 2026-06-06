//! Scoped Operational Context search.

use chrono::{DateTime, NaiveDateTime, Utc};
use rusqlite::{params_from_iter, types::Value as SqlValue, Connection, Row};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::Result;

const DEFAULT_MAX_RESULTS: usize = 25;
const MAX_RESULTS: usize = 200;
const DEFAULT_STALE_AFTER_DAYS: i64 = 7;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ContextSearchRequest {
    #[serde(default)]
    pub query: Option<String>,
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
    pub event_type: Option<String>,
    #[serde(default)]
    pub event_types: Vec<String>,
    #[serde(default)]
    pub event_type_filters: Vec<String>,
    #[serde(default)]
    pub failure_only: bool,
    #[serde(default)]
    pub max_results: Option<usize>,
    #[serde(default)]
    pub include_artifact_pointers: bool,
    #[serde(default)]
    pub current_git_branch: Option<String>,
    #[serde(default)]
    pub current_commit_hash: Option<String>,
    #[serde(default)]
    pub stale_after_days: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextSearchResponse {
    pub query: Option<String>,
    pub count: usize,
    pub max_results: usize,
    pub scope: ContextScopeView,
    pub filters: ContextFilterView,
    pub results: Vec<ContextSearchItem>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextScopeView {
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub isolation_applied: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextFilterView {
    pub event_types: Vec<String>,
    pub failure_only: bool,
    pub include_artifact_pointers: bool,
    pub stale_after_days: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextSearchItem {
    pub result_type: String,
    pub relevance_score: f64,
    pub event: ContextEventView,
    pub summary: Option<ContextSummaryView>,
    pub metadata_keys: Vec<String>,
    pub extracted_files: Vec<String>,
    pub artifact_pointers: Vec<ArtifactPointer>,
    pub staleness: Vec<StalenessWarning>,
    pub provenance: ContextProvenance,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextEventView {
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
    pub exit_code: Option<i64>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub redaction_status: String,
    pub retention_policy: String,
    pub metadata: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextSummaryView {
    pub id: i64,
    pub source_event_id: i64,
    pub source_artifact_id: Option<String>,
    pub reducer_name: String,
    pub reducer_version: String,
    pub derived: bool,
    pub lossy: bool,
    pub confidence: f64,
    pub summary: String,
    pub structured_facts: Value,
    pub warnings: Value,
    pub tokens_raw_est: Option<i64>,
    pub tokens_compact_est: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ArtifactPointer {
    pub pointer_type: String,
    pub artifact_id: String,
    pub event_id: i64,
    pub summary_id: Option<i64>,
    pub provenance: ContextProvenance,
}

#[derive(Debug, Clone, Serialize)]
pub struct StalenessWarning {
    pub kind: String,
    pub message: String,
    pub observed: Option<String>,
    pub current: Option<String>,
    pub age_days: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextProvenance {
    pub event_id: i64,
    pub summary_id: Option<i64>,
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub session_id: String,
    pub task_id: Option<String>,
    pub source: String,
    pub started_at: String,
    pub summary_created_at: Option<String>,
    pub reducer_name: Option<String>,
    pub reducer_version: Option<String>,
    pub lossy: Option<bool>,
}

pub fn search_context(
    conn: &Connection,
    request: &ContextSearchRequest,
) -> Result<ContextSearchResponse> {
    let max_results = request
        .max_results
        .unwrap_or(DEFAULT_MAX_RESULTS)
        .clamp(1, MAX_RESULTS);
    let stale_after_days = request
        .stale_after_days
        .unwrap_or(DEFAULT_STALE_AFTER_DAYS)
        .max(1);
    let query = normalized_nonempty(request.query.as_deref());
    let workspace_path_hash = request
        .workspace_path_hash
        .clone()
        .or_else(|| request.workspace.clone());
    let event_types = normalized_event_types(request);

    let mut clauses = Vec::new();
    let mut params = Vec::new();

    if let Some(repo_id) = normalized_nonempty(request.repo_id.as_deref()) {
        clauses.push("e.repo_id = ?".to_string());
        params.push(SqlValue::Text(repo_id));
    }
    if let Some(workspace_hash) = normalized_nonempty(workspace_path_hash.as_deref()) {
        clauses.push("e.workspace_path_hash = ?".to_string());
        params.push(SqlValue::Text(workspace_hash));
    }
    if let Some(session_id) = normalized_nonempty(request.session_id.as_deref()) {
        clauses.push("e.session_id = ?".to_string());
        params.push(SqlValue::Text(session_id));
    }
    if let Some(task_id) = normalized_nonempty(request.task_id.as_deref()) {
        clauses.push("e.task_id = ?".to_string());
        params.push(SqlValue::Text(task_id));
    }
    if !event_types.is_empty() {
        let placeholders = vec!["?"; event_types.len()].join(", ");
        clauses.push(format!("e.event_type IN ({placeholders})"));
        for event_type in &event_types {
            params.push(SqlValue::Text(event_type.clone()));
        }
    }
    if request.failure_only {
        clauses.push(
            "(e.exit_code IS NOT NULL AND e.exit_code <> 0
              OR lower(e.event_type) LIKE '%fail%'
              OR lower(e.event_type) LIKE '%error%')"
                .to_string(),
        );
    }
    if let Some(ref q) = query {
        let like = format!("%{}%", q.to_lowercase());
        let mut query_clause = String::from(
            "(lower(e.source) LIKE ?
              OR lower(e.event_type) LIKE ?
              OR lower(COALESCE(e.command_name, '')) LIKE ?
              OR lower(COALESCE(e.tool_name, '')) LIKE ?
              OR lower(COALESCE(e.cwd, '')) LIKE ?
              OR lower(COALESCE(e.raw_artifact_id, '')) LIKE ?
              OR lower(e.metadata) LIKE ?
              OR lower(COALESCE(s.source_artifact_id, '')) LIKE ?
              OR lower(COALESCE(s.summary, '')) LIKE ?
              OR lower(COALESCE(s.structured_facts, '')) LIKE ?
              OR lower(COALESCE(s.warnings, '')) LIKE ?",
        );
        if query_mentions_failure(q) {
            query_clause.push_str(
                " OR (e.exit_code IS NOT NULL AND e.exit_code <> 0)
                  OR lower(e.event_type) LIKE '%fail%'
                  OR lower(e.event_type) LIKE '%error%'",
            );
        }
        query_clause.push(')');
        clauses.push(query_clause);
        for _ in 0..11 {
            params.push(SqlValue::Text(like.clone()));
        }
    }

    let where_sql = if clauses.is_empty() {
        "1 = 1".to_string()
    } else {
        clauses.join(" AND ")
    };
    let sql = format!(
        "SELECT
             e.id, e.repo_id, e.workspace_path_hash, e.git_branch, e.worktree_name,
             e.commit_hash, e.session_id, e.task_id, e.agent_id, e.source, e.event_type,
             e.command_name, e.tool_name, e.cwd, e.exit_code, e.started_at, e.finished_at,
             e.redaction_status, e.retention_policy, e.raw_artifact_id, e.metadata,
             e.created_at,
             s.id, s.source_artifact_id, s.reducer_name, s.reducer_version, s.lossy,
             s.confidence, s.summary, s.structured_facts, s.warnings, s.tokens_raw_est,
             s.tokens_compact_est, s.created_at
         FROM context_events e
         LEFT JOIN context_summaries s ON s.source_event_id = e.id
         WHERE {where_sql}
         ORDER BY e.started_at DESC, e.id DESC, s.created_at DESC
         LIMIT ?"
    );
    params.push(SqlValue::Integer(max_results as i64));

    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query(params_from_iter(params))?;
    let mut results = Vec::new();
    while let Some(row) = rows.next()? {
        results.push(row_to_search_item(
            row,
            query.as_deref(),
            request.include_artifact_pointers,
            request.current_git_branch.as_deref(),
            request.current_commit_hash.as_deref(),
            stale_after_days,
        )?);
    }

    Ok(ContextSearchResponse {
        query,
        count: results.len(),
        max_results,
        scope: ContextScopeView {
            repo_id: request.repo_id.clone(),
            workspace_path_hash,
            session_id: request.session_id.clone(),
            task_id: request.task_id.clone(),
            isolation_applied: request.repo_id.is_some()
                || request.workspace_path_hash.is_some()
                || request.workspace.is_some()
                || request.session_id.is_some()
                || request.task_id.is_some(),
        },
        filters: ContextFilterView {
            event_types,
            failure_only: request.failure_only,
            include_artifact_pointers: request.include_artifact_pointers,
            stale_after_days,
        },
        results,
    })
}

fn row_to_search_item(
    row: &Row<'_>,
    query: Option<&str>,
    include_artifact_pointers: bool,
    current_git_branch: Option<&str>,
    current_commit_hash: Option<&str>,
    stale_after_days: i64,
) -> rusqlite::Result<ContextSearchItem> {
    let metadata_raw: String = row.get(20)?;
    let metadata = parse_json_or(&metadata_raw, json!({}));
    let event = ContextEventView {
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
        metadata,
        created_at: row.get(21)?,
    };
    let raw_artifact_id: Option<String> = row.get(19)?;
    let summary_id: Option<i64> = row.get(22)?;
    let summary = if let Some(id) = summary_id {
        let structured_facts_raw: Option<String> = row.get(29)?;
        let warnings_raw: Option<String> = row.get(30)?;
        let lossy_int: Option<i64> = row.get(26)?;
        Some(ContextSummaryView {
            id,
            source_event_id: event.id,
            source_artifact_id: row.get(23)?,
            reducer_name: row.get::<_, Option<String>>(24)?.unwrap_or_default(),
            reducer_version: row.get::<_, Option<String>>(25)?.unwrap_or_default(),
            derived: true,
            lossy: lossy_int.unwrap_or(1) != 0,
            confidence: row.get::<_, Option<f64>>(27)?.unwrap_or(0.0),
            summary: row.get::<_, Option<String>>(28)?.unwrap_or_default(),
            structured_facts: structured_facts_raw
                .as_deref()
                .map(|raw| parse_json_or(raw, json!({})))
                .unwrap_or_else(|| json!({})),
            warnings: warnings_raw
                .as_deref()
                .map(|raw| parse_json_or(raw, json!([])))
                .unwrap_or_else(|| json!([])),
            tokens_raw_est: row.get(31)?,
            tokens_compact_est: row.get(32)?,
            created_at: row.get::<_, Option<String>>(33)?.unwrap_or_default(),
        })
    } else {
        None
    };

    let metadata_keys = metadata_keys(&event.metadata);
    let extracted_files = extract_files_from_metadata(&event.metadata);
    let artifact_pointers = artifact_pointers(
        &event,
        summary.as_ref(),
        raw_artifact_id,
        include_artifact_pointers,
    );
    let staleness = staleness_warnings(
        &event,
        current_git_branch,
        current_commit_hash,
        stale_after_days,
    );
    let provenance = provenance_for(&event, summary.as_ref());
    let relevance_score = relevance_score(&event, summary.as_ref(), query);
    let result_type = if summary.is_some() {
        "summary".to_string()
    } else {
        "event".to_string()
    };

    Ok(ContextSearchItem {
        result_type,
        relevance_score,
        event,
        summary,
        metadata_keys,
        extracted_files,
        artifact_pointers,
        staleness,
        provenance,
    })
}

fn artifact_pointers(
    event: &ContextEventView,
    summary: Option<&ContextSummaryView>,
    raw_artifact_id: Option<String>,
    include: bool,
) -> Vec<ArtifactPointer> {
    if !include {
        return Vec::new();
    }
    let mut pointers = Vec::new();
    if let Some(artifact_id) = raw_artifact_id {
        pointers.push(ArtifactPointer {
            pointer_type: "event_raw_artifact".to_string(),
            artifact_id,
            event_id: event.id,
            summary_id: None,
            provenance: provenance_for(event, None),
        });
    }
    if let Some(summary) = summary {
        if let Some(artifact_id) = &summary.source_artifact_id {
            pointers.push(ArtifactPointer {
                pointer_type: "summary_source_artifact".to_string(),
                artifact_id: artifact_id.clone(),
                event_id: event.id,
                summary_id: Some(summary.id),
                provenance: provenance_for(event, Some(summary)),
            });
        }
    }
    pointers
}

fn staleness_warnings(
    event: &ContextEventView,
    current_git_branch: Option<&str>,
    current_commit_hash: Option<&str>,
    stale_after_days: i64,
) -> Vec<StalenessWarning> {
    let mut warnings = Vec::new();
    if let (Some(observed), Some(current)) = (event.git_branch.as_deref(), current_git_branch) {
        if !observed.is_empty() && !current.is_empty() && observed != current {
            warnings.push(StalenessWarning {
                kind: "branch_mismatch".to_string(),
                message: "Event was recorded on a different branch.".to_string(),
                observed: Some(observed.to_string()),
                current: Some(current.to_string()),
                age_days: None,
            });
        }
    }
    if let (Some(observed), Some(current)) = (event.commit_hash.as_deref(), current_commit_hash) {
        if !observed.is_empty() && !current.is_empty() && observed != current {
            warnings.push(StalenessWarning {
                kind: "commit_mismatch".to_string(),
                message: "Event was recorded at a different commit.".to_string(),
                observed: Some(observed.to_string()),
                current: Some(current.to_string()),
                age_days: None,
            });
        }
    }
    if let Some(started_at) = parse_time(&event.started_at) {
        let age_days = Utc::now()
            .signed_duration_since(started_at)
            .num_days()
            .max(0);
        if age_days >= stale_after_days {
            warnings.push(StalenessWarning {
                kind: "age".to_string(),
                message: format!("Event is at least {stale_after_days} days old."),
                observed: Some(event.started_at.clone()),
                current: Some(Utc::now().to_rfc3339()),
                age_days: Some(age_days),
            });
        }
    }
    warnings
}

fn provenance_for(
    event: &ContextEventView,
    summary: Option<&ContextSummaryView>,
) -> ContextProvenance {
    ContextProvenance {
        event_id: event.id,
        summary_id: summary.map(|s| s.id),
        repo_id: event.repo_id.clone(),
        workspace_path_hash: event.workspace_path_hash.clone(),
        session_id: event.session_id.clone(),
        task_id: event.task_id.clone(),
        source: event.source.clone(),
        started_at: event.started_at.clone(),
        summary_created_at: summary.map(|s| s.created_at.clone()),
        reducer_name: summary.map(|s| s.reducer_name.clone()),
        reducer_version: summary.map(|s| s.reducer_version.clone()),
        lossy: summary.map(|s| s.lossy),
    }
}

fn relevance_score(
    event: &ContextEventView,
    summary: Option<&ContextSummaryView>,
    query: Option<&str>,
) -> f64 {
    let Some(query) = query else {
        return 0.0;
    };
    let q = query.to_lowercase();
    let mut score = 0.0;
    score += contains_score(&event.event_type, &q, 2.0);
    score += contains_score(&event.source, &q, 1.0);
    score += contains_score(event.command_name.as_deref().unwrap_or(""), &q, 2.0);
    score += contains_score(event.tool_name.as_deref().unwrap_or(""), &q, 2.0);
    score += contains_score(event.cwd.as_deref().unwrap_or(""), &q, 0.5);
    score += contains_score(&event.metadata.to_string(), &q, 1.0);
    if let Some(summary) = summary {
        score += contains_score(&summary.summary, &q, 3.0);
        score += contains_score(&summary.structured_facts.to_string(), &q, 1.5);
        score += summary.confidence.min(1.0);
    }
    score
}

fn contains_score(haystack: &str, needle: &str, weight: f64) -> f64 {
    if haystack.to_lowercase().contains(needle) {
        weight
    } else {
        0.0
    }
}

fn query_mentions_failure(query: &str) -> bool {
    contains_score(query, "fail", 1.0) > 0.0
        || contains_score(query, "error", 1.0) > 0.0
        || contains_score(query, "failure", 1.0) > 0.0
}

fn normalized_event_types(request: &ContextSearchRequest) -> Vec<String> {
    let mut event_types = Vec::new();
    if let Some(value) = request.event_type.as_deref() {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            event_types.push(trimmed.to_string());
        }
    }
    for value in request
        .event_types
        .iter()
        .chain(request.event_type_filters.iter())
    {
        let trimmed = value.trim();
        if !trimmed.is_empty() && !event_types.iter().any(|v| v == trimmed) {
            event_types.push(trimmed.to_string());
        }
    }
    event_types
}

fn normalized_nonempty(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn metadata_keys(metadata: &Value) -> Vec<String> {
    match metadata {
        Value::Object(map) => map.keys().cloned().collect(),
        _ => Vec::new(),
    }
}

fn extract_files_from_metadata(metadata: &Value) -> Vec<String> {
    let mut files = Vec::new();
    collect_file_values(None, metadata, &mut files);
    files.sort();
    files.dedup();
    files
}

fn collect_file_values(key: Option<&str>, value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => {
            if key.map(is_file_key).unwrap_or(false) || looks_like_path(s) {
                out.push(s.clone());
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_file_values(key, item, out);
            }
        }
        Value::Object(map) => {
            for (child_key, child_value) in map {
                collect_file_values(Some(child_key), child_value, out);
            }
        }
        _ => {}
    }
}

fn is_file_key(key: &str) -> bool {
    let key = key.to_lowercase();
    key.contains("file") || key.contains("path")
}

fn looks_like_path(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() || value.len() > 512 || value.contains('\n') {
        return false;
    }
    value.contains('/')
        || value.ends_with(".rs")
        || value.ends_with(".md")
        || value.ends_with(".toml")
        || value.ends_with(".json")
        || value.ends_with(".yaml")
        || value.ends_with(".yml")
        || value.ends_with(".ts")
        || value.ends_with(".tsx")
        || value.ends_with(".py")
}

fn parse_json_or(raw: &str, fallback: Value) -> Value {
    serde_json::from_str(raw).unwrap_or(fallback)
}

fn parse_time(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
        .or_else(|| {
            NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
                .map(|dt| dt.and_utc())
                .ok()
        })
}
