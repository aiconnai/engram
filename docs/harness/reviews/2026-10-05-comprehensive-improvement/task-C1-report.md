# Task C1 report: workspace authorization on the real path (dispatcher + HTTP)

Status: DONE_WITH_CONCERNS (see Concerns)
Commit: `44d42f8 fix(mcp): authorize memory IDs by persisted workspace` (base `a87a3ce`, which is Q2's commit on top of `46fc3e2`)

## Findings reproduced (RED, before any src change)

1. **The principal never reached dispatch.** HTTP (`mcp_handler.rs`) and gRPC authenticated a principal for their pre-check, then called `McpHandler::handle_request`. `engram-server` built every `HandlerContext` with `principal: None`. Handlers had no workspace check on by-ID lookups.
2. **IDOR through a claimed workspace.** If a principal restricted to A sent `{id: <B id>, workspace: "A"}`, the pre-check passed. In that case:
   - `memory_get` / `memory_get_public` returned B's content and reinforced `stability` (side effect);
   - `memory_update` overwrote the B row;
   - `memory_delete` with `cascade_chain` deleted B members chained under an A root.
3. **Claim ignored by the tool.** Anonymous loopback HTTP calling `memory_export_graph {workspace:"default"}` exported every workspace's content (HTTP 200).
4. **Resources bypass.** On HTTP, anonymous `resources/read engram://memory/{private id}` returned private content. The HTTP pre-check only inspects `tools/call`.

## What I implemented

- `McpHandler::handle_request_as(request, principal)` in `src/mcp/protocol.rs`:
  - The default delegates to `handle_request`. `Arc<T>` forwards the call.
  - HTTP and gRPC now call it with `Some(principal)`. stdio keeps `None`, which means the local owner.
  - `EngramHandler` (`src/bin/server.rs`) threads the principal into `dispatch` and `resources/read`.
- New module `src/mcp/workspace_guard.rs`:
  - `ensure_memory_access(conn, principal, id)` reads `memories.workspace`. A missing row and a foreign row both return `EngramError::NotFound(id)`. Unrestricted principals and `None` pass.
  - `denial_for_memory_arguments` runs in `dispatch` after the permission check and before the handler. It covers top-level `MEMORY_ID_ARGUMENT_KEYS` (ints, int strings, arrays). It fails closed on a DB error.
  - `tool_honors_workspace_claim`: true only if the tool schema declares `workspace`/`workspaces`, or the tool is in `ID_SCOPED_TOOLS` (get, get_public, versions, update, delete, delete_batch, link, unlink).
- `permission::check_tool_authorization`: a restricted principal calling a tool whose claim can't be honored gets `permission_denied` with `details.reason="tool_not_workspace_scoped"`. This is a conservative denial. On HTTP it is a 403 in the pre-check.
- In-transaction re-checks, on the same connection or transaction as the mutation:
  - `memory_update` and `memory_delete`; for cascade chains, every member is checked and a foreign member reports the root as not found;
  - `memory_delete_batch`, plain and cascade;
  - `memory_link` / `memory_unlink`;
  - `workspace_move`, now inside `with_transaction`;
  - `memory_get` / `memory_get_public` check before access tracking and reinforcement.
- `resources::read_resource_as`:
  - restricted principals: memory by persisted workspace, with the same message as a missing ID and checked before access tracking;
  - `workspace/{name}[/memories]` only for an allowed name;
  - `stats` / `entities` / unknown URIs denied.
- `docs/security/workspace-operation-matrix.md`: principals per transport, the four rules, and an operation matrix (check location, permission mode, test, status). It separates conservative denial from IDOR fixes and lists the gaps.

No MCP tool names, params or envelope changed. Foreign IDs reuse the existing `not_found` ToolError. Unscoped denials reuse the `permission_denied` envelope and add `details.reason`.

## TDD evidence

RED: `source scripts/ci-required-features.env; cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --test workspace_auth_enforcement_tests` gave 7 passed, 4 failed:
- `test_claimed_workspace_does_not_authorize_foreign_id`: `authenticated/memory_get: expected structured error, got {... "content":"zebracorn foreign payroll secret" ... "stability":1.1124999523162842 ... "workspace":"tenant-b"}`.
- `test_foreign_id_mutations_denied_without_side_effects`: `memory_update ... got {... "content":"overwritten" ... "version":2,"workspace":"tenant-b"}`.
- `test_handler_checks_workspace_inside_mutation_transaction`: `memory_delete cascade_chain ... got {"count":2,"deleted_ids":[1,2]}`.
- `test_restricted_principal_cannot_scope_unscoped_tool_with_claim`: `memory_export_graph ... got {"nodes":[{"label":"zebracorn foreign payroll secret"...`.

The case-variant and listing tests passed on RED. They are regression guards for behavior that already held.

RED, HTTP: `... --test http_transport_security workspace_auth` gave 1 passed (the keyed positive), 3 failed:
- export_graph returned 200 with the private label;
- `memory_get` over HTTP: `left: Null right: "not_found"`, with the private content in the body;
- `resources/read` returned the private memory.

The first HTTP run also showed lock-poison cascades. The new module now uses a poison-tolerant lock.

All of these failures are behavioral (leak or mutation), which is what the brief predicted.

GREEN, after the implementation:
- `--test workspace_auth_enforcement_tests`: 11 passed, 0 failed
- `--test http_transport_security`: 16 passed (12 existing + 4 new)
- `--test permission_modes_tests`: 7 passed. `test_permission_mode_and_workspace_guard_compose_on_foreign_id` was written after the implementation; its final assertion depends on the guard.
- `--test mcp_protocol_tests`: 56 passed
- `--test grpc_transport`: 25 passed; `--lib mcp::`: 350 passed; `--bin engram-server`: 2 passed
- Full: `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests` exited 0: 52 binaries, 2003 passed, 0 failed, 1 ignored.
- `cargo fmt --all -- --check` OK. `cargo clippy --all-targets --all-features -- -D warnings`: no issues. The pre-commit hook also passed.

All commands used the required feature set from `scripts/ci-required-features.env`.

## Tests added

`tests/workspace_auth_enforcement_tests.rs` (dispatch with a principal, plus direct handler calls):
- `test_claimed_workspace_does_not_authorize_foreign_id`:
  - covers an authenticated principal (`StoredToken`, namespace A) and anonymous loopback, for both `memory_get` and `memory_get_public`;
  - asserts `not_found`, no content, and that the foreign response equals the nonexistent one after id normalization;
  - compares row state before and after: access_count, last_accessed_at, updated_at, stability, version, reinforcements, crossrefs;
  - positive case: a legitimate own ID is readable; it also checks `memory_versions`.
- `test_foreign_id_mutations_denied_without_side_effects` (admin namespaced writer):
  - covers update, delete, delete_batch, link, unlink, workspace_move, `memory_create` with `summary_of_id` and `memory_create_section` with `parent_id`;
  - asserts no row change and no row created or removed; positive own update.
- `test_handler_checks_workspace_inside_mutation_transaction`:
  - cascade chain rollback through dispatch;
  - direct handler calls that bypass the dispatcher guard: get, get_public, update, delete, delete_batch, link, unlink, workspace_move.
- `test_restricted_principal_cannot_scope_unscoped_tool_with_claim`: export_graph (with and without `focus_id`), memory_related, memory_get_full.
- `test_listing_and_search_stay_in_claimed_workspace`, `test_case_variant_workspace_keys_do_not_widen_scope`, `test_unrestricted_and_missing_principal_contracts`.

`tests/http_transport_security/workspace_auth.rs` (real `engram-server` process, DB seeded before start; included from `http_transport_security.rs` via `#[path]`):
- `test_claimed_workspace_does_not_authorize_foreign_id` (HTTP, anonymous loopback, raw DB state before and after)
- `anonymous_resources_read_is_bound_to_persisted_workspace`
- `anonymous_claim_cannot_scope_tool_without_workspace_parameter`
- `keyed_principal_reads_any_workspace_by_id`

`tests/permission_modes_tests.rs`:
- `test_permission_mode_and_workspace_guard_compose_on_foreign_id`: the mode denial comes first and is identical for foreign and missing IDs; the per-call override narrows; a sufficient mode still hits the guard.

`src/mcp/workspace_guard.rs` unit tests check the catalog classification and ID-key extraction.

## Files changed

- src: `bin/server.rs`, `mcp/{protocol,permission,resources,mod,grpc_transport,workspace_guard(new)}.rs`, `mcp/http_transport/mcp_handler.rs`, `mcp/handlers/{mod,graph,workspace}.rs`, `mcp/handlers/memory_crud/read_update_delete.rs`
- tests: `workspace_auth_enforcement_tests.rs`, `permission_modes_tests.rs`, `http_transport_security.rs` (seeded-spawn helpers + module include), `http_transport_security/workspace_auth.rs` (new)
- docs: `docs/security/workspace-operation-matrix.md` (new), `docs/harness/progress/2026-10-05-improvement-lane-r.md` (appended)

## Deviations from the brief

- Files outside the brief's list:
  - `src/mcp/workspace_guard.rs`: new, so `permission.rs` (605 lines) doesn't grow toward 800;
  - `protocol.rs`, `resources.rs`, `grpc_transport.rs`, `handlers/workspace.rs`, `server.rs`: needed to propagate the principal and to fix the resources IDOR.
- The HTTP tests live in a submodule file so `http_transport_security.rs` stays near its previous size. They still run under `--test http_transport_security`.
- gRPC propagation was added for consistency. Its existing transport allowlist already blocked these attacks for anonymous callers.
- The brief mentions journey/SDK runs. No SDK or journey suite was run beyond the full Rust `--tests` run (NOT RUN: SDK and journey suites).

## Concerns / open items

1. **Behavior change for HTTP anonymous loopback** (fail-closed, deliberate). Tools whose schema has no `workspace` parameter and that are not in `ID_SCOPED_TOOLS` are now refused, even with `workspace:"default"`. That is 65 by-ID tools, including traversal (`memory_related`, `memory_traverse`, `memory_find_path`, `graph_query`), `memory_get_full`, `memory_export_graph`, `salience_*`, `quality_*`, and other no-workspace tools such as `memory_stats` / `discover_tools`. Previously these ran unscoped. gRPC was already stricter. Owner sign-off is recommended; the allowlist can grow tool by tool after review.
2. **The `McpHandler::handle_request_as` default drops the principal.** This keeps test handlers and external implementors compatible. A custom handler that reads workspace data must override it. This is documented on the trait.
3. **Generic-guard-only coverage.** Workspace-declaring tools with ID params beyond create/create_section (summarize, feedback, predict_links, enrichment_audit, agent_writeback, harness_verify) are covered by the pre-dispatch guard only. There is no in-transaction re-check and no per-tool test. No production transport issues restricted writers today (anonymous loopback is read-only).
4. **Out-of-scope gaps recorded in the matrix:**
   - the dispatcher aliases `graph_predict_links` / `graph_cluster_concepts` have `required_mode == None`, so permission modes skip them for unrestricted callers;
   - `resources/subscribe` is not principal-aware (a global registry; notifications carry only the URI);
   - there is no gRPC end-to-end test with the real `EngramHandler`.
5. A numeric-string non-memory `id` (e.g. `context_record_artifact`) under a restricted writer is refused (fail-closed).

---

# Fix report: review round 1

Commit: `6750b9b fix(mcp): close context alias and permission-mode bypasses` (on `44d42f8`)

## Changes

1. **[Important] `context_*` workspace alias.**
   - Audit: I reviewed all 96 catalog tools that declare `workspace`/`workspaces`, by schema description. Only `context_record`, `context_record_artifact`, `context_get_artifact`, `context_search` and `context_build_bundle` use `workspace` as something other than the memory workspace (an alias for `workspace_path_hash`).
   - Fix: these five tools are now in `WORKSPACE_ALIAS_TOOLS` (`src/mcp/workspace_guard.rs`). `tool_honors_workspace_claim` returns false for them, so a restricted principal gets `permission_denied` with `reason: tool_not_workspace_scoped` before any lookup.
2. **[Important] Permission-mode bypass via aliases** (`src/mcp/permission.rs`).
   - `DISPATCH_ALIASES` and `canonical_tool_name` are applied inside `required_mode`.
   - New `required_mode_for_call(tool, params)`:
     - unknown tool names require `admin` (fail closed);
     - `WRITE_FLAG_PARAMS` (`memory_predict_links.auto_apply`, `sync_state.update_version`) raise a read-only call to `scoped_write`. The flag is read from the arguments or from `params.arguments`.
   - The per-call override, the env mode and the principal mode in `check_tool_authorization` all use it now.
   - Public signatures are unchanged. `required_mode("nonexistent_tool")` still returns `None` (existing assertion kept).
   - Searched `dispatch` for other aliases: only the two pairs exist. `memory_seed` is a catalog tool, not an alias. A test that parses `dispatch` guards against new ones.
   - Note: the reviewer's scenario (`read_only` + `auto_apply`) needed both fixes. Canonicalization alone would still allow the write, because the canonical `memory_predict_links` is annotated read_only. `sync_state` with `update_version` was a second instance of the same flaw (a read_only tool that writes).
3. **[Minor] Own-ID positive controls and nested-reference gap.**
   - Added `test_restricted_principal_own_id_positive_controls`: `workspace_move`, link, unlink, delete, delete_batch and own-chain cascade, all done by a restricted admin writer.
   - Added a matrix residual entry: nested references are not authorized by rule 3. This covers `memory_agent_writeback` `evidence[].source_id`, `memory_explain_search` `results[].memory_id`, and batch items.
4. **Controller ruling.** `CATALOG_METADATA_TOOLS` (`discover_tools`, `permission_mode_status`) skip the workspace rules for restricted principals; only the permission mode applies. I confirmed neither handler touches storage. Data tools stay denied: the test uses `memory_stats` as the control.
5. **Matrix:** added rows for the context alias tools, aliases, write flags, metadata tools and positive controls; an audit section; and the nested-reference residual.

## TDD evidence

All runs used `source scripts/ci-required-features.env` and `--no-default-features --features "$CI_REQUIRED_FEATURES"`.

RED, before any src change:
- `--test permission_modes_tests`: 7 passed, 2 failed.
  - `test_every_dispatchable_name_has_a_permission_mode`: "dispatchable tool `graph_predict_links` has no permission mode (fail-open)".
  - `test_read_only_mode_blocks_write_flags_on_read_tools_and_aliases`: `graph_predict_links: {"applied_count":1,"count":1,...}` under `ENGRAM_PERMISSION_MODE=read_only`, so a crossref was written.
- `--test workspace_auth_enforcement_tests`: 12 passed, 2 failed.
  - `namespaced/context_get_artifact: {"error":"Storage error: Context artifact not found: artifact-x"}`, which is the existence oracle.
  - `discover_tools must stay available: {"error":{"code":"permission_denied",...,"reason":"tool_not_workspace_scoped"}}`.
  - The positive-controls test passed on RED, as expected for a positive control.
- The two new HTTP assertions (context_get_artifact returns 403; discover_tools is available) were added before the fix but first run after it. Their dispatcher equivalents were RED.

GREEN:
- `--test workspace_auth_enforcement_tests`: 14/14
- `--test permission_modes_tests`: 9/9
- `--test http_transport_security`: 16/16
- `--lib`: 1566 passed, 1 ignored
- `--test grpc_transport`: 25; `--test mcp_protocol_tests`: 56; `--test normalized_error_tests`: 6; `--test portability_permissions_routing_tests`: 3
- Full `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests`: exit 0, 52 binaries, 2008 passed, 0 failed, 1 ignored.
- `cargo fmt --all -- --check` OK. `cargo clippy --all-targets --all-features -- -D warnings`: no issues. The pre-commit hook passed.

## Item 5: SDK and journey scripts

- `bash scripts/test-canonical-journey.sh` (offline, default features): exit 0, 8 passed.
- `scripts/verify-sdk-artifacts.sh`: **NOT RUN.** It needs built artifacts, and building or installing them offline is not possible here:
  - Python: `python3 -m build --no-isolation sdks/python` fails because `setuptools.build_meta` is unavailable in the PATH python3. The verify step's venv install would need `httpx` from PyPI, which is not in the pip cache.
  - npm: `npm ci --offline` fails with `ENOTCACHED` (yocto-queue).
  - Fetching from PyPI or npm is external network, which is not authorized.
  - The partial `node_modules` was removed. The worktree is clean apart from the commits.
  - Only the offline self-test ran: `scripts/verify-sdk-artifacts.sh --self-test-version-mismatch` PASS. This covers the script's logic, not the artifacts.
  - I reviewed the `--live` anonymous dream checks by reading them. They still match the new behavior: `dream_candidate_get` gets a rule-2 denial with `current_mode == required_mode` (`read_only`); review/apply are denied by mode.

## Remaining concerns

- `WORKSPACE_ALIAS_TOOLS` and `CATALOG_METADATA_TOOLS` are hand-maintained lists. The alias audit was done by schema description. A new tool that reuses `workspace` for another meaning would be admitted until someone adds it to the list.
- The behavior change for anonymous loopback from round 0 still applies: 65 by-ID tools without a `workspace` parameter, plus other no-workspace data tools, are denied. It still needs owner sign-off.
