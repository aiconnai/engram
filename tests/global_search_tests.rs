//! Integration tests for ENG-1316: opt-in global search across workspaces.
//!
//! Run with: cargo test --test global_search_tests

use std::sync::Arc;

use parking_lot::Mutex;
use serde_json::json;

use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::handlers;
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::queries::create_memory;
use engram::storage::Storage;
use engram::types::{CreateMemoryInput, EmbeddingConfig};

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
        progress_reporter: None,
    }
}

fn add_memory_in_workspace(ctx: &handlers::HandlerContext, content: &str, workspace: &str) -> i64 {
    ctx.storage
        .with_transaction(|conn| {
            let input = CreateMemoryInput {
                content: content.to_string(),
                workspace: Some(workspace.to_string()),
                importance: Some(0.8),
                ..Default::default()
            };
            create_memory(conn, &input).map(|memory| memory.id)
        })
        .expect("create memory")
}

// ---------------------------------------------------------------------------
// Global search tests (ENG-1316)
// ---------------------------------------------------------------------------

/// When `global: false` (default), workspace filter is respected.
#[test]
fn test_search_workspace_filter_respected_without_global() {
    let ctx = test_ctx();

    add_memory_in_workspace(&ctx, "unique alpha content for workspace A", "workspace-a");
    add_memory_in_workspace(&ctx, "unique beta content for workspace B", "workspace-b");

    // Search with workspace filter — should only see workspace-a results.
    let result = handlers::dispatch(
        &ctx,
        "memory_search",
        json!({
            "query": "unique content",
            "workspace": "workspace-a",
            "global": false,
            "rerank": false
        }),
    );

    assert!(result.get("error").is_none(), "unexpected error: {result}");
    let results = result.as_array().unwrap_or_else(|| {
        result["results"]
            .as_array()
            .expect("expected results array")
    });

    // All results should be from workspace-a
    for r in results {
        let ws = r
            .get("memory")
            .and_then(|m| m.get("workspace"))
            .and_then(|w| w.as_str())
            .unwrap_or("");
        assert_eq!(ws, "workspace-a", "result from wrong workspace: {r}");
    }
}

/// When `global: true`, results from all workspaces are returned.
#[test]
fn test_global_search_returns_results_from_all_workspaces() {
    let ctx = test_ctx();

    add_memory_in_workspace(&ctx, "global test memory in alpha workspace", "alpha");
    add_memory_in_workspace(&ctx, "global test memory in beta workspace", "beta");

    let result = handlers::dispatch(
        &ctx,
        "memory_search",
        json!({
            "query": "global test memory",
            "global": true,
            "rerank": false
        }),
    );

    assert!(result.get("error").is_none(), "unexpected error: {result}");
    let results = result.as_array().unwrap_or_else(|| {
        result["results"]
            .as_array()
            .expect("expected results array")
    });

    let workspaces: Vec<&str> = results
        .iter()
        .filter_map(|r| {
            r.get("memory")
                .and_then(|m| m.get("workspace"))
                .and_then(|w| w.as_str())
        })
        .collect();

    assert!(
        workspaces.contains(&"alpha"),
        "expected result from 'alpha' workspace, got: {workspaces:?}"
    );
    assert!(
        workspaces.contains(&"beta"),
        "expected result from 'beta' workspace, got: {workspaces:?}"
    );
}

/// When `global: true`, each result has a top-level `workspace` field.
#[test]
fn test_global_search_includes_workspace_field_in_results() {
    let ctx = test_ctx();

    add_memory_in_workspace(&ctx, "workspace field test memory", "my-workspace");

    let result = handlers::dispatch(
        &ctx,
        "memory_search",
        json!({
            "query": "workspace field test",
            "global": true,
            "rerank": false
        }),
    );

    assert!(result.get("error").is_none(), "unexpected error: {result}");
    let results = result.as_array().unwrap_or_else(|| {
        result["results"]
            .as_array()
            .expect("expected results array")
    });

    assert!(!results.is_empty(), "expected at least one result");
    for r in results {
        assert!(
            r.get("workspace").is_some(),
            "result missing top-level workspace field: {r}"
        );
    }
}

/// `memory_search_compact` with `global: true` returns workspace field in each item.
#[test]
fn test_compact_global_search_includes_workspace_field() {
    let ctx = test_ctx();

    add_memory_in_workspace(&ctx, "compact global search test", "compact-ws");

    let result = handlers::dispatch(
        &ctx,
        "memory_search_compact",
        json!({
            "query": "compact global search",
            "global": true
        }),
    );

    assert!(result.get("error").is_none(), "unexpected error: {result}");
    let results = result["results"]
        .as_array()
        .expect("expected results array");

    assert!(!results.is_empty(), "expected at least one result");
    for r in results {
        assert!(
            r.get("workspace").is_some(),
            "compact result missing workspace field: {r}"
        );
    }
}

/// `memory_search_compact` without `global` does NOT include workspace field.
#[test]
fn test_compact_search_without_global_omits_workspace_field() {
    let ctx = test_ctx();

    add_memory_in_workspace(&ctx, "compact no-global search test", "default");

    let result = handlers::dispatch(
        &ctx,
        "memory_search_compact",
        json!({
            "query": "compact no-global search"
        }),
    );

    assert!(result.get("error").is_none(), "unexpected error: {result}");
    let results = result["results"]
        .as_array()
        .expect("expected results array");

    for r in results {
        assert!(
            r.get("workspace").is_none(),
            "compact result unexpectedly has workspace field: {r}"
        );
    }
}

/// When `global: true` but a workspace param is also supplied, the workspace filter is ignored.
#[test]
fn test_global_overrides_workspace_param() {
    let ctx = test_ctx();

    add_memory_in_workspace(&ctx, "override workspace test in alpha", "alpha");
    add_memory_in_workspace(&ctx, "override workspace test in beta", "beta");

    // Pass workspace: "alpha" but global: true — should see both
    let result = handlers::dispatch(
        &ctx,
        "memory_search",
        json!({
            "query": "override workspace test",
            "workspace": "alpha",
            "global": true,
            "rerank": false
        }),
    );

    assert!(result.get("error").is_none(), "unexpected error: {result}");
    let results = result.as_array().unwrap_or_else(|| {
        result["results"]
            .as_array()
            .expect("expected results array")
    });

    let workspaces: Vec<&str> = results
        .iter()
        .filter_map(|r| {
            r.get("memory")
                .and_then(|m| m.get("workspace"))
                .and_then(|w| w.as_str())
        })
        .collect();

    assert!(
        workspaces.contains(&"beta"),
        "global:true should override workspace filter — expected 'beta' in results, got: {workspaces:?}"
    );
}
