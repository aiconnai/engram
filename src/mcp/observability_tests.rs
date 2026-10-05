//! O1: the critical failure paths are counted with bounded labels, and the
//! protocol payloads that carry the failures stay exactly as before.

use std::sync::Arc;
use std::time::Duration;

use serde_json::{json, Value};

use super::redaction_tests::{ctx_with, LeakyProvider};
use crate::auth::TransportPrincipal;
use crate::embedding::{run_embedding_drain_cycle_logged, Embedder, EmbeddingQueueHygieneConfig};
use crate::error::{EngramError, Result};
use crate::mcp::handlers::dispatch;
use crate::mcp::permission::check_tool_authorization;
use crate::observability::test_capture::{assert_logs_exclude, captured, install};
use crate::observability::{counters, PermissionDeniedReason as Reason};

fn denied(reason: Reason) -> u64 {
    counters().permission_denied_count(reason)
}

fn assert_denial_counted(reason: Reason, denial: Option<Value>, before: u64) {
    let denial = denial.expect("the call is denied");
    // The wire envelope is the same permission_denied shape as before O1.
    assert_eq!(denial["error"]["code"], "permission_denied", "{denial}");
    assert!(
        denied(reason) > before,
        "{} was not counted",
        reason.as_str()
    );
}

#[test]
fn permission_denials_are_counted_by_c1_reason_without_changing_the_envelope() {
    let anonymous = TransportPrincipal::anonymous_loopback();

    let before = denied(Reason::ToolNotWorkspaceScoped);
    let unscoped = check_tool_authorization(
        None,
        "memory_export_graph",
        &json!({"workspace": "default"}),
        Some(&anonymous),
    );
    assert_eq!(
        unscoped
            .as_ref()
            .and_then(|d| d["error"]["details"]["reason"].as_str()),
        Some("tool_not_workspace_scoped")
    );
    assert_denial_counted(Reason::ToolNotWorkspaceScoped, unscoped, before);

    let before = denied(Reason::WorkspaceClaimMissing);
    let no_claim = check_tool_authorization(None, "memory_list", &json!({}), Some(&anonymous));
    assert_denial_counted(Reason::WorkspaceClaimMissing, no_claim, before);

    let before = denied(Reason::WorkspaceNotAllowed);
    let foreign = check_tool_authorization(
        None,
        "memory_list",
        &json!({"workspace": "someone-elses"}),
        Some(&anonymous),
    );
    assert_denial_counted(Reason::WorkspaceNotAllowed, foreign, before);

    let before = denied(Reason::GlobalScopeRequested);
    let global = check_tool_authorization(
        None,
        "memory_search",
        &json!({"query": "q", "global": true}),
        Some(&anonymous),
    );
    assert_denial_counted(Reason::GlobalScopeRequested, global, before);

    let before = denied(Reason::ModeInsufficient);
    let mode = check_tool_authorization(
        None,
        "memory_delete",
        &json!({"id": 1, "_permission_mode": "read_only"}),
        None,
    );
    assert_denial_counted(Reason::ModeInsufficient, mode, before);

    // An allowed call counts nothing.
    let allowed_before = Reason::ALL.map(denied);
    let allowed = check_tool_authorization(
        None,
        "memory_list",
        &json!({"workspace": "default"}),
        Some(&anonymous),
    );
    assert!(allowed.is_none());
    // Other tests may deny concurrently, so only assert nothing went *down*.
    assert!(Reason::ALL
        .map(denied)
        .iter()
        .zip(allowed_before)
        .all(|(a, b)| *a >= b));
}

#[test]
fn foreign_memory_denial_is_counted_but_stays_indistinguishable_from_missing() {
    let mut ctx = ctx_with(Arc::new(LeakyProvider { body: "unused" }));
    let created = dispatch(
        &ctx,
        "memory_create",
        json!({"content": "foreign-workspace note", "workspace": "private-ws", "defer_embedding": true}),
    );
    let id = created["id"].as_i64().expect("created");
    ctx.principal = Some(TransportPrincipal::anonymous_loopback());

    let before = denied(Reason::ForeignOrMissingMemory);
    let foreign = dispatch(
        &ctx,
        "memory_get",
        json!({"id": id, "workspace": "default"}),
    );
    let missing = dispatch(
        &ctx,
        "memory_get",
        json!({"id": id + 1000, "workspace": "default"}),
    );

    assert_eq!(foreign["error"]["code"], "not_found");
    assert_eq!(missing["error"]["code"], "not_found");
    assert!(denied(Reason::ForeignOrMissingMemory) >= before + 2);
    // The counter has no id/workspace label: only the reason.
    let snapshot = serde_json::to_string(&counters().snapshot()).expect("json");
    assert!(!snapshot.contains("private-ws"));
}

#[test]
fn sqlite_busy_is_counted_when_it_converts_into_engram_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("busy.db");
    let holder = rusqlite::Connection::open(&path).expect("open");
    holder
        .execute_batch("CREATE TABLE t(x); BEGIN IMMEDIATE;")
        .expect("hold the write lock");
    let contender = rusqlite::Connection::open(&path).expect("open");
    contender
        .busy_timeout(Duration::from_millis(0))
        .expect("timeout");

    let before = counters().snapshot().sqlite_busy_total;
    let result: Result<()> = (|| {
        contender.execute_batch("BEGIN IMMEDIATE")?;
        Ok(())
    })();

    assert!(matches!(result, Err(EngramError::Database(_))));
    assert!(
        counters().snapshot().sqlite_busy_total > before,
        "a real SQLITE_BUSY must be counted"
    );
    // Non-busy database errors are not counted as busy.
    let other_before = counters().snapshot().sqlite_busy_total;
    let _: EngramError = rusqlite::Error::QueryReturnedNoRows.into();
    assert_eq!(counters().snapshot().sqlite_busy_total, other_before);
}

struct FailingProvider {
    timeout: bool,
}

impl Embedder for FailingProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        Err(if self.timeout {
            EngramError::Io(std::io::Error::new(
                std::io::ErrorKind::TimedOut,
                format!("timed out waiting for provider (input {text})"),
            ))
        } else {
            EngramError::Embedding("provider rejected the request".to_string())
        })
    }

    fn dimensions(&self) -> usize {
        8
    }

    fn model_name(&self) -> &str {
        "failing-provider-test"
    }
}

fn drain_once(provider: Arc<dyn Embedder>, content: &str) {
    let ctx = ctx_with(provider.clone());
    dispatch(
        &ctx,
        "memory_create",
        json!({"content": content, "defer_embedding": true}),
    );
    run_embedding_drain_cycle_logged(
        &ctx.storage,
        provider.as_ref(),
        10,
        &EmbeddingQueueHygieneConfig::default(),
        &|_, _| {},
    );
}

#[test]
fn provider_timeouts_and_failures_are_counted_separately_by_the_drain() {
    install();
    let before = counters().snapshot();

    drain_once(
        Arc::new(FailingProvider { timeout: true }),
        "drain timeout case",
    );
    drain_once(
        Arc::new(FailingProvider { timeout: false }),
        "drain failure case",
    );

    let after = counters().snapshot();
    assert!(after.provider_timeout_total["embedding"] > before.provider_timeout_total["embedding"]);
    assert!(after.provider_failure_total["embedding"] > before.provider_failure_total["embedding"]);
}

#[test]
fn drain_failure_is_logged_as_an_operation_with_class_only() {
    install();
    const BODY: &str = "drain-provider-body-sentinel-1d2e";
    const TEXT: &str = "drain-private-text-sentinel-3f4a";
    drain_once(Arc::new(LeakyProvider { body: BODY }), TEXT);

    // Other drain tests run in parallel, so look for any event with this class.
    let logs = captured();
    let line = logs
        .lines()
        .find(|l| {
            l.contains("embedding.drain")
                && l.contains("outcome=\"failed\"")
                && l.contains("error_class=\"embedding\"")
        })
        .expect("drain operation event with the error class");
    assert!(line.contains("duration_ms="), "{line}");
    assert_logs_exclude(&[BODY, TEXT]);
}
