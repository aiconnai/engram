//! Workspace authorization for batch tools whose items carry their own
//! `workspace` (`memory_create_batch`, `memory_ingest_fact_batch`).
//!
//! A workspace-restricted principal must not write into a workspace it may not
//! access by hiding the foreign claim inside a batch item. Every denial case
//! asserts that NOTHING was written; positive controls prove the legitimate
//! paths still work and that unrestricted callers are unaffected.

use std::sync::Arc;

use chrono::Utc;
use parking_lot::Mutex;
use serde_json::{json, Value};

use engram::auth::{PermissionSet, TokenClaims, TransportPrincipal, UserId};
use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::error::ToolError;
use engram::mcp::handlers::{dispatch, memory_crud, HandlerContext};
use engram::mcp::permission::check_tool_authorization;
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::Storage;
use engram::types::EmbeddingConfig;

const OWN_WS: &str = "tenant-a";
const FOREIGN_WS: &str = "tenant-b";

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

fn principal(namespace: Option<&str>, permissions: PermissionSet) -> TransportPrincipal {
    TransportPrincipal::from_token_claims(TokenClaims {
        user_id: UserId::from_string("agent-batch"),
        key_id: "key-agent-batch".to_string(),
        permissions,
        namespace: namespace.map(str::to_string),
        issued_at: Utc::now(),
        expires_at: None,
    })
    .expect("valid token claims")
}

/// Token bound to `OWN_WS` with write permission (restricted principal).
fn restricted_writer() -> TransportPrincipal {
    principal(Some(OWN_WS), PermissionSet::standard_user())
}

fn memory_count(ctx: &HandlerContext) -> i64 {
    ctx.storage
        .with_connection(|conn| {
            Ok(conn.query_row("SELECT COUNT(*) FROM memories", [], |r| r.get(0))?)
        })
        .expect("memory count")
}

fn count_in_workspace(ctx: &HandlerContext, workspace: &str) -> i64 {
    ctx.storage
        .with_connection(|conn| {
            Ok(conn.query_row(
                "SELECT COUNT(*) FROM memories WHERE workspace = ?1",
                [workspace],
                |r| r.get(0),
            )?)
        })
        .expect("workspace count")
}

fn dispatch_as(
    ctx: &mut HandlerContext,
    who: &TransportPrincipal,
    tool: &str,
    params: Value,
) -> Value {
    ctx.principal = Some(who.clone());
    let response = dispatch(ctx, tool, params);
    ctx.principal = None;
    response
}

fn assert_permission_denied(response: &Value, context: &str) {
    assert!(
        ToolError::is_error_response(response),
        "{context}: expected a structured error, got {response}"
    );
    assert_eq!(
        response["error"]["code"], "permission_denied",
        "{context}: {response}"
    );
}

/// Hostile and malformed shapes for `memory_create_batch`.
fn hostile_create_batch_params() -> Vec<(&'static str, Value)> {
    vec![
        (
            "top-level own claim + foreign item",
            json!({"workspace": OWN_WS, "memories": [
                {"content": "own item", "workspace": OWN_WS},
                {"content": "foreign item", "workspace": FOREIGN_WS}
            ]}),
        ),
        (
            "no top-level claim + foreign item",
            json!({"memories": [
                {"content": "own item", "workspace": OWN_WS},
                {"content": "foreign item", "workspace": FOREIGN_WS}
            ]}),
        ),
        (
            "only a foreign item",
            json!({"memories": [{"content": "foreign item", "workspace": FOREIGN_WS}]}),
        ),
        (
            "case-variant foreign item",
            json!({"workspace": OWN_WS, "memories": [
                {"content": "foreign item", "workspace": "TENANT-B"}
            ]}),
        ),
        (
            "non-string item workspace",
            json!({"workspace": OWN_WS, "memories": [
                {"content": "weird item", "workspace": 123}
            ]}),
        ),
        (
            "decoy arguments envelope with a different name",
            json!({"name": "x", "arguments": {"workspace": OWN_WS}, "memories": [
                {"content": "foreign item", "workspace": FOREIGN_WS}
            ]}),
        ),
        (
            "decoy arguments envelope reusing the tool name",
            json!({"name": "memory_create_batch", "arguments": {"workspace": OWN_WS}, "memories": [
                {"content": "foreign item", "workspace": FOREIGN_WS}
            ]}),
        ),
        (
            "top-level own claim + item without workspace (lands in default)",
            json!({"workspace": OWN_WS, "memories": [
                {"content": "unscoped item"}
            ]}),
        ),
        (
            "own item + item without workspace (lands in default)",
            json!({"memories": [
                {"content": "own item", "workspace": OWN_WS},
                {"content": "unscoped item"}
            ]}),
        ),
    ]
}

#[test]
fn create_batch_foreign_or_unscoped_item_denied_by_dispatch_and_nothing_written() {
    let mut ctx = test_ctx();
    let who = restricted_writer();

    for (label, params) in hostile_create_batch_params() {
        let response = dispatch_as(&mut ctx, &who, "memory_create_batch", params.clone());
        assert_permission_denied(&response, label);
        assert_eq!(memory_count(&ctx), 0, "{label}: rows were written");
    }
}

#[test]
fn create_batch_foreign_or_unscoped_item_denied_by_permission_layer_alone() {
    // The central check must also catch explicit foreign claims on its own
    // (transports pre-check with it before dispatching).
    let who = restricted_writer();
    for (label, params) in hostile_create_batch_params() {
        // Items that merely omit the workspace resolve to the handler default,
        // which only the handler can see; the permission layer judges claims.
        if label.contains("without workspace") {
            continue;
        }
        let denial = check_tool_authorization(None, "memory_create_batch", &params, Some(&who));
        assert!(
            denial.as_ref().is_some_and(ToolError::is_error_response),
            "{label}: permission layer let the call through: {denial:?}"
        );
    }
}

#[test]
fn create_batch_handler_denies_without_dispatch_guard() {
    // Direct handler calls (no dispatcher) must enforce the principal too.
    let mut ctx = test_ctx();
    ctx.principal = Some(restricted_writer());

    for (label, params) in hostile_create_batch_params() {
        let response = memory_crud::memory_create_batch(&ctx, params.clone());
        // Decoy envelopes carry no `memories` claim the handler would misread;
        // they are exercised through dispatch above. The handler sees `memories`
        // at the top level regardless, so it must deny them as well.
        assert_permission_denied(&response, label);
        assert_eq!(memory_count(&ctx), 0, "{label}: rows were written");
    }
}

#[test]
fn create_batch_all_own_items_allowed() {
    let mut ctx = test_ctx();
    let who = restricted_writer();

    // Per-item claims only (the schema declares `workspace` on items).
    let per_item = dispatch_as(
        &mut ctx,
        &who,
        "memory_create_batch",
        json!({"memories": [
            {"content": "alpha own note", "workspace": OWN_WS},
            {"content": "beta own note", "workspace": "TENANT-A"}
        ]}),
    );
    assert!(
        !ToolError::is_error_response(&per_item),
        "per-item own workspace denied: {per_item}"
    );
    assert_eq!(per_item["total_created"], 2, "{per_item}");

    // Top-level claim plus explicit item claims.
    let both = dispatch_as(
        &mut ctx,
        &who,
        "memory_create_batch",
        json!({"workspace": OWN_WS, "memories": [
            {"content": "gamma own note", "workspace": OWN_WS}
        ]}),
    );
    assert_eq!(both["total_created"], 1, "{both}");

    assert_eq!(count_in_workspace(&ctx, OWN_WS), 3);
    assert_eq!(memory_count(&ctx), 3, "nothing may land outside {OWN_WS}");
}

#[test]
fn create_batch_unrestricted_callers_unchanged() {
    let items = json!({"memories": [
        {"content": "default note"},
        {"content": "tenant-a note", "workspace": OWN_WS},
        {"content": "tenant-b note", "workspace": FOREIGN_WS}
    ]});

    // Unrestricted token (no namespace).
    let mut ctx = test_ctx();
    let unrestricted = principal(None, PermissionSet::standard_user());
    let response = dispatch_as(
        &mut ctx,
        &unrestricted,
        "memory_create_batch",
        items.clone(),
    );
    assert_eq!(response["total_created"], 3, "{response}");
    assert_eq!(count_in_workspace(&ctx, "default"), 1);
    assert_eq!(count_in_workspace(&ctx, OWN_WS), 1);
    assert_eq!(count_in_workspace(&ctx, FOREIGN_WS), 1);

    // stdio transport: no principal at all.
    let ctx = test_ctx();
    let response = dispatch(&ctx, "memory_create_batch", items);
    assert_eq!(response["total_created"], 3, "{response}");
    assert_eq!(memory_count(&ctx), 3);
}

fn hostile_fact_batch_params() -> Vec<(&'static str, Value)> {
    vec![
        (
            "top-level own claim + foreign fact",
            json!({"workspace": OWN_WS, "facts": [
                {"fact": "own fact", "workspace": OWN_WS},
                {"fact": "foreign fact", "workspace": FOREIGN_WS}
            ]}),
        ),
        (
            "no top-level claim + foreign fact",
            json!({"facts": [
                {"fact": "own fact", "workspace": OWN_WS},
                {"fact": "foreign fact", "workspace": FOREIGN_WS}
            ]}),
        ),
        (
            "case-variant foreign fact",
            json!({"workspace": OWN_WS, "facts": [
                {"fact": "foreign fact", "workspace": "Tenant-B"}
            ]}),
        ),
        (
            "non-string fact workspace",
            json!({"workspace": OWN_WS, "facts": [
                {"fact": "weird fact", "workspace": ["tenant-b"]}
            ]}),
        ),
        (
            "decoy arguments envelope with a different name",
            json!({"name": "x", "arguments": {"workspace": OWN_WS}, "facts": [
                {"fact": "foreign fact", "workspace": FOREIGN_WS}
            ]}),
        ),
        (
            "own fact + fact falling back to the default workspace",
            json!({"facts": [
                {"fact": "own fact", "workspace": OWN_WS},
                {"fact": "unscoped fact"}
            ]}),
        ),
        (
            "top-level default claim",
            json!({"workspace": "default", "facts": [{"fact": "default fact"}]}),
        ),
    ]
}

#[test]
fn fact_batch_foreign_or_unscoped_item_denied_by_dispatch_and_nothing_written() {
    let mut ctx = test_ctx();
    let who = restricted_writer();

    for (label, params) in hostile_fact_batch_params() {
        let response = dispatch_as(&mut ctx, &who, "memory_ingest_fact_batch", params.clone());
        assert_permission_denied(&response, label);
        assert_eq!(memory_count(&ctx), 0, "{label}: rows were written");
    }
}

#[test]
fn fact_batch_handler_denies_without_dispatch_guard() {
    let mut ctx = test_ctx();
    ctx.principal = Some(restricted_writer());

    for (label, params) in hostile_fact_batch_params() {
        let response = memory_crud::memory_ingest_fact_batch(&ctx, params.clone());
        assert_permission_denied(&response, label);
        assert_eq!(memory_count(&ctx), 0, "{label}: rows were written");
    }
}

#[test]
fn fact_batch_foreign_claim_denied_by_permission_layer_alone() {
    let who = restricted_writer();
    for (label, params) in hostile_fact_batch_params() {
        if label.contains("default workspace") {
            continue;
        }
        let denial =
            check_tool_authorization(None, "memory_ingest_fact_batch", &params, Some(&who));
        assert!(
            denial.as_ref().is_some_and(ToolError::is_error_response),
            "{label}: permission layer let the call through: {denial:?}"
        );
    }
}

#[test]
fn fact_batch_own_items_allowed_and_inherit_top_level_workspace() {
    let mut ctx = test_ctx();
    let who = restricted_writer();

    // Item without workspace inherits the (authorized) top-level workspace.
    let inherited = dispatch_as(
        &mut ctx,
        &who,
        "memory_ingest_fact_batch",
        json!({"workspace": OWN_WS, "facts": [
            {"fact": "inherited fact"},
            {"fact": "explicit fact", "workspace": "TENANT-A"}
        ]}),
    );
    assert_eq!(inherited["count"], 2, "{inherited}");

    // Per-item claims only.
    let per_item = dispatch_as(
        &mut ctx,
        &who,
        "memory_ingest_fact_batch",
        json!({"facts": [{"fact": "per item fact", "workspace": OWN_WS}]}),
    );
    assert_eq!(per_item["count"], 1, "{per_item}");

    assert_eq!(count_in_workspace(&ctx, OWN_WS), 3);
    assert_eq!(memory_count(&ctx), 3, "nothing may land outside {OWN_WS}");
}

#[test]
fn fact_batch_unrestricted_callers_unchanged() {
    let params = json!({"facts": [
        {"fact": "default fact"},
        {"fact": "tenant-a fact", "workspace": OWN_WS},
        {"fact": "tenant-b fact", "workspace": FOREIGN_WS}
    ]});

    let mut ctx = test_ctx();
    let unrestricted = principal(None, PermissionSet::standard_user());
    let response = dispatch_as(
        &mut ctx,
        &unrestricted,
        "memory_ingest_fact_batch",
        params.clone(),
    );
    assert_eq!(response["count"], 3, "{response}");
    assert_eq!(count_in_workspace(&ctx, "default"), 1);
    assert_eq!(count_in_workspace(&ctx, OWN_WS), 1);
    assert_eq!(count_in_workspace(&ctx, FOREIGN_WS), 1);

    let ctx = test_ctx();
    let response = dispatch(&ctx, "memory_ingest_fact_batch", params);
    assert_eq!(response["count"], 3, "{response}");
}

#[test]
fn http_style_envelope_is_checked_against_item_workspaces() {
    // The HTTP transport unwraps the `tools/call` params object
    // (`{"name", "arguments"}`) once and pre-checks the bare `arguments`
    // exactly as the dispatcher will see them.
    let who = restricted_writer();
    let pre_check = |envelope: &serde_json::Value| {
        let (name, arguments) = engram::mcp::permission::tool_call_parts(envelope).unwrap();
        check_tool_authorization(None, name, arguments, Some(&who))
    };

    let hostile = json!({
        "name": "memory_create_batch",
        "arguments": {"workspace": OWN_WS, "memories": [
            {"content": "foreign item", "workspace": FOREIGN_WS}
        ]}
    });
    let denial = pre_check(&hostile);
    assert!(
        denial.as_ref().is_some_and(ToolError::is_error_response),
        "envelope with foreign item must be denied: {denial:?}"
    );

    let legit = json!({
        "name": "memory_create_batch",
        "arguments": {"memories": [{"content": "own item", "workspace": OWN_WS}]}
    });
    assert!(
        pre_check(&legit).is_none(),
        "envelope with only own items must pass"
    );
}

#[test]
fn dispatch_does_not_unwrap_name_and_arguments_keys_inside_arguments() {
    // `name`/`arguments` inside real arguments are data. The dispatcher must not
    // treat them as an envelope: neither letting them hide the real claim nor
    // letting them smuggle a different workspace into the check.
    let mut ctx = test_ctx();
    let who = restricted_writer();

    let own_with_decoy = dispatch_as(
        &mut ctx,
        &who,
        "memory_list",
        json!({"workspace": OWN_WS, "name": "x", "arguments": {"workspace": FOREIGN_WS}}),
    );
    assert!(
        !ToolError::is_error_response(&own_with_decoy),
        "own workspace with a decoy `arguments` key must pass: {own_with_decoy}"
    );

    let foreign_with_cover = dispatch_as(
        &mut ctx,
        &who,
        "memory_list",
        json!({"workspace": FOREIGN_WS, "name": "x", "arguments": {"workspace": OWN_WS}}),
    );
    assert!(
        ToolError::is_error_response(&foreign_with_cover),
        "foreign top-level workspace must be denied: {foreign_with_cover}"
    );
}
