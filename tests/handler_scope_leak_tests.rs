//! Scope-leak regressions for handlers that reach beyond the caller's workspace.
//!
//! - A-6: `memory_digest` must not reach `context_build_bundle` data a restricted
//!   principal is denied when calling that tool directly.
//! - A-7: `harness_status` must not expose the server checkout's git state to a
//!   principal that cannot read all workspaces and lacks maintenance rights.
//! - Q-A: `memory_agent_status` must not count memories outside the readable
//!   workspace for a restricted principal.

use std::sync::Arc;

use chrono::Utc;
use parking_lot::Mutex;
use serde_json::{json, Value};

use engram::auth::{PermissionSet, TokenClaims, TransportPrincipal, UserId};
use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::handlers::{dispatch, HandlerContext};
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::Storage;
use engram::types::EmbeddingConfig;

const OWN_WS: &str = "tenant-a";
const FOREIGN_WS: &str = "tenant-b";
const MARKER: &str = "scopeleakmarker";

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
        user_id: UserId::from_string("agent"),
        key_id: "key-agent".to_string(),
        permissions,
        namespace: namespace.map(str::to_string),
        issued_at: Utc::now(),
        expires_at: None,
    })
    .expect("valid token claims")
}

fn dispatch_as(
    ctx: &mut HandlerContext,
    principal: Option<&TransportPrincipal>,
    tool: &str,
    params: Value,
) -> Value {
    ctx.principal = principal.cloned();
    let response = dispatch(ctx, tool, params);
    ctx.principal = None;
    response
}

fn seed_memory(ctx: &mut HandlerContext, workspace: &str, content: &str) {
    let created = dispatch_as(
        ctx,
        None,
        "memory_create",
        json!({"content": content, "workspace": workspace}),
    );
    assert!(created.get("error").is_none(), "seed failed: {created}");
}

// ── A-6: memory_digest -> context_build_bundle ───────────────────────────────

fn seed_context_event(ctx: &mut HandlerContext, workspace_path_hash: &str) {
    let record = dispatch_as(
        ctx,
        None,
        "context_record",
        json!({
            "source": "codex",
            "repo_id": "github:aiconnai/engram",
            "workspace_path_hash": workspace_path_hash,
            "session_id": "session-leak",
            "task_id": "LEAK-1",
            "event_type": "command",
            "command": format!("cargo test {MARKER}"),
            "cwd": "/repo",
            "exit_code": 1,
            "git_branch": "main",
            "commit_hash": "abc123",
            "started_at": "2026-05-01T00:00:00Z",
            "summary": format!("Decision: {MARKER} foreign operational note. Blocker unresolved."),
            "key_errors": [format!("{MARKER} failure")],
            "touched_files": ["src/secret.rs"],
            "reducer": {
                "name": "leak_fixture",
                "version": "1",
                "lossy": true,
                "confidence": 0.9,
                "structured_facts": {"decision": format!("{MARKER} decision")},
                "tokens_raw_est": 100,
                "tokens_compact_est": 10
            }
        }),
    );
    assert!(
        record.get("error").is_none(),
        "context_record failed: {record}"
    );
}

fn assert_digest_hides_operational_context(label: &str, response: &Value) {
    assert!(
        response.get("error").is_none(),
        "{label}: digest itself must still answer: {response}"
    );
    assert_eq!(
        response["operational_context"]["included"],
        json!(false),
        "{label}: operational context must be withheld: {response}"
    );
    assert!(
        !response["operational_context"].to_string().contains(MARKER),
        "{label}: operational context leaked: {response}"
    );
    let leaked_ids = response["provenance"]["source_context_event_ids"]
        .as_array()
        .map_or(0, Vec::len);
    assert_eq!(
        leaked_ids, 0,
        "{label}: context event ids leaked: {response}"
    );
}

#[test]
fn memory_digest_withholds_operational_context_from_restricted_principals() {
    let mut ctx = test_ctx();
    seed_context_event(&mut ctx, "default");
    seed_context_event(&mut ctx, OWN_WS);

    // Control: the direct tool is denied for restricted principals.
    let anonymous = TransportPrincipal::anonymous_loopback();
    let namespaced = principal(Some(OWN_WS), PermissionSet::standard_user());
    for (label, who, ws) in [
        ("anonymous", &anonymous, "default"),
        ("namespaced", &namespaced, OWN_WS),
    ] {
        let direct = dispatch_as(
            &mut ctx,
            Some(who),
            "context_build_bundle",
            json!({"workspace": ws, "query": MARKER}),
        );
        assert_eq!(
            direct["error"]["code"], "permission_denied",
            "{label}: {direct}"
        );

        let digest = dispatch_as(
            &mut ctx,
            Some(who),
            "memory_digest",
            json!({"topic": MARKER, "workspace": ws}),
        );
        assert_digest_hides_operational_context(label, &digest);
    }
}

#[test]
fn memory_digest_keeps_operational_context_for_unrestricted_callers() {
    let mut ctx = test_ctx();
    seed_context_event(&mut ctx, "default");
    let unscoped = principal(None, PermissionSet::standard_user());

    for (label, who) in [("stdio owner", None), ("unscoped token", Some(&unscoped))] {
        let digest = dispatch_as(
            &mut ctx,
            who,
            "memory_digest",
            json!({"topic": MARKER, "workspace": "default"}),
        );
        assert_eq!(
            digest["operational_context"]["included"],
            json!(true),
            "{label}: {digest}"
        );
        assert!(
            digest["operational_context"].to_string().contains(MARKER),
            "{label}: authorized caller must keep operational context: {digest}"
        );
    }
}

// ── A-7: harness_status git state ────────────────────────────────────────────

#[test]
fn harness_status_withholds_git_state_from_restricted_principals() {
    let mut ctx = test_ctx();
    let anonymous = TransportPrincipal::anonymous_loopback();
    let namespaced_reader = principal(Some(OWN_WS), PermissionSet::read_only());

    for (label, who, ws, expected_ws) in [
        ("anonymous", &anonymous, "DEFAULT", "default"),
        ("namespaced", &namespaced_reader, "  TENANT-A  ", OWN_WS),
    ] {
        let status = dispatch_as(
            &mut ctx,
            Some(who),
            "harness_status",
            json!({"workspace": ws, "include_git": true}),
        );
        assert!(status.get("error").is_none(), "{label}: {status}");
        assert_eq!(status["workspace"], json!(expected_ws), "{label}: {status}");
        assert!(
            status["git_state"].is_null(),
            "{label}: git state must be withheld: {status}"
        );
    }
}

#[test]
fn harness_status_keeps_git_state_for_trusted_callers() {
    let mut ctx = test_ctx();
    let unscoped_reader = principal(None, PermissionSet::read_only());
    let namespaced_admin = principal(Some(OWN_WS), PermissionSet::admin());

    for (label, who, ws) in [
        ("stdio owner", None, "default"),
        ("unscoped reader", Some(&unscoped_reader), "default"),
        ("namespaced admin", Some(&namespaced_admin), OWN_WS),
    ] {
        let status = dispatch_as(
            &mut ctx,
            who,
            "harness_status",
            json!({"workspace": ws, "include_git": true}),
        );
        assert!(status.get("error").is_none(), "{label}: {status}");
        assert!(
            status["git_state"].is_object(),
            "{label}: git state expected: {status}"
        );
        assert!(
            status["git_state"].get("branch").is_some(),
            "{label}: {status}"
        );
    }
}

// ── Q-A: memory_agent_status memory count ────────────────────────────────────

#[test]
fn memory_agent_status_counts_only_readable_workspace_for_restricted_principals() {
    let mut ctx = test_ctx();
    seed_memory(&mut ctx, OWN_WS, "own note one");
    seed_memory(&mut ctx, OWN_WS, "own note two");
    seed_memory(&mut ctx, FOREIGN_WS, "foreign note");
    seed_memory(&mut ctx, "default", "default note");

    let namespaced = principal(Some(OWN_WS), PermissionSet::read_only());
    let status = dispatch_as(
        &mut ctx,
        Some(&namespaced),
        "memory_agent_status",
        json!({"workspace": OWN_WS}),
    );
    assert_eq!(status["total_memories"], json!(2), "{status}");

    let anonymous = TransportPrincipal::anonymous_loopback();
    let status = dispatch_as(
        &mut ctx,
        Some(&anonymous),
        "memory_agent_status",
        json!({"workspace": "default"}),
    );
    assert_eq!(status["total_memories"], json!(1), "{status}");
}

#[test]
fn memory_agent_status_omits_count_when_workspace_is_not_readable() {
    // Direct handler call: the dispatcher guard is not in the path.
    let mut ctx = test_ctx();
    seed_memory(&mut ctx, FOREIGN_WS, "foreign note");
    ctx.principal = Some(principal(Some(OWN_WS), PermissionSet::read_only()));

    let status = engram::mcp::handlers::autonomous::memory_agent_status(
        &ctx,
        json!({"workspace": FOREIGN_WS}),
    );
    assert!(status.get("total_memories").is_none(), "{status}");
}

#[test]
fn memory_agent_status_keeps_global_count_for_unrestricted_callers() {
    let mut ctx = test_ctx();
    seed_memory(&mut ctx, OWN_WS, "own note");
    seed_memory(&mut ctx, FOREIGN_WS, "foreign note");
    let unscoped = principal(None, PermissionSet::read_only());

    for who in [None, Some(&unscoped)] {
        let status = dispatch_as(&mut ctx, who, "memory_agent_status", json!({}));
        assert_eq!(status["total_memories"], json!(2), "{status}");
    }
}
