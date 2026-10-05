# P1 report: fix performance regressions measured by Q7

Status: DONE_WITH_CONCERNS (entity regression fixed and gated green; storage change deliberately
not made because measurement shows the permission re-assert is not the bottleneck; see concerns).

Commits (branch `claude/engram-improvement-plan-edb43d`):

- `772e84a perf(intelligence): lowercase entity text once per extraction`
- `0a46e9b docs(harness): record P1 entity extraction perf fix progress`

## 1. entity_extraction/extract_mixed regression

Root cause confirmed: `find_case_insensitive_match` (bf45c0b, #90) walked every char position and,
for each of the ~57 known organizations/concepts, sliced a window and called `to_lowercase()` on it:
O(text_chars x terms) allocations per `extract` (measured: 149k-193k allocations for 2.6k-3.4k chars).

Fix (`src/text_util.rs`, new `LowercaseIndex`; `src/intelligence/entities.rs` uses it, the old helper is removed):

- The text is lowercased once per `extract` (`char::to_lowercase` per char) with a
  `(original_offset, lower_offset)` map per char (no map for ASCII text, where offsets are identical).
- Each term is found with `str::find` in the lowercased copy; a hit counts only if it starts on a char
  group boundary and ends exactly after `needle.chars().count()` original chars, and the returned slice
  is taken from the original via the map (invariant 28: no offsets from case-folded text are used on
  the original).
- `Σ` is the only char whose `str::to_lowercase` depends on context (final sigma). Texts containing it
  use an exact scan: an allocation-free plausibility check per start (`Σ` accepted as `σ`/`ς`), and
  `window.to_lowercase() == needle` only for windows that pass.
- Semantics are identical to the C3-era matcher, including its edge cases (e.g. `İ` never matches
  because its lowercase has 2 chars and windows are counted in original chars; this pre-existing
  behavior is preserved, not changed).

### TDD evidence

RED (before implementation, test written first):
`source scripts/ci-required-features.env; cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --test entity_extraction_alloc_tests`
```
extract allocated 192985 times for 3392 chars of "plain words without known names and lowercase filler " (bound 64) ...
extract allocated 171097 times for 2880 chars of "café façade naïve déjà vu İ ẞ e\u{301} plain words " (bound 64) ...
extract allocated 149209 times for 2624 chars of "ΟΔΥΣΣΕΑΣ plain words without known names " (bound 64) ...
test result: FAILED. 0 passed; 3 failed
```
Expected: the old matcher allocates per char position per term; the test (counting global allocator,
thread-local counter, regex caches warmed) bounds one `extract` call to 64 allocations for 64x and 512x
repetitions of entity-free filler (ASCII, non-ASCII, and capital-sigma variants).

GREEN: same command -> `test result: ok. 3 passed; 0 failed`.

Equivalence (in `src/text_util.rs` tests, old matcher kept verbatim as `reference_find_window` oracle):
- `find_window_matches_reference_on_adversarial_unicode`: 13 texts x 24 needles (İ, ẞ/ß/SS, combining
  acute and dot-above, Kelvin sign, Σ/σ/ς final-sigma contexts, ǅ titlecase, ﬁ ligature, emoji, overlap, empty).
- `find_window_matches_reference_on_seeded_random_inputs`: 20 000 cases, xorshift64* with fixed seed
  `0x5EED_0FE1_7000_0001`, alphabet of case/length/context-changing chars; needles are lowercased windows
  of the text or random strings.
- `find_window_maps_matches_back_to_original_slices`: concrete offsets/slices.
- Mutation check: disabling the Σ fallback killed 2 tests; dropping the end-alignment check killed 3
  (mutations reverted, file restored byte-identical).

### Before/after (Criterion defaults: 100 samples, 3 s warm-up, 5 s measurement; same machine; sequential;
clean detached worktrees at 98103b0 (before) and 98103b0+patch (after), shared scratch target dir)

Host: Apple M5 Pro, 18 cores, macOS 27, rustc 1.96.0. Load averages recorded per run.

| run | load (1m) | extract_mixed before | extract_mixed after | new/default before | after |
|---|---|---|---|---|---|
| 1 | 5.0 / 9.0 | 228.77 us | 5.58 us | 1.318 us | 1.343 us |
| 2 | 4.7 / 8.4 | 234.11 us | 5.60 us | 1.307 us | 1.526 us |
| 3 | 5.1 / 8.6 | 246.16 us | 5.02 us | 1.365 us | 1.453 us |

~43x faster; ~3.1x faster than the 17.144 us budgets.json baseline. `entity_extractor_new` is unaffected
(construction; difference is load noise).

### Q7 candidate runner on the fix commit (clean detached worktree of 772e84a, `git status` clean)

```
export CARGO_TARGET_DIR=<scratch>/target ENGRAM_QUALITY_SUPERVISOR=p1-local-run-1
source scripts/ci-required-features.env; SHA=772e84aaab95b511d3f1e421edacd897971bfc90
python3 scripts/capture-criterion-candidate.py --candidate-sha "$SHA" --features "$CI_REQUIRED_FEATURES" --output out/criterion.txt   -> exit 0
python3 scripts/run-quality-candidate.py --candidate-sha "$SHA" --corpus tests/fixtures/retrieval_quality/candidate_corpus.json \
  --features "$CI_REQUIRED_FEATURES" --criterion out/criterion.txt --output out/report.json                                           -> exit 0, status: pass
```
Host load during capture/runner: 10.7 -> 25.4 -> 24.7 (other lanes compiling).
Report hot paths: `entity_extraction/extract_mixed` observed 8.50 us vs 17.144 us, **ratio 0.496** (ceiling 1.15
untouched); `entity_extractor_new/default` 2.37 us vs 4.9578 ms, ratio 0.00048. Retrieval floor checks passed.
`floors.anchored: false` / `accepted: false` (no `--floors-anchor` supplied; floors still pending independent
review, as before). `--require-supervisor` not used (local run). Artifacts: scratchpad `out/report.json`,
`out/criterion.txt`, `out/runner.log`.

No rebaseline/relaxation applied. `budgets.json`, `benchmark_baseline.txt` untouched.

## 2. Storage permission re-assert (measurement only, no code change)

Experiment (scratch detached worktree at 98103b0 only, never committed): two env-gated toggles:
`ENGRAM_P1_SKIP_REASSERT` made `reassert_sqlite_artifact_permissions_for_config` a no-op;
`ENGRAM_P1_READONLY` made the bench's reader threads run a plain `SELECT content ... WHERE id = ?`
instead of `get_memory`. Bench: `memory_ops` `storage_concurrency/readers_{1,4,8}` and
`storage_modes/get/disk_wal`, `--sample-size 10 --warm-up-time 1 --measurement-time 3`, 2 interleaved rounds,
load 5-13. (A zsh word-splitting slip made my first "ro_skip" column actually "skip only"; it was rerun
correctly; the raw logs say so.)

| config | readers_1 (200 ops) | readers_4 | readers_8 |
|---|---|---|---|
| base (`get_memory`, re-assert on) | 12.3 / 13.5 ms | 254 / 191 ms | 809 / 569 ms |
| re-assert skipped | 11.6 / 14.4 ms | 262 / 200 ms (p=0.36/0.11) | 831 / 597 ms (p=0.64/0.15) |
| read-only SELECT, re-assert on | 1.03 / 1.05 ms | 2.6 / 4.3 ms | 6.9 / 7.3 ms |
| read-only SELECT, re-assert skipped | 0.34 / 0.42 ms | 1.5 / 1.6 ms | 3.6 / 4.3 ms |

Conclusions:
- The Q7 non-scaling is **not** caused by the per-operation re-assert: skipping it changes nothing
  significant. The cause is that `get_memory` does access tracking (`UPDATE memories SET access_count ...`)
  on every call, so the bench's "readers" are writers serialized by SQLite's single write lock
  (~60 us/op alone, ~350-500 us/op with 8 contending threads). With true reads the pool is ~100x faster.
- The re-assert costs ~3.5 us per storage call (3 `lstat`s on the db, -wal, -shm). That is negligible for
  writes but is ~2/3 of a trivial point read (5.2 us -> 1.7 us without it).
- Per the brief's condition ("if so"), I did not change `src/storage`: invariant #27 and the per-op 0600
  re-assert are untouched; `src/storage/connection.rs` tests (12) and
  `tests/storage_posix_lock_regression_tests.rs` (7) pass.

## Verification (main worktree, after the fix, with other lanes' uncommitted WIP present)

- `cargo fmt --all -- --check` (rustfmt on the 3 files): clean.
- `cargo clippy --no-default-features --features "$CI_REQUIRED_FEATURES" --lib --tests --benches -- -D warnings`: exit 0
  (first run caught an unevenly grouped hex literal in the new test; fixed before commit).
- `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests`: exit 0, 57 test binaries,
  2182 passed, 0 failed, 2 ignored. Includes lib unit tests 1661 passed (text_util + entities + storage::connection),
  `unicode_adverse_parsers_tests` 12, `property_tests` 33, `storage_posix_lock_regression_tests` 7,
  `entity_extraction_alloc_tests` 3.
- Pre-commit hook (fmt + clippy all-targets all-features) passed on both commits.

## Files changed

- `src/text_util.rs` (new `LowercaseIndex` + helpers + tests; 276 lines added, file now ~355 lines)
- `src/intelligence/entities.rs` (uses `LowercaseIndex`; old helper removed; net -17 lines)
- `tests/entity_extraction_alloc_tests.rs` (new; counting-allocator regression guard)
- `docs/harness/progress/2026-10-05-improvement-lane-r.md` (P1 entry, PT-BR)

## Concerns / proposals (not applied)

1. Q7's `storage_concurrency/readers_N` measures write contention, not read scaling, because `get_memory`
   tracks access. Proposal: add a read-only variant (e.g. `get_memory_internal(.., false)` exposed, or a
   `readers_N_untracked` bench) so the group means what its name says. Not done: bench semantics are Q7's
   and changing them would break comparability without review.
2. Permission re-assert on read paths (~3.5 us/op) could be limited to calls that can create or change
   artifacts (open, write transactions, checkpoint, vacuum, compact), since SQLite creates -wal/-shm with
   the 0600 db mode. That narrows the "re-narrow after an external chmod" guarantee from every call to
   every write, so it is a G1 security design decision for review/owner, not a perf tweak. Not done.
3. Rebaseline of `entity_extractor_new/default` (4.9578 ms baseline vs ~1.3-2.4 us now) and possibly
   `extract_mixed` (17.144 us vs ~5-8.5 us): I did not create a rebaseline commit. Measured variance at the
   microsecond scale on this loaded laptop is +/-15-60% between runs (1.31-2.37 us for new/default), so a
   tightened baseline with the 1.15 ceiling would be flaky; it needs a reviewed per-runner-class baseline
   (also touches `benches/results/benchmark_baseline.txt` that budgets.json points to).
4. Inherited semantic quirk kept on purpose: windows are counted in original chars, so a term can never
   match text containing a char whose lowercase is longer (e.g. `İ`). Preserved for C3 compatibility.

## NOT RUN

- Hosted/CI-runner benchmarks; `--require-supervisor` and `--floors-anchor` runner modes (need supervisor
  ids and an accepted anchor).
- v1 corpus runner pass; `memory_ops` full Criterion run at default params (only the filtered experiment above).
- cargo-nextest; `--all-features` test suite (clippy all-features ran via the pre-commit hook).
