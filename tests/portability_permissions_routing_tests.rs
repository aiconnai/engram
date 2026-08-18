//! Integration tests for:
//! 1. Markdown & Obsidian Portability Engine (RFC 0004)
//! 2. Permission Modes for MCP Surfaces (RFC 0010)
//! 3. Unified Model Routing Contract (RFC 0011)

use std::sync::Arc;

use engram::embedding::{EmbeddingCache, TfIdfEmbedder};
use engram::mcp::handlers::{dispatch, HandlerContext};
use engram::mcp::permission::{permission_denial_for_mode, PermissionMode};
use engram::mcp::tools::get_tool_definitions;
use engram::portability::{
    export_markdown, import_markdown, ExportGrouping, ExportOptions, ImportOptions,
};
use engram::routing::inspect_model_routing;
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::queries::create_memory;
use engram::storage::Storage;
use engram::types::{CreateMemoryInput, EmbeddingConfig, MemoryType};
use serde_json::json;

fn create_test_context() -> HandlerContext {
    let storage = Storage::open_in_memory().expect("in-memory sqlite");
    HandlerContext {
        storage,
        embedder: Arc::new(TfIdfEmbedder::new(128)),
        fuzzy_engine: Arc::new(parking_lot::Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(AdaptiveCacheConfig::default())),
        hnsw_index: Arc::new(parking_lot::RwLock::new(engram::search::HnswIndex::new(
            engram::search::HnswConfig::new(128, engram::search::VectorMetric::Cosine),
        ))),
        #[cfg(feature = "meilisearch")]
        meili: None,
        #[cfg(feature = "meilisearch")]
        meili_indexer: None,
        #[cfg(feature = "meilisearch")]
        meili_sync_interval: 300,
        #[cfg(feature = "langfuse")]
        langfuse_runtime: Arc::new(
            tokio::runtime::Builder::new_current_thread()
                .build()
                .expect("langfuse runtime"),
        ),
        progress_reporter: None,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Track 1: Markdown & Obsidian Portability Engine (RFC 0004)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_markdown_portability_export_and_import_roundtrip() {
    let ctx = create_test_context();

    // 1. Create test memories
    let _id1 = ctx
        .storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Architecture decision: Use Rust for persistent cognitive core"
                        .to_string(),
                    memory_type: MemoryType::Decision,
                    workspace: Some("portability_test".to_string()),
                    tags: vec!["rust".to_string(), "architecture".to_string()],
                    importance: Some(0.9),
                    ..Default::default()
                },
            )
        })
        .expect("create memory 1");

    let _id2 = ctx
        .storage
        .with_transaction(|conn| {
            create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Guideline: All public MCP tools require explicit schemas".to_string(),
                    memory_type: MemoryType::Learning,
                    workspace: Some("portability_test".to_string()),
                    tags: vec!["mcp".to_string(), "api".to_string()],
                    importance: Some(0.75),
                    ..Default::default()
                },
            )
        })
        .expect("create memory 2");

    let temp_dir = tempfile::tempdir().expect("tempdir");

    // 2. Export to Markdown with different groupings
    let export_report = export_markdown(
        &ctx.storage,
        &ExportOptions {
            output_dir: temp_dir.path().to_path_buf(),
            grouping: ExportGrouping::Type,
            workspace: Some("portability_test".to_string()),
        },
    )
    .expect("export markdown type grouping");

    assert_eq!(export_report.files_written, 3); // 2 memories + index overview

    let temp_ws_dir = tempfile::tempdir().expect("temp ws dir");
    let export_ws_report = export_markdown(
        &ctx.storage,
        &ExportOptions {
            output_dir: temp_ws_dir.path().to_path_buf(),
            grouping: ExportGrouping::Workspace,
            workspace: Some("portability_test".to_string()),
        },
    )
    .expect("export markdown workspace grouping");
    assert_eq!(export_ws_report.files_written, 3);

    let temp_entity_dir = tempfile::tempdir().expect("temp entity dir");
    let export_entity_report = export_markdown(
        &ctx.storage,
        &ExportOptions {
            output_dir: temp_entity_dir.path().to_path_buf(),
            grouping: ExportGrouping::Entity,
            workspace: Some("portability_test".to_string()),
        },
    )
    .expect("export markdown entity grouping");
    assert_eq!(export_entity_report.files_written, 3);

    // 3. Dry-run import
    let dry_run_report = import_markdown(
        &ctx.storage,
        &ImportOptions {
            input_dir: temp_dir.path().to_path_buf(),
            dry_run: true,
            target_workspace: Some("portability_test".to_string()),
        },
    )
    .expect("dry-run import");

    assert_eq!(dry_run_report.in_sync, 2);
    assert_eq!(dry_run_report.conflict, 0);

    // 4. MCP tool dispatch export & import
    let temp_dir_mcp = tempfile::tempdir().expect("tempdir mcp");
    let export_res = dispatch(
        &ctx,
        "memory_export_markdown",
        json!({
            "workspace": "portability_test",
            "output_dir": temp_dir_mcp.path().to_str().unwrap(),
            "include_links": true
        }),
    );
    assert_eq!(export_res["files_written"], 3);
    assert_eq!(export_res["memories_exported"], 2);

    let import_res = dispatch(
        &ctx,
        "memory_import_markdown",
        json!({
            "input_dir": temp_dir_mcp.path().to_str().unwrap(),
            "workspace": "portability_test",
            "confirm": false
        }),
    );
    assert_eq!(import_res["in_sync"], 2);
    assert_eq!(import_res["conflicts"], 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Track 2: Permission Modes for MCP Surfaces (RFC 0010)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_mcp_permission_modes_enforcement_and_denial_shapes() {
    // 1. ReadOnly mode checks
    assert!(permission_denial_for_mode("memory_get", PermissionMode::ReadOnly).is_none());
    assert!(permission_denial_for_mode("memory_search", PermissionMode::ReadOnly).is_none());
    assert!(permission_denial_for_mode("model_routing_status", PermissionMode::ReadOnly).is_none());
    assert!(
        permission_denial_for_mode("memory_export_markdown", PermissionMode::ReadOnly).is_none()
    );

    let ro_write_denial = permission_denial_for_mode("memory_create", PermissionMode::ReadOnly)
        .expect("should deny memory_create in read_only");
    assert_eq!(ro_write_denial["error"]["code"], "permission_denied");
    assert_eq!(ro_write_denial["error"]["tool"], "memory_create");
    assert_eq!(ro_write_denial["error"]["current_mode"], "read_only");
    assert_eq!(ro_write_denial["error"]["required_mode"], "scoped_write");

    let ro_delete_denial = permission_denial_for_mode("memory_delete", PermissionMode::ReadOnly)
        .expect("should deny memory_delete in read_only");
    assert_eq!(ro_delete_denial["error"]["required_mode"], "admin");

    // 2. ScopedWrite mode checks
    assert!(permission_denial_for_mode("memory_create", PermissionMode::ScopedWrite).is_none());
    assert!(permission_denial_for_mode("memory_update", PermissionMode::ScopedWrite).is_none());
    let sw_maint_denial = permission_denial_for_mode("lifecycle_run", PermissionMode::ScopedWrite)
        .expect("should deny lifecycle_run in scoped_write");
    assert_eq!(sw_maint_denial["error"]["required_mode"], "maintenance");

    // 3. Maintenance mode checks
    assert!(permission_denial_for_mode("lifecycle_run", PermissionMode::Maintenance).is_none());
    assert!(
        permission_denial_for_mode("memory_cleanup_expired", PermissionMode::Maintenance).is_none()
    );
    let maint_admin_denial =
        permission_denial_for_mode("memory_delete", PermissionMode::Maintenance)
            .expect("should deny memory_delete in maintenance");
    assert_eq!(maint_admin_denial["error"]["required_mode"], "admin");

    // 4. Admin mode checks
    assert!(permission_denial_for_mode("memory_delete", PermissionMode::Admin).is_none());
    assert!(permission_denial_for_mode("memory_create", PermissionMode::Admin).is_none());
    assert!(permission_denial_for_mode("lifecycle_run", PermissionMode::Admin).is_none());
}

// ─────────────────────────────────────────────────────────────────────────────
// Track 3: Unified Model Routing Contract (RFC 0011)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn test_unified_model_routing_inspection_and_mcp_dispatch() {
    let ctx = create_test_context();

    // 1. Inspect programmatic model routing
    let config = EmbeddingConfig {
        model: "tfidf".to_string(),
        dimensions: 128,
        ..Default::default()
    };
    let report = inspect_model_routing(&config);

    assert_eq!(report.primary_embedding_provider, "tfidf");
    assert_eq!(report.primary_dimensions, 128);
    assert!(report.is_fully_local);
    assert!(report
        .providers
        .iter()
        .any(|p| p.name == "tfidf" && p.is_available));

    // 2. Dispatch via MCP tool
    let tools = get_tool_definitions();
    assert!(
        tools.iter().any(|t| t.name == "model_routing_status"),
        "model_routing_status must be registered in MCP tools"
    );

    let res = dispatch(
        &ctx,
        "model_routing_status",
        json!({"model": "tfidf", "dimensions": 128}),
    );
    assert_eq!(res["status"], "success");
    assert_eq!(res["routing"]["primary_embedding_provider"], "tfidf");
    assert_eq!(res["routing"]["primary_dimensions"], 128);
    assert_eq!(res["routing"]["is_fully_local"], true);
}
