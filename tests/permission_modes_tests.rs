//! Integration tests for RFC 0010: Permission Modes for MCP and Harness Operations.

use parking_lot::{Mutex, RwLock};
use serde_json::json;
use std::sync::Arc;

use engram::mcp::handlers::{dispatch, HandlerContext};
use engram::mcp::permission::{
    active_permission_mode, permission_denial_for_mode, required_mode, PermissionMode,
};
use engram::Storage;

fn setup_test_context() -> HandlerContext {
    let storage = Storage::open_in_memory().expect("in-memory database");
    let embedder = engram::embedding::create_embedder(&engram::types::EmbeddingConfig::default())
        .expect("tfidf embedder");

    HandlerContext {
        storage,
        embedder,
        fuzzy_engine: Arc::new(Mutex::new(engram::search::FuzzyEngine::new())),
        search_config: engram::search::SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(engram::embedding::EmbeddingCache::default()),
        search_cache: Arc::new(engram::search::SearchResultCache::new(
            engram::search::AdaptiveCacheConfig::default(),
        )),
        hnsw_index: Arc::new(RwLock::new(engram::search::HnswIndex::new(
            engram::search::HnswConfig::new(128, engram::search::VectorMetric::Cosine),
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

#[test]
fn test_permission_mode_hierarchy_and_parsing() {
    assert!(PermissionMode::ReadOnly < PermissionMode::ScopedWrite);
    assert!(PermissionMode::ScopedWrite < PermissionMode::Maintenance);
    assert!(PermissionMode::Maintenance < PermissionMode::Admin);

    assert_eq!(
        PermissionMode::parse("read_only"),
        Some(PermissionMode::ReadOnly)
    );
    assert_eq!(
        PermissionMode::parse("scoped_write"),
        Some(PermissionMode::ScopedWrite)
    );
    assert_eq!(
        PermissionMode::parse("maintenance"),
        Some(PermissionMode::Maintenance)
    );
    assert_eq!(PermissionMode::parse("admin"), Some(PermissionMode::Admin));
    assert_eq!(PermissionMode::parse("invalid_mode"), None);

    assert_eq!(PermissionMode::ReadOnly.as_str(), "read_only");
    assert_eq!(PermissionMode::ScopedWrite.as_str(), "scoped_write");
    assert_eq!(PermissionMode::Maintenance.as_str(), "maintenance");
    assert_eq!(PermissionMode::Admin.as_str(), "admin");
}

#[test]
fn test_tool_required_modes_classification() {
    // Read-only tools
    assert_eq!(required_mode("memory_get"), Some(PermissionMode::ReadOnly));
    assert_eq!(required_mode("memory_list"), Some(PermissionMode::ReadOnly));
    assert_eq!(
        required_mode("memory_search"),
        Some(PermissionMode::ReadOnly)
    );
    assert_eq!(
        required_mode("model_routes_list"),
        Some(PermissionMode::ReadOnly)
    );
    assert_eq!(
        required_mode("permission_mode_status"),
        Some(PermissionMode::ReadOnly)
    );

    // Scoped-write tools
    assert_eq!(
        required_mode("memory_create"),
        Some(PermissionMode::ScopedWrite)
    );
    assert_eq!(
        required_mode("context_seed"),
        Some(PermissionMode::ScopedWrite)
    );
    assert_eq!(
        required_mode("memory_link"),
        Some(PermissionMode::ScopedWrite)
    );

    // Maintenance tools
    assert_eq!(
        required_mode("lifecycle_run"),
        Some(PermissionMode::Maintenance)
    );
    assert_eq!(
        required_mode("memory_cleanup_expired"),
        Some(PermissionMode::Maintenance)
    );
    assert_eq!(
        required_mode("memory_rebuild_embeddings"),
        Some(PermissionMode::Maintenance)
    );
    assert_eq!(
        required_mode("sync_cleanup"),
        Some(PermissionMode::Maintenance)
    );

    // Admin tools
    assert_eq!(required_mode("memory_delete"), Some(PermissionMode::Admin));
    assert_eq!(
        required_mode("memory_delete_batch"),
        Some(PermissionMode::Admin)
    );
    assert_eq!(
        required_mode("workspace_delete"),
        Some(PermissionMode::Admin)
    );
    assert_eq!(
        required_mode("memory_grant_access"),
        Some(PermissionMode::Admin)
    );
    assert_eq!(
        required_mode("memory_cache_clear"),
        Some(PermissionMode::Admin)
    );
}

#[test]
fn test_structured_permission_denial_payload() {
    let denial = permission_denial_for_mode("memory_delete", PermissionMode::ReadOnly).unwrap();

    let err = denial.get("error").expect("error object");
    assert_eq!(err["code"], "permission_denied");
    assert_eq!(err["tool"], "memory_delete");
    assert_eq!(err["current_mode"], "read_only");
    assert_eq!(err["required_mode"], "admin");
    assert!(err["message"].as_str().unwrap().contains("requires admin"));
}

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn test_per_request_permission_mode_override() {
    let _guard = ENV_LOCK.lock();
    std::env::remove_var("ENGRAM_PERMISSION_MODE");
    let ctx = setup_test_context();

    // 1. Calling memory_create normally without explicit mode succeeds
    let ok_val = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Test note",
            "memory_type": "note"
        }),
    );
    assert!(ok_val.get("id").is_some());

    // 2. Calling memory_create with _permission_mode: "read_only" fails closed
    let denied_val = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Another note",
            "memory_type": "note",
            "_permission_mode": "read_only"
        }),
    );
    assert_eq!(denied_val["error"]["code"], "permission_denied");
    assert_eq!(denied_val["error"]["current_mode"], "read_only");
    assert_eq!(denied_val["error"]["required_mode"], "scoped_write");

    // 3. Calling memory_delete with _permission_mode: "scoped_write" fails closed
    let delete_denied = dispatch(
        &ctx,
        "memory_delete",
        json!({
            "id": 1,
            "_permission_mode": "scoped_write"
        }),
    );
    assert_eq!(delete_denied["error"]["code"], "permission_denied");
    assert_eq!(delete_denied["error"]["current_mode"], "scoped_write");
    assert_eq!(delete_denied["error"]["required_mode"], "admin");
}

#[test]
fn test_env_permission_mode_enforcement() {
    let _guard = ENV_LOCK.lock();
    let ctx = setup_test_context();

    // Set ENGRAM_PERMISSION_MODE=read_only
    std::env::set_var("ENGRAM_PERMISSION_MODE", "read_only");
    assert_eq!(active_permission_mode(), Some(PermissionMode::ReadOnly));

    // Read-only tool still succeeds
    let list_res = dispatch(&ctx, "memory_list", json!({}));
    assert!(list_res.is_array());

    // Mutating tool is blocked
    let create_denied = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Blocked in read-only mode"
        }),
    );
    assert_eq!(create_denied["error"]["code"], "permission_denied");

    // Clean up env
    std::env::remove_var("ENGRAM_PERMISSION_MODE");
    assert_eq!(active_permission_mode(), None);
}

#[test]
fn test_permission_mode_status_tool() {
    let _guard = ENV_LOCK.lock();
    std::env::remove_var("ENGRAM_PERMISSION_MODE");
    let ctx = setup_test_context();

    // 1. Inspect status with no specific tool
    let status_val = dispatch(&ctx, "permission_mode_status", json!({}));
    assert_eq!(status_val["active_mode"], "unconstrained");
    assert!(status_val["total_tools_count"].as_u64().unwrap() > 50);

    // 2. Inspect status for specific tool
    let tool_val = dispatch(
        &ctx,
        "permission_mode_status",
        json!({
            "tool": "memory_delete"
        }),
    );
    assert_eq!(tool_val["tool"], "memory_delete");
    assert_eq!(tool_val["required_mode"], "admin");
    assert_eq!(tool_val["allowed"], true); // unconstrained allows all

    // 3. Inspect status when env is set to scoped_write
    std::env::set_var("ENGRAM_PERMISSION_MODE", "scoped_write");
    let scoped_val = dispatch(
        &ctx,
        "permission_mode_status",
        json!({
            "tool": "memory_delete"
        }),
    );
    assert_eq!(scoped_val["active_mode"], "scoped_write");
    assert_eq!(scoped_val["allowed"], false);

    let write_val = dispatch(
        &ctx,
        "permission_mode_status",
        json!({
            "tool": "memory_create"
        }),
    );
    assert_eq!(write_val["allowed"], true);

    std::env::remove_var("ENGRAM_PERMISSION_MODE");
}
