//! Session handoff handler — "land the plane" protocol.
//!
//! Generates structured session handoffs with copy-ready continuation packets for
//! seamless cross-session continuity. Inspired by Beads' land-the-plane pattern.

use serde_json::{json, Value};

use super::HandlerContext;

/// Land the plane: generate a structured session handoff.
///
/// Params:
/// - `session_id` (string, optional) — session to hand off; omitted uses builder fallback
/// - `workspace` (string, optional, default "default") — workspace scope
/// - `summary` (string, optional) — human-provided summary of what was accomplished
/// - `next_session_hints` (array of strings, optional) — hints for next session
pub fn session_land(ctx: &HandlerContext, params: Value) -> Value {
    let mut request: crate::intelligence::SessionHandoffRequest =
        match serde_json::from_value(params.clone()) {
            Ok(request) => request,
            Err(err) => return json!({"error": err.to_string()}),
        };

    if request.workspace.is_none() {
        request.workspace = params
            .get("workspace")
            .and_then(|value| value.as_str())
            .map(str::to_string);
    }

    match crate::intelligence::build_session_handoff(&ctx.storage, request) {
        Ok(packet) => {
            let checkpoint_id = packet.checkpoint_id;
            json!({
                "handoff": packet,
                "checkpoint_id": checkpoint_id,
            })
        }
        Err(err) => json!({"error": format!("Failed to build session handoff: {err}")}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    use parking_lot::Mutex;
    use serde_json::json;

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
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 60,
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
        }
    }

    #[test]
    fn test_session_land_without_session_id_returns_workspace_packet() {
        let ctx = test_context();
        let result = session_land(
            &ctx,
            json!({
                "workspace": "default",
                "summary": "Manual session rotation"
            }),
        );

        assert!(
            result.get("error").is_none(),
            "unexpected error: {result:?}"
        );
        assert!(result["handoff"]["copy_block"]
            .as_str()
            .expect("copy block")
            .contains("# Continue this work in a new AI session"));
        assert!(result["handoff"]["warnings"]
            .as_array()
            .expect("warnings")
            .iter()
            .any(|warning| warning
                .as_str()
                .unwrap_or("")
                .contains("No concrete session resolved")));
    }
}
