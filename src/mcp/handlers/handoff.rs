//! Session handoff handler — "land the plane" protocol.
//!
//! Generates structured session handoffs with copy-ready continuation packets for
//! seamless cross-session continuity. Inspired by Beads' land-the-plane pattern.

use serde_json::{json, Value};

use super::HandlerContext;
use crate::intelligence::{HandoffItem, SessionHandoffPacket};

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

    let next_session_hints = request.next_session_hints.clone();

    match crate::intelligence::build_session_handoff(&ctx.storage, request) {
        Ok(packet) => {
            let checkpoint_id = packet.checkpoint_id;
            let handoff = compatibility_handoff(&packet, &next_session_hints);
            json!({
                "handoff": handoff,
                "checkpoint_id": checkpoint_id,
            })
        }
        Err(err) => json!({"error": format!("Failed to build session handoff: {err}")}),
    }
}

fn compatibility_handoff(packet: &SessionHandoffPacket, next_session_hints: &[String]) -> Value {
    let mut handoff = json!(packet);
    if let Some(object) = handoff.as_object_mut() {
        object.insert(
            "recent_decisions".to_string(),
            json!(compatibility_items(&packet.decisions)),
        );
        object.insert(
            "memories_count".to_string(),
            json!(compatibility_memories_count(packet)),
        );
        object.insert("next_session_hints".to_string(), json!(next_session_hints));
        object.insert("bootstrap_prompt".to_string(), json!(packet.copy_block));
        object.insert("checkpoint_id".to_string(), json!(packet.checkpoint_id));
    }
    handoff
}

fn compatibility_items(items: &[HandoffItem]) -> Vec<Value> {
    items
        .iter()
        .map(|item| {
            json!({
                "title": item.title,
                "content": item.title,
                "detail": item.detail,
                "source_memory_id": item.source_memory_id,
                "source_context_event_id": item.source_context_event_id,
            })
        })
        .collect()
}

fn compatibility_memories_count(packet: &SessionHandoffPacket) -> usize {
    let mut source_ids = packet.source_memory_ids.clone();
    source_ids.extend(
        packet
            .open_items
            .iter()
            .chain(packet.decisions.iter())
            .filter_map(|item| item.source_memory_id),
    );
    source_ids.sort_unstable();
    source_ids.dedup();

    if source_ids.is_empty() {
        packet.open_items.len() + packet.decisions.len()
    } else {
        source_ids.len()
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
                "summary": "Manual session rotation",
                "decisions_made": ["Keep session_land compatibility"],
                "next_session_hints": ["Resume from the copy block"]
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
        assert!(result.get("checkpoint_id").is_some());
        assert!(result["handoff"].get("checkpoint_id").is_some());
        assert_eq!(result["checkpoint_id"], result["handoff"]["checkpoint_id"]);
        assert!(result["handoff"].get("bootstrap_prompt").is_some());
        assert_eq!(
            result["handoff"]["bootstrap_prompt"],
            result["handoff"]["copy_block"]
        );
        assert!(result["handoff"].get("recent_decisions").is_some());
        assert!(result["handoff"]["recent_decisions"]
            .as_array()
            .expect("recent decisions")
            .iter()
            .any(|decision| decision["title"] == "Keep session_land compatibility"));
        assert!(result["handoff"].get("memories_count").is_some());
        assert!(result["handoff"].get("next_session_hints").is_some());
        assert_eq!(
            result["handoff"]["next_session_hints"],
            json!(["Resume from the copy block"])
        );
        assert!(result["handoff"].get("copy_block").is_some());
        assert!(result["handoff"].get("warnings").is_some());
    }
}
