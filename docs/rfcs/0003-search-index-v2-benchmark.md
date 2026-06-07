# RFC 0003 Search Index v2 Benchmark Package

This benchmark package measures the current SQLite + FTS5 + manual cosine
baseline before adopting Tantivy, ANN/HNSW, or any other search index v2. It
also includes a bench-only `sqlite-vec` `vec0` spike so constant-factor vector
optimizations can be measured without touching production schema.

## Commands

Default daily benchmark:

```bash
cargo bench --bench search
```

Medium scale:

```bash
ENGRAM_SEARCH_BENCH_SCALE=medium cargo bench --bench search search_scale
```

Full RFC scale:

```bash
ENGRAM_SEARCH_BENCH_SCALE=large cargo bench --bench search search_scale
```

Markdown report for the five RFC dimensions:

```bash
ENGRAM_SEARCH_BENCH_REPORT=1 \
ENGRAM_SEARCH_BENCH_REPORT_SIZES=100000,1000000 \
ENGRAM_SEARCH_BENCH_EMBEDDING_DIMS=384 \
ENGRAM_SEARCH_BENCH_REPORT_ITERS=100 \
cargo bench --bench search search_index_v2_report
```

The report is written to:

```text
target/criterion/search-index-v2/report.md
```

## Dimensions

The report measures:

| Dimension | Measurement |
|---|---|
| Quality | `precision@10`, `MRR`, and `nDCG@10` over deterministic topic queries |
| Latency | p50/p95 for keyword-only, hybrid, manual cosine, `vec0_ideal`, and `vec0_postfilter` |
| Rebuild time | `rebuild_derived_indexes(..., rebuild_fts=true, apply=true)` elapsed time |
| Delete lag | Time until a soft-deleted memory disappears from search results |
| Disk growth | SQLite database + WAL + SHM bytes before load, after load, after rebuild, and after delete |

## Interpretation

The benchmark intentionally keeps SQLite as the canonical store. Embeddings are
stored in the existing `embeddings` table and semantic search uses the current
manual cosine path. That means `manual_cosine` and the semantic side of
`hybrid` expose the current O(n) scan behavior directly.

The report also creates a disposable `bench_vec` `vec0` table with
`rowid = memories.id`. `vec0_ideal` measures best-case KNN without production
filters. `vec0_postfilter` over-fetches, applies the default search filters
(`valid_to`, `expires_at`, `transcript_chunk`, `archived`, workspace), then
truncates to 10 results. `vec0_postfilter` is the acceptance line for the
`sqlite-vec` spike because `sqlite-vec` 0.1.x remains brute-force O(n).

Quality metrics use deterministic TF-IDF embeddings. They are useful for
checking ranking plumbing and synthetic-topic relevance, but they undersell
production semantic quality from OpenAI, ONNX, Cohere, Voyage, or similar
providers. Treat the quality numbers as baseline-shape evidence unless the run
uses a production-grade embedding backend.

Latency scales with `memories x embedding_dimensions`. Smoke runs can use fewer
dimensions, but architecture decisions should use at least `384` dimensions and
at least `100` report iterations.

Delete lag measures logical search visibility after Engram soft-deletes a
memory. For FTS5 this propagation is deterministic because SQLite triggers keep
the derived index in transaction scope. ANN/HNSW candidates must prove this axis
separately because physical stale vectors are the historical failure mode.

Tantivy, ANN/HNSW, or another index should only advance if this package shows a
measured problem or quality gap that is not already addressed by the current
multi-signal ranking stack.

## Environment

Optional knobs:

| Variable | Default | Purpose |
|---|---:|---|
| `ENGRAM_SEARCH_BENCH_SCALE` | default | `medium` adds 100K; `large` adds 100K and 1M |
| `ENGRAM_SEARCH_BENCH_REPORT` | unset | Set to `1` to write the Markdown report |
| `ENGRAM_SEARCH_BENCH_REPORT_SIZES` | `10000` | Comma-separated report corpus sizes |
| `ENGRAM_SEARCH_BENCH_REPORT_ITERS` | `10` | Manual latency samples per search mode in the report; use `100+` for decision-grade p95 |
| `ENGRAM_SEARCH_BENCH_EMBEDDING_DIMS` | `64` | TF-IDF dimensions for scale/report corpora; use `384+` for production-like scan cost |
| `ENGRAM_SEARCH_BENCH_LOAD_CHUNK` | `5000` | Insert transaction chunk size for synthetic corpus loading |
| `ENGRAM_SEARCH_BENCH_VEC0_OVERFETCH` | `10` | `vec0_postfilter` over-fetch multiplier before applying production-style filters |
