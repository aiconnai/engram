# Test coverage map (Q4)

Status: inventory of which invariant or contract is proved by which test, how
strong that proof is, and which CI lane runs it. It is a map, not a gate and not a
coverage number. Nothing here authorizes weakening a test.

| | |
|---|---|
| Date | 2026-10-05 |
| Scope | MCP tool surface, transports, workspace/permission authorization, persistence journeys, Python and TypeScript SDKs |
| Not in scope | Unicode/truncation parsers (task C3), retrieval quality scoring, WAL replication internals (cited only) |

## 1. Vocabulary

**Kind** says what a test can and cannot prove. Never read a row as stronger than
its kind.

| Kind | Meaning | Cannot prove |
|---|---|---|
| `unit/mock` | A stub or mocked HTTP client (a stub server on a real socket still counts: the server side is fake) | Anything about the real `engram-server` |
| `in-process` | The library API (`dispatch`, `Storage`, handlers) in the test process, on an in-memory or temp-file database, no server process | Process lifecycle, transports, anything the binary wires differently |
| `protocol` | A real JSON-RPC `tools/call` through `McpHandler`/`dispatch`, decoding the same `ToolCallResult` envelope the server builds, but in-process | That `engram-server` wires it identically (the handler is a mirror) |
| `real` | A real `engram-server` child process (stdio and/or HTTP/gRPC) on an isolated database, driven over its transport | Anything about a published package |
| `static` | Parses source/docs/registry; no execution of the system under test | Runtime behavior |
| `live` | An installed wheel or packed tarball driving a real local `engram-server` | Production, real providers, other platforms |

**Lane** says when it runs.

| Lane | Where | Blocks a PR |
|---|---|---|
| R | `Test (ubuntu-latest)` in `.github/workflows/ci.yml`: `cargo nextest run --no-default-features --features "$CI_REQUIRED_FEATURES" --lib --tests --bin engram-server --bin engram-watcher` (features in `scripts/ci-required-features.env`) | yes |
| A | Path-filtered PR workflows `python-sdk-live.yml`, `typescript-sdk-live.yml` (paths include `sdks/**`, `src/**`) | not one of the four required checks |
| O | Scheduled / push-to-main / dispatch: full-feature tests, property and golden jobs, `coverage`, nightly fuzz/mutation | no |
| M | Manual dispatch: `sdk-release.yml` (`verify-sdk-artifacts.sh`), release verification scripts | no |
| L | Exists, runs locally, is not wired into any workflow | no |

`scripts/test-canonical-journey.sh` is a local wrapper (lane L) around a test target
that also runs in lane R.

## 2. What "coverage" means here

- **Executed coverage** exists in exactly one place: the `coverage` job in
  `ci.yml` (`cargo llvm-cov --features "$CI_FEATURES"`, `--fail-under-lines 55`),
  lane O (schedule/dispatch). It was NOT run for this task; no percentage is
  claimed anywhere in this document.
- Counting files that contain `#[test]` is not coverage and is never used as one.
- A row below means "this test asserts this behavior". It does not mean "this code
  path is covered".

## 3. Map: invariant or contract to test

Paths are under `tests/` unless stated. "Features" lists non-default Cargo features
the test needs; required-set features are always enabled in lane R. A feature that
is not in the required set means lane R does not run that test.

### 3.1 Authentication and transport

| Contract | Tests | Features | Lane | Kind |
|---|---|---|---|---|
| Invalid, missing or malformed bearer is rejected before dispatch (HTTP 401, JSON-RPC -32001), on `/mcp` and `/v1/mcp`, and the rejected body would otherwise mutate | `canonical_journey::contract::rejected_requests_never_mutate_shared_state_across_stdio_and_http`; `canonical_journey::canonical_real_binary_journey_over_stdio_and_authenticated_http` (wrong bearer); `http_transport_security::public_http_with_key_authenticates_mcp_and_sse` | none | R | real |
| gRPC missing/malformed/wrong token is `UNAUTHENTICATED` | `grpc_transport` scenarios e, h, i | `grpc` | R | real (in-process tonic server) |
| Public bind without a key refuses to start | `http_transport_security::public_http_without_key_exits_before_listening`; `grpc_transport::scenario_q_*` | `grpc` for gRPC | R | real |
| Request timeout bounds slow body and SSE setup with a stable response | `http_transport_security::http_request_timeout_bounds_slow_body_setup_with_stable_response`, `http_timeout_bounds_sse_setup_but_not_established_stream` | none | R | real |
| Body size limit rejects limit+1 before parsing | `http_transport_security::http_body_limit_*`, `http_auth_rejects_oversized_body_*` | none | R | real |
| Malformed JSON body is rejected by the transport (HTTP 400) before dispatch | `canonical_journey::contract::*` (HTTP phase) | none | R | real |
| Both SDKs' endpoint `/v1/mcp` exists and authenticates like `/mcp` | `canonical_journey::contract::*` | none | R | real |
| Trusted-proxy forwarding cannot be spoofed | `http_transport_security::trusted_proxy_*` | none | R | real |
| Transport security matrix fixture is internally complete | `transport_security_contract` + `scripts/validate_transport_security_contract.py` | none | R (Rust), L (script) | static |

### 3.2 Workspace and permission authorization

| Contract | Tests | Features | Lane | Kind |
|---|---|---|---|---|
| A workspace claim does not authorize a foreign memory ID; foreign and absent IDs are indistinguishable; no side effects | `workspace_auth_enforcement_tests::{test_claimed_workspace_does_not_authorize_foreign_id, test_foreign_id_mutations_denied_without_side_effects, test_handler_checks_workspace_inside_mutation_transaction}` | required set | R | in-process (shared connection, so its before/after row checks are valid) |
| Restricted principals are denied by-ID tools that cannot honor a workspace claim (`tool_not_workspace_scoped`); context alias tools denied; metadata tools allowed | `workspace_auth_enforcement_tests::{test_restricted_principal_cannot_scope_unscoped_tool_with_claim, test_workspace_alias_context_tools_denied_for_restricted_principal, test_catalog_metadata_tools_allowed_for_restricted_principals}` | required set | R | in-process |
| Same, through the real HTTP transport, loopback anonymous principal | `http_transport_security::workspace_auth::*` | none | R | real (raw-row "unchanged" checks carry a positive control since the G-1 fix; they were vacuous before it) |
| Same, through gRPC | `grpc_transport` scenarios j-p2, r | `grpc` | R | real |
| Cross-tenant: stdio has no principal (owner), HTTP-with-key is admin; cross-tenant isolation exists only for anonymous loopback and namespaced tokens, so Q4 adds no transport here. `X-Tenant-Slug` is not an authorization boundary | `canonical_journey::contract::*` (header pin); `docs/security/workspace-operation-matrix.md` | none | R | real |
| Permission modes: env mode, per-call `_permission_mode`, hierarchy, every dispatchable name has a mode | `permission_modes_tests::*` | required set | R | in-process |
| Per-call read-only override denies mutation and leaves the row unchanged | `canonical_journey::contract::rejected_requests_never_mutate_shared_state_*` (stdio + HTTP, raw rows read after process exit); `mcp_protocol_tests::contract_matrix::per_call_permission_override_returns_normalized_denial` | none | R | real / protocol |
| Unknown tool, no principal and no env mode: `tool_not_found` | `mcp_protocol_tests::contract_matrix::unknown_tool_without_principal_is_tool_not_found`; `canonical_journey::contract::unknown_tool_behavior_follows_the_environment_permission_mode` | none | R | protocol / real |
| Unknown tool under a restricted principal or env mode below admin: `permission_denied` (names are not revealed); admin: `tool_not_found` | `contract_matrix::{unknown_tool_for_restricted_principals_is_permission_denied, unknown_tool_for_unrestricted_admin_principal_is_tool_not_found}`; `canonical_journey::contract::unknown_tool_behavior_follows_the_environment_permission_mode` (read_only, scoped_write, admin, none) | none | R | protocol / real |

### 3.3 MCP protocol and envelopes

| Contract | Tests | Features | Lane | Kind |
|---|---|---|---|---|
| Protocol negotiation 2025-11-25 and legacy 2024-11-05; tools/list annotations; resources; prompts | `mcp_protocol_tests` (pre-existing 56) | `attestation` tests only in O | R | protocol (handler is a mirror; its `tools/call` never sets `isError`) |
| Tool failures travel in the result with `isError: true`, one text block, JSON text; success omits `isError` | `contract_matrix::{success_envelope_has_no_error_flag_and_text_json, invalid_params_return_normalized_envelopes_with_is_error}` | none | R | protocol |
| Normalized error object: `code` in the RFC 0006 set, non-empty `message`, `details`/`tool`/modes where defined | `contract_matrix::{invalid_params_*, missing_and_absent_entities_carry_structured_details, per_call_permission_override_*}`; `normalized_error_tests` | none / required | R | protocol / in-process |
| Same rejection yields byte-identical error objects over stdio and HTTP | `canonical_journey::contract::rejected_requests_never_mutate_shared_state_*` | none | R | real |
| Legacy string errors (`{"error":"..."}`) still set `isError` | `contract_matrix::legacy_string_errors_are_still_flagged_is_error` | none | R | protocol (characterization; G-7) |
| Unknown JSON-RPC method is a protocol error (-32601), not a tool error | `contract_matrix::unknown_jsonrpc_method_*`; `grpc_transport::scenario_g_*` | `grpc` for gRPC | R | protocol |
| Registry tool names and argument names are snake_case | `contract_matrix::registry_tool_and_argument_names_are_snake_case` | none | R | protocol (reads the registry) |
| camelCase argument keys are dropped, not aliased (SDKs must send snake_case) | `contract_matrix::camel_case_argument_keys_are_ignored_not_aliased`; `canonical_journey::contract::*` | none | R | protocol / real (characterization; G-6) |
| Pagination: `memory_list` limit/offset pages are disjoint, ordered, exhaustive, then empty; `memory_search` limit bounds results | `contract_matrix::{memory_list_pages_*, memory_search_limit_*}`; `canonical_journey::contract::*` (parity across transports) | none | R | protocol / real |
| `memory_search` result cache is keyed on every shaping option (`limit`, `min_score`, `strategy`, `scope`, `workspaces`, `scope_path`, `filter`): a cached answer is never served for a different limit or filter (G-3, fixed) | `contract_matrix::{memory_search_limit_survives_the_result_cache, memory_search_cache_distinguishes_workspaces_filter_and_min_score}`; `result_cache::tests::cache_key_distinguishes_*` | none | R | protocol / unit |
| Async lifecycle: a persisted job moves pending, canceled (idempotent), archived; state survives a process restart and a transport change | `contract_matrix::dream_job_lifecycle_*`; `canonical_journey::contract::dream_job_lifecycle_persists_across_processes_and_transports` | `dream-phase` (in required set) | R | protocol / real |
| Server process lifecycle: ready, serve, kill, port released, caller-owned state kept | `canonical_journey::canonical_real_binary_journey_*`; `real_server_harness` | none | R | real |
| Progress notifications wire format | `streaming_protocol_tests`, `streaming_tier2/3_*` | required set | R | in-process |

### 3.4 Persistence and state

| Contract | Tests | Features | Lane | Kind |
|---|---|---|---|---|
| Rejected requests leave persisted rows unchanged (content, hash, type, importance, workspace, tier, lifecycle, timestamps, version, validity, metadata, tags). Proof reads SQLite after the server process exits and has a positive control (an accepted update changes the snapshot) | `canonical_journey::contract::rejected_requests_never_mutate_shared_state_across_stdio_and_http` | none | R | real |
| One database, two transports, one after the other, sees the same state (create, get, list, search, update, export, markdown read-back, workspace isolation) | `canonical_journey::canonical_real_binary_journey_over_stdio_and_authenticated_http` | none | R | real |
| WAL replay limits, recovery staging, replication | `wal_replay_hardening_tests`, `wal_recovery_staging_tests`, `wal_replication_tests` | required set | R | in-process |
| Storage concurrency regressions | `storage_concurrency_regression_tests` | required set | R | in-process |
| Snapshot and attestation round-trips | `snapshot_attestation` | `agent-portability` | R | in-process |
| A second process opening the live database must not corrupt it (G-1, fixed); MCP file-path tools never open or attach the live SQLite files or their descriptor aliases | `storage_posix_lock_regression_tests::{committed_writes_survive_another_process_open_and_close, second_in_process_open_does_not_drop_the_first_handles_locks}` (real second process via re-exec); `storage_posix_lock_regression_tests::mcp_paths::*` (`replication_recover`, document ingest, `/dev/fd` aliases; DuckDB and media cases under `duckdb-graph` / `multimodal`) | required set (+ `duckdb-graph` for the DuckDB case) | R | real (second process) / in-process |

### 3.5 Registry, reference and SDKs

| Contract | Tests | Features | Lane | Kind |
|---|---|---|---|---|
| `docs/MCP_TOOLS.md` equals what `src/mcp/tools` generates | `scripts/generate-mcp-reference.sh --check` (`scripts/ci.sh` step 5, `just docs`); `scripts/test_generate_mcp_reference.py` | none | R (Documentation job) / L | static |
| Critical tools have required fields and annotations | `scripts/validate_mcp_contract.py`, `scripts/test_validate_mcp_contract.py` | none | L | static |
| SDK calls use only tools and argument names the registry declares; new drift fails (entries are per SDK call site, so a new method calling an already-baselined tool is new drift), fixed drift must be removed from the baseline, and `--update-baseline` is shrink-only (growth needs `--allow-new-drift --reason`); baseline and checker are CODEOWNERS-gated | `scripts/check-sdk-contract-alignment.py` (baseline `docs/quality/sdk-contract-drift-baseline.json`); run by `sdks/python/tests/test_contract.py::test_sdk_calls_match_the_registry_baseline` and `sdks/typescript/src/contract.test.ts` (alignment block); checker unit tests `scripts/test_check_sdk_contract_alignment.py` | none | A (both SDK workflows) / L (checker tests) | static + executed (Python methods run against a recording transport) |
| Python SDK request/response/error/timeout/lifecycle behavior over a real socket | `sdks/python/tests/test_contract.py` (24 tests) | none | A | unit/mock (real socket, stub server) |
| TypeScript SDK request/response/error/timeout behavior over a real socket | `sdks/typescript/src/contract.test.ts` (14 tests) | none | A | unit/mock (real socket, stub server) |
| SDK method serialization (mocked transport) | `sdks/python/tests/test_client.py` and `test_resources.py` etc.; `sdks/typescript/src/index.test.ts`, `integrations.test.ts` | none | A | unit/mock |
| Installed wheel against a real local server: CRUD, typed wrong-bearer, typed killed server | `scripts/test-python-sdk-live.sh` | needs PyPI for `build`, `pytest` | A / L | live |
| Packed tarball against a real local server: CRUD, typed 401, typed 404 | `scripts/test-typescript-sdk-live.sh` | needs npm registry | A / L | live |
| Release artifacts (wheel, sdist, tarball) match version/SHA metadata and install | `scripts/verify-sdk-artifacts.sh` | needs registries | M | live |

## 4. Offline acceptance versus live compatibility

These are different claims and must not be conflated.

- **Offline acceptance** (lane R and A, no registry, no installed package):
  serialization, error decoding, timeouts, lifecycle and registry alignment of the
  SDK *source*, plus the real server's behavior on its own. Passing it does not
  show that a wheel or tarball works.
- **Live SDK compatibility**: an *installed* wheel/tarball against a real local
  server (`scripts/test-python-sdk-live.sh`, `scripts/test-typescript-sdk-live.sh`).
  Both scripts download build tooling (PyPI `build`, `pytest`, `pytest-asyncio`;
  npm `npm ci`), so they cannot run offline. Their equivalents were executed by
  hand for Q4 and are recorded as "manual offline
  equivalent", never as the scripts having run.

## 5. Findings and gaps (not fixed by Q4)

Each entry is evidence for a decision, not a change. Rows marked **fixed** keep their
history here; the covering tests now live in section 3 (G-1 in 3.4, G-3 in 3.3). Where a test exists it
characterizes today's behavior without blessing it.

| ID | Finding | Evidence | Status |
|---|---|---|---|
| G-1 | **Data-loss hazard.** After every storage call `restrict_sqlite_artifact_permissions` (`src/storage/connection.rs`) opens and closes the db, `-wal` and `-shm` files. Closing any descriptor on a file releases all of the process's POSIX locks on it, including SQLite's. A second process (CLI, backup, a test) that opens and closes the database then deletes the live WAL; every later server write goes to an unlinked file and is lost on restart | Observed with the real server: create, read the file from a second process, create again, kill; the second create is gone. Consequence for tests: `persisted_state` in `canonical_journey/contract.rs` reads SQLite only after the server exits, and the raw `row_state` reads in `http_transport_security/workspace_auth.rs` happen while the server runs | **fixed** by task G1 (`974b7a6`, review fixes `9712ade`, `bbe9203`): file modes are narrowed with `fchmodat` without opening descriptors on the SQLite files, and MCP file-path tools refuse or snapshot (`VACUUM INTO`) the live database instead of opening it. Regression tests: `storage_posix_lock_regression_tests` (section 3.4); the C1 raw-row checks now have a positive control |
| G-2 | SDKs call tools/arguments the server does not have: 26 unknown tools (`memory_scope_*`, `memory_federation_*`, `memory_temporal_*`, `memory_add_knowledge`, `resources/list` sent as a tool name, ...), 43 unknown argument keys, 11 required arguments never sent (`memory_synthesis`, `memory_block_*`, `memory_reflect`, ...). Python and TypeScript agree except `memory_search.include_archived` (TypeScript only) | `scripts/check-sdk-contract-alignment.py --list`; real server returned `tool_not_found` / `X is required` | recorded as a ratchet baseline; decision on fix or removal needed (public contract) |
| G-3 | `memory_search` result cache key (`CacheFilterParams`) omits `limit`, `min_score`, `strategy` and others, so a repeated query returns the first answer regardless of `limit` | Real server: limit 3, then limit 1, then limit 2 for one query all return 3 rows | **fixed** by Q2F (`98103b0`): the cache key now includes `limit`, `min_score`, `strategy`, `scope`, `workspaces`, `scope_path` and `filter`. `contract_matrix::memory_search_limit_survives_the_result_cache` is no longer ignored, `contract_matrix::memory_search_cache_distinguishes_workspaces_filter_and_min_score` and `result_cache::tests::cache_key_distinguishes_*` were added; `memory_search_limit_bounds_the_result_count` still bypasses the cache with `skip_cache` |
| G-4 | TypeScript SDK returns the raw MCP envelope and does not raise on `isError`; Python decodes and raises `EngramError` | `contract.test.ts` characterization; Python `mcp_result.py` | characterized |
| G-5 | TypeScript timeout and connection failure surface as platform errors (`AbortError`, `TypeError`), not `EngramError`; Python wraps both | `contract.test.ts` characterization | characterized |
| G-6 | Wrong-typed numeric arguments are reported as `missing_argument` (`id`), numeric-string ids are rejected, and wrong-typed `limit` is silently replaced by the default; unknown keys (camelCase) are dropped silently | `contract_matrix::invalid_params_return_normalized_envelopes_with_is_error` (accepts either code for a wrong-typed id) | camelCase pinned; the rest accepted-set only (`missing_argument` or `invalid_params`) |
| G-7 | Two error shapes coexist: normalized `{"error":{"code","message"}}` and legacy `{"error":"..."}` (e.g. `memory_block_create`, `memory_synthesis`) | `contract_matrix::legacy_string_errors_*` | characterized |
| G-8 | A malformed JSON body gets HTTP 400 `text/plain`, not a JSON-RPC -32700 body; SDKs surface it as `HTTP 400` | `canonical_journey::contract::*` | pinned |
| G-9 | `X-Tenant-Slug`, sent by both SDKs, is ignored by the server; the SDK `tenant` argument is not an authorization scope | `canonical_journey::contract::*` | pinned |
| G-10 | `dream_get` for an unknown job id returns `{"job": null}`, not an error | `contract_matrix::dream_job_lifecycle_*` | characterized |

## 6. Updating this map

- Add a row when a new invariant or public contract is introduced; cite the test by
  path and name; state the kind honestly (a test that mocks the transport is
  `unit/mock` however good its assertions are).
- When a gap is fixed, move its row into the sections above in the same change and
  remove its baseline entries (`python3 scripts/check-sdk-contract-alignment.py
  --update-baseline`; it refuses to add entries).
- Commands for the Q4 contracts:

```bash
bash scripts/test-canonical-journey.sh                       # real binary journeys (required features)
source scripts/ci-required-features.env
cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --test mcp_protocol_tests contract_matrix
python3 -m pytest sdks/python/tests -q
npm --prefix sdks/typescript test && npm --prefix sdks/typescript run type-check
python3 scripts/test_check_sdk_contract_alignment.py
python3 scripts/check-sdk-contract-alignment.py              # full ratchet, both SDKs
scripts/generate-mcp-reference.sh --check
```
