//! Integration tests for Central Workspace & Scoped Auth Enforcement (RFC 0006 — Tier 1).

use std::sync::Arc;

use chrono::Utc;
use parking_lot::Mutex;
use serde_json::json;

use engram::auth::{PermissionSet, TokenClaims, TransportPrincipal, UserId};
use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::error::ToolError;
use engram::mcp::handlers::{dispatch, HandlerContext};
use engram::mcp::permission::check_tool_authorization;
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::scope_grants::grant_scope_access;
use engram::storage::Storage;
use engram::types::EmbeddingConfig;

fn test_ctx() -> HandlerContext {
    let storage = Storage::open_in_memory().expect("in-memory storage");
    let embedder = create_embedder(&EmbeddingConfig::default()).expect("tfidf embedder");
    HandlerContext {
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

fn create_principal(
    user_id: &str,
    namespace: Option<&str>,
    permissions: PermissionSet,
) -> TransportPrincipal {
    TransportPrincipal::from_token_claims(TokenClaims {
        user_id: UserId::from_string(user_id),
        key_id: format!("key-{user_id}"),
        permissions,
        namespace: namespace.map(str::to_string),
        issued_at: Utc::now(),
        expires_at: None,
    })
    .expect("valid token claims")
}

#[test]
fn test_principal_workspace_namespace_isolation() {
    let principal = create_principal(
        "agent-finance",
        Some("finance_workspace"),
        PermissionSet::standard_user(),
    );

    // 1. Permitted own workspace
    let res_ok = check_tool_authorization(
        None,
        "memory_create",
        &json!({"content": "Q3 balance report", "workspace": "finance_workspace"}),
        Some(&principal),
    );
    assert!(res_ok.is_none(), "should permit own workspace access");

    // 2. Denied foreign workspace
    let res_denied = check_tool_authorization(
        None,
        "memory_create",
        &json!({"content": "Access foreign data", "workspace": "engineering_workspace"}),
        Some(&principal),
    );
    assert!(res_denied.is_some(), "should deny foreign workspace access");
    let denial = res_denied.unwrap();
    assert!(ToolError::is_error_response(&denial));
    assert_eq!(denial["error"]["code"], "permission_denied");

    // 3. Denied global query when namespace restricted
    let res_global = check_tool_authorization(
        None,
        "memory_search",
        &json!({"query": "cross workspace search", "global": true}),
        Some(&principal),
    );
    assert!(
        res_global.is_some(),
        "should deny global search across namespaces"
    );
}

#[test]
fn test_principal_permission_mode_enforcement() {
    let read_only_principal = create_principal("agent-readonly", None, PermissionSet::read_only());

    // 1. Read tool allowed
    let read_check = check_tool_authorization(
        None,
        "memory_search",
        &json!({"query": "hello"}),
        Some(&read_only_principal),
    );
    assert!(read_check.is_none(), "read-only principal can search");

    // 2. Write tool denied
    let write_check = check_tool_authorization(
        None,
        "memory_create",
        &json!({"content": "attempt write"}),
        Some(&read_only_principal),
    );
    assert!(
        write_check.is_some(),
        "read-only principal cannot create memories"
    );
    let denial = write_check.unwrap();
    assert_eq!(denial["error"]["code"], "permission_denied");
    assert_eq!(denial["error"]["current_mode"], "read_only");
    assert_eq!(denial["error"]["required_mode"], "scoped_write");

    // 3. Admin tool denied
    let admin_check = check_tool_authorization(
        None,
        "memory_delete",
        &json!({"id": 123}),
        Some(&read_only_principal),
    );
    assert!(admin_check.is_some(), "read-only principal cannot delete");
    assert_eq!(admin_check.unwrap()["error"]["required_mode"], "admin");
}

#[test]
fn test_hierarchical_scope_grant_enforcement_in_dispatch() {
    let ctx = test_ctx();
    let scope_target = "global/org:acme/project:engram";

    // Setup initial scope grant for agent-1 (read-only on ancestor "global/org:acme")
    ctx.storage
        .with_connection(|conn| {
            grant_scope_access(conn, "agent-1", "global/org:acme", "read", Some("admin"))?;
            Ok(())
        })
        .expect("grant setup");

    // 1. Read check inherits ancestor permission and succeeds
    ctx.storage
        .with_connection(|conn| {
            let res = check_tool_authorization(
                Some(conn),
                "memory_search",
                &json!({
                    "query": "architecture",
                    "agent_id": "agent-1",
                    "scope_path": scope_target
                }),
                None,
            );
            assert!(
                res.is_none(),
                "ancestor read grant permits child scope read"
            );
            Ok(())
        })
        .unwrap();

    // 2. Write check fails because agent-1 only has read permission
    ctx.storage
        .with_connection(|conn| {
            let res = check_tool_authorization(
                Some(conn),
                "memory_create",
                &json!({
                    "content": "new memory",
                    "agent_id": "agent-1",
                    "scope_path": scope_target
                }),
                None,
            );
            assert!(res.is_some(), "read-only grant should deny write operation");
            let denial = res.unwrap();
            assert_eq!(denial["error"]["code"], "permission_denied");
            assert_eq!(denial["error"]["details"]["required_permission"], "write");
            assert_eq!(denial["error"]["details"]["agent_id"], "agent-1");
            assert_eq!(denial["error"]["details"]["scope"], scope_target);
            Ok(())
        })
        .unwrap();

    // 3. Upgrade grant to write and verify dispatch succeeds
    ctx.storage
        .with_connection(|conn| {
            grant_scope_access(conn, "agent-1", scope_target, "write", Some("admin"))?;
            Ok(())
        })
        .expect("upgrade grant");

    let create_result = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Scoped memory content",
            "agent_id": "agent-1",
            "scope_path": scope_target
        }),
    );
    assert!(
        create_result.get("error").is_none(),
        "dispatch should permit memory creation after write grant: {create_result}"
    );
    assert!(create_result.get("id").is_some());

    // 4. Unauthorized agent-2 fails through dispatch
    let unauth_result = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Unauthorized attempt",
            "agent_id": "agent-2",
            "scope_path": scope_target
        }),
    );
    assert!(ToolError::is_error_response(&unauth_result));
    assert_eq!(unauth_result["error"]["code"], "permission_denied");
    assert_eq!(unauth_result["error"]["details"]["agent_id"], "agent-2");
}
