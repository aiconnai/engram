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

#[test]
fn test_dispatch_enforces_transport_principal() {
    let mut ctx = test_ctx();
    let principal = create_principal(
        "agent-scoped",
        Some("tenant-a"),
        PermissionSet::standard_user(),
    );
    ctx.principal = Some(principal);

    // 1. Dispatch to allowed workspace succeeds
    let ok_res = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Allowed tenant data",
            "workspace": "tenant-a"
        }),
    );
    assert!(
        ok_res.get("error").is_none(),
        "dispatch must allow operation within principal workspace: {ok_res}"
    );
    assert!(ok_res.get("id").is_some());

    // 2. Dispatch to foreign workspace is rejected by central authorization in dispatch()
    let denied_res = dispatch(
        &ctx,
        "memory_create",
        json!({
            "content": "Cross-tenant intrusion attempt",
            "workspace": "tenant-b"
        }),
    );
    assert!(
        ToolError::is_error_response(&denied_res),
        "dispatch must reject operation in foreign workspace"
    );
    assert_eq!(denied_res["error"]["code"], "permission_denied");
}

// ── C1: persisted-workspace authorization on the real dispatch path ─────────
//
// These tests drive `dispatch` (the function every transport ends in) with a
// transport principal attached, and also call the handlers directly to prove
// the in-transaction check does not depend on the dispatcher guard. Every
// negative case asserts: structured error, no content leak, foreign and
// nonexistent IDs indistinguishable, and no side effects on the target row.

const OWN_WS: &str = "tenant-a";
const FOREIGN_WS: &str = "tenant-b";
const FOREIGN_SECRET: &str = "zebracorn foreign payroll secret";
const OWN_CONTENT: &str = "zebracorn own roadmap note";

fn seed_memory(ctx: &HandlerContext, content: &str, workspace: &str) -> i64 {
    let created = dispatch(
        ctx,
        "memory_create",
        json!({"content": content, "workspace": workspace, "importance": 0.9}),
    );
    created["id"]
        .as_i64()
        .unwrap_or_else(|| panic!("seed memory_create failed: {created}"))
}

/// Full persisted state of one memory row plus rows that reference it.
fn row_state(ctx: &HandlerContext, id: i64) -> String {
    ctx.storage
        .with_connection(|conn| {
            let row: Vec<rusqlite::types::Value> = conn
                .query_row(
                    "SELECT content, workspace, access_count, last_accessed_at, updated_at,
                            stability, version, importance, valid_to, lifecycle_state, tier
                     FROM memories WHERE id = ?1",
                    [id],
                    |r| (0..11).map(|i| r.get(i)).collect(),
                )
                .unwrap_or_default();
            let reinforcements: i64 = conn.query_row(
                "SELECT COUNT(*) FROM memory_reinforcements WHERE memory_id = ?1",
                [id],
                |r| r.get(0),
            )?;
            let crossrefs: i64 = conn.query_row(
                "SELECT COUNT(*) FROM crossrefs
                 WHERE (from_id = ?1 OR to_id = ?1) AND valid_to IS NULL",
                [id],
                |r| r.get(0),
            )?;
            Ok(format!(
                "{row:?} reinforcements={reinforcements} crossrefs={crossrefs}"
            ))
        })
        .expect("read row state")
}

fn memory_count(ctx: &HandlerContext) -> i64 {
    ctx.storage
        .with_connection(|conn| {
            Ok(conn.query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))?)
        })
        .expect("memory count")
}

fn nonexistent_id(ctx: &HandlerContext) -> i64 {
    ctx.storage
        .with_connection(|conn| {
            let max: i64 =
                conn.query_row("SELECT COALESCE(MAX(id), 0) FROM memories", [], |r| {
                    r.get(0)
                })?;
            Ok(max + 10_000)
        })
        .expect("max id")
}

/// Replace the caller-supplied id so foreign and nonexistent denials compare.
fn normalized(response: &serde_json::Value, id: i64) -> String {
    // Replace only whole-token occurrences of the caller-supplied id.
    let id = id.to_string();
    response
        .to_string()
        .replace(&format!("'{id}'"), "'<ID>'")
        .replace(&format!("\\\"{id}\\\""), "\\\"<ID>\\\"")
        .replace(&format!("\"{id}\""), "\"<ID>\"")
        .replace(&format!(": {id}\""), ": <ID>\"")
        .replace(&format!(": {id}\\\""), ": <ID>\\\"")
}

fn dispatch_as(
    ctx: &mut HandlerContext,
    principal: &TransportPrincipal,
    tool: &str,
    params: serde_json::Value,
) -> serde_json::Value {
    ctx.principal = Some(principal.clone());
    let response = dispatch(ctx, tool, params);
    ctx.principal = None;
    response
}

fn assert_not_found_without_leak(response: &serde_json::Value, context: &str) {
    assert!(
        ToolError::is_error_response(response),
        "{context}: expected structured error, got {response}"
    );
    assert_eq!(
        response["error"]["code"], "not_found",
        "{context}: foreign IDs must be indistinguishable from missing ones: {response}"
    );
    assert!(
        !response.to_string().contains("payroll"),
        "{context}: foreign content leaked: {response}"
    );
}

#[test]
fn test_claimed_workspace_does_not_authorize_foreign_id() {
    let mut ctx = test_ctx();
    let own_id = seed_memory(&ctx, OWN_CONTENT, OWN_WS);
    let foreign_id = seed_memory(&ctx, FOREIGN_SECRET, FOREIGN_WS);
    let default_id = seed_memory(&ctx, "zebracorn default public note", "default");
    let foreign_default_id = seed_memory(&ctx, FOREIGN_SECRET, "private");
    let missing_id = nonexistent_id(&ctx);

    let authenticated = create_principal("agent-a", Some(OWN_WS), PermissionSet::standard_user());
    let anonymous = TransportPrincipal::anonymous_loopback();

    // (principal, claimed workspace, own id, foreign id)
    let cases = [
        ("authenticated", &authenticated, OWN_WS, own_id, foreign_id),
        (
            "anonymous_loopback",
            &anonymous,
            "default",
            default_id,
            foreign_default_id,
        ),
    ];

    for (label, principal, claimed, legit_id, target_id) in cases {
        for tool in ["memory_get", "memory_get_public"] {
            let context = format!("{label}/{tool}");
            let before = row_state(&ctx, target_id);

            let foreign = dispatch_as(
                &mut ctx,
                principal,
                tool,
                json!({"id": target_id, "workspace": claimed}),
            );
            assert_not_found_without_leak(&foreign, &context);
            assert_eq!(
                before,
                row_state(&ctx, target_id),
                "{context}: denied read must not reinforce or touch access tracking"
            );

            let missing = dispatch_as(
                &mut ctx,
                principal,
                tool,
                json!({"id": missing_id, "workspace": claimed}),
            );
            assert_eq!(
                normalized(&foreign, target_id),
                normalized(&missing, missing_id),
                "{context}: foreign and nonexistent IDs must be indistinguishable"
            );

            let legit = dispatch_as(
                &mut ctx,
                principal,
                tool,
                json!({"id": legit_id, "workspace": claimed}),
            );
            assert!(
                !ToolError::is_error_response(&legit),
                "{context}: legitimate ID in own workspace must be readable: {legit}"
            );
            assert_eq!(legit["id"], legit_id, "{context}: wrong memory returned");
        }

        // Version history is another by-ID read of the same row.
        let versions = dispatch_as(
            &mut ctx,
            principal,
            "memory_versions",
            json!({"id": target_id, "workspace": claimed}),
        );
        assert_not_found_without_leak(&versions, &format!("{label}/memory_versions"));
    }
}

#[test]
fn test_foreign_id_mutations_denied_without_side_effects() {
    let mut ctx = test_ctx();
    let own_id = seed_memory(&ctx, OWN_CONTENT, OWN_WS);
    let foreign_id = seed_memory(&ctx, FOREIGN_SECRET, FOREIGN_WS);
    let foreign_peer = seed_memory(&ctx, "zebracorn foreign peer", FOREIGN_WS);
    let linked = dispatch(
        &ctx,
        "memory_link",
        json!({"from_id": foreign_id, "to_id": foreign_peer, "edge_type": "related_to"}),
    );
    assert!(
        !ToolError::is_error_response(&linked),
        "seed link: {linked}"
    );

    // Namespaced admin: every permission mode allows the tool, so only the
    // workspace check can stop the call.
    let writer = create_principal("agent-a", Some(OWN_WS), PermissionSet::admin());

    let attempts = [
        (
            "memory_update",
            json!({"id": foreign_id, "workspace": OWN_WS, "content": "overwritten"}),
        ),
        (
            "memory_delete",
            json!({"id": foreign_id, "workspace": OWN_WS}),
        ),
        (
            "memory_delete_batch",
            json!({"ids": [own_id, foreign_id], "workspace": OWN_WS}),
        ),
        (
            "memory_link",
            json!({"from_id": own_id, "to_id": foreign_id, "workspace": OWN_WS}),
        ),
        (
            "memory_unlink",
            json!({"from_id": foreign_id, "to_id": foreign_peer, "workspace": OWN_WS}),
        ),
        (
            "workspace_move",
            json!({"id": foreign_id, "workspace": OWN_WS}),
        ),
        // Creation tools that reference another memory by ID.
        (
            "memory_create",
            json!({"content": "summary", "workspace": OWN_WS, "summary_of_id": foreign_id}),
        ),
        (
            "memory_create_section",
            json!({"title": "s", "content": "c", "workspace": OWN_WS, "parent_id": foreign_id}),
        ),
    ];

    for (tool, params) in attempts {
        let own_before = row_state(&ctx, own_id);
        let foreign_before = row_state(&ctx, foreign_id);
        let peer_before = row_state(&ctx, foreign_peer);
        let count_before = memory_count(&ctx);

        let response = dispatch_as(&mut ctx, &writer, tool, params);
        assert_eq!(
            count_before,
            memory_count(&ctx),
            "{tool}: row created or removed"
        );

        assert_not_found_without_leak(&response, tool);
        assert_eq!(
            foreign_before,
            row_state(&ctx, foreign_id),
            "{tool}: foreign row changed"
        );
        assert_eq!(
            peer_before,
            row_state(&ctx, foreign_peer),
            "{tool}: foreign peer changed"
        );
        assert_eq!(
            own_before,
            row_state(&ctx, own_id),
            "{tool}: partial mutation of own row"
        );
    }

    // Positive control: the same principal can mutate its own memory.
    let updated = dispatch_as(
        &mut ctx,
        &writer,
        "memory_update",
        json!({"id": own_id, "workspace": OWN_WS, "content": "own update"}),
    );
    assert_eq!(
        updated["content"], "own update",
        "own update failed: {updated}"
    );
}

#[test]
fn test_handler_checks_workspace_inside_mutation_transaction() {
    use engram::mcp::handlers::memory_crud;

    let mut ctx = test_ctx();
    let own_root = seed_memory(&ctx, OWN_CONTENT, OWN_WS);
    let foreign_id = seed_memory(&ctx, FOREIGN_SECRET, FOREIGN_WS);
    // An unrestricted caller chained a foreign memory under the own root.
    let chained = dispatch(
        &ctx,
        "memory_link",
        json!({"from_id": own_root, "to_id": foreign_id, "edge_type": "supersedes"}),
    );
    assert!(
        !ToolError::is_error_response(&chained),
        "seed chain: {chained}"
    );

    let writer = create_principal("agent-a", Some(OWN_WS), PermissionSet::admin());

    // Through dispatch: the root passes the pre-dispatch guard, but the chain
    // member is foreign, so the whole transaction must roll back.
    let own_before = row_state(&ctx, own_root);
    let foreign_before = row_state(&ctx, foreign_id);
    let cascade = dispatch_as(
        &mut ctx,
        &writer,
        "memory_delete",
        json!({"id": own_root, "workspace": OWN_WS, "cascade_chain": true}),
    );
    assert_not_found_without_leak(&cascade, "memory_delete cascade_chain");
    assert_eq!(
        own_before,
        row_state(&ctx, own_root),
        "cascade partially deleted root"
    );
    assert_eq!(
        foreign_before,
        row_state(&ctx, foreign_id),
        "cascade deleted foreign"
    );

    // Direct handler calls bypass the dispatcher guard entirely: the handler's
    // own transaction must still refuse the foreign row.
    ctx.principal = Some(writer);
    let direct = [
        memory_crud::memory_update(&ctx, json!({"id": foreign_id, "content": "overwritten"})),
        memory_crud::memory_delete(&ctx, json!({"id": foreign_id})),
        memory_crud::memory_delete_batch(&ctx, json!({"ids": [foreign_id]})),
        memory_crud::memory_get(&ctx, json!({"id": foreign_id})),
        memory_crud::memory_get_public(&ctx, json!({"id": foreign_id})),
    ];
    for response in &direct {
        assert_not_found_without_leak(response, "direct handler call");
    }
    // Handlers with the legacy `{"error": "<text>"}` shape still refuse the row.
    for response in [
        engram::mcp::handlers::graph::memory_link(
            &ctx,
            json!({"from_id": own_root, "to_id": foreign_id}),
        ),
        engram::mcp::handlers::graph::memory_unlink(
            &ctx,
            json!({"from_id": own_root, "to_id": foreign_id, "edge_type": "supersedes"}),
        ),
        engram::mcp::handlers::workspace::workspace_move(
            &ctx,
            json!({"id": foreign_id, "workspace": OWN_WS}),
        ),
    ] {
        let text = response["error"].as_str().unwrap_or_default();
        assert!(text.contains("not found"), "legacy handler: {response}");
        assert!(
            !response.to_string().contains("payroll"),
            "leak: {response}"
        );
    }
    assert_eq!(
        foreign_before,
        row_state(&ctx, foreign_id),
        "direct handler mutated"
    );
}

#[test]
fn test_restricted_principal_cannot_scope_unscoped_tool_with_claim() {
    let mut ctx = test_ctx();
    let own_id = seed_memory(&ctx, OWN_CONTENT, OWN_WS);
    let _foreign = seed_memory(&ctx, FOREIGN_SECRET, FOREIGN_WS);
    let reader = create_principal("agent-a", Some(OWN_WS), PermissionSet::read_only());

    // These tools ignore a `workspace` argument (no such parameter in their
    // schema), so the claim cannot scope them: conservative denial.
    for (tool, params) in [
        (
            "memory_export_graph",
            json!({"format": "json", "workspace": OWN_WS}),
        ),
        (
            "memory_export_graph",
            json!({"format": "json", "focus_id": own_id, "workspace": OWN_WS}),
        ),
        ("memory_related", json!({"id": own_id, "workspace": OWN_WS})),
        (
            "memory_get_full",
            json!({"id": own_id, "workspace": OWN_WS}),
        ),
    ] {
        let response = dispatch_as(&mut ctx, &reader, tool, params);
        assert!(
            ToolError::is_error_response(&response),
            "{tool}: expected denial, got {response}"
        );
        assert_eq!(
            response["error"]["code"], "permission_denied",
            "{tool}: {response}"
        );
        assert!(
            !response.to_string().contains("payroll"),
            "{tool}: foreign content leaked: {response}"
        );
    }
}

#[test]
fn test_listing_and_search_stay_in_claimed_workspace() {
    let mut ctx = test_ctx();
    let own_id = seed_memory(&ctx, OWN_CONTENT, OWN_WS);
    let _foreign = seed_memory(&ctx, FOREIGN_SECRET, FOREIGN_WS);
    let reader = create_principal("agent-a", Some(OWN_WS), PermissionSet::read_only());

    let listed = dispatch_as(
        &mut ctx,
        &reader,
        "memory_list",
        json!({"workspace": OWN_WS}),
    );
    assert!(
        !ToolError::is_error_response(&listed),
        "memory_list: {listed}"
    );
    assert!(
        listed.to_string().contains(OWN_CONTENT),
        "own row missing: {listed}"
    );
    assert!(
        !listed.to_string().contains("payroll"),
        "list leaked: {listed}"
    );

    let searched = dispatch_as(
        &mut ctx,
        &reader,
        "memory_search",
        json!({"query": "zebracorn", "workspace": OWN_WS}),
    );
    assert!(
        !ToolError::is_error_response(&searched),
        "memory_search: {searched}"
    );
    assert!(
        !searched.to_string().contains("payroll"),
        "search leaked: {searched}"
    );

    // A foreign workspace claim is denied before any listing happens.
    for tool in ["memory_list", "memory_search"] {
        let denied = dispatch_as(
            &mut ctx,
            &reader,
            tool,
            json!({"query": "zebracorn", "workspace": FOREIGN_WS}),
        );
        assert_eq!(
            denied["error"]["code"], "permission_denied",
            "{tool}: {denied}"
        );
    }
    let _ = own_id;
}

#[test]
fn test_case_variant_workspace_keys_do_not_widen_scope() {
    let mut ctx = test_ctx();
    let _own = seed_memory(&ctx, OWN_CONTENT, OWN_WS);
    let _foreign = seed_memory(&ctx, FOREIGN_SECRET, FOREIGN_WS);
    let reader = create_principal("agent-a", Some(OWN_WS), PermissionSet::read_only());

    // Only the lowercase key is a workspace claim; a case variant alone is no
    // claim at all, so a namespaced principal is denied conservatively.
    for key in ["Workspace", "WORKSPACE"] {
        let mut params = serde_json::Map::new();
        params.insert(key.to_string(), json!(FOREIGN_WS));
        let response = dispatch_as(
            &mut ctx,
            &reader,
            "memory_list",
            serde_json::Value::Object(params),
        );
        assert_eq!(
            response["error"]["code"], "permission_denied",
            "{key}: {response}"
        );
    }

    // Mixed keys: the handler honors only `workspace`, which is the own one.
    let mixed = dispatch_as(
        &mut ctx,
        &reader,
        "memory_list",
        json!({"workspace": OWN_WS, "WORKSPACE": FOREIGN_WS, "Workspace": FOREIGN_WS}),
    );
    assert!(!ToolError::is_error_response(&mixed), "mixed keys: {mixed}");
    assert!(
        !mixed.to_string().contains("payroll"),
        "mixed keys leaked: {mixed}"
    );
}

#[test]
fn test_unrestricted_and_missing_principal_contracts() {
    let mut ctx = test_ctx();
    let foreign_id = seed_memory(&ctx, FOREIGN_SECRET, FOREIGN_WS);

    // stdio contract: no transport principal means the local process owner.
    let stdio = dispatch(&ctx, "memory_get", json!({"id": foreign_id}));
    assert_eq!(stdio["id"], foreign_id, "stdio owner read failed: {stdio}");

    // A principal without a namespace (process bearer / unscoped token) is
    // authorized for every workspace.
    let unscoped = create_principal("operator", None, PermissionSet::standard_user());
    let read = dispatch_as(&mut ctx, &unscoped, "memory_get", json!({"id": foreign_id}));
    assert_eq!(read["id"], foreign_id, "unscoped read failed: {read}");
}

// ── C1 review round 1 ────────────────────────────────────────────────────────

/// `context_*` tools declare `workspace` as an alias for `workspace_path_hash`
/// (a caller-asserted scope compared by the artifact policy), not the memory
/// workspace. A restricted principal's claim cannot bind them, so they are
/// refused before any lookup (no existence oracle on artifact IDs).
#[test]
fn test_workspace_alias_context_tools_denied_for_restricted_principal() {
    let mut ctx = test_ctx();
    let writer = create_principal("agent-a", Some(OWN_WS), PermissionSet::standard_user());
    let anonymous = TransportPrincipal::anonymous_loopback();

    let calls = [
        (
            "context_get_artifact",
            json!({"artifact_id": "artifact-x", "reason": "audit", "workspace_path_hash": "other-hash"}),
        ),
        (
            "context_search",
            json!({"query": "deploy", "workspace_path_hash": "other-hash"}),
        ),
        (
            "context_build_bundle",
            json!({"workspace_path_hash": "other-hash"}),
        ),
        (
            "context_record",
            json!({"kind": "note", "workspace_path_hash": "other-hash"}),
        ),
        (
            "context_record_artifact",
            json!({"kind": "log", "content": "x", "workspace_path_hash": "other-hash"}),
        ),
    ];

    for (label, principal, claim) in [
        ("namespaced", &writer, OWN_WS),
        ("anonymous", &anonymous, "default"),
    ] {
        for (tool, base) in &calls {
            let mut params = base.clone();
            params["workspace"] = json!(claim);
            let response = dispatch_as(&mut ctx, principal, tool, params);
            assert_eq!(
                response["error"]["code"], "permission_denied",
                "{label}/{tool}: {response}"
            );
            if label == "namespaced" || !tool.starts_with("context_record") {
                assert_eq!(
                    response["error"]["details"]["reason"], "tool_not_workspace_scoped",
                    "{label}/{tool}: {response}"
                );
            }
        }
    }
}

/// Positive controls: the guards never block a restricted principal acting on
/// its own rows.
#[test]
fn test_restricted_principal_own_id_positive_controls() {
    let mut ctx = test_ctx();
    let writer = create_principal("agent-a", Some(OWN_WS), PermissionSet::admin());
    let a = seed_memory(&ctx, "own a", OWN_WS);
    let b = seed_memory(&ctx, "own b", OWN_WS);
    let c = seed_memory(&ctx, "own c", OWN_WS);
    let root = seed_memory(&ctx, "own chain root", OWN_WS);
    let member = seed_memory(&ctx, "own chain member", OWN_WS);
    let ok = |response: &serde_json::Value, what: &str| {
        assert!(
            !ToolError::is_error_response(response),
            "{what}: {response}"
        );
    };

    let moved = dispatch_as(
        &mut ctx,
        &writer,
        "workspace_move",
        json!({"id": a, "workspace": OWN_WS}),
    );
    ok(&moved, "workspace_move");
    assert_eq!(moved["memory"]["workspace"], OWN_WS);

    let linked = dispatch_as(
        &mut ctx,
        &writer,
        "memory_link",
        json!({"from_id": a, "to_id": b, "workspace": OWN_WS}),
    );
    ok(&linked, "memory_link");
    let unlinked = dispatch_as(
        &mut ctx,
        &writer,
        "memory_unlink",
        json!({"from_id": a, "to_id": b, "workspace": OWN_WS}),
    );
    ok(&unlinked, "memory_unlink");
    assert_eq!(unlinked["unlinked"], true);

    let deleted = dispatch_as(
        &mut ctx,
        &writer,
        "memory_delete",
        json!({"id": c, "workspace": OWN_WS}),
    );
    ok(&deleted, "memory_delete");
    assert_eq!(deleted["deleted"], c);

    let batch = dispatch_as(
        &mut ctx,
        &writer,
        "memory_delete_batch",
        json!({"ids": [a, b], "workspace": OWN_WS}),
    );
    ok(&batch, "memory_delete_batch");
    assert_eq!(batch["total_deleted"], 2, "{batch}");

    let chained = dispatch_as(
        &mut ctx,
        &writer,
        "memory_link",
        json!({"from_id": root, "to_id": member, "edge_type": "supersedes", "workspace": OWN_WS}),
    );
    ok(&chained, "chain link");
    let cascade = dispatch_as(
        &mut ctx,
        &writer,
        "memory_delete",
        json!({"id": root, "workspace": OWN_WS, "cascade_chain": true}),
    );
    ok(&cascade, "own cascade");
    assert_eq!(cascade["count"], 2, "{cascade}");
}

/// Controller ruling: tools that return only catalog/permission metadata (no
/// workspace data) stay available to restricted principals, even without a
/// workspace claim. Data tools stay denied.
#[test]
fn test_catalog_metadata_tools_allowed_for_restricted_principals() {
    let mut ctx = test_ctx();
    let _foreign = seed_memory(&ctx, FOREIGN_SECRET, FOREIGN_WS);
    let reader = create_principal("agent-a", Some(OWN_WS), PermissionSet::read_only());
    let anonymous = TransportPrincipal::anonymous_loopback();

    for principal in [&reader, &anonymous] {
        for (tool, params) in [
            ("discover_tools", json!({"detail": "names"})),
            ("permission_mode_status", json!({"tool": "memory_get"})),
        ] {
            let response = dispatch_as(&mut ctx, principal, tool, params);
            assert!(
                !ToolError::is_error_response(&response),
                "{tool} must stay available: {response}"
            );
            assert!(
                !response.to_string().contains("payroll"),
                "{tool}: {response}"
            );
        }
        let data = dispatch_as(
            &mut ctx,
            principal,
            "memory_stats",
            json!({"workspace": "default"}),
        );
        assert_eq!(
            data["error"]["code"], "permission_denied",
            "memory_stats: {data}"
        );
    }
}
