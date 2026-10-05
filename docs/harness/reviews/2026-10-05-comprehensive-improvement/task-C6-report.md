# C6 report - hooks, multimodal, sync under failure/cancellation

Status: DONE_WITH_CONCERNS. Three independent commits on claude/engram-improvement-plan-edb43d.

| Sub | Commit | Subject |
|---|---|---|
| a hooks | c601d69 | fix(hooks): bound and de-duplicate session injections and reinforcement |
| b multimodal | 437d0e0 | fix(multimodal): bound ffmpeg/vision work and own keyframe dirs |
| c sync | 909cdea | fix(sync): keep WAL streaming and cloud pulls safe across restarts |

Note: `src/hooks/failure_tests.rs` was swept by a shared-index race into C7's ec3093c. `mod failure_tests;` is wired in c601d69; at HEAD the 7 tests compile and run once (verified). History not rewritten.

## (a) Hooks
Fixed (each with RED first):
- `pending_injections::drain_for_workspace` was SELECT then DELETE: two connections received the same rows. RED: 4 threads x 400 rows delivered 1600 (expected 400). Now one `DELETE ... RETURNING`, limit `MAX_DRAIN_ROWS`=50.
- `enqueue(ttl_days=i64::MAX)` panicked (`TimeDelta::days out of bounds`); clamped to 365 days. Payload > 64 KiB now rejected (InvalidInput).
- SessionEnd replay duplicated the queue row (RED: 3 rows); `enqueue_once_for_session` is one atomic statement. Notes > 8 KiB (RED: 16384 kept) truncated on a char boundary with `notes_truncated`. SessionStart reports `remaining` (RED: Null).
- `HookManager::trigger`: a panicking handler unwound through dispatch (RED); now isolated like an Err.
- PostToolUse: 60 ids -> 60 writes on the tool-call path (RED); capped at 50. Cross-workspace ids reinforced (RED); skipped when the context names a workspace.
Semantics: queue is at-most-once; replay deduped only while queued; no exactly-once claim.

## (b) Multimodal
RED (old video.rs + old-API tests, TMPDIR-isolated): hanging ffmpeg never returns; spawn error leaked `engram_frames_*`; vision failure leaked it; ffmpeg exit 0 with no frames returned `Ok([])`. Also found by reading: every successful `memory_process_video` leaked its frames dir.
Fixed:
- `multimodal/process.rs::run_bounded`: deadline, own process group, kill group + wait on every path incl. drop, capped output. Used for ffprobe, ffmpeg, `-version`.
- `FramesDir` RAII (0700): removed on error, timeout, future cancellation and after the last clone of `VideoMemory`; `extract_keyframes` documents caller-owned transfer (`release`). Zero frames is an error. Per-frame vision timeout with frame index in error. Hash streamed. Provider HTTP clients get connect/total timeouts.
- `memory_ingest_media`: retry of same bytes/workspace returns the live memory (`deduplicated:true`), never revives a deleted one, never crosses workspaces; `asset_id` real (was stale `last_insert_rowid` after upsert). RED: second ingest created memory 2.
- pdf_worker: 5 characterization tests (SIGKILL, garbage, early exit, spawn failure, pid gone after timeout); code was already correct, no RED.
Tests verify child and grandchild pids are gone and no temp dir remains (fake ffmpeg/ffprobe shell scripts via existing `ffmpeg_bin`/`ffprobe_bin` overrides, plus new test-only `frames_base`).

## (c) Sync
- WAL streamer (C2 deferred item hit and fixed): first checkpoint with `checkpoint_seq==0` baseline never detected, new-generation frames skipped (RED: no pack). `baseline_seen` flag. Truncated WAL now `Ok(None)` not UnexpectedEof+last_error (RED).
- `CloudStorage::download`: atomic temp+fsync+rename (RED: open reader saw rewritten bytes), no temp leftovers.
- `SyncWorker`: restart clears stale `is_syncing` (RED: survived) and records the interruption; failed debounced push previously dropped the dirty marker and was never retried -> `DirtyTracker` (backoff x2, cap 300 s, 5 attempts then stop until next change); explicit `Sync` never auto-retried.
- Characterized (already correct): wrong-key non-destructive (existing cloud_tests_security), repeated upload idempotent, concurrent writers never tear the remote, failed flush does not advance offset, restart without state re-emits from frame 1 and `reset_offset` resumes.
`SyncWorker` is not started by any binary today (documented in cloud.rs); changes are to dormant code.

## Verification
- Focused: `--lib pending_injections` 14 ok; `--lib hooks` 24 ok; `--lib multimodal handlers::multimodal pdf_worker` 103 ok x3; `--test multimodal_artifact_indexing_tests --test pdf_worker` ok; `--features required,cloud --lib sync::` 71 ok (failure tests x3 stable); `--test wal_streamer_restart_tests` 4, wal_replication 10, wal_replay_hardening 32 ok.
- Full: `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests --no-fail-fast` exit 0, 2151 passed, 0 failed, 3 ignored, 55 binaries.
- `cargo clippy ... --all-targets -- -D warnings` clean for required and required+cloud; fmt clean; pre-commit hooks passed on all three commits. No orphan `sleep`/ffmpeg processes after runs.

## Not run / concerns
- Real ffmpeg/ffprobe, real vision/S3 providers: NOT RUN (offline only). Provider HTTP timeout test exercises the client helper, not a real provider.
- `screencapture` (screenshot.rs) still runs unbounded: macOS-only, binary hard-coded, not testable here. Open.
- `memory_ingest_media`, `describe_image`, `search_by_image` still `fs::read` whole files (no size cap); only video hashing was streamed. Open.
- MCP `replication_sync_now` builds a fresh streamer per call, so each call re-reports the whole WAL from frame 1 and delivers no payload; wire format unchanged, semantics only documented here.
- ConflictQueue duplicates by id and `Conflict::resolve` overwrite were reviewed, not demonstrated as defects, left unchanged.
- Hooks: SessionStart handler is not wired to drain in server.rs (SessionEnd runs policy-summary-only), so the queue fixes protect library/tests, not a live path.
- Progress file entry for C7 was committed together with mine in c601d69 (shared file).

---
# Fix report, review round 1

Commits: df2f97c fix(hooks): filter by workspace before the cap and report unknown backlog; 7abedef fix(multimodal): normalize workspace in ingest dedup and close PGID window; b3fb284 fix(sync): guard live-db pulls and record interrupted restarts. History not rewritten; the hooks sub-PR is recorded as c601d69 + df2f97c + ec3093c (failure_tests.rs).

## Important
1. Panic isolation: `HookManager` docs and the `catch_unwind` comment now say isolation holds only under panic=unwind; release profile has `panic = "abort"` where a handler panic ends the process. Profile untouched. (The original report's claim "a panicking handler no longer unwinds through dispatch" applies to unwind builds only.)
2. Ingest dedup normalizes the workspace with `crate::types::normalize_workspace` before lookup; invalid workspace returns `{"error": "Invalid workspace: ..."}`. RED: "WS-A" then "ws-a" created memory 2 (orphaning the first asset); now both return the first memory, " ws-a " too. Retry semantics (same bytes + workspace returns the existing live memory, retry's content/tags/importance ignored, `deduplicated:true`) documented in the catalog description and `docs/MCP_TOOLS.md`; `generate-mcp-reference.sh --check` green.

## Minors
- session_start: failed `pending_count` is logged and reported as `remaining: null` (`remaining_value`, unit-tested).
- post_tool_use: workspace filter before the 50-write cap (RED: 55 foreign lower ids consumed the cap, own memory not reinforced); lookups bounded at 1000.
- Dedup response returns `perceptual_hash` (computed from the bytes, since it is not stored) with the same key set as a first ingest (RED: null).
- process.rs: leader exit is detected with `waitid(WNOWAIT)`, the group is killed before the leader is reaped, so there is no PGID reuse window; Windows doc qualified; new straggler test (pipe-holding and detached children both killed, return < 3 s).
- Hash test renamed `hash_file_matches_a_known_digest_across_chunk_boundaries` (3 full chunks + tail, independent sha256); no memory claim.
- `CloudStorage::download` refuses a target with non-empty `-wal`, or `-shm`/`-journal` (RED: replaced); new `download_checked(path, &Storage)` calls `refuse_active_sqlite_artifact` (RED); `SyncWorker` Pull refuses its own connection's database (RED); parent dir fsynced after rename (best effort, logged; not unit-testable).
- Restart records the interruption even after an earlier `last_error` (appended; RED: lost). Explicit `Sync` success resets backoff via `DirtyTracker::on_explicit_sync` (unit-tested; failure only clears the marker).
- `reset_offset` and `wal_streamer_restart_tests` docs: only the frame is restored, generation identity (checkpoint seq + salts) is not; resume is safe only if the WAL was not reset while down.

## Verification
- `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests --no-fail-fast`: exit 0, 2182 passed, 0 failed, 2 ignored, 57 binaries (working tree also contains other lanes' uncommitted edits).
- `--features required,cloud --lib sync::`: 76 passed. Focused earlier: hooks 26, multimodal/handlers/pdf 133 (1 ignored), wal_streamer_restart 4.
- clippy `--all-targets -D warnings` clean for required and required+cloud; MCP reference check green; pre-commit passed on all three commits; no orphan processes.
- NOT RUN: `waitid` path on Linux (only macOS exercised here); real ffmpeg/providers.
