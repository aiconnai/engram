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

/// C1: permission modes and the persisted-workspace guard compose. The mode
/// check runs first and does not depend on the target row (no existence
/// oracle); a mode that allows the tool never lets a restricted principal
/// reach a foreign row.
#[test]
fn test_permission_mode_and_workspace_guard_compose_on_foreign_id() {
    use engram::auth::{PermissionSet, TokenClaims, TransportPrincipal, UserId};

    let _guard = ENV_LOCK.lock();
    std::env::remove_var("ENGRAM_PERMISSION_MODE");
    let mut ctx = setup_test_context();
    let foreign = dispatch(
        &ctx,
        "memory_create",
        json!({"content": "foreign payroll", "workspace": "tenant-b"}),
    )["id"]
        .as_i64()
        .expect("seed foreign memory");
    let missing = foreign + 10_000;

    let principal = |permissions: PermissionSet| {
        TransportPrincipal::from_token_claims(TokenClaims {
            user_id: UserId::from_string("agent-a"),
            key_id: "key-agent-a".to_string(),
            permissions,
            namespace: Some("tenant-a".to_string()),
            issued_at: chrono::Utc::now(),
            expires_at: None,
        })
        .expect("token principal")
    };

    // Insufficient mode: identical permission_denied for foreign and missing.
    ctx.principal = Some(principal(PermissionSet::read_only()));
    for id in [foreign, missing] {
        let denied = dispatch(
            &ctx,
            "memory_delete",
            json!({"id": id, "workspace": "tenant-a"}),
        );
        assert_eq!(denied["error"]["code"], "permission_denied", "{denied}");
        assert_eq!(denied["error"]["required_mode"], "admin");
    }

    // Per-request override narrows an admin principal before any lookup.
    ctx.principal = Some(principal(PermissionSet::admin()));
    let narrowed = dispatch(
        &ctx,
        "memory_update",
        json!({"id": foreign, "workspace": "tenant-a", "content": "x", "_permission_mode": "read_only"}),
    );
    assert_eq!(narrowed["error"]["code"], "permission_denied", "{narrowed}");

    // Sufficient mode: the workspace guard still refuses the foreign row.
    let refused = dispatch(
        &ctx,
        "memory_delete",
        json!({"id": foreign, "workspace": "tenant-a"}),
    );
    assert_eq!(refused["error"]["code"], "not_found", "{refused}");

    ctx.principal = None;
    let still_there = dispatch(&ctx, "memory_get", json!({"id": foreign}));
    assert_eq!(still_there["content"], "foreign payroll", "{still_there}");
}

/// Every name the dispatcher routes, as written in `dispatch`.
fn dispatchable_tool_names() -> Vec<String> {
    let source = include_str!("../src/mcp/handlers/mod.rs");
    let body = source
        .split("pub fn dispatch(")
        .nth(1)
        .expect("dispatch function");
    let mut names = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with('"') || !trimmed.contains("=>") {
            continue;
        }
        let arms = trimmed.split("=>").next().unwrap_or_default();
        for part in arms.split('|') {
            let name = part.trim().trim_matches('"');
            if !name.is_empty() {
                names.push(name.to_string());
            }
        }
    }
    assert!(
        names.len() > 200,
        "dispatch parse found only {}",
        names.len()
    );
    names
}

/// C1 review: a dispatchable name must never skip the permission mode. Alias
/// arms inherit the canonical tool's mode; nothing dispatchable maps to `None`.
#[test]
fn test_every_dispatchable_name_has_a_permission_mode() {
    for name in dispatchable_tool_names() {
        assert!(
            required_mode(&name).is_some(),
            "dispatchable tool `{name}` has no permission mode (fail-open)"
        );
    }
    for (alias, canonical) in [
        ("graph_predict_links", "memory_predict_links"),
        ("graph_cluster_concepts", "memory_cluster_concepts"),
        ("memory_seed", "context_seed"),
    ] {
        assert_eq!(required_mode(alias), required_mode(canonical), "{alias}");
    }
}

/// C1 review: read_only mode must not let predict-links write crossrefs
/// through either name, nor sync_state persist a version.
#[test]
fn test_read_only_mode_blocks_write_flags_on_read_tools_and_aliases() {
    let _guard = ENV_LOCK.lock();
    let ctx = setup_test_context();
    for content in ["rust async runtime tokio", "rust async runtime tokio tasks"] {
        dispatch(&ctx, "memory_create", json!({"content": content}));
    }
    let crossrefs = |ctx: &HandlerContext| -> i64 {
        ctx.storage
            .with_connection(|conn| {
                Ok(conn.query_row("SELECT COUNT(*) FROM crossrefs", [], |r| r.get(0))?)
            })
            .expect("crossref count")
    };
    let before = crossrefs(&ctx);

    std::env::set_var("ENGRAM_PERMISSION_MODE", "read_only");
    let mut responses = Vec::new();
    for tool in ["graph_predict_links", "memory_predict_links"] {
        responses.push((
            tool,
            dispatch(
                &ctx,
                tool,
                json!({"auto_apply": true, "min_confidence": 0.0}),
            ),
        ));
    }
    responses.push((
        "sync_state",
        dispatch(
            &ctx,
            "sync_state",
            json!({"agent_id": "a", "update_version": 9}),
        ),
    ));
    let plain = dispatch(&ctx, "memory_predict_links", json!({}));
    std::env::remove_var("ENGRAM_PERMISSION_MODE");

    for (tool, response) in responses {
        assert_eq!(
            response["error"]["code"], "permission_denied",
            "{tool}: {response}"
        );
        assert_eq!(response["error"]["required_mode"], "scoped_write", "{tool}");
    }
    assert_eq!(before, crossrefs(&ctx), "crossrefs written under read_only");
    assert_ne!(
        plain["error"]["code"], "permission_denied",
        "read-only prediction stays allowed: {plain}"
    );
}
