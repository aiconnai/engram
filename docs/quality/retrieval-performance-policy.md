# Retrieval Quality and Performance Baseline Policy

This policy freezes what Engram can truthfully measure today without inventing
hosted SLOs or retrieval-quality floors before the evaluation corpus exists.
It consolidates the existing Criterion benchmark evidence, the dream snapshot
eval runbook, and CI benchmark regression policy into one baseline contract.

## Current evidence sources

- `benches/README.md` defines the local Criterion benchmark suites for storage,
  search, MCP dispatch, entity extraction, graph traversal, and the RFC 0003
  search-index-v2 report package.
- `benches/results/benchmark_baseline.txt` is the current tracked text snapshot
  for historical Criterion evidence. The checker accepts this format only when
  it contains a `Baseline:` header plus one or more named positive metrics.
- `benches/results/benchmark_results.txt` preserves fuller historical Criterion
  output for review context, but it is not a floor by itself.
- `scripts/bench-baseline.sh --name <baseline>` records a Criterion baseline,
  and `scripts/bench-compare.sh --name <baseline>` compares against it.
- `.github/workflows/ci.yml` sets the existing PR performance-regression ceiling
  through `benchmark-action/github-action-benchmark` with
  `alert-threshold: "115%"`. A PR that regresses any tracked benchmark by more
  than 15% must be investigated or explicitly accepted in review.
- `docs/DREAM_SNAPSHOT_EVALS.md` defines deterministic, local, non-networked
  dream snapshot metrics. Those metrics are proposal-quality checks, not hosted
  search SLOs.
- `docs/OPERATIONS.md` explicitly labels hosted latency and availability values
  as planning baselines. Do not publish them as public SLOs until a concrete
  deployment, monitoring path, and incident process have been verified.

## Frozen retrieval fixture schema

Todo 24 owns the actual retrieval corpus and measured floors. The baseline file
it creates must follow `docs/quality/baseline.schema.json` and include:

- `schema_version`: `engram.quality-baseline.v1`.
- `source_revision`: the exact 40-character Git SHA used to generate results.
- `generated_at`: RFC3339 UTC timestamp in canonical Zulu form (`YYYY-MM-DDTHH:MM:SSZ`).
- `deterministic_seed`: integer seed for corpus order, query order, and any
  randomized tie-breaking.
- `corpus`: name, version, fixture path, memory count, query count, and explicit
  field lists for memories, queries, and relevance judgments.
- `metrics`: exactly `recall@10`, `mrr`, and `ndcg@10`, each in `[0, 1]`.
- `benchmark_evidence`: paths to the Criterion baseline and dream eval runbook
  used as context for the review.

The fixture schema is intentionally content-agnostic: private or proprietary
memories belong in a committed synthetic fixture or a documented private fixture
path, not in this policy text.

## Deterministic metrics

The first retrieval-quality baseline must compute these metrics over the frozen
fixture and deterministic seed:

| Metric | Meaning |
|---|---|
| `recall@10` | Fraction of expected relevant memories retrieved in the top 10 results. |
| `mrr` | Mean reciprocal rank of the first relevant result per query. |
| `ndcg@10` | Normalized discounted cumulative gain over the top 10 results. |

The metrics are local engineering evidence. They do not claim production
semantic quality for hosted embeddings or customer corpora.

## Baseline generation and review rule

Use these commands for the existing performance evidence:

```bash
./scripts/bench-baseline.sh --name main
./scripts/bench-compare.sh --name main
python3 scripts/check-quality-baseline.py benches/results/benchmark_baseline.txt
```

Todo 24 must add the retrieval fixture generator/runner and produce a JSON
baseline that satisfies `docs/quality/baseline.schema.json`. Its measured
`recall@10`, `mrr`, and `ndcg@10` values become floors only after review accepts
that corpus, seed, query set, and relevance judgments. Floor changes require:

1. A diff to the frozen fixture or runner, not an unexplained number edit.
2. A regenerated baseline tied to the new `source_revision`.
3. Review acknowledgment of whether the floor moved because quality changed or
   because the corpus/relevance contract changed.

Do not lower a floor to make CI pass without documenting the root cause and the
review decision.

## Local budgets vs hosted SLOs

Criterion medians, the 115% PR ceiling, and the future retrieval fixture floors
are local engineering budgets. They are useful to catch regressions and compare
indexing approaches. They are not public hosted SLOs. Hosted SLOs require a
specific deployment, telemetry, alert routing, and incident-response contract as
called out in `docs/OPERATIONS.md`.
