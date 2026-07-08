//! Meilisearch tool handlers (feature-gated behind `meilisearch`).

use serde_json::{json, Value};

use crate::mcp::handlers::HandlerContext;

#[cfg(feature = "meilisearch")]
pub fn meilisearch_search(ctx: &HandlerContext, params: Value) -> Value {
    use crate::storage::StorageBackend;
    use crate::types::SearchOptions;

    let meili = match &ctx.meili {
        Some(m) => m,
        None => {
            return json!({"error": "Meilisearch not configured. Start server with --meilisearch-url and --meilisearch-indexer."})
        }
    };

    let query = match params.get("query").and_then(|v| v.as_str()) {
        Some(q) => q.to_string(),
        None => return json!({"error": "query is required"}),
    };

    let options = SearchOptions {
        limit: params.get("limit").and_then(|v| v.as_i64()),
        workspace: params
            .get("workspace")
            .and_then(|v| v.as_str())
            .map(String::from),
        tags: params.get("tags").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .filter_map(|t| t.as_str().map(String::from))
                .collect()
        }),
        memory_type: params
            .get("memory_type")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok()),
        ..Default::default()
    };

    match meili.search_memories(&query, options) {
        Ok(results) => {
            let items: Vec<Value> = results
                .iter()
                .map(|r| {
                    json!({
                        "id": r.memory.id,
                        "content": r.memory.content,
                        "memory_type": r.memory.memory_type.as_str(),
                        "tags": r.memory.tags,
                        "workspace": r.memory.workspace,
                        "score": r.score,
                        "created_at": r.memory.created_at.to_rfc3339(),
                    })
                })
                .collect();
            json!({
                "results": items,
                "count": items.len(),
                "backend": "meilisearch"
            })
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}

#[cfg(feature = "meilisearch")]
pub fn meilisearch_reindex(ctx: &HandlerContext, _params: Value) -> Value {
    let indexer = match &ctx.meili_indexer {
        Some(i) => i.clone(),
        None => {
            return json!({"error": "Meilisearch indexer not configured. Start server with --meilisearch-url and --meilisearch-indexer."})
        }
    };

    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
        if let Err(e) = rt.block_on(indexer.run_full_sync()) {
            tracing::error!("Meilisearch reindex failed: {}", e);
        }
    });

    json!({
        "status": "reindex_started",
        "message": "Full re-sync from SQLite to Meilisearch started in background."
    })
}

#[cfg(feature = "meilisearch")]
pub fn meilisearch_status(ctx: &HandlerContext, _params: Value) -> Value {
    use crate::storage::StorageBackend;

    let meili = match &ctx.meili {
        Some(m) => m,
        None => return json!({"error": "Meilisearch not configured."}),
    };

    match meili.get_index_stats() {
        Ok(stats) => {
            let health = meili.health_check();
            json!({
                "configured": true,
                "url": meili.url(),
                "index_stats": stats,
                "healthy": health.as_ref().map(|h| h.healthy).unwrap_or(false),
                "health_error": health.as_ref().ok().and_then(|h| h.error.clone()),
            })
        }
        Err(e) => json!({"error": e.to_string()}),
    }
}

#[cfg(feature = "meilisearch")]
pub fn meilisearch_config(ctx: &HandlerContext, _params: Value) -> Value {
    match &ctx.meili {
        Some(meili) => json!({
            "configured": true,
            "url": meili.url(),
            "has_api_key": meili.has_api_key(),
            "indexer_enabled": ctx.meili_indexer.is_some(),
            "sync_interval_seconds": ctx.meili_sync_interval,
        }),
        None => json!({
            "configured": false,
            "message": "Meilisearch not configured. Use --meilisearch-url to enable."
        }),
    }
}
