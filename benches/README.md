# Benchmarks

Performance benchmarks for Engram's hot paths. All benchmarks use [Criterion.rs](https://bheisler.github.io/criterion.rs/book/) with in-memory SQLite to isolate CPU cost from disk I/O.

## Quick Start

```bash
# Run all benchmarks
cargo bench

# Run a specific suite
cargo bench --bench memory_ops
cargo bench --bench search
cargo bench --bench mcp_dispatch
cargo bench --bench entity_extraction
cargo bench --bench community_detection
cargo bench --bench traversal
```

## Benchmark Suites

| File | Layer | What it measures |
|------|-------|-----------------|
| [`memory_ops.rs`](memory_ops.rs) | Storage | Create, get, list, cross-reference, and stats queries |
| [`search.rs`](search.rs) | Search | BM25, hybrid (BM25+vector), TF-IDF embedding, fuzzy correction, scale tests |
| [`mcp_dispatch.rs`](mcp_dispatch.rs) | MCP | End-to-end tool dispatch latency (JSON params through handler to response) |
| [`entity_extraction.rs`](entity_extraction.rs) | Intelligence | NER construction cost and extraction throughput |
| [`community_detection.rs`](community_detection.rs) | Graph | Louvain-style community detection on clustered synthetic graphs |
| [`traversal.rs`](traversal.rs) | Graph | Multi-hop BFS traversal on balanced trees |

## Performance Targets

| Operation | Target | Baseline (v0.19.0) | Status |
|-----------|--------|-------------------|--------|
| `memory_create/no_embedding` | < 200 us | 208 us | ~at target |
| `memory_get/by_id` | < 100 us | 55 us | well under |
| `hybrid_search` (any variant) | < 10 ms | 40-444 us | well under |
| `search_scale/10K memories` | < 10 ms | 2.2 ms | well under |
| `entity_extractor_new/default` | < 50 ms | 3.6 us | 740x under |
| `entity_extraction/extract_mixed` | < 50 ms | 13.5 us | well under |

## v0.19.0 Baseline

Captured on Apple Silicon. Median values from 100-sample Criterion runs.

### Storage (`memory_ops.rs`)

| Benchmark | Median |
|-----------|--------|
| `memory_create/no_embedding` | 208 us |
| `memory_get/by_id` | 55 us |
| `memory_list/limit/10` | 127 us |
| `memory_list/limit/50` | 495 us |
| `memory_list/limit/100` | 837 us |
| `memory_list/with_tag_filter/10` | 340 us |
| `memory_list/with_tag_filter/50` | 792 us |
| `memory_list/with_tag_filter/100` | 1.10 ms |
| `crossref/create` | 36 us |
| `crossref/get_related` | 22 us |
| `get_stats` | 334 us |

### Search (`search.rs`)

| Benchmark | Median |
|-----------|--------|
| `bm25_search/authentication` | 202 us |
| `bm25_search/database migration` | 229 us |
| `bm25_search/React hooks optimization` | 271 us |
| `bm25_search/API rate limiting Redis` | 306 us |
| `hybrid_search/short` | 40 us |
| `hybrid_search/medium` | 444 us |
| `hybrid_search/long` | 95 us |
| `tfidf_embedding/short` | 941 ns |
| `tfidf_embedding/medium` | 2.9 us |
| `tfidf_embedding/long` | 6.1 us |
| `tfidf_embedding/batch_100` | 253 us |
| `fuzzy_search/1_char_typo` | 21 us |
| `fuzzy_search/2_char_typo` | 21 us |
| `fuzzy_search/transposition` | 21 us |
| `search_scale/100 memories` | 211 us |
| `search_scale/1K memories` | 516 us |
| `search_scale/10K memories` | 2.2 ms |

### MCP Dispatch (`mcp_dispatch.rs`)

| Benchmark | Median |
|-----------|--------|
| `memory_create` | 215 us |
| `memory_search` | 584 us |
| `memory_list` | 321 us |
| `memory_stats` | 127 us |
| `unknown_tool` (error path) | 284 ns |

### Entity Extraction (`entity_extraction.rs`)

| Benchmark | Median |
|-----------|--------|
| `entity_extractor_new/default` | 3.6 us |
| `entity_extraction/extract_mixed` | 13.5 us |

### Graph (`community_detection.rs`, `traversal.rs`)

| Benchmark | Median |
|-----------|--------|
| `community_detection/500 nodes` | 7.5 ms |
| `traversal/bfs_depth_3` (156 nodes) | 1.2 ms |

## Test Data Shapes

Each benchmark constructs its own dataset to control for variance:

| Suite | Corpus | Structure |
|-------|--------|-----------|
| `memory_ops` | 100-1000 memories | Flat, 10 tag groups, mixed Note/Todo types |
| `search` | 100-10000 memories | 10 software-engineering topics, rotated content |
| `mcp_dispatch` | 100 memories | Flat, synthetic content for dispatch overhead |
| `entity_extraction` | 1 sentence | Mixed entities: persons, orgs, dates, URLs |
| `community_detection` | 500 nodes | 10 clusters of 50, 5% inter-cluster density |
| `traversal` | 156 nodes | Balanced tree, branching factor 5, depth 3 |

## Adding a New Benchmark

1. Create `benches/your_bench.rs` with a module doc explaining what it measures
2. Register it in `Cargo.toml`:
   ```toml
   [[bench]]
   name = "your_bench"
   harness = false
   ```
3. Add function-level doc comments to each `bench_*` function
4. Run it: `cargo bench --bench your_bench`
5. Add the baseline results to this file
