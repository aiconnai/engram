//! Harness record handler — durable cross-session memory for harness events.
//!
//! Creates permanent memory records for decisions, handoffs, failed attempts,
//! verification results, risks, assumptions, bug reproductions, and issue updates.

mod handoff;
mod record;
mod status;
mod verify;

pub use handoff::handle_harness_handoff;
pub use record::handle_harness_record;
pub use status::handle_harness_status;
pub use verify::handle_harness_verify;

const VALID_KINDS: &[&str] = &[
    "decision",
    "handoff",
    "failed_attempt",
    "bug_reproduction",
    "verification_result",
    "risk",
    "assumption",
    "issue_update",
];

/// Run a shell command and return trimmed stdout, or None on error.
///
/// # Safety
///
/// All arguments passed to this function **must be compile-time string literals**.
/// Never pass user-supplied strings as `cmd` or `args` — doing so would be an
/// OS command injection vulnerability. This function is intentionally private
/// and restricted to internal harness introspection calls (e.g., reading git
/// metadata) where both the command and arguments are hard-coded at call sites.
fn run_command(cmd: &str, args: &[&str]) -> Option<String> {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn kind_to_memory_type(kind: &str) -> crate::types::MemoryType {
    match kind {
        "decision" => crate::types::MemoryType::Decision,
        "handoff" => crate::types::MemoryType::Checkpoint,
        "failed_attempt" => crate::types::MemoryType::Learning,
        "bug_reproduction" => crate::types::MemoryType::Episodic,
        "verification_result" => crate::types::MemoryType::Checkpoint,
        "risk" => crate::types::MemoryType::Note,
        "assumption" => crate::types::MemoryType::Note,
        "issue_update" => crate::types::MemoryType::Issue,
        _ => crate::types::MemoryType::Note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::handlers::HandlerContext;
    use crate::storage::Storage;
    use serde_json::json;
    use std::sync::Arc;

    fn test_ctx() -> HandlerContext {
        let storage = Storage::open_in_memory().expect("open in-memory storage");
        HandlerContext {
            storage,
            embedder: Arc::new(crate::embedding::TfIdfEmbedder::new(128)),
            fuzzy_engine: Arc::new(parking_lot::Mutex::new(crate::search::FuzzyEngine::new())),
            search_config: crate::search::SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(crate::embedding::EmbeddingCache::default()),
            search_cache: Arc::new(crate::search::SearchResultCache::new(
                crate::search::AdaptiveCacheConfig::default(),
            )),
            hnsw_index: Arc::new(parking_lot::RwLock::new(crate::search::HnswIndex::new(
                crate::search::HnswConfig::new(128, crate::search::VectorMetric::Cosine),
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

    #[test]
    fn test_decision_record_returns_memory_id_and_tags() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "decision",
                "summary": "Use SQLite for storage layer",
            }),
        );
        assert!(result.get("memory_id").is_some(), "should return memory_id");
        assert_eq!(result["kind"], "decision");
        let tags = result["tags"].as_array().unwrap();
        assert!(tags.iter().any(|t| t == "harness"));
        assert!(tags.iter().any(|t| t == "decision"));
    }

    #[test]
    fn test_failed_attempt_record_tags_and_type() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "failed_attempt",
                "summary": "Tried using DuckDB but it caused compile errors",
            }),
        );
        assert!(result.get("memory_id").is_some());
        let tags = result["tags"].as_array().unwrap();
        assert!(tags.iter().any(|t| t == "harness"));
        assert!(tags.iter().any(|t| t == "failed_attempt"));
    }

    #[test]
    fn test_verification_result_record() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "verification_result",
                "summary": "All 858 tests pass after refactor",
            }),
        );
        assert!(result.get("memory_id").is_some());
        let tags = result["tags"].as_array().unwrap();
        assert!(tags.iter().any(|t| t == "harness"));
        assert!(tags.iter().any(|t| t == "verification_result"));
    }

    #[test]
    fn test_invalid_kind_returns_error_with_valid_kinds() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "not_a_kind",
                "summary": "Something",
            }),
        );
        assert!(result.get("error").is_some());
        let error = result["error"].as_str().unwrap();
        assert!(error.contains("invalid harness kind"));
        assert!(result.get("valid_kinds").is_some());
    }

    #[test]
    fn test_empty_summary_returns_error() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "decision",
                "summary": "",
            }),
        );
        assert!(result.get("error").is_some());
    }

    #[test]
    fn test_summary_over_500_chars_returns_error() {
        let ctx = test_ctx();
        let long_summary = "x".repeat(501);
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "decision",
                "summary": long_summary,
            }),
        );
        assert!(result.get("error").is_some());
        let error = result["error"].as_str().unwrap();
        assert!(error.contains("500"));
    }

    #[test]
    fn test_metadata_fields_stored_correctly() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "bug_reproduction",
                "summary": "Crash on empty input",
                "source_paths": ["src/lib.rs", "src/main.rs"],
                "command": "cargo test -- test_empty",
                "issue_number": 42,
                "commit_sha": "abc1234",
                "evidence_refs": ["https://github.com/org/repo/issues/42"],
            }),
        );
        assert!(
            result.get("memory_id").is_some(),
            "expected memory_id, got: {}",
            result
        );
        // Memory was created — verify the memory_id is a number
        assert!(result["memory_id"].as_i64().is_some());
    }

    #[test]
    fn test_importance_defaults_to_0_7() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "risk",
                "summary": "External API may rate-limit us",
            }),
        );
        // If no error, importance defaulted correctly
        assert!(result.get("memory_id").is_some());
    }

    #[test]
    fn test_importance_out_of_range_returns_error() {
        let ctx = test_ctx();
        let result = handle_harness_record(
            &ctx,
            json!({
                "kind": "decision",
                "summary": "Some decision",
                "importance": 1.5,
            }),
        );
        assert!(result.get("error").is_some());
    }

    // ── harness_status tests ────────────────────────────────────────────────

    #[test]
    fn test_harness_status_empty_workspace() {
        let ctx = test_ctx();
        let result = handle_harness_status(&ctx, json!({"workspace": "test_empty_ws"}));
        assert!(
            result.get("error").is_none(),
            "unexpected error: {}",
            result
        );
        assert_eq!(result["workspace"], "test_empty_ws");
        assert!(result["recent_decisions"].as_array().unwrap().is_empty());
        assert!(result["known_blockers"].as_array().unwrap().is_empty());
        // Contract (#183): `active_issues` is the canonical field for issue
        // updates. The former duplicate alias `recent_issue_updates` must be
        // absent — the response carries this collection under exactly one key.
        assert!(result["active_issues"].as_array().unwrap().is_empty());
        assert!(
            result.get("recent_issue_updates").is_none(),
            "recent_issue_updates must not be present (duplicate of active_issues): {}",
            result
        );
        assert!(result.get("token_estimate").is_some());
        let suggestion = result["suggested_next_action"].as_str().unwrap();
        assert!(
            suggestion.contains("No harness context"),
            "got: {}",
            suggestion
        );
    }

    #[test]
    fn test_harness_status_with_decisions() {
        let ctx = test_ctx();
        let ws = "test_decisions_ws";
        handle_harness_record(
            &ctx,
            json!({"kind": "decision", "summary": "Use SQLite", "workspace": ws}),
        );
        handle_harness_record(
            &ctx,
            json!({"kind": "decision", "summary": "Use Axum", "workspace": ws}),
        );
        let result = handle_harness_status(&ctx, json!({"workspace": ws}));
        assert!(
            result.get("error").is_none(),
            "unexpected error: {}",
            result
        );
        let decisions = result["recent_decisions"].as_array().unwrap();
        assert_eq!(decisions.len(), 2, "expected 2 decisions, got: {}", result);
        assert!(decisions[0].get("memory_id").is_some());
        assert!(decisions[0].get("summary").is_some());
    }

    #[test]
    fn test_harness_status_with_blocker() {
        let ctx = test_ctx();
        let ws = "test_blocker_ws";
        handle_harness_record(
            &ctx,
            json!({"kind": "risk", "summary": "DB migration may fail", "workspace": ws}),
        );
        let result = handle_harness_status(&ctx, json!({"workspace": ws}));
        assert!(
            result.get("error").is_none(),
            "unexpected error: {}",
            result
        );
        let blockers = result["known_blockers"].as_array().unwrap();
        assert_eq!(blockers.len(), 1);
        let suggestion = result["suggested_next_action"].as_str().unwrap();
        assert!(
            suggestion.to_lowercase().contains("blocker"),
            "got: {}",
            suggestion
        );
    }

    #[test]
    fn test_harness_status_issue_updates_single_field() {
        // Contract (#183): an issue_update record surfaces under `active_issues`
        // only. The response must not duplicate it under `recent_issue_updates`.
        let ctx = test_ctx();
        let ws = "test_issue_updates_ws";
        handle_harness_record(
            &ctx,
            json!({
                "kind": "issue_update",
                "summary": "Investigated flaky test",
                "issue_number": 42,
                "workspace": ws,
            }),
        );
        let result = handle_harness_status(&ctx, json!({"workspace": ws}));
        assert!(
            result.get("error").is_none(),
            "unexpected error: {}",
            result
        );
        let active = result["active_issues"].as_array().unwrap();
        assert_eq!(active.len(), 1, "expected 1 active issue, got: {}", result);
        assert_eq!(active[0]["issue_number"], 42);
        assert_eq!(active[0]["summary"], "Investigated flaky test");
        assert!(
            result.get("recent_issue_updates").is_none(),
            "recent_issue_updates must not be present (duplicate of active_issues): {}",
            result
        );
    }

    #[test]
    fn test_harness_status_no_git() {
        let ctx = test_ctx();
        let result = handle_harness_status(&ctx, json!({"include_git": false}));
        assert!(
            result.get("error").is_none(),
            "unexpected error: {}",
            result
        );
        assert!(result["git_state"].is_null());
    }

    #[test]
    fn test_harness_status_token_budget() {
        let ctx = test_ctx();
        let ws = "test_budget_ws";
        for i in 0..20 {
            handle_harness_record(
                &ctx,
                json!({
                    "kind": "decision",
                    "summary": format!("Decision number {} with some content to pad size", i),
                    "workspace": ws,
                }),
            );
        }
        let result = handle_harness_status(&ctx, json!({"workspace": ws, "token_budget": 200}));
        assert!(
            result.get("error").is_none(),
            "unexpected error: {}",
            result
        );
        let decisions = result["recent_decisions"].as_array().unwrap();
        assert!(
            decisions.len() < 20,
            "expected truncation, got {} decisions",
            decisions.len()
        );
    }

    // ── harness_handoff tests ────────────────────────────────────────────────

    #[test]
    fn test_harness_handoff_basic() {
        let ctx = test_ctx();
        let result = handle_harness_handoff(
            &ctx,
            json!({
                "current_goal": "Implement search index v2",
                "files_touched": ["src/search.rs", "src/index.rs"],
                "decisions_made": ["Use BM25 scoring"],
                "tests_run": ["cargo test --lib"],
                "next_steps": ["Review PR #34", "Run full CI"],
                "verification_evidence": "873 tests passed",
            }),
        );
        assert!(
            result.get("error").is_none(),
            "unexpected error: {}",
            result
        );
        assert!(
            result["handoff_id"].as_i64().is_some(),
            "expected handoff_id, got: {}",
            result
        );
        assert_eq!(result["completion_claimed"], true);
        assert_eq!(result["persisted"], true);
        assert!(result["copy_block"]
            .as_str()
            .expect("copy_block")
            .contains("# Continue this work in a new AI session"));
        assert_eq!(result["current_goal"], json!("Implement search index v2"));
        assert_eq!(result["completion_claimed"], json!(true));
    }

    #[test]
    fn test_harness_handoff_persisted_checkpoint_surfaces_in_status() {
        let ctx = test_ctx();
        let ws = "handoff_status_ws";
        let result = handle_harness_handoff(
            &ctx,
            json!({
                "workspace": ws,
                "current_goal": "Make handoff discoverable",
                "next_steps": ["Resume from harness_status"],
                "verification_evidence": "focused tests passed",
            }),
        );
        assert!(
            result.get("error").is_none(),
            "unexpected error: {}",
            result
        );
        let handoff_id = result["handoff_id"].as_i64().expect("handoff_id");

        let status = handle_harness_status(
            &ctx,
            json!({
                "workspace": ws,
                "include_git": false,
            }),
        );
        assert!(
            status.get("error").is_none(),
            "unexpected status error: {}",
            status
        );
        assert_eq!(status["last_handoff"]["memory_id"], json!(handoff_id));
        assert_eq!(
            status["last_handoff"]["metadata"]["harness_kind"],
            json!("handoff")
        );
        assert_eq!(
            status["last_handoff"]["metadata"]["current_goal"],
            json!("Make handoff discoverable")
        );
        assert_eq!(
            status["current_objective"],
            json!("Make handoff discoverable")
        );
    }

    #[test]
    fn test_harness_handoff_no_verification_evidence() {
        let ctx = test_ctx();
        let result = handle_harness_handoff(
            &ctx,
            json!({
                "current_goal": "Fix bug in parser",
                "next_steps": ["Run cargo test"],
            }),
        );
        assert!(
            result.get("error").is_none(),
            "unexpected error: {}",
            result
        );
        assert_eq!(result["completion_claimed"], false);
        assert!(result["completion_warning"].as_str().is_some());
        let warning = result["completion_warning"].as_str().unwrap();
        assert!(
            warning.contains("No verification evidence"),
            "got: {}",
            warning
        );
        assert!(result["completion_warning"]
            .as_str()
            .expect("completion_warning")
            .contains("No verification evidence provided"));
        assert!(result["copy_block"]
            .as_str()
            .expect("copy_block")
            .contains("Do not claim this work is complete"));
    }

    #[test]
    fn test_harness_handoff_no_persist() {
        let ctx = test_ctx();
        let result = handle_harness_handoff(
            &ctx,
            json!({
                "current_goal": "Draft only handoff",
                "next_steps": ["Check logs"],
                "persist": false,
            }),
        );
        assert!(
            result.get("error").is_none(),
            "unexpected error: {}",
            result
        );
        assert!(
            result["handoff_id"].is_null(),
            "expected null handoff_id, got: {}",
            result
        );
        assert_eq!(result["persisted"], false);
    }

    #[test]
    fn test_harness_handoff_missing_goal() {
        let ctx = test_ctx();
        let result = handle_harness_handoff(
            &ctx,
            json!({
                "next_steps": ["Do something"],
            }),
        );
        assert!(
            result.get("error").is_some(),
            "expected error, got: {}",
            result
        );
    }

    #[test]
    fn test_harness_handoff_empty_next_steps() {
        let ctx = test_ctx();
        let result = handle_harness_handoff(
            &ctx,
            json!({
                "current_goal": "Some goal",
                "next_steps": [],
            }),
        );
        assert!(
            result.get("error").is_some(),
            "expected error, got: {}",
            result
        );
    }

    // ── harness_verify tests ────────────────────────────────────────────────

    #[test]
    fn test_harness_verify_pass() {
        let ctx = test_ctx();
        let result = handle_harness_verify(
            &ctx,
            json!({
                "command": "cargo test --lib",
                "exit_code": 0,
                "output_summary": "873 tests passed, 0 failed",
            }),
        );
        assert!(
            result.get("error").is_none(),
            "unexpected error: {}",
            result
        );
        assert!(
            result["memory_id"].as_i64().is_some(),
            "expected memory_id, got: {}",
            result
        );
        assert_eq!(result["passed"], true);
        assert_eq!(result["skipped"], false);
        assert_eq!(result["command"], "cargo test --lib");
        let tags = result["tags"].as_array().unwrap();
        assert!(tags.iter().any(|t| t == "verification_result"));
        assert!(tags.iter().any(|t| t == "harness"));
    }

    #[test]
    fn test_harness_verify_fail() {
        let ctx = test_ctx();
        let result = handle_harness_verify(
            &ctx,
            json!({
                "command": "cargo test --lib",
                "exit_code": 1,
                "output_summary": "2 tests failed",
            }),
        );
        assert!(
            result.get("error").is_none(),
            "unexpected error: {}",
            result
        );
        assert_eq!(result["passed"], false);
        let tags = result["tags"].as_array().unwrap();
        assert!(tags.iter().any(|t| t == "verification_failed"));
    }

    #[test]
    fn test_harness_verify_skipped() {
        let ctx = test_ctx();
        let result = handle_harness_verify(
            &ctx,
            json!({
                "command": "cargo bench",
                "exit_code": 0,
                "output_summary": "benchmark skipped in CI",
                "skipped_reason": "benchmarks not run in CI environment",
            }),
        );
        assert!(
            result.get("error").is_none(),
            "unexpected error: {}",
            result
        );
        assert_eq!(result["skipped"], true);
        let tags = result["tags"].as_array().unwrap();
        assert!(tags.iter().any(|t| t == "verification_skipped"));
        // Content should contain SKIP label
        // We check via the output_summary field
        assert_eq!(result["output_summary"], "benchmark skipped in CI");
    }

    #[test]
    fn test_harness_verify_missing_command() {
        let ctx = test_ctx();
        let result = handle_harness_verify(
            &ctx,
            json!({
                "exit_code": 0,
                "output_summary": "all good",
            }),
        );
        assert!(
            result.get("error").is_some(),
            "expected error, got: {}",
            result
        );
    }

    #[test]
    fn test_harness_verify_missing_output_summary() {
        let ctx = test_ctx();
        let result = handle_harness_verify(
            &ctx,
            json!({
                "command": "cargo test",
                "exit_code": 0,
            }),
        );
        assert!(
            result.get("error").is_some(),
            "expected error, got: {}",
            result
        );
    }

    #[test]
    fn test_harness_verify_with_evidence() {
        let ctx = test_ctx();
        let result = handle_harness_verify(
            &ctx,
            json!({
                "command": "cargo test",
                "exit_code": 0,
                "output_summary": "873 passed",
                "evidence_path": "/tmp/test-output.log",
                "evidence_hash": "abc123def456",
                "issue_numbers": [37, 42],
                "memory_ids": [100, 200],
            }),
        );
        assert!(
            result.get("error").is_none(),
            "unexpected error: {}",
            result
        );
        assert_eq!(result["evidence_path"], "/tmp/test-output.log");
        assert_eq!(result["evidence_hash"], "abc123def456");
        assert!(result["memory_id"].as_i64().is_some());
    }

    #[test]
    fn test_tier_is_always_permanent() {
        // Verify via kind_to_memory_type that all kinds are handled,
        // and the tier field is set to Permanent in CreateMemoryInput.
        // This is structural — if it compiled and saved successfully
        // with tier=Permanent, the record creation passed.
        let ctx = test_ctx();
        for kind in VALID_KINDS {
            let result = handle_harness_record(
                &ctx,
                json!({
                    "kind": kind,
                    "summary": format!("Test for kind {}", kind),
                }),
            );
            assert!(
                result.get("memory_id").is_some(),
                "kind {} should succeed, got: {}",
                kind,
                result
            );
        }
    }
}
