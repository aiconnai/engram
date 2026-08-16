//! Integration tests for Phase 3b MCP Tool Surface Consolidation.
//!
//! Tests canonical high-leverage facades (`memory_lifecycle_update`, `graph_query`, `graph_mutate`)
//! as well as backward-compatible legacy alias routing through MCP dispatch.
//!
//! Run with: cargo test --test mcp_facade_consolidation_tests

use std::sync::Arc;

use parking_lot::Mutex;
use serde_json::json;

use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::handlers::{self, dispatch};
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::Storage;
use engram::types::EmbeddingConfig;

fn test_ctx() -> handlers::HandlerContext {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
    handlers::HandlerContext {
        storage,
        embedder,
        fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
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

fn create_memory(ctx: &handlers::HandlerContext, content: &str, tier: &str) -> i64 {
    let res = dispatch(
        ctx,
        "memory_create",
        json!({
            "content": content,
            "memory_type": "note",
            "tier": tier,
            "importance": 0.7,
            "workspace": "default"
        }),
    );
    res["id"].as_i64().expect("memory id")
}

#[test]
fn test_memory_lifecycle_update_facade() {
    let ctx = test_ctx();
    let mem_id = create_memory(
        &ctx,
        "Ephemeral context needing reinforcement and promotion",
        "daily",
    );

    // Action: Score evaluation
    let score_res = dispatch(
        &ctx,
        "memory_lifecycle_update",
        json!({
            "id": mem_id,
            "action": "score"
        }),
    );
    assert_eq!(score_res["memory_id"].as_i64(), Some(mem_id));
    assert!(score_res.get("score").is_some() || score_res.get("policy").is_some());

    // Action: Promote policy
    let promote_res = dispatch(
        &ctx,
        "memory_lifecycle_update",
        json!({
            "id": mem_id,
            "action": "promote"
        }),
    );
    assert_eq!(promote_res["memory_id"].as_i64(), Some(mem_id));
    assert_eq!(promote_res["canonical_tier"].as_bool(), Some(false));

    // Action: Promote to permanent canonical tier
    let perm_res = dispatch(
        &ctx,
        "memory_lifecycle_update",
        json!({
            "id": mem_id,
            "action": "promote_permanent"
        }),
    );
    assert_eq!(perm_res["memory_id"].as_i64(), Some(mem_id));
    assert_eq!(perm_res["canonical_tier"].as_bool(), Some(true));

    // Action: Transition lifecycle state to archived
    let trans_res = dispatch(
        &ctx,
        "memory_lifecycle_update",
        json!({
            "id": mem_id,
            "action": "transition",
            "state": "archived"
        }),
    );
    assert_eq!(trans_res["id"].as_i64(), Some(mem_id));
    assert_eq!(trans_res["lifecycle_state"].as_str(), Some("archived"));

    // Action: Restore lifecycle state to active
    let restore_res = dispatch(
        &ctx,
        "memory_lifecycle_update",
        json!({
            "id": mem_id,
            "action": "restore"
        }),
    );
    assert_eq!(restore_res["id"].as_i64(), Some(mem_id));
    assert_eq!(restore_res["lifecycle_state"].as_str(), Some("active"));
}

#[test]
fn test_graph_facades_query_and_mutate() {
    let ctx = test_ctx();
    let mem_a = create_memory(
        &ctx,
        "Architecture decision for vector embeddings",
        "permanent",
    );
    let mem_b = create_memory(
        &ctx,
        "Implementation of cosine similarity index",
        "permanent",
    );

    // Mutate: Link memories
    let link_res = dispatch(
        &ctx,
        "graph_mutate",
        json!({
            "action": "link",
            "from_id": mem_a,
            "to_id": mem_b,
            "edge_type": "implements",
            "strength": 0.95,
            "source_context": "Embeddings indexing implementation"
        }),
    );
    assert!(link_res.get("error").is_none());

    // Query: Related / Neighborhood
    let query_res = dispatch(
        &ctx,
        "graph_query",
        json!({
            "action": "relations",
            "id": mem_a
        }),
    );
    let related_arr = query_res.as_array().expect("array of relations");
    assert!(!related_arr.is_empty());
    assert_eq!(related_arr[0]["to_id"].as_i64(), Some(mem_b));

    // Query: Path finding
    let path_res = dispatch(
        &ctx,
        "graph_query",
        json!({
            "action": "path",
            "from_id": mem_a,
            "to_id": mem_b
        }),
    );
    assert_eq!(path_res["found"].as_bool(), Some(true));

    // Query: Export graph
    let export_res = dispatch(
        &ctx,
        "graph_query",
        json!({
            "action": "export",
            "format": "json"
        }),
    );
    assert!(
        export_res.get("nodes").is_some()
            || export_res.get("edges").is_some()
            || export_res.is_object()
    );

    // Mutate: Unlink memories
    let unlink_res = dispatch(
        &ctx,
        "graph_mutate",
        json!({
            "action": "unlink",
            "from_id": mem_a,
            "to_id": mem_b,
            "edge_type": "implements"
        }),
    );
    assert_eq!(unlink_res["unlinked"].as_bool(), Some(true));
}

#[test]
fn test_backward_compatible_legacy_aliases() {
    let ctx = test_ctx();
    let mem_a = create_memory(&ctx, "Legacy cross-reference source", "permanent");
    let mem_b = create_memory(&ctx, "Legacy cross-reference target", "permanent");

    // Legacy memory_link
    let link_res = dispatch(
        &ctx,
        "memory_link",
        json!({
            "from_id": mem_a,
            "to_id": mem_b,
            "edge_type": "related_to"
        }),
    );
    assert!(link_res.get("error").is_none());

    // Legacy memory_related
    let rel_res = dispatch(
        &ctx,
        "memory_related",
        json!({
            "id": mem_a
        }),
    );
    let rel_arr = rel_res.as_array().expect("array of related");
    assert!(!rel_arr.is_empty());

    // Legacy memory_find_path
    let path_res = dispatch(
        &ctx,
        "memory_find_path",
        json!({
            "from_id": mem_a,
            "to_id": mem_b
        }),
    );
    assert_eq!(path_res["found"].as_bool(), Some(true));

    // Legacy memory_unlink
    let unlink_res = dispatch(
        &ctx,
        "memory_unlink",
        json!({
            "from_id": mem_a,
            "to_id": mem_b
        }),
    );
    assert_eq!(unlink_res["unlinked"].as_bool(), Some(true));
}
