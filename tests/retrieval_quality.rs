//! Deterministic retrieval-quality evaluation over a frozen synthetic corpus.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::Arc;

use engram::embedding::{create_embedder, EmbeddingCache};
use engram::mcp::handlers::{self, HandlerContext};
use engram::search::{AdaptiveCacheConfig, FuzzyEngine, SearchConfig, SearchResultCache};
use engram::storage::queries::create_memory;
use engram::storage::Storage;
use engram::types::{CreateMemoryInput, EmbeddingConfig};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

const CORPUS_JSON: &str = include_str!("fixtures/retrieval_quality/corpus.json");
const BASELINE_JSON: &str = include_str!("fixtures/retrieval_quality/baseline.json");

#[derive(Clone, Deserialize)]
struct Corpus {
    schema_version: String,
    name: String,
    version: String,
    license: String,
    source: String,
    deterministic_seed: u64,
    memories: Vec<FixtureMemory>,
    queries: Vec<FixtureQuery>,
}

#[derive(Clone, Deserialize)]
struct FixtureMemory {
    key: String,
    workspace: String,
    content: String,
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

fn evaluate(corpus: &Corpus) -> Result<Metrics, Vec<String>> {
    let ctx = context();
    let mut ids = HashMap::new();

    for memory in &corpus.memories {
        let created = ctx
            .storage
            .with_transaction(|conn| {
                create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: memory.content.clone(),
                        workspace: Some(memory.workspace.clone()),
                        ..Default::default()
                    },
                )
            })
            .expect("fixture memory must be valid");
        ids.insert(created.id, memory.key.clone());
        ctx.fuzzy_engine.lock().add_to_vocabulary(&memory.content);
    }

    let mut errors = Vec::new();
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
        let response = handlers::dispatch(
            &ctx,
            "memory_search",
            json!({
                "query": effective_query,
                "workspace": query.workspace,
                "limit": 10,
                "rerank": false,
                "skip_cache": true
            }),
        );
        let results = result_array(&response).unwrap_or_else(|| {
            errors.push(format!("{}: search returned {response}", query.key));
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
                errors.push(format!(
                    "{}: forbidden workspace {workspace} leaked through result {key}",
                    query.key
                ));
            }
        }

        let relevant: HashSet<&str> = query.relevance.keys().map(String::as_str).collect();
        let found = ranked
            .iter()
            .take(10)
            .filter(|(key, _)| relevant.contains(key.as_str()))
            .count();
        if found != relevant.len() {
            errors.push(format!(
                "{}: retrieved {found}/{} relevant memories; ranking={ranked:?}",
                query.key,
                relevant.len()
            ));
        }
        recall_sum += found as f64 / relevant.len() as f64;

        if let Some(rank) = ranked
            .iter()
            .position(|(key, _)| relevant.contains(key.as_str()))
        {
            reciprocal_rank_sum += 1.0 / (rank + 1) as f64;
        }

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
        ndcg_sum += if ideal == 0.0 { 0.0 } else { dcg / ideal };
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    let count = corpus.queries.len() as f64;
    Ok(Metrics {
        recall_at_10: recall_sum / count,
        mrr: reciprocal_rank_sum / count,
        ndcg_at_10: ndcg_sum / count,
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
