//! Summarization, full-memory retrieval, context budget, and archival handlers.

use serde_json::{json, Value};

use super::HandlerContext;
use crate::storage::enrichment_events::{emit_best_effort, latest_version_id, EnrichmentEvent};

pub fn memory_summarize(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::{create_memory, get_memory};
    use crate::types::{CreateMemoryInput, MemoryTier, MemoryType};

    let memory_ids: Vec<i64> = match params.get("memory_ids") {
        Some(v) => match serde_json::from_value(v.clone()) {
            Ok(ids) => ids,
            Err(e) => return json!({"error": format!("Invalid memory_ids: {}", e)}),
        },
        None => return json!({"error": "memory_ids is required"}),
    };

    if memory_ids.is_empty() {
        return json!({"error": "memory_ids cannot be empty"});
    }

    let provided_summary = params.get("summary").and_then(|v| v.as_str());
    let max_length = params
        .get("max_length")
        .and_then(|v| v.as_u64())
        .unwrap_or(500) as usize;
    let workspace = params.get("workspace").and_then(|v| v.as_str());
    let tags: Option<Vec<String>> = params
        .get("tags")
        .and_then(|v| serde_json::from_value(v.clone()).ok());

    let tx_result = ctx.storage.with_transaction(|conn| {
        let mut contents: Vec<String> = Vec::with_capacity(memory_ids.len());
        let mut first_memory_workspace: Option<String> = None;

        for id in &memory_ids {
            match get_memory(conn, *id) {
                Ok(mem) => {
                    contents.push(mem.content);
                    if first_memory_workspace.is_none() {
                        first_memory_workspace = Some(mem.workspace);
                    }
                }
                Err(e) => {
                    return Err(crate::error::EngramError::Internal(format!(
                        "Memory {} not found: {}",
                        id, e
                    )));
                }
            }
        }

        let summary_text = if let Some(s) = provided_summary {
            s.to_string()
        } else {
            let combined = contents.join("\n\n---\n\n");
            if combined.len() <= max_length {
                combined
            } else {
                let head_len = (max_length as f64 * 0.6) as usize;
                let tail_len = (max_length as f64 * 0.3) as usize;
                let head: String = combined.chars().take(head_len).collect();
                let tail: String = combined
                    .chars()
                    .rev()
                    .take(tail_len)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
                let truncated = combined.len() - head_len - tail_len;
                format!("{}...[{} chars truncated]...{}", head, truncated, tail)
            }
        };

        let input = CreateMemoryInput {
            content: summary_text,
            memory_type: MemoryType::Summary,
            importance: Some(0.6),
            tags: tags.unwrap_or_default(),
            workspace: workspace.map(|s| s.to_string()).or(first_memory_workspace),
            tier: MemoryTier::Permanent,
            summary_of_id: Some(memory_ids[0]),
            ..Default::default()
        };

        let memory = create_memory(conn, &input)?;

        // Return data needed for the post-commit emit.
        Ok((memory.id, memory.workspace, memory.content.len()))
    });

    match tx_result {
        Ok((summary_id, summary_workspace, summary_len)) => {
            // Emit SUCCESS event in a separate connection, outside the now-committed transaction.
            let operation_id = uuid::Uuid::new_v4().to_string();
            ctx.storage
                .with_connection(|conn| {
                    let vid = latest_version_id(conn, summary_id).unwrap_or(None);
                    emit_best_effort(
                        conn,
                        &EnrichmentEvent {
                            operation_id: &operation_id,
                            event_type: "consolidation",
                            memory_id: Some(summary_id),
                            version_id: vid,
                            triggered_by: "memory_summarize",
                            agent_id: None,
                            workspace: Some(summary_workspace.as_str()),
                            params: serde_json::json!({"source_count": memory_ids.len()}),
                            outcome: serde_json::json!({"summary_id": summary_id}),
                            status: "completed",
                            dry_run: false,
                        },
                    );
                    Ok::<_, crate::error::EngramError>(())
                })
                .ok();
            json!({
                "id": summary_id,
                "memory_type": "summary",
                "summarized_count": memory_ids.len(),
                "original_ids": memory_ids,
                "summary_length": summary_len
            })
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}

pub fn memory_get_full(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::get_memory;
    use crate::types::MemoryType;

    let id = match params.get("id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "id is required"}),
    };

    ctx.storage
        .with_connection(|conn| {
            let memory = match get_memory(conn, id) {
                Ok(m) => m,
                Err(_) => return Ok(json!({"error": "Memory not found"})),
            };

            if memory.memory_type == MemoryType::Summary {
                if let Some(original_id) = memory.summary_of_id {
                    match get_memory(conn, original_id) {
                        Ok(original) => {
                            return Ok(json!({
                                "id": id,
                                "is_summary": true,
                                "original_id": original_id,
                                "original_content": original.content,
                                "summary_content": memory.content
                            }));
                        }
                        Err(_) => {
                            return Ok(json!({
                                "error": "original_deleted",
                                "id": id,
                                "original_id": original_id,
                                "summary": memory.content
                            }));
                        }
                    }
                }
            }

            Ok(json!({
                "id": id,
                "is_summary": false,
                "content": memory.content
            }))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn context_budget_check(ctx: &HandlerContext, params: Value) -> Value {
    use crate::intelligence::compression::check_context_budget;
    use crate::storage::queries::get_memory;

    let memory_ids: Vec<i64> = match params.get("memory_ids") {
        Some(v) => match serde_json::from_value(v.clone()) {
            Ok(ids) => ids,
            Err(e) => return json!({"error": format!("Invalid memory_ids: {}", e)}),
        },
        None => return json!({"error": "memory_ids is required"}),
    };

    let model = match params.get("model").and_then(|v| v.as_str()) {
        Some(m) => m,
        None => return json!({"error": "model is required"}),
    };

    let encoding = params.get("encoding").and_then(|v| v.as_str());

    let budget = match params.get("budget").and_then(|v| v.as_u64()) {
        Some(b) => b as usize,
        None => return json!({"error": "budget is required"}),
    };

    ctx.storage
        .with_connection(|conn| {
            let mut contents: Vec<(i64, String)> = Vec::with_capacity(memory_ids.len());

            for id in &memory_ids {
                match get_memory(conn, *id) {
                    Ok(mem) => contents.push((*id, mem.content)),
                    Err(_) => return Ok(json!({"error": format!("Memory {} not found", id)})),
                }
            }

            match check_context_budget(&contents, model, encoding, budget) {
                Ok(result) => Ok(json!(result)),
                Err(e) => Ok(json!({"error": e.to_string()})),
            }
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

pub fn memory_archive_old(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::{create_memory, list_memories};
    use crate::types::{CreateMemoryInput, LifecycleState, ListOptions, MemoryTier, MemoryType};
    use chrono::{Duration, Utc};

    let max_age_days = params
        .get("max_age_days")
        .and_then(|v| v.as_i64())
        .unwrap_or(90);
    let max_importance = params
        .get("max_importance")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.5) as f32;
    let min_access_count = params
        .get("min_access_count")
        .and_then(|v| v.as_i64())
        .unwrap_or(5);
    let workspace = params.get("workspace").and_then(|v| v.as_str());
    let dry_run = params
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let cutoff_date = Utc::now() - Duration::days(max_age_days);

    ctx.storage
        .with_connection(|conn| {
            let options = ListOptions {
                workspace: workspace.map(|s| s.to_string()),
                limit: Some(1000),
                include_archived: true,
                ..Default::default()
            };

            let all_memories = list_memories(conn, &options)?;

            let mut candidates = Vec::new();
            for memory in all_memories {
                if memory.created_at < cutoff_date
                    && memory.importance <= max_importance
                    && memory.access_count < min_access_count as i32
                    && memory.lifecycle_state == LifecycleState::Archived
                    && memory.memory_type != MemoryType::Summary
                    && memory.memory_type != MemoryType::Checkpoint
                    && !memory_has_live_summary(conn, memory.id)?
                {
                    candidates.push(memory);
                }
            }

            if dry_run {
                let summaries: Vec<_> = candidates
                    .iter()
                    .map(|m| {
                        json!({
                            "id": m.id,
                            "memory_type": m.memory_type,
                            "importance": m.importance,
                            "access_count": m.access_count,
                            "created_at": m.created_at.to_rfc3339(),
                            "content_preview": m.content.chars().take(100).collect::<String>()
                        })
                    })
                    .collect();

                return Ok(json!({
                    "dry_run": true,
                    "would_compress": candidates.len(),
                    "candidates": summaries
                }));
            }

            let mut compressed = 0;
            let mut errors: Vec<String> = Vec::new();
            let operation_id = uuid::Uuid::new_v4().to_string();

            for memory in candidates {
                let summary_text = if memory.content.len() > 200 {
                    let head: String = memory.content.chars().take(120).collect();
                    let tail: String = memory
                        .content
                        .chars()
                        .rev()
                        .take(60)
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect();
                    format!("{}...{}", head, tail)
                } else {
                    memory.content.clone()
                };

                let input = CreateMemoryInput {
                    content: format!("[Compressed {:?}] {}", memory.memory_type, summary_text),
                    memory_type: MemoryType::Summary,
                    importance: Some(memory.importance),
                    tags: memory.tags.clone(),
                    workspace: Some(memory.workspace.clone()),
                    tier: MemoryTier::Permanent,
                    summary_of_id: Some(memory.id),
                    ..Default::default()
                };

                match create_memory(conn, &input) {
                    Ok(_) => {
                        compressed += 1;
                        emit_best_effort(
                            conn,
                            &EnrichmentEvent {
                                operation_id: &operation_id,
                                event_type: "compression",
                                memory_id: Some(memory.id),
                                version_id: None,
                                triggered_by: "memory_archive_old",
                                agent_id: None,
                                workspace: Some(memory.workspace.as_str()),
                                params: json!({}),
                                outcome: json!({"compressed": true, "summary_created": true}),
                                status: "completed",
                                dry_run: false,
                            },
                        );
                    }
                    Err(e) => errors.push(format!("Memory {}: {}", memory.id, e)),
                }
            }

            Ok(json!({
                "dry_run": false,
                "compressed": compressed,
                "errors": errors
            }))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

fn memory_has_live_summary(
    conn: &rusqlite::Connection,
    memory_id: i64,
) -> crate::error::Result<bool> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*)
         FROM memories
         WHERE summary_of_id = ?1
           AND memory_type = 'summary'
           AND valid_to IS NULL",
        rusqlite::params![memory_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

#[cfg(test)]
mod summarize_tests {
    use super::*;

    fn test_ctx() -> super::super::HandlerContext {
        use crate::embedding::{create_embedder, EmbeddingCache};
        use crate::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
        use crate::storage::Storage;
        use crate::types::EmbeddingConfig;
        use parking_lot::Mutex;
        use std::sync::Arc;

        let storage = Storage::open_in_memory().expect("in-memory storage");
        let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
        super::super::HandlerContext {
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

    fn seed_memory(ctx: &super::super::HandlerContext, content: &str) -> i64 {
        use crate::storage::queries::create_memory;
        use crate::types::{CreateMemoryInput, MemoryTier, MemoryType};
        ctx.storage
            .with_transaction(|conn| {
                create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: content.to_string(),
                        memory_type: MemoryType::Note,
                        workspace: Some("default".to_string()),
                        tier: MemoryTier::Permanent,
                        ..Default::default()
                    },
                )
            })
            .expect("seed failed")
            .id
    }

    fn seed_old_memory(
        ctx: &super::super::HandlerContext,
        content: &str,
        lifecycle_state: &str,
    ) -> i64 {
        use chrono::{Duration, Utc};

        let id = seed_memory(ctx, content);
        let old_ts = (Utc::now() - Duration::days(120)).to_rfc3339();
        let target_state = lifecycle_state;
        let target_id = id;
        ctx.storage
            .with_transaction(|conn| {
                conn.execute(
                    "UPDATE memories
                     SET created_at = ?1,
                         updated_at = ?1,
                         last_accessed_at = ?1,
                         importance = 0.0,
                         access_count = 0,
                         lifecycle_state = ?2
                     WHERE id = ?3",
                    rusqlite::params![old_ts, target_state, target_id],
                )?;
                Ok(())
            })
            .expect("age memory");
        id
    }

    fn lifecycle_state(ctx: &super::super::HandlerContext, id: i64) -> String {
        ctx.storage
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT lifecycle_state FROM memories WHERE id = ?1",
                    [id],
                    |row| row.get::<_, String>(0),
                )?)
            })
            .expect("query lifecycle state")
    }

    fn summary_count_for(ctx: &super::super::HandlerContext, id: i64) -> i64 {
        ctx.storage
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM memories
                     WHERE summary_of_id = ?1 AND memory_type = 'summary' AND valid_to IS NULL",
                    [id],
                    |row| row.get(0),
                )?)
            })
            .expect("query summary count")
    }

    #[test]
    fn test_memory_summarize_emits_enrichment_event() {
        let ctx = test_ctx();

        let id1 = seed_memory(&ctx, "Memory alpha about Rust ownership");
        let id2 = seed_memory(&ctx, "Memory beta about Rust lifetimes");

        let result = memory_summarize(
            &ctx,
            serde_json::json!({
                "memory_ids": [id1, id2],
                "summary": "Rust ownership and lifetimes overview",
                "workspace": "default"
            }),
        );

        assert!(
            result.get("error").is_none(),
            "memory_summarize returned error: {:?}",
            result
        );
        assert!(
            result["id"].as_i64().is_some(),
            "expected summary memory id"
        );

        // Verify an enrichment_events row was written
        let count: i64 = ctx
            .storage
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM enrichment_events \
                     WHERE event_type = 'consolidation' \
                       AND triggered_by = 'memory_summarize'",
                    [],
                    |row| row.get(0),
                )?)
            })
            .expect("query failed");

        assert_eq!(count, 1, "expected exactly 1 enrichment_events row");
    }

    #[test]
    fn test_memory_archive_old_does_not_compress_active_rows() {
        let ctx = test_ctx();
        let id = seed_old_memory(&ctx, "old active memory", "active");

        let result = memory_archive_old(
            &ctx,
            serde_json::json!({
                "dry_run": false,
                "workspace": "default",
                "max_age_days": 90,
                "max_importance": 0.5,
                "min_access_count": 5
            }),
        );

        assert!(result.get("error").is_none(), "unexpected error: {result}");
        assert_eq!(result["compressed"].as_u64(), Some(0), "{result}");
        assert_eq!(summary_count_for(&ctx, id), 0);
        assert_eq!(lifecycle_state(&ctx, id), "active");
    }

    #[test]
    fn test_memory_archive_old_compresses_already_archived_rows() {
        let ctx = test_ctx();
        let id = seed_old_memory(&ctx, "old archived memory", "archived");

        let result = memory_archive_old(
            &ctx,
            serde_json::json!({
                "dry_run": false,
                "workspace": "default",
                "max_age_days": 90,
                "max_importance": 0.5,
                "min_access_count": 5
            }),
        );

        assert!(result.get("error").is_none(), "unexpected error: {result}");
        assert_eq!(result["compressed"].as_u64(), Some(1), "{result}");
        assert_eq!(summary_count_for(&ctx, id), 1);
        assert_eq!(lifecycle_state(&ctx, id), "archived");
    }

    #[test]
    fn test_memory_archive_old_is_idempotent_for_archived_rows() {
        let ctx = test_ctx();
        let id = seed_old_memory(&ctx, "old archived idempotent memory", "archived");

        let args = serde_json::json!({
            "dry_run": false,
            "workspace": "default",
            "max_age_days": 90,
            "max_importance": 0.5,
            "min_access_count": 5
        });
        let first = memory_archive_old(&ctx, args.clone());
        let second = memory_archive_old(&ctx, args);

        assert!(
            first.get("error").is_none(),
            "unexpected first error: {first}"
        );
        assert!(
            second.get("error").is_none(),
            "unexpected second error: {second}"
        );
        assert_eq!(first["compressed"].as_u64(), Some(1), "{first}");
        assert_eq!(second["compressed"].as_u64(), Some(0), "{second}");
        assert_eq!(summary_count_for(&ctx, id), 1);
        assert_eq!(lifecycle_state(&ctx, id), "archived");
    }
}
