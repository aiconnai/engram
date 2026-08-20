//! Performance benchmarks for the search stack (RML-902).
//!
//! Covers every layer of the retrieval pipeline in isolation and combined:
//! BM25 (FTS5), hybrid (BM25 + vector), TF-IDF embedding, fuzzy correction,
//! and scale tests up to 10 000 memories by default.
//!
//! Run with: `cargo bench --bench search`
//!
//! RFC 0003 scale package:
//! - `ENGRAM_SEARCH_BENCH_SCALE=medium cargo bench --bench search search_scale`
//!   adds 100K memories.
//! - `ENGRAM_SEARCH_BENCH_SCALE=large cargo bench --bench search search_scale`
//!   adds 100K and 1M memories.
//! - `ENGRAM_SEARCH_BENCH_REPORT=1 ENGRAM_SEARCH_BENCH_REPORT_SIZES=100000,1000000 \
//!   cargo bench --bench search search_index_v2_report` writes a Markdown report
//!   with quality, latency, rebuild time, delete lag, and disk growth.
//!
//! ## Performance targets
//! | Operation                     | Target   |
//! |-------------------------------|----------|
//! | `hybrid_search` (any variant) | < 10 ms  |
//! | `search_scale/10K memories`   | < 10 ms  |

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Once;
use std::time::{Duration, Instant};

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use engram::embedding::{cosine_similarity, Embedder, TfIdfEmbedder};
use engram::search::{bm25_search, hybrid_search, FuzzyEngine, SearchConfig};
use engram::storage::queries::*;
use engram::storage::Storage;
use engram::types::*;
use rusqlite::ffi::sqlite3_auto_extension;
use rusqlite::params;
use serde_json::json;
use sqlite_vec::sqlite3_vec_init;
use tempfile::{tempdir, TempDir};

static SEARCH_INDEX_V2_REPORT_ONCE: Once = Once::new();
static SQLITE_VEC_ONCE: Once = Once::new();

const DEFAULT_SCALE_SIZES: &[usize] = &[100, 1000, 10000];
const MEDIUM_SCALE_SIZES: &[usize] = &[100, 1000, 10000, 100000];
const LARGE_SCALE_SIZES: &[usize] = &[100, 1000, 10000, 100000, 1000000];
const REPORT_DEFAULT_SIZES: &[usize] = &[10000];
const REPORT_WORKSPACE: &str = "default";
const BENCH_TOPICS: [&str; 10] = [
    "Authentication using JWT tokens and refresh mechanism",
    "Database migration strategy for PostgreSQL",
    "React component lifecycle and hooks optimization",
    "API rate limiting implementation with Redis",
    "Docker container orchestration with Kubernetes",
    "GraphQL schema design best practices",
    "Microservices communication patterns",
    "CI/CD pipeline configuration with GitHub Actions",
    "Memory leak detection in Node.js applications",
    "Rust ownership and borrowing concepts",
];
const QUALITY_QUERIES: [(&str, usize); 5] = [
    ("authentication jwt refresh tokens", 0),
    ("postgres database migration strategy", 1),
    ("react hooks lifecycle optimization", 2),
    ("redis api rate limiting", 3),
    ("rust ownership borrowing", 9),
];

#[derive(Clone, Copy)]
struct BenchCorpusOptions {
    embeddings: bool,
    embedding_dimensions: usize,
    file_backed: bool,
    vec0: bool,
    filter_noise: bool,
}

impl Default for BenchCorpusOptions {
    fn default() -> Self {
        Self {
            embeddings: false,
            embedding_dimensions: 384,
            file_backed: false,
            vec0: false,
            filter_noise: false,
        }
    }
}

struct FileBackedCorpus {
    _tempdir: TempDir,
    storage: Storage,
    db_path: PathBuf,
    base_bytes: u64,
}

#[derive(Debug)]
struct QualityMetrics {
    precision_at_10: f64,
    mrr: f64,
    ndcg_at_10: f64,
}

#[derive(Debug)]
struct LatencyMetrics {
    keyword_p50_us: u128,
    keyword_p95_us: u128,
    hybrid_p50_us: u128,
    hybrid_p95_us: u128,
    semantic_p50_us: u128,
    semantic_p95_us: u128,
    vec0_ideal_p50_us: u128,
    vec0_ideal_p95_us: u128,
    vec0_postfilter_p50_us: u128,
    vec0_postfilter_p95_us: u128,
}

/// Postfilter completeness: how many of the requested `limit` results the
/// `vec0_postfilter` path actually returned after over-fetch + production
/// filters. Latency is only comparable to production when `underfilled_samples`
/// is 0 — a fast postfilter that returns fewer than `requested` is fast because
/// it did less work, not because it is efficient.
#[derive(Debug)]
struct CompletenessMetrics {
    requested: usize,
    returned_min: usize,
    returned_p50: usize,
    returned_p95: usize,
    underfilled_samples: usize,
}

#[derive(Debug)]
struct ReportRow {
    size: usize,
    quality: QualityMetrics,
    latency: LatencyMetrics,
    completeness: CompletenessMetrics,
    rebuild_ms: u128,
    fts_drift_after: i64,
    delete_lag_us: u128,
    delete_visibility_checks: usize,
    disk_base_bytes: u64,
    disk_loaded_bytes: u64,
    disk_after_rebuild_bytes: u64,
    disk_after_delete_bytes: u64,
}

/// Create an in-memory [`Storage`] pre-loaded with `count` memories.
///
/// Content is drawn from 10 realistic software-engineering topics, rotated
/// with a per-item suffix so FTS5 tokenises each row distinctly. Tags follow
/// a 5-bucket rotation (`topic0`…`topic4` + `development`) to simulate
/// realistic tag cardinality.
fn setup_storage_with_data(count: usize) -> Storage {
    setup_storage_with_options(count, BenchCorpusOptions::default())
}

fn setup_storage_with_embeddings(count: usize, dimensions: usize) -> Storage {
    setup_storage_with_options(
        count,
        BenchCorpusOptions {
            embeddings: true,
            embedding_dimensions: dimensions,
            file_backed: false,
            ..Default::default()
        },
    )
}

fn setup_storage_with_options(count: usize, options: BenchCorpusOptions) -> Storage {
    let storage = if options.file_backed {
        panic!("use setup_file_backed_corpus for file-backed benchmark corpora")
    } else {
        Storage::open_in_memory().unwrap()
    };

    load_bench_corpus(&storage, count, options);

    storage
}

fn setup_file_backed_corpus(count: usize, options: BenchCorpusOptions) -> FileBackedCorpus {
    if options.vec0 {
        register_sqlite_vec_once();
    }

    let tempdir = tempdir().unwrap();
    let db_path = tempdir.path().join("engram-search-bench.sqlite");
    let config = StorageConfig {
        db_path: db_path.to_string_lossy().to_string(),
        storage_mode: StorageMode::Local,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    };
    let storage = Storage::open(config).unwrap();
    if options.vec0 {
        validate_sqlite_vec_binding(&storage, options.embedding_dimensions);
        create_bench_vec0_index(&storage, options.embedding_dimensions);
    }
    let base_bytes = sqlite_file_bytes(&db_path);

    load_bench_corpus(
        &storage,
        count,
        BenchCorpusOptions {
            file_backed: true,
            ..options
        },
    );

    FileBackedCorpus {
        _tempdir: tempdir,
        storage,
        db_path,
        base_bytes,
    }
}

fn load_bench_corpus(storage: &Storage, count: usize, options: BenchCorpusOptions) {
    let embedder = TfIdfEmbedder::new(options.embedding_dimensions);
    let chunk_size = env_usize("ENGRAM_SEARCH_BENCH_LOAD_CHUNK", 5000);
    let mut start = 0;

    while start < count {
        let end = (start + chunk_size).min(count);
        storage
            .with_transaction(|conn| {
                for i in start..end {
                    let topic = i % BENCH_TOPICS.len();
                    let content = bench_content(i);
                    let workspace = bench_workspace(i, options.filter_noise);
                    let input = CreateMemoryInput {
                        content,
                        memory_type: MemoryType::Note,
                        tags: vec![
                            format!("topic{}", i % 5),
                            format!("bench-topic-{}", topic),
                            "development".to_string(),
                        ],
                        metadata: [("bench_topic".to_string(), json!(topic))]
                            .into_iter()
                            .collect(),
                        importance: Some((i % 10) as f32 / 10.0),
                        defer_embedding: true,
                        scope: MemoryScope::Global,
                        ttl_seconds: None,
                        dedup_mode: DedupMode::Allow,
                        dedup_threshold: None,
                        workspace: Some(workspace.to_string()),
                        tier: MemoryTier::Permanent,
                        event_time: None,
                        event_duration_seconds: None,
                        trigger_pattern: None,
                        summary_of_id: None,
                        media_url: None,
                    };
                    let memory = create_memory(conn, &input)?;
                    apply_bench_filter_noise(conn, memory.id, i, options.filter_noise)?;
                    if options.embeddings {
                        let embedding = embedder.embed(&memory.content)?;
                        store_bench_embedding(conn, memory.id, &embedding)?;
                        if options.vec0 {
                            store_bench_vec0_embedding(conn, memory.id, &embedding)?;
                        }
                    }
                }
                Ok(())
            })
            .unwrap();
        start = end;
    }
}

fn bench_content(i: usize) -> String {
    format!(
        "{} - variation {} benchunique{} with additional context about software development",
        BENCH_TOPICS[i % BENCH_TOPICS.len()],
        i,
        i
    )
}

fn bench_workspace(i: usize, filter_noise: bool) -> &'static str {
    if filter_noise && (i / BENCH_TOPICS.len()) % 2 == 1 {
        "other"
    } else {
        REPORT_WORKSPACE
    }
}

fn bench_is_transcript_noise(i: usize) -> bool {
    i.is_multiple_of(20)
}

fn bench_is_archived_noise(i: usize) -> bool {
    i % 20 == 3
}

fn bench_row_visible_in_report_search(i: usize, filter_noise: bool) -> bool {
    !filter_noise
        || (bench_workspace(i, true) == REPORT_WORKSPACE
            && !bench_is_transcript_noise(i)
            && !bench_is_archived_noise(i))
}

fn apply_bench_filter_noise(
    conn: &rusqlite::Connection,
    memory_id: MemoryId,
    row_index: usize,
    filter_noise: bool,
) -> engram::error::Result<()> {
    if !filter_noise {
        return Ok(());
    }

    if bench_is_transcript_noise(row_index) {
        conn.execute(
            "UPDATE memories SET memory_type = 'transcript_chunk' WHERE id = ?1",
            params![memory_id],
        )?;
    }
    if bench_is_archived_noise(row_index) {
        conn.execute(
            "UPDATE memories SET lifecycle_state = 'archived' WHERE id = ?1",
            params![memory_id],
        )?;
    }

    Ok(())
}

fn f32_blob(embedding: &[f32]) -> Vec<u8> {
    embedding.iter().flat_map(|f| f.to_le_bytes()).collect()
}

fn store_bench_embedding(
    conn: &rusqlite::Connection,
    memory_id: MemoryId,
    embedding: &[f32],
) -> engram::error::Result<()> {
    let bytes = f32_blob(embedding);
    conn.execute(
        "INSERT OR REPLACE INTO embeddings (memory_id, embedding, model, dimensions)
         VALUES (?1, ?2, 'bench-tfidf', ?3)",
        params![memory_id, bytes, embedding.len() as i64],
    )?;
    conn.execute(
        "UPDATE memories SET has_embedding = 1 WHERE id = ?1",
        params![memory_id],
    )?;
    Ok(())
}

fn register_sqlite_vec_once() {
    SQLITE_VEC_ONCE.call_once(|| unsafe {
        // Canonical sqlite-vec registration, mirrored verbatim from
        // sqlite-vec-0.1.6/src/lib.rs. The fn-pointer transmute target shape is
        // an internal libsqlite3-sys detail that shifts between versions, so we
        // suppress the pedantic annotation lint rather than pin a brittle type.
        #[allow(clippy::missing_transmute_annotations)]
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    });
}

fn validate_sqlite_vec_binding(storage: &Storage, dimensions: usize) {
    storage
        .with_connection(|conn| {
            let version: String = conn.query_row("SELECT vec_version()", [], |row| row.get(0))?;
            assert!(
                version.starts_with('v'),
                "sqlite-vec vec_version() returned unexpected value: {version}"
            );

            conn.execute("DROP TABLE IF EXISTS bench_vec_binding_gate", [])?;
            conn.execute(
                "CREATE VIRTUAL TABLE bench_vec_binding_gate USING vec0(embedding float[3] distance_metric=cosine)",
                [],
            )?;

            let vectors = [
                (1_i64, vec![1.0_f32, 0.0, 0.0]),
                (2_i64, vec![0.95_f32, 0.05, 0.0]),
                (3_i64, vec![0.0_f32, 1.0, 0.0]),
            ];
            for (id, vector) in &vectors {
                conn.execute(
                    "INSERT INTO bench_vec_binding_gate(rowid, embedding) VALUES (?1, ?2)",
                    params![id, f32_blob(vector)],
                )?;
            }

            let query = vec![0.90_f32, 0.10, 0.0];
            let expected_id = vectors
                .iter()
                .max_by(|(_, left), (_, right)| {
                    cosine_similarity(&query, left)
                        .partial_cmp(&cosine_similarity(&query, right))
                        .unwrap()
                })
                .map(|(id, _)| *id)
                .unwrap();
            let actual_id: i64 = conn.query_row(
                "SELECT rowid FROM bench_vec_binding_gate
                 WHERE embedding MATCH ?1 AND k = 1
                 ORDER BY distance",
                params![f32_blob(&query)],
                |row| row.get(0),
            )?;

            assert_eq!(
                actual_id, expected_id,
                "sqlite-vec binding gate failed: nearest neighbor disagrees with production cosine_similarity"
            );

            conn.execute("DROP TABLE bench_vec_binding_gate", [])?;
            let _ = dimensions;
            Ok(())
        })
        .unwrap();
}

fn create_bench_vec0_index(storage: &Storage, dimensions: usize) {
    let dim = dimensions;
    storage
        .with_connection(|conn| {
            conn.execute("DROP TABLE IF EXISTS bench_vec", [])?;
            let ddl = format!(
                "CREATE VIRTUAL TABLE bench_vec USING vec0(embedding float[{}] distance_metric=cosine)",
                dim
            );
            conn.execute(&ddl, [])?;
            Ok(())
        })
        .unwrap();
}

fn store_bench_vec0_embedding(
    conn: &rusqlite::Connection,
    memory_id: MemoryId,
    embedding: &[f32],
) -> engram::error::Result<()> {
    conn.execute(
        "INSERT INTO bench_vec(rowid, embedding) VALUES (?1, ?2)",
        params![memory_id, f32_blob(embedding)],
    )?;
    Ok(())
}

fn search_scale_sizes() -> Vec<usize> {
    match env::var("ENGRAM_SEARCH_BENCH_SCALE")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "medium" => MEDIUM_SCALE_SIZES.to_vec(),
        "large" | "full" => LARGE_SCALE_SIZES.to_vec(),
        _ => DEFAULT_SCALE_SIZES.to_vec(),
    }
}

fn report_sizes() -> Vec<usize> {
    if let Ok(raw) = env::var("ENGRAM_SEARCH_BENCH_REPORT_SIZES") {
        let sizes: Vec<usize> = raw
            .split(',')
            .filter_map(|part| part.trim().parse::<usize>().ok())
            .collect();
        if !sizes.is_empty() {
            return sizes;
        }
    }

    match env::var("ENGRAM_SEARCH_BENCH_SCALE")
        .unwrap_or_default()
        .to_lowercase()
        .as_str()
    {
        "medium" => vec![100000],
        "large" | "full" => vec![100000, 1000000],
        _ => REPORT_DEFAULT_SIZES.to_vec(),
    }
}

fn scale_embedding_dimensions() -> usize {
    env_usize("ENGRAM_SEARCH_BENCH_EMBEDDING_DIMS", 64)
}

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default)
}

fn maybe_write_search_index_v2_report() {
    SEARCH_INDEX_V2_REPORT_ONCE.call_once(|| {
        if env::var("ENGRAM_SEARCH_BENCH_REPORT").as_deref() != Ok("1") {
            return;
        }
        let rows = run_search_index_v2_report();
        let report = format_search_index_v2_report(&rows);
        let report_path = Path::new("target")
            .join("criterion")
            .join("search-index-v2")
            .join("report.md");
        if let Some(parent) = report_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&report_path, report).unwrap();
        eprintln!(
            "search-index-v2 benchmark report written to {}",
            report_path.display()
        );
    });
}

fn run_search_index_v2_report() -> Vec<ReportRow> {
    let dimensions = scale_embedding_dimensions();
    report_sizes()
        .into_iter()
        .map(|size| {
            let corpus = setup_file_backed_corpus(
                size,
                BenchCorpusOptions {
                    embeddings: true,
                    embedding_dimensions: dimensions,
                    file_backed: true,
                    vec0: true,
                    filter_noise: true,
                },
            );
            let loaded_bytes = sqlite_file_bytes(&corpus.db_path);
            let quality = measure_quality(&corpus.storage, dimensions, size, true);
            let (latency, completeness) = measure_latency(&corpus.storage, dimensions);

            let rebuild_start = Instant::now();
            let rebuild_report = corpus
                .storage
                .with_transaction(|conn| rebuild_derived_indexes(conn, true, false, true))
                .unwrap();
            let rebuild_ms = rebuild_start.elapsed().as_millis();
            let after_rebuild_bytes = sqlite_file_bytes(&corpus.db_path);

            let (delete_lag_us, delete_visibility_checks) =
                measure_delete_visibility_lag(&corpus.storage, size / 2);
            let after_delete_bytes = sqlite_file_bytes(&corpus.db_path);

            ReportRow {
                size,
                quality,
                latency,
                completeness,
                rebuild_ms,
                fts_drift_after: rebuild_report.fts_drift_after,
                delete_lag_us,
                delete_visibility_checks,
                disk_base_bytes: corpus.base_bytes,
                disk_loaded_bytes: loaded_bytes,
                disk_after_rebuild_bytes: after_rebuild_bytes,
                disk_after_delete_bytes: after_delete_bytes,
            }
        })
        .collect()
}

fn measure_quality(
    storage: &Storage,
    dimensions: usize,
    corpus_size: usize,
    filter_noise: bool,
) -> QualityMetrics {
    let embedder = TfIdfEmbedder::new(dimensions);
    let config = SearchConfig::default();
    let mut precision_total = 0.0;
    let mut mrr_total = 0.0;
    let mut ndcg_total = 0.0;

    for (query, expected_topic) in QUALITY_QUERIES {
        let query_embedding = embedder.embed(query).unwrap();
        let results = storage
            .with_connection(|conn| {
                hybrid_search(
                    conn,
                    query,
                    Some(query_embedding.as_slice()),
                    &report_search_options(SearchStrategy::Hybrid),
                    &config,
                )
            })
            .unwrap();

        let relevance: Vec<bool> = results
            .iter()
            .take(10)
            .map(|result| memory_topic(&result.memory) == Some(expected_topic))
            .collect();
        precision_total +=
            relevance.iter().filter(|is_relevant| **is_relevant).count() as f64 / 10.0;
        mrr_total += relevance
            .iter()
            .position(|is_relevant| *is_relevant)
            .map(|rank| 1.0 / (rank as f64 + 1.0))
            .unwrap_or(0.0);
        ndcg_total += ndcg_at_10(
            &relevance,
            expected_topic_count(corpus_size, expected_topic, filter_noise),
        );
    }

    let query_count = QUALITY_QUERIES.len() as f64;
    QualityMetrics {
        precision_at_10: precision_total / query_count,
        mrr: mrr_total / query_count,
        ndcg_at_10: ndcg_total / query_count,
    }
}

fn memory_topic(memory: &Memory) -> Option<usize> {
    memory
        .metadata
        .get("bench_topic")
        .and_then(|value| value.as_u64())
        .map(|value| value as usize)
}

fn expected_topic_count(corpus_size: usize, topic: usize, filter_noise: bool) -> usize {
    (0..corpus_size)
        .filter(|i| {
            i % BENCH_TOPICS.len() == topic && bench_row_visible_in_report_search(*i, filter_noise)
        })
        .count()
}

fn ndcg_at_10(relevance: &[bool], relevant_total: usize) -> f64 {
    let dcg = relevance
        .iter()
        .take(10)
        .enumerate()
        .filter_map(|(rank, is_relevant)| is_relevant.then_some(1.0 / ((rank + 2) as f64).log2()))
        .sum::<f64>();
    let ideal_len = relevant_total.min(10);
    let idcg = (0..ideal_len)
        .map(|rank| 1.0 / ((rank + 2) as f64).log2())
        .sum::<f64>();

    if idcg == 0.0 {
        0.0
    } else {
        dcg / idcg
    }
}

fn measure_latency(storage: &Storage, dimensions: usize) -> (LatencyMetrics, CompletenessMetrics) {
    let iterations = env_usize("ENGRAM_SEARCH_BENCH_REPORT_ITERS", 10);
    let vec0_overfetch = env_usize("ENGRAM_SEARCH_BENCH_VEC0_OVERFETCH", 10).max(1);
    let requested = 10usize;
    let embedder = TfIdfEmbedder::new(dimensions);
    let config = SearchConfig::default();
    let query = "authentication jwt refresh tokens";
    let query_embedding = embedder.embed(query).unwrap();
    let query_blob = f32_blob(&query_embedding);
    let keyword_options = report_search_options(SearchStrategy::KeywordOnly);
    let hybrid_options = report_search_options(SearchStrategy::Hybrid);
    let semantic_options = report_search_options(SearchStrategy::SemanticOnly);

    let keyword = measure_operation(iterations, || {
        storage
            .with_connection(|conn| {
                hybrid_search(conn, query, None, &keyword_options, &config).map(|_| ())
            })
            .unwrap();
    });
    let hybrid = measure_operation(iterations, || {
        storage
            .with_connection(|conn| {
                hybrid_search(
                    conn,
                    query,
                    Some(query_embedding.as_slice()),
                    &hybrid_options,
                    &config,
                )
                .map(|_| ())
            })
            .unwrap();
    });
    let semantic = measure_operation(iterations, || {
        storage
            .with_connection(|conn| {
                hybrid_search(
                    conn,
                    query,
                    Some(query_embedding.as_slice()),
                    &semantic_options,
                    &config,
                )
                .map(|_| ())
            })
            .unwrap();
    });
    let vec0_ideal = measure_operation(iterations, || {
        storage
            .with_connection(|conn| {
                vec0_ideal_search(conn, query_blob.as_slice(), 10)?;
                Ok(())
            })
            .unwrap();
    });
    let vec0_postfilter = measure_operation(iterations, || {
        storage
            .with_connection(|conn| {
                vec0_postfilter_search(conn, query_blob.as_slice(), requested, vec0_overfetch)?;
                Ok(())
            })
            .unwrap();
    });

    // Completeness is measured outside the timing loop so it does not pollute the
    // postfilter latency samples. Same query, same over-fetch: count how many of
    // the requested results survive the production-style filters per iteration.
    let mut returned: Vec<usize> = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let count = storage
            .with_connection(|conn| -> engram::error::Result<usize> {
                let results =
                    vec0_postfilter_search(conn, query_blob.as_slice(), requested, vec0_overfetch)?;
                Ok(results.len())
            })
            .unwrap();
        returned.push(count);
    }
    let completeness = CompletenessMetrics {
        requested,
        returned_min: returned.iter().copied().min().unwrap_or(0),
        returned_p50: percentile_usize(&returned, 0.50),
        returned_p95: percentile_usize(&returned, 0.95),
        underfilled_samples: returned.iter().filter(|count| **count < requested).count(),
    };

    let latency = LatencyMetrics {
        keyword_p50_us: percentile_us(&keyword, 0.50),
        keyword_p95_us: percentile_us(&keyword, 0.95),
        hybrid_p50_us: percentile_us(&hybrid, 0.50),
        hybrid_p95_us: percentile_us(&hybrid, 0.95),
        semantic_p50_us: percentile_us(&semantic, 0.50),
        semantic_p95_us: percentile_us(&semantic, 0.95),
        vec0_ideal_p50_us: percentile_us(&vec0_ideal, 0.50),
        vec0_ideal_p95_us: percentile_us(&vec0_ideal, 0.95),
        vec0_postfilter_p50_us: percentile_us(&vec0_postfilter, 0.50),
        vec0_postfilter_p95_us: percentile_us(&vec0_postfilter, 0.95),
    };

    (latency, completeness)
}

fn report_search_options(strategy: SearchStrategy) -> SearchOptions {
    SearchOptions {
        limit: Some(10),
        strategy: Some(strategy),
        workspace: Some(REPORT_WORKSPACE.to_string()),
        ..Default::default()
    }
}

fn vec0_ideal_search(
    conn: &rusqlite::Connection,
    query_blob: &[u8],
    limit: usize,
) -> rusqlite::Result<Vec<(MemoryId, f64)>> {
    let mut stmt = conn.prepare(
        "SELECT rowid, distance FROM bench_vec
         WHERE embedding MATCH ?1 AND k = ?2
         ORDER BY distance",
    )?;
    let rows = stmt.query_map(params![query_blob, limit as i64], |row| {
        Ok((row.get::<_, MemoryId>(0)?, row.get::<_, f64>(1)?))
    })?;
    rows.collect()
}

fn vec0_postfilter_search(
    conn: &rusqlite::Connection,
    query_blob: &[u8],
    limit: usize,
    overfetch_multiplier: usize,
) -> rusqlite::Result<Vec<(MemoryId, f64)>> {
    let fetch_limit = limit.saturating_mul(overfetch_multiplier.max(1));
    let mut stmt = conn.prepare(
        "SELECT v.memory_id, v.distance, m.memory_type, m.lifecycle_state, m.valid_to,
                (m.expires_at IS NULL OR m.expires_at > datetime('now')) AS expires_visible,
                m.workspace
         FROM (
             SELECT rowid AS memory_id, distance FROM bench_vec
             WHERE embedding MATCH ?1 AND k = ?2
             ORDER BY distance
         ) AS v
         JOIN memories AS m ON m.id = v.memory_id
         ORDER BY v.distance",
    )?;
    let rows = stmt.query_map(params![query_blob, fetch_limit as i64], |row| {
        let memory_id: MemoryId = row.get(0)?;
        let distance: f64 = row.get(1)?;
        let memory_type: String = row.get(2)?;
        let lifecycle_state: Option<String> = row.get(3)?;
        let valid_to: Option<String> = row.get(4)?;
        let expires_visible: bool = row.get(5)?;
        let workspace: String = row.get(6)?;
        Ok((
            memory_id,
            distance,
            memory_type,
            lifecycle_state,
            valid_to,
            expires_visible,
            workspace,
        ))
    })?;

    let mut filtered = Vec::with_capacity(limit);
    for row in rows {
        let (
            memory_id,
            distance,
            memory_type,
            lifecycle_state,
            valid_to,
            expires_visible,
            workspace,
        ) = row?;
        let visible = valid_to.is_none()
            && expires_visible
            && memory_type != "transcript_chunk"
            && lifecycle_state.as_deref() != Some("archived")
            && workspace == REPORT_WORKSPACE;
        if visible {
            filtered.push((memory_id, distance));
            if filtered.len() == limit {
                break;
            }
        }
    }

    Ok(filtered)
}

fn measure_operation<F>(iterations: usize, mut operation: F) -> Vec<Duration>
where
    F: FnMut(),
{
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let started = Instant::now();
        operation();
        samples.push(started.elapsed());
    }
    samples
}

fn percentile_us(samples: &[Duration], percentile: f64) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort();
    let index = ((sorted.len().saturating_sub(1)) as f64 * percentile).round() as usize;
    sorted[index].as_micros()
}

fn percentile_usize(samples: &[usize], percentile: f64) -> usize {
    if samples.is_empty() {
        return 0;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let index = ((sorted.len().saturating_sub(1)) as f64 * percentile).round() as usize;
    sorted[index]
}

fn measure_delete_visibility_lag(storage: &Storage, offset: usize) -> (u128, usize) {
    let offset_val = offset as i64;
    let (target_id, query) = storage
        .with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id FROM memories WHERE valid_to IS NULL ORDER BY id LIMIT 1 OFFSET ?1",
            )?;
            let target_id: i64 = stmt.query_row([offset_val], |row| row.get(0))?;
            Ok((target_id, format!("benchunique{}", target_id - 1)))
        })
        .unwrap();

    storage
        .with_transaction(|conn| delete_memory(conn, target_id))
        .unwrap();

    let started = Instant::now();
    for checks in 1..=100 {
        let still_visible = storage
            .with_connection(|conn| {
                bm25_search(conn, &query, 10, false)
                    .map(|results| results.iter().any(|result| result.memory.id == target_id))
            })
            .unwrap();
        if !still_visible {
            return (started.elapsed().as_micros(), checks);
        }
    }

    (started.elapsed().as_micros(), 100)
}

fn sqlite_file_bytes(db_path: &Path) -> u64 {
    let mut total = file_len(db_path);
    total += file_len(&PathBuf::from(format!("{}-wal", db_path.display())));
    total += file_len(&PathBuf::from(format!("{}-shm", db_path.display())));
    total
}

fn file_len(path: &Path) -> u64 {
    fs::metadata(path)
        .map(|metadata| metadata.len())
        .unwrap_or(0)
}

fn format_search_index_v2_report(rows: &[ReportRow]) -> String {
    let embedding_dimensions = scale_embedding_dimensions();
    let latency_iterations = env_usize("ENGRAM_SEARCH_BENCH_REPORT_ITERS", 10);
    let vec0_overfetch = env_usize("ENGRAM_SEARCH_BENCH_VEC0_OVERFETCH", 10).max(1);
    let mut report = String::from(
        "# Search Index v2 Benchmark Report\n\n\
         This report measures the current SQLite + FTS5 + manual cosine baseline required by RFC 0003 before adopting Tantivy, ANN/HNSW, or another derived index. It also includes a bench-only `sqlite-vec` `vec0` spike to separate brute-force vector constant-factor gains from scaling-class changes.\n\n\
         ## Corpus\n\n\
         Synthetic memories rotate across 10 software-engineering topics. Embeddings use local TF-IDF and are stored in the existing `embeddings` table so `semantic_only_search` exercises the current O(n) cosine path. Report corpora also create a disposable `bench_vec` `vec0` table with `rowid = memories.id`; every other topic cycle is routed to another workspace, and a small deterministic subset is marked as `transcript_chunk` or `archived` so `vec0_postfilter` measures KNN-first then production-style filtering.\n\n",
    );

    report.push_str(&format!(
        "Embedding dimensions: `{embedding_dimensions}`. Latency samples per mode: `{latency_iterations}`. Report workspace filter: `{REPORT_WORKSPACE}`. `vec0_postfilter` over-fetch multiplier: `{vec0_overfetch}`.\n\n"
    ));

    let mut warnings = Vec::new();
    if embedding_dimensions < 384 {
        warnings.push(format!(
            "`ENGRAM_SEARCH_BENCH_EMBEDDING_DIMS={embedding_dimensions}` underestimates production-like semantic scan cost; use at least `384` for architecture decisions."
        ));
    }
    if latency_iterations < 50 {
        warnings.push(format!(
            "`ENGRAM_SEARCH_BENCH_REPORT_ITERS={latency_iterations}` is a smoke-test sample; use at least `100` before treating p95 as decision-grade."
        ));
    }
    if !warnings.is_empty() {
        report.push_str("## Warnings\n\n");
        for warning in warnings {
            report.push_str("- ");
            report.push_str(&warning);
            report.push('\n');
        }
        report.push('\n');
    }

    report.push_str(
        "## Results\n\n\
         | memories | precision@10 | mrr | ndcg@10 | keyword p50/p95 us | hybrid p50/p95 us | manual_cosine p50/p95 us | vec0_ideal p50/p95 us | vec0_postfilter p50/p95 us | postfilter returned@N (min/p50/p95 of N) | postfilter underfilled | rebuild ms | fts drift after | delete lag us | disk loaded | disk after rebuild | disk after delete |\n\
         |---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n",
    );

    for row in rows {
        report.push_str(&format!(
            "| {} | {:.3} | {:.3} | {:.3} | {}/{} | {}/{} | {}/{} | {}/{} | {}/{} | {}/{}/{} of {} | {}/{} | {} | {} | {} ({} check{}) | {} | {} | {} |\n",
            row.size,
            row.quality.precision_at_10,
            row.quality.mrr,
            row.quality.ndcg_at_10,
            row.latency.keyword_p50_us,
            row.latency.keyword_p95_us,
            row.latency.hybrid_p50_us,
            row.latency.hybrid_p95_us,
            row.latency.semantic_p50_us,
            row.latency.semantic_p95_us,
            row.latency.vec0_ideal_p50_us,
            row.latency.vec0_ideal_p95_us,
            row.latency.vec0_postfilter_p50_us,
            row.latency.vec0_postfilter_p95_us,
            row.completeness.returned_min,
            row.completeness.returned_p50,
            row.completeness.returned_p95,
            row.completeness.requested,
            row.completeness.underfilled_samples,
            latency_iterations,
            row.rebuild_ms,
            row.fts_drift_after,
            row.delete_lag_us,
            row.delete_visibility_checks,
            if row.delete_visibility_checks == 1 { "" } else { "s" },
            bytes(row.disk_loaded_bytes),
            bytes(row.disk_after_rebuild_bytes),
            bytes(row.disk_after_delete_bytes),
        ));
    }

    report.push_str(
        "\n## Interpretation\n\n\
         - `keyword` isolates the SQLite FTS5/BM25 path.\n\
         - `manual_cosine` isolates the current manual cosine scan path.\n\
         - `vec0_ideal` runs `sqlite-vec` KNN without production filters; it measures the best-case constant-factor gain only.\n\
         - `vec0_postfilter` over-fetches with `vec0`, applies the same default report filters (`valid_to`, `expires_at`, `transcript_chunk`, `archived`, workspace), and truncates to 10 results; this is the acceptance line for the spike.\n\
         - `postfilter returned@N` reports how many of the requested results the postfilter actually returned per iteration (min/p50/p95). `postfilter underfilled` counts iterations that returned fewer than requested. The postfilter latency is only comparable to production when `underfilled` is 0 — a fast line that returns fewer results is fast because it did less work, not because it is efficient. Underfilled rows mean the over-fetch multiplier is too small for the filter selectivity and the latency number is optimistic.\n\
         - `sqlite-vec` 0.1.x `vec0` remains brute-force O(n); a faster `vec0` line is not enough unless projected 1M p95 stays inside the 10 ms target.\n\
         - `hybrid` includes BM25 plus semantic scan and RRF fusion.\n\
         - Quality metrics use deterministic TF-IDF embeddings; they validate ranking plumbing and synthetic-topic relevance, but they should not be read as production semantic retrieval quality for OpenAI, ONNX, Cohere, or similar embedding providers.\n\
         - `delete lag` is search-visibility lag for soft deletes; physical derived-index cleanup remains a separate rebuild/maintenance concern.\n\
         - FTS5 delete propagation is deterministic by construction because it is maintained transactionally by SQLite triggers; ANN/HNSW-style indexes must prove this axis separately.\n\
         - `disk loaded` includes the SQLite database, WAL, and SHM files. Base migrated database size is excluded from the table but was measured as ",
    );
    if let Some(first) = rows.first() {
        report.push_str(&bytes(first.disk_base_bytes));
    } else {
        report.push_str("0 B");
    }
    report.push_str(".\n");

    report
}

fn bytes(value: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB"];
    let mut size = value as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit + 1 < UNITS.len() {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", value, UNITS[unit])
    } else {
        format!("{:.2} {}", size, UNITS[unit])
    }
}

/// Benchmark BM25 keyword search over 1 000 memories.
///
/// Exercises the SQLite FTS5 `MATCH` path only — no vector similarity.
/// Four representative queries test term frequency variance: single-token,
/// two-token, three-token, and a four-token phrase with a Redis-specific term.
fn bench_bm25_search(c: &mut Criterion) {
    let storage = setup_storage_with_data(1000);

    let mut group = c.benchmark_group("bm25_search");

    let queries = vec![
        "authentication",
        "database migration",
        "React hooks optimization",
        "API rate limiting Redis",
    ];

    for query in queries {
        group.bench_with_input(BenchmarkId::new("query", query), &query, |b, query| {
            b.iter(|| {
                storage
                    .with_connection(|conn| bm25_search(conn, black_box(query), 10, false))
                    .unwrap()
            })
        });
    }

    group.finish();
}

/// Benchmark hybrid search (BM25 + TF-IDF vector + RRF fusion) over 1 000 memories.
///
/// The query embedding is computed once outside the hot loop so only the
/// retrieval and fusion cost is measured. Three query lengths stress different
/// BM25 term-match densities while the vector component stays constant in
/// dimension (384-d TF-IDF):
/// - **`short`** — single abbreviated token (`auth`).
/// - **`medium`** — three tokens matching a common topic.
/// - **`long`** — full sentence, many tokens, maximum BM25 overlap.
fn bench_hybrid_search(c: &mut Criterion) {
    let storage = setup_storage_with_embeddings(1000, 384);
    let embedder = TfIdfEmbedder::new(384);
    let config = SearchConfig::default();

    let mut group = c.benchmark_group("hybrid_search");

    let queries = vec![
        ("short", "auth"),
        ("medium", "database migration strategy"),
        (
            "long",
            "how to implement authentication with JWT tokens and refresh mechanism",
        ),
    ];

    for (name, query) in queries {
        let query_embedding = embedder.embed(query).unwrap();

        group.bench_with_input(
            BenchmarkId::new("query_type", name),
            &(query, &query_embedding),
            |b, (query, embedding)| {
                b.iter(|| {
                    let options = SearchOptions {
                        limit: Some(10),
                        ..Default::default()
                    };
                    storage
                        .with_connection(|conn| {
                            hybrid_search(
                                conn,
                                black_box(query),
                                Some(embedding.as_slice()),
                                &options,
                                &config,
                            )
                        })
                        .unwrap()
                })
            },
        );
    }

    group.finish();
}

/// Benchmark TF-IDF embedding throughput at three text lengths and in batch.
///
/// - **`short`** — two tokens; isolates tokeniser overhead.
/// - **`medium`** — nine tokens; typical query length.
/// - **`long`** — 27 tokens; realistic document sentence.
/// - **`batch_100`** — 100 mixed-length strings via `embed_batch`; measures
///   amortised cost and any batch-path allocation savings.
fn bench_tfidf_embedding(c: &mut Criterion) {
    let embedder = TfIdfEmbedder::new(384);

    let mut group = c.benchmark_group("tfidf_embedding");

    let texts = vec![
        ("short", "hello world"),
        ("medium", "The quick brown fox jumps over the lazy dog"),
        ("long", "Authentication using JWT tokens requires careful consideration of security best practices including token expiration, refresh token rotation, and secure storage mechanisms"),
    ];

    for (name, text) in texts {
        group.bench_with_input(BenchmarkId::new("text_length", name), &text, |b, text| {
            b.iter(|| embedder.embed(black_box(text)).unwrap())
        });
    }

    // Batch embedding
    let batch: Vec<&str> = (0..100)
        .map(|i| {
            if i % 3 == 0 {
                "Short text"
            } else if i % 3 == 1 {
                "Medium length text with more content"
            } else {
                "Longer text with significantly more content to process and embed into vector space"
            }
        })
        .collect();

    group.bench_function("batch_100", |b| {
        b.iter(|| embedder.embed_batch(black_box(&batch)).unwrap())
    });

    group.finish();
}

/// Benchmark the fuzzy spelling-correction engine at three error levels.
///
/// Builds a 12-word vocabulary (each repeated 10× to simulate realistic
/// term frequency), then runs `correct_query` for:
/// - **`1_char_typo`** — one missing character (`authentcation`).
/// - **`2_char_typo`** — two missing characters (`authentcatin`).
/// - **`transposition`** — two transpositions (`authetnicaiton`).
///
/// The engine uses BK-tree / edit-distance lookup; this benchmark tracks
/// regression in worst-case correction cost.
fn bench_fuzzy_search(c: &mut Criterion) {
    let mut engine = FuzzyEngine::new();

    // Build vocabulary
    let words = vec![
        "authentication",
        "authorization",
        "configuration",
        "implementation",
        "documentation",
        "optimization",
        "synchronization",
        "initialization",
        "serialization",
        "deserialization",
        "transformation",
        "compilation",
    ];

    for word in &words {
        for _ in 0..10 {
            engine.add_to_vocabulary(word);
        }
    }

    let mut group = c.benchmark_group("fuzzy_search");

    let typos = vec![
        ("1_char_typo", "authentcation"),
        ("2_char_typo", "authentcatin"),
        ("transposition", "authetnicaiton"),
    ];

    for (name, query) in typos {
        group.bench_with_input(BenchmarkId::new("typo_type", name), &query, |b, query| {
            b.iter(|| engine.correct_query(black_box(query)))
        });
    }

    group.finish();
}

/// Benchmark hybrid search scalability at 100, 1 000, and 10 000 memories.
///
/// Uses a fixed query (`"authentication JWT tokens"`) with a pre-computed
/// embedding so only the retrieval and RRF fusion scale with corpus size.
/// Throughput is expressed as elements processed (corpus size), not results
/// returned, so the chart shows sub-linear scaling of the SQLite FTS5 + vec
/// combined plan.
///
/// Sample size is reduced to 50 for the 10 K case to keep CI run time
/// reasonable while still producing statistically stable estimates.
fn bench_search_at_scale(c: &mut Criterion) {
    maybe_write_search_index_v2_report();

    let sizes = search_scale_sizes();
    let dimensions = scale_embedding_dimensions();
    let mut group = c.benchmark_group("search_scale");
    group.sample_size(if sizes.iter().any(|size| *size > 10000) {
        10
    } else {
        50
    }); // Fewer samples for slow benchmarks

    for size in sizes {
        let storage = setup_storage_with_embeddings(size, dimensions);
        let embedder = TfIdfEmbedder::new(dimensions);
        let config = SearchConfig::default();
        let query = "authentication JWT tokens";
        let query_embedding = embedder.embed(query).unwrap();

        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(
            BenchmarkId::new("hybrid_memories", size),
            &(query.to_string(), query_embedding.clone()),
            |b, (query, embedding): &(String, Vec<f32>)| {
                b.iter(|| {
                    let options = SearchOptions {
                        limit: Some(10),
                        ..Default::default()
                    };
                    storage
                        .with_connection(|conn| {
                            hybrid_search(
                                conn,
                                black_box(query.as_str()),
                                Some(embedding.as_slice()),
                                &options,
                                &config,
                            )
                        })
                        .unwrap()
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("semantic_only_memories", size),
            &(query.to_string(), query_embedding.clone()),
            |b, (query, embedding): &(String, Vec<f32>)| {
                b.iter(|| {
                    let options = SearchOptions {
                        limit: Some(10),
                        strategy: Some(SearchStrategy::SemanticOnly),
                        ..Default::default()
                    };
                    storage
                        .with_connection(|conn| {
                            hybrid_search(
                                conn,
                                black_box(query.as_str()),
                                Some(embedding.as_slice()),
                                &options,
                                &config,
                            )
                        })
                        .unwrap()
                })
            },
        );
    }

    group.finish();
}

/// Opt-in RFC 0003 report hook.
///
/// Criterion still needs a benchmark function to select, but the expensive
/// report is generated once by `maybe_write_search_index_v2_report()` when
/// `ENGRAM_SEARCH_BENCH_REPORT=1` is present.
fn bench_search_index_v2_report(c: &mut Criterion) {
    maybe_write_search_index_v2_report();
    c.bench_function("search_index_v2_report/noop", |b| b.iter(|| black_box(())));
}

criterion_group!(
    benches,
    bench_bm25_search,
    bench_hybrid_search,
    bench_tfidf_embedding,
    bench_fuzzy_search,
    bench_search_at_scale,
    bench_search_index_v2_report,
);

criterion_main!(benches);
