//! Memory creation tools (create, seed, daily, episodic, procedural, section, batch-create).
use serde_json::{json, Value};

use super::super::HandlerContext;
use crate::mcp::error::ToolError;
use crate::mcp::progress::ProgressReporterExt;
use crate::realtime::RealtimeEvent;
use crate::storage::enrichment_events::{emit_best_effort, EnrichmentEvent};
use crate::storage::queries::*;
use crate::types::*;

pub fn memory_create(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::find_similar_by_embedding;

    let input: CreateMemoryInput = match serde_json::from_value(params) {
        Ok(i) => i,
        Err(e) => return ToolError::invalid_params(e.to_string()).into_value(),
    };

    // Semantic deduplication
    if input.dedup_mode != DedupMode::Allow {
        if let Some(threshold) = input.dedup_threshold {
            if let Ok(query_embedding) = ctx.embedder.embed(&input.content) {
                let workspace = input.workspace.as_deref();
                let similar_result = ctx.storage.with_connection(|conn| {
                    find_similar_by_embedding(
                        conn,
                        &query_embedding,
                        &input.scope,
                        workspace,
                        threshold,
                    )
                });

                if let Ok(Some((existing, similarity))) = similar_result {
                    match input.dedup_mode {
                        DedupMode::Reject => {
                            return ToolError::conflict(format!(
                                "Similar memory detected (id={}, similarity={:.3}). Use dedup_mode='allow' to create anyway.",
                                existing.id, similarity
                            ))
                            .with_details(json!({
                                "existing_id": existing.id,
                                "similarity": similarity
                            }))
                            .into_value();
                        }
                        DedupMode::Skip => {
                            return json!(existing);
                        }
                        DedupMode::Merge => {
                            let merge_result = ctx.storage.with_transaction(|conn| {
                                let mut merged_tags = existing.tags.clone();
                                for tag in &input.tags {
                                    if !merged_tags.contains(tag) {
                                        merged_tags.push(tag.clone());
                                    }
                                }

                                let mut merged_metadata = existing.metadata.clone();
                                for (key, value) in &input.metadata {
                                    merged_metadata.insert(key.clone(), value.clone());
                                }

                                let update_input = UpdateMemoryInput {
                                    content: None,
                                    memory_type: None,
                                    tags: Some(merged_tags),
                                    metadata: Some(merged_metadata),
                                    importance: input.importance,
                                    scope: None,
                                    ttl_seconds: input.ttl_seconds,
                                    event_time: None,
                                    trigger_pattern: None,
                                    media_url: input.media_url.map(Some),
                                };

                                update_memory(conn, existing.id, &update_input)
                            });

                            return match merge_result {
                                Ok(memory) => json!(memory),
                                Err(e) => json!({"error": e.to_string()}),
                            };
                        }
                        DedupMode::Allow => {}
                    }
                }
            }
        }
    }

    let result = ctx.storage.with_transaction(|conn| {
        let memory = create_memory(conn, &input)?;
        let mut fuzzy = ctx.fuzzy_engine.lock();
        fuzzy.add_to_vocabulary(&memory.content);
        let op_id = uuid::Uuid::new_v4().to_string();
        emit_best_effort(
            conn,
            &EnrichmentEvent {
                operation_id: &op_id,
                event_type: "memory_created",
                memory_id: Some(memory.id),
                version_id: None,
                triggered_by: "memory_create",
                agent_id: None,
                workspace: Some(memory.workspace.as_str()),
                params: json!({}),
                outcome: json!({"id": memory.id}),
                status: "completed",
                dry_run: false,
            },
        );
        Ok(memory)
    });

    match result {
        Ok(memory) => {
            if !input.defer_embedding {
                if let Ok(emb) = ctx.embedder.embed(&memory.content) {
                    let _ = ctx.storage.with_connection(|conn| {
                        let mut bytes = Vec::with_capacity(emb.len() * 4);
                        for f in &emb {
                            bytes.extend_from_slice(&f.to_le_bytes());
                        }
                        conn.execute(
                            "INSERT OR REPLACE INTO embeddings (memory_id, embedding, model, dimensions, created_at)
                             VALUES (?1, ?2, 'default', ?3, datetime('now'))",
                            rusqlite::params![memory.id, bytes, emb.len()],
                        )?;
                        conn.execute(
                            "UPDATE memories SET has_embedding = 1 WHERE id = ?",
                            rusqlite::params![memory.id],
                        )?;
                        Ok(())
                    });
                    ctx.hnsw_index.write().insert(memory.id, &emb);
                }
            } else {
                let _ = ctx.storage.with_connection(|conn| {
                    if let Ok(Some(emb)) = crate::embedding::get_embedding(conn, memory.id) {
                        ctx.hnsw_index.write().insert(memory.id, &emb);
                    }
                    Ok(())
                });
            }
            ctx.search_cache
                .invalidate_for_workspace(Some(memory.workspace.as_str()));
            if let Some(ref manager) = ctx.realtime {
                manager.broadcast(RealtimeEvent::memory_created(
                    memory.id,
                    memory.content.clone(),
                    memory.workspace.clone(),
                ));
            }
            json!(memory)
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}

pub fn context_seed(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::create_memory_batch;
    use std::collections::HashMap;

    #[derive(serde::Deserialize)]
    struct ContextSeedFact {
        content: String,
        category: Option<String>,
        confidence: Option<f32>,
    }

    #[derive(serde::Deserialize)]
    struct ContextSeedInput {
        entity_context: Option<String>,
        workspace: Option<String>,
        base_tags: Option<Vec<String>>,
        ttl_seconds: Option<i64>,
        disable_ttl: Option<bool>,
        facts: Vec<ContextSeedFact>,
    }

    let input: ContextSeedInput = match serde_json::from_value(params) {
        Ok(i) => i,
        Err(e) => return json!({"error": e.to_string()}),
    };

    if input.facts.is_empty() {
        return json!({"error": "facts must have at least 1 item"});
    }

    fn norm_tag(tag: &str) -> String {
        tag.trim()
            .trim_start_matches('#')
            .replace(' ', "_")
            .to_lowercase()
    }

    fn norm_entity(entity: &str) -> Option<String> {
        let e = entity.trim();
        if e.is_empty() || e.eq_ignore_ascii_case("general") {
            return None;
        }
        Some(format!("entity:{}", e.replace(' ', "_").to_lowercase()))
    }

    fn clamp_confidence(val: Option<f32>) -> f32 {
        val.unwrap_or(0.7).clamp(0.0, 1.0)
    }

    fn ttl_for_confidence(confidence: f32) -> Option<i64> {
        if confidence >= 0.85 {
            None
        } else if confidence >= 0.6 {
            Some(90 * 24 * 60 * 60)
        } else {
            Some(30 * 24 * 60 * 60)
        }
    }

    let mut entity_context = input
        .entity_context
        .unwrap_or_else(|| "General".to_string());
    if entity_context.len() > 200 {
        entity_context.truncate(200);
    }
    let entity_tag = norm_entity(&entity_context);
    let base_tags: Vec<String> = input
        .base_tags
        .unwrap_or_default()
        .iter()
        .map(|t| norm_tag(t))
        .filter(|t| !t.is_empty())
        .collect();
    let ttl_override = input.ttl_seconds;
    let disable_ttl = input.disable_ttl.unwrap_or(false);

    let mut inputs = Vec::with_capacity(input.facts.len());

    for fact in input.facts {
        let content = fact.content.trim().to_string();
        if content.is_empty() {
            continue;
        }

        let category = fact.category.unwrap_or_else(|| "fact".to_string());
        let confidence = clamp_confidence(fact.confidence);
        let ttl_seconds = if disable_ttl {
            None
        } else if let Some(ttl) = ttl_override {
            if ttl <= 0 {
                None
            } else {
                Some(ttl)
            }
        } else {
            ttl_for_confidence(confidence)
        };
        let (tier, ttl) = if let Some(ttl) = ttl_seconds {
            (MemoryTier::Daily, Some(ttl))
        } else {
            (MemoryTier::Permanent, None)
        };

        let rich_content = if entity_context.eq_ignore_ascii_case("General") {
            content.clone()
        } else {
            format!("[{}] {}", entity_context.trim(), content)
        };

        let mut tags = base_tags.clone();
        tags.push("origin:seed".to_string());
        tags.push("status:unverified".to_string());
        tags.push(format!("category:{}", norm_tag(&category)));
        tags.push(format!("confidence:{:.2}", confidence));
        if let Some(et) = &entity_tag {
            tags.push(et.clone());
        }
        tags.sort();
        tags.dedup();

        let mut metadata: HashMap<String, Value> = HashMap::new();
        metadata.insert("origin".to_string(), json!("seed"));
        metadata.insert("status".to_string(), json!("unverified"));
        metadata.insert("confidence".to_string(), json!(confidence));
        metadata.insert("entity_context".to_string(), json!(entity_context));
        metadata.insert("category".to_string(), json!(category));
        metadata.insert(
            "seeded_at".to_string(),
            json!(chrono::Utc::now().to_rfc3339()),
        );

        inputs.push(CreateMemoryInput {
            content: rich_content,
            memory_type: MemoryType::Context,
            tags,
            metadata,
            importance: None,
            scope: MemoryScope::Global,
            workspace: input.workspace.clone(),
            tier,
            defer_embedding: false,
            ttl_seconds: ttl,
            dedup_mode: DedupMode::Allow,
            dedup_threshold: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            summary_of_id: None,
            media_url: None,
        });
    }

    if inputs.is_empty() {
        return ToolError::invalid_params("facts must contain at least one non-empty content")
            .into_value();
    }

    let total_facts = inputs.len() as u64;
    ctx.reporter().step(
        0,
        total_facts,
        format!("Starting context seed of {total_facts} facts"),
    );

    let result = ctx
        .storage
        .with_transaction(|conn| create_memory_batch(conn, &inputs));

    match result {
        Ok(batch) => {
            ctx.reporter().complete(
                total_facts,
                format!(
                    "Context seed completed: {} created, {} failed",
                    batch.total_created, batch.total_failed
                ),
            );

            let _ = ctx.storage.with_connection(|conn| {
                for memory in &batch.created {
                    if let Ok(emb) = ctx.embedder.embed(&memory.content) {
                        let mut bytes = Vec::with_capacity(emb.len() * 4);
                        for f in &emb {
                            bytes.extend_from_slice(&f.to_le_bytes());
                        }
                        let _ = conn.execute(
                            "INSERT OR REPLACE INTO embeddings (memory_id, embedding, model, dimensions, created_at)
                             VALUES (?1, ?2, 'default', ?3, datetime('now'))",
                            rusqlite::params![memory.id, bytes, emb.len()],
                        );
                        let _ = conn.execute(
                            "UPDATE memories SET has_embedding = 1 WHERE id = ?",
                            rusqlite::params![memory.id],
                        );
                        ctx.hnsw_index.write().insert(memory.id, &emb);
                    }
                }
                Ok(())
            });

            ctx.search_cache
                .invalidate_for_workspace(input.workspace.as_deref());

            {
                let mut fuzzy = ctx.fuzzy_engine.lock();
                for memory in &batch.created {
                    fuzzy.add_to_vocabulary(&memory.content);
                }
            }

            for memory in &batch.created {
                if let Some(ref manager) = ctx.realtime {
                    manager.broadcast(RealtimeEvent::memory_created(
                        memory.id,
                        memory.content.clone(),
                        memory.workspace.clone(),
                    ));
                }
            }

            json!({
                "status": "success",
                "seeded_count": batch.total_created,
                "memory_ids": batch.created.iter().map(|m| m.id).collect::<Vec<_>>(),
                "entity": if entity_context.is_empty() { "General" } else { entity_context.as_str() },
                "failed": batch.failed
            })
        }
        Err(e) => ToolError::from(e).into_value(),
    }
}

pub fn memory_create_daily(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::create_memory;

    let content = match params.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return ToolError::missing_argument("content").into_value(),
    };

    let memory_type = params
        .get("type")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .unwrap_or(MemoryType::Note);

    let tags: Vec<String> = params
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let metadata: std::collections::HashMap<String, serde_json::Value> = params
        .get("metadata")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    let importance = params
        .get("importance")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32);

    let ttl_seconds = params
        .get("ttl_seconds")
        .and_then(|v| v.as_i64())
        .unwrap_or(86400);

    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .map(String::from);

    let input = CreateMemoryInput {
        content,
        memory_type,
        tags,
        metadata,
        importance,
        scope: Default::default(),
        workspace,
        tier: MemoryTier::Daily,
        defer_embedding: false,
        ttl_seconds: Some(ttl_seconds),
        dedup_mode: Default::default(),
        dedup_threshold: None,
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern: None,
        summary_of_id: None,
        media_url: None,
    };

    ctx.storage
        .with_connection(|conn| {
            let memory = create_memory(conn, &input)?;
            Ok(json!(memory))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

pub fn memory_create_episodic(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::create_memory;
    use chrono::DateTime;

    let content = match params.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return ToolError::missing_argument("content").into_value(),
    };

    let event_time = match params.get("event_time").and_then(|v| v.as_str()) {
        Some(s) => match DateTime::parse_from_rfc3339(s) {
            Ok(dt) => Some(dt.with_timezone(&chrono::Utc)),
            Err(e) => {
                return ToolError::invalid_params(format!("Invalid event_time format: {}", e))
                    .into_value()
            }
        },
        None => return ToolError::missing_argument("event_time").into_value(),
    };

    let event_duration_seconds = params
        .get("event_duration_seconds")
        .and_then(|v| v.as_i64());
    let tags: Vec<String> = params
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let metadata: std::collections::HashMap<String, Value> = params
        .get("metadata")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    let importance = params
        .get("importance")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32);
    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .map(String::from);

    let input = CreateMemoryInput {
        content,
        memory_type: MemoryType::Episodic,
        tags,
        metadata,
        importance,
        scope: MemoryScope::Global,
        workspace,
        tier: MemoryTier::Permanent,
        defer_embedding: false,
        ttl_seconds: None,
        dedup_mode: DedupMode::Allow,
        dedup_threshold: None,
        event_time,
        event_duration_seconds,
        trigger_pattern: None,
        summary_of_id: None,
        media_url: None,
    };

    ctx.storage
        .with_transaction(|conn| {
            let memory = create_memory(conn, &input)?;
            Ok(json!(memory))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

pub fn memory_create_procedural(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::create_memory;

    let content = match params.get("content").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return ToolError::missing_argument("content").into_value(),
    };

    let trigger_pattern = match params.get("trigger_pattern").and_then(|v| v.as_str()) {
        Some(p) => Some(p.to_string()),
        None => return ToolError::missing_argument("trigger_pattern").into_value(),
    };

    let tags: Vec<String> = params
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let metadata: std::collections::HashMap<String, Value> = params
        .get("metadata")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default();
    let importance = params
        .get("importance")
        .and_then(|v| v.as_f64())
        .map(|f| f as f32);
    let workspace = params
        .get("workspace")
        .and_then(|v| v.as_str())
        .map(String::from);

    let input = CreateMemoryInput {
        content,
        memory_type: MemoryType::Procedural,
        tags,
        metadata,
        importance,
        scope: MemoryScope::Global,
        workspace,
        tier: MemoryTier::Permanent,
        defer_embedding: false,
        ttl_seconds: None,
        dedup_mode: DedupMode::Allow,
        dedup_threshold: None,
        event_time: None,
        event_duration_seconds: None,
        trigger_pattern,
        summary_of_id: None,
        media_url: None,
    };

    ctx.storage
        .with_transaction(|conn| {
            let memory = create_memory(conn, &input)?;
            Ok(json!(memory))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

pub fn memory_create_section(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::create_section_memory;

    let title = match params.get("title").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => return ToolError::missing_argument("title").into_value(),
    };

    let content = params.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let parent_id = params.get("parent_id").and_then(|v| v.as_i64());
    let level = params.get("level").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
    let workspace = params.get("workspace").and_then(|v| v.as_str());

    ctx.storage
        .with_connection(|conn| {
            let memory = create_section_memory(conn, title, content, parent_id, level, workspace)?;
            Ok(json!(memory))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

pub fn memory_create_batch(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::create_memory;
    use crate::storage::queries::{BatchCreateResult, BatchError};

    let memories = match params.get("memories").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return ToolError::missing_argument("memories").into_value(),
    };

    if memories.is_empty() {
        return ToolError::invalid_params("memories array cannot be empty").into_value();
    }

    let total = memories.len() as u64;
    ctx.reporter().step(
        0,
        total,
        format!("Starting batch creation of {total} memories"),
    );

    let result = ctx.storage.with_transaction(|conn| {
        let mut created = Vec::new();
        let mut failed = Vec::new();

        for (index, m) in memories.iter().enumerate() {
            let step_num = (index + 1) as u64;
            let input: CreateMemoryInput = match serde_json::from_value(m.clone()) {
                Ok(inp) => inp,
                Err(e) => {
                    ctx.reporter().step(
                        step_num,
                        total,
                        format!("Failed deserializing batch item {step_num}/{total}: {e}"),
                    );
                    failed.push(BatchError {
                        index,
                        id: None,
                        error: format!("Invalid memory input: {e}"),
                    });
                    continue;
                }
            };

            match create_memory(conn, &input) {
                Ok(memory) => {
                    ctx.reporter().step(
                        step_num,
                        total,
                        format!("Created batch memory {step_num}/{total}"),
                    );
                    created.push(memory);
                }
                Err(e) => {
                    ctx.reporter().step(
                        step_num,
                        total,
                        format!("Failed creating batch item {step_num}/{total}: {e}"),
                    );
                    failed.push(BatchError {
                        index,
                        id: None,
                        error: e.to_string(),
                    });
                }
            }
        }

        Ok(BatchCreateResult {
            total_created: created.len(),
            total_failed: failed.len(),
            created,
            failed,
        })
    });

    match result {
        Ok(batch) => {
            let _ = ctx.storage.with_connection(|conn| {
                for memory in &batch.created {
                    if let Ok(emb) = ctx.embedder.embed(&memory.content) {
                        let mut bytes = Vec::with_capacity(emb.len() * 4);
                        for f in &emb {
                            bytes.extend_from_slice(&f.to_le_bytes());
                        }
                        let _ = conn.execute(
                            "INSERT OR REPLACE INTO embeddings (memory_id, embedding, model, dimensions, created_at)
                             VALUES (?1, ?2, 'default', ?3, datetime('now'))",
                            rusqlite::params![memory.id, bytes, emb.len()],
                        );
                        let _ = conn.execute(
                            "UPDATE memories SET has_embedding = 1 WHERE id = ?",
                            rusqlite::params![memory.id],
                        );
                        ctx.hnsw_index.write().insert(memory.id, &emb);
                    }
                }
                Ok(())
            });

            ctx.reporter().complete(
                total,
                format!(
                    "Batch creation completed: {} created, {} failed",
                    batch.total_created, batch.total_failed
                ),
            );
            json!(batch)
        }
        Err(e) => ToolError::from(e).into_value(),
    }
}
