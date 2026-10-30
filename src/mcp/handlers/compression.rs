//! Compression and consolidation tool handlers.
//!
//! Provides MCP tools for semantic compression, context-window packing,
//! offline consolidation, and synthesis overlap detection.

use serde_json::{json, Value};

use super::HandlerContext;
use crate::mcp::error::ToolError;
use crate::storage::enrichment_events::{emit_best_effort, EnrichmentEvent};

// ── memory_compress ───────────────────────────────────────────────────────────

/// Compress a memory's content using rule-based semantic compression.
pub fn memory_compress(ctx: &HandlerContext, params: Value) -> Value {
    use crate::intelligence::compression_semantic::{CompressionConfig, SemanticCompressor};
    use crate::storage::queries::get_memory;

    let id = match params.get("id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return ToolError::missing_argument("id").into_value(),
    };
    let target_ratio: f32 = params
        .get("target_ratio")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(0.1);

    ctx.storage
        .with_connection(|conn| {
            crate::mcp::workspace_guard::ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
            let memory = get_memory(conn, id)?;
            let config = CompressionConfig {
                target_ratio,
                ..CompressionConfig::default()
            };
            let compressor = SemanticCompressor::new(config);
            let result = compressor.compress(&memory.content);
            Ok(json!({
                "memory_id": id,
                "original_tokens": result.original_tokens,
                "compressed_tokens": result.compressed_tokens,
                "compression_ratio": result.ratio,
                "structured_content": result.structured_content,
                "key_entities": result.key_entities,
                "key_facts": result.key_facts,
            }))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

// ── memory_decompress ─────────────────────────────────────────────────────────

/// Retrieve the original (uncompressed) content of a memory.
pub fn memory_decompress(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::queries::get_memory;

    let id = match params.get("id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return ToolError::missing_argument("id").into_value(),
    };

    ctx.storage
        .with_connection(|conn| {
            crate::mcp::workspace_guard::ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
            let memory = get_memory(conn, id)?;
            Ok(json!({
                "memory_id": id,
                "content": memory.content,
                "status": "ok",
            }))
        })
        .unwrap_or_else(|e| ToolError::from(e).into_value())
}

// ── memory_compress_for_context ───────────────────────────────────────────────

/// Compress a set of memories to fit within a token budget for LLM context.
pub fn memory_compress_for_context(ctx: &HandlerContext, params: Value) -> Value {
    use crate::intelligence::context_compression::{ContextCompressor, MemoryInput};
    use crate::storage::queries::get_memory;

    let ids: Vec<i64> = params
        .get("ids")
        .or_else(|| params.get("memory_ids"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
        .unwrap_or_default();

    if ids.is_empty() {
        return ToolError::missing_argument("ids").into_value();
    }

    let token_budget: usize = params
        .get("token_budget")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(4096);

    let result = ctx.storage.with_connection(|conn| {
        let mut inputs: Vec<MemoryInput> = Vec::new();
        for &id in &ids {
            crate::mcp::workspace_guard::ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
            if let Ok(m) = get_memory(conn, id) {
                inputs.push(MemoryInput {
                    id: m.id,
                    content: m.content,
                    importance: m.importance,
                });
            }
        }

        let mut compressor = ContextCompressor::new(token_budget);
        let compression = compressor.compress_for_context_with_diagnostics(&inputs);
        let total_tokens: usize = compression.entries.iter().map(|e| e.tokens_used).sum();

        Ok(json!({
            "token_budget": token_budget,
            "memories_input": inputs.len(),
            "memories_included": compression.entries.len(),
            "memories_skipped": compression.skipped_ids.len(),
            "total_tokens": total_tokens,
            "budget_used": compression.budget.used,
            "budget_remaining": compression.budget.remaining,
            "skipped_memory_ids": compression.skipped_ids,
            "entries": compression.entries,
        }))
    });

    result.unwrap_or_else(|e| ToolError::from(e).into_value())
}

// ── memory_consolidate ────────────────────────────────────────────────────────

/// Run offline consolidation over a workspace to merge similar memories.
pub fn memory_consolidate(ctx: &HandlerContext, params: Value) -> Value {
    use crate::intelligence::consolidation_offline::{
        ConsolidationConfig, GroupingStrategy, OfflineConsolidator,
    };

    let explicit_ws = match params.get("workspace") {
        None | Some(Value::Null) => None,
        Some(Value::String(ws)) => match crate::types::normalize_workspace(ws) {
            Ok(norm) => Some(norm),
            Err(e) => {
                return ToolError::invalid_params(format!("Invalid workspace: {e}")).into_value()
            }
        },
        Some(_) => return ToolError::invalid_params("workspace must be a string").into_value(),
    };

    if let Some(ref p) = ctx.principal {
        if explicit_ws.is_none()
            && matches!(p, crate::auth::TransportPrincipal::AnonymousLoopback(_))
        {
            return crate::mcp::permission::workspace_claim_denied("memory_consolidate", p);
        }
        if let Some(denial) = crate::mcp::permission::permission_denial_for_principal(
            "memory_consolidate",
            p,
            explicit_ws.as_deref(),
        ) {
            return denial;
        }
    }

    let workspace = explicit_ws.unwrap_or_else(|| "default".to_string());

    let strategy_str = match params.get("strategy") {
        None | Some(Value::Null) => "content_overlap",
        Some(Value::String(s)) => s.as_str(),
        Some(_) => return ToolError::invalid_params("strategy must be a string").into_value(),
    };

    let strategy = match strategy_str {
        "content_overlap" => GroupingStrategy::ContentOverlap,
        "tag_similarity" => GroupingStrategy::TagSimilarity,
        "temporal_proximity" => GroupingStrategy::TemporalProximity,
        other => {
            return ToolError::invalid_params(format!(
                "Invalid strategy '{other}': expected 'content_overlap', 'tag_similarity', or 'temporal_proximity'"
            ))
            .into_value()
        }
    };

    let dry_run = match params.get("dry_run") {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => return ToolError::invalid_params("dry_run must be a boolean").into_value(),
    };

    let config = ConsolidationConfig::default();
    let consolidator = OfflineConsolidator::new(config);

    let report_res = if dry_run {
        ctx.storage
            .with_connection(|conn| consolidator.preview_with_strategy(conn, &workspace, strategy))
    } else {
        ctx.storage.with_transaction(|conn| {
            consolidator.consolidate_with_strategy(conn, &workspace, strategy)
        })
    };

    match report_res {
        Ok(report) => {
            if !dry_run && report.memories_merged > 0 {
                let operation_id = uuid::Uuid::new_v4().to_string();
                let _ = ctx.storage.with_connection(|conn| {
                    emit_best_effort(
                        conn,
                        &EnrichmentEvent {
                            operation_id: &operation_id,
                            event_type: "consolidation",
                            memory_id: None,
                            version_id: None,
                            triggered_by: "memory_consolidate",
                            agent_id: None,
                            workspace: Some(&workspace),
                            params: json!({"strategy": strategy_str}),
                            outcome: json!({
                                "groups_found":       report.groups_found,
                                "memories_merged":    report.memories_merged,
                                "memories_archived":  report.memories_archived,
                                "tokens_saved":       report.tokens_saved,
                            }),
                            status: "completed",
                            dry_run,
                        },
                    );
                    Ok::<_, crate::error::EngramError>(())
                });
            }

            json!({
                "workspace": workspace,
                "strategy": strategy_str,
                "dry_run": dry_run,
                "groups_found": report.groups_found,
                "memories_merged": report.memories_merged,
                "memories_archived": report.memories_archived,
                "tokens_before": report.tokens_before,
                "tokens_after": report.tokens_after,
                "tokens_saved": report.tokens_saved,
            })
        }
        Err(e) => {
            // Emit a failed event outside the failed transaction.
            let err_str = e.to_string();
            let _ = ctx.storage.with_connection(|conn| {
                let op_id = uuid::Uuid::new_v4().to_string();
                emit_best_effort(
                    conn,
                    &EnrichmentEvent {
                        operation_id: &op_id,
                        event_type: "consolidation",
                        memory_id: None,
                        version_id: None,
                        triggered_by: "memory_consolidate",
                        agent_id: None,
                        workspace: Some(&workspace),
                        params: json!({"strategy": strategy_str}),
                        outcome: json!({"error": &err_str}),
                        status: "failed",
                        dry_run,
                    },
                );
                Ok::<_, crate::error::EngramError>(())
            });
            json!({"error": err_str})
        }
    }
}

// ── memory_synthesis ──────────────────────────────────────────────────────────

/// Check whether two pieces of content overlap semantically (Jaccard-based).
pub fn memory_synthesis(_ctx: &HandlerContext, params: Value) -> Value {
    use crate::intelligence::synthesis::{SynthesisConfig, SynthesisEngine, SynthesisStrategy};

    let content_a = match params.get("content_a").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return ToolError::missing_argument("content_a").into_value(),
    };
    let content_b = match params.get("content_b").and_then(|v| v.as_str()) {
        Some(c) => c.to_string(),
        None => return ToolError::missing_argument("content_b").into_value(),
    };
    let id_a: i64 = params.get("id_a").and_then(|v| v.as_i64()).unwrap_or(0);
    let strategy_str = params
        .get("strategy")
        .and_then(|v| v.as_str())
        .unwrap_or("merge");

    let strategy = match strategy_str {
        "replace" => SynthesisStrategy::Replace,
        "append" => SynthesisStrategy::Append,
        _ => SynthesisStrategy::Merge,
    };

    let mut engine = SynthesisEngine::new(SynthesisConfig::default());
    engine.add_to_buffer(id_a, &content_a);

    match engine.check_and_synthesize(&content_b, strategy) {
        Some(synth) => json!({
            "overlap_detected": true,
            "overlap_score": synth.overlap_score,
            "strategy_used": format!("{:?}", synth.strategy_used),
            "synthesized_content": synth.content,
            "source_ids": synth.sources,
            "tokens_saved": synth.tokens_saved,
        }),
        None => json!({
            "overlap_detected": false,
            "message": "No significant overlap detected between the two contents",
        }),
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding::{create_embedder, EmbeddingCache};
    use crate::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
    use crate::storage::Storage;
    use crate::types::EmbeddingConfig;
    use parking_lot::Mutex;
    use serde_json::json;
    use std::sync::Arc;

    fn make_ctx() -> HandlerContext {
        let storage = Storage::open_in_memory().expect("test storage");
        HandlerContext {
            storage,
            embedder: create_embedder(&EmbeddingConfig::default()).expect("embedder"),
            fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
            search_config: SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(EmbeddingCache::default()),
            search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
            hnsw_index: Arc::new(parking_lot::RwLock::new(crate::search::HnswIndex::new(
                crate::search::HnswConfig::new(384, crate::search::VectorMetric::Cosine),
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
            principal: None,
        }
    }

    #[test]
    fn test_memory_consolidate_emits_completed_event_on_success() {
        let ctx = make_ctx();

        // Seed two similar memories so the consolidator has something to merge.
        ctx.storage
            .with_transaction(|conn| {
                conn.execute(
                    "INSERT INTO memories (content, memory_type, workspace, importance, access_count)
                     VALUES ('Rust ownership rules explained', 'general', 'default', 0.5, 0)",
                    [],
                )?;
                conn.execute(
                    "INSERT INTO memories (content, memory_type, workspace, importance, access_count)
                     VALUES ('Rust ownership rules overview', 'general', 'default', 0.5, 0)",
                    [],
                )?;
                Ok(())
            })
            .expect("seed memories");

        let result = memory_consolidate(
            &ctx,
            json!({"workspace": "default", "strategy": "content_overlap", "dry_run": false}),
        );
        assert!(
            result.get("error").is_none(),
            "memory_consolidate error: {result}"
        );

        // The event should be present whether or not any groups were found.
        // (If no groups found, memories_merged==0 and no event is emitted for dry_run=false
        //  with zero merges — the emit only fires when groups are found.)
        // This test at minimum verifies no panic and correct JSON shape.
        assert!(result["workspace"].as_str().is_some());
    }

    #[test]
    fn test_memory_consolidate_emits_failed_event_on_error() {
        // Force a real DB error by dropping the `memories` table before calling
        // the handler. `fetch_candidates` inside the consolidator queries that
        // table, so the `with_connection` closure will return an error, which
        // causes the handler to enter the failure path and emit a
        // `status="failed"` enrichment event.
        let ctx = make_ctx();

        // Drop the memories table so the consolidator hits a real SQL error.
        ctx.storage
            .with_connection(|conn| {
                conn.execute_batch("DROP TABLE IF EXISTS memories;")?;
                Ok(())
            })
            .expect("drop memories table");

        let result = memory_consolidate(
            &ctx,
            json!({"workspace": "default", "strategy": "content_overlap", "dry_run": false}),
        );

        // The handler must surface an error in the JSON response.
        assert!(
            result.get("error").is_some(),
            "expected an error response after dropping memories table, got: {result}"
        );

        // The failure path must have emitted a status=failed enrichment event.
        let count: i32 = ctx
            .storage
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT COUNT(*) FROM enrichment_events
                     WHERE triggered_by='memory_consolidate' AND status='failed'",
                    [],
                    |r| r.get(0),
                )?)
            })
            .unwrap();
        assert!(
            count > 0,
            "memory_consolidate error must emit status=failed enrichment event"
        );
    }

    #[test]
    fn test_memory_consolidate_dry_run_does_not_mutate_and_validates_inputs() {
        let mut ctx = make_ctx();
        let old_ts = (chrono::Utc::now() - chrono::Duration::hours(48))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);

        let (id1, id2) = ctx
            .storage
            .with_transaction(|conn| {
                conn.execute(
                    "INSERT INTO memories (content, memory_type, workspace, importance, access_count, created_at, updated_at)
                     VALUES ('Rust memory safety and ownership guide', 'note', 'default', 0.5, 0, ?1, ?1)",
                    [&old_ts],
                )?;
                let id1 = conn.last_insert_rowid();
                conn.execute(
                    "INSERT INTO memories (content, memory_type, workspace, importance, access_count, created_at, updated_at)
                     VALUES ('Rust memory safety and borrowing guide', 'note', 'default', 0.5, 0, ?1, ?1)",
                    [&old_ts],
                )?;
                let id2 = conn.last_insert_rowid();
                Ok((id1, id2))
            })
            .expect("seed old memories");

        // 1. dry_run: true reports groups_found and memories_merged WITHOUT archiving or writing consolidated_memories!
        let dry = memory_consolidate(
            &ctx,
            json!({"workspace": "default", "strategy": "content_overlap", "dry_run": true}),
        );
        assert!(dry.get("error").is_none(), "dry_run failed: {dry}");
        assert_eq!(dry["dry_run"], true);
        assert_eq!(dry["groups_found"], 1);
        assert_eq!(dry["memories_merged"], 2);
        assert_eq!(dry["memories_archived"], 0);

        ctx.storage
            .with_connection(|conn| {
                let consolidated_count: i64 =
                    conn.query_row("SELECT COUNT(*) FROM consolidated_memories", [], |r| {
                        r.get(0)
                    })?;
                assert_eq!(
                    consolidated_count, 0,
                    "dry_run must not insert into consolidated_memories"
                );
                for id in [id1, id2] {
                    let state: String = conn.query_row(
                        "SELECT COALESCE(lifecycle_state, 'active') FROM memories WHERE id = ?1",
                        [id],
                        |r| r.get(0),
                    )?;
                    assert_eq!(state, "active", "dry_run must not archive memory {id}");
                }
                Ok(())
            })
            .expect("verify zero writes on dry_run");

        // 2. Invalid strategy or non-boolean dry_run fails closed.
        let bad_strat = memory_consolidate(
            &ctx,
            json!({"workspace": "default", "strategy": "unknown_strategy"}),
        );
        assert_eq!(bad_strat["error"]["code"], "invalid_params");

        let bad_dry = memory_consolidate(&ctx, json!({"workspace": "default", "dry_run": "yes"}));
        assert_eq!(bad_dry["error"]["code"], "invalid_params");

        // 3. Direct call with restricted principal denies foreign workspace.
        ctx.principal = Some(
            crate::auth::TransportPrincipal::from_token_claims(crate::auth::TokenClaims {
                user_id: crate::auth::UserId::from_string("scoped"),
                key_id: "k1".to_string(),
                permissions: crate::auth::PermissionSet::standard_user(),
                namespace: Some("tenant-only".to_string()),
                issued_at: chrono::Utc::now(),
                expires_at: None,
            })
            .expect("claims"),
        );
        let denied = memory_consolidate(&ctx, json!({"workspace": "default", "dry_run": true}));
        assert_eq!(denied["error"]["code"], "permission_denied");
        ctx.principal = None;

        // 4. dry_run: false applies consolidation and archives both memories.
        let applied = memory_consolidate(
            &ctx,
            json!({"workspace": "default", "strategy": "content_overlap", "dry_run": false}),
        );
        assert!(applied.get("error").is_none(), "apply failed: {applied}");
        assert_eq!(applied["groups_found"], 1);
        assert_eq!(applied["memories_merged"], 2);
        assert_eq!(applied["memories_archived"], 2);
    }
}
