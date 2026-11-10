//! Retrieval-excellence tool handlers.
//!
//! Exposes embedding cache/provider management tools and the semantic query
//! cache that was introduced as part of Round 3 retrieval improvements.

use serde_json::{json, Value};

use super::HandlerContext;

// ── Semantic / search cache ───────────────────────────────────────────────────

/// Return hit/miss statistics for the semantic search cache.
pub fn memory_cache_stats(ctx: &HandlerContext, _params: Value) -> Value {
    let stats = ctx.search_cache.stats();
    json!({
        "hits": stats.hits,
        "misses": stats.misses,
        "entries": stats.entries,
        "hit_rate": stats.hit_rate
    })
}

/// Evict entries from the semantic search cache (optionally scoped to a workspace).
///
/// Every scope answers with the same shape, `{success, entries_cleared}`:
/// callers must not have to branch on whether they passed a workspace.
pub fn memory_cache_clear(ctx: &HandlerContext, params: Value) -> Value {
    match super::search::evict_search_cache("memory_cache_clear", &params, ctx) {
        Ok((_workspace, evicted)) => json!({
            "success": true,
            "entries_cleared": evicted
        }),
        Err(denial) => denial,
    }
}

// ── Embedding providers ───────────────────────────────────────────────────────

/// List all registered embedding providers in the registry.
pub fn memory_embedding_providers(ctx: &HandlerContext, _params: Value) -> Value {
    // Report the active provider.
    let model_name = ctx.embedder.model_name().to_string();
    let dimensions = ctx.embedder.dimensions();

    // The full registry is exposed via EmbeddingRegistry when multiple
    // providers are registered.  For now, report the active provider.
    json!({
        "active": {
            "id": model_name,
            "model": model_name,
            "dimensions": dimensions
        },
        "count": 1
    })
}

/// Count the memories a re-embedding migration would touch.
///
/// Only `dry_run` is implemented. Re-embedding is not: storing the new
/// vectors and rebuilding the index are missing, so a write call answers with
/// an error and changes nothing, rather than relabelling `embedding_model` on
/// vectors that were never recomputed. `memory_rebuild_embeddings` queues
/// every memory for re-embedding with the active model.
pub fn memory_embedding_migrate(ctx: &HandlerContext, params: Value) -> Value {
    let dry_run = params
        .get("dry_run")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let target_model = params
        .get("target_model")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| ctx.embedder.model_name().to_string());

    if !dry_run {
        return json!({
            "success": false,
            "error": "memory_embedding_migrate does not re-embed yet; nothing was changed. \
                      Use dry_run=true to count affected memories, or \
                      memory_rebuild_embeddings to queue every memory for re-embedding \
                      with the active model.",
            "migrated": 0,
            "target_model": target_model
        });
    }

    let count = ctx.storage.with_connection(|conn| {
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM memories WHERE has_embedding = 1",
            [],
            |row| row.get(0),
        )?;
        Ok(n)
    });

    match count {
        Ok(count) => json!({
            "dry_run": true,
            "memories_to_migrate": count,
            "target_model": target_model
        }),
        Err(e) => json!({"error": e.to_string()}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::queries::create_memory;
    use crate::types::CreateMemoryInput;
    use parking_lot::{Mutex, RwLock};
    use std::sync::Arc;

    fn test_context() -> HandlerContext {
        HandlerContext {
            storage: crate::Storage::open_in_memory().expect("in-memory storage"),
            embedder: crate::embedding::create_embedder(&crate::types::EmbeddingConfig::default())
                .expect("tfidf embedder"),
            fuzzy_engine: Arc::new(Mutex::new(crate::search::FuzzyEngine::new())),
            search_config: crate::search::SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(crate::embedding::EmbeddingCache::default()),
            search_cache: Arc::new(crate::search::SearchResultCache::new(
                crate::search::AdaptiveCacheConfig::default(),
            )),
            hnsw_index: Arc::new(RwLock::new(crate::search::HnswIndex::new(
                crate::search::HnswConfig::new(128, crate::search::VectorMetric::Cosine),
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

    /// One embedded memory recorded under `old-model`.
    fn seed_embedded_memory(ctx: &HandlerContext) -> i64 {
        ctx.storage
            .with_connection(|conn| {
                let id = create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: "embedded memory".to_string(),
                        ..Default::default()
                    },
                )?
                .id;
                conn.execute(
                    "UPDATE memories SET has_embedding = 1, embedding_model = 'old-model' \
                     WHERE id = ?1",
                    rusqlite::params![id],
                )?;
                Ok(id)
            })
            .expect("seed memory")
    }

    fn embedding_model_of(ctx: &HandlerContext, id: i64) -> String {
        ctx.storage
            .with_connection(|conn| {
                Ok(conn.query_row(
                    "SELECT embedding_model FROM memories WHERE id = ?1",
                    rusqlite::params![id],
                    |row| row.get(0),
                )?)
            })
            .expect("embedding_model")
    }

    #[test]
    fn embedding_migrate_dry_run_counts_embedded_memories() {
        let ctx = test_context();
        seed_embedded_memory(&ctx);

        let response = memory_embedding_migrate(&ctx, json!({"dry_run": true}));

        assert_eq!(response["dry_run"], json!(true));
        assert_eq!(response["memories_to_migrate"], json!(1));
    }

    /// Re-embedding is not implemented: the call must say so and must not
    /// relabel vectors that were never recomputed.
    #[test]
    fn embedding_migrate_without_dry_run_reports_unsupported_and_writes_nothing() {
        let ctx = test_context();
        let id = seed_embedded_memory(&ctx);

        let response =
            memory_embedding_migrate(&ctx, json!({"dry_run": false, "target_model": "new-model"}));

        assert!(
            crate::mcp::error::ToolError::is_error_response(&response),
            "must be an error response, got {response}"
        );
        assert_eq!(response["success"], json!(false));
        assert_eq!(response["migrated"], json!(0));
        assert_eq!(embedding_model_of(&ctx, id), "old-model");
    }
}
