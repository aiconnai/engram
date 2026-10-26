//! Workspace scoping and archive handling of the secondary search tools:
//! `memory_search_suggest`, `memory_find_duplicates`, `memory_search_by_identity`
//! and `memory_session_search`, plus `include_archived` on `memory_search`.
//!
//! e04d3307 reworked these handlers and their storage queries; none of them had
//! a test that a workspace-restricted principal stays inside its workspace.

use std::sync::Arc;

use chrono::Utc;
use parking_lot::Mutex;
use serde_json::{json, Value};

use engram::auth::{PermissionSet, TokenClaims, TransportPrincipal, UserId};
use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::error::ToolError;
use engram::mcp::handlers::{dispatch, memory_crud, search, HandlerContext};
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

fn principal(namespace: Option<&str>) -> TransportPrincipal {
    TransportPrincipal::from_token_claims(TokenClaims {
        user_id: UserId::from_string("agent-reader"),
        key_id: "key-agent-reader".to_string(),
        permissions: PermissionSet::read_only(),
        namespace: namespace.map(str::to_string),
        issued_at: Utc::now(),
        expires_at: None,
    })
    .expect("valid token claims")
}

fn create(ctx: &HandlerContext, params: Value) -> i64 {
    let created = memory_crud::memory_create(ctx, params.clone());
    created["id"]
        .as_i64()
        .unwrap_or_else(|| panic!("seed {params} failed: {created}"))
}

fn archive(ctx: &HandlerContext, id: i64) {
    ctx.storage
        .with_connection(|conn| {
            conn.execute(
                "UPDATE memories SET lifecycle_state = 'archived' WHERE id = ?1",
                [id],
            )?;
            Ok(())
        })
        .expect("archive memory");
}

/// Ids of the archived rows, so assertions can name them.
struct Seeded {
    own_archived_note: i64,
    own_archived_chunk: i64,
}

/// Two tenants with a note, a session transcript chunk (same session id) and a
/// duplicated note each, plus one archived note and chunk in the own tenant.
fn seeded_ctx() -> (HandlerContext, Seeded) {
    let ctx = test_ctx();
    for (ws, label) in [(OWN_WS, "roadmap"), (FOREIGN_WS, "payroll")] {
        create(
            &ctx,
            json!({"content": format!("nightjar {label} note"), "workspace": ws}),
        );
        create(
            &ctx,
            json!({
                "content": format!("nightjar {label} transcript"),
                "type": "transcriptchunk",
                "tags": ["session:s1"],
                "workspace": ws
            }),
        );
        for _ in 0..2 {
            create(
                &ctx,
                json!({"content": format!("duplicated {label} entry"), "workspace": ws}),
            );
        }
    }
    let own_archived_note = create(
        &ctx,
        json!({"content": "nightjar archived note", "workspace": OWN_WS}),
    );
    let own_archived_chunk = create(
        &ctx,
        json!({
            "content": "nightjar archived transcript",
            "type": "transcriptchunk",
            "tags": ["session:s1"],
            "workspace": OWN_WS
        }),
    );
    archive(&ctx, own_archived_note);
    archive(&ctx, own_archived_chunk);
    (
        ctx,
        Seeded {
            own_archived_note,
            own_archived_chunk,
        },
    )
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

fn assert_denied(response: &Value, context: &str) {
    assert!(
        ToolError::is_error_response(response),
        "{context}: expected a structured error, got {response}"
    );
    assert_eq!(
        response["error"]["code"], "permission_denied",
        "{context}: {response}"
    );
    assert!(
        !response.to_string().contains("payroll"),
        "{context}: foreign content leaked: {response}"
    );
}

/// (tool, base arguments) for the four secondary search tools.
fn tool_calls() -> Vec<(&'static str, Value)> {
    vec![
        ("memory_search_suggest", json!({"query": "nightjar"})),
        ("memory_find_duplicates", json!({})),
        ("memory_search_by_identity", json!({"identity": "nightjar"})),
        (
            "memory_session_search",
            json!({"query": "nightjar", "session_id": "s1"}),
        ),
    ]
}

fn with_workspace(base: &Value, workspace: &str) -> Value {
    let mut params = base.clone();
    params["workspace"] = json!(workspace);
    params
}

fn direct(ctx: &HandlerContext, tool: &str, params: Value) -> Value {
    match tool {
        "memory_search_suggest" => search::search_suggest(ctx, params),
        "memory_find_duplicates" => search::find_duplicates(ctx, params),
        "memory_search_by_identity" => search::memory_search_by_identity(ctx, params),
        "memory_session_search" => search::memory_session_search(ctx, params),
        other => panic!("unexpected tool {other}"),
    }
}

#[test]
fn restricted_principal_is_confined_to_its_workspace_on_every_secondary_search_tool() {
    let (mut ctx, _) = seeded_ctx();
    let reader = principal(Some(OWN_WS));

    for (tool, base) in tool_calls() {
        // No claim, a foreign claim and a global request are all refused.
        assert_denied(
            &dispatch_as(&mut ctx, &reader, tool, base.clone()),
            &format!("{tool} without workspace"),
        );
        assert_denied(
            &dispatch_as(&mut ctx, &reader, tool, with_workspace(&base, FOREIGN_WS)),
            &format!("{tool} foreign workspace"),
        );
        let mut global = with_workspace(&base, OWN_WS);
        global["global"] = json!(true);
        assert_denied(
            &dispatch_as(&mut ctx, &reader, tool, global),
            &format!("{tool} global"),
        );

        // The same refusals hold for a direct handler call (no dispatcher).
        ctx.principal = Some(reader.clone());
        assert_denied(
            &direct(&ctx, tool, base.clone()),
            &format!("{tool} direct without workspace"),
        );
        assert_denied(
            &direct(&ctx, tool, with_workspace(&base, FOREIGN_WS)),
            &format!("{tool} direct foreign workspace"),
        );
        ctx.principal = None;

        // Positive control: the own workspace works and carries no foreign data.
        let own = dispatch_as(&mut ctx, &reader, tool, with_workspace(&base, OWN_WS));
        assert!(
            !ToolError::is_error_response(&own),
            "{tool}: own workspace refused: {own}"
        );
        assert!(
            !own.to_string().contains("payroll"),
            "{tool}: foreign content leaked: {own}"
        );
    }
}

#[test]
fn identity_and_session_search_return_only_own_live_rows() {
    let (mut ctx, seeded) = seeded_ctx();
    let reader = principal(Some(OWN_WS));

    let by_identity = dispatch_as(
        &mut ctx,
        &reader,
        "memory_search_by_identity",
        json!({"identity": "nightjar", "workspace": OWN_WS}),
    );
    let ids: Vec<i64> = by_identity["memories"]
        .as_array()
        .expect("memories array")
        .iter()
        .map(|m| m["id"].as_i64().unwrap())
        .collect();
    assert!(!ids.is_empty(), "own live notes expected: {by_identity}");
    assert!(
        !ids.contains(&seeded.own_archived_note),
        "archived note must be excluded: {by_identity}"
    );
    assert!(
        by_identity["memories"]
            .as_array()
            .unwrap()
            .iter()
            .all(|m| m["workspace"] == OWN_WS
                && !m["type"]
                    .as_str()
                    .is_some_and(|t| t.starts_with("transcript"))),
        "identity search is scoped and skips transcript chunks: {by_identity}"
    );

    let sessions = dispatch_as(
        &mut ctx,
        &reader,
        "memory_session_search",
        json!({"query": "nightjar", "session_id": "s1", "workspace": OWN_WS}),
    );
    let chunks = sessions["memories"].as_array().expect("memories array");
    assert_eq!(chunks.len(), 1, "one live own chunk expected: {sessions}");
    assert_eq!(chunks[0]["workspace"], OWN_WS);
    assert_ne!(chunks[0]["id"].as_i64(), Some(seeded.own_archived_chunk));
    assert!(
        chunks[0]["tags"]
            .as_array()
            .is_some_and(|tags| tags.iter().any(|t| t == "session:s1")),
        "tags are loaded on session results: {sessions}"
    );
}

#[test]
fn duplicates_are_reported_per_workspace() {
    let (mut ctx, _) = seeded_ctx();
    let reader = principal(Some(OWN_WS));

    let own = dispatch_as(
        &mut ctx,
        &reader,
        "memory_find_duplicates",
        json!({"workspace": OWN_WS}),
    );
    assert_eq!(own["count"], 1, "{own}");
    assert!(own.to_string().contains("duplicated roadmap entry"));
    assert!(!own.to_string().contains("payroll"), "{own}");
}

#[test]
fn unrestricted_principal_sees_every_workspace() {
    let (mut ctx, seeded) = seeded_ctx();
    let admin_reader = principal(None);

    let by_identity = dispatch_as(
        &mut ctx,
        &admin_reader,
        "memory_search_by_identity",
        json!({"identity": "nightjar"}),
    );
    let text = by_identity.to_string();
    assert!(
        text.contains("roadmap") && text.contains("payroll"),
        "{text}"
    );
    assert!(
        !text.contains("archived"),
        "archived rows stay excluded: {text}"
    );

    let sessions = dispatch_as(
        &mut ctx,
        &admin_reader,
        "memory_session_search",
        json!({"query": "nightjar", "session_id": "s1"}),
    );
    assert_eq!(
        sessions["memories"].as_array().unwrap().len(),
        2,
        "{sessions}"
    );

    let duplicates = dispatch_as(&mut ctx, &admin_reader, "memory_find_duplicates", json!({}));
    assert_eq!(duplicates["count"], 2, "{duplicates}");

    let _ = seeded;
}

#[test]
fn search_include_archived_is_opt_in_and_stays_inside_the_workspace() {
    let (mut ctx, seeded) = seeded_ctx();
    let reader = principal(Some(OWN_WS));
    let ids = |response: &Value| -> Vec<i64> {
        response
            .as_array()
            .unwrap_or_else(|| panic!("search failed: {response}"))
            .iter()
            .map(|r| r["memory"]["id"].as_i64().unwrap())
            .collect()
    };

    let default = dispatch_as(
        &mut ctx,
        &reader,
        "memory_search",
        json!({"query": "nightjar", "workspace": OWN_WS, "skip_cache": true}),
    );
    assert!(
        !ids(&default).contains(&seeded.own_archived_note),
        "archived memories are excluded by default: {default}"
    );

    let explicit_false = dispatch_as(
        &mut ctx,
        &reader,
        "memory_search",
        json!({
            "query": "nightjar", "workspace": OWN_WS,
            "include_archived": false, "skip_cache": true
        }),
    );
    assert!(!ids(&explicit_false).contains(&seeded.own_archived_note));

    let with_archived = dispatch_as(
        &mut ctx,
        &reader,
        "memory_search",
        json!({
            "query": "nightjar", "workspace": OWN_WS,
            "include_archived": true, "skip_cache": true
        }),
    );
    assert!(
        ids(&with_archived).contains(&seeded.own_archived_note),
        "include_archived must return the archived note: {with_archived}"
    );
    assert!(
        !with_archived.to_string().contains("payroll"),
        "include_archived must not widen the workspace: {with_archived}"
    );

    // Cached results for the two settings must not be served for each other.
    let cached_default = dispatch_as(
        &mut ctx,
        &reader,
        "memory_search",
        json!({"query": "nightjar", "workspace": OWN_WS}),
    );
    let cached_archived = dispatch_as(
        &mut ctx,
        &reader,
        "memory_search",
        json!({"query": "nightjar", "workspace": OWN_WS, "include_archived": true}),
    );
    assert!(!ids(&cached_default).contains(&seeded.own_archived_note));
    assert!(ids(&cached_archived).contains(&seeded.own_archived_note));
}
