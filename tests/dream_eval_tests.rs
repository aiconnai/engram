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
