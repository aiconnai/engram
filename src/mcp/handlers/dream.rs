//! MCP handlers for the Dream Phase.

use std::collections::{HashMap, HashSet};

use crate::dream::candidates::{run_candidate_generation, DreamCandidateGenerationConfig};
use crate::dream::eval::{run_dream_eval, DreamEvalOptions};
use crate::dream::{run_once_all, DreamConfig};
use crate::error::{EngramError, Result};
use crate::mcp::handlers::HandlerContext;
use crate::storage::queries::{
    create_crossref, create_memory, get_policy_record, record_reinforcement, update_memory,
    update_memory_lifecycle_state, upsert_policy_record, PolicyRecordInput,
};
use crate::storage::{
    get_dream_candidate_with_sources, get_dream_job, list_dream_candidates, list_dream_jobs,
    record_dream_candidate_application, review_dream_candidate, transition_dream_job,
    DreamCandidate, DreamCandidateApplication,
};
use crate::types::{
    CreateCrossRefInput, CreateMemoryInput, EdgeType, LifecycleState, MemoryId, MemoryType,
    UpdateMemoryInput,
};
use rusqlite::params;
use serde_json::{json, Value};

/// Manually trigger a Dream Phase pass across all workspaces.
pub fn dream_run_now(ctx: &HandlerContext, _params: Value) -> Value {
    // For now, use default config. In the future, we could allow overriding
    // consolidation parameters via params.
    let config = DreamConfig::default();

    let report = run_once_all(&ctx.storage, &config);

    json!({
        "status": "success",
        "report": report
    })
}

/// Create a reviewable dream snapshot job and optionally run it immediately.
pub fn dream_create(ctx: &HandlerContext, params: Value) -> Value {
    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();
    let run = params.get("run").and_then(|v| v.as_bool()).unwrap_or(true);

    if run {
        let config = DreamCandidateGenerationConfig {
            workspace,
            job_id: params
                .get("job_id")
                .and_then(|v| v.as_str())
                .map(ToString::to_string),
            instructions: params
                .get("instructions")
                .and_then(|v| v.as_str())
                .map(ToString::to_string),
            max_memories: params
                .get("max_memories")
                .and_then(|v| v.as_u64())
                .unwrap_or(50) as usize,
            max_candidates: params
                .get("max_candidates")
                .and_then(|v| v.as_u64())
                .unwrap_or(25) as usize,
            summary_min_memories: params
                .get("summary_min_memories")
                .and_then(|v| v.as_u64())
                .unwrap_or(2) as usize,
        };
        return match run_candidate_generation(&ctx.storage, &config) {
            Ok(report) => json!({"status": "success", "ran": true, "report": report}),
            Err(e) => json!({"error": e.to_string()}),
        };
    }

    let input_summary = json!({
        "workspace": workspace,
        "created_by": "dream_create",
        "run": false
    });
    let result = ctx.storage.with_connection(|conn| {
        crate::storage::create_dream_job(
            conn,
            &crate::storage::NewDreamJob {
                id: params.get("job_id").and_then(|v| v.as_str()),
                workspace: &workspace,
                instructions: params.get("instructions").and_then(|v| v.as_str()),
                model_profile: Some("deterministic-local-v1"),
                input_summary: &input_summary,
            },
        )
    });
    result
        .map(|job| json!({"status": "success", "ran": false, "job": job}))
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn dream_get(ctx: &HandlerContext, params: Value) -> Value {
    let id = match required_string(&params, "id") {
        Ok(id) => id,
        Err(error) => return error,
    };
    ctx.storage
        .with_connection(|conn| get_dream_job(conn, &id))
        .map(|job| json!({"job": job}))
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn dream_list(ctx: &HandlerContext, params: Value) -> Value {
    let workspace = params.get("workspace").and_then(|v| v.as_str());
    let status = params.get("status").and_then(|v| v.as_str());
    let limit = params.get("limit").and_then(|v| v.as_i64());
    ctx.storage
        .with_connection(|conn| list_dream_jobs(conn, workspace, status, limit))
        .map(|jobs| json!({"jobs": jobs, "count": jobs.len()}))
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn dream_cancel(ctx: &HandlerContext, params: Value) -> Value {
    let id = match required_string(&params, "id") {
        Ok(id) => id,
        Err(error) => return error,
    };
    ctx.storage
        .with_connection(|conn| transition_dream_job(conn, &id, "canceled", None, None))
        .map(|job| json!({"status": "success", "job": job}))
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn dream_archive(ctx: &HandlerContext, params: Value) -> Value {
    let id = match required_string(&params, "id") {
        Ok(id) => id,
        Err(error) => return error,
    };
    ctx.storage
        .with_connection(|conn| transition_dream_job(conn, &id, "archived", None, None))
        .map(|job| json!({"status": "success", "job": job}))
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn dream_candidates_list(ctx: &HandlerContext, params: Value) -> Value {
    let workspace = params.get("workspace").and_then(|v| v.as_str());
    let job_id = params.get("job_id").and_then(|v| v.as_str());
    let review_state = params.get("review_state").and_then(|v| v.as_str());
    let limit = params.get("limit").and_then(|v| v.as_i64());
    ctx.storage
        .with_connection(|conn| list_dream_candidates(conn, workspace, job_id, review_state, limit))
        .map(|candidates| json!({"candidates": candidates, "count": candidates.len()}))
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn dream_candidate_get(ctx: &HandlerContext, params: Value) -> Value {
    let id = match required_string(&params, "id") {
        Ok(id) => id,
        Err(error) => return error,
    };
    ctx.storage
        .with_connection(|conn| get_dream_candidate_with_sources(conn, &id))
        .map(|candidate| json!({"candidate": candidate}))
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn dream_candidate_review(ctx: &HandlerContext, params: Value) -> Value {
    let id = match required_string(&params, "id") {
        Ok(id) => id,
        Err(error) => return error,
    };
    let review_state = match required_string(&params, "review_state") {
        Ok(state) => state,
        Err(error) => return error,
    };
    let edited_content = params.get("edited_content").and_then(|v| v.as_str());
    let metadata_patch = params.get("metadata_patch");

    ctx.storage
        .with_connection(|conn| {
            review_dream_candidate(conn, &id, &review_state, edited_content, metadata_patch)
        })
        .map(|candidate| json!({"status": "success", "candidate": candidate}))
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn dream_candidate_apply(ctx: &HandlerContext, params: Value) -> Value {
    let id = match required_string(&params, "id") {
        Ok(id) => id,
        Err(error) => return error,
    };
    let confirm = params
        .get("confirm")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let dry_run = params
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !confirm && !dry_run {
        return json!({"error": "dream_candidate_apply requires confirm=true unless dry_run=true"});
    }

    let result = if dry_run {
        ctx.storage
            .with_connection(|conn| apply_candidate(conn, &id, true))
    } else {
        ctx.storage
            .with_transaction(|conn| apply_candidate(conn, &id, false))
    };

    result.unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn dream_eval_run(ctx: &HandlerContext, params: Value) -> Value {
    let _ = ctx;
    let fixtures = match params.get("fixtures") {
        Some(Value::Array(values)) => {
            let mut fixtures = Vec::with_capacity(values.len());
            for value in values {
                let Some(fixture) = value.as_str().filter(|fixture| !fixture.trim().is_empty())
                else {
                    return json!({"error": "fixtures must contain non-empty strings"});
                };
                fixtures.push(fixture.to_string());
            }
            Some(fixtures)
        }
        Some(_) => return json!({"error": "fixtures must be an array of strings"}),
        None => None,
    };
    let include_details = params
        .get("include_details")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    run_dream_eval(DreamEvalOptions {
        fixtures,
        include_details,
    })
    .map(|report| json!(report))
    .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

fn apply_candidate(conn: &rusqlite::Connection, id: &str, dry_run: bool) -> Result<Value> {
    let with_sources = get_dream_candidate_with_sources(conn, id)?
        .ok_or_else(|| EngramError::InvalidInput(format!("dream candidate not found: {}", id)))?;
    let candidate = with_sources.candidate;

    if candidate.review_state == "applied" {
        return Ok(json!({
            "status": "already_applied",
            "dry_run": dry_run,
            "candidate": candidate
        }));
    }
    if !matches!(candidate.review_state.as_str(), "accepted" | "edited") {
        return Err(EngramError::Conflict(format!(
            "candidate {} must be accepted or edited before apply; current state is {}",
            id, candidate.review_state
        )));
    }

    let planned = planned_application(&candidate)?;
    if dry_run {
        return Ok(json!({
            "status": "dry_run",
            "candidate_id": candidate.id,
            "planned": planned
        }));
    }

    let application = execute_application(conn, &candidate)?;
    let recorded = record_dream_candidate_application(conn, &candidate.id, &application)?;
    Ok(json!({
        "status": application.status,
        "candidate": recorded,
        "application": application
    }))
}

fn planned_application(candidate: &DreamCandidate) -> Result<Value> {
    let target_ids = planned_target_memory_ids(candidate)?;
    Ok(json!({
        "candidate_id": candidate.id,
        "kind": candidate.kind,
        "proposed_action": candidate.proposed_action,
        "target_memory_ids": target_ids,
        "will_mutate_canonical_memory": candidate.proposed_action != "ignore"
    }))
}

fn execute_application(
    conn: &rusqlite::Connection,
    candidate: &DreamCandidate,
) -> Result<DreamCandidateApplication> {
    let mut canonical_memory_ids = Vec::new();
    let mut lifecycle_changes = json!({});

    match candidate.proposed_action.as_str() {
        "create" => {
            let memory = create_candidate_memory(conn, candidate, None)?;
            canonical_memory_ids.push(memory.id);
        }
        "update" => {
            let target = exactly_one_target(candidate)?;
            let content = required_candidate_content(candidate)?;
            let memory = update_memory(
                conn,
                target,
                &UpdateMemoryInput {
                    content: Some(content),
                    memory_type: None,
                    tags: None,
                    metadata: None,
                    importance: None,
                    scope: None,
                    ttl_seconds: None,
                    event_time: None,
                    trigger_pattern: None,
                    media_url: None,
                },
            )?;
            canonical_memory_ids.push(memory.id);
        }
        "merge" => {
            let targets = merge_targets(candidate)?;
            let memory = create_candidate_memory(conn, candidate, Some(targets.clone()))?;
            for target in &targets {
                create_crossref(
                    conn,
                    &CreateCrossRefInput {
                        from_id: memory.id,
                        to_id: *target,
                        edge_type: EdgeType::DerivedFrom,
                        strength: Some(0.75),
                        source_context: Some(format!("dream_candidate:{}", candidate.id)),
                        pinned: false,
                    },
                )?;
            }
            canonical_memory_ids.push(memory.id);
            lifecycle_changes = json!({"merged_source_memory_ids": targets});
        }
        "supersede" => {
            let target = exactly_one_target(candidate)?;
            let successor_id = candidate
                .metadata
                .get("superseded_by_memory_id")
                .and_then(|v| v.as_i64());
            let successor = if let Some(successor_id) = successor_id {
                successor_id
            } else {
                create_candidate_memory(conn, candidate, Some(vec![target]))?.id
            };
            create_crossref(
                conn,
                &CreateCrossRefInput {
                    from_id: successor,
                    to_id: target,
                    edge_type: EdgeType::Supersedes,
                    strength: Some(1.0),
                    source_context: Some(format!("dream_candidate:{}", candidate.id)),
                    pinned: false,
                },
            )?;
            canonical_memory_ids.push(successor);
            lifecycle_changes = json!({"superseded_memory_id": target});
        }
        "expire" => {
            let targets = one_or_more_targets(candidate)?;
            for target in &targets {
                ensure_expirable_target(conn, candidate, *target)?;
            }
            let mut changed = 0usize;
            for target in &targets {
                update_memory_lifecycle_state(conn, *target, LifecycleState::Archived)?;
                changed += 1;
            }
            if changed != targets.len() {
                return Err(EngramError::Conflict(format!(
                    "expire candidate {} changed {} of {} target memories",
                    candidate.id,
                    changed,
                    targets.len()
                )));
            }
            canonical_memory_ids.extend(targets.iter().copied());
            lifecycle_changes = json!({"archived": changed});
        }
        "promote" => {
            let targets = one_or_more_targets(candidate)?;
            for target in &targets {
                record_reinforcement(conn, *target, 0.10, "dream_candidate_apply")?;
            }
            canonical_memory_ids.extend(targets.iter().copied());
            lifecycle_changes = json!({"policy_reinforced": canonical_memory_ids.len()});
        }
        "demote" => {
            let targets = one_or_more_targets(candidate)?;
            for target in &targets {
                demote_policy(conn, *target)?;
            }
            canonical_memory_ids.extend(targets.iter().copied());
            lifecycle_changes = json!({"policy_demoted": canonical_memory_ids.len()});
        }
        "ignore" => {
            lifecycle_changes = json!({"ignored": true});
        }
        other => {
            return Err(EngramError::InvalidInput(format!(
                "unsupported dream candidate action: {}",
                other
            )));
        }
    }

    Ok(DreamCandidateApplication {
        status: "completed".to_string(),
        canonical_memory_ids,
        lifecycle_changes,
        dry_run: false,
        error: None,
    })
}

fn create_candidate_memory(
    conn: &rusqlite::Connection,
    candidate: &DreamCandidate,
    source_ids: Option<Vec<MemoryId>>,
) -> Result<crate::types::Memory> {
    let mut metadata = HashMap::new();
    metadata.insert("dream_candidate_id".to_string(), json!(candidate.id));
    metadata.insert("dream_job_id".to_string(), json!(candidate.job_id));
    metadata.insert("dream_kind".to_string(), json!(candidate.kind));
    if let Some(source_ids) = source_ids {
        metadata.insert("dream_source_memory_ids".to_string(), json!(source_ids));
    }

    create_memory(
        conn,
        &CreateMemoryInput {
            content: required_candidate_content(candidate)?,
            memory_type: memory_type_for_candidate(candidate),
            workspace: Some(candidate.workspace.clone()),
            tags: vec!["dream-candidate".to_string(), candidate.kind.clone()],
            metadata,
            importance: Some(candidate.confidence as f32),
            defer_embedding: true,
            ..Default::default()
        },
    )
}

fn demote_policy(conn: &rusqlite::Connection, memory_id: MemoryId) -> Result<()> {
    let current = get_policy_record(conn, memory_id)?;
    let (salience, retention, retrieval, reason) = current
        .map(|record| {
            (
                record.salience_score,
                record.retention_score,
                record.retrieval_priority,
                record.policy_reason,
            )
        })
        .unwrap_or((0.5, 0.5, 0.5, "dream_candidate_apply".to_string()));
    upsert_policy_record(
        conn,
        PolicyRecordInput {
            memory_id,
            salience_score: (salience - 0.10).max(0.0),
            retention_score: (retention - 0.10).max(0.0),
            retrieval_priority: (retrieval - 0.10).max(0.0),
            policy_version: "heuristic-v1".to_string(),
            policy_reason: format!("dream_candidate_apply:demote:{}", reason),
        },
    )?;
    Ok(())
}

fn memory_type_for_candidate(candidate: &DreamCandidate) -> MemoryType {
    match candidate.kind.as_str() {
        "summary" => MemoryType::Summary,
        "preference" => MemoryType::Preference,
        "constraint" => MemoryType::Decision,
        "project_state" => MemoryType::Context,
        _ => MemoryType::Note,
    }
}

fn planned_target_memory_ids(candidate: &DreamCandidate) -> Result<Vec<MemoryId>> {
    match candidate.proposed_action.as_str() {
        "create" | "ignore" => Ok(Vec::new()),
        "update" | "supersede" => exactly_one_target(candidate).map(|target| vec![target]),
        "merge" => merge_targets(candidate),
        "expire" | "promote" | "demote" => one_or_more_targets(candidate),
        other => Err(EngramError::InvalidInput(format!(
            "unsupported dream candidate action: {}",
            other
        ))),
    }
}

fn target_memory_ids(candidate: &DreamCandidate) -> Result<Vec<MemoryId>> {
    let ids = candidate
        .metadata
        .get("target_memory_ids")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            EngramError::InvalidInput(format!(
                "candidate {} requires metadata.target_memory_ids",
                candidate.id
            ))
        })?;

    let mut parsed = Vec::with_capacity(ids.len());
    let mut seen = HashSet::with_capacity(ids.len());
    for id in ids {
        let id = if let Some(id) = id.as_i64() {
            id
        } else if let Some(id) = id.as_str().and_then(|s| s.parse::<i64>().ok()) {
            id
        } else {
            return Err(EngramError::InvalidInput(
                "metadata.target_memory_ids must contain integers".to_string(),
            ));
        };
        if id <= 0 {
            return Err(EngramError::InvalidInput(
                "metadata.target_memory_ids must contain positive integers".to_string(),
            ));
        }
        if !seen.insert(id) {
            return Err(EngramError::InvalidInput(
                "metadata.target_memory_ids must not contain duplicate ids".to_string(),
            ));
        }
        parsed.push(id);
    }
    Ok(parsed)
}

fn one_or_more_targets(candidate: &DreamCandidate) -> Result<Vec<MemoryId>> {
    let ids = target_memory_ids(candidate)?;
    if ids.is_empty() {
        Err(EngramError::InvalidInput(format!(
            "{} candidates require at least one target_memory_ids entry",
            candidate.proposed_action
        )))
    } else {
        Ok(ids)
    }
}

fn merge_targets(candidate: &DreamCandidate) -> Result<Vec<MemoryId>> {
    let ids = target_memory_ids(candidate)?;
    if ids.len() < 2 {
        Err(EngramError::InvalidInput(
            "merge candidates require at least two target_memory_ids".to_string(),
        ))
    } else {
        Ok(ids)
    }
}

fn exactly_one_target(candidate: &DreamCandidate) -> Result<MemoryId> {
    let ids = target_memory_ids(candidate)?;
    if ids.len() == 1 {
        Ok(ids[0])
    } else {
        Err(EngramError::InvalidInput(format!(
            "{} candidates require exactly one target_memory_ids entry",
            candidate.proposed_action
        )))
    }
}

fn ensure_expirable_target(
    conn: &rusqlite::Connection,
    candidate: &DreamCandidate,
    target: MemoryId,
) -> Result<()> {
    let expirable_count: i64 = conn.query_row(
        "SELECT COUNT(*)
         FROM memories
         WHERE id = ?1
           AND valid_to IS NULL
           AND COALESCE(lifecycle_state, 'active') != 'archived'",
        params![target],
        |row| row.get(0),
    )?;
    if expirable_count == 1 {
        Ok(())
    } else {
        Err(EngramError::Conflict(format!(
            "expire candidate {} target memory {} is missing, archived, or no longer active",
            candidate.id, target
        )))
    }
}

fn required_candidate_content(candidate: &DreamCandidate) -> Result<String> {
    candidate
        .proposed_content
        .as_ref()
        .filter(|content| !content.trim().is_empty())
        .cloned()
        .ok_or_else(|| {
            EngramError::InvalidInput(format!(
                "{} candidates require proposed_content",
                candidate.proposed_action
            ))
        })
}

fn required_string(params: &Value, key: &str) -> std::result::Result<String, Value> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|value| !value.trim().is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| json!({"error": format!("{} is required", key)}))
}
