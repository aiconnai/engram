//! Deterministic candidate generation for reviewable dream snapshots.
//!
//! This generator reads canonical evidence and writes review candidates only.
//! It does not create, update, expire, promote, or demote canonical memories.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::{EngramError, Result};
use crate::storage::{
    add_dream_candidate_source, create_dream_candidate, create_dream_job, transition_dream_job,
    NewDreamCandidate, NewDreamCandidateSource, NewDreamJob, Storage,
};

const GENERATOR_VERSION: &str = "deterministic-dream-candidates-v1";
const SUMMARY_SOURCE_LIMIT: usize = 6;
const PREVIEW_LIMIT: usize = 160;

#[derive(Debug, Clone)]
pub struct DreamCandidateGenerationConfig {
    pub workspace: String,
    pub job_id: Option<String>,
    pub instructions: Option<String>,
    pub max_memories: usize,
    pub max_candidates: usize,
    pub summary_min_memories: usize,
}

impl Default for DreamCandidateGenerationConfig {
    fn default() -> Self {
        Self {
            workspace: "default".to_string(),
            job_id: None,
            instructions: Some(
                "Generate reviewable dream snapshot candidates without mutating canonical memory."
                    .to_string(),
            ),
            max_memories: 50,
            max_candidates: 25,
            summary_min_memories: 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DreamCandidateGenerationReport {
    pub job_id: String,
    pub workspace: String,
    pub memories_scanned: usize,
    pub candidates_created: usize,
    pub sources_created: usize,
    pub candidate_ids: Vec<String>,
    pub skipped: Vec<String>,
}

#[derive(Debug, Clone)]
struct MemoryEvidence {
    id: i64,
    content: String,
    memory_type: String,
    importance: f64,
    event_time: Option<DateTime<Utc>>,
    expires_at: Option<DateTime<Utc>>,
    lifecycle_state: String,
    salience_score: Option<f64>,
    retention_score: Option<f64>,
    retrieval_priority: Option<f64>,
    policy_reason: Option<String>,
}

struct CandidateProposal {
    kind: &'static str,
    proposed_action: &'static str,
    confidence: f64,
    freshness_state: &'static str,
    content_preview: String,
    proposed_content: Option<String>,
    reason_codes: Vec<&'static str>,
    policy_explanation: serde_json::Value,
    metadata: serde_json::Value,
    sources: Vec<SourceProposal>,
}

struct SourceProposal {
    source_type: &'static str,
    source_id: String,
    source_ref: Option<String>,
    evidence: serde_json::Value,
}

/// Run deterministic candidate generation for one workspace.
pub fn run_candidate_generation(
    storage: &Storage,
    config: &DreamCandidateGenerationConfig,
) -> Result<DreamCandidateGenerationReport> {
    validate_config(config)?;

    let input_summary = json!({
        "workspace": config.workspace,
        "max_memories": config.max_memories,
        "max_candidates": config.max_candidates,
        "summary_min_memories": config.summary_min_memories,
        "generator_version": GENERATOR_VERSION
    });

    let job = storage.with_connection(|conn| {
        create_dream_job(
            conn,
            &NewDreamJob {
                id: config.job_id.as_deref(),
                workspace: &config.workspace,
                instructions: config.instructions.as_deref(),
                model_profile: Some("deterministic-local-v1"),
                input_summary: &input_summary,
            },
        )
    })?;
    let job_id = job.id.clone();

    if let Err(err) = storage.with_connection(|conn| {
        transition_dream_job(conn, &job_id, "running", None, None)?;
        Ok(())
    }) {
        mark_failed_best_effort(storage, &job_id, &err);
        return Err(err);
    }

    let result = build_and_store_candidates(storage, config, &job_id);
    match result {
        Ok(report) => Ok(report),
        Err(err) => {
            mark_failed_best_effort(storage, &job_id, &err);
            Err(err)
        }
    }
}

fn build_and_store_candidates(
    storage: &Storage,
    config: &DreamCandidateGenerationConfig,
    job_id: &str,
) -> Result<DreamCandidateGenerationReport> {
    let memories = storage.with_connection(|conn| {
        load_memory_evidence(conn, &config.workspace, config.max_memories)
    })?;
    let proposals = build_proposals(&memories, config);

    let mut report = DreamCandidateGenerationReport {
        job_id: job_id.to_string(),
        workspace: config.workspace.clone(),
        memories_scanned: memories.len(),
        candidates_created: 0,
        sources_created: 0,
        candidate_ids: Vec::new(),
        skipped: Vec::new(),
    };

    storage.with_connection(|conn| {
        for (index, proposal) in proposals
            .into_iter()
            .take(config.max_candidates)
            .enumerate()
        {
            let candidate_id = format!("{}:candidate:{}", job_id, index + 1);
            let reason_codes = json!(proposal.reason_codes);
            let candidate = create_dream_candidate(
                conn,
                &NewDreamCandidate {
                    id: Some(&candidate_id),
                    job_id,
                    workspace: &config.workspace,
                    kind: proposal.kind,
                    proposed_action: proposal.proposed_action,
                    confidence: proposal.confidence,
                    freshness_state: proposal.freshness_state,
                    content_preview: &proposal.content_preview,
                    proposed_content: proposal.proposed_content.as_deref(),
                    reason_codes: &reason_codes,
                    policy_explanation: &proposal.policy_explanation,
                    metadata: &proposal.metadata,
                },
            )?;

            for source in proposal.sources {
                add_dream_candidate_source(
                    conn,
                    &NewDreamCandidateSource {
                        candidate_id: &candidate.id,
                        source_type: source.source_type,
                        source_id: &source.source_id,
                        source_ref: source.source_ref.as_deref(),
                        evidence: &source.evidence,
                    },
                )?;
                report.sources_created += 1;
            }

            report.candidates_created += 1;
            report.candidate_ids.push(candidate.id);
        }

        let output_summary = json!({
            "memories_scanned": report.memories_scanned,
            "candidates_created": report.candidates_created,
            "sources_created": report.sources_created,
            "generator_version": GENERATOR_VERSION
        });
        transition_dream_job(conn, job_id, "completed", Some(&output_summary), None)?;
        Ok(())
    })?;

    Ok(report)
}

fn build_proposals(
    memories: &[MemoryEvidence],
    config: &DreamCandidateGenerationConfig,
) -> Vec<CandidateProposal> {
    let mut proposals = Vec::new();
    let safe_memories: Vec<_> = memories
        .iter()
        .filter(|memory| !is_unsafe_raw_payload(&memory.content))
        .collect();

    if safe_memories.len() >= config.summary_min_memories {
        proposals.push(summary_proposal(&safe_memories, &config.workspace));
    }

    for proposal in stable_signal_proposals(&safe_memories) {
        proposals.push(proposal);
    }

    for memory in &safe_memories {
        if let Some(proposal) = stale_or_expired_proposal(memory) {
            proposals.push(proposal);
        }
    }

    for memory in &safe_memories {
        if let Some(proposal) = policy_proposal(memory) {
            proposals.push(proposal);
        }
    }

    proposals
}

fn summary_proposal(memories: &[&MemoryEvidence], workspace: &str) -> CandidateProposal {
    let selected: Vec<_> = memories
        .iter()
        .copied()
        .take(SUMMARY_SOURCE_LIMIT)
        .collect();
    let bullets: Vec<String> = selected
        .iter()
        .map(|memory| format!("- {}", preview(&memory.content, 96)))
        .collect();
    let proposed_content = format!(
        "Dream snapshot summary for workspace `{}`:\n{}",
        workspace,
        bullets.join("\n")
    );
    let avg_confidence = if selected.is_empty() {
        0.5
    } else {
        selected
            .iter()
            .map(|memory| memory.retrieval_priority.unwrap_or(memory.importance))
            .sum::<f64>()
            / selected.len() as f64
    }
    .clamp(0.35, 0.92);

    CandidateProposal {
        kind: "summary",
        proposed_action: "create",
        confidence: avg_confidence,
        freshness_state: "current",
        content_preview: preview(&proposed_content, PREVIEW_LIMIT),
        proposed_content: Some(proposed_content),
        reason_codes: vec!["workspace_digest", "carry_forward_context"],
        policy_explanation: json!({
            "generator": GENERATOR_VERSION,
            "source_count": selected.len()
        }),
        metadata: json!({
            "generator_version": GENERATOR_VERSION,
            "reducer": "deterministic_top_memories_v1",
            "target_memory_ids": selected.iter().map(|memory| memory.id).collect::<Vec<_>>()
        }),
        sources: selected
            .into_iter()
            .map(|memory| memory_source(memory, "summary_source"))
            .collect(),
    }
}

fn stable_signal_proposals(memories: &[&MemoryEvidence]) -> Vec<CandidateProposal> {
    let preference_sources: Vec<_> = memories
        .iter()
        .copied()
        .filter(|memory| durable_preference_signal(memory))
        .take(SUMMARY_SOURCE_LIMIT)
        .collect();
    let constraint_sources: Vec<_> = memories
        .iter()
        .copied()
        .filter(|memory| durable_constraint_signal(memory))
        .take(SUMMARY_SOURCE_LIMIT)
        .collect();
    let mut proposals = Vec::new();
    if preference_sources.len() >= 2 {
        proposals.push(stable_signal_proposal(
            "preference",
            "Stable preferences detected",
            "stable_preference",
            "repeated_preference_evidence",
            preference_sources,
        ));
    }
    if constraint_sources.len() >= 2 {
        proposals.push(stable_signal_proposal(
            "constraint",
            "Stable constraints detected",
            "stable_constraint",
            "repeated_constraint_evidence",
            constraint_sources,
        ));
    }
    proposals
}

fn stable_signal_proposal(
    kind: &'static str,
    label: &str,
    primary_reason: &'static str,
    secondary_reason: &'static str,
    sources: Vec<&MemoryEvidence>,
) -> CandidateProposal {
    let bullets: Vec<String> = sources
        .iter()
        .map(|memory| format!("- {}", preview(&memory.content, 96)))
        .collect();
    let proposed_content = format!("{}:\n{}", label, bullets.join("\n"));
    let avg_confidence = sources
        .iter()
        .map(|memory| memory.retrieval_priority.unwrap_or(memory.importance))
        .sum::<f64>()
        / sources.len() as f64;

    CandidateProposal {
        kind,
        proposed_action: "create",
        confidence: avg_confidence.clamp(0.5, 0.9),
        freshness_state: "current",
        content_preview: preview(&proposed_content, PREVIEW_LIMIT),
        proposed_content: Some(proposed_content),
        reason_codes: vec![primary_reason, secondary_reason],
        policy_explanation: json!({
            "generator": GENERATOR_VERSION,
            "source_count": sources.len()
        }),
        metadata: json!({
            "generator_version": GENERATOR_VERSION,
            "reducer": "deterministic_stable_signal_v1",
            "target_memory_ids": sources.iter().map(|memory| memory.id).collect::<Vec<_>>()
        }),
        sources: sources
            .into_iter()
            .map(|memory| memory_source(memory, secondary_reason))
            .collect(),
    }
}

fn durable_preference_signal(memory: &MemoryEvidence) -> bool {
    let content = memory.content.to_lowercase();
    memory.memory_type == "preference"
        || content.contains("prefer")
        || content.contains("preference")
        || content.contains("wants ")
}

fn durable_constraint_signal(memory: &MemoryEvidence) -> bool {
    let content = memory.content.to_lowercase();
    memory.memory_type == "decision"
        || content.contains("must ")
        || content.contains("requires")
        || content.contains("required")
        || content.contains("cannot")
        || content.contains("never ")
}

fn stale_or_expired_proposal(memory: &MemoryEvidence) -> Option<CandidateProposal> {
    let now = Utc::now();
    if memory
        .expires_at
        .is_some_and(|expires_at| expires_at <= now)
    {
        return Some(CandidateProposal {
            kind: "stale_fact",
            proposed_action: "expire",
            confidence: 0.88,
            freshness_state: "expired",
            content_preview: format!(
                "Expired memory should be reviewed: {}",
                preview(&memory.content, 96)
            ),
            proposed_content: None,
            reason_codes: vec!["expired_memory", "review_lifecycle_state"],
            policy_explanation: policy_explanation(memory),
            metadata: json!({
                "generator_version": GENERATOR_VERSION,
                "target_memory_ids": [memory.id],
                "expiration_reason": "memory_expires_at_is_past"
            }),
            sources: vec![memory_source(memory, "expired_memory")],
        });
    }

    if planned_event_is_stale(memory, now) {
        return Some(CandidateProposal {
            kind: "temporal_update",
            proposed_action: "update",
            confidence: 0.72,
            freshness_state: "stale",
            content_preview: format!(
                "Past planned event needs confirmation: {}",
                preview(&memory.content, 96)
            ),
            proposed_content: Some(format!(
                "Review whether this planned event is still current: {}",
                memory.content
            )),
            reason_codes: vec!["past_planned_event", "needs_confirmation"],
            policy_explanation: policy_explanation(memory),
            metadata: json!({
                "generator_version": GENERATOR_VERSION,
                "target_memory_ids": [memory.id],
                "temporal_signal": "event_time_in_past_with_planning_language"
            }),
            sources: vec![memory_source(memory, "past_planned_event")],
        });
    }

    None
}

fn policy_proposal(memory: &MemoryEvidence) -> Option<CandidateProposal> {
    let retrieval_priority = memory.retrieval_priority.unwrap_or(memory.importance);
    let salience_score = memory.salience_score.unwrap_or(memory.importance);
    if retrieval_priority >= 0.85 || salience_score >= 0.85 {
        return Some(CandidateProposal {
            kind: "promotion",
            proposed_action: "promote",
            confidence: retrieval_priority.max(salience_score).clamp(0.0, 0.95),
            freshness_state: "current",
            content_preview: format!(
                "High-salience memory should be easier to retrieve: {}",
                preview(&memory.content, 96)
            ),
            proposed_content: None,
            reason_codes: vec!["high_salience", "retrieval_priority"],
            policy_explanation: policy_explanation(memory),
            metadata: json!({
                "generator_version": GENERATOR_VERSION,
                "target_memory_ids": [memory.id],
                "policy_action": "promote_retrieval"
            }),
            sources: vec![memory_source(memory, "memory_policy")],
        });
    }

    let retention_score = memory.retention_score.unwrap_or(0.5);
    if retention_score <= 0.2 || memory.lifecycle_state == "stale" {
        return Some(CandidateProposal {
            kind: "decay",
            proposed_action: "demote",
            confidence: (1.0 - retention_score).clamp(0.35, 0.9),
            freshness_state: "stale",
            content_preview: format!(
                "Low-retention memory should be reviewed for demotion: {}",
                preview(&memory.content, 96)
            ),
            proposed_content: None,
            reason_codes: vec!["low_retention", "review_lifecycle_state"],
            policy_explanation: policy_explanation(memory),
            metadata: json!({
                "generator_version": GENERATOR_VERSION,
                "target_memory_ids": [memory.id],
                "policy_action": "demote_retrieval"
            }),
            sources: vec![memory_source(memory, "memory_policy")],
        });
    }

    None
}

fn load_memory_evidence(
    conn: &Connection,
    workspace: &str,
    limit: usize,
) -> Result<Vec<MemoryEvidence>> {
    let mut stmt = conn.prepare(
        "SELECT m.id, m.content, m.memory_type, m.importance, m.event_time,
                m.expires_at, COALESCE(m.lifecycle_state, 'active'),
                mp.salience_score, mp.retention_score, mp.retrieval_priority,
                mp.policy_reason
         FROM memories m
         LEFT JOIN memory_policy mp ON mp.memory_id = m.id
         WHERE m.workspace = ?1
           AND m.valid_to IS NULL
           AND COALESCE(m.lifecycle_state, 'active') != 'archived'
         ORDER BY COALESCE(mp.retrieval_priority, m.importance) DESC,
                  m.updated_at DESC,
                  m.id DESC
         LIMIT ?2",
    )?;

    let rows = stmt.query_map(params![workspace, limit as i64], |row| {
        let event_time: Option<String> = row.get(4)?;
        let expires_at: Option<String> = row.get(5)?;
        Ok(MemoryEvidence {
            id: row.get(0)?,
            content: row.get(1)?,
            memory_type: row.get(2)?,
            importance: row.get::<_, f64>(3)?,
            event_time: parse_optional_rfc3339(event_time),
            expires_at: parse_optional_rfc3339(expires_at),
            lifecycle_state: row.get(6)?,
            salience_score: row.get(7)?,
            retention_score: row.get(8)?,
            retrieval_priority: row.get(9)?,
            policy_reason: row.get(10)?,
        })
    })?;

    Ok(rows.collect::<std::result::Result<Vec<_>, _>>()?)
}

fn planned_event_is_stale(memory: &MemoryEvidence, now: DateTime<Utc>) -> bool {
    let Some(event_time) = memory.event_time else {
        return false;
    };
    if event_time > now {
        return false;
    }
    let content = memory.content.to_lowercase();
    ["planned", "scheduled", "due", "todo", "will ", "deadline"]
        .iter()
        .any(|needle| content.contains(needle))
}

fn is_unsafe_raw_payload(content: &str) -> bool {
    let lower = content.to_lowercase();
    lower.contains("-----begin")
        || lower.contains("api_key=")
        || lower.contains("secret=")
        || lower.contains("password=")
        || lower.contains("authorization: bearer")
        || lower.contains("env:")
        || lower.contains("terminal dump")
}

fn memory_source(memory: &MemoryEvidence, reason: &'static str) -> SourceProposal {
    SourceProposal {
        source_type: "memory",
        source_id: memory.id.to_string(),
        source_ref: Some(format!("memory:{}", memory.id)),
        evidence: json!({
            "reason": reason,
            "preview": preview(&memory.content, PREVIEW_LIMIT),
            "memory_type": memory.memory_type,
            "importance": memory.importance
        }),
    }
}

fn policy_explanation(memory: &MemoryEvidence) -> serde_json::Value {
    json!({
        "salience_score": memory.salience_score,
        "retention_score": memory.retention_score,
        "retrieval_priority": memory.retrieval_priority,
        "policy_reason": memory.policy_reason
    })
}

fn parse_optional_rfc3339(value: Option<String>) -> Option<DateTime<Utc>> {
    value
        .and_then(|raw| DateTime::parse_from_rfc3339(&raw).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn preview(content: &str, limit: usize) -> String {
    let mut out = String::new();
    for ch in content
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
    {
        if out.len() + ch.len_utf8() > limit {
            out.push_str("...");
            return out;
        }
        out.push(ch);
    }
    out
}

fn validate_config(config: &DreamCandidateGenerationConfig) -> Result<()> {
    if config.workspace.trim().is_empty() {
        return Err(EngramError::InvalidInput(
            "workspace must not be empty".to_string(),
        ));
    }
    if config.max_memories == 0 {
        return Err(EngramError::InvalidInput(
            "max_memories must be greater than zero".to_string(),
        ));
    }
    if config.max_candidates == 0 {
        return Err(EngramError::InvalidInput(
            "max_candidates must be greater than zero".to_string(),
        ));
    }
    Ok(())
}

fn mark_failed_best_effort(storage: &Storage, job_id: &str, err: &EngramError) {
    let error = json!({
        "message": err.to_string(),
        "kind": "candidate_generation_failed"
    });
    let _ = storage.with_connection(|conn| {
        transition_dream_job(conn, job_id, "failed", None, Some(&error))?;
        Ok(())
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::queries::create_memory;
    use crate::types::{CreateMemoryInput, MemoryTier, MemoryType};

    fn create_test_memory(storage: &Storage, content: &str, importance: f32) -> i64 {
        storage
            .with_transaction(|conn| {
                let memory = create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: content.to_string(),
                        memory_type: MemoryType::Note,
                        workspace: Some("default".to_string()),
                        importance: Some(importance),
                        defer_embedding: true,
                        ..Default::default()
                    },
                )?;
                Ok(memory.id)
            })
            .expect("create memory")
    }

    #[test]
    fn generator_creates_review_candidates_without_canonical_mutation() {
        let storage = Storage::open_in_memory().expect("open storage");
        create_test_memory(&storage, "Release checklist requires local CI.", 0.9);
        create_test_memory(&storage, "Huly issues track review ownership.", 0.8);

        let before_count: i64 = storage
            .with_connection(|conn| {
                conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
                    .map_err(Into::into)
            })
            .expect("count before");

        let report = run_candidate_generation(
            &storage,
            &DreamCandidateGenerationConfig {
                job_id: Some("job-generator-summary".to_string()),
                max_candidates: 3,
                ..Default::default()
            },
        )
        .expect("run generator");

        assert_eq!(report.job_id, "job-generator-summary");
        assert_eq!(report.memories_scanned, 2);
        assert!(report.candidates_created >= 1);
        assert!(report.sources_created >= 2);

        let after_count: i64 = storage
            .with_connection(|conn| {
                conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))
                    .map_err(Into::into)
            })
            .expect("count after");
        assert_eq!(before_count, after_count);

        storage
            .with_connection(|conn| {
                let job_status: String = conn.query_row(
                    "SELECT status FROM dream_jobs WHERE id = 'job-generator-summary'",
                    [],
                    |row| row.get(0),
                )?;
                assert_eq!(job_status, "completed");
                let summary_count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM dream_candidates
                     WHERE job_id = 'job-generator-summary'
                       AND kind = 'summary'
                       AND proposed_action = 'create'
                       AND review_state = 'pending'",
                    [],
                    |row| row.get(0),
                )?;
                assert_eq!(summary_count, 1);
                Ok(())
            })
            .expect("inspect generated candidates");
    }

    #[test]
    fn generator_emits_expired_memory_candidate() {
        let storage = Storage::open_in_memory().expect("open storage");
        let memory_id = create_test_memory(&storage, "Temporary deployment note.", 0.5);
        storage
            .with_transaction(|conn| {
                conn.execute(
                    "UPDATE memories
                     SET tier = ?1, expires_at = ?2
                     WHERE id = ?3",
                    params![
                        MemoryTier::Daily.as_str(),
                        "2026-01-01T00:00:00Z",
                        memory_id
                    ],
                )?;
                Ok(())
            })
            .expect("expire memory");

        let report = run_candidate_generation(
            &storage,
            &DreamCandidateGenerationConfig {
                job_id: Some("job-generator-expired".to_string()),
                summary_min_memories: 2,
                max_candidates: 5,
                ..Default::default()
            },
        )
        .expect("run generator");

        assert_eq!(report.candidates_created, 1);
        storage
            .with_connection(|conn| {
                let (action, freshness): (String, String) = conn.query_row(
                    "SELECT proposed_action, freshness_state
                     FROM dream_candidates
                     WHERE job_id = 'job-generator-expired'",
                    [],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )?;
                assert_eq!(action, "expire");
                assert_eq!(freshness, "expired");
                let sources: i64 =
                    conn.query_row("SELECT COUNT(*) FROM dream_candidate_sources", [], |row| {
                        row.get(0)
                    })?;
                assert_eq!(sources, 1);
                Ok(())
            })
            .expect("inspect expired candidate");
    }

    #[test]
    fn generator_emits_policy_promotion_candidate() {
        let storage = Storage::open_in_memory().expect("open storage");
        let memory_id = create_test_memory(&storage, "Durable architectural decision.", 0.7);
        storage
            .with_transaction(|conn| {
                conn.execute(
                    "UPDATE memory_policy
                     SET salience_score = 0.92,
                         retrieval_priority = 0.91,
                         policy_reason = 'test high salience'
                     WHERE memory_id = ?1",
                    params![memory_id],
                )?;
                Ok(())
            })
            .expect("raise policy");

        run_candidate_generation(
            &storage,
            &DreamCandidateGenerationConfig {
                job_id: Some("job-generator-policy".to_string()),
                summary_min_memories: 2,
                max_candidates: 5,
                ..Default::default()
            },
        )
        .expect("run generator");

        storage
            .with_connection(|conn| {
                let count: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM dream_candidates
                     WHERE job_id = 'job-generator-policy'
                       AND kind = 'promotion'
                       AND proposed_action = 'promote'",
                    [],
                    |row| row.get(0),
                )?;
                assert_eq!(count, 1);
                Ok(())
            })
            .expect("inspect policy candidate");
    }
}
