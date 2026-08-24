//! Integration tests for Two-Tier Semantic Query Caching in MCP search handlers.
//!
//! Tests exact query short-circuiting, semantic cosine similarity matching,
//! cache invalidation on memory updates/deletions, and cache management tools.
//!
//! Run with: cargo test --test semantic_cache_integration

use std::sync::Arc;

use parking_lot::Mutex;
use serde_json::json;

use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::handlers::{self, memory_crud, search};
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::Storage;
use engram::types::EmbeddingConfig;

fn test_ctx() -> handlers::HandlerContext {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
    handlers::HandlerContext {
        storage,
        embedder: embedder.clone(),
        fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig {
            similarity_threshold: 0.90,
            ..Default::default()
        })),
        hnsw_index: Arc::new(parking_lot::RwLock::new(engram::search::HnswIndex::new(
            engram::search::HnswConfig::new(
                embedder.dimensions(),
                engram::search::VectorMetric::Cosine,
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
        principal: None,
    }
}

fn create_sample_memory(ctx: &handlers::HandlerContext, content: &str, tag: &str) -> i64 {
    let input = json!({
        "content": content,
        "memory_type": "note",
        "tags": [tag],
        "importance": 0.8,
        "workspace": "default"
    });
    let result = memory_crud::memory_create(ctx, input);
    result["id"].as_i64().expect("memory id")
}

#[test]
fn test_exact_query_cache_hit_tier() {
    let ctx = test_ctx();
    create_sample_memory(
        &ctx,
        "Rust tokio broadcast channels handle slow consumers via RecvError::Lagged",
        "rust",
    );

    let query_params = json!({
        "query": "Rust tokio broadcast channels",
        "workspace": "default"
    });

    // 1st search: Cache Miss -> Hybrid search executes
    let res1 = search::memory_search(&ctx, query_params.clone());
    let results1 = res1.as_array().expect("results array");
    assert!(!results1.is_empty());

    let stats_after_first = ctx.search_cache.stats();
    assert_eq!(stats_after_first.exact_hits, 0);
    assert_eq!(stats_after_first.entries, 1);

    // 2nd search with identical query: Exact Cache Hit (Tier 1)
    let res2 = search::memory_search(&ctx, query_params);
    let results2 = res2.as_array().expect("results array");
    assert_eq!(results1.len(), results2.len());

    let stats_after_second = ctx.search_cache.stats();
    assert_eq!(stats_after_second.exact_hits, 1);
    assert_eq!(stats_after_second.hits, 1);
}

#[test]
fn test_semantic_query_cache_hit_tier() {
    let ctx = test_ctx();
    create_sample_memory(
        &ctx,
        "PostgreSQL connection pooling architecture using pgbouncer and keepalive settings",
        "database",
    );

    // Initial search caches the results under the query embedding
    let initial_query = json!({
        "query": "PostgreSQL connection pooling architecture",
        "workspace": "default"
    });
    let res1 = search::memory_search(&ctx, initial_query);
    assert!(res1.as_array().is_some());

    // Search with identical embedding via manual put with similar embedding
    let query_emb = ctx
        .embedder
        .embed("PostgreSQL connection pooling architecture")
        .expect("embedding");

    // Slightly perturb embedding
    let mut similar_emb = query_emb.clone();
    if !similar_emb.is_empty() {
        similar_emb[0] += 0.001;
    }

    // Direct semantic lookup in cache
    let filters = engram::search::CacheFilterParams {
        workspace: Some("default".to_string()),
        rerank_strategy: Some("Heuristic".to_string()),
        ..Default::default()
    };
    let semantic_hit = ctx.search_cache.get_semantic(&similar_emb, &filters);
    assert!(semantic_hit.is_some());

    let stats = ctx.search_cache.stats();
    assert_eq!(stats.semantic_hits, 1);
}

#[test]
fn test_cache_invalidation_on_memory_crud() {
    let ctx = test_ctx();
    let mem_id = create_sample_memory(
        &ctx,
        "Cache invalidation must purge stale search query results on memory mutations",
        "caching",
    );

    let query_params = json!({
        "query": "Cache invalidation must purge stale results",
        "workspace": "default"
    });

    // Populate cache
    let res1 = search::memory_search(&ctx, query_params.clone());
    assert!(res1.as_array().is_some());
    assert_eq!(ctx.search_cache.stats().entries, 1);

    // Verify it hits exact cache
    let res2 = search::memory_search(&ctx, query_params.clone());
    assert!(res2.as_array().is_some());
    assert_eq!(ctx.search_cache.stats().exact_hits, 1);

    // Update memory content via MCP handler
    let update_params = json!({
        "id": mem_id,
        "content": "Updated memory content with new caching semantics"
    });
    memory_crud::memory_update(&ctx, update_params);

    // Verify cache was invalidated: cache entries dropped
    assert_eq!(ctx.search_cache.stats().entries, 0);

    // Subsequent search must be a miss and repopulate cache
    let res3 = search::memory_search(&ctx, query_params);
    assert!(res3.as_array().is_some());
    assert_eq!(ctx.search_cache.stats().entries, 1);
}

#[test]
fn test_search_cache_management_mcp_tools() {
    let ctx = test_ctx();
    create_sample_memory(
        &ctx,
        "Exploring memory search and caching feedback loops",
        "search",
    );

    let query = "memory search caching feedback";
    let search_params = json!({
        "query": query,
        "workspace": "default"
    });

    // Run search to populate cache
    search::memory_search(&ctx, search_params.clone());
    search::memory_search(&ctx, search_params);

    // Check stats via search_cache_stats tool
    let stats_val = search::search_cache_stats(&ctx, json!({}));
    assert!(stats_val["hits"].as_u64().unwrap_or(0) >= 1);

    // Record negative feedback via search_cache_feedback tool (tightens threshold)
    let initial_threshold = ctx.search_cache.current_threshold();
    let feedback_res = search::search_cache_feedback(
        &ctx,
        json!({
            "query": query,
            "positive": false,
            "workspace": "default"
        }),
    );
    assert_eq!(feedback_res["recorded"].as_bool(), Some(true));
    let new_threshold = feedback_res["current_threshold"].as_f64().unwrap() as f32;
    assert!(new_threshold >= initial_threshold);

    // Clear cache via search_cache_clear tool
    let clear_res = search::search_cache_clear(&ctx, json!({}));
    assert_eq!(clear_res["cleared"].as_bool(), Some(true));

    let stats_after = search::search_cache_stats(&ctx, json!({}));
    assert_eq!(stats_after["entries"].as_u64(), Some(0));
}
