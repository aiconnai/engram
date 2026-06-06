//! Database queries for durable memory policy records.

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::error::{EngramError, Result};
use crate::storage::enrichment_events::{self, EnrichmentEvent};
use crate::types::MemoryId;

const DEFAULT_POLICY_SCORE: f32 = 0.5;
const MAX_REINFORCEMENT_BOOST: f32 = 0.25;
const CONTRADICTION_DEMOTION: f32 = 0.15;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRecord {
    pub memory_id: MemoryId,
    pub salience_score: f32,
    pub retention_score: f32,
    pub retrieval_priority: f32,
    pub last_reinforced_at: Option<String>,
    pub reinforcement_count: i64,
    pub contradiction_count: i64,
    pub policy_version: String,
    pub policy_reason: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct PolicyRecordInput {
    pub memory_id: MemoryId,
    pub salience_score: f32,
    pub retention_score: f32,
    pub retrieval_priority: f32,
    pub policy_version: String,
    pub policy_reason: String,
}

fn clamp_score(score: f32) -> f32 {
    if !score.is_finite() {
        DEFAULT_POLICY_SCORE
    } else {
        score.clamp(0.0, 1.0)
    }
}

fn bounded_boost(boost: f32) -> f32 {
    if boost.is_nan() || boost <= 0.0 {
        0.0
    } else if boost >= MAX_REINFORCEMENT_BOOST {
        MAX_REINFORCEMENT_BOOST
    } else {
        boost
    }
}

fn policy_record_from_row(row: &Row<'_>) -> rusqlite::Result<PolicyRecord> {
    Ok(PolicyRecord {
        memory_id: row.get("memory_id")?,
        salience_score: row.get("salience_score")?,
        retention_score: row.get("retention_score")?,
        retrieval_priority: row.get("retrieval_priority")?,
        last_reinforced_at: row.get("last_reinforced_at")?,
        reinforcement_count: row.get("reinforcement_count")?,
        contradiction_count: row.get("contradiction_count")?,
        policy_version: row.get("policy_version")?,
        policy_reason: row.get("policy_reason")?,
        updated_at: row.get("updated_at")?,
    })
}

pub fn get_policy_record(conn: &Connection, memory_id: MemoryId) -> Result<Option<PolicyRecord>> {
    let record = conn
        .query_row(
            "SELECT memory_id, salience_score, retention_score, retrieval_priority,
                    last_reinforced_at, reinforcement_count, contradiction_count,
                    policy_version, policy_reason, updated_at
             FROM memory_policy
             WHERE memory_id = ?1",
            params![memory_id],
            policy_record_from_row,
        )
        .optional()?;

    Ok(record)
}

pub fn upsert_policy_record(conn: &Connection, input: PolicyRecordInput) -> Result<PolicyRecord> {
    let salience_score = clamp_score(input.salience_score);
    let retention_score = clamp_score(input.retention_score);
    let retrieval_priority = clamp_score(input.retrieval_priority);
    let updated_at = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO memory_policy (
             memory_id, salience_score, retention_score, retrieval_priority,
             policy_version, policy_reason, updated_at
         )
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
         ON CONFLICT(memory_id) DO UPDATE SET
             salience_score = excluded.salience_score,
             retention_score = excluded.retention_score,
             retrieval_priority = excluded.retrieval_priority,
             policy_version = excluded.policy_version,
             policy_reason = excluded.policy_reason,
             updated_at = excluded.updated_at",
        params![
            input.memory_id,
            salience_score,
            retention_score,
            retrieval_priority,
            input.policy_version,
            input.policy_reason,
            updated_at,
        ],
    )?;

    get_policy_record(conn, input.memory_id)?.ok_or_else(|| {
        EngramError::Storage(format!(
            "memory_policy upsert did not return row for memory_id {}",
            input.memory_id
        ))
    })
}

pub fn record_reinforcement(
    conn: &Connection,
    memory_id: MemoryId,
    boost: f32,
    triggered_by: &str,
) -> Result<PolicyRecord> {
    let now = Utc::now().to_rfc3339();
    let boost = bounded_boost(boost);

    conn.execute(
        "INSERT OR IGNORE INTO memory_policy (memory_id, updated_at)
         VALUES (?1, ?2)",
        params![memory_id, now],
    )?;

    conn.execute(
        "UPDATE memory_policy
         SET salience_score = MIN(1.0, MAX(0.0, salience_score + ?1)),
             retention_score = MIN(1.0, MAX(0.0, retention_score + ?1)),
             retrieval_priority = MIN(1.0, MAX(0.0, retrieval_priority + ?1)),
             last_reinforced_at = ?2,
             reinforcement_count = reinforcement_count + 1,
             updated_at = ?2
         WHERE memory_id = ?3",
        params![boost, now, memory_id],
    )?;

    let record = get_policy_record(conn, memory_id)?.ok_or(EngramError::NotFound(memory_id))?;
    emit_policy_event(conn, triggered_by, &record, false);
    Ok(record)
}

pub fn record_contradiction(
    conn: &Connection,
    memory_id: MemoryId,
    triggered_by: &str,
    reason: &str,
) -> Result<PolicyRecord> {
    let now = Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR IGNORE INTO memory_policy (memory_id, updated_at)
         VALUES (?1, ?2)",
        params![memory_id, now],
    )?;

    conn.execute(
        "UPDATE memory_policy
         SET salience_score = MIN(1.0, MAX(0.0, salience_score - ?1)),
             retention_score = MIN(1.0, MAX(0.0, retention_score - ?1)),
             retrieval_priority = MIN(1.0, MAX(0.0, retrieval_priority - ?1)),
             contradiction_count = contradiction_count + 1,
             policy_reason = ?2,
             updated_at = ?3
         WHERE memory_id = ?4",
        params![CONTRADICTION_DEMOTION, reason, now, memory_id],
    )?;

    let record = get_policy_record(conn, memory_id)?.ok_or(EngramError::NotFound(memory_id))?;
    emit_policy_event(conn, triggered_by, &record, false);
    Ok(record)
}

pub fn emit_policy_event(
    conn: &Connection,
    triggered_by: &str,
    record: &PolicyRecord,
    dry_run: bool,
) {
    let now = Utc::now();
    let operation_id = format!(
        "memory-policy-{}-{}",
        record.memory_id,
        now.timestamp_micros()
    );
    let version_id = enrichment_events::latest_version_id(conn, record.memory_id)
        .ok()
        .flatten();
    let event_type = if record.contradiction_count > 0 && !record.policy_reason.is_empty() {
        "memory_policy_conflict"
    } else {
        "memory_policy"
    };
    let event = EnrichmentEvent {
        operation_id: &operation_id,
        event_type,
        memory_id: Some(record.memory_id),
        version_id,
        triggered_by,
        agent_id: None,
        workspace: None,
        params: json!({
            "policy_version": record.policy_version,
            "dry_run": dry_run
        }),
        outcome: json!({
            "salience_score": record.salience_score,
            "retention_score": record.retention_score,
            "retrieval_priority": record.retrieval_priority,
            "reinforcement_count": record.reinforcement_count,
            "contradiction_count": record.contradiction_count,
            "policy_reason": record.policy_reason,
            "updated_at": record.updated_at
        }),
        status: "completed",
        dry_run,
    };

    enrichment_events::emit_best_effort(conn, &event);
}
