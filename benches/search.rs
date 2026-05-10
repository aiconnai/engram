//! Performance benchmarks for the search stack (RML-902).
//!
//! Covers every layer of the retrieval pipeline in isolation and combined:
//! BM25 (FTS5), hybrid (BM25 + vector), TF-IDF embedding, fuzzy correction,
//! and scale tests up to 10 000 memories.
//!
//! Run with: `cargo bench --bench search`
//!
//! ## Performance targets
//! | Operation                     | Target   |
//! |-------------------------------|----------|
//! | `hybrid_search` (any variant) | < 10 ms  |
//! | `search_scale/10K memories`   | < 10 ms  |

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use engram::embedding::{Embedder, TfIdfEmbedder};
use engram::search::{bm25_search, hybrid_search, FuzzyEngine, SearchConfig};
use engram::storage::queries::*;
use engram::storage::Storage;
use engram::types::*;

/// Create an in-memory [`Storage`] pre-loaded with `count` memories.
///
/// Content is drawn from 10 realistic software-engineering topics, rotated
/// with a per-item suffix so FTS5 tokenises each row distinctly. Tags follow
/// a 5-bucket rotation (`topic0`…`topic4` + `development`) to simulate
/// realistic tag cardinality.
fn setup_storage_with_data(count: usize) -> Storage {
    let storage = Storage::open_in_memory().unwrap();

    let sample_contents = vec![
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

    for i in 0..count {
        let content = format!(
            "{} - variation {} with additional context about software development",
            sample_contents[i % sample_contents.len()],
            i
        );

        storage
            .with_transaction(|conn| {
                let input = CreateMemoryInput {
                    content,
                    memory_type: MemoryType::Note,
                    tags: vec![format!("topic{}", i % 5), "development".to_string()],
                    metadata: Default::default(),
                    importance: Some((i % 10) as f32 / 10.0),
                    defer_embedding: true,
                    scope: MemoryScope::Global,
                    ttl_seconds: None,
                    dedup_mode: DedupMode::Allow,
                    dedup_threshold: None,
                    workspace: Some("default".to_string()),
                    tier: MemoryTier::Permanent,
                    event_time: None,
                    event_duration_seconds: None,
                    trigger_pattern: None,
                    summary_of_id: None,
                    media_url: None,
                };
                create_memory(conn, &input)
            })
            .unwrap();
    }

    storage
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
    let storage = setup_storage_with_data(1000);
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
    let mut group = c.benchmark_group("search_scale");
    group.sample_size(50); // Fewer samples for slow benchmarks

    for &size in &[100, 1000, 10000] {
        let storage = setup_storage_with_data(size);
        let embedder = TfIdfEmbedder::new(384);
        let config = SearchConfig::default();
        let query = "authentication JWT tokens";
        let query_embedding = embedder.embed(query).unwrap();

        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(
            BenchmarkId::new("memories", size),
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
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_bm25_search,
    bench_hybrid_search,
    bench_tfidf_embedding,
    bench_fuzzy_search,
    bench_search_at_scale,
);

criterion_main!(benches);
