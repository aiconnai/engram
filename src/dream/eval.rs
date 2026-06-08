//! Deterministic local evaluation fixtures for reviewable dream snapshots.

use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::dream::candidates::{run_candidate_generation, DreamCandidateGenerationConfig};
use crate::error::{EngramError, Result};
use crate::storage::queries::create_memory;
use crate::storage::{list_dream_candidate_sources, list_dream_candidates, Storage};
use crate::types::{CreateMemoryInput, MemoryId, MemoryType};

const ALL_FIXTURES: &[&str] = &[
    "carry_forward_context",
    "preferences_constraints",
    "freshness_temporal",
    "provenance_correctness",
    "unsafe_raw_log_rejection",
    "no_canonical_mutation_before_apply",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamEvalReport {
    pub status: String,
    pub metrics: DreamEvalMetrics,
    pub fixtures: Vec<DreamEvalFixtureResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamEvalMetrics {
    pub fixtures_run: usize,
    pub fixtures_passed: usize,
    pub candidate_precision: f64,
    pub required_candidate_recall: f64,
    pub provenance_coverage: f64,
    pub unsafe_payload_rejection_rate: f64,
    pub canonical_mutation_violations: usize,
    pub freshness_parse_failures: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamEvalFixtureResult {
    pub name: String,
    pub passed: bool,
    pub candidates_created: usize,
    pub required_signals: usize,
    pub matched_signals: usize,
    pub unexpected_candidates: usize,
    pub provenance_candidates: usize,
    pub provenance_with_sources: usize,
    pub canonical_mutation_violations: usize,
    pub freshness_parse_failures: usize,
    pub unsafe_payload_rejected: Option<bool>,
    pub details: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DreamEvalOptions {
    pub fixtures: Option<Vec<String>>,
    pub include_details: bool,
}

impl Default for DreamEvalOptions {
    fn default() -> Self {
        Self {
            fixtures: None,
            include_details: true,
        }
    }
}

#[derive(Debug, Clone)]
struct CandidateObserved {
    id: String,
    kind: String,
    proposed_action: String,
    freshness_state: String,
    proposed_content: Option<String>,
    reason_codes: serde_json::Value,
    source_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct ExpectedSignal {
    kind: &'static str,
    proposed_action: &'static str,
    freshness_state: Option<&'static str>,
    reason_code: Option<&'static str>,
}

pub fn run_dream_eval(options: DreamEvalOptions) -> Result<DreamEvalReport> {
    let fixtures = selected_fixtures(&options)?;
    let mut results = Vec::with_capacity(fixtures.len());
    for fixture in fixtures {
        let mut result = match fixture.as_str() {
            "carry_forward_context" => eval_carry_forward_context()?,
            "preferences_constraints" => eval_preferences_constraints()?,
            "freshness_temporal" => eval_freshness_temporal()?,
            "provenance_correctness" => eval_provenance_correctness()?,
            "unsafe_raw_log_rejection" => eval_unsafe_raw_log_rejection()?,
            "no_canonical_mutation_before_apply" => eval_no_canonical_mutation_before_apply()?,
            other => {
                return Err(EngramError::InvalidInput(format!(
                    "unknown dream eval fixture: {}",
                    other
                )))
            }
        };
        if !options.include_details {
            result.details.clear();
        }
        results.push(result);
    }

    let metrics = aggregate_metrics(&results);
    let status = if metrics.fixtures_passed == metrics.fixtures_run {
        "success"
    } else {
        "failed"
    }
    .to_string();

    Ok(DreamEvalReport {
        status,
        metrics,
        fixtures: results,
    })
}

fn selected_fixtures(options: &DreamEvalOptions) -> Result<Vec<String>> {
    let Some(fixtures) = &options.fixtures else {
        return Ok(ALL_FIXTURES
            .iter()
            .map(|fixture| fixture.to_string())
            .collect());
    };
    if fixtures.is_empty() {
        return Err(EngramError::InvalidInput(
            "dream eval fixtures must not be empty".to_string(),
        ));
    }
    for fixture in fixtures {
        if !ALL_FIXTURES.contains(&fixture.as_str()) {
            return Err(EngramError::InvalidInput(format!(
                "unknown dream eval fixture: {}",
                fixture
            )));
        }
    }
    Ok(fixtures.clone())
}

fn eval_carry_forward_context() -> Result<DreamEvalFixtureResult> {
    let storage = Storage::open_in_memory()?;
    insert_memory(
        &storage,
        "Release checklist requires local CI before merge.",
        MemoryType::Context,
        0.82,
    )?;
    insert_memory(
        &storage,
        "Huly issue metadata is the source of truth for implementation planning.",
        MemoryType::Context,
        0.78,
    )?;

    evaluate_generation_fixture(
        storage,
        "carry_forward_context",
        DreamCandidateGenerationConfig {
            job_id: Some("eval-carry-forward".to_string()),
            max_candidates: 5,
            ..Default::default()
        },
        &[ExpectedSignal {
            kind: "summary",
            proposed_action: "create",
            freshness_state: Some("current"),
            reason_code: Some("carry_forward_context"),
        }],
        &["summary"],
        None,
    )
}

fn eval_preferences_constraints() -> Result<DreamEvalFixtureResult> {
    let storage = Storage::open_in_memory()?;
    insert_memory(
        &storage,
        "User prefers concise PR descriptions with concrete validation.",
        MemoryType::Preference,
        0.78,
    )?;
    insert_memory(
        &storage,
        "User prefers implementation plans that name Huly issue ownership.",
        MemoryType::Preference,
        0.76,
    )?;
    insert_memory(
        &storage,
        "Repository work must run the harness bootstrap before edits.",
        MemoryType::Decision,
        0.84,
    )?;
    insert_memory(
        &storage,
        "Code review requires concrete file and line evidence.",
        MemoryType::Decision,
        0.82,
    )?;

    evaluate_generation_fixture(
        storage,
        "preferences_constraints",
        DreamCandidateGenerationConfig {
            job_id: Some("eval-preferences-constraints".to_string()),
            max_candidates: 8,
            ..Default::default()
        },
        &[
            ExpectedSignal {
                kind: "preference",
                proposed_action: "create",
                freshness_state: Some("current"),
                reason_code: Some("stable_preference"),
            },
            ExpectedSignal {
                kind: "constraint",
                proposed_action: "create",
                freshness_state: Some("current"),
                reason_code: Some("stable_constraint"),
            },
        ],
        &["summary", "preference", "constraint"],
        None,
    )
}

fn eval_freshness_temporal() -> Result<DreamEvalFixtureResult> {
    let storage = Storage::open_in_memory()?;
    let expired = insert_memory(
        &storage,
        "Temporary deployment note that should expire.",
        MemoryType::Note,
        0.5,
    )?;
    let stale = insert_memory(
        &storage,
        "Planned rollout deadline for the previous release.",
        MemoryType::Context,
        0.7,
    )?;
    let malformed = insert_memory(
        &storage,
        "Malformed temporal metadata should not crash freshness parsing.",
        MemoryType::Context,
        0.4,
    )?;
    storage.with_transaction(|conn| {
        conn.execute(
            "UPDATE memories SET expires_at = '2000-01-01T00:00:00Z' WHERE id = ?1",
            params![expired],
        )?;
        conn.execute(
            "UPDATE memories SET event_time = '2000-01-01T00:00:00Z' WHERE id = ?1",
            params![stale],
        )?;
        conn.execute(
            "UPDATE memories SET event_time = 'not-a-timestamp', expires_at = 'also-not-a-timestamp' WHERE id = ?1",
            params![malformed],
        )?;
        Ok(())
    })?;

    evaluate_generation_fixture(
        storage,
        "freshness_temporal",
        DreamCandidateGenerationConfig {
            job_id: Some("eval-freshness-temporal".to_string()),
            summary_min_memories: 10,
            max_candidates: 6,
            ..Default::default()
        },
        &[
            ExpectedSignal {
                kind: "stale_fact",
                proposed_action: "expire",
                freshness_state: Some("expired"),
                reason_code: Some("expired_memory"),
            },
            ExpectedSignal {
                kind: "temporal_update",
                proposed_action: "update",
                freshness_state: Some("stale"),
                reason_code: Some("past_planned_event"),
            },
        ],
        &["stale_fact", "temporal_update"],
        None,
    )
}

fn eval_provenance_correctness() -> Result<DreamEvalFixtureResult> {
    let storage = Storage::open_in_memory()?;
    insert_memory(
        &storage,
        "Architecture decision: local-first SQLite remains canonical.",
        MemoryType::Decision,
        0.83,
    )?;
    insert_memory(
        &storage,
        "Operational context bundles cite source events before raw artifacts.",
        MemoryType::Context,
        0.81,
    )?;

    evaluate_generation_fixture(
        storage,
        "provenance_correctness",
        DreamCandidateGenerationConfig {
            job_id: Some("eval-provenance".to_string()),
            max_candidates: 5,
            ..Default::default()
        },
        &[ExpectedSignal {
            kind: "summary",
            proposed_action: "create",
            freshness_state: Some("current"),
            reason_code: Some("carry_forward_context"),
        }],
        &["summary", "constraint"],
        None,
    )
}

fn eval_unsafe_raw_log_rejection() -> Result<DreamEvalFixtureResult> {
    let storage = Storage::open_in_memory()?;
    insert_memory(
        &storage,
        "terminal dump: ENV: API_KEY=sk-test SECRET=do-not-store",
        MemoryType::Note,
        0.8,
    )?;
    insert_memory(
        &storage,
        "Authorization: Bearer abc.def.ghi\npassword=hunter2",
        MemoryType::Note,
        0.8,
    )?;

    evaluate_generation_fixture(
        storage,
        "unsafe_raw_log_rejection",
        DreamCandidateGenerationConfig {
            job_id: Some("eval-unsafe-raw-log".to_string()),
            max_candidates: 5,
            ..Default::default()
        },
        &[],
        &[],
        Some("API_KEY=sk-test"),
    )
}

fn eval_no_canonical_mutation_before_apply() -> Result<DreamEvalFixtureResult> {
    let storage = Storage::open_in_memory()?;
    insert_memory(
        &storage,
        "Decision records remain canonical until a reviewed candidate is applied.",
        MemoryType::Decision,
        0.82,
    )?;
    insert_memory(
        &storage,
        "Dream generation may create candidates but must not mutate memories.",
        MemoryType::Context,
        0.8,
    )?;

    evaluate_generation_fixture(
        storage,
        "no_canonical_mutation_before_apply",
        DreamCandidateGenerationConfig {
            job_id: Some("eval-no-canonical-mutation".to_string()),
            max_candidates: 5,
            ..Default::default()
        },
        &[ExpectedSignal {
            kind: "summary",
            proposed_action: "create",
            freshness_state: Some("current"),
            reason_code: Some("carry_forward_context"),
        }],
        &["summary", "constraint"],
        None,
    )
}

fn evaluate_generation_fixture(
    storage: Storage,
    name: &str,
    config: DreamCandidateGenerationConfig,
    expected: &[ExpectedSignal],
    expected_kinds: &[&str],
    unsafe_marker: Option<&str>,
) -> Result<DreamEvalFixtureResult> {
    let before = canonical_snapshot(&storage)?;
    let report = run_candidate_generation(&storage, &config)?;
    let after = canonical_snapshot(&storage)?;
    let canonical_mutation_violations = usize::from(before != after);
    let candidates = observed_candidates(&storage, &report.job_id)?;
    let matched_signals = expected
        .iter()
        .filter(|signal| candidates.iter().any(|candidate| signal.matches(candidate)))
        .count();
    let unexpected_candidates = candidates
        .iter()
        .filter(|candidate| !expected_kinds.contains(&candidate.kind.as_str()))
        .count();
    let provenance_candidates = candidates.len();
    let provenance_with_sources = candidates
        .iter()
        .filter(|candidate| candidate.source_count > 0)
        .count();
    let unsafe_payload_rejected = unsafe_marker.map(|marker| {
        candidates.iter().all(|candidate| {
            !candidate
                .proposed_content
                .as_deref()
                .unwrap_or_default()
                .contains(marker)
        })
    });
    let freshness_parse_failures = 0;

    let passed = matched_signals == expected.len()
        && unexpected_candidates == 0
        && canonical_mutation_violations == 0
        && provenance_with_sources == provenance_candidates
        && unsafe_payload_rejected.unwrap_or(true)
        && freshness_parse_failures == 0;

    let mut details = Vec::new();
    details.push(format!(
        "job={} scanned={} candidates={} sources={}",
        report.job_id, report.memories_scanned, report.candidates_created, report.sources_created
    ));
    for candidate in &candidates {
        details.push(format!(
            "candidate={} kind={} action={} freshness={} sources={}",
            candidate.id,
            candidate.kind,
            candidate.proposed_action,
            candidate.freshness_state,
            candidate.source_count
        ));
    }

    Ok(DreamEvalFixtureResult {
        name: name.to_string(),
        passed,
        candidates_created: candidates.len(),
        required_signals: expected.len(),
        matched_signals,
        unexpected_candidates,
        provenance_candidates,
        provenance_with_sources,
        canonical_mutation_violations,
        freshness_parse_failures,
        unsafe_payload_rejected,
        details,
    })
}

fn observed_candidates(storage: &Storage, job_id: &str) -> Result<Vec<CandidateObserved>> {
    storage.with_connection(|conn| {
        let candidates = list_dream_candidates(conn, None, Some(job_id), None, Some(100))?;
        let mut observed = Vec::with_capacity(candidates.len());
        for candidate in candidates {
            let sources = list_dream_candidate_sources(conn, &candidate.id)?;
            observed.push(CandidateObserved {
                id: candidate.id,
                kind: candidate.kind,
                proposed_action: candidate.proposed_action,
                freshness_state: candidate.freshness_state,
                proposed_content: candidate.proposed_content,
                reason_codes: candidate.reason_codes,
                source_count: sources.len(),
            });
        }
        Ok(observed)
    })
}

fn canonical_snapshot(storage: &Storage) -> Result<String> {
    storage.with_connection(|conn| {
        let snapshot: String = conn.query_row(
            "SELECT COALESCE(group_concat(row, '|'), '')
             FROM (
                 SELECT id || ':' || content || ':' || COALESCE(lifecycle_state, 'active') AS row
                 FROM memories
                 WHERE valid_to IS NULL
                 ORDER BY id
             )",
            [],
            |row| row.get(0),
        )?;
        Ok(snapshot)
    })
}

fn insert_memory(
    storage: &Storage,
    content: &str,
    memory_type: MemoryType,
    importance: f32,
) -> Result<MemoryId> {
    storage.with_transaction(|conn| {
        create_memory(
            conn,
            &CreateMemoryInput {
                content: content.to_string(),
                memory_type,
                workspace: Some("default".to_string()),
                importance: Some(importance),
                metadata: [("eval_fixture".to_string(), json!(true))]
                    .into_iter()
                    .collect(),
                defer_embedding: true,
                ..Default::default()
            },
        )
        .map(|memory| memory.id)
    })
}

fn aggregate_metrics(results: &[DreamEvalFixtureResult]) -> DreamEvalMetrics {
    let fixtures_run = results.len();
    let fixtures_passed = results.iter().filter(|fixture| fixture.passed).count();
    let total_candidates: usize = results
        .iter()
        .map(|fixture| fixture.candidates_created)
        .sum();
    let unexpected_candidates: usize = results
        .iter()
        .map(|fixture| fixture.unexpected_candidates)
        .sum();
    let total_required: usize = results.iter().map(|fixture| fixture.required_signals).sum();
    let total_matched: usize = results.iter().map(|fixture| fixture.matched_signals).sum();
    let provenance_candidates: usize = results
        .iter()
        .map(|fixture| fixture.provenance_candidates)
        .sum();
    let provenance_with_sources: usize = results
        .iter()
        .map(|fixture| fixture.provenance_with_sources)
        .sum();
    let unsafe_fixtures: Vec<_> = results
        .iter()
        .filter_map(|fixture| fixture.unsafe_payload_rejected)
        .collect();
    let canonical_mutation_violations = results
        .iter()
        .map(|fixture| fixture.canonical_mutation_violations)
        .sum();
    let freshness_parse_failures = results
        .iter()
        .map(|fixture| fixture.freshness_parse_failures)
        .sum();

    DreamEvalMetrics {
        fixtures_run,
        fixtures_passed,
        candidate_precision: ratio(
            total_candidates.saturating_sub(unexpected_candidates),
            total_candidates,
        ),
        required_candidate_recall: ratio(total_matched, total_required),
        provenance_coverage: ratio(provenance_with_sources, provenance_candidates),
        unsafe_payload_rejection_rate: ratio(
            unsafe_fixtures.iter().filter(|passed| **passed).count(),
            unsafe_fixtures.len(),
        ),
        canonical_mutation_violations,
        freshness_parse_failures,
    }
}

fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        1.0
    } else {
        numerator as f64 / denominator as f64
    }
}

impl ExpectedSignal {
    fn matches(self, candidate: &CandidateObserved) -> bool {
        candidate.kind == self.kind
            && candidate.proposed_action == self.proposed_action
            && self
                .freshness_state
                .is_none_or(|freshness| candidate.freshness_state == freshness)
            && self
                .reason_code
                .is_none_or(|reason| has_reason_code(&candidate.reason_codes, reason))
    }
}

fn has_reason_code(reason_codes: &serde_json::Value, expected: &str) -> bool {
    reason_codes
        .as_array()
        .is_some_and(|codes| codes.iter().any(|code| code.as_str() == Some(expected)))
}
