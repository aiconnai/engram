//! Public MCP handlers for durable memory policy operations.

use rusqlite::{params, OptionalExtension};
use serde_json::{json, Value};

use crate::error::{EngramError, Result};
use crate::intelligence::{
    explain_policy_score, extract_features, score_policy, PolicyFeatureInput, PolicyFeatures,
    PolicyScore,
};
use crate::storage::queries::{
    emit_policy_event, get_policy_record, memory_from_row, promote_to_permanent,
    record_contradiction, record_reinforcement, upsert_policy_record, PolicyRecord,
    PolicyRecordInput,
};
use crate::types::{LifecycleState, Memory, MemoryId};

use super::HandlerContext;

const POLICY_REINFORCEMENT_BOOST: f32 = 0.10;
const DECAY_STEP: f32 = 0.05;

pub fn memory_score(ctx: &HandlerContext, params: Value) -> Value {
    let id = match required_id(&params) {
        Ok(id) => id,
        Err(error) => return error,
    };
    let persist = params
        .get("persist")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let result = if persist {
        ctx.storage.with_transaction(|conn| {
            let (memory, existing_policy, features, score) = score_memory(conn, id)?;
            let policy = upsert_policy_record(conn, policy_input(id, &score))?;
            emit_policy_event(conn, "memory_score", &policy, false);
            Ok(json!({
                "memory_id": id,
                "persisted": true,
                "source": if existing_policy.is_some() { "stored" } else { "heuristic-v1" },
                "memory": memory,
                "policy": policy,
                "components": features,
                "score": score
            }))
        })
    } else {
        ctx.storage.with_connection(|conn| {
            let (memory, existing_policy, features, score) = score_memory(conn, id)?;
            Ok(json!({
                "memory_id": id,
                "persisted": false,
                "source": if existing_policy.is_some() { "stored" } else { "heuristic-v1" },
                "memory": memory,
                "policy": score,
                "components": features
            }))
        })
    };

    result.unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_promote(ctx: &HandlerContext, params: Value) -> Value {
    let id = match required_id(&params) {
        Ok(id) => id,
        Err(error) => return error,
    };
    let canonical_tier = params
        .get("canonical_tier")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    ctx.storage
        .with_transaction(|conn| {
            load_memory_readonly(conn, id)?;
            let policy =
                record_reinforcement(conn, id, POLICY_REINFORCEMENT_BOOST, "memory_promote")?;

            if canonical_tier {
                let canonical_memory = promote_to_permanent(conn, id)?;
                return Ok(json!({
                    "memory_id": id,
                    "canonical_tier": true,
                    "policy": policy,
                    "canonical_memory": canonical_memory
                }));
            }

            Ok(json!({
                "memory_id": id,
                "canonical_tier": false,
                "policy": policy
            }))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_decay(ctx: &HandlerContext, params: Value) -> Value {
    let dry_run = params
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .unwrap_or("default")
        .to_string();

    let result = if dry_run {
        ctx.storage.with_connection(|conn| {
            let candidates = decay_candidates(conn, &workspace)?;
            Ok(json!({
                "workspace": workspace,
                "dry_run": true,
                "candidate_count": candidates.len(),
                "candidates": candidates,
                "concern": "dry-run only; no policy or lifecycle rows mutated"
            }))
        })
    } else {
        ctx.storage.with_transaction(|conn| {
            let candidates = decay_candidates(conn, &workspace)?;
            let mut policy_updates = 0usize;

            for candidate in &candidates {
                let policy = upsert_policy_record(
                    conn,
                    PolicyRecordInput {
                        memory_id: candidate.memory_id,
                        salience_score: candidate.new_salience_score,
                        retention_score: candidate.new_retention_score,
                        retrieval_priority: candidate.new_retrieval_priority,
                        policy_version: "heuristic-v1".to_string(),
                        policy_reason: candidate.reason.clone(),
                    },
                )?;
                emit_policy_event(conn, "memory_decay", &policy, false);
                policy_updates += 1;
            }

            Ok(json!({
                "workspace": workspace,
                "dry_run": false,
                "candidate_count": candidates.len(),
                "policy_updates": policy_updates,
                "lifecycle_updates": 0,
                "candidates": candidates,
                "concern": "apply is conservative: only memory_policy scores are updated; use lifecycle_run for lifecycle transitions"
            }))
        })
    };

    result.unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_explain(ctx: &HandlerContext, params: Value) -> Value {
    let id = match required_id(&params) {
        Ok(id) => id,
        Err(error) => return error,
    };

    ctx.storage
        .with_connection(|conn| {
            let (_memory, stored_policy, features, score) = score_memory(conn, id)?;
            let explanation = explain_policy_score(&features, &score);
            let latest_policy_event_count = policy_event_count(conn, id)?;

            Ok(json!({
                "memory_id": id,
                "policy": stored_policy.unwrap_or_else(|| policy_record_from_score(id, &score)),
                "components": features,
                "reason": format!(
                    "novelty:{} reinforcement:{} contradictions:{}",
                    bucket(features.novelty),
                    reinforcement_count(conn, id)?,
                    contradiction_count(conn, id)?
                ),
                "explanation": explanation,
                "audit": {
                    "latest_policy_event_count": latest_policy_event_count
                }
            }))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_reconcile_conflict(ctx: &HandlerContext, params: Value) -> Value {
    let id = match required_id(&params) {
        Ok(id) => id,
        Err(error) => return error,
    };
    let reason = match params.get("reason").and_then(|v| v.as_str()) {
        Some(reason) if !reason.trim().is_empty() => reason.trim().to_string(),
        _ => return json!({"error": "reason is required"}),
    };

    ctx.storage
        .with_transaction(|conn| {
            load_memory_readonly(conn, id)?;
            let policy = record_contradiction(conn, id, "memory_reconcile_conflict", &reason)?;
            Ok(json!({
                "memory_id": id,
                "reason": reason,
                "policy": policy,
                "content_mutated": false,
                "deleted": false
            }))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

fn required_id(params: &Value) -> std::result::Result<MemoryId, Value> {
    params
        .get("id")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| json!({"error": "id is required"}))
}

fn score_memory(
    conn: &rusqlite::Connection,
    id: MemoryId,
) -> Result<(Memory, Option<PolicyRecord>, PolicyFeatures, PolicyScore)> {
    let memory = load_memory_readonly(conn, id)?;
    let existing_policy = get_policy_record(conn, id)?;
    let features = extract_features(PolicyFeatureInput {
        memory: &memory,
        existing_policy: existing_policy.as_ref(),
        event: None,
        hybrid_search_score: None,
        session_relevance: None,
    });
    let score = score_policy(&features);
    Ok((memory, existing_policy, features, score))
}

fn policy_input(memory_id: MemoryId, score: &PolicyScore) -> PolicyRecordInput {
    PolicyRecordInput {
        memory_id,
        salience_score: score.salience_score,
        retention_score: score.retention_score,
        retrieval_priority: score.retrieval_priority,
        policy_version: score.policy_version.clone(),
        policy_reason: score.policy_reason.clone(),
    }
}

fn policy_record_from_score(memory_id: MemoryId, score: &PolicyScore) -> PolicyRecord {
    PolicyRecord {
        memory_id,
        salience_score: score.salience_score,
        retention_score: score.retention_score,
        retrieval_priority: score.retrieval_priority,
        last_reinforced_at: None,
        reinforcement_count: 0,
        contradiction_count: 0,
        policy_version: score.policy_version.clone(),
        policy_reason: score.policy_reason.clone(),
        updated_at: String::new(),
    }
}

fn load_memory_readonly(conn: &rusqlite::Connection, id: MemoryId) -> Result<Memory> {
    let mut memory = conn
        .query_row(
            "SELECT id, content, memory_type, importance, access_count,
                    created_at, updated_at, last_accessed_at, owner_id, visibility,
                    version, has_embedding, metadata, scope_type, scope_id, workspace,
                    tier, expires_at, content_hash, event_time, event_duration_seconds,
                    trigger_pattern, procedure_success_count, procedure_failure_count,
                    summary_of_id, lifecycle_state, media_url
             FROM memories
             WHERE id = ?1 AND valid_to IS NULL",
            params![id],
            memory_from_row,
        )
        .optional()?
        .ok_or(EngramError::NotFound(id))?;

    let mut stmt = conn.prepare(
        "SELECT t.name
         FROM tags t
         JOIN memory_tags mt ON t.id = mt.tag_id
         WHERE mt.memory_id = ?1
         ORDER BY t.name",
    )?;
    memory.tags = stmt
        .query_map(params![id], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(memory)
}

#[derive(serde::Serialize)]
struct DecayCandidate {
    memory_id: MemoryId,
    current_salience_score: f32,
    current_retention_score: f32,
    current_retrieval_priority: f32,
    new_salience_score: f32,
    new_retention_score: f32,
    new_retrieval_priority: f32,
    lifecycle_state: String,
    lifecycle_target: Option<LifecycleState>,
    reason: String,
}

fn decay_candidates(conn: &rusqlite::Connection, workspace: &str) -> Result<Vec<DecayCandidate>> {
    let mut stmt = conn.prepare(
        "SELECT id
         FROM memories
         WHERE workspace = ?1 AND valid_to IS NULL
         ORDER BY updated_at ASC, id ASC
         LIMIT 500",
    )?;
    let ids = stmt
        .query_map(params![workspace], |row| row.get::<_, MemoryId>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;

    let mut candidates = Vec::with_capacity(ids.len());
    for id in ids {
        let (memory, stored_policy, _features, score) = score_memory(conn, id)?;
        let current_salience = stored_policy
            .as_ref()
            .map(|p| p.salience_score)
            .unwrap_or(score.salience_score);
        let current_retention = stored_policy
            .as_ref()
            .map(|p| p.retention_score)
            .unwrap_or(score.retention_score);
        let current_priority = stored_policy
            .as_ref()
            .map(|p| p.retrieval_priority)
            .unwrap_or(score.retrieval_priority);

        let new_salience = decay_score(current_salience);
        let new_retention = decay_score(current_retention);
        let new_priority = decay_score(current_priority);
        let lifecycle_target = None;

        candidates.push(DecayCandidate {
            memory_id: id,
            current_salience_score: current_salience,
            current_retention_score: current_retention,
            current_retrieval_priority: current_priority,
            new_salience_score: new_salience,
            new_retention_score: new_retention,
            new_retrieval_priority: new_priority,
            lifecycle_state: memory.lifecycle_state.to_string(),
            lifecycle_target,
            reason: format!("memory_decay:{}", score.policy_reason),
        });
    }

    Ok(candidates)
}

fn decay_score(score: f32) -> f32 {
    if score.is_finite() {
        (score - DECAY_STEP).clamp(0.0, 1.0)
    } else {
        0.5
    }
}

fn policy_event_count(conn: &rusqlite::Connection, id: MemoryId) -> Result<i64> {
    Ok(conn.query_row(
        "SELECT COUNT(*)
         FROM enrichment_events
         WHERE memory_id = ?1 AND event_type IN ('memory_policy', 'memory_policy_conflict')",
        params![id],
        |row| row.get(0),
    )?)
}

fn reinforcement_count(conn: &rusqlite::Connection, id: MemoryId) -> Result<i64> {
    Ok(get_policy_record(conn, id)?
        .map(|policy| policy.reinforcement_count)
        .unwrap_or(0))
}

fn contradiction_count(conn: &rusqlite::Connection, id: MemoryId) -> Result<i64> {
    Ok(get_policy_record(conn, id)?
        .map(|policy| policy.contradiction_count)
        .unwrap_or(0))
}

fn bucket(score: f32) -> &'static str {
    if score >= 0.75 {
        "high"
    } else if score >= 0.35 {
        "medium"
    } else {
        "low"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_ctx() -> super::HandlerContext {
        use crate::embedding::{create_embedder, EmbeddingCache};
        use crate::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
        use crate::storage::Storage;
        use crate::types::EmbeddingConfig;
        use parking_lot::Mutex;
        use std::sync::Arc;

        let storage = Storage::open_in_memory().expect("in-memory storage");
        let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
        super::HandlerContext {
            storage,
            embedder: embedder.clone(),
            fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
            search_config: SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(EmbeddingCache::default()),
            search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
            hnsw_index: Arc::new(parking_lot::RwLock::new(crate::search::HnswIndex::new(
                crate::search::HnswConfig::new(
                    embedder.dimensions(),
                    crate::search::VectorMetric::Cosine,
                ),
            ))),
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 60,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
            progress_reporter: None,
        }
    }

    #[test]
    fn memory_decay_updates_policy_scores_without_lifecycle_transition() {
        use crate::storage::queries::create_memory;
        use crate::types::{CreateMemoryInput, MemoryTier, MemoryType};

        let ctx = test_ctx();
        let memory_id = ctx
            .storage
            .with_transaction(|conn| {
                let memory = create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: "policy decay lifecycle guard".to_string(),
                        memory_type: MemoryType::Note,
                        workspace: Some("default".to_string()),
                        tier: MemoryTier::Permanent,
                        ..Default::default()
                    },
                )?;
                upsert_policy_record(
                    conn,
                    PolicyRecordInput {
                        memory_id: memory.id,
                        salience_score: 0.30,
                        retention_score: 0.20,
                        retrieval_priority: 0.40,
                        policy_version: "heuristic-test".to_string(),
                        policy_reason: "test low retention".to_string(),
                    },
                )?;
                Ok(memory.id)
            })
            .expect("seed policy memory");

        let result = memory_decay(
            &ctx,
            serde_json::json!({
                "dry_run": false,
                "workspace": "default"
            }),
        );

        assert!(result.get("error").is_none(), "unexpected error: {result}");
        assert_eq!(result["policy_updates"].as_u64(), Some(1), "{result}");
        assert_eq!(result["lifecycle_updates"].as_u64(), Some(0), "{result}");

        let (lifecycle_state, policy) = ctx
            .storage
            .with_connection(|conn| {
                let lifecycle_state: String = conn.query_row(
                    "SELECT lifecycle_state FROM memories WHERE id = ?1",
                    params![memory_id],
                    |row| row.get(0),
                )?;
                let policy = get_policy_record(conn, memory_id)?.expect("policy record");
                Ok((lifecycle_state, policy))
            })
            .expect("query policy and lifecycle");

        assert_eq!(lifecycle_state, "active");
        assert!(
            policy.retention_score < 0.20,
            "retention score should decay, got {}",
            policy.retention_score
        );
    }
}
