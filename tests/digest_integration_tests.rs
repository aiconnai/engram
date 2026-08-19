//! Integration tests for RFC 0008: `memory_digest` Actionable Retrieval Digest.
//!
//! Tests the memory_digest MCP tool across modes, graph expansions,
//! operational context attachments, provenance tracking, and error boundaries.

use parking_lot::Mutex;
use serde_json::json;
use std::sync::Arc;

use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::handlers::{self, HandlerContext};
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::Storage;
use engram::types::EmbeddingConfig;

fn setup_test_context() -> (Storage, HandlerContext) {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let embedder = create_embedder(&EmbeddingConfig::default()).expect("embedder");
    let ctx = HandlerContext {
        storage: storage.clone(),
        embedder: embedder.clone(),
        fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
        hnsw_index: Arc::new(parking_lot::RwLock::new(engram::search::HnswIndex::new(
            engram::search::HnswConfig::new(
                embedder.dimensions(),
                engram::search::VectorMetric::Cosine,
            ),
        ))),
        progress_reporter: None,
        #[cfg(feature = "meilisearch")]
        meili: None,
        #[cfg(feature = "meilisearch")]
        meili_indexer: None,
        #[cfg(feature = "meilisearch")]
        meili_sync_interval: 60,
        #[cfg(feature = "langfuse")]
        langfuse_runtime: Arc::new(tokio::runtime::Runtime::new().expect("langfuse runtime")),
    };
    (storage, ctx)
}

fn seed_memory(
    ctx: &HandlerContext,
    content: &str,
    mem_type: &str,
    workspace: Option<&str>,
    tags: Vec<&str>,
) -> i64 {
    let mut params = json!({
        "content": content,
        "memory_type": mem_type,
        "tags": tags,
        "importance": 0.9
    });
    if let Some(ws) = workspace {
        params["workspace"] = json!(ws);
    }
    let res = handlers::dispatch(ctx, "memory_create", params);
    res["id"].as_i64().expect("created memory id")
}

#[test]
fn test_memory_digest_standard_flow() {
    let (_, ctx) = setup_test_context();

    let m1 = seed_memory(
        &ctx,
        "PostgreSQL connection pooling is configured with max 20 connections per pod.",
        "decision",
        Some("prod"),
        vec!["database", "postgres", "infra"],
    );
    let m2 = seed_memory(
        &ctx,
        "PostgreSQL timeout settings: statement_timeout=5000ms, idle_in_transaction=10000ms.",
        "context",
        Some("prod"),
        vec!["database", "postgres", "timeouts"],
    );

    // Call memory_digest
    let params = json!({
        "topic": "PostgreSQL connection pooling and timeouts",
        "workspace": "prod",
        "mode": "standard",
        "limit": 10,
        "include_graph": false,
        "include_operational_context": false,
    });

    let resp = handlers::digest::memory_digest(&ctx, params);
    assert!(resp.get("error").is_none(), "unexpected error: {resp}");

    assert_eq!(resp["topic"], "PostgreSQL connection pooling and timeouts");
    assert_eq!(resp["workspace"], "prod");
    assert_eq!(resp["mode"], "standard");

    let top_memories = resp["top_memories"].as_array().expect("top_memories array");
    assert!(!top_memories.is_empty(), "expected top_memories");

    let top_ids: Vec<i64> = top_memories
        .iter()
        .filter_map(|m| m["id"].as_i64())
        .collect();
    assert!(top_ids.contains(&m1) || top_ids.contains(&m2));

    // Check provenance
    let provenance = &resp["provenance"];
    let source_ids = provenance["source_memory_ids"]
        .as_array()
        .expect("source_memory_ids array");
    assert!(!source_ids.is_empty());
}

#[test]
fn test_memory_digest_brief_and_deep_modes() {
    let (_, ctx) = setup_test_context();

    seed_memory(
        &ctx,
        "Redis cluster cache key prefix standard: use tenant:workspace:entity format.",
        "decision",
        Some("default"),
        vec!["redis", "caching"],
    );

    // Brief mode
    let brief_resp = handlers::digest::memory_digest(
        &ctx,
        json!({
            "topic": "Redis cache keys",
            "mode": "brief",
            "include_operational_context": false
        }),
    );
    assert!(brief_resp.get("error").is_none());
    assert_eq!(brief_resp["mode"], "brief");

    // Deep mode
    let deep_resp = handlers::digest::memory_digest(
        &ctx,
        json!({
            "topic": "Redis cache keys",
            "mode": "deep",
            "include_operational_context": false
        }),
    );
    assert!(deep_resp.get("error").is_none());
    assert_eq!(deep_resp["mode"], "deep");
}

#[test]
fn test_memory_digest_graph_relationships() {
    let (storage, ctx) = setup_test_context();

    let m1 = seed_memory(
        &ctx,
        "AuthService validates JWT bearer tokens before passing to upstream.",
        "decision",
        Some("auth"),
        vec!["auth", "jwt"],
    );
    let m2 = seed_memory(
        &ctx,
        "GatewayWorker routes authenticated traffic to service mesh.",
        "context",
        Some("auth"),
        vec!["gateway", "routing"],
    );

    // Link memories
    storage
        .with_connection(|conn| {
            engram::storage::queries::create_crossref(
                conn,
                &engram::types::CreateCrossRefInput {
                    from_id: m1,
                    to_id: m2,
                    edge_type: engram::types::EdgeType::RelatedTo,
                    strength: Some(0.95),
                    source_context: Some(
                        "Gateway routes after AuthService authentication".to_string(),
                    ),
                    pinned: false,
                },
            )
        })
        .expect("create crossref");

    let resp = handlers::digest::memory_digest(
        &ctx,
        json!({
            "topic": "AuthService and Gateway routing",
            "workspace": "auth",
            "include_graph": true,
            "related_depth": 1,
            "include_operational_context": false
        }),
    );
    assert!(resp.get("error").is_none());

    let relationships = resp["relationships"]
        .as_array()
        .expect("relationships array");
    assert!(
        !relationships.is_empty(),
        "expected graph relationship to be returned"
    );
}

#[test]
fn test_memory_digest_type_filtering() {
    let (_, ctx) = setup_test_context();

    let m_decision = seed_memory(
        &ctx,
        "Security decision: rotate encryption keys every 30 days.",
        "decision",
        Some("sec"),
        vec!["crypto"],
    );
    let _m_daily = seed_memory(
        &ctx,
        "Daily scratch: testing new encryption key script.",
        "note",
        Some("sec"),
        vec!["crypto"],
    );

    let resp = handlers::digest::memory_digest(
        &ctx,
        json!({
            "topic": "encryption keys",
            "workspace": "sec",
            "include_types": ["decision"],
            "include_operational_context": false
        }),
    );
    assert!(resp.get("error").is_none());

    let top_memories = resp["top_memories"].as_array().expect("top_memories");
    for item in top_memories {
        assert_eq!(
            item["memory_type"].as_str(),
            Some("decision"),
            "only decision type should match"
        );
        assert_eq!(item["id"].as_i64(), Some(m_decision));
    }
}

#[test]
fn test_memory_digest_empty_and_invalid_inputs() {
    let (_, ctx) = setup_test_context();

    // Missing topic
    let missing_topic = handlers::digest::memory_digest(&ctx, json!({}));
    assert!(missing_topic.get("error").is_some());

    // Empty topic
    let empty_topic = handlers::digest::memory_digest(&ctx, json!({"topic": ""}));
    assert!(empty_topic.get("error").is_some());

    // Invalid mode
    let invalid_mode = handlers::digest::memory_digest(
        &ctx,
        json!({
            "topic": "valid topic",
            "mode": "invalid_mode_xyz"
        }),
    );
    assert!(invalid_mode.get("error").is_some());

    // Out of range budget
    let small_budget = handlers::digest::memory_digest(
        &ctx,
        json!({
            "topic": "valid topic",
            "total_budget": 100
        }),
    );
    assert!(small_budget.get("error").is_some());
}
