use rusqlite::Connection;
use serde_json::{json, Value};

use crate::error::{EngramError, Result};
use crate::mcp::handlers::agent_writeback_plan::{
    dry_run_response, parse_agent_writeback_plan, AgentWritebackPlan, AGENT_WRITEBACK_ACTION,
    AGENT_WRITEBACK_KIND, AGENT_WRITEBACK_MODEL_PROFILE,
};
use crate::mcp::handlers::HandlerContext;
use crate::storage::{
    add_dream_candidate_source, create_dream_candidate, create_dream_job,
    get_dream_candidate_with_sources, get_dream_job, NewDreamCandidate, NewDreamCandidateSource,
    NewDreamJob,
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

fn create_pending_agent_writeback(conn: &Connection, plan: &AgentWritebackPlan) -> Result<Value> {
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
    )?;
    let candidate_id = candidate.id;

    for source in &plan.sources {
        add_dream_candidate_source(
            conn,
            &NewDreamCandidateSource {
                candidate_id: &candidate_id,
                source_type: &source.source_type,
                source_id: &source.source_id,
                source_ref: source.source_ref.as_deref(),
                evidence: &source.evidence,
            },
        )?;
    }

    let candidate = get_dream_candidate_with_sources(conn, &candidate_id)?.ok_or_else(|| {
        EngramError::Storage(format!(
            "agent writeback candidate {candidate_id} was not readable after insert"
        ))
    })?;
    Ok(json!({
        "status": "success",
        "dry_run": false,
        "canonical_memory_mutated": false,
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
