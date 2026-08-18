//! Memory CRUD tool handlers.

/// Removes `<private>...</private>` tagged sections from content.
///
/// Supports multiline content within tags. If a `<private>` tag is not closed,
/// everything from the opening tag to the end of the string is removed.
///
/// # Examples
/// ```text
/// Input:  "Hello <private>secret</private> world"
/// Output: "Hello  world"
/// ```
pub(super) fn strip_private_content(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let mut remaining = content;
    while let Some(start) = remaining.find("<private>") {
        result.push_str(&remaining[..start]);
        if let Some(end_offset) = remaining[start..].find("</private>") {
            remaining = &remaining[start + end_offset + "</private>".len()..];
        } else {
            // Unclosed tag — remove everything after <private>; stop processing.
            return result;
        }
    }
    result.push_str(remaining);
    result
}

mod create;
mod facts;
mod lifecycle;
mod procedural;
mod read_update_delete;
mod tasks;

pub use create::{
    context_seed, memory_create, memory_create_batch, memory_create_daily, memory_create_episodic,
    memory_create_procedural, memory_create_section,
};
pub use facts::{memory_ingest_fact, memory_ingest_fact_batch};
pub use lifecycle::{
    cleanup_expired, memory_boost, memory_checkpoint, memory_promote_to_permanent, set_expiration,
};
pub use procedural::{memory_get_procedures, memory_get_timeline, record_procedure_outcome};
pub use read_update_delete::{
    memory_delete, memory_delete_batch, memory_get, memory_get_public, memory_list, memory_update,
};
pub use tasks::{create_issue, create_todo};

#[cfg(test)]
mod privacy_tests {
    use super::strip_private_content;

    #[test]
    fn test_no_private_tags() {
        assert_eq!(strip_private_content("Hello, world!"), "Hello, world!");
    }

    #[test]
    fn test_single_private_tag() {
        assert_eq!(
            strip_private_content("Hello <private>secret</private> world"),
            "Hello  world"
        );
    }

    #[test]
    fn test_multiple_private_tags() {
        assert_eq!(
            strip_private_content("a <private>1</private> b <private>2</private> c"),
            "a  b  c"
        );
    }

    #[test]
    fn test_multiline_private_content() {
        assert_eq!(
            strip_private_content("start\n<private>\nline one\nline two\n</private>\nend"),
            "start\n\nend"
        );
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(strip_private_content(""), "");
    }

    #[test]
    fn test_entirely_private() {
        assert_eq!(
            strip_private_content("<private>everything is private</private>"),
            ""
        );
    }

    #[test]
    fn test_unclosed_tag() {
        assert_eq!(
            strip_private_content("visible <private>dangling content"),
            "visible "
        );
    }
}

#[cfg(test)]
mod ingest_fact_tests {
    use super::*;
    use crate::mcp::handlers::HandlerContext;
    use crate::{
        embedding::{create_embedder, EmbeddingCache},
        search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache},
        types::EmbeddingConfig,
    };
    use parking_lot::Mutex;
    use std::sync::Arc;

    fn make_ctx() -> HandlerContext {
        use crate::storage::Storage;
        let storage = Storage::open_in_memory().expect("in-memory storage");
        let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
        HandlerContext {
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
    fn test_ingest_fact_single() {
        let ctx = make_ctx();
        let params = serde_json::json!({
            "fact": "The sky is blue",
            "source": "session:test-123",
            "session_id": "test-123",
            "tags": ["color", "sky"],
            "importance": 0.9
        });
        let result = memory_ingest_fact(&ctx, params);
        assert!(result.get("id").is_some(), "should return id");
        assert_eq!(result["created"], true, "should return created=true");

        let id = result["id"].as_i64().unwrap();
        let get_result = memory_get(&ctx, serde_json::json!({"id": id}));
        assert_eq!(get_result["type"], "fact");
        assert!(
            (get_result["importance"].as_f64().unwrap() - 0.9).abs() < 0.01,
            "importance should be ~0.9"
        );
        let metadata = &get_result["metadata"];
        assert_eq!(metadata["source"], "session:test-123");
        assert_eq!(metadata["session_id"], "test-123");
    }

    #[test]
    fn test_ingest_fact_default_importance() {
        let ctx = make_ctx();
        let params = serde_json::json!({"fact": "Default importance fact"});
        let result = memory_ingest_fact(&ctx, params);
        assert!(result.get("id").is_some());
        let id = result["id"].as_i64().unwrap();
        let get_result = memory_get(&ctx, serde_json::json!({"id": id}));
        assert!(
            (get_result["importance"].as_f64().unwrap() - 0.8).abs() < 0.01,
            "default importance should be 0.8"
        );
        assert_eq!(get_result["type"], "fact");
    }

    #[test]
    fn test_ingest_fact_batch() {
        let ctx = make_ctx();
        let params = serde_json::json!({
            "facts": [
                {"fact": "Fact one", "source": "watcher:/some/file"},
                {"fact": "Fact two", "tags": ["batch"]},
                {"fact": "Fact three", "importance": 0.5}
            ],
            "workspace": "test-ws"
        });
        let result = memory_ingest_fact_batch(&ctx, params);
        assert_eq!(result["count"], 3, "should insert all 3 facts");
        let ids = result["ids"].as_array().unwrap();
        assert_eq!(ids.len(), 3, "should return 3 ids");

        let id = ids[0].as_i64().unwrap();
        let get_result = memory_get(&ctx, serde_json::json!({"id": id}));
        assert_eq!(get_result["type"], "fact");
        assert_eq!(get_result["metadata"]["source"], "watcher:/some/file");
    }

    #[test]
    fn test_ingest_fact_missing_field() {
        let ctx = make_ctx();
        let result = memory_ingest_fact(&ctx, serde_json::json!({}));
        assert!(
            result.get("error").is_some(),
            "should error when fact missing"
        );
    }

    #[test]
    fn test_ingest_fact_batch_empty() {
        let ctx = make_ctx();
        let result = memory_ingest_fact_batch(&ctx, serde_json::json!({"facts": []}));
        assert!(result.get("error").is_some(), "should error on empty batch");
    }

    #[test]
    fn test_ingest_fact_batch_missing_fact_field_returns_error() {
        let ctx = make_ctx();
        // Item at index 1 is missing "fact" — must return an error, not silently skip.
        let params = serde_json::json!({
            "facts": [
                {"fact": "Good fact"},
                {"source": "orphan", "tags": ["no-fact"]}
            ]
        });
        let result = memory_ingest_fact_batch(&ctx, params);
        assert!(
            result.get("error").is_some(),
            "should return error when an item is missing 'fact'"
        );
        let err_msg = result["error"].as_str().unwrap();
        assert!(
            err_msg.contains("index 1"),
            "error should mention the offending index; got: {}",
            err_msg
        );
    }

    #[test]
    fn test_ingest_fact_batch_atomicity_on_missing_fact() {
        let ctx = make_ctx();
        // Even though item 0 is valid, the batch must be rolled back if item 1 is invalid.
        let params = serde_json::json!({
            "facts": [
                {"fact": "Valid fact"},
                {}
            ]
        });
        let result = memory_ingest_fact_batch(&ctx, params);
        assert!(result.get("error").is_some(), "batch must fail");
        // Nothing should have been persisted.
        let list = memory_list(&ctx, serde_json::json!({}));
        let empty = vec![];
        let memories = list.as_array().unwrap_or(&empty);
        assert_eq!(
            memories.len(),
            0,
            "no memories should be persisted after rollback"
        );
    }

    #[test]
    fn test_ingest_fact_scope_default_global() {
        let ctx = make_ctx();
        let params = serde_json::json!({"fact": "Scoped fact"});
        let result = memory_ingest_fact(&ctx, params);
        assert!(result.get("id").is_some());
        let id = result["id"].as_i64().unwrap();
        let get_result = memory_get(&ctx, serde_json::json!({"id": id}));
        assert_eq!(get_result["scope"], "global");
    }

    #[test]
    fn test_ingest_fact_invalid_scope_returns_error() {
        let ctx = make_ctx();
        let params = serde_json::json!({"fact": "Fact", "scope": "invalid_scope"});
        let result = memory_ingest_fact(&ctx, params);
        assert!(
            result.get("error").is_some(),
            "unsupported scope must return error"
        );
    }

    #[test]
    fn test_ingest_fact_batch_scope_default_global() {
        let ctx = make_ctx();
        let params = serde_json::json!({
            "facts": [{"fact": "Batch scoped fact"}]
        });
        let result = memory_ingest_fact_batch(&ctx, params);
        assert_eq!(result["count"], 1);
        let id = result["ids"][0].as_i64().unwrap();
        let get_result = memory_get(&ctx, serde_json::json!({"id": id}));
        assert_eq!(get_result["scope"], "global");
    }
}
