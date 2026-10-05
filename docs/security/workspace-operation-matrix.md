# Workspace authorization: operation matrix

Status as of task C1 (2026-10-05, Lane R). This matrix lists every by-ID and listing
operation family, where the workspace check happens on each transport, how permission
modes interact, which test proves it, and whether it is covered, fixed in C1, or a gap.

Implementation: `src/mcp/workspace_guard.rs` (persisted-workspace guard),
`src/mcp/permission.rs` (`check_tool_authorization`), `src/mcp/handlers/mod.rs`
(`dispatch`), `src/mcp/resources.rs` (`read_resource_as`).

## Principals per transport

| Transport | Principal reaching dispatch | Workspace scope | Test |
|---|---|---|---|
| stdio (`engram-server`, default) | `None`: the local process owner | Unrestricted. A `workspace` argument is a filter, not an authorization boundary. Permission modes (`ENGRAM_PERMISSION_MODE`, per-call `_permission_mode`) still apply. | `test_unrestricted_and_missing_principal_contracts` |
| HTTP, `--http-api-key` set | `ProcessBearer` (admin, no namespace) | Unrestricted | `keyed_principal_reads_any_workspace_by_id`, `keyed_memory_search_preserves_cross_workspace_shapes` |
| HTTP, loopback, no key | `AnonymousLoopback` (read-only, `default` only) | Restricted | `workspace_auth::*` in `tests/http_transport_security.rs` |
| gRPC, key set / loopback no key | `ProcessBearer` / `AnonymousLoopback` | Anonymous: transport allowlist (`memory_list` with exactly `{workspace: "default"}`), then the same dispatch checks | `scenario_j_*`, `scenario_k*_*`, `scenario_l_*`, `scenario_m_*` in `tests/grpc_transport.rs` |
| Library `dispatch` with `HandlerContext::principal` | Any `TransportPrincipal`, incl. `StoredToken` bound to a namespace | Restricted when the principal has a namespace or is anonymous | `tests/workspace_auth_enforcement_tests.rs` |

Before C1 the HTTP and gRPC transports authenticated a principal for their pre-check but
called `McpHandler::handle_request`, and `engram-server` built every `HandlerContext` with
`principal: None`. The principal never reached dispatch or storage. C1 adds
`McpHandler::handle_request_as(request, principal)`; both transports call it, and
`EngramHandler` threads the principal into tool dispatch and `resources/read`. No
production transport mints `StoredToken` principals today; they are exercised through the
library dispatch path.

## Rules for a workspace-restricted principal

A principal is restricted when `allows_all_workspaces` is false (anonymous loopback, or a
token with a namespace).

1. **Claim pre-check (existing).** Every `workspace`/`workspaces` value anywhere in the
   arguments must be allowed. No claim at all: conservative denial (`permission_denied`).
   `global: true`: denial. This runs on the HTTP pre-check (403) and in `dispatch`.
2. **A claim only scopes tools that honor it (new).** The tool's schema must declare a
   `workspace`/`workspaces` parameter, or the tool must be in `ID_SCOPED_TOOLS` (its whole
   effect is limited to verified memory IDs). Otherwise: `permission_denied` with
   `details.reason = "tool_not_workspace_scoped"`. This is a **conservative denial**, not
   an IDOR finding: the tool cannot honor the claim, so it is refused.
   `WORKSPACE_ALIAS_TOOLS` are refused even though they declare `workspace`, because there
   the parameter is not the memory workspace (see the audit below).
   Exception (controller ruling, review round 1): `CATALOG_METADATA_TOOLS`
   (`discover_tools`, `permission_mode_status`) return only catalog/permission metadata and
   no workspace data, so restricted principals may call them without a claim; only the
   permission mode applies.
3. **Persisted-workspace guard (new).** Every memory ID in the top-level arguments
   (`MEMORY_ID_ARGUMENT_KEYS`: `id`, `ids`, `memory_id`, `memory_ids`, `from_id`, `to_id`,
   `parent_id`, `summary_of_id`, `focus_id`, `source_memory_ids`; integers, integer strings
   and arrays) is checked against `memories.workspace` before the handler runs. A foreign
   row and a missing row both return the existing normalized envelope
   `{"error":{"code":"not_found","message":"memory not found: '<id>'","details":{"entity":"memory","id":"<id>"}}}`,
   so neither content nor existence leaks. A storage error during the check is a denial.
4. **In-transaction re-check (new).** Mutating core handlers call `ensure_memory_access`
   on the same connection/transaction as the mutation, and the read handlers call it before
   access tracking and reinforcement.

A missing workspace never means allow: no claim is a denial (rule 1), a missing row is a
denial (rule 3).

No MCP wire format changed. Denials reuse the `ToolError` envelope. On HTTP, rules 1-2 deny
in the pre-check (HTTP 403, JSON-RPC error `-32003`, as before); rule 3 denies inside the
tool result (HTTP 200, `isError: true`) because it needs storage.

## Operation matrix

Columns: where the check happens; permission mode required (`required_mode`); proof; status.
"Guard" = rule 3, "in-tx" = rule 4, "unscoped" = rule 2.

| Operation | Workspace check (dispatcher and HTTP) | Permission mode | Test | Status |
|---|---|---|---|---|
| `memory_get` | Guard + handler check before access tracking/reinforcement | `read_only` | `test_claimed_workspace_does_not_authorize_foreign_id` (dispatcher and HTTP, anonymous and authenticated, nonexistent vs foreign, before/after row state, legit ID positive) | Fixed in C1 (was IDOR: content returned, `stability` reinforced) |
| `memory_get_public` | Same as `memory_get` | `read_only` | Same test | Fixed in C1 |
| `memory_versions` | Guard | `read_only` | `test_claimed_workspace_does_not_authorize_foreign_id` | Fixed in C1 |
| `memory_get_full` | Unscoped (follows `summary_of_id` to other rows) | `read_only` | `test_restricted_principal_cannot_scope_unscoped_tool_with_claim` | Conservative denial |
| `memory_update` | Guard + in-tx | `scoped_write` | `test_foreign_id_mutations_denied_without_side_effects`, `test_handler_checks_workspace_inside_mutation_transaction` | Fixed in C1 (was IDOR: foreign row overwritten) |
| `memory_delete` | Guard + in-tx | `admin` | Same tests | Fixed in C1 |
| `memory_delete` with `cascade_chain` | Guard on root + in-tx on every chain member; a foreign member reports the root as not found and rolls back | `admin` | `test_handler_checks_workspace_inside_mutation_transaction` | Fixed in C1 (was: foreign chain members deleted) |
| `memory_delete_batch` (± `cascade_chain`) | Guard on all IDs + in-tx on targets/chains; the batch is all-or-nothing on a foreign ID | `admin` | `test_foreign_id_mutations_denied_without_side_effects`, `test_handler_checks_workspace_inside_mutation_transaction` | Fixed in C1 |
| `memory_link`, `memory_unlink` | Guard on both ends + in-tx | `scoped_write` | `test_foreign_id_mutations_denied_without_side_effects`, `test_handler_checks_workspace_inside_mutation_transaction` | Fixed in C1 |
| `workspace_move` | Claim pre-check on target + guard on source + in-tx (now a transaction) | `scoped_write` | Same tests | Fixed in C1 (was: foreign memory movable into own workspace) |
| `memory_create` (`summary_of_id`), `memory_create_section` (`parent_id`) | Claim pre-check + guard on the referenced ID (pre-dispatch, not in the creating transaction) | `scoped_write` | `test_foreign_id_mutations_denied_without_side_effects` (no row created) | Fixed in C1 |
| Other tools that declare `workspace` and take memory IDs (`memory_summarize`, `memory_feedback`, `memory_predict_links`, `memory_enrichment_audit`, `memory_agent_writeback` `source_memory_ids`, `harness_verify`) | Claim pre-check + guard (pre-dispatch) | Per tool | Generic guard only; `memory_id_arguments_collects_known_keys_only` (unit) | Fixed by the generic guard; gap: no per-tool negative test, and the check is not in the handler's transaction |
| `memory_list` | Claim pre-check + handler filters by `workspace` | `read_only` | `test_listing_and_search_stay_in_claimed_workspace`, `anonymous_claim_cannot_scope_tool_without_workspace_parameter` (HTTP) | Covered |
| `memory_search` (incl. `workspaces`, `filters.workspace`, `global`) | Claim pre-check + handler filter | `read_only` | `test_listing_and_search_stay_in_claimed_workspace`, `loopback_anonymous_memory_search_rejects_cross_workspace_shapes`, `keyed_memory_search_preserves_cross_workspace_shapes` | Covered |
| `memory_export_graph` | Unscoped (ignores `workspace` and `focus_id`, lists every workspace) | `read_only` | `test_restricted_principal_cannot_scope_unscoped_tool_with_claim`, `anonymous_claim_cannot_scope_tool_without_workspace_parameter` (HTTP 403) | Fixed in C1 (was: all workspaces exported to anonymous loopback with `workspace: "default"`) |
| Link traversal: `memory_related`, `memory_traverse`, `memory_find_path`, `graph_query` | Unscoped: results can include IDs from other workspaces | `read_only` | `test_restricted_principal_cannot_scope_unscoped_tool_with_claim` (`memory_related`), `workspace_parameter_detection_matches_catalog` | Conservative denial. Gap: enabling them for restricted principals needs per-node workspace filtering |
| Other by-ID tools without a `workspace` parameter (59 more; 65 by-ID tools are unscoped in all: `salience_*`, `quality_*`, `memory_explain*`, `memory_score`, `memory_boost`, `memory_set_expiration`, `memory_promote*`, `memory_extract_*`, `scope_get/set`, ...) | Unscoped | Per tool | `workspace_parameter_detection_matches_catalog` (classification) | Conservative denial. Each can join `ID_SCOPED_TOOLS` after a review shows it only touches the verified row |
| Dispatcher aliases `graph_predict_links`, `graph_cluster_concepts` | Unscoped for restricted principals | Canonicalized (`canonical_tool_name`) to `memory_predict_links` / `memory_cluster_concepts`; every dispatchable name has a mode | `test_every_dispatchable_name_has_a_permission_mode` (parses `dispatch`), `workspace_parameter_detection_matches_catalog` | Fixed in C1 round 1 (was: `required_mode` `None`, so env/per-call modes were skipped) |
| Write flags on read-only tools: `memory_predict_links`/`graph_predict_links` `auto_apply`, `sync_state` `update_version` | n/a | `required_mode_for_call` raises the call to `scoped_write` when the flag is truthy; unknown tools require `admin` | `test_read_only_mode_blocks_write_flags_on_read_tools_and_aliases` | Fixed in C1 round 1 (was: `read_only` mode + `auto_apply` wrote crossrefs) |
| `context_record`, `context_record_artifact`, `context_get_artifact`, `context_search`, `context_build_bundle` | `workspace` is an alias for `workspace_path_hash` (caller-asserted scope compared by the artifact policy), not the memory workspace: `WORKSPACE_ALIAS_TOOLS`, refused for restricted principals | per tool | `test_workspace_alias_context_tools_denied_for_restricted_principal`, `anonymous_claim_cannot_scope_tool_without_workspace_parameter` (HTTP 403) | Fixed in C1 round 1 (was: claim accepted, lookup bound to caller hash; "artifact not found" vs denial = existence oracle) |
| Catalog metadata: `discover_tools`, `permission_mode_status` | No workspace data; allowed without a claim, mode still checked | `read_only` | `test_catalog_metadata_tools_allowed_for_restricted_principals`, `anonymous_claim_cannot_scope_tool_without_workspace_parameter` (HTTP) | Covered (controller ruling) |
| Own-ID positive controls (restricted writer): `workspace_move`, `memory_link`, `memory_unlink`, `memory_delete`, `memory_delete_batch`, own-chain cascade | Guards pass for persisted own workspace | per tool | `test_restricted_principal_own_id_positive_controls` | Covered |
| `resources/read engram://memory/{id}` | `read_resource_as`: persisted workspace checked before `get_memory` (which tracks access); foreign message equals missing | n/a (resources) | `anonymous_resources_read_is_bound_to_persisted_workspace` | Fixed in C1 (was IDOR over HTTP: no pre-check for non-tool methods) |
| `resources/read engram://workspace/{name}[/memories]` | Name must be allowed | n/a | Same test | Fixed in C1 |
| `resources/read engram://stats`, `engram://entities` | Denied for restricted principals (global data) | n/a | Same test | Fixed in C1 |
| `resources/subscribe` | Not principal-aware; registry is process-global and notifications carry only the URI | n/a | none | Gap (no content read). gRPC anonymous: rejected by transport (`scenario_k5_loopback_no_key_rejects_resource_read`) |
| Missing workspace claim (ID-only call) | Conservative denial for restricted principals | any | `test_principal_workspace_namespace_isolation`, `loopback_anonymous_principal_rejects_omitted_workspace_before_dispatch` (unit) | Covered |
| Case-variant keys (`Workspace`, `WORKSPACE`) | Not a claim: alone they are a conservative denial; mixed with `workspace` only the lowercase key is honored | any | `test_case_variant_workspace_keys_do_not_widen_scope` | Covered |
| Reserved governance metadata (`memory_agent_writeback`, case-insensitive) | `reject_reserved_metadata` lowercases keys | `scoped_write` | `test_mcp_memory_agent_writeback_rejects_reuse_and_spoofing` (`tests/dream_integration.rs`) | Covered (existing) |
| Permission mode before workspace guard | Mode denial runs first and does not depend on the row (no existence oracle); a sufficient mode never bypasses the guard | all | `test_permission_mode_and_workspace_guard_compose_on_foreign_id` | Covered |

## Audit: `workspace` parameters that are not the memory workspace

Every `workspace`/`workspaces` property in the catalog (96 tools) was reviewed by its schema
description (C1 review round 1). Only the five Operational Context tools above use it as
something else (`workspace_path_hash` alias); they are in `WORKSPACE_ALIAS_TOOLS`. All others
name a memory workspace (filter, target, or scope). This audit is by description; handlers
of workspace-declaring tools other than `memory_list`/`memory_search` are not individually
proven to filter (see residual risks).

## Known residual risks

- Rule 3 runs before the handler on a separate connection. For the core mutators
  (`memory_update`, `memory_delete[_batch]`, `memory_link/unlink`, `workspace_move`) the
  in-transaction re-check closes the time-of-check gap. Other mutating tools reachable by a
  restricted writer rely on rule 3 only. No production transport issues restricted writers
  today (anonymous loopback is read-only).
- Tools that declare `workspace` are trusted to filter by it. `memory_list` and
  `memory_search` are proven by tests; the rest are not individually audited.
- Rule 3 only reads top-level ID keys. Nested references are not authorized by the guard:
  `memory_agent_writeback` `evidence[].source_id` (string source references),
  `memory_explain_search` `results[].memory_id`, and per-item fields of batch tools
  (`memory_create_batch` `memories[]`, `memory_ingest_fact_batch` `facts[]`) rely on the
  claim pre-check (which does walk nested `workspace` keys) and on the handler.
- The guard treats an integer string under an ID key as a memory ID. A workspace-scoped
  tool whose `id` is a non-memory string (`context_record_artifact`) can be refused for a
  restricted writer when that string is numeric (fail-closed, no leak).
- gRPC propagation of the principal into dispatch has no end-to-end test with
  `EngramHandler`; the gRPC suite uses a test handler.
