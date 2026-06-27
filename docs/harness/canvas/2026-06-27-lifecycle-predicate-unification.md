# Review Canvas: lifecycle predicate unification

Scope: unify decay-derived lifecycle decisions behind one canonical predicate
and keep salience, policy, and compression surfaces from writing
`memories.lifecycle_state`.

## Approaches considered

| Approach | Decision | Reason |
|---|---|---|
| Keep existing SQL predicates and tune thresholds | Rejected | Preserves multiple lifecycle writers and divergent decay rules. |
| Put lifecycle thresholds in `SalienceConfig` | Rejected | Salience scoring must remain score/history-only after this change. |
| Add schema fields such as `lifecycle_changed_at` or stability | Rejected | The approved plan is zero-migration and keeps `SCHEMA_VERSION = 44`. |
| Add pure `decide_lifecycle_state` and route `lifecycle_run` through it | Accepted | Gives one testable decay predicate while preserving explicit domain writers. |
| Make compression tools summarize only already-Archived rows | Accepted | Removes compression as an implicit lifecycle writer. |

## Hot-path complexity

| Path | Before | After |
|---|---|---|
| `lifecycle_run` | Two restrictive SQL scans by stale/archive path | One permissive non-archived candidate scan plus pure predicate decisions |
| `run_salience_decay` | Score calculation plus lifecycle update path | Score/history writes only |
| `memory_decay` | Policy writes plus lifecycle update path | Policy writes only |
| `memory_archive_old` | Candidate scan plus summary and lifecycle update | Already-Archived candidate scan plus summary only |
| `compress_old_memories` | Active candidate scan plus summary and lifecycle update | Already-Archived candidate scan plus summary only |

## Edge cases covered

| Edge case | Evidence |
|---|---|
| High-importance, high-access, idle memory archives despite old filters | `test_lifecycle_run_archives_high_importance_high_access_candidate` |
| Dry-run and apply lifecycle candidates stay in parity | `test_lifecycle_run_dry_run_apply_parity` |
| Direct `Active -> Archived` transition is allowed | `test_lifecycle_run_allows_direct_active_to_archived_transition` |
| Repeated lifecycle apply is idempotent | `test_lifecycle_run_apply_is_idempotent` |
| Salience decay records history without lifecycle mutation | `test_salience_decay_records_history_without_lifecycle_transition` |
| Policy decay updates policy rows without lifecycle mutation | `memory_decay_updates_policy_scores_without_lifecycle_transition` |
| Active rows are not summarized by compression paths | summarize and retention compression regression tests |
| Compression paths do not duplicate summaries on repeated runs | summarize and retention idempotency regression tests |
| Retention auto-delete remains explicit and archived-only | `retention_auto_delete_still_soft_deletes_archived_rows` |

## Breakage-risk table

| Risk | Level | Mitigation |
|---|---|---|
| Public tool docs continue advertising old lifecycle side effects | High | Updated registry/memory metadata and regenerated `docs/MCP_TOOLS.md`. |
| Compression scheduler silently archives active rows | High | `compress_old_memories` now selects only `Archived`; server log says compressed. |
| Lifecycle dry-run diverges from apply | Medium | Shared transition list used for both paths and covered by handler test. |
| Stale memories reactivate automatically | Medium | Predicate preserves stale monotonicity until explicit lifecycle set. |
| Retention max-count domain writer accidentally removed | Medium | Left existing max-count archival SQL unchanged; writer inventory records it. |
| Zero-migration boundary drifts | Medium | `SCHEMA_VERSION` remains 44 and migration tests still pass. |

## Verification summary

- `rtk cargo test --workspace --all-targets --locked` — PASS, 1227 tests.
- `rtk cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS.
- `rtk ./scripts/generate-mcp-reference.sh --check` — PASS.
- `rtk bash docs/harness/bin/sensors.sh` — PASS.
- Writer inventory contains only canonical, manual/domain, helper, and test
  lifecycle writes.
