//! Deterministic retrieval-quality evaluation over frozen synthetic corpora.
//!
//! Two corpora live under `tests/fixtures/retrieval_quality/`:
//!
//! - `corpus.json` (v1) is the original frozen smoke corpus. It is evaluated in
//!   strict mode (every relevant memory must be retrieved) and compared byte for
//!   byte with `baseline.json`; its metrics are 1.0 by construction and prove
//!   plumbing, not retrieval quality on customer-like data.
//! - `candidate_corpus.json` (v2) is the candidate-evaluation corpus: PT/EN,
//!   typos, negation, ambiguous entities, duplicates, daily/transcript filtering
//!   and workspace isolation. It is evaluated in report mode: isolation/filter
//!   violations are hard failures, missed relevant memories are reported as
//!   metrics. `candidate_retrieval_metrics` prints one machine-readable line
//!   consumed by `scripts/run-quality-candidate.py`.
//!
//! Owner ruling #152: document ingestion limits are character based and
//! independent of tokens; this evaluation never tokenizes content for budgets.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use engram::embedding::{create_embedder, persist_computed_embedding, EmbeddingCache};
use engram::mcp::handlers::{self, HandlerContext};
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::queries::create_memory;
use engram::storage::Storage;
use engram::types::{CreateMemoryInput, EmbeddingConfig, MemoryTier, MemoryType};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const CORPUS_JSON: &str = include_str!("fixtures/retrieval_quality/corpus.json");
const CANDIDATE_CORPUS_JSON: &str =
    include_str!("fixtures/retrieval_quality/candidate_corpus.json");
const BASELINE_JSON: &str = include_str!("fixtures/retrieval_quality/baseline.json");

#[derive(Clone, Deserialize)]
struct Corpus {
    schema_version: String,
    name: String,
    version: String,
    license: String,
    source: String,
    deterministic_seed: u64,
    #[serde(default)]
    labeling: Option<Labeling>,
    memories: Vec<FixtureMemory>,
    queries: Vec<FixtureQuery>,
}

#[derive(Clone, Deserialize)]
struct Labeling {
    authored_by: String,
    labels_frozen_before_first_measurement: bool,
    human_review: String,
    review_status: String,
}

#[derive(Clone, Deserialize)]
struct FixtureMemory {
    key: String,
    workspace: String,
    content: String,
    #[serde(default)]
    memory_type: Option<String>,
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    language: Option<String>,
}

#[derive(Clone, Deserialize)]
struct FixtureQuery {
    key: String,
    query: String,
    workspace: String,
    #[serde(default)]
    fuzzy: bool,
    relevance: BTreeMap<String, u8>,
    #[serde(default)]
    forbidden_workspaces: Vec<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    include_transcripts: bool,
    /// Hard invariant: a memory that must never be returned for this query.
    #[serde(default)]
    forbidden_keys: Vec<String>,
    /// Soft diagnostic (negation, ambiguity): counted and reported, never gated.
    #[serde(default)]
    discouraged_keys: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
struct Metrics {
    #[serde(rename = "recall@10")]
    recall_at_10: f64,
    mrr: f64,
    #[serde(rename = "ndcg@10")]
    ndcg_at_10: f64,
}

#[derive(Serialize)]
struct Baseline<'a> {
    schema_version: &'a str,
    source_revision: &'a str,
    generated_at: &'a str,
    deterministic_seed: u64,
    corpus: BaselineCorpus<'a>,
    metrics: Metrics,
    benchmark_evidence: BenchmarkEvidence<'a>,
}

#[derive(Serialize)]
struct BaselineCorpus<'a> {
    name: &'a str,
    version: &'a str,
    fixture_path: &'a str,
    schema: FixtureSchema,
    memory_count: usize,
    query_count: usize,
}

#[derive(Serialize)]
struct FixtureSchema {
    memory_fields: [&'static str; 3],
    query_fields: [&'static str; 5],
    relevance_fields: [&'static str; 2],
}

#[derive(Serialize)]
struct BenchmarkEvidence<'a> {
    criterion_baseline: &'a str,
    dream_eval_runbook: &'a str,
}

fn context() -> HandlerContext {
    let embedder = create_embedder(&EmbeddingConfig::default()).expect("deterministic embedder");
    HandlerContext {
        storage: Storage::open_in_memory().expect("in-memory storage"),
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

/// How fixture memories are indexed before querying.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// FTS5/BM25 only: no embeddings are stored (what the v1 evaluation measures).
    Lexical,
    /// Deterministic TF-IDF embeddings are stored, so hybrid search runs.
    HybridTfidf,
}

impl Mode {
    const ALL: [Mode; 2] = [Mode::Lexical, Mode::HybridTfidf];

    fn name(self) -> &'static str {
        match self {
            Mode::Lexical => "lexical_fts5",
            Mode::HybridTfidf => "hybrid_tfidf",
        }
    }
}

#[derive(Debug, Clone)]
struct QueryOutcome {
    category: String,
    recall: f64,
    reciprocal_rank: f64,
    ndcg: f64,
    discouraged_hits: usize,
    discouraged_total: usize,
}

struct EvalReport {
    metrics: Metrics,
    outcomes: Vec<QueryOutcome>,
    /// Hard invariant failures (workspace/tier/transcript leaks, search errors).
    violations: Vec<String>,
    /// Queries that did not retrieve every relevant memory in the top 10.
    misses: Vec<String>,
}

fn evaluate_report(corpus: &Corpus, mode: Mode) -> EvalReport {
    let ctx = context();
    let mut ids = HashMap::new();

    for memory in &corpus.memories {
        let memory_type = memory
            .memory_type
            .as_deref()
            .map(|raw| MemoryType::from_str(raw).expect("fixture memory_type must be valid"))
            .unwrap_or_default();
        let tier = memory
            .tier
            .as_deref()
            .map(|raw| MemoryTier::from_str(raw).expect("fixture tier must be valid"))
            .unwrap_or_default();
        let created = ctx
            .storage
            .with_transaction(|conn| {
                create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: memory.content.clone(),
                        workspace: Some(memory.workspace.clone()),
                        memory_type,
                        tier,
                        ..Default::default()
                    },
                )
            })
            .expect("fixture memory must be valid");
        if mode == Mode::HybridTfidf {
            let embedding = ctx
                .embedder
                .embed(&memory.content)
                .expect("deterministic embedding");
            let model = ctx.embedder.model_name().to_owned();
            ctx.storage
                .with_transaction(|conn| {
                    persist_computed_embedding(
                        conn,
                        created.id,
                        &memory.content,
                        &embedding,
                        &model,
                    )
                })
                .expect("fixture embedding must persist");
        }
        ids.insert(created.id, memory.key.clone());
        ctx.fuzzy_engine.lock().add_to_vocabulary(&memory.content);
    }

    let key_workspace: HashMap<&str, &str> = corpus
        .memories
        .iter()
        .map(|memory| (memory.key.as_str(), memory.workspace.as_str()))
        .collect();

    let mut violations = Vec::new();
    let mut misses = Vec::new();
    let mut outcomes = Vec::new();
    let mut recall_sum = 0.0;
    let mut reciprocal_rank_sum = 0.0;
    let mut ndcg_sum = 0.0;

    for query in &corpus.queries {
        let effective_query = if query.fuzzy {
            ctx.fuzzy_engine
                .lock()
                .correct_query(&query.query)
                .corrected_query
                .unwrap_or_else(|| query.query.clone())
        } else {
            query.query.clone()
        };
        let mut params = json!({
            "query": effective_query,
            "workspace": query.workspace,
            "limit": 10,
            "rerank": false,
            "skip_cache": true
        });
        if let Some(tier) = &query.tier {
            params["tier"] = json!(tier);
        }
        if query.include_transcripts {
            params["include_transcripts"] = json!(true);
        }
        let response = handlers::dispatch(&ctx, "memory_search", params);
        let results = result_array(&response).unwrap_or_else(|| {
            violations.push(format!("{}: search returned {response}", query.key));
            &[]
        });

        let ranked: Vec<(String, String)> = results
            .iter()
            .filter_map(|result| {
                let memory = result.get("memory")?;
                let id = memory.get("id")?.as_i64()?;
                let workspace = memory.get("workspace")?.as_str()?.to_owned();
                Some((ids.get(&id)?.clone(), workspace))
            })
            .collect();

        for (key, workspace) in &ranked {
            if query
                .forbidden_workspaces
                .iter()
                .any(|item| item == workspace)
            {
                violations.push(format!(
                    "{}: forbidden workspace {workspace} leaked through result {key}",
                    query.key
                ));
            }
            if query.forbidden_keys.iter().any(|item| item == key) {
                violations.push(format!(
                    "{}: forbidden memory {key} returned (workspace {workspace})",
                    query.key
                ));
            }
            if let Some(expected) = key_workspace.get(key.as_str()) {
                if *expected != workspace {
                    violations.push(format!(
                        "{}: result {key} reported workspace {workspace}, fixture says {expected}",
                        query.key
                    ));
                }
            }
        }

        let relevant: HashSet<&str> = query.relevance.keys().map(String::as_str).collect();
        let found = ranked
            .iter()
            .take(10)
            .filter(|(key, _)| relevant.contains(key.as_str()))
            .count();
        if found != relevant.len() {
            misses.push(format!(
                "{}: retrieved {found}/{} relevant memories; ranking={ranked:?}",
                query.key,
                relevant.len()
            ));
        }
        let recall = found as f64 / relevant.len() as f64;
        recall_sum += recall;

        let reciprocal_rank = ranked
            .iter()
            .position(|(key, _)| relevant.contains(key.as_str()))
            .map_or(0.0, |rank| 1.0 / (rank + 1) as f64);
        reciprocal_rank_sum += reciprocal_rank;

        let dcg: f64 = ranked
            .iter()
            .take(10)
            .enumerate()
            .map(|(rank, (key, _))| {
                let grade = f64::from(*query.relevance.get(key).unwrap_or(&0));
                (2_f64.powf(grade) - 1.0) / ((rank + 2) as f64).log2()
            })
            .sum();
        let mut grades: Vec<u8> = query.relevance.values().copied().collect();
        grades.sort_unstable_by(|a, b| b.cmp(a));
        let ideal: f64 = grades
            .iter()
            .take(10)
            .enumerate()
            .map(|(rank, grade)| (2_f64.powf(f64::from(*grade)) - 1.0) / ((rank + 2) as f64).log2())
            .sum();
        // `+ 0.0` normalizes the negative zero produced by an empty float sum.
        let ndcg = if ideal == 0.0 { 0.0 } else { dcg / ideal } + 0.0;
        ndcg_sum += ndcg;

        let discouraged_hits = ranked
            .iter()
            .take(10)
            .filter(|(key, _)| query.discouraged_keys.iter().any(|item| item == key))
            .count();
        outcomes.push(QueryOutcome {
            category: query
                .category
                .clone()
                .unwrap_or_else(|| "uncategorized".to_owned()),
            recall,
            reciprocal_rank,
            ndcg,
            discouraged_hits,
            discouraged_total: query.discouraged_keys.len(),
        });
    }

    let count = corpus.queries.len() as f64;
    EvalReport {
        metrics: Metrics {
            recall_at_10: recall_sum / count,
            mrr: reciprocal_rank_sum / count,
            ndcg_at_10: ndcg_sum / count,
        },
        outcomes,
        violations,
        misses,
    }
}

/// Strict evaluation used by the frozen v1 corpus: any violation or missed
/// relevant memory is an error.
fn evaluate(corpus: &Corpus) -> Result<Metrics, Vec<String>> {
    let report = evaluate_report(corpus, Mode::Lexical);
    let mut errors = report.violations;
    errors.extend(report.misses);
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(report.metrics)
}

fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// Per-category aggregate of the per-query outcomes (diagnostic, not gated).
fn category_breakdown(outcomes: &[QueryOutcome]) -> Value {
    let mut grouped: BTreeMap<&str, Vec<&QueryOutcome>> = BTreeMap::new();
    for outcome in outcomes {
        grouped
            .entry(outcome.category.as_str())
            .or_default()
            .push(outcome);
    }
    let mut out = serde_json::Map::new();
    for (category, items) in grouped {
        let n = items.len() as f64;
        let mean = |f: fn(&QueryOutcome) -> f64| items.iter().map(|o| f(o)).sum::<f64>() / n;
        out.insert(
            category.to_owned(),
            json!({
                "queries": items.len(),
                "recall@10": mean(|o| o.recall),
                "mrr": mean(|o| o.reciprocal_rank),
                "ndcg@10": mean(|o| o.ndcg),
            }),
        );
    }
    Value::Object(out)
}

/// Fraction of discouraged (negated / ambiguous-sibling) memories that reached
/// the top 10, over all discouraged memories declared by the corpus.
fn discouraged_hit_rate(outcomes: &[QueryOutcome]) -> Value {
    let total: usize = outcomes.iter().map(|o| o.discouraged_total).sum();
    let hits: usize = outcomes.iter().map(|o| o.discouraged_hits).sum();
    json!({
        "discouraged_declared": total,
        "discouraged_returned_top10": hits,
        "rate": if total == 0 { 0.0 } else { hits as f64 / total as f64 },
    })
}

fn result_array(response: &Value) -> Option<&[Value]> {
    response
        .as_array()
        .or_else(|| response.get("results")?.as_array())
        .map(Vec::as_slice)
}

#[test]
fn frozen_corpus_matches_byte_identical_baseline() {
    let corpus: Corpus = serde_json::from_str(CORPUS_JSON).expect("valid corpus fixture");
    assert_eq!(corpus.schema_version, "engram.retrieval-corpus.v1");
    assert_eq!(corpus.license, "CC0-1.0");
    assert!(corpus.source.contains("no private data or PII"));

    let metrics = evaluate(&corpus).unwrap_or_else(|errors| panic!("{}", errors.join("\n")));
    let baseline = Baseline {
        schema_version: "engram.quality-baseline.v1",
        source_revision: "81be152c713230c082901899e6880579fcedabb3",
        generated_at: "2026-07-12T00:00:00Z",
        deterministic_seed: corpus.deterministic_seed,
        corpus: BaselineCorpus {
            name: &corpus.name,
            version: &corpus.version,
            fixture_path: "tests/fixtures/retrieval_quality/corpus.json",
            schema: FixtureSchema {
                memory_fields: ["key", "workspace", "content"],
                query_fields: ["key", "query", "workspace", "fuzzy", "forbidden_workspaces"],
                relevance_fields: ["memory_key", "grade"],
            },
            memory_count: corpus.memories.len(),
            query_count: corpus.queries.len(),
        },
        metrics,
        benchmark_evidence: BenchmarkEvidence {
            criterion_baseline: "benches/results/benchmark_baseline.txt",
            dream_eval_runbook: "docs/DREAM_SNAPSHOT_EVALS.md",
        },
    };
    let emitted = serde_json::to_string_pretty(&baseline).expect("serialize baseline") + "\n";
    assert_eq!(
        emitted, BASELINE_JSON,
        "regenerate and review baseline drift"
    );
    println!("{emitted}");
}

#[test]
fn evaluator_reports_missing_relevant_memory() {
    let mut corpus: Corpus = serde_json::from_str(CORPUS_JSON).expect("valid corpus fixture");
    corpus.memories.retain(|memory| memory.key != "exact-rust");
    let errors = evaluate(&corpus).expect_err("missing relevant memory must fail");
    assert!(errors
        .iter()
        .any(|error| error.contains("exact: retrieved 0/1")));
}

#[test]
fn evaluator_reports_cross_workspace_leak() {
    let mut corpus: Corpus = serde_json::from_str(CORPUS_JSON).expect("valid corpus fixture");
    let isolation = corpus
        .queries
        .iter_mut()
        .find(|query| query.key == "workspace-isolation")
        .expect("workspace isolation query");
    isolation.workspace = "alpha".to_owned();
    isolation.query = "Orchid launch".to_owned();
    isolation
        .relevance
        .insert("workspace-alpha-distractor".to_owned(), 1);
    let errors = evaluate(&corpus).expect_err("forbidden workspace must fail");
    assert!(errors
        .iter()
        .any(|error| error.contains("forbidden workspace alpha leaked")));
}

// ---------------------------------------------------------------------------
// Candidate corpus (v2): well-formedness, evaluator behavior, runner output.
// ---------------------------------------------------------------------------

const REQUIRED_CATEGORIES: [&str; 10] = [
    "exact",
    "paraphrase",
    "accent",
    "typo",
    "negation",
    "ambiguous_entity",
    "duplicates",
    "daily_filter",
    "transcript_filter",
    "workspace_isolation",
];

fn candidate_corpus() -> Corpus {
    serde_json::from_str(CANDIDATE_CORPUS_JSON).expect("valid candidate corpus")
}

#[test]
fn candidate_corpus_is_well_formed_and_covers_required_categories() {
    let corpus = candidate_corpus();
    assert_eq!(corpus.schema_version, "engram.retrieval-corpus.v2");
    assert_eq!(corpus.license, "CC0-1.0");
    assert!(corpus.source.contains("no private data or PII"));

    let labeling = corpus.labeling.as_ref().expect("labeling provenance");
    assert!(labeling.labels_frozen_before_first_measurement);
    assert!(labeling.authored_by.starts_with("LLM"));
    // An LLM-authored label set must never be recorded as human-reviewed.
    assert_eq!(labeling.human_review, "none");
    assert_eq!(labeling.review_status, "llm-authored-not-human-reviewed");

    let mut memory_keys = HashSet::new();
    for memory in &corpus.memories {
        assert!(
            memory_keys.insert(memory.key.as_str()),
            "duplicate memory key {}",
            memory.key
        );
    }
    let workspaces: HashSet<&str> = corpus
        .memories
        .iter()
        .map(|m| m.workspace.as_str())
        .collect();
    let by_key: HashMap<&str, &FixtureMemory> = corpus
        .memories
        .iter()
        .map(|m| (m.key.as_str(), m))
        .collect();

    let mut query_keys = HashSet::new();
    let mut categories = HashSet::new();
    let mut languages = HashSet::new();
    for query in &corpus.queries {
        assert!(
            query_keys.insert(query.key.as_str()),
            "duplicate query key {}",
            query.key
        );
        assert!(
            workspaces.contains(query.workspace.as_str()),
            "{}: unknown workspace",
            query.key
        );
        assert!(!query.relevance.is_empty(), "{}: no relevance", query.key);
        categories.insert(query.category.clone().expect("every query has a category"));
        languages.insert(
            query
                .language
                .clone()
                .expect("every query declares a language"),
        );
        for key in query
            .relevance
            .keys()
            .chain(&query.forbidden_keys)
            .chain(&query.discouraged_keys)
        {
            assert!(
                by_key.contains_key(key.as_str()),
                "{}: unknown key {key}",
                query.key
            );
        }
        // Labels must be reachable under the query's own filters: a relevant
        // memory in another workspace/tier or hidden transcript is a label bug.
        for key in query.relevance.keys() {
            let memory = by_key[key.as_str()];
            assert_eq!(
                memory.workspace, query.workspace,
                "{}: relevant {key} lives in another workspace",
                query.key
            );
            if let Some(tier) = &query.tier {
                let memory_tier = memory.tier.as_deref().unwrap_or("permanent");
                let memory_tier = if memory.memory_type.as_deref() == Some("transcript_chunk")
                    && memory.tier.is_none()
                {
                    "daily"
                } else {
                    memory_tier
                };
                assert_eq!(
                    memory_tier, tier,
                    "{}: relevant {key} not in tier {tier}",
                    query.key
                );
            }
            if memory.memory_type.as_deref() == Some("transcript_chunk") {
                assert!(
                    query.include_transcripts,
                    "{}: relevant transcript {key} needs include_transcripts",
                    query.key
                );
            }
        }
        for key in query.relevance.keys().chain(&query.discouraged_keys) {
            assert!(
                !query.forbidden_keys.contains(key),
                "{}: {key} is both wanted/discouraged and forbidden",
                query.key
            );
        }
    }
    for required in REQUIRED_CATEGORIES {
        assert!(
            categories.contains(required),
            "candidate corpus lacks category {required}"
        );
    }
    assert!(languages.contains("pt") && languages.contains("en"));
    let mem_languages: HashSet<&str> = corpus
        .memories
        .iter()
        .map(|m| {
            m.language
                .as_deref()
                .expect("every memory declares a language")
        })
        .collect();
    assert_eq!(mem_languages, HashSet::from(["pt", "en"]));
    // Exact duplicate content must exist (duplicates category is real).
    let mut contents = HashSet::new();
    assert!(
        corpus
            .memories
            .iter()
            .any(|m| !contents.insert((&m.workspace, &m.content))),
        "no exact duplicate memory in the candidate corpus"
    );
    assert!(corpus.memories.len() >= 40 && corpus.queries.len() >= 40);
}

#[test]
fn candidate_evaluator_reports_isolation_leak_as_violation() {
    let mut corpus = candidate_corpus();
    let query = corpus
        .queries
        .iter_mut()
        .find(|q| q.key == "q-iso-orchid-globex")
        .expect("isolation query");
    // Declare the query's own workspace forbidden: every result is a leak.
    query.forbidden_workspaces = vec!["globex".to_owned()];
    let report = evaluate_report(&corpus, Mode::Lexical);
    assert!(report
        .violations
        .iter()
        .any(|error| error.contains("forbidden workspace globex leaked")));
}

#[test]
fn candidate_evaluator_reports_missing_relevant_memory_as_miss_not_violation() {
    let mut corpus = candidate_corpus();
    corpus.memories.retain(|m| m.key != "acme-rust-ci");
    let report = evaluate_report(&corpus, Mode::Lexical);
    assert!(report
        .misses
        .iter()
        .any(|miss| miss.contains("q-en-exact-ci: retrieved 0/1")));
    assert!(report.violations.is_empty(), "{:?}", report.violations);
    let baseline = evaluate_report(&candidate_corpus(), Mode::Lexical);
    assert!(report.metrics.recall_at_10 < baseline.metrics.recall_at_10);
}

#[test]
fn candidate_evaluation_is_deterministic() {
    let corpus = candidate_corpus();
    for mode in Mode::ALL {
        let first = evaluate_report(&corpus, mode);
        let second = evaluate_report(&corpus, mode);
        assert_eq!(first.metrics, second.metrics, "{} drifted", mode.name());
    }
}

/// Computes the candidate metrics and prints exactly one
/// `ENGRAM_RETRIEVAL_EVAL_JSON=<json>` line for `scripts/run-quality-candidate.py`.
///
/// `ENGRAM_RETRIEVAL_CORPUS` selects the corpus file (default: the candidate
/// corpus compiled in); the corpus hash reported is the hash of the bytes this
/// process actually evaluated. `ENGRAM_CANDIDATE_SHA` and
/// `ENGRAM_CANDIDATE_FEATURES` are echoed so the supervisor can verify that the
/// process saw the candidate it asked for. Hard violations fail the test.
#[test]
fn candidate_retrieval_metrics() {
    let corpus_bytes = match std::env::var("ENGRAM_RETRIEVAL_CORPUS") {
        Ok(path) => std::fs::read(&path).unwrap_or_else(|e| panic!("read corpus {path}: {e}")),
        Err(_) => CANDIDATE_CORPUS_JSON.as_bytes().to_vec(),
    };
    let corpus: Corpus = serde_json::from_slice(&corpus_bytes).expect("valid corpus");
    assert!(
        corpus
            .schema_version
            .starts_with("engram.retrieval-corpus."),
        "unsupported corpus schema {}",
        corpus.schema_version
    );

    let mut modes = serde_json::Map::new();
    let mut failures = Vec::new();
    for mode in Mode::ALL {
        let report = evaluate_report(&corpus, mode);
        failures.extend(
            report
                .violations
                .iter()
                .map(|violation| format!("[{}] {violation}", mode.name())),
        );
        modes.insert(
            mode.name().to_owned(),
            json!({
                "metrics": report.metrics,
                "categories": category_breakdown(&report.outcomes),
                "discouraged": discouraged_hit_rate(&report.outcomes),
                "missed_queries": report.misses.len(),
            }),
        );
    }
    assert!(
        failures.is_empty(),
        "hard retrieval invariants violated:\n{}",
        failures.join("\n")
    );

    let embedder = create_embedder(&EmbeddingConfig::default()).expect("deterministic embedder");
    let line = json!({
        "schema": "engram.retrieval-candidate-eval.v1",
        "candidate_sha": std::env::var("ENGRAM_CANDIDATE_SHA").ok(),
        "features": std::env::var("ENGRAM_CANDIDATE_FEATURES").ok(),
        "corpus_sha256": sha256_hex(&corpus_bytes),
        "corpus_name": corpus.name,
        "corpus_version": corpus.version,
        "deterministic_seed": corpus.deterministic_seed,
        "memory_count": corpus.memories.len(),
        "query_count": corpus.queries.len(),
        "provider": {
            "embedder": embedder.model_name(),
            "dimensions": embedder.dimensions(),
            "network": false,
        },
        "label_provenance": corpus.labeling.as_ref().map(|l| json!({
            "authored_by": l.authored_by,
            "human_review": l.human_review,
            "review_status": l.review_status,
        })),
        "modes": modes,
    });
    println!("ENGRAM_RETRIEVAL_EVAL_JSON={line}");
}
