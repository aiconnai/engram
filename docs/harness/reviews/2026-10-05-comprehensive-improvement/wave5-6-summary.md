# Review summary — Engram comprehensive improvement program (post-merge waves 5–6)

| Field | Value |
|---|---|
| Program | `engram-comprehensive-improvement` (Waves 5–6 post-merge hardening & stabilization) |
| Date | 2026-10-14 |
| Base PR | PR #234 squash-merged (`26b40d3`) |
| Base review record | [`README.md`](./README.md) |
| Wave 5 commits | `661b140`, `1ad6586`, `336ef07`, `dd3b6ae`, `e17fb5e` |
| Wave 6 scope | SDK/server drift, error parity, parameter normalization, SQLite read path, benchmarks, owner sign-offs |
| Review nature | Independent SDD subagent reviews & verification reconciliation; no forged receipts |

## Executive overview

This document records the verification, implementation, and reconciliation of post-merge
Waves 5 and 6 of the Engram comprehensive improvement program.

On 2026-10-05, the waves 0–4 improvement program concluded with a whole-branch independent
review (Opus) and a FINALFIX wave documented in [`README.md`](./README.md).
The review surfaced several owner decisions, follow-up issues (#235–#241), and contract
alignments requiring post-merge execution.

PR #234 was subsequently squash-merged as commit `26b40d3`, cleanly resolving the mixed commit
hygiene items (`35a1592` and `ec3093c`). Following the merge, Waves 5 and 6 implemented and verified
all follow-up issues, performance optimizations, and contract parity requirements.

Per [`WHAT_WE_DONT_DO.md`](../../WHAT_WE_DONT_DO.md) and harness invariant 10, no forged `REVIEW_VERDICT: PASS`
markers are placed in this record; verification is substantiated by deterministic offline lanes,
contract tests, and explicit owner sign-offs.

## Wave 5 — Follow-up issues (#235–#241) and post-merge hardening

Wave 5 implemented the follow-up issues identified during the comprehensive improvement review:

1. **`661b140` — Security patches, doctor fix, and standard library migration**:
   - Upgraded `quinn-proto` to `0.11.18` to resolve 5 RustSec advisories (`RUSTSEC-2024-0425` through `0429`).
   - Fixed GNU Bash 5.3.15 here-string deadlock in `docs/harness/bin/doctor.sh`.
   - Migrated deprecated `once_cell::sync::Lazy` to standard library `std::sync::LazyLock` across 7 intelligence and context modules.

2. **`1ad6586` — Storage isolation & bounded reads (Issues #235 & #236 pt. 1)**:
   - **#235**: Scoped `media_assets` by workspace via migration `v49`, adding `UNIQUE(workspace, file_hash)` compound key and preventing cross-workspace asset collisions.
   - **#236**: Enforced bounded 50 MiB media reads in `src/mcp/handlers/multimodal.rs` via `read_media_file_bounded`.
   - **Invariant 27**: Enforced Invariant 27 on `CloudStorage::upload` and `SyncWorker` using `VACUUM INTO` snapshots, closing raw database file reads on live SQLite stores.

3. **`336ef07` — Dependency cleanup, bounded storage & standing checks (Issues #236 pt. 2, #239, #241)**:
   - **#236**: Extended 50 MiB bounded file read limits to `src/storage/image_storage.rs`.
   - **#239**: Upgraded `aws-sdk-s3` and `aws-config` with an explicit `BehaviorVersion::v2026_01_12()` pin, eliminating transitive unmaintained dependency `lru 0.16.4` (`RUSTSEC-2026-0253`).
   - **#241**: Pinned all workflow image references by SHA256 digest in [`docs/security/supply-chain-pins.toml`](../../../security/supply-chain-pins.toml) (`unpinned_images = []`).
   - Configured O4 standing-checks ownership to `Ronaldo` in `docs/harness/goals/registry.json`.
   - Extracted dedicated test modules (`multimodal_tests.rs`, `chain_tests.rs`, `connection_tests.rs`) to improve maintainability.

4. **`dd3b6ae` — Server wiring, macOS WAL robustness & budget anchoring (Issues #237, #238, #240 pt. 1)**:
   - **#237**: Integrated `SessionStartHandler` (queue drain on `INITIALIZE`), workspace-scoped `PostToolUse` (`CALL_TOOL`), and background `SyncWorker` into `src/bin/server.rs`.
   - **#238**: Added `SQLITE_READONLY` `-shm` open probe (`BEGIN IMMEDIATE; ROLLBACK;`) with bounded retry under `DATABASE_OPEN_LOCK` in `src/storage/connection.rs` to prevent transient macOS concurrency errors.
   - **#240**: Anchored candidate hot-path budgets against `--floors-anchor` in `scripts/quality_candidate/criterion.py` and `runner.py`.

5. **`e17fb5e` — Supply-chain patch, benchmark isolation & floor sealing (Issue #240 pt. 2)**:
   - Upgraded `rustls` 0.23.45 to 0.23.46 (`RUSTSEC-2026-0346`..`0349`), with governed exceptions for Turso/libsql dependencies.
   - Set `autobins = false` in `Cargo.toml` and reorganized `src/bin/server/tests.rs` to avoid bencher argument collisions during `cargo bench`.
   - **#240**: Sealed [`docs/quality/candidate-floors.json`](../../../quality/candidate-floors.json) with owner `Ronaldo` independent review acceptance.

## Wave 6 — Contract parity, performance & owner sign-offs

Wave 6 addressed systemic SDK and server drift, performance characterization, and architectural rulings:

- **SDK/server drift elimination (`G-2`)**:
  - Eliminated all 161 drift entries between the MCP server tool definitions and client SDK declarations, establishing exact 0-drift parity (`known_drift: []` in [`docs/quality/sdk-contract-drift-baseline.json`](../../../quality/sdk-contract-drift-baseline.json)).
- **TypeScript SDK error & timeout parity (`G-4`, `G-5`)**:
  - Aligned `isError: true` decoding (`G-4`) and timeout/network error wrapping (`G-5`) in TypeScript `EngramClient.rpcCall` to raise `EngramError(message, code, details)`.
- **MCP parameter and error normalization (`G-6`, `G-7`, `G-10`)**:
  - Validated wrong-typed `id`, `limit`, and `offset` parameters as `ToolError::invalid_params` (`invalid_params`) across `read_update_delete.rs` and `search.rs` (`G-6`).
  - Upgraded legacy string error payloads in `blocks.rs`, `compression.rs`, and `dream.rs` to RFC 0006 structured `ToolError` objects and added `src/mcp/error_normalizer.rs` (`G-7`, `G-10`).
- **SQLite read-path permission check & `EntityExtractor` hot-path optimization (`G1` / `P1`)**:
  - Sampled `sqlite3_total_changes64` before/after `with_connection` in `src/storage/connection.rs` to skip redundant `-wal`/`-shm` `lstat` syscalls on read-only queries (`memory_get/by_id` improved to `15.181 µs`, +17.1%).
  - Added `readers_readonly_{1,4,8}` concurrency benchmarks in `benches/memory_ops.rs` (`readers_readonly_8` at `2.94 ms` / `544.8 Kelem/s`, 226× faster than write-serialized `readers_8`).
  - Eliminated per-instance `HashSet`/`Regex` cloning in `EntityExtractor` (`entity_extractor_new/default` at `1.82 ns`, `entity_extraction/extract_mixed` at `4.50 µs`, 4.75× faster).
- **Owner architectural sign-offs**:
  - Recorded in [`docs/decisions/2026-10-14-default-features-loopback-and-chunking-signoff.md`](../../../decisions/2026-10-14-default-features-loopback-and-chunking-signoff.md):
    - **D7 (`default` features)**: Confirmed `default = ["openai"]` with `cloud` as explicit opt-in.
    - **C1 (workspace authorization)**: Confirmed denying the 65 by-ID unscoped tools to restricted principals (including anonymous loopback HTTP).
    - **#152 (ingestion chunk bounds)**: Confirmed character-based chunking in `document_ingest.rs`.
  - **RFCs 0009–0011**: Promoted to `accepted (implemented)`:
    - [RFC 0009 (MCP Contract Validation)](../../../rfcs/0009-mcp-contract-validation.md)
    - [RFC 0010 (Permission Modes)](../../../rfcs/0010-permission-modes.md)
    - [RFC 0011 (Model Routing)](../../../rfcs/0011-model-routing.md)

## Verification status

All items were verified locally and conform to harness requirements:
- Offline harness lane: `OFFLINE_LANE: PASS components=12 checks=529`.
- Quality contract tests: deterministic test suite passes without skips.
- Security gate: zero unpinned images in `supply-chain-pins.toml`; vulnerability audit clean.
- Cross-references and links verified via `docs/harness/bin/check-doc-links.py`.
