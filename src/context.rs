//! Operational Context search, bundle assembly, artifact types, and internal
//! context-efficiency audit metadata.

pub mod metrics;

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Duration, Utc};
use rusqlite::{params_from_iter, types::ToSql, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::context::metrics::{
    estimate_bytes_tokens, estimate_text_tokens, summary_estimate_metadata,
    ContextEfficiencyMetrics,
};
use crate::error::{EngramError, Result};

pub type ArtifactRedactionStatus = String;
pub type ArtifactRetentionPolicy = String;
pub type ArtifactAccessPolicy = String;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NewContextEvent {
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
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub redaction_status: Option<String>,
    pub retention_policy: Option<String>,
    pub raw_artifact_id: Option<String>,
    pub raw_payload: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub exit_code: Option<i64>,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub redaction_status: String,
    pub retention_policy: String,
    pub raw_artifact_id: Option<String>,
    #[serde(skip_serializing)]
    pub raw_payload: Option<String>,
    #[serde(default)]
    pub metadata: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NewContextSummary {
    pub source_event_id: i64,
    pub source_artifact_id: Option<String>,
    pub reducer_name: String,
    pub reducer_version: String,
    pub lossy: bool,
    pub confidence: f64,
    pub summary: String,
    #[serde(default)]
    pub structured_facts: Value,
    #[serde(default)]
    pub warnings: Value,
    pub tokens_raw_est: Option<i64>,
    pub tokens_compact_est: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_estimate_metadata: Option<metrics::TokenEstimateMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSummary {
    pub id: i64,
    pub source_event_id: i64,
    pub source_artifact_id: Option<String>,
    pub reducer_name: String,
    pub reducer_version: String,
    pub lossy: bool,
    pub confidence: f64,
    pub summary: String,
    #[serde(default)]
    pub structured_facts: Value,
    #[serde(default)]
    pub warnings: Value,
    pub tokens_raw_est: Option<i64>,
    pub tokens_compact_est: Option<i64>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NewContextArtifact {
    pub id: String,
    pub source_event_id: Option<i64>,
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub agent_id: Option<String>,
    pub kind: String,
    pub label: Option<String>,
    pub uri: Option<String>,
    pub media_type: Option<String>,
    pub content_sha256: Option<String>,
    pub byte_len: Option<i64>,
    pub redaction_status: Option<ArtifactRedactionStatus>,
    pub retention_policy: Option<ArtifactRetentionPolicy>,
    pub access_policy: Option<ArtifactAccessPolicy>,
    pub retain_raw: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_content: Option<Vec<u8>>,
    pub stale_at: Option<String>,
    pub expires_at: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextArtifact {
    pub id: String,
    pub source_event_id: Option<i64>,
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub agent_id: Option<String>,
    pub kind: String,
    pub label: Option<String>,
    pub uri: Option<String>,
    pub media_type: Option<String>,
    pub content_sha256: Option<String>,
    pub byte_len: Option<i64>,
    pub redaction_status: ArtifactRedactionStatus,
    pub retention_policy: ArtifactRetentionPolicy,
    pub access_policy: ArtifactAccessPolicy,
    pub retain_raw: bool,
    pub stale_at: Option<String>,
    pub expires_at: Option<String>,
    #[serde(default)]
    pub metadata: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArtifactRetrievalRequest {
    pub artifact_id: String,
    pub requester_agent_id: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub max_bytes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievedContextArtifact {
    pub artifact: ContextArtifact,
    pub content: Vec<u8>,
    pub returned_bytes: i64,
    pub truncated: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ContextSearchRequest {
    pub query: Option<String>,
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub workspace: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub event_type: Option<String>,
    #[serde(default)]
    pub event_types: Vec<String>,
    #[serde(default)]
    pub event_type_filters: Vec<String>,
    #[serde(default)]
    pub failure_only: bool,
    pub max_results: Option<usize>,
    #[serde(default)]
    pub include_artifact_pointers: bool,
    pub current_git_branch: Option<String>,
    pub current_commit_hash: Option<String>,
    pub stale_after_days: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextSearchResponse {
    pub query: Option<String>,
    pub count: usize,
    pub results: Vec<ContextSearchResult>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextSearchResult {
    pub event: ContextEventView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ContextSummaryView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifact_pointers: Vec<ContextArtifactPointer>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub staleness_warnings: Vec<String>,
    pub provenance: ContextProvenance,
    #[serde(skip_serializing)]
    pub internal: ContextResultInternal,
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
    pub raw_artifact_id: Option<String>,
    #[serde(default)]
    pub metadata: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextSummaryView {
    pub id: i64,
    pub source_artifact_id: Option<String>,
    pub reducer_name: String,
    pub reducer_version: String,
    pub lossy: bool,
    pub confidence: f64,
    pub summary: String,
    #[serde(default)]
    pub structured_facts: Value,
    #[serde(default)]
    pub warnings: Value,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextArtifactPointer {
    pub id: String,
    pub kind: String,
    pub label: Option<String>,
    pub uri: Option<String>,
    pub media_type: Option<String>,
    pub byte_len: Option<i64>,
    pub redaction_status: ArtifactRedactionStatus,
    pub retention_policy: ArtifactRetentionPolicy,
    pub access_policy: ArtifactAccessPolicy,
    pub stale_at: Option<String>,
    pub expires_at: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextProvenance {
    pub event_id: i64,
    pub summary_id: Option<i64>,
    pub artifact_ids: Vec<String>,
    pub source: String,
    pub session_id: String,
    pub task_id: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Default)]
pub struct ContextResultInternal {
    pub raw_payload: Option<String>,
    pub summary_tokens_raw_est: Option<i64>,
    pub summary_tokens_compact_est: Option<i64>,
    pub summary_token_metadata: Option<metrics::TokenEstimateMetadata>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ContextBundleRequest {
    pub query: Option<String>,
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub workspace: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
    pub max_results: Option<usize>,
    pub section_limit: Option<usize>,
    #[serde(default)]
    pub include_artifact_pointers: bool,
    pub current_git_branch: Option<String>,
    pub current_commit_hash: Option<String>,
    pub stale_after_days: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextBundle {
    pub query: Option<String>,
    pub scope: ContextScope,
    pub sections: BTreeMap<String, Vec<ContextBundleItem>>,
    pub warnings: Vec<String>,
    pub provenance: Vec<ContextProvenance>,
    pub audit: ContextBundleAudit,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextScope {
    pub repo_id: Option<String>,
    pub workspace_path_hash: Option<String>,
    pub session_id: Option<String>,
    pub task_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextBundleItem {
    pub title: String,
    pub detail: String,
    pub event_id: i64,
    pub summary_id: Option<i64>,
    pub started_at: String,
    pub source: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContextBundleAudit {
    pub bundle_usage_event_id: Option<i64>,
    pub metrics_recorded: bool,
}

pub fn search_context(
    conn: &Connection,
    request: &ContextSearchRequest,
) -> Result<ContextSearchResponse> {
    let max_results = request.max_results.unwrap_or(25).clamp(1, 200);
    let events = query_context_events(conn, request, max_results)?;
    let mut results = Vec::with_capacity(events.len());

    for event in events {
        let summary = crate::storage::operational_context::latest_context_summary_for_event(
            conn, event.id,
        )?;
        let artifact_pointers = if request.include_artifact_pointers {
            crate::storage::operational_context::list_context_artifacts_for_event(conn, event.id)?
                .into_iter()
                .map(ContextArtifactPointer::from)
                .collect()
        } else {
            Vec::new()
        };
        results.push(build_search_result(
            event,
            summary,
            artifact_pointers,
            request,
        ));
    }

    Ok(ContextSearchResponse {
        query: request.query.clone(),
        count: results.len(),
        results,
        warnings: Vec::new(),
    })
}

pub fn build_context_bundle(
    conn: &Connection,
    request: &ContextBundleRequest,
) -> Result<ContextBundle> {
    let section_limit = request.section_limit.unwrap_or(12).clamp(1, 50);
    let search_request = ContextSearchRequest {
        query: request.query.clone(),
        repo_id: request.repo_id.clone(),
        workspace_path_hash: request.workspace_path_hash.clone(),
        workspace: request.workspace.clone(),
        session_id: request.session_id.clone(),
        task_id: request.task_id.clone(),
        max_results: Some(request.max_results.unwrap_or(80).clamp(1, 200)),
        include_artifact_pointers: request.include_artifact_pointers,
        current_git_branch: request.current_git_branch.clone(),
        current_commit_hash: request.current_commit_hash.clone(),
        stale_after_days: request.stale_after_days,
        ..ContextSearchRequest::default()
    };
    let search = search_context(conn, &search_request)?;
    let mut sections: BTreeMap<String, Vec<ContextBundleItem>> = BTreeMap::new();
    let mut warnings = search.warnings.clone();
    let mut provenance = Vec::new();

    let mut failures = Vec::new();
    let mut blockers = Vec::new();
    let mut decisions = Vec::new();
    let mut commands = Vec::new();
    let mut files = Vec::new();
    let mut artifacts = Vec::new();
    let mut recent = Vec::new();

    for result in &search.results {
        warnings.extend(result.staleness_warnings.clone());
        provenance.push(result.provenance.clone());
        let item = bundle_item_from_result(result);
        if is_failure_event(&result.event) {
            failures.push(item.clone());
        }
        if is_blocker_result(result) {
            blockers.push(item.clone());
        }
        if is_decision_result(result) {
            decisions.push(item.clone());
        }
        if result.event.command_name.is_some() || result.event.tool_name.is_some() {
            commands.push(item.clone());
        }
        for file in files_from_metadata(&result.event.metadata) {
            files.push(ContextBundleItem {
                title: format!("File context: {file}"),
                detail: result
                    .summary
                    .as_ref()
                    .map(|summary| summary.summary.clone())
                    .unwrap_or_else(|| result.event.event_type.clone()),
                event_id: result.event.id,
                summary_id: result.summary.as_ref().map(|summary| summary.id),
                started_at: result.event.started_at.clone(),
                source: result.event.source.clone(),
                warnings: result.staleness_warnings.clone(),
            });
        }
        for artifact in &result.artifact_pointers {
            artifacts.push(ContextBundleItem {
                title: format!("Artifact pointer: {}", artifact.id),
                detail: artifact
                    .label
                    .clone()
                    .or_else(|| artifact.uri.clone())
                    .unwrap_or_else(|| artifact.kind.clone()),
                event_id: result.event.id,
                summary_id: result.summary.as_ref().map(|summary| summary.id),
                started_at: result.event.started_at.clone(),
                source: result.event.source.clone(),
                warnings: result.staleness_warnings.clone(),
            });
        }
        recent.push(item);
    }

    insert_limited_section(&mut sections, "failures", failures, section_limit);
    insert_limited_section(&mut sections, "unresolved_blockers", blockers, section_limit);
    insert_limited_section(&mut sections, "decisions", decisions, section_limit);
    insert_limited_section(
        &mut sections,
        "commands_already_run",
        commands,
        section_limit,
    );
    insert_limited_section(
        &mut sections,
        "inspected_or_touched_files",
        files,
        section_limit,
    );
    insert_limited_section(&mut sections, "artifact_pointers", artifacts, section_limit);
    insert_limited_section(&mut sections, "recent_context", recent, section_limit);

    let metrics = compute_bundle_metrics(&search.results);
    let audit_event_id = audit_bundle_usage(conn, request, &metrics, search.results.len())?;

    warnings.sort();
    warnings.dedup();

    Ok(ContextBundle {
        query: request.query.clone(),
        scope: ContextScope {
            repo_id: request.repo_id.clone(),
            workspace_path_hash: request
                .workspace_path_hash
                .clone()
                .or_else(|| request.workspace.clone()),
            session_id: request.session_id.clone(),
            task_id: request.task_id.clone(),
        },
        sections,
        warnings,
        provenance,
        audit: ContextBundleAudit {
            bundle_usage_event_id: audit_event_id,
            metrics_recorded: audit_event_id.is_some(),
        },
    })
}

fn query_context_events(
    conn: &Connection,
    request: &ContextSearchRequest,
    max_results: usize,
) -> Result<Vec<ContextEvent>> {
    let mut clauses = Vec::new();
    let mut sql_params: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(repo_id) = non_empty(request.repo_id.as_deref()) {
        clauses.push("e.repo_id = ?".to_string());
        sql_params.push(Box::new(repo_id.to_string()));
    }
    if let Some(workspace) = request.workspace_scope() {
        clauses.push("e.workspace_path_hash = ?".to_string());
        sql_params.push(Box::new(workspace.to_string()));
    }
    if let Some(session_id) = non_empty(request.session_id.as_deref()) {
        clauses.push("e.session_id = ?".to_string());
        sql_params.push(Box::new(session_id.to_string()));
    }
    if let Some(task_id) = non_empty(request.task_id.as_deref()) {
        clauses.push("e.task_id = ?".to_string());
        sql_params.push(Box::new(task_id.to_string()));
    }

    let event_types = request.normalized_event_types();
    if !event_types.is_empty() {
        let placeholders = std::iter::repeat("?")
            .take(event_types.len())
            .collect::<Vec<_>>()
            .join(",");
        clauses.push(format!("e.event_type IN ({placeholders})"));
        for event_type in event_types {
            sql_params.push(Box::new(event_type));
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

    if let Some(query) = non_empty(request.query.as_deref()) {
        let like = format!("%{query}%");
        clauses.push(
            "(e.source LIKE ?
              OR e.event_type LIKE ?
              OR e.command_name LIKE ?
              OR e.tool_name LIKE ?
              OR e.cwd LIKE ?
              OR e.metadata LIKE ?
              OR EXISTS (
                    SELECT 1 FROM context_summaries s
                    WHERE s.source_event_id = e.id
                      AND (
                          s.summary LIKE ?
                          OR s.structured_facts LIKE ?
                          OR s.warnings LIKE ?
                          OR s.reducer_name LIKE ?
                      )
                ))"
                .to_string(),
        );
        for _ in 0..10 {
            sql_params.push(Box::new(like.clone()));
        }
    }

    let where_sql = if clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", clauses.join(" AND "))
    };
    let sql = format!(
        "SELECT
             e.id, e.repo_id, e.workspace_path_hash, e.git_branch, e.worktree_name,
             e.commit_hash, e.session_id, e.task_id, e.agent_id, e.source,
             e.event_type, e.command_name, e.tool_name, e.cwd, e.exit_code,
             e.started_at, e.finished_at, e.redaction_status, e.retention_policy,
             e.raw_artifact_id, e.raw_payload, e.metadata, e.created_at
         FROM context_events e
         {where_sql}
         ORDER BY e.started_at DESC, e.id DESC
         LIMIT ?"
    );
    sql_params.push(Box::new(max_results as i64));

    let mut stmt = conn.prepare(&sql)?;
    let params = params_from_iter(sql_params.iter().map(|param| param.as_ref()));
    let rows = stmt.query_map(params, crate::storage::operational_context::map_context_event)?;
    let mut events = Vec::new();
    for row in rows {
        events.push(row?);
    }
    Ok(events)
}

fn build_search_result(
    event: ContextEvent,
    summary: Option<ContextSummary>,
    artifact_pointers: Vec<ContextArtifactPointer>,
    request: &ContextSearchRequest,
) -> ContextSearchResult {
    let staleness_warnings = staleness_warnings(&event, request);
    let summary_id = summary.as_ref().map(|summary| summary.id);
    let summary_view = summary.as_ref().map(ContextSummaryView::from);
    let artifact_ids = artifact_pointers
        .iter()
        .map(|artifact| artifact.id.clone())
        .collect::<Vec<_>>();
    let internal = ContextResultInternal {
        raw_payload: event.raw_payload.clone(),
        summary_tokens_raw_est: summary.as_ref().and_then(|summary| summary.tokens_raw_est),
        summary_tokens_compact_est: summary.as_ref().and_then(|summary| summary.tokens_compact_est),
        summary_token_metadata: summary
            .as_ref()
            .and_then(|summary| summary_estimate_metadata(&summary.structured_facts)),
    };
    let mut event_view = ContextEventView::from(event);
    if !request.include_artifact_pointers {
        event_view.raw_artifact_id = None;
    }

    ContextSearchResult {
        provenance: ContextProvenance {
            event_id: event_view.id,
            summary_id,
            artifact_ids,
            source: event_view.source.clone(),
            session_id: event_view.session_id.clone(),
            task_id: event_view.task_id.clone(),
            created_at: event_view.created_at.clone(),
        },
        event: event_view,
        summary: summary_view,
        artifact_pointers,
        staleness_warnings,
        internal,
    }
}

fn bundle_item_from_result(result: &ContextSearchResult) -> ContextBundleItem {
    let title = if let Some(command) = result.event.command_name.as_deref() {
        format!("Command: {command}")
    } else if let Some(tool) = result.event.tool_name.as_deref() {
        format!("Tool: {tool}")
    } else {
        format!("Event: {}", result.event.event_type)
    };

    let detail = result
        .summary
        .as_ref()
        .map(|summary| summary.summary.clone())
        .or_else(|| summary_from_metadata(&result.event.metadata))
        .unwrap_or_else(|| result.event.event_type.clone());

    ContextBundleItem {
        title,
        detail,
        event_id: result.event.id,
        summary_id: result.summary.as_ref().map(|summary| summary.id),
        started_at: result.event.started_at.clone(),
        source: result.event.source.clone(),
        warnings: result.staleness_warnings.clone(),
    }
}

fn insert_limited_section(
    sections: &mut BTreeMap<String, Vec<ContextBundleItem>>,
    name: &str,
    mut items: Vec<ContextBundleItem>,
    limit: usize,
) {
    items.truncate(limit);
    sections.insert(name.to_string(), items);
}

fn compute_bundle_metrics(results: &[ContextSearchResult]) -> ContextEfficiencyMetrics {
    let mut metrics = ContextEfficiencyMetrics::default();
    let mut command_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut failure_counts: BTreeMap<String, usize> = BTreeMap::new();

    for result in results {
        if let Some(command) = result.event.command_name.as_deref() {
            *command_counts.entry(command.to_string()).or_insert(0) += 1;
            if is_failure_event(&result.event) {
                let key = format!(
                    "{}:{}",
                    command,
                    result
                        .event
                        .exit_code
                        .map(|code| code.to_string())
                        .unwrap_or_else(|| "unknown".to_string())
                );
                *failure_counts.entry(key).or_insert(0) += 1;
            }
        }

        if let Some(summary) = result.summary.as_ref() {
            metrics.summaries_reused += 1;
            metrics.add_known_pair(
                result.internal.summary_tokens_raw_est,
                result.internal.summary_tokens_compact_est,
                result.internal.summary_token_metadata.clone(),
            );
            if result.internal.summary_tokens_compact_est.is_none() {
                if let Some(estimate) =
                    estimate_text_tokens(&summary.summary, "context_bundle.summary_text")
                {
                    metrics.add_compact_estimate(estimate);
                }
            }
            if summary.source_artifact_id.is_some() {
                metrics.artifacts_summarized += 1;
                metrics.raw_artifact_retrievals_avoided += 1;
            }
        } else if let Some(detail) = summary_from_metadata(&result.event.metadata) {
            if let Some(estimate) = estimate_text_tokens(&detail, "context_bundle.event_metadata") {
                metrics.add_compact_estimate(estimate);
            }
        }

        if result.internal.summary_tokens_raw_est.is_none() {
            if let Some(raw_payload) = result.internal.raw_payload.as_deref() {
                if let Some(estimate) =
                    estimate_text_tokens(raw_payload, "context_bundle.raw_payload")
                {
                    metrics.add_raw_estimate(estimate);
                }
            }
        }

        for artifact in &result.artifact_pointers {
            metrics.artifacts_returned_as_pointers += 1;
            if let Some(byte_len) = artifact.byte_len {
                if let Some(estimate) =
                    estimate_bytes_tokens(byte_len, "context_bundle.artifact_byte_len")
                {
                    metrics.add_raw_estimate(estimate);
                }
            }
            if result.summary.is_some() {
                metrics.artifacts_summarized += 1;
                metrics.raw_artifact_retrievals_avoided += 1;
            }
        }
    }

    metrics.repeated_command_context_reused =
        command_counts.values().map(|count| count.saturating_sub(1)).sum();
    metrics.repeated_failures_found =
        failure_counts.values().map(|count| count.saturating_sub(1)).sum();
    metrics
}

fn audit_bundle_usage(
    conn: &Connection,
    request: &ContextBundleRequest,
    metrics: &ContextEfficiencyMetrics,
    result_count: usize,
) -> Result<Option<i64>> {
    let repo_id = request.repo_id.clone();
    let workspace_path_hash = request
        .workspace_path_hash
        .clone()
        .or_else(|| request.workspace.clone());

    if repo_id.as_deref().is_none_or(str::is_empty)
        && workspace_path_hash.as_deref().is_none_or(str::is_empty)
    {
        return Ok(None);
    }

    let now = Utc::now().to_rfc3339();
    let event = NewContextEvent {
        repo_id,
        workspace_path_hash,
        session_id: request
            .session_id
            .clone()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "context-bundle".to_string()),
        task_id: request.task_id.clone(),
        source: "context_build_bundle".to_string(),
        event_type: "bundle_reuse".to_string(),
        started_at: Some(now.clone()),
        finished_at: Some(now),
        metadata: json!({
            "_internal": {
                "context_efficiency": metrics.to_internal_json(),
                "bundle_usage": {
                    "query_present": request.query.as_deref().is_some_and(|query| !query.trim().is_empty()),
                    "result_count": result_count,
                    "include_artifact_pointers": request.include_artifact_pointers
                }
            }
        }),
        ..NewContextEvent::default()
    };

    crate::storage::operational_context::create_context_event(conn, &event).map(|event| Some(event.id))
}

fn staleness_warnings(event: &ContextEvent, request: &ContextSearchRequest) -> Vec<String> {
    let mut warnings = Vec::new();
    if let (Some(current), Some(event_branch)) = (
        non_empty(request.current_git_branch.as_deref()),
        non_empty(event.git_branch.as_deref()),
    ) {
        if current != event_branch {
            warnings.push(format!(
                "branch mismatch: event branch {event_branch}, current branch {current}"
            ));
        }
    }
    if let (Some(current), Some(event_commit)) = (
        non_empty(request.current_commit_hash.as_deref()),
        non_empty(event.commit_hash.as_deref()),
    ) {
        if current != event_commit {
            warnings.push(format!(
                "commit mismatch: event commit {event_commit}, current commit {current}"
            ));
        }
    }
    let stale_after_days = request.stale_after_days.unwrap_or(7).max(1);
    if let Ok(started_at) = DateTime::parse_from_rfc3339(&event.started_at) {
        let cutoff = Utc::now() - Duration::days(stale_after_days);
        if started_at.with_timezone(&Utc) < cutoff {
            warnings.push(format!("event older than {stale_after_days} days"));
        }
    }
    warnings
}

fn is_failure_event(event: &ContextEventView) -> bool {
    event.exit_code.is_some_and(|code| code != 0)
        || event.event_type.to_ascii_lowercase().contains("fail")
        || event.event_type.to_ascii_lowercase().contains("error")
}

fn is_blocker_result(result: &ContextSearchResult) -> bool {
    let haystack = format!(
        "{} {} {}",
        result.event.event_type,
        result
            .summary
            .as_ref()
            .map(|summary| summary.summary.as_str())
            .unwrap_or(""),
        result.event.metadata
    )
    .to_ascii_lowercase();
    haystack.contains("blocker") || haystack.contains("blocked") || haystack.contains("unresolved")
}

fn is_decision_result(result: &ContextSearchResult) -> bool {
    let haystack = format!(
        "{} {} {}",
        result.event.event_type,
        result
            .summary
            .as_ref()
            .map(|summary| summary.summary.as_str())
            .unwrap_or(""),
        result.event.metadata
    )
    .to_ascii_lowercase();
    haystack.contains("decision") || haystack.contains("decided")
}

fn files_from_metadata(metadata: &Value) -> Vec<String> {
    let mut files = BTreeSet::new();
    for key in [
        "files",
        "file_paths",
        "inspected_files",
        "touched_files",
        "changed_files",
    ] {
        collect_string_values(metadata.get(key), &mut files);
    }
    files.into_iter().collect()
}

fn collect_string_values(value: Option<&Value>, out: &mut BTreeSet<String>) {
    match value {
        Some(Value::String(path)) if !path.trim().is_empty() => {
            out.insert(path.clone());
        }
        Some(Value::Array(values)) => {
            for value in values {
                collect_string_values(Some(value), out);
            }
        }
        Some(Value::Object(map)) => {
            for value in map.values() {
                collect_string_values(Some(value), out);
            }
        }
        _ => {}
    }
}

fn summary_from_metadata(metadata: &Value) -> Option<String> {
    for key in ["summary", "message", "reason", "outcome"] {
        if let Some(value) = metadata.get(key).and_then(Value::as_str) {
            if !value.trim().is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

fn scrub_internal_metadata(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut scrubbed = Map::new();
            for (key, value) in map {
                if key == "_internal"
                    || key == "context_efficiency"
                    || key == "efficiency_metadata"
                {
                    continue;
                }
                scrubbed.insert(key.clone(), scrub_internal_metadata(value));
            }
            Value::Object(scrubbed)
        }
        Value::Array(values) => Value::Array(values.iter().map(scrub_internal_metadata).collect()),
        other => other.clone(),
    }
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

impl ContextSearchRequest {
    fn workspace_scope(&self) -> Option<&str> {
        non_empty(self.workspace_path_hash.as_deref())
            .or_else(|| non_empty(self.workspace.as_deref()))
    }

    fn normalized_event_types(&self) -> Vec<String> {
        let mut types = BTreeSet::new();
        if let Some(event_type) = non_empty(self.event_type.as_deref()) {
            types.insert(event_type.to_string());
        }
        for event_type in self
            .event_types
            .iter()
            .chain(self.event_type_filters.iter())
        {
            if let Some(event_type) = non_empty(Some(event_type.as_str())) {
                types.insert(event_type.to_string());
            }
        }
        types.into_iter().collect()
    }
}

impl From<ContextEvent> for ContextEventView {
    fn from(event: ContextEvent) -> Self {
        Self {
            id: event.id,
            repo_id: event.repo_id,
            workspace_path_hash: event.workspace_path_hash,
            git_branch: event.git_branch,
            worktree_name: event.worktree_name,
            commit_hash: event.commit_hash,
            session_id: event.session_id,
            task_id: event.task_id,
            agent_id: event.agent_id,
            source: event.source,
            event_type: event.event_type,
            command_name: event.command_name,
            tool_name: event.tool_name,
            cwd: event.cwd,
            exit_code: event.exit_code,
            started_at: event.started_at,
            finished_at: event.finished_at,
            redaction_status: event.redaction_status,
            retention_policy: event.retention_policy,
            raw_artifact_id: event.raw_artifact_id,
            metadata: scrub_internal_metadata(&event.metadata),
            created_at: event.created_at,
        }
    }
}

impl From<&ContextSummary> for ContextSummaryView {
    fn from(summary: &ContextSummary) -> Self {
        Self {
            id: summary.id,
            source_artifact_id: summary.source_artifact_id.clone(),
            reducer_name: summary.reducer_name.clone(),
            reducer_version: summary.reducer_version.clone(),
            lossy: summary.lossy,
            confidence: summary.confidence,
            summary: summary.summary.clone(),
            structured_facts: scrub_internal_metadata(&summary.structured_facts),
            warnings: summary.warnings.clone(),
            created_at: summary.created_at.clone(),
        }
    }
}

impl From<ContextArtifact> for ContextArtifactPointer {
    fn from(artifact: ContextArtifact) -> Self {
        Self {
            id: artifact.id,
            kind: artifact.kind,
            label: artifact.label,
            uri: artifact.uri,
            media_type: artifact.media_type,
            byte_len: artifact.byte_len,
            redaction_status: artifact.redaction_status,
            retention_policy: artifact.retention_policy,
            access_policy: artifact.access_policy,
            stale_at: artifact.stale_at,
            expires_at: artifact.expires_at,
            metadata: scrub_internal_metadata(&artifact.metadata),
        }
    }
}

pub(crate) fn json_object_or_default(value: &Value) -> Value {
    if value.is_object() {
        value.clone()
    } else {
        json!({})
    }
}

pub(crate) fn json_array_or_default(value: &Value) -> Value {
    if value.is_array() {
        value.clone()
    } else {
        json!([])
    }
}

pub(crate) fn validate_context_scope(
    repo_id: Option<&str>,
    workspace_path_hash: Option<&str>,
) -> Result<()> {
    if non_empty(repo_id).is_none() && non_empty(workspace_path_hash).is_none() {
        return Err(EngramError::InvalidInput(
            "context event or artifact requires repo_id or workspace_path_hash".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

pub(crate) fn optional_row<T>(result: rusqlite::Result<T>) -> Result<Option<T>> {
    result.optional().map_err(EngramError::from)
}
