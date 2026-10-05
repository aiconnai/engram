# C5 report: selective persistence refactor in the create handler

Status: DONE (small, selective; the main extraction named in the brief was already delivered by C7).

## Characterization after C7 (what raw SQL/persistence logic remained)
- `src/mcp/handlers/memory_crud/*.rs` (non-test): zero raw SQL (`INSERT|UPDATE|DELETE|SELECT|execute|prepare` grep clean).
  The embedding SQL is already a typed function, `persist_computed_embedding` / `enqueue_embedding_job`
  (`src/embedding/queue/jobs.rs`), used by create, drain and worker; provider call is outside any tx; HNSW updated after commit.
  The brief's "extract embedding SQL into a typed storage function" is therefore DONE by C7: not redone, no invented refactor.
- Remaining real persistence-boundary issue in `memory_create`: inside the create transaction closure the handler locked
  the shared `fuzzy_engine` mutex and added the content to the in-memory vocabulary BEFORE commit. A failed commit
  rolled back the DB but left the vocabulary polluted (process state diverged from persisted state) and held the mutex inside a tx.
- Also readability: one 130-line function mixing dedup, merge SQL-layer call, insert tx, post-commit effects.

## Changes (src/mcp/handlers/memory_crud/create.rs only; no public rename, no file split, no schema/wire change)
- `insert_memory_transaction(ctx, &input) -> Result<Memory>`: the create tx (insert + enqueue for defer=true + audit event), local writes only.
- Vocabulary update moved to after a successful commit (fixes the leak, no mutex held in the tx).
- `mirror_existing_embedding` (deferred path HNSW mirror) and `merge_into_existing` (dedup merge) extracted as private helpers, behaviour identical.
- Batch / seed / daily / episodic / procedural / section untouched (already go through storage fns + shared helper).

## Tests (tests/mcp_protocol_tests.rs, real MCP `tools/call` dispatch; `TestHandler::with_embedder` added, `new()` delegates)
Six `c5_*` tests: response shape identical with/without immediate embedding (+ row/flag/job/HNSW); provider outage -> durable memory,
explicit pending job, health Backlogged, HNSW empty, drain converges; exact duplicate skip -> no new row/job; semantic duplicate
reject/skip/merge (409-style error with existing id, tag union, one embedding/job, content untouched, HNSW len 1);
commit failure (deferred-FK injected, defer false and true) -> no memory/job/embedding/HNSW entry/vocabulary; batch with an invalid item
+ immediate + deferred coherence.

TDD evidence
- RED: `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --test mcp_protocol_tests c5_` ->
  5 passed (pure characterization, green on pre-change code, expected) and 1 FAILED:
  `c5_commit_failure_leaves_no_memory_...`: `defer=false: a rolled-back create must not leak into the in-memory vocabulary  left: 3 right: 0`.
- GREEN after refactor: same filter + `test_deferred_create test_immediate_create`: 8 passed, 0 failed; in-crate
  `--lib create_embedding_tests`: 5 passed.

## Verification (required features)
- `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests --no-fail-fast`: exit 0, 55 binaries, 2151 passed, 0 failed, 3 ignored.
- `cargo clippy --no-default-features --features "$CI_REQUIRED_FEATURES" --all-targets -- -D warnings`: exit 0. `rustfmt --check` on touched files clean.

## Benchmark (before/after)
- `benches/mcp_dispatch.rs` group `mcp_dispatch_memory_create` (full dispatch incl. TF-IDF embed). `benches/memory_ops.rs`
  `memory_create/no_embedding` exercises storage `create_memory` only (untouched), so it was not rerun.
- Method: prebuilt before/after binaries (cargo bench --no-run, copied to scratchpad), interleaved A/B x3,
  `--sample-size 30 --warm-up-time 1 --measurement-time 4 --noplot`, private `CRITERION_HOME` (did not touch Q7's target/criterion).
- Median per run (us): before 322.6 / 317.8 / 303.9; after 345.0 / 303.2 / 520.9 (outlier, CI 394-621).
- NOISY: machine load average 6.5-7 (Q7's cargo bench / compiles concurrently active). No regression distinguishable from noise;
  expected effect is nil/slightly positive (one mutex lock removed from the transaction). Not claimed as an improvement.

## Notes / not done
- Not run: no real process-kill crash test (commit failure simulated with deferred-FK trigger; SQLite rollback proves atomicity).
- `emit_best_effort` still swallows its own errors by design (pre-existing, outside brief).
- `memory_create_batch` still does not add to the fuzzy vocabulary (pre-existing behaviour, left unchanged).
- Unlisted-in-brief files touched: none besides the progress file. A bare `cargo fmt` early on may have reformatted concurrent WIP
  files of other lanes (git showed only src/multimodal/* modified besides my test file; not staged/committed by me).
