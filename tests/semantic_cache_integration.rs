//! Integration tests for Two-Tier Semantic Query Caching in MCP search handlers.
//!
//! Tests exact query short-circuiting, semantic cosine similarity matching,
//! cache invalidation on memory updates/deletions, and cache management tools.
//!
//! Run with: cargo test --test semantic_cache_integration

use std::sync::Arc;

use chrono::Utc;
use parking_lot::Mutex;
use serde_json::{json, Value};

use engram::auth::{PermissionSet, TokenClaims, TransportPrincipal, UserId};
use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::handlers::{self, memory_crud, retrieval, search};
use engram::search::{
    AdaptiveCacheConfig, CacheFilterParams, FuzzyEngine, SearchConfig, SearchResultCache,
};
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

// ── memory_cache_clear: one response contract for every scope ───────────────

fn admin_principal(namespace: Option<&str>) -> TransportPrincipal {
    TransportPrincipal::from_token_claims(TokenClaims {
        user_id: UserId::from_string("agent-cache"),
        key_id: "key-agent-cache".to_string(),
        permissions: PermissionSet::admin(),
        namespace: namespace.map(str::to_string),
        issued_at: Utc::now(),
        expires_at: None,
    })
    .expect("valid token claims")
}

fn put_cache_entry(ctx: &handlers::HandlerContext, query: &str, workspace: Option<&str>) {
    ctx.search_cache.put(
        query,
        None,
        CacheFilterParams {
            workspace: workspace.map(str::to_string),
            ..Default::default()
        },
        Vec::new(),
    );
}

/// Two `tenant-a` entries, one `tenant-b` entry and one workspace-less entry.
fn ctx_with_cached_entries() -> handlers::HandlerContext {
    let ctx = test_ctx();
    put_cache_entry(&ctx, "alpha one", Some("tenant-a"));
    put_cache_entry(&ctx, "alpha two", Some("tenant-a"));
    put_cache_entry(&ctx, "beta one", Some("tenant-b"));
    put_cache_entry(&ctx, "nowhere one", None);
    assert_eq!(ctx.search_cache.stats().entries, 4);
    ctx
}

fn cleared(entries: u64) -> Value {
    json!({"success": true, "entries_cleared": entries})
}

#[test]
fn memory_cache_clear_unscoped_keeps_its_wire_shape() {
    let ctx = ctx_with_cached_entries();

    let response = retrieval::memory_cache_clear(&ctx, json!({}));

    assert_eq!(response, cleared(4));
    assert_eq!(ctx.search_cache.stats().entries, 0);
}

#[test]
fn memory_cache_clear_scoped_returns_the_unscoped_shape() {
    let ctx = ctx_with_cached_entries();

    let response = retrieval::memory_cache_clear(&ctx, json!({"workspace": "tenant-a"}));

    // tenant-a's two entries plus the workspace-less one (it may hold tenant-a rows).
    assert_eq!(
        response,
        cleared(3),
        "same keys as the unscoped call: {response}"
    );
    assert_eq!(ctx.search_cache.stats().entries, 1, "tenant-b must survive");
}

#[test]
fn memory_cache_clear_scoped_dispatch_for_restricted_principal() {
    let mut ctx = ctx_with_cached_entries();
    ctx.principal = Some(admin_principal(Some("tenant-a")));

    let response = handlers::dispatch(&ctx, "memory_cache_clear", json!({"workspace": "tenant-a"}));
    assert_eq!(response, cleared(3), "restricted principal: {response}");
    assert_eq!(ctx.search_cache.stats().entries, 1, "tenant-b must survive");

    // A foreign workspace is denied and evicts nothing.
    let denied = handlers::dispatch(&ctx, "memory_cache_clear", json!({"workspace": "tenant-b"}));
    assert_eq!(denied["error"]["code"], "permission_denied", "{denied}");
    assert_eq!(ctx.search_cache.stats().entries, 1);
}

#[test]
fn memory_cache_clear_without_workspace_is_denied_for_a_restricted_principal() {
    let mut ctx = ctx_with_cached_entries();
    ctx.principal = Some(admin_principal(Some("tenant-a")));

    // Direct handler call: a restricted principal must name its workspace;
    // an unscoped clear would evict other tenants' entries.
    let response = retrieval::memory_cache_clear(&ctx, json!({}));

    assert_eq!(response["error"]["code"], "permission_denied", "{response}");
    assert_eq!(
        response["error"]["tool"], "memory_cache_clear",
        "{response}"
    );
    assert_eq!(
        ctx.search_cache.stats().entries,
        4,
        "nothing may be evicted"
    );
}

#[test]
fn memory_cache_clear_unrestricted_principal_uses_the_same_shapes() {
    let mut ctx = ctx_with_cached_entries();
    ctx.principal = Some(admin_principal(None));

    let scoped = handlers::dispatch(&ctx, "memory_cache_clear", json!({"workspace": "tenant-b"}));
    assert_eq!(
        scoped,
        cleared(2),
        "scoped, unrestricted principal: {scoped}"
    );
    assert_eq!(ctx.search_cache.stats().entries, 2);

    let all = handlers::dispatch(&ctx, "memory_cache_clear", json!({}));
    assert_eq!(all, cleared(2), "unscoped, unrestricted principal: {all}");
    assert_eq!(ctx.search_cache.stats().entries, 0);
}

#[test]
fn search_cache_clear_keeps_its_own_wire_shape() {
    let ctx = ctx_with_cached_entries();

    let scoped = search::search_cache_clear(&ctx, json!({"workspace": "tenant-a"}));
    assert_eq!(
        scoped,
        json!({"cleared": true, "scope": "workspace", "workspace": "tenant-a"})
    );
    let all = search::search_cache_clear(&ctx, json!({}));
    assert_eq!(all, json!({"cleared": true, "scope": "all"}));
}

// ── search cache isolation across principals and workspaces ─────────────────
//
// The cache key carries every option that shapes a result (workspace(s),
// global, filters, limit, offset, ...) but not the caller. That is safe only
// because authorization runs BEFORE the lookup and forces a restricted
// principal's workspace into the key. These tests pin that ordering.

fn reader(namespace: &str) -> TransportPrincipal {
    TransportPrincipal::from_token_claims(TokenClaims {
        user_id: UserId::from_string(format!("reader-{namespace}")),
        key_id: format!("key-reader-{namespace}"),
        permissions: PermissionSet::read_only(),
        namespace: Some(namespace.to_string()),
        issued_at: Utc::now(),
        expires_at: None,
    })
    .expect("valid token claims")
}

fn create_in(ctx: &handlers::HandlerContext, content: &str, workspace: &str) {
    let created =
        memory_crud::memory_create(ctx, json!({"content": content, "workspace": workspace}));
    assert!(created["id"].is_i64(), "seed failed: {created}");
}

#[test]
fn search_cache_is_not_a_side_channel_between_principals() {
    let mut ctx = test_ctx();
    create_in(&ctx, "zebracorn alpha roadmap note", "tenant-a");
    create_in(&ctx, "zebracorn beta payroll secret", "tenant-b");

    // An unrestricted caller warms entries for every scope, including the
    // two a restricted caller might collide with.
    ctx.principal = Some(admin_principal(None));
    for params in [
        json!({"query": "zebracorn"}),
        json!({"query": "zebracorn", "global": true}),
        json!({"query": "zebracorn", "workspace": "tenant-b"}),
        json!({"query": "zebracorn", "workspaces": ["tenant-b"]}),
    ] {
        let warmed = search::memory_search(&ctx, params.clone());
        assert!(warmed.is_array(), "warm-up {params} failed: {warmed}");
    }
    assert!(ctx.search_cache.stats().entries >= 3);

    // A tenant-a principal is refused every scope that has a cached entry
    // holding tenant-b rows, and its own scope never contains them.
    ctx.principal = Some(reader("tenant-a"));
    for params in [
        json!({"query": "zebracorn", "workspace": "tenant-b"}),
        json!({"query": "zebracorn", "workspaces": ["tenant-b"]}),
        json!({"query": "zebracorn", "global": true}),
        json!({"query": "zebracorn"}),
        json!({"query": "zebracorn", "workspace": "tenant-a", "filter": {"workspace": "tenant-b"}}),
    ] {
        let denied = search::memory_search(&ctx, params.clone());
        assert_eq!(
            denied["error"]["code"], "permission_denied",
            "{params} must be refused despite a warm cache: {denied}"
        );
        assert!(
            !denied.to_string().contains("payroll"),
            "{params}: {denied}"
        );
    }
    for params in [
        json!({"query": "zebracorn", "workspace": "tenant-a"}),
        json!({"query": "zebracorn", "workspaces": ["tenant-a"]}),
    ] {
        for attempt in 0..2 {
            // The second attempt is served from the cache.
            let own = search::memory_search(&ctx, params.clone());
            assert!(own.is_array(), "{params} attempt {attempt}: {own}");
            assert!(!own.to_string().contains("payroll"), "{params}: {own}");
            assert!(own.to_string().contains("roadmap"), "{params}: {own}");
        }
    }

    assert!(
        ctx.search_cache.stats().exact_hits >= 2,
        "the repeated own-scope searches must be served from the cache"
    );

    // And the other way round: tenant-b never sees tenant-a's cached rows.
    ctx.principal = Some(reader("tenant-b"));
    let denied =
        search::memory_search(&ctx, json!({"query": "zebracorn", "workspace": "tenant-a"}));
    assert_eq!(denied["error"]["code"], "permission_denied", "{denied}");
    let own = search::memory_search(&ctx, json!({"query": "zebracorn", "workspace": "tenant-b"}));
    assert!(!own.to_string().contains("roadmap"), "{own}");
}
