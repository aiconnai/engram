# Task C2 report — atomicity, WAL replay, recovery and migrations

Status: DONE_WITH_CONCERNS (see "Open concerns")
Branch: `claude/engram-improvement-plan-edb43d` (local commits only, nothing pushed)
Platform: macOS darwin/arm64 only. Linux NOT RUN locally.

## Commits (sub-PR order)

| # | SHA | Subject |
|---|-----|---------|
| 1 | `1398258` | fix(sync): bound WAL replay by byte budget before touching target |
| 2 | `ba07ff3` | feat(sync): verify SQLite WAL checksum chain before replay |
| 3 | `d1a9840` | fix(sync): stage WAL recovery and replace target atomically |
| 4 | `5c113f0` | fix(storage): serialize migrations and wait for concurrent writers |
| 5 | `f2f6e7a` | test(snapshot): guard bounded entry reads and record C2 lessons |

No Co-Authored-By or AI trailers (per the user's commit-format rule). The pre-commit hook (`cargo fmt --all --check` + `cargo clippy --all-targets --all-features -D warnings`) passed on every commit, and `cargo clippy --all-targets --all-features -- -D warnings` was re-run explicitly at the end: exit 0.

## What was implemented

### 1. Preflight and byte budget (`src/sync/wal_replay_guard.rs`, new)
- `ReplayLimits { max_target_bytes }`. The default is the approved 64 GiB (`DEFAULT_MAX_REPLAY_TARGET_BYTES = 64 * 1024^3`) and the same value is also the ceiling. `validate()` refuses 0 and anything above the ceiling. There is no "unlimited" option.
- `preflight_replay(page_size, frames, limits)` is pure (no I/O). It checks:
  - page_size is a power of two in 512..=65536;
  - frame payload length equals page_size;
  - `page_number` is in 1..=`MAX_ALLOWED_DB_PAGES`;
  - `db_size_pages` is at most `MAX_ALLOWED_DB_PAGES`;
  - checked `(page-1)*page_size + len`, `db_size_pages*page_size` and cumulative frame bytes all stay within the budget (overflow counts as a refusal).
- `WalRecoveryEngine::{replay_frames_to_db, recover_from_delta_packs, point_in_time_recovery}` keep their signatures and apply the default limits. New `*_with_limits` variants take injected limits.
- All frames are validated before the target or its parent directory is created or opened. Packs are unpacked and validated before the base is copied. PITR extracts and validates the WAL before copying the source.
- New error variant `WalReplicationError::ReplayBudgetExceeded`.

### 2. Checksum chain (`src/sync/wal_chain.rs`, new)
- Uses SQLite semantics: the magic's low bit selects checksum word order, and stored checksum fields are always big-endian. The header checksum seeds frame 1.
- `WalDeltaPack.chain: Option<WalChainContext { magic, seed }>` and `WalDelta.chain_seed` are new fields, both `#[serde(default)]`.
- `validate_pack_chain` checks, before replay:
  - the chain context is present (pre-C2 packs are refused);
  - page_size is the same across packs;
  - `frame_count`, `start_frame` and `end_frame` match the payload, and frame indices are contiguous;
  - frame salts equal the pack salts;
  - the chain verifies inside each pack, and each pack's seed equals the previous pack's tail within one generation;
  - a new generation restarts at frame 1 with a higher `checkpoint_seq` and different salts.
- `validate_frame_order` (strictly increasing `frame_index`, uniform salts) applies to the direct `replay_frames_to_db` API.
- `WalDeltaReader::extract_delta_frames_from_bytes` now verifies the header checksum (an invalid one returns `InvalidHeader`). It stops at the first frame with a salt or chain mismatch, which is SQLite's valid-prefix semantics, and reports the chain seed. Verified against WAL files written by real SQLite.
- Docs and comments state that this is integrity, not authenticity: anyone can recompute both the SHA-256 and the chain.

### 3. Staged recovery (`src/sync/wal_recovery.rs`, `src/sync/wal_recovery_staging.rs`, new)
- The recovery engine moved out of `wal_replication.rs`, which dropped from 1300 to 984 lines. Public paths are unchanged because the items are re-exported.
- `StagedTarget` steps:
  - create a `.<name>.engram-replay-<pid>-<nanos>-<n>.tmp` file (create_new) in the target's directory;
  - seed it from the base or source, or from the existing target for in-place replay semantics;
  - apply frames through a `PageSink` (a seam for late I/O failure injection in tests);
  - fsync, then run `integrity_check` on the staged file;
  - rename over the target, then fsync the parent directory (Unix).
- Drop removes the staging file and its `-wal`/`-shm`/`-journal` side files. Any failure before the rename leaves the target, base and source byte-identical (asserted in tests).
- PITR without a WAL now goes through the same staging path and really runs `integrity_check`. It used to report `"ok"` unconditionally.
- A target with a non-empty `-wal` or `-journal` is refused, because SQLite would apply that file over the replacement.

### 4. Migrations and concurrency (`src/storage/migrations/mod.rs`, `src/storage/connection.rs`)
- Gaps found by new tests:
  - concurrent `run_migrations` failed with `UNIQUE constraint failed: schema_version.version`, `duplicate column name`, or `table dream_candidates_new already exists`;
  - a failing step left partial DDL behind;
  - `with_transaction` failed instantly with "database is locked" when another writer held the lock;
  - `PRAGMA journal_mode` intermittently returned SQLITE_BUSY during concurrent first opens.
- Fixes:
  - **Migration runner:** the 48 if-blocks became a `MIGRATIONS` table. Each step runs in `BEGIN IMMEDIATE` and re-reads `MAX(version)` under the lock, then commits, or rolls back on error. The newer-version refusal is kept. The runner refuses to run inside an open transaction, and the version read now propagates errors (it used to `unwrap_or(0)`, which would re-run from v1).
  - **v45:** its SQL toggles `PRAGMA foreign_keys`, which does nothing inside a transaction. The runner now turns FK off before `BEGIN` for that step and restores the previous value afterwards. This is SQLite's documented table-rebuild procedure and keeps v45's no-cascade behaviour.
  - **`with_transaction`:** now uses `TransactionBehavior::Immediate`.
  - **Journal mode:** `busy_timeout` is set first, and the journal-mode switch retries on BUSY/LOCKED within 30 s with backoff from 5 to 100 ms.
- Guards that confirm current behaviour (they passed before any change):
  - a slow provider does not hold the connection: a blocked mock embedder inside `memory_create`, while a second writer on the same Storage finishes within a 10 s bound;
  - a second open is a no-op and preserves data;
  - a newer schema is refused without changes;
  - IDs are not reused after a rollback or a hard delete (AUTOINCREMENT).

### 5. Snapshot
`read_entry_limited` boundary tests with tiny limits: exactly at the limit is accepted, limit+1 is refused, and an endless stream is cut at limit+1 without being drained. The existing 256 MiB per-entry guard and scenario_11 already covered the bomb case. This adds boundary regressions only, and no code change was needed.

## Files changed
- New: `src/sync/wal_replay_guard.rs`, `src/sync/wal_chain.rs`, `src/sync/wal_recovery.rs`, `src/sync/wal_recovery_staging.rs`, `tests/wal_replay_hardening_tests.rs`, `tests/wal_recovery_staging_tests.rs`, `tests/storage_concurrency_regression_tests.rs`, `docs/harness/progress/2026-10-05-improvement-lane-r.md`.
- Modified: `src/sync/mod.rs`, `src/sync/wal_replication.rs`, `tests/wal_replication_tests.rs` (one literal gains `chain_seed: None`, no assertion changed), `src/storage/migrations/mod.rs`, `src/storage/migrations/tests.rs`, `src/storage/queries/tests/migrations.rs`, `src/storage/connection.rs`, `src/snapshot/loader.rs`, `ERRORS_AND_LESSONS.md`.
- Outside the brief's file list:
  - `src/storage/connection.rs`: the IMMEDIATE transaction and journal-mode retry, both required by failing tests;
  - `src/sync/mod.rs`: module wiring;
  - the new modules and test files: `wal_replication.rs` was already over 800 lines;
  - `ERRORS_AND_LESSONS.md`: the CLAUDE.md post-task rule.
- `docs/harness/progress.md` and `SPEC.md` were not touched.

## TDD evidence (argv prefix for every run: `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES"`)
- **Sub-PR 1:** RED `--test wal_replay_hardening_tests` gave 1 passed, 14 failed. Examples:
  - `write beyond budget: ReplayPreflight { max_write_end_bytes: 0, .. }` (the stub accepted it);
  - `assertion failed: matches!(result, Err(InvalidFrame { index: 2, .. }))`;
  - PITR `assertion failed: matches!(result, Err(ReplayBudgetExceeded(_)))`.

  The stubbed API ignored the limits, and every I/O test used tiny sizes. GREEN: 15/15, plus `wal_replication_tests` 10/10.
- **Sub-PR 2:** RED gave 17 passed, 13 failed. Examples:
  - tampered checksum with recomputed SHA: `expect_err("chain break refused")` got Ok;
  - reader `chain_seed` was None;
  - streamer `pack.chain` was None.

  GREEN: 30/30 plus 10/10. The real-SQLite WAL chain verified.
- **Sub-PR 3:** RED `--test wal_recovery_staging_tests` gave 2 passed, 5 failed: `assert_sentinel` "pre-existing target must be untouched", corrupt no-WAL source returned Ok, and the hot side file did not block. RED `--lib sync::wal_recovery_staging` gave 1 passed, 3 failed: late write/sync failures modified the target. GREEN: 7/7 integration, `--lib sync::` 26/26.
- **Sub-PR 4:**
  - RED `--test storage_concurrency_regression_tests` gave 4 passed, 3 failed: `concurrent open failed: Err("Database error: database is locked")`; `table dream_candidates_new already exists`; `writer returned while the lock was held (should wait): Err("Database error: database is locked")`.
  - Migration RED, with the HEAD `migrations/mod.rs` swapped in temporarily via a scratch copy (no stash): `--lib storage::queries::tests::migrations` gave 2 passed, 2 failed: "failed step must be rolled back entirely"; `assertion failed: run_migrations(&conn).is_err()`.
  - Journal-mode RED, found while checking for flakiness: 5 of 15 repeated runs failed, on `test_concurrent_first_open...` and `test_concurrent_open_switching_existing_db_to_wal_succeeds` with "database is locked".
  - GREEN: 7/7 in 20 consecutive runs.
- **Snapshot guards:** green on the first run, as expected, since they confirm an existing limit.

## Other verification
- `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests` (all 52 test binaries, run before the journal-mode retry and the snapshot tests): exit 0, 1979 passed, 0 failed, 1 ignored.
- After the final changes:
  - `--lib`: 1560 passed, 0 failed, 1 ignored;
  - `storage_concurrency_regression_tests` 7, `wal_recovery_staging_tests` 7, `wal_replay_hardening_tests` 30, `wal_replication_tests` 10, `snapshot_attestation` 11, `watch_miner_tests` 1: all ok.
- `cargo clippy --all-targets --all-features -- -D warnings`: exit 0. `cargo fmt --all -- --check`: exit 0.
- No sparse or large files were created. MAX/MAX+1/u32::MAX cases go only through the pure `preflight_replay`, and filesystem tests use budgets of 4 to 64 pages of 512 B.

## Deviations and design decisions (flagged)
1. **Migration serialization:** this needed a design choice. I implemented per-step `BEGIN IMMEDIATE` plus a version recheck, with FK disabled outside the transaction for v45 only. Effects:
   - every step is now atomic;
   - migrations run with the caller's FK setting, unchanged except v45;
   - raw `run_migrations` callers no longer get v45's side effect of leaving `foreign_keys=ON` on the connection.
2. **`with_transaction` is now IMMEDIATE globally.** Read-only closures inside `with_transaction` now take the write lock (cross-process contention). In-process behaviour was already serialized by the Mutex.
3. **Staging and atomic replace:** the conservative option.
   - Rename is atomic on POSIX on the same filesystem. On Windows, `MoveFileExW(REPLACE_EXISTING)` is not documented as atomic.
   - A crash before the rename leaves an orphan `.<name>.engram-replay-*.tmp` file, which is safe to delete.
   - If the parent-directory fsync fails after the rename, a warning is logged and replay still succeeds (the replacement has already happened).
   - The target must not be open by another process. A hot `-wal`/`-journal` is refused.
   - Copying a *live* PITR source DB file is still not snapshot-consistent; this is pre-existing and documented.
4. **Pre-C2 `WalDeltaPack`s** (no `chain`) are now refused at replay. This is a compatibility break for any persisted old packs.
5. **PITR from a raw WAL** stops at the first invalid frame (SQLite semantics) instead of refusing. Packs and the direct frame API refuse.
6. **No-WAL PITR** now reports `integrity_check: "skipped"` when `verify_integrity=false`. It used to always say `"ok"`.
7. **`total_frame_bytes` budget:** the cumulative frame payload is bounded by the same byte budget. That budget also bounds the in-memory frames loaded from many packs.

## Open concerns
- **Existing naming bug, not fixed (it would break an existing test's assertion):** `WalHeader::parse` treats `0x377f0683` as "little-endian" and decodes the stored checksum fields as LE. `WalHeader::new` computes the header checksum with BE words for `0x377f0682`. Both are the inverse of SQLite. The chain code normalizes around this (`stored_checksum`, `checksum_words_le`). Recorded in ERRORS_AND_LESSONS; follow-up recommended.
- **`INVARIANTS.md` #25** still mentions only the page cap. It should also cite the 64 GiB byte budget and the chain validation. Not edited (outside the file list).
- **Migration atomicity is verified for v48 only** (failure injected by trigger). Other steps rely on the same runner. There is no backup/restore step because no destructive schema change was made.
- **Not covered:** "cancelamento" (sync API, no async cancellation point in these paths) and checkpoint/recovery interleaving beyond the existing streamer tests.

## NOT RUN
- Linux test run (macOS only locally; CI covers Linux).
- Windows rename semantics.
- Any network, cloud or production work (none required).

---

# Fix report — review round 1 (base HEAD f2f6e7a)

## Commits
- `c74f1c0` fix(sync): owner-only staging and running decode budget
- `a9c3bd8` fix(storage): roll back failed migration commits, check v45 FKs
- `7a7809f` docs(sync): record WAL replay budget and C2 API changes
- `46fc3e2` docs(harness): log C2 review fix round in lane R progress

Only explicit paths were committed. The Q2 files (`scripts/rust_risk_inventory*.py`) were left untouched and untracked.

## Changes, with test-first evidence
All runs used the argv prefix `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES"`.

1. **[Important] Staging file mode.** `staging_open_options()` creates the staging file with `create_new` + `OpenOptionsExt::mode(0o600)` on Unix (`STAGING_FILE_MODE`).
   - Tests: `test_pitr_replaced_target_is_owner_only` and `test_pack_recovery_replaced_target_is_owner_only` in `tests/wal_recovery_staging_tests.rs`.
   - RED (`--test wal_recovery_staging_tests owner_only`): 0 passed, 2 failed, `left: 420 right: 384` (0644 vs 0600).
   - GREEN: 9/9.
2. **[Important] v45 FK safeguard on the production path.**
   - The v44 fixture was extracted to `V44_DREAM_FIXTURE`. New test `test_v45_preserves_sources_with_foreign_keys_on` sets `PRAGMA foreign_keys=ON` before `run_migrations`, then asserts:
     - the candidate and its `dream_candidate_sources` row survive;
     - `PRAGMA foreign_keys` reads 1 afterwards;
     - `PRAGMA foreign_key_check` returns no rows.
   - No `dream_candidate_reviews` table exists in the schema (grep: only `dream_candidate_sources` references `dream_candidates`), so sources are the cascade-relevant rows.
   - Safeguard proof: with `FOREIGN_KEYS_OFF_DURING` temporarily emptied (`[i32; 0] = []`, then restored from a scratch copy, empty diff confirmed), the new test FAILED with "rebuild of dream_candidates must not cascade-delete sources" (`left: 0, right: 1`). The existing `test_v45_preserves_existing_dream_candidate_data` also failed. With the safeguard restored, both pass.
3. **[Spec gap] Running decoded-size budget.**
   - `WalDeltaPack::unpack_frames_counted(max)` returns the frames plus the decoded length, or `None` when the decoded payload would exceed `max`. Decoding stops at max+1. Uncompressed payloads are now measured too; before, uncompressed payloads had no cap.
   - `unpack_within_budget` in `wal_recovery.rs` keeps a checked running sum. Each pack may decode at most `min(remaining budget, 64 MiB)`. When the remaining budget is the binding cap it returns `ReplayBudgetExceeded`; when the per-pack 64 MiB cap binds it returns `DecompressionFailed`.
   - Tests (`tests/wal_replay_hardening_tests.rs`): `test_recover_refuses_cumulative_decompressed_bytes_over_budget` uses 3 packs × 1 frame and a 4 KiB budget. Frame bytes and write extents fit, but the decoded payloads do not. `..._within_budget` is the positive control at 16 KiB.
   - RED: the over-budget test got `Ok(RecoveryReport { frames_replayed: 3, .. })`.
   - GREEN: 32/32.
4. **[Minor]**
   - **Side-file check fails closed.** It now refuses on any stat error other than NotFound. Unit test `test_side_file_check_fails_closed_on_unreadable_metadata` uses a parent path component that is a regular file, giving NotADirectory. RED: `must fail closed: ()`. GREEN.
   - **Migration COMMIT failure.** `run_step_in_transaction` now issues ROLLBACK whenever the outcome is an error and the connection is not in autocommit, which covers a failed COMMIT. Test `test_commit_failure_rolls_back_and_leaves_no_open_transaction` uses a deferred FK violation, so COMMIT fails. RED: "failed COMMIT must be rolled back". GREEN.
   - **FK check for v45.** Steps listed in `FOREIGN_KEYS_OFF_DURING` count `PRAGMA foreign_key_check` rows before and after the step. They refuse *new* violations before committing; pre-existing violations do not block an upgrade. Test `test_fk_off_step_refuses_new_foreign_key_violations` asserts the step is rolled back, the version stays at 44, and `foreign_keys` is restored to 1. RED: "new FK violation must abort the step". GREEN.
   - **Specific error variants.** The 4 late-failure tests now go through `assert_not_a_database` (`WalReplicationError::Sqlite(SqliteFailure { code: NotADatabase })`). The hot-side-file test matches `RecoveryError` containing the suffix.
   - **Docs.** INVARIANTS.md #25 was rewritten (64 GiB budget, pre-target validation, decoded-payload accounting, chain check with integrity ≠ authenticity, staged replacement). The CHANGELOG `[Unreleased]` section now notes the new pub fields `WalDelta::chain_seed` / `WalDeltaPack::chain`, the new `ReplayBudgetExceeded` variant, pre-change packs being refused (fail-closed), and the uncompressed 64 MiB cap.

## Verification
- `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests` (full suite, after all code changes; includes the journal-mode retry for every `Storage::open`): exit 0. 1989 passed, 0 failed, 1 ignored, 52 test binaries.
- `--lib -- storage:: sync::`: 347 passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: exit 0, "No issues found". `cargo fmt --all -- --check`: exit 0.
- The pre-commit hook passed on every commit.

## Follow-ups recorded (not fixed, per coordinator)
- **Checksum endianness naming bug:** `WalHeader::parse`/`new` invert SQLite's word order. Normalized only in `wal_chain`.
- **Streamer reset:** `WalReplicationStreamer::flush_delta` only resets `last_replicated_frame` when `last_checkpoint_seq != 0`. A WAL reset while `nCkpt == 0`, or the first observed generation, can therefore skip the reset.

## Notes
- **Behavior change:** uncompressed packs larger than 64 MiB decoded are now refused (this was previously unbounded).
- Linux was NOT RUN locally (macOS only).
