# Review Canvas: context-quality-and-harness-sync

Date: 2026-11-19
Owner: owner-authorized Frente 1 (`fix/context-quality-and-harness-sync` over `origin/main` `597b71d3`)
Scope: Align `src/intelligence/context_quality/` (`conflicts.rs`, `duplicates.rs`, `report.rs`) SQL queries with `SCHEMA_VERSION = 49` (`valid_to`, `expires_at`, `lifecycle_state`, normalized `memory_tags`, and workspace isolation), route `ResolutionType::DeleteBoth` through normal soft-delete bookkeeping (`delete_memory_if_active`), stop swallowing database errors in `auto_consolidate::run_consolidation`, close the remaining `[LOW]` commit-failure rollback gap in `src/sync/worker.rs` (`reset_interrupted_sync`), and reconcile `docs/harness/` + `risk-register.yaml` + `CLAUDE.md` after PRs `#280` and `#281`.

## Trigger

| Trigger | Evidence |
|---|---|
| Runtime SQL failures (`no such column: deleted_at` / `no such column: tags`) across `quality_report`, `quality_find_duplicates`, `quality_find_conflicts`, and `quality_resolve_conflict(resolution="delete_both")` | `src/intelligence/context_quality/conflicts.rs`, `src/intelligence/context_quality/duplicates.rs`, `src/intelligence/context_quality/report.rs` |
| Silent `.unwrap_or_default()` error suppression hiding `context_quality` SQL failures in `auto_consolidate::run_consolidation` | `src/intelligence/auto_consolidate.rs:239-243,279-281` |
| Cross-workspace conflict/duplicate leakage risk in `detect_conflicts`, `find_near_duplicates`, and `generate_quality_report` | `conflicts.rs` (`OR EXISTS` tag overlap), `duplicates.rs` (unscoped pair loop), `report.rs` (unscoped `memory_conflicts` / `duplicate_candidates` counts) |
| `[LOW]` finding from `2026-11-16-remediation-2026-10-26-post.md`: `reset_interrupted_sync` leaves transaction open if `COMMIT` / `RELEASE SAVEPOINT` fails | `src/sync/worker.rs:458-503` |
| Harness live-state & `risk-register.yaml` drift after squash merges `#281` (`33992508`) and `#280` (`597b71d3`) | `docs/harness/progress.md`, `docs/harness/SPEC.md`, `docs/harness/risk-register.yaml`, `CLAUDE.md` |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Add legacy `deleted_at` / `tags` columns to `memories` via a schema v50 migration vs align `context_quality` SQL with canonical `valid_to`, `expires_at`, `lifecycle_state`, and `memory_tags` | Accepted (align SQL, zero schema migration) | `memories` already tracks soft-deletion via `valid_to`, archival via `lifecycle_state`, and normalized tags via `tags` + `memory_tags`; adding duplicate columns would split source-of-truth invariants |
| Raw `UPDATE memories SET valid_to = ?` in `ResolutionType::DeleteBoth` vs `pub(crate) fn delete_memory_if_active` delegating to `delete_memory` | Accepted (`delete_memory_if_active`) | Preserves `crossrefs` invalidation, `MemoryEventType::Deleted` event logging, and `sync_state.pending_changes` parity (matching `KeepA`/`KeepB` in `#279`), while treating already soft-deleted targets as idempotent `Ok(false)` and keeping `engram-core` public Rust API (`19908` lines) unchanged |
| Keep `.unwrap_or_default()` around `find_near_duplicates` and `detect_conflicts` in `run_consolidation` vs propagate `EngramError` (ignoring only `EngramError::NotFound` on concurrently removed `mid`) | Accepted (propagate DB errors, skip `NotFound`) | `.unwrap_or_default()` masked broken SQL for months by turning schema/DB failures into empty candidate lists; skipping `NotFound` handles concurrent deletion/expiry without hiding storage errors |
| `reset_interrupted_sync`: inline `match` on `COMMIT` vs RAII `ResetTxGuard` mirroring `SavepointGuard` | Accepted (`ResetTxGuard`) | Sets `committed = true` only after `COMMIT` / `RELEASE SAVEPOINT` succeeds; `Drop` executes `ROLLBACK` when `!conn.is_autocommit()` (or `ROLLBACK TO SAVEPOINT; RELEASE SAVEPOINT` when nested), guaranteeing clean connection state on commit-time errors |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| `detect_conflicts` | `O(C)` over `C <= 100` live same-workspace candidates; tag-overlap priority uses indexed `memory_tags(memory_id, tag_id)` | `O(C)` | Uses `get_memory_readonly` so background/consolidation scans do not inflate `memories.access_count` |
| `find_near_duplicates` | `O(N^2)` character trigram comparison over `N <= 2 * limit` live memories within matching workspaces | `O(N)` | Skips cross-workspace pairs before computing n-grams; returns real `duplicate_candidates.id` idempotently while `status = 'pending'` |
| `generate_quality_report` | Indexed `workspace` scans + 2 `JOIN memories` count queries on `memory_conflicts` / `duplicate_candidates` | `O(1)` | Confines conflict/duplicate counts to live memories in `workspace_filter` |
| `reset_interrupted_sync` | `O(1)` (`ResetTxGuard` stack struct, zero heap allocation) | `O(1)` | Cold sync-worker startup path (`cloud` feature) |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Two workspaces share identical content and identical tag names (`alpha` vs `beta`) | `test_detect_conflicts_and_delete_both_use_live_schema_and_workspace_isolation` and `test_duplicates_and_quality_report_respect_valid_to_and_workspace` verify zero cross-workspace pairing or report count leakage |
| `ResolutionType::DeleteBoth` invoked twice or when one target is already soft-deleted | `delete_memory_if_active` maps `EngramError::NotFound` to `Ok(false)`; verified by repeat `DeleteBoth` call in `context_quality::tests` |
| Repeated `detect_conflicts` / `find_near_duplicates` calls before and after resolution | Verified real `id > 0` stability across reruns while unresolved/pending, and suppression once `resolved_at` is set or target is soft-deleted/archived |
| `run_consolidation` encountering real SQLite failure (missing `duplicate_candidates` table) vs normal duplicate+conflict proposals | `run_consolidation_surfaces_duplicates_conflicts_and_propagates_db_errors` in `src/intelligence/auto_consolidate.rs` |
| Deferred foreign-key constraint failing at `COMMIT` time inside `reset_interrupted_sync` after `UPDATE sync_state` succeeded | Extended `reset_interrupted_sync_supports_autocommit_and_nested_transactions` in `src/sync/worker.rs` (`--features cloud`) |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| `delete_memory_if_active` accidentally widens `engram-core` public Rust API | High (breaks `check-advertised-contracts.py`) | Mark helper and re-export `pub(crate)` | `python3 -B scripts/check-advertised-contracts.py --inventory docs/contracts/advertised-surfaces.toml` (`PASS`, `19908` lines) |
| `sync::worker` gated behind `#[cfg(feature = "cloud")]` | Medium (missed by default `--lib` runs) | Run `--features cloud` `clippy` and `cargo test --locked --features cloud --lib sync::worker` explicitly | `cargo test --locked --features cloud --lib sync::worker` + `cargo clippy --all-targets --features cloud --locked -- -D warnings` |
| `risk-register.yaml` score mismatch (`score != probability * impact`) or `CLAUDE.md` / `progress.md` budget overflow | Medium (fails `doctor.sh` / `harness-risk-register.sh validate` / `test_context_budget.py`) | Validate with `harness-risk-register.sh validate`, `doctor.sh`, `check-live-state.sh`, and `run-offline-lane.sh` | All deterministic harness gates `PASS` |

## Decision

Proceed / split / block: **Proceed** (run full `sensors.sh`, independent post-review, and strict `check-live-state.sh`, then open lean PR against `main` keeping human `H1` receipts pending and external registries untouched).
