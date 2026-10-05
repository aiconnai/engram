# Q7 report: retrieval and performance of the candidate

Status: DONE_WITH_CONCERNS. Commits (branch `claude/engram-improvement-plan-edb43d`):

- `12d2c3b feat(search): add candidate retrieval runner, corpus and mode benches`
- `8e7f76f docs(search): record Q7 floors justification and progress`

## What was implemented

1. `scripts/run-quality-candidate.py --candidate-sha --corpus --features --criterion --output`
   (interface verbatim) plus package `scripts/quality_candidate/` (common, gitstate, floors,
   criterion, runner, capture) and `scripts/capture-criterion-candidate.py`.
   - Only children: `rustc -V`, `cargo -V` and the fixed
     `cargo test --locked --test retrieval_quality --no-default-features --features <list> -- --exact candidate_retrieval_metrics --nocapture --test-threads=1`
     (`shell=False`, env allowlist). Features validated against `Cargo.toml [features]`.
   - Verifies `HEAD == --candidate-sha` (40-hex), clean worktree, corpus tracked in the commit.
   - Report: candidate SHA/tree, toolchain, features, hardware, argv, corpus path/SHA256/seed/counts/label provenance,
     provider (offline TF-IDF), per-mode metrics and per-category breakdown, floor checks, floors digest/anchored flag,
     Criterion file+body SHA256 + marker, hot-path ratios, historical file hash labelled "comparative context only".
   - Any incomplete/stale/mismatch/NaN/missing metric/regression exits 1 and writes `status: fail`; a stale passing report at `--output`
     is deleted first.
   - **Criterion binding (defined by me):** `capture-criterion-candidate.py` (fixed `cargo bench` argv, approved bench list,
     bounded numeric params) writes a marker header before the raw Criterion output: candidate SHA and tree, features,
     `rustc -V`, supervisor id (`ENGRAM_QUALITY_SUPERVISOR`, default `local`), UTC capture time (max age 24 h), bench argv/params,
     hardware, SHA256 of the body. The runner recomputes everything from its own checkout. Bare files, tracked historical files,
     other candidate/tree/features/toolchain/supervisor, edited numbers, stale/future captures are rejected. Tamper-evident, not
     authenticated: trust requires capture and runner in the same supervised job.
   - **Floors file** `docs/quality/candidate-floors.json`: entries keyed by corpus SHA256 (a relabeled corpus has no entry =
     "corpus hash divergence"), per-entry `entry_digest` (edit a floor without reviewed `seal` fails), optional `anchor_revision`
     (shrink-only vs an earlier commit; null until the controller sets it after review, reports show `anchored: false`), and v1
     smoke-corpus floors can never be below `budgets.json` retrieval floors. `budgets.json` floors and the 1.15 ceiling untouched
     (a ceiling != 1.15 in budgets.json makes the runner fail).
2. `tests/retrieval_quality.rs`: report-mode evaluator (`evaluate_report`, two modes: `lexical_fts5` = no embeddings, and
   `hybrid_tfidf` = deterministic offline embeddings stored), strict `evaluate` wrapper kept for v1 (existing tests unchanged,
   `baseline.json` untouched and byte-identical), new tests: corpus well-formedness/coverage/label-provenance assertions,
   isolation-leak violation, miss-not-violation, determinism, and `candidate_retrieval_metrics` which prints one
   `ENGRAM_RETRIEVAL_EVAL_JSON=` line (echoes candidate SHA, features, hash of the corpus bytes actually read, seed, counts, provider).
3. `tests/fixtures/retrieval_quality/candidate_corpus.json` (v2, 50 memories, 49 queries, 11 categories): PT/EN exact and paraphrase,
   PT accents, typos (fuzzy), negation, ambiguous entities (Apple, Jaguar, Mercurio, same-name people), exact + near duplicates,
   daily-tier and transcript filtering, 6 workspace-isolation queries (incl. identical text in two workspaces), dates.
   `forbidden_workspaces`/`forbidden_keys` are hard invariants; `discouraged_keys` are diagnostics only.
4. Benches: `benches/memory_ops.rs` gains `storage_modes/{create,get,search}/{in_memory,disk_wal,disk_cloud_safe}`,
   `storage_concurrency/readers_N[_one_writer]` (WAL `StoragePool`), opt-in `latency_percentiles` (p50/p95/p99 after explicit warmup);
   `benches/mcp_dispatch.rs` gains `mcp_dispatch_memory_search_uncached/{in_memory,disk_wal}` (the original search bench repeats one query
   and mostly measures the exact-match cache). `benches/search.rs` untouched (already has file-backed corpus/scale/report).
5. Docs: `docs/quality/retrieval-performance-policy.md` (runner contract, corpora, label provenance, char-based limits per ruling #152,
   separate benchmark modes), `benches/README.md`, `tests/fixtures/README.md`, progress entry (PT-BR).

## Label provenance (explicit)

The v2 memories and relevance labels were authored by an LLM (Claude Sonnet 5.5, this task) and **have not been reviewed by a human**.
They were written before any metric was computed and not edited afterwards. The JSON `labeling` block says so
(`human_review: "none"`, `review_status: "llm-authored-not-human-reviewed"`) and a Rust test asserts that an LLM-authored set is never
marked human-reviewed. "Reviewed separately from tuning" is only structurally satisfied (separation + freeze); an independent human
review is still PENDING and is required before the floors are treated as accepted.

## Measured results (candidate 12d2c3ba, clean detached checkout, Apple M5 Pro, 18 cores, 64 GiB, macOS 27.0.1, rustc 1.96.0)

Host load average during runs: 15-20 (other lanes were building); every number below is noisy.

Retrieval (deterministic, offline TF-IDF, 50 mem / 49 q, seed 20261005, corpus sha256 `14b7eb0f...cfa1`):

| mode | recall@10 | mrr | ndcg@10 |
|---|---|---|---|
| lexical_fts5 | 0.8571 | 0.8776 | 0.8715 |
| hybrid_tfidf | 0.9490 | 0.9099 | 0.9172 |

Findings visible in the per-category breakdown: lexical mode scores 0.0 on all 5 paraphrase queries (FTS5 evidently AND-matches every
term, so natural-language questions return nothing); hybrid recovers part (paraphrase recall 0.8, mrr 0.42); negation lexical recall 0.75;
transcript-included lexical recall 0.67; hard invariants (workspace isolation, tier and transcript filters) held with zero violations.
Typo queries in hybrid mode regress (recall 0.83) vs lexical (1.0). v1 smoke corpus: both modes 1.0.

Criterion `entity_extraction` (default 100 samples, 3 s warmup, 5 s measurement; marker records all):
`entity_extractor_new/default` 1.74 us median; `entity_extraction/extract_mixed` 212.78 us median (re-runs 243.9 and 300.3 us at load 8-15).

**Real regression found:** the runner failed the candidate: `criterion regression: entity_extraction/extract_mixed ratio 12.4113 exceeds 1.15`
against `budgets.json` baseline 17.144 us. Reproducible across three runs, so not noise. Probable cause: commit `bf45c0b` (2026-06-19, #90)
replaced one `text.to_lowercase()` + `find` by `find_case_insensitive_match`, which allocates at every char position for every known
organization/concept. I did not touch src/intelligence, did not rebaseline and did not relax the ceiling. Separately, the
`entity_extractor_new/default` baseline (4.9578 ms) predates the lazy-regex optimization (now ~1.7 us), so the 1.15 ceiling cannot catch a
regression of that path until a reviewed tightening of that baseline (shrink-only is already allowed by the Q1a logic).

Storage modes, reduced Criterion (`--sample-size 20 --warm-up-time 2 --measurement-time 3`; smoke-grade, load ~15) medians:
create in_memory 138.8 us / disk_wal 448.8 us / disk_cloud_safe 1.51 ms; get 18.3 us / 91.3 us / 520 us; search 366.6 / 394.4 / 412.8 us.
Concurrency (batch of 200 reads per thread, WAL pool): readers_1 44.7 ms, readers_4 217.8 ms, readers_8 665.2 ms; with one writer 37.9 / 241.8 / 622.4 ms.
Per-op throughput does not scale with readers (~220-330 us per read in every case), suggesting serialization (likely the per-operation
SQLite file-permission re-assert in `StoragePool::with_connection` / `Storage::with_connection`, which is in the other lanes' src/storage; not
investigated further, not changed).
Percentiles (2000 samples, 200 warmup, us p50/p95/p99): in_memory create 89/123/159, get 16/47/73, search 511/615/697;
disk_wal create 306/1884/2999, get 48/109/188, search 525/590/700; disk_cloud_safe create 1360/2954/4125, get 388/1454/2615, search 489/541/623.
No hosted SLO is claimed.

## TDD evidence (honest)

The Rust evaluator/corpus and the Python runner were written together with their tests rather than strictly test-first, so there is no
classic RED run. Substitute evidence: mutation testing of the runner. Each of 15 single-line mutations that disable a guard was applied,
`python3 -m unittest scripts/test_run_quality_candidate.py` run, and the mutation reverted (final `diff -r` clean). All 15 were KILLED, e.g.
HEAD check skipped -> `test_different_candidate_sha_is_rejected`; entry digest not enforced -> `test_floor_edited_without_reseal_is_rejected`;
NaN accepted -> `test_nan_metric_is_rejected`; ceiling relaxed to 1.5 -> `test_criterion_regression_above_ceiling_is_rejected`;
Criterion body hash unchecked -> `test_numbers_edited_after_capture_are_rejected`; historical file accepted ->
`test_tracked_historical_file_is_rejected_even_with_a_marker`; corpus-hash selection ignored -> `test_edited_corpus_without_reviewed_floors_entry_is_rejected`.
Log: scratchpad `mutation-evidence.txt`.

GREEN:
- `python3 -m unittest scripts/test_run_quality_candidate.py` -> `Ran 49 tests ... OK` (includes a CLI-wrapper subprocess test; fake `cargo`/`rustc` on PATH, throw-away git repos).
- `python3 -m unittest scripts/test_optional_lane_contracts.py` -> 37 OK (neighbouring lane, unaffected).
- `source scripts/ci-required-features.env; cargo test --test retrieval_quality --no-default-features --features "$CI_REQUIRED_FEATURES"` -> exit 0, 8 passed, 0 failed
  (the 3 original tests unchanged and passing + 5 new). Filtered run `-- --exact candidate_retrieval_metrics` runs 1 test.
- `cargo fmt --all -- --check` clean; `cargo clippy --test retrieval_quality --bench memory_ops --bench mcp_dispatch ... -- -D warnings` clean; pre-commit hook (fmt + clippy all-targets/all-features) passed on both commits.
- End to end on a clean detached worktree at 12d2c3ba (`ENGRAM_QUALITY_SUPERVISOR=q7-local-run-1`): `capture-criterion-candidate.py` exit 0; `run-quality-candidate.py` exit 1 only because of the real Criterion regression above (retrieval floor checks all passed, report `status: fail`).
  Logs/report: scratchpad `e2e.log`, `out/report.json`, `out/criterion.txt`.

## Files changed

New: `scripts/run-quality-candidate.py`, `scripts/capture-criterion-candidate.py`, `scripts/quality_candidate/{__init__,common,gitstate,floors,criterion,runner,capture}.py`,
`scripts/test_run_quality_candidate.py`, `docs/quality/candidate-floors.json`, `tests/fixtures/retrieval_quality/candidate_corpus.json`.
Modified: `tests/retrieval_quality.rs`, `benches/memory_ops.rs`, `benches/mcp_dispatch.rs`, `benches/README.md`, `docs/quality/retrieval-performance-policy.md`,
`tests/fixtures/README.md`, `docs/harness/progress/2026-10-05-improvement-lane-r.md`. Not touched: `.github/workflows/*`, `scripts/ci.sh`, `budgets.json`,
`baseline.json`, `baseline.schema.json`, src/*.

## New floors proposed (needs review)

`docs/quality/candidate-floors.json`, v2 corpus entry: lexical_fts5 recall@10 0.857 / mrr 0.877 / ndcg@10 0.871; hybrid_tfidf 0.948 / 0.909 / 0.917 (measured, floored to 3 decimals; one
lost query moves a metric by about 0.02, so any real drop fails). Justification: deterministic regression detectors for a labeled-by-LLM corpus, not a quality bar. v1 corpus entry: floors 1.0
(lexical == existing budgets.json floors; hybrid measured 1.0). Status `proposed-pending-independent-review`; `anchor_revision` null until review accepts.

## Wiring for Q1b (after integration; I did not touch workflows or ci.sh)

1. Required Linux job, after the build and before/alongside the existing budgets step (keep that step but relabel it "historical baseline integrity"):
   ```
   source scripts/ci-required-features.env
   export ENGRAM_QUALITY_SUPERVISOR="${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}"   # same value for both steps
   SHA="$(git rev-parse HEAD)"                                                  # must equal checked-out commit; worktree must be clean
   python3 scripts/capture-criterion-candidate.py --candidate-sha "$SHA" --features "$CI_REQUIRED_FEATURES" --output "$RUNNER_TEMP/criterion.txt"
   python3 scripts/run-quality-candidate.py --candidate-sha "$SHA" --corpus tests/fixtures/retrieval_quality/candidate_corpus.json --features "$CI_REQUIRED_FEATURES" --criterion "$RUNNER_TEMP/criterion.txt" --output "$RUNNER_TEMP/quality-candidate-report.json"
   ```
   Optionally a second run with `--corpus tests/fixtures/retrieval_quality/corpus.json` (reconciles v1 floors with budgets.json). Upload the reports as artifacts.
2. Add `python3 -m unittest scripts/test_run_quality_candidate.py` to the plan/unit job and to `scripts/ci.sh` step 5 (pure Python, ~15 s, no cargo).
3. Q1 consumption rule: require `status == "pass"`, `candidate.sha == $SHA`, `supervisor` equals the job id, `floors.anchored == true`, `criterion.marker.candidate_sha == $SHA`.
4. Gate readiness: do not make the Criterion half blocking until the `extract_mixed` regression is fixed or explicitly accepted/rebaselined with review (the capture is exactly the evidence the runner needs; CI runner variance vs a dev-laptop baseline also needs a reviewed baseline per runner class).
5. After independent review of corpus labels, runner and floors: set `anchor_revision` in `docs/quality/candidate-floors.json` to the accepting commit and run `python3 -m quality_candidate.floors seal` (from `scripts/`).

## Deviations, concerns, NOT RUN

- `tests/retrieval_quality.rs` is 816 lines (guideline 800); I did not split it because the brief names that file and the runner's argv pins `--test retrieval_quality`. A split into a `tests/support` module would be the follow-up if wanted.
- Corpus labels are LLM-authored and unreviewed (see above); floors are proposals; `anchor_revision` is null, so reports say `floors.anchored: false`.
- The runner cannot yet pass end to end on the real repository because of the genuine `extract_mixed` regression; passing paths are covered with the fake toolchain only.
- Benchmarks were run on a heavily loaded host with reduced Criterion parameters; percentile and concurrency numbers are indicative only. Not run: `benches/search.rs` (long scale/report runs), `token_reduction`, graph benches, 100K/1M scale, hosted/CI runner measurements. cargo-nextest full suite not run (only the retrieval test binary and Python tests were required by this task).
- Criterion ceiling comparison uses the baselines in `budgets.json` (dev-hardware historical values); cross-hardware comparison validity is an existing Q1a design property, not changed here.
- `sdk`/server/MCP code unchanged; no schema change, no `SCHEMA_VERSION` bump.

---

# Fix report, review round 1 (commit `4d067f1 fix(search): trust supervisor anchor and require supervisor id in candidate runner`)

## Changes

1. **Trusted anchor (Important 1).** `anchor_revision`/`anchor_note` removed from `docs/quality/candidate-floors.json` (a forged field is ignored; test covers it). New `--floors-anchor <sha>` supplied by the supervisor. `floors.assess_anchor` requires: 40-hex format (else error), anchor != candidate's own commit, `git merge-base --is-ancestor anchor candidate` (new `gitstate.is_ancestor`; unknown/shallow = not an ancestor), a valid floors file at the anchor with an entry for the same corpus hash, and current floors >= anchored floors (a lower floor is a hard failure even if resealed). Result is `floors.anchored` plus `anchor_reason` explaining every not-anchored case. Acceptance status constant: `accepted-independent-review`.
2. **Supervisor and trust model (Important 2).** `--require-supervisor` on runner and capture (fails on unset/empty/`local`, before running any command). New `--candidate-dir`: the runner/capture run from a trusted ref and inspect the candidate checkout; the Q1a budgets module is now imported from the trusted root, never from the candidate (test plants a sentinel-writing `check-quality-budgets.py` in the candidate and asserts it never ran). Policy doc has a "Trust model" section (trusted ref in a separate dir, supervisor-owned id such as `${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}`, marker and echo-back checks are tamper-evidence/plumbing only).
3. **`floors.accepted` (Important 3).** True only when anchored by the trusted anchor AND the anchored entry's review status is accepted. Runner also reports `supervisor_required`. Q1 consumption rule moved into the policy doc ("Consuming a report"): `status == pass`, `candidate.sha == SHA`, `supervisor == job id` (== marker supervisor), `supervisor_required == true`, `floors.accepted == true`.
4. **Minor, folded in.** `--output` inside either checkout is refused before anything is deleted or written (runner and capture); `verify_checkout` re-run after the cargo test run and after the benches; report records `build_env` (`RUSTFLAGS`, `RUSTC_WRAPPER`, `CARGO_TARGET_DIR`, `CARGO_BUILD_JOBS`, `RUSTUP_TOOLCHAIN`) and `cargo_lock_sha256` (missing `Cargo.lock` fails); docs label echo-back checks as plumbing only; `latency_percentiles/noop` placeholder bench removed (the function registers nothing without `ENGRAM_BENCH_PERCENTILES=1`); `storage_modes/create` now uses `iter_custom` with a fresh database every 1000 rows (open outside the timed section).

## Tests (covering the amended code)

- `python3 -m unittest scripts/test_run_quality_candidate.py` -> `Ran 63 tests ... OK` (49 before; new: anchor none/valid+accepted/valid+proposed/own-commit/off-ancestry/unknown/malformed/forged-field/relaxed-below-anchor, require-supervisor missing/local/blank/real id and marker binding, capture require-supervisor, trusted-dir run with malicious candidate module, output inside checkout for runner and capture with tracked file preserved, checkout dirtied by the cargo run, build_env + lock hash).
- Mutation check of the new guards, 10/10 KILLED (ancestry unchecked, own-commit unchecked, accepted ignores status, relaxed-floor check skipped, require-supervisor ignored in runner and in capture, output check skipped, post-run re-verify removed, budgets module loaded from candidate, lock hash unrecorded). Log: scratchpad `mutation-evidence-round1.txt`; package restored byte-identical afterwards.
- `python3 -m unittest scripts/test_optional_lane_contracts.py` -> OK (neighbour unaffected).
- `source scripts/ci-required-features.env; cargo test --test retrieval_quality --no-default-features --features "$CI_REQUIRED_FEATURES"` -> exit 0, 8 passed, 0 failed.
- `cargo check --bench memory_ops` clean; `cargo bench ... --bench memory_ops -- storage_modes/create --sample-size 10 --warm-up-time 1 --measurement-time 1` ran (in_memory 216 us, disk_wal 549 us, disk_cloud_safe 2.17 ms medians, smoke-grade, loaded host); rustfmt check clean; pre-commit hook (fmt + clippy all-targets/all-features) passed on the final commit. A first commit attempt was blocked by another lane's transient clippy error in `src/hooks/post_tool_use.rs`; I did not bypass the hook, waited until it was fixed and retried.

## Updated Q1b wiring (supersedes the earlier snippet)

```
# trusted/ = checkout of the protected base branch (separate dir); cand/ = candidate checkout, clean, at $SHA
source cand/scripts/ci-required-features.env
export ENGRAM_QUALITY_SUPERVISOR="${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}"   # same value for both steps
SHA="$(git -C cand rev-parse HEAD)"
ANCHOR="$(git -C cand merge-base "$SHA" origin/main)"                        # supervisor-chosen; needs enough history
python3 trusted/scripts/capture-criterion-candidate.py --candidate-dir cand --require-supervisor \
  --candidate-sha "$SHA" --features "$CI_REQUIRED_FEATURES" --output "$RUNNER_TEMP/criterion.txt"
python3 trusted/scripts/run-quality-candidate.py --candidate-dir cand --require-supervisor \
  --floors-anchor "$ANCHOR" --candidate-sha "$SHA" \
  --corpus cand/tests/fixtures/retrieval_quality/candidate_corpus.json \
  --features "$CI_REQUIRED_FEATURES" --criterion "$RUNNER_TEMP/criterion.txt" \
  --output "$RUNNER_TEMP/quality-candidate-report.json"      # must be outside both checkouts
```
Q1 gate: `status == "pass"`, `candidate.sha == $SHA`, `supervisor == $ENGRAM_QUALITY_SUPERVISOR` and equal to `criterion.marker.supervisor`, `supervisor_required == true`, `floors.accepted == true`. Keep `python3 -m unittest scripts/test_run_quality_candidate.py` in the plan job / `ci.sh` step 5.

## Notes and remaining concerns

- Floors stay `proposed-pending-independent-review`, so every report currently has `floors.accepted: false` by design. To accept: independent review, then set `review.status` to `accepted-independent-review`, `seal`, merge, and use the merged commit (or a later ancestor-of-candidate) as the anchor. The v1 smoke entry also still carries `carried-over-from-budgets-json` and is therefore not "accepted" either; the reviewer should decide whether to promote it.
- The extract_mixed 12.4x Criterion regression from the first report is unchanged (P1 owns `src/intelligence/entities.rs`); the Criterion half remains non-blocking-ready only after that is resolved or reviewed.
- Trust still depends on the CI wiring actually running from a trusted ref; the runner cannot detect that it is itself being run from the candidate tree.
- The progress-file entry for this round was swept into another lane's commit (`df2f97c`) because the file is shared and was staged by them; content is intact.
