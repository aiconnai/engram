# 2026-05-31 — Compression Benchmark Trace (`token_reduction/semantic_compression/fixed_corpus_ratio_recall`)

## Evidence collected

- Command:
  - `cargo bench --bench token_reduction -- fixed_corpus_ratio_recall`
- Target artifact:
  - `docs/harness/decisions/phase2-3-compression-benchmark-2026-05-31.md`
  - `docs/rfcs/0002-compression-benchmarks-for-context.md`

## Raw output (canonical capture)

```text
Finished `bench` profile [optimized] target(s) in 0.15s
Running benches/token_reduction.rs (target/release/deps/token_reduction-b1747be70bd9882a)
Gnuplot not found, using plotters backend
token_reduction/semantic_compression/ratio_recall avg_ratio=0.7525 avg_recall=0.7778 covered_facts=7/9
Benchmarking token_reduction/semantic_compression/fixed_corpus_ratio_recall
Benchmarking token_reduction/semantic_compression/fixed_corpus_ratio_recall: Warming up for 3.0000 s

Warning: Unable to complete 20 samples in 5.0s. You may wish to increase target time to 7.0s, enable flat sampling, or reduce sample count to 10.
Benchmarking token_reduction/semantic_compression/fixed_corpus_ratio_recall: Collecting 20 samples in estimated 6.9996 s (210 iterations)
Benchmarking token_reduction/semantic_compression/fixed_corpus_ratio_recall: Analyzing
token_reduction/semantic_compression/fixed_corpus_ratio_recall
                        time:   [33.422 ms 34.129 ms 35.222 ms]
                        thrpt:  [3.9464 Kelem/s 4.0728 Kelem/s 4.1590 Kelem/s]
                 change:
                        time:   [-0.7831% +1.3268% +3.7586%] (p = 0.26 > 0.05)
                        thrpt:  [-3.6224% -1.3094% +0.7893%]
                        No change in performance detected.
Found 1 outliers among 20 measurements (5.00%)
  1 (5.00%) high severe
```
