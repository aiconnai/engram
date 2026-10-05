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

## Historical baseline integrity lane (CI and local)

The required GitHub job (`Test (ubuntu-latest)`) and `scripts/ci.sh` (`make ci`,
`just ci`, or `./scripts/ci.sh quality-budgets` for this lane alone) run
`scripts/check-quality-budgets.py` with the same complete argv:

```bash
python3 scripts/check-quality-budgets.py \
  --budgets docs/quality/budgets.json \
  --retrieval tests/fixtures/retrieval_quality/baseline.json \
  --criterion benches/results/benchmark_results.txt
# plus the same call with --self-test-degraded
```

This lane verifies **historical baseline integrity**: the committed retrieval
floors, the immutable source-revision evidence, and the committed Criterion
snapshot are consistent and the 1.15 ceiling and floors are not relaxed. It is
not candidate performance evidence: `benches/results/benchmark_results.txt` is a
tracked historical snapshot, not a measurement of the code under review. A green
lane must not be cited as "performance verified". Current-run measurement (runtime
input with SHA, toolchain, features and a fresh Criterion file) is the separate,
report-only candidate lane below (Q1b); it does not replace this lane.

Rules:

- `--criterion` is mandatory (the checker exits 2 without it); never make it
  optional and never lower floors or the 1.15 ceiling to get green.
- A missing tool (`python3`), input file, Criterion hot path, or git history for
  the immutable baseline is a failure, never a skip.
- `scripts/test_check_quality_ci_contract.py` pins the checker contract, the
  workflow argv, the local `scripts/ci.sh` argv, and their agreement;
  `scripts/ci-parity-check.sh` executes the argv-level contract tests.

## Candidate evidence runner (Q7)

The historical baseline integrity lane above (`scripts/check-quality-budgets.py`)
checks the **historical** baseline files committed in the repository. It proves
"baseline integrity", not that the candidate under review performs the same.
Candidate evidence comes from `scripts/run-quality-candidate.py`.

### Trust model (read before wiring CI)

The candidate controls its own tree. A verifier that runs candidate code, reads
the candidate's floors as authority, or takes its supervisor id from a default
proves nothing. In CI:

- Run `capture-criterion-candidate.py` and `run-quality-candidate.py` from a
  **trusted ref**: check out the base branch in a separate directory and point
  `--candidate-dir` at the candidate checkout. The runner then never imports the
  candidate's `check-quality-budgets.py`; only data files (budgets, floors,
  corpus, Cargo files) come from the candidate.
- Set `ENGRAM_QUALITY_SUPERVISOR` to a supervisor-owned id for both steps (for
  example `${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}`) and pass `--require-supervisor`
  to both, which fails on an empty or `local` id.
- Pass the trusted review anchor with `--floors-anchor <sha>`: a commit chosen by
  the supervisor (for example the merge-base with the protected base branch). The
  `anchor_revision` field no longer exists in the floors file; a forged one is
  ignored.
- The Criterion marker and the echo-back checks of the test process (candidate
  SHA, corpus hash, counts) are **tamper-evidence and plumbing consistency
  checks only**. They stop stale, mixed-up and hand-edited evidence; they do not
  authenticate anything against a candidate that deliberately lies.

```bash
# trusted/ = base branch checkout, cand/ = candidate checkout (clean, at $SHA)
source cand/scripts/ci-required-features.env
export ENGRAM_QUALITY_SUPERVISOR="${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}"
SHA="$(git -C cand rev-parse HEAD)"
ANCHOR="$(git -C cand merge-base "$SHA" origin/main)"
python3 trusted/scripts/capture-criterion-candidate.py --candidate-dir cand --require-supervisor \
  --candidate-sha "$SHA" --features "$CI_REQUIRED_FEATURES" --output "$RUNNER_TEMP/criterion.txt"
python3 trusted/scripts/run-quality-candidate.py --candidate-dir cand --require-supervisor \
  --floors-anchor "$ANCHOR" --candidate-sha "$SHA" \
  --corpus cand/tests/fixtures/retrieval_quality/candidate_corpus.json \
  --features "$CI_REQUIRED_FEATURES" --criterion "$RUNNER_TEMP/criterion.txt" \
  --output "$RUNNER_TEMP/quality-candidate-report.json"
```

`--output` must be outside both checkouts (the runner refuses and never deletes a
path inside them). Locally, without `--require-supervisor`, the supervisor id is
`local` and floors are reported as not anchored and not accepted.

### Consuming a report (Q1 rule)

Accept a report only when **all** hold: `status == "pass"`,
`candidate.sha == $SHA`, `supervisor == <the job's own id>` (and equal to
`criterion.marker.supervisor`), `supervisor_required == true`, and
`floors.accepted == true`. `floors.anchored` alone is not enough: a report whose
floors are anchored but whose anchored review status is not
`accepted-independent-review` still passes the numeric checks yet carries
unreviewed floors, so it must not be treated as accepted evidence.

`scripts/consume-quality-candidate.py --report --candidate-sha --supervisor
[--summary]` applies this rule: exit 0 accepted, exit 1 rejected (integrity:
status, SHA or supervisor binding), exit 3 intact but `floors.accepted` is not
true (floors pending review), exit 2 usage. `scripts/test_quality_candidate_consume.py`
pins it.

### CI wiring (Q1b): report-only until floors are accepted

`.github/workflows/quality-candidate.yml` (job "Candidate Quality Evidence
(report-only, not required)") runs the snippet above on pull requests and pushes
to `main`. It is **not** a branch-protection context, is not part of `ci.yml`
and is in no `needs:` chain, so it cannot block a merge while the floors are
pending. It uses no `continue-on-error`:

- The candidate is checked out in `cand/` with full history. The anchor is
  `git merge-base HEAD origin/main`; when that equals the candidate (a push to
  `main`), the first parent is used. The anchor is checked out as a separate
  worktree `trusted/`, and capture, runner and consumer run from there. If the
  anchor does not have them yet (bootstrap, before this lane reaches the
  protected base), the job fails with an explicit error instead of running
  candidate-provided verifier code.
- The feature list is read as data from `trusted/scripts/ci-required-features.env`
  (never sourced from the candidate). `ENGRAM_QUALITY_SUPERVISOR` is
  `${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}` and both steps pass
  `--require-supervisor`.
- A failed capture or run, or a consumer exit 1, fails the job. Consumer exit 3
  (intact evidence, floors not accepted) is reported as a warning and in the step
  summary as `NOT ACCEPTED`; that is the expected state of the report-only phase.
  The report and Criterion capture are uploaded as the
  `quality-candidate-evidence` artifact.
- The candidate's build scripts and tests still execute in the same job and user
  as the trusted verifier. The trusted checkout protects against verifier edits
  in the diff, not against a malicious build script at runtime; acceptance stays
  a human decision.

Fast offline unit tests are blocking: the required `Test (ubuntu-latest)` job and
`scripts/ci.sh` step 5 run `python3 -m unittest scripts/test_run_quality_candidate.py`
and `scripts/test_quality_candidate_consume.py`. Promotion of the candidate lane to
a required context needs, first, independent human review of the v2 corpus labels,
runner and floors (`review.status = accepted-independent-review`, sealed, merged so
the anchor carries it), a reviewed per-runner-class Criterion baseline, and
anchoring of the hot-path baselines: `runner.py` still loads
`docs/quality/budgets.json` from the candidate tree, so a candidate can raise
its own hot-path baselines until they are read from the trusted anchor the way
the floors are.

### What the runner enforces

All failures exit non-zero and leave a `status: fail` report (a stale passing
report at `--output` is deleted first).

- **Fixed argv, no shell.** The only child commands are `rustc -V`, `cargo -V`
  and `cargo test --locked --test retrieval_quality --no-default-features
  --features <list> -- --exact candidate_retrieval_metrics --nocapture
  --test-threads=1`. Features are validated against `Cargo.toml` `[features]`.
- **Candidate identity.** `HEAD` must equal `--candidate-sha`, the worktree must
  be clean before **and after** the cargo run, and the report records SHA, tree,
  toolchain, features, hardware, `RUSTFLAGS`/`RUSTC_WRAPPER`/`CARGO_TARGET_DIR`
  and the `Cargo.lock` SHA256. The corpus must be tracked in that commit.
- **Corpus identity.** The corpus SHA256 must have a reviewed entry in
  `docs/quality/candidate-floors.json`, and the test process must echo the same
  hash, seed, memory/query counts and candidate SHA (plumbing consistency). A
  relabeled corpus has no floors entry and fails ("corpus hash divergence").
- **Metrics.** `recall@10`, `mrr`, `ndcg@10` for two modes (`lexical_fts5`: no
  embeddings, FTS5/BM25 only; `hybrid_tfidf`: deterministic offline TF-IDF
  embeddings stored, hybrid search) must all be present, finite and in [0, 1]
  and at or above the floors. Hard invariants (workspace leak, forbidden
  tier/transcript memory) fail the evaluation itself.
- **Criterion bound to the candidate.** `--criterion` must be produced by
  `capture-criterion-candidate.py`: a marker header carries candidate SHA and
  tree, features, `rustc -V`, supervisor id, capture time (max age 24 h), bench
  argv/params, hardware and a SHA256 of the body. Bare files, tracked historical
  files (`benches/results/*`), another candidate/tree/toolchain/features/
  supervisor, edited numbers and stale captures are rejected.
- **Ceiling.** Hot paths from `docs/quality/budgets.json` are compared with the
  existing 1.15 ratio ceiling. A `budgets.json` ceiling other than 1.15 fails.
  `benches/results/benchmark_results.txt` is recorded as comparative context only
  (`historical_context`) and is never candidate evidence.

### Floors file

`docs/quality/candidate-floors.json` entries are keyed by corpus SHA256. Guards:
each entry has an `entry_digest` (edit a floor without a `seal` and it fails;
tamper-evidence for accidents only, since the candidate controls the file); the
supervisor anchor (`--floors-anchor`) must be a strict ancestor of the candidate,
hold an entry for the same corpus hash, and current floors may never be lower
than the anchored ones. `floors.anchored` means that holds; `floors.accepted`
additionally requires the anchored entry's `review.status` to be
`accepted-independent-review`. An anchor equal to the candidate's own commit, off
its ancestry, unknown, or absent yields `anchored: false` with `anchor_reason`.
The v1 smoke corpus may never have a floor below the `budgets.json` retrieval
floors. Re-seal after a reviewed edit with
`cd scripts && python3 -m quality_candidate.floors seal ../docs/quality/candidate-floors.json`.
Floors only rise through a diff to the corpus or runner plus review; never lower
a floor to make a run pass, and never rebaseline to hide a regression.

### Corpora and label provenance

| Corpus | Memories / queries | Purpose |
|---|---|---|
| `tests/fixtures/retrieval_quality/corpus.json` (v1, frozen) | 9 / 6 | Plumbing smoke test; strict mode; byte-compared with `baseline.json`. Metrics are 1.0 by construction and do **not** show retrieval quality on customer-like data. |
| `tests/fixtures/retrieval_quality/candidate_corpus.json` (v2) | 50 / 49 | PT/EN exact and paraphrase, accent-insensitive PT, typos (fuzzy), negation, ambiguous entities, exact and near duplicates, daily-tier and transcript filtering, cross-workspace isolation, dates. |

Label provenance of v2 (recorded in the file's `labeling` block and asserted by
`candidate_corpus_is_well_formed_and_covers_required_categories`): the memories
and relevance labels were **authored by an LLM (Claude Sonnet 5.5, task Q7,
2026-10-05) and have not been reviewed by a human**. They were written before any
metric was computed and are not edited to move a number; a label change needs a
corpus version bump, a new corpus SHA256 and a new reviewed floors entry. Until a
human reviews them, the floors are `proposed-pending-independent-review`
regression detectors, not an accepted quality bar. `forbidden_workspaces` and
`forbidden_keys` are hard invariants; `discouraged_keys` (negation/ambiguity
siblings) are reported as diagnostics only because lexical engines cannot honour
them as invariants.

Document ingestion limits are character based and independent of tokens (owner
ruling #152). The corpora and runner therefore never tokenize content to size
ingested documents, and no tokenizer is part of the evaluation.

### Benchmarks that stay separate

`benches/memory_ops.rs` reports `storage_modes/{create,get,search}/{in_memory,
disk_wal,disk_cloud_safe}` and `storage_concurrency/readers_N[_one_writer]` as
distinct ids, and `latency_percentiles` (`ENGRAM_BENCH_PERCENTILES=1`) prints
p50/p95/p99 after a stated warmup. `benches/mcp_dispatch.rs` adds
`mcp_dispatch_memory_search_uncached/{in_memory,disk_wal}` because the original
`mcp_dispatch_memory_search` repeats one query and mostly measures the cache hit.
`storage_modes/create` opens a fresh database every 1000 rows so file growth does not skew the number. Never compare an in-memory number with a disk or WAL one, and record the machine
load next to any result: Criterion on a busy host is noise.

## Local budgets vs hosted SLOs

Criterion medians, the 115% PR ceiling, and the retrieval fixture floors and the candidate floors
are local engineering budgets. They are useful to catch regressions and compare
indexing approaches. They are not public hosted SLOs. Hosted SLOs require a
specific deployment, telemetry, alert routing, and incident-response contract as
called out in `docs/OPERATIONS.md`.
