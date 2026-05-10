//! Performance benchmarks for entity extraction (NER pipeline).
//!
//! Measures two distinct costs:
//! 1. **Construction** — `EntityExtractor::new()` compiles regex patterns.
//!    The lazy-load optimization in v0.19.0 reduced this from ~4.9 ms to
//!    ~3.6 µs (740× faster).
//! 2. **Extraction** — running NER on mixed-entity text (persons, orgs,
//!    dates, URLs). Throughput is expressed in bytes to track per-character
//!    scaling.
//!
//! Run with: `cargo bench --bench entity_extraction`
//!
//! ## Performance targets
//! | Operation                          | Target   |
//! |------------------------------------|----------|
//! | `entity_extractor_new/default`     | < 100 ms  |
//! | `entity_extraction/extract_mixed`  | < 100 ms  |

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use engram::intelligence::entities::{EntityExtractionConfig, EntityExtractor};

/// Benchmark `EntityExtractor` construction cost.
///
/// The extractor compiles multiple regex patterns on creation. With lazy-load
/// (v0.19.0), this is deferred to first use, making construction near-instant.
/// This benchmark guards against regressions that move work back into `new()`.
fn bench_entity_extractor_new(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_extractor_new");
    group.throughput(Throughput::Elements(1));

    group.bench_function("default", |b| {
        b.iter(|| EntityExtractor::new(EntityExtractionConfig::default()))
    });

    group.finish();
}

/// Benchmark NER extraction on a mixed-entity sentence.
///
/// The input contains persons (Mr. John Smith, Ms. Jane Doe), an
/// organisation (Anthropic), a project name (Claude), a date
/// (2024-01-25), technical terms (semantic search, vector databases),
/// and a URL. Throughput is bytes so regressions in per-character
/// scanning cost are visible.
fn bench_entity_extraction(c: &mut Criterion) {
    let extractor = EntityExtractor::default();
    let text = "Mr. John Smith and Ms. Jane Doe are working at Anthropic on the Claude project. \
                They met yesterday at 2024-01-25 to discuss semantic search and vector databases. \
                You can find the code at https://github.com/engram/engram.";

    let mut group = c.benchmark_group("entity_extraction");
    group.throughput(Throughput::Bytes(text.len() as u64));

    group.bench_function("extract_mixed", |b| b.iter(|| extractor.extract(text)));

    group.finish();
}

criterion_group!(benches, bench_entity_extractor_new, bench_entity_extraction);

criterion_main!(benches);
