use rusqlite::{Connection, ErrorCode};
use serde_json::{json, Value};

use crate::error::{EngramError, Result};
use crate::mcp::handlers::agent_writeback_plan::{
    parse_agent_writeback_plan, AgentWritebackPlan, AgentWritebackSourcePlan,
    AGENT_WRITEBACK_ACTION, AGENT_WRITEBACK_KIND, AGENT_WRITEBACK_MODEL_PROFILE,
};
use crate::mcp::handlers::HandlerContext;
use crate::storage::{
    add_dream_candidate_source, create_dream_candidate, create_dream_job, get_dream_candidate,
    get_dream_job, transition_dream_job, DreamCandidateWithSources, NewDreamCandidate,
    NewDreamCandidateSource, NewDreamJob,
};

pub fn memory_agent_writeback(ctx: &HandlerContext, params: Value) -> Value {
    let plan = match parse_agent_writeback_plan(params) {
        Ok(plan) => plan,
        Err(error) => return json!({"error": error.to_string()}),
    };

    if plan.dry_run {
        return dry_run_response(&plan);
    }
    if !plan.confirm {
        return json!({"error": "memory_agent_writeback requires confirm=true unless dry_run=true"});
    }

    ctx.storage
        .with_transaction(|conn| create_pending_agent_writeback(conn, &plan))
        .unwrap_or_else(|error| json!({"error": error.to_string()}))
}

fn dry_run_response(plan: &AgentWritebackPlan) -> Value {
    let sources = plan
        .sources
        .iter()
        .map(source_plan_response)
        .collect::<Vec<_>>();
    json!({
        "status": "dry_run",
        "dry_run": true,
        "canonical_memory_mutated": false,
        "candidate": {
            "candidate": {
                "id": plan.candidate_id,
                "job_id": plan.job_id,
                "workspace": plan.workspace,
                "kind": AGENT_WRITEBACK_KIND,
                "proposed_action": AGENT_WRITEBACK_ACTION,
                "review_state": "pending",
                "confidence": plan.confidence,
                "freshness_state": "current",
                "content_preview": plan.content_preview,
                "proposed_content": plan.proposed_content,
                "reason_codes": plan.reason_codes,
                "policy_explanation": plan.policy_explanation,
                "metadata": plan.metadata,
                "application_result": null,
                "created_at": null,
                "reviewed_at": null,
                "applied_at": null
            },
            "sources": sources
        },
        "sources_count": plan.sources.len()
    })
}

fn source_plan_response(source: &AgentWritebackSourcePlan) -> Value {
    json!({
        "candidate_id": null,
        "source_type": source.source_type,
        "source_id": source.source_id,
        "source_ref": source.source_ref,
        "evidence": source.evidence
    })
}

fn create_pending_agent_writeback(conn: &Connection, plan: &AgentWritebackPlan) -> Result<Value> {
    ensure_candidate_id_available(conn, plan)?;
    ensure_agent_writeback_job(conn, plan)?;
    let candidate = create_dream_candidate(
        conn,
        &NewDreamCandidate {
            id: plan.candidate_id.as_deref(),
            job_id: &plan.job_id,
            workspace: &plan.workspace,
            kind: AGENT_WRITEBACK_KIND,
            proposed_action: AGENT_WRITEBACK_ACTION,
            confidence: plan.confidence,
            freshness_state: "current",
            content_preview: &plan.content_preview,
            proposed_content: Some(&plan.proposed_content),
            reason_codes: &plan.reason_codes,
            policy_explanation: &plan.policy_explanation,
            metadata: &plan.metadata,
        },
    )
    .map_err(|error| clean_candidate_create_error(error, plan.candidate_id.as_deref()))?;
    let candidate_id = candidate.id.clone();
    let mut sources = Vec::with_capacity(plan.sources.len());

    for source in &plan.sources {
        let created_source = add_dream_candidate_source(
            conn,
            &NewDreamCandidateSource {
                candidate_id: &candidate_id,
                source_type: &source.source_type,
                source_id: &source.source_id,
                source_ref: source.source_ref.as_deref(),
                evidence: &source.evidence,
            },
        )?;
        sources.push(created_source);
    }

    complete_agent_writeback_job(conn, plan, &candidate_id, sources.len())?;
    let candidate = DreamCandidateWithSources { candidate, sources };
    Ok(json!({
        "status": "success",
        "dry_run": false,
        "canonical_memory_mutated": false,
        "sources_count": candidate.sources.len(),
        "candidate": candidate
    }))
}

fn ensure_agent_writeback_job(conn: &Connection, plan: &AgentWritebackPlan) -> Result<()> {
    if let Some(job) = get_dream_job(conn, &plan.job_id)? {
        if job.workspace != plan.workspace {
            return Err(EngramError::Conflict(format!(
                "dream job {} belongs to workspace {}, not {}",
                plan.job_id, job.workspace, plan.workspace
            )));
        }
        let created_by = job.input_summary.get("created_by").and_then(Value::as_str);
        let candidate_kind = job
            .input_summary
            .get("candidate_kind")
            .and_then(Value::as_str);
        let canonical_memory_mutated = job
            .input_summary
            .get("canonical_memory_mutated")
            .and_then(Value::as_bool);
        if job.model_profile != AGENT_WRITEBACK_MODEL_PROFILE
            || created_by != Some("memory_agent_writeback")
            || candidate_kind != Some(AGENT_WRITEBACK_KIND)
            || canonical_memory_mutated != Some(false)
        {
            return Err(EngramError::Conflict(format!(
                "dream job {} is not an agent writeback job",
                plan.job_id
            )));
        }
        if job.status != "pending" {
            return Err(EngramError::Conflict(format!(
                "agent writeback job {} must be pending, found {}",
                plan.job_id, job.status
            )));
        }
        return Ok(());
    }

    let input_summary = json!({
        "created_by": "memory_agent_writeback",
        "workspace": plan.workspace,
        "candidate_kind": AGENT_WRITEBACK_KIND,
        "canonical_memory_mutated": false,
        "evidence_source_count": plan.sources.len()
    });
    create_dream_job(
        conn,
        &NewDreamJob {
            id: Some(&plan.job_id),
            workspace: &plan.workspace,
            instructions: Some("agent writeback pending review"),
            model_profile: Some(AGENT_WRITEBACK_MODEL_PROFILE),
            input_summary: &input_summary,
        },
    )?;
    Ok(())
}

fn ensure_candidate_id_available(conn: &Connection, plan: &AgentWritebackPlan) -> Result<()> {
    let Some(candidate_id) = plan.candidate_id.as_deref() else {
        return Ok(());
    };
    if get_dream_candidate(conn, candidate_id)?.is_some() {
        Err(EngramError::Conflict(format!(
            "dream candidate already exists: {candidate_id}"
        )))
    } else {
        Ok(())
    }
}

fn complete_agent_writeback_job(
    conn: &Connection,
    plan: &AgentWritebackPlan,
    candidate_id: &str,
    sources_created: usize,
) -> Result<()> {
    let output_summary = json!({
        "created_by": "memory_agent_writeback",
        "candidate_id": candidate_id,
        "candidate_kind": AGENT_WRITEBACK_KIND,
        "canonical_memory_mutated": false,
        "sources_created": sources_created
    });
    transition_dream_job(conn, &plan.job_id, "running", None, None)?;
    transition_dream_job(conn, &plan.job_id, "completed", Some(&output_summary), None)?;
    Ok(())
}

fn clean_candidate_create_error(error: EngramError, candidate_id: Option<&str>) -> EngramError {
    match error {
        EngramError::Database(database_error) if is_constraint_violation(&database_error) => {
            let detail = candidate_id
                .map(|id| format!("dream candidate already exists: {id}"))
                .unwrap_or_else(|| {
                    "dream candidate already exists or violates storage constraints".to_string()
                });
            EngramError::Conflict(detail)
        }
        other => other,
    }
}

fn is_constraint_violation(error: &rusqlite::Error) -> bool {
    matches!(
        error,
        rusqlite::Error::SqliteFailure(sql_error, _)
            if sql_error.code == ErrorCode::ConstraintViolation
    )
}
