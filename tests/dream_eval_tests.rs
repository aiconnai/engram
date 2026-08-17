const RFC_0007: &str = include_str!("../docs/rfcs/0007-dream-snapshot-review-pipeline.md");
const EVAL_RUNBOOK: &str = include_str!("../docs/DREAM_SNAPSHOT_EVALS.md");

fn contains_normalized(haystack: &str, needle: &str) -> bool {
    let normalized_haystack = haystack.split_whitespace().collect::<Vec<_>>().join(" ");
    let normalized_needle = needle.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized_haystack.contains(&normalized_needle)
}

#[test]
fn dream_snapshot_rfc_preserves_review_boundary_contract() {
    for required in [
        "Candidates are derived proposals, not accepted memories.",
        "No dream job may silently create, update, delete, expire, promote, demote, or supersede canonical memory rows.",
        "Applying a candidate requires explicit review and confirmation through a mutating MCP tool.",
        "`dream_candidate_apply` is deliberately narrow in v1.",
        "requires `confirm=true`",
        "Default `memory_search`, `context_search`, and `context_build_bundle` results must not include unaccepted dream candidates.",
        "Raw payloads are excluded by default.",
        "The evals are local and deterministic in v1.",
    ] {
        assert!(
            contains_normalized(RFC_0007, required),
            "RFC 0007 is missing required dream snapshot invariant: {required}"
        );
    }
}

#[test]
fn dream_snapshot_eval_runbook_covers_required_fixture_lanes() {
    for required in [
        "carry_forward_context",
        "preferences_constraints",
        "freshness_temporal",
        "provenance_correctness",
        "unsafe_raw_log_rejection",
        "no_canonical_mutation_before_apply",
    ] {
        assert!(
            EVAL_RUNBOOK.contains(required),
            "dream eval runbook is missing fixture lane: {required}"
        );
    }
}

#[test]
fn dream_snapshot_eval_runbook_defines_ci_safe_metrics() {
    for required in [
        "fixtures_run",
        "fixtures_passed",
        "required_candidate_recall",
        "provenance_coverage",
        "unsafe_payload_rejection_rate",
        "canonical_mutation_violations",
        "freshness_parse_failures",
        "does not require network",
    ] {
        assert!(
            EVAL_RUNBOOK.contains(required),
            "dream eval runbook is missing deterministic metric contract: {required}"
        );
    }
}

#[cfg(feature = "dream-phase")]
fn test_handler_context(
    storage: engram::storage::Storage,
) -> engram::mcp::handlers::HandlerContext {
    use engram::embedding::EmbeddingCache;
    use engram::mcp::handlers::HandlerContext;
    use engram::search::{FuzzyEngine, SearchConfig, SearchResultCache};
    use parking_lot::Mutex;
    use std::sync::Arc;

    HandlerContext {
        storage,
        embedder: engram::embedding::create_embedder(&Default::default()).unwrap(),
        fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
        search_config: SearchConfig::default(),
        realtime: None,
        embedding_cache: Arc::new(EmbeddingCache::default()),
        search_cache: Arc::new(SearchResultCache::new(Default::default())),
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

#[cfg(feature = "dream-phase")]
#[test]
fn dream_eval_runner_emits_required_metrics() {
    use engram::dream::eval::{run_dream_eval, DreamEvalOptions};

    let report = run_dream_eval(DreamEvalOptions::default()).expect("run dream eval");
    assert_eq!(report.status, "success");
    assert_eq!(report.metrics.fixtures_run, 6);
    assert_eq!(report.metrics.fixtures_passed, report.metrics.fixtures_run);
    assert_eq!(report.metrics.required_candidate_recall, 1.0);
    assert_eq!(report.metrics.provenance_coverage, 1.0);
    assert_eq!(report.metrics.unsafe_payload_rejection_rate, 1.0);
    assert_eq!(report.metrics.canonical_mutation_violations, 0);
    assert_eq!(report.metrics.freshness_parse_failures, 0);
}

#[cfg(feature = "dream-phase")]
#[test]
fn dream_eval_run_mcp_tool_is_registered_and_dispatches() {
    use engram::mcp::handlers::dispatch;
    use engram::mcp::tools::get_tool_definitions;
    use engram::storage::Storage;
    use serde_json::json;

    let has_tool = get_tool_definitions()
        .iter()
        .any(|tool| tool.name == "dream_eval_run");
    assert!(has_tool, "tools/list should include dream_eval_run");

    let ctx = test_handler_context(Storage::open_in_memory().unwrap());
    let result = dispatch(
        &ctx,
        "dream_eval_run",
        json!({"fixtures": ["freshness_temporal"], "include_details": false}),
    );
    assert_eq!(result["status"], "success");
    assert_eq!(result["metrics"]["fixtures_run"], 1);
    assert_eq!(result["metrics"]["fixtures_passed"], 1);
    assert_eq!(result["metrics"]["freshness_parse_failures"], 0);
    assert_eq!(
        result["fixtures"][0]["details"].as_array().unwrap().len(),
        0
    );
}
