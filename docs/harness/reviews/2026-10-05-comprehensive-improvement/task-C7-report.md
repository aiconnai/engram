# C7 report — embeddings, queue and health coherent end-to-end

Commit: f560a6c `fix(embedding): make defer_embedding enqueue and persist atomically` (local only, not pushed).
Status: DONE_WITH_CONCERNS (see Concerns).

## Owner decision applied
`defer_embedding=true` = enqueue for background processing (catalog promise kept, catalog text unchanged);
`false` = immediate embedding after commit. Before: `true` neither enqueued nor embedded (memory never embedded,
health "Healthy"); `false` embedded but left the job `pending` (re-embedded later, backlog reported falsely).

## What changed
- `src/embedding/queue/jobs.rs` (new): per-memory job lifecycle doc + `enqueue_embedding_job` (idempotent, never resurrects
  failed/complete jobs, skips memories that already have a row) + `persist_computed_embedding` (embedding row + `has_embedding`
  + job completion; must run in a tx; guards: content unchanged, job still pending/processing; idempotent).
- `create.rs`: `memory_create` enqueues inside the create transaction when `defer_embedding=true` (atomic: failed enqueue ->
  no memory); `false` -> `embed_after_commit` (provider outside tx, one persist tx, HNSW insert only after commit; failure
  logged, memory durable with pending job; response shape unchanged). Same for `memory_create_batch` (per-item defer) and
  `context_seed`. Fixes Q2-B11 (swallowed persist error + HNSW insert).
- `drain.rs`: claim tx -> provider call (no tx) -> one tx per memory via the shared helper; short provider batch -> jobs
  `failed` (was stranded in `processing`); per-memory persist failure -> only that job `failed`; `drain_pending_embeddings_observed`
  (observer after commit, used to mirror drained embeddings into the HNSW index in `server.rs`); `run_embedding_drain_cycle`
  = stale-lease recovery (15 min, retry budget 3) + drain until empty. `worker.rs` uses the same helper/tx.
- `bin/server.rs`: drain thread runs the cycle on start and every interval (was: sleep first, no recovery, no HNSW update).
- `sqlite_backend/health.rs`: new degraded diagnostics `missing_embedding_unqueued` (no row, no job) and
  `complete_job_without_embedding_row`; distinct from `flagged_without_embedding_row`, `embedding_row_without_flag`,
  orphans, pending/processing backlog, stale. `queue/health.rs`: `max_retry_count` no longer swallows SQL errors (`unwrap_or(0)`).
- Docs: `docs/OPERATIONS.md` runbook "Embedding queue backlog or stale jobs"; doc comment on `defer_embedding`
  (`crates/engram-types/src/config.rs`). No schema change, no `SCHEMA_VERSION` bump, no wire-format change.

Transition/idempotency (documented in jobs.rs): pending -> processing (claim) -> complete (persist) | failed;
processing -> pending/failed only via explicit stale-lease recovery; failed -> pending only via explicit hygiene
(`maintenance queue-hygiene --requeue-failed --apply`). One row per memory (PK memory_id); persist twice = no-op.

## TDD evidence
RED (new tests against the pre-change implementation; because a concurrent agent briefly broke the shared tree's build,
RED was captured by temporarily restoring HEAD versions of my impl files, then restoring mine):
- `rtk proxy cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --lib -- embedding::queue::tests sqlite_backend::health create_embedding_tests`
  -> 19 passed, 10 FAILED, e.g. `injected_failures...`: left job `processing`/right `failed`; `content_changed_in_flight`:
  stale embedding stored (rows 1, flag 1, complete) vs expected (0,0,pending); `health_not_healthy_when_memory_has_no_embedding_and_no_job`:
  Healthy vs Degraded; `deferred_create_enqueue_is_atomic...`: create succeeded; `immediate_create_with_failed_persistence...`:
  row written (1,0,pending) vs (0,0,pending); `skip_dedup...`/`batch_create...`: job `pending` vs `complete`.
- `--test mcp_protocol_tests -- test_deferred_create_enqueues_and_drains test_immediate_create_embeds`
  -> 2 FAILED: deferred `(0,0,None)` vs `(0,0,Some("pending"))`; immediate job `pending` vs `complete`.
GREEN: same commands after the change: lib 29 passed (initial) then queue tests 17 passed, 0 failed; mcp_protocol 2 passed.
Tests added after RED that exercise new API (green on first run): on-disk crash/reopen, drain cycle exhausted lease,
observer, enqueue idempotence, worker atomicity.

## Test cases executed (all green)
Queue (`embedding::queue::tests`): injected failure at embedding row / flag / completion (trigger RAISE) -> nothing partial,
job `failed(1)`, health Degraded, explicit requeue -> drain converges (row 1, flag 1, complete, one queue row), health
Healthy, extra drain no-op; short provider batch; content changed during provider call; crash after claim -> reopen real
on-disk DB in caller-owned tempdir -> health after reopen Backlogged (processing=1, stale=0, indexed=0), lease not expired:
cycle leaves job alone, lease expired: health Degraded stale=1, cycle recovers+drains (requeued_stale 1, drained 1), health
Healthy, second reopen shows durable converged state and a no-op cycle; retry budget exhausted -> failed; observer only for
persisted; enqueue idempotence; worker atomicity (flag failure injected, retry converges).
Health unit tests: missing-unqueued, complete-job-without-row, distinct diagnostics (flag w/o row, row w/o flag, orphan).
MCP: `test_deferred_create_enqueues_and_drains` (tools/call -> pending job + health Backlogged pending_count 1 + HNSW empty
-> drain -> row/flag/complete, health Healthy, second drain no-op), `test_immediate_create_embeds_and_completes_job`;
in-crate create tests: provider outage leaves explicit pending job then converges; persist failure -> no embedding reported
or indexed; deferred enqueue atomic with insert; skip-dedup creates no second row/job; batch (immediate + deferred + outage).

## Verification
- Required features full: `source scripts/ci-required-features.env; cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests --no-fail-fast`
  -> exit 0, 2090 passed, 0 failed, 3 ignored, 54 binaries. (First run without `--no-fail-fast` stopped at
  `server_listener_config` — 2 failures `unexpected argument '--grpc-port'` caused by the shared `target/debug/engram-server`
  being rebuilt without grpc by a concurrent session; rerun alone = 10 passed, and the full rerun above is green.)
- Clippy: `cargo clippy --no-default-features --features "$CI_REQUIRED_FEATURES" --all-targets -- -D warnings` clean; `cargo fmt --check` OK; pre-commit hook passed.
- Default-feature matrix: `cargo test --lib -- embedding::queue::tests sqlite_backend::health create_embedding_tests` 34 passed;
  `--test mcp_protocol_tests` deferred/immediate 2 passed.

## Files
src/embedding/{mod.rs, queue/{jobs.rs(new),drain.rs,worker.rs,health.rs,mod.rs,types.rs,util.rs,tests.rs}};
src/mcp/handlers/memory_crud/{create.rs,create_embedding_tests.rs(new),mod.rs}; src/storage/sqlite_backend/health.rs;
tests/mcp_protocol_tests.rs. Unlisted-in-brief but needed: src/bin/server.rs (wire cycle + HNSW observer),
crates/engram-types/src/config.rs (doc comment only), docs/OPERATIONS.md (runbook), progress file.

## Concerns / not done
- Storage-level `create_memory` unchanged: `defer_embedding=true` there still means "caller owns enqueue" (~100 internal/test
  callers, snapshot loader, dream, entity queries rely on it). Only MCP create/batch honor the enqueue promise.
- CLI `create` / `interactive` (`defer_embedding: true`, no enqueue) still produce memories with no embedding and no job;
  health now reports them Degraded (`missing_embedding_unqueued`), runbook says how to repair. Not changed (outside brief files).
- `memory_create_daily/episodic/procedural/section` unchanged (job enqueued by storage for defer=false; drained in background).
- Drain increments `retry_count` on failure and hygiene requeue increments it too, so budget 3 gives ~2 attempts (pre-existing semantics).
- Embedding `model` stored by the immediate path is now `embedder.model_name()` instead of the literal `'default'` (matches drain).
- Cannot simulate a literal process kill mid-transaction; atomicity is proven by trigger-injected failures + SQLite rollback, crash
  after claim proven by dropping/reopening the on-disk Storage.
- HNSW for drained embeddings is mirrored only in `engram-server`'s drain thread (observer); other embedders of the library must pass an observer themselves.

---
# Fix report — review round 1

Commit: 081c73f `fix(embedding): enqueue jobs for CLI, snapshot and dream creates`.

## 1. [Important] Remaining `defer_embedding: true` producers
Cause: storage `create_memory` only enqueues for `false`; four production callers passed `true` and never enqueued, so their
memories were never embedded and the new health rule kept embeddings Degraded.
Fix: each calls `enqueue_embedding_job` inside its own create transaction: `src/bin/cli/core.rs` (`create`),
`src/bin/cli/interactive.rs` (`create_note`), `src/snapshot/loader.rs` (only the one enqueue line, inside the existing tx),
`src/mcp/handlers/dream.rs` (`create_candidate_memory`, covers create/merge/supersede promotion; runs in `apply_candidate`'s tx).
Other `defer_embedding: true` hits are tests/Default-style inputs, not production create paths.

RED (before the fix, required features), 4 failures:
- `cargo test ... --lib -- snapshot::loader::tests::test_load_enqueues` -> pending count left 0 / right 1
- `cargo test ... --bin engram-cli -- create_enqueues create_note_enqueues` -> both FAILED (0 vs 1)
- `cargo test ... --test dream_integration -- promotion_enqueues` -> "promoted memory needs a pending job" left 0 / right 1
GREEN: lib `snapshot::loader embedding::queue sqlite_backend::health create_embedding_tests` 50 passed; bin 2 passed;
`dream_integration` 9 passed (new test: promotion -> pending job, health Backlogged -> drain -> Healthy).
New tests: `snapshot::loader::tests::test_load_enqueues_embedding_jobs_for_loaded_memories`, `core::tests::create_enqueues_embedding_job_instead_of_leaving_memory_unembedded`,
`interactive::tests::create_note_enqueues_embedding_job`, `test_mcp_dream_promotion_enqueues_embedding_job_and_drains`.

## 2. Minor items (all folded in)
- drain.rs doc command -> `maintenance queue-hygiene --requeue-failed --apply`.
- jobs.rs lifecycle doc: drain cycle recovers only stale `processing`; failed -> pending is explicit hygiene; added
  `update_memory` content change (any state -> pending, flag reset, invalidates old embedding).
- Health: chose option 1: `missing_embedding_unqueued` now requires `has_embedding = 0` (no double count with
  `flagged_without_embedding_row`); runbook states `maintenance rebuild --embeddings --apply` repairs all three (it re-queues every
  live memory and resets the flag). Distinct-diagnostics test updated to expect 0 for the flagged memory.
- `embed_after_commit`: `Ok(false)` now logs `warn`.
- Runbook: notes `retry_count` incremented on fail and on requeue (budget 3 ≈ 2 attempts); also notes CLI/snapshot/dream now enqueue.
- Health tests moved to `src/storage/sqlite_backend/health/tests.rs` (health.rs 854 -> 420 lines).

## Verification
- `cargo fmt --check` OK. `cargo clippy --no-default-features --features "$CI_REQUIRED_FEATURES" --all-targets -- -D warnings` clean.
- `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests --no-fail-fast`: 54 binaries, 2101 passed, 1 failed, 3 ignored.
  The one failure, `storage_posix_lock_regression_tests::mcp_paths::descriptor_alias_paths_are_refused_or_snapshotted`
  ("commit made after a foreign open/close was lost"), is in files being edited concurrently by the G1 fix
  (`tests/storage_posix_lock_regression_tests.rs`, `src/storage/active_artifacts.rs`, both uncommitted, not mine). Rerun alone
  right after: 7 passed, 0 failed, 1 ignored. Not caused by C7 (no storage connection/lock code touched).
- The pre-commit hook failed twice on a concurrent `unused_mut` in `src/storage/active_artifacts.rs` (G1, not mine); commit
  succeeded on retry once that was fixed; hook (fmt + clippy) passed on the commit.

---
# Fix report — review round 2

Commit: ec3093c `fix(storage): make embedding rebuild repair flag-without-row cases`.

Problem: runbook claimed `maintenance rebuild --embeddings --apply` repairs `missing_embedding_unqueued`,
`complete_job_without_embedding_row` and `flagged_without_embedding_row`; `rebuild_derived_indexes` only requeued
`has_embedding = 0`, so flag=1 without an `embeddings` row (with or without a complete job) stayed Degraded.

Code fix (`src/storage/queries/maintenance.rs`): new `MISSING_EMBEDDING_PREDICATE` = live memory with
`has_embedding = 0` OR no `embeddings` row. On apply (the caller's single transaction): (1) INSERT/UPSERT a `pending` job for every
such memory (resets complete/failed/stale-processing, `retry_count = 0`), then (2) set `has_embedding = 0` where the flag was 1 with no row.
Dry-run counts use the same predicate; `embeddings_missing` now includes flag-without-row, `embeddings_present = live - missing`
(shape of `RebuildReport` unchanged). Memories with flag 1 + row are untouched; orphaned rows are not repaired (stated in runbook).

RED: `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --lib -- rebuild_embeddings_repairs` ->
FAILED, `left: 1, right: 3` (only the flag=0 memory was requeued/reported).
Test `embedding::queue::tests::test_rebuild_embeddings_repairs_every_degraded_diagnostic_then_drains_healthy`: three memories
(A no row/flag/job, B flag=1 no row no job, C flag=1 + complete job no row) -> health Degraded with counters (1, 2 flagged, 1) ->
dry-run: missing 3, requeued 0, nothing changed -> apply: requeued 3, each (row 0, flag 0, one pending job), health Backlogged ->
drain 3 -> each (row 1, flag 1, one complete job), health Healthy -> second apply requeues 0, missing 0.
GREEN: same command: 2 passed (new test + existing `test_rebuild_derived_indexes`).

Runbook (`docs/OPERATIONS.md`) rewritten to match the code exactly (selection predicate, flag reset, job reset/replace,
untouched cases, orphans not repaired).

Verification: fmt OK; clippy `-D warnings` (required features, all targets) clean;
`cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests --no-fail-fast`: exit 0, 54 binaries, 2103 passed, 0 failed, 3 ignored.
Pre-commit hook passed.
