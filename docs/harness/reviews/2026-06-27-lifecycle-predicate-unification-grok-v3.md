I'll run the harness bootstrap, read the spec, and grep the codebase for every lifecycle-state write site to verify the draft against the code.
Expanding the search for lifecycle writes and reading the key code paths the spec cites.
REVIEW_VERDICT: PASS Re-review v2 scheduler blocker is fixed; writer inventory is complete and convergence/disarm decisions are safe to implement.

## Writer inventory (grep-verified)

| Site | Classification | Spec agreement |
|---|---|---|
| `src/mcp/handlers/lifecycle.rs:178,184` → `update_memory_lifecycle_state` in `lifecycle_run` | decay-engine (canonical writer) | **Agree** — spec L43, L95-96 |
| `src/intelligence/salience.rs:450` in `run_salience_decay` | decay-engine (salience predicate) | **Agree** — spec L44, L97; verified `score < 0.2 && days_inactive >= 90` at L439-445 |
| `src/mcp/handlers/memory_policy.rs:143-152` (predicate at L352-357) → `memory_decay` | decay-engine (retention-score predicate) | **Agree** — spec L45, L98-104; `Active && new_retention < 0.25 → Stale` |
| `src/mcp/handlers/summarize.rs:329` in `memory_archive_old` | decay-engine (age/importance/access) | **Agree** — spec L46, L105-111; filter at L261-269, write at L329 |
| `src/storage/queries/retention.rs:312` in `compress_old_memories` | decay-engine (retention compression) | **Agree** — spec L47, L112-119; selects `active` at L241, archives at L312 |
| `src/storage/queries/retention.rs:157` via `apply_retention_policies` | decay-engine entry point | **Agree** — spec L51, L117-118 |
| `src/bin/server.rs:122-136` (`ENGRAM_COMPRESSION_INTERVAL` args) + `726-749` (scheduler spawn) → `compress_old_memories` | decay-engine entry point (optional scheduler) | **Agree** — **re-review v2 blocker fixed**; verified scheduler calls `compress_old_memories` at L747 and logs "archived" at L753 |
| `src/intelligence/consolidation_offline.rs:568` | domain (consolidation/supersession) | **Agree** — spec L64 |
| `src/storage/queries/retention.rs:182` (max-count in `apply_retention_policies`) | domain (workspace cap) | **Agree** — spec L65 |
| `src/storage/queries/retention.rs:202-206` (`auto_delete_after_days` → `valid_to`) | domain (visibility soft-delete, not lifecycle) | **Agree** — spec L66, L72-73; writes `valid_to`, not `lifecycle_state` |
| `src/intelligence/context_quality.rs:730,737` | domain (conflict resolution) | **Agree** — spec L67 |
| `src/mcp/handlers/dream.rs:377` (`expire` candidate) | domain (approved action) | **Agree** — spec L68, L303 |
| `src/mcp/handlers/lifecycle.rs:239` → `memory_set_lifecycle` | manual explicit | **Agree** — spec L69 |
| `src/storage/queries/lifecycle.rs:28,39` → `update_memory_lifecycle_state` | helper (callers decide semantics) | **Agree** — spec L76-77 |
| `src/storage/turso_backend.rs:675`, `migrations.rs:926` | initializer/default | **Agree** — spec L77-78 |
| `src/storage/queries/tests.rs:1543`, `benches/search.rs:328` | test/bench fixtures | **Agree** — spec L75-79 |
| `src/intelligence/salience.rs:254-278` → `suggest_lifecycle_state` | advisory (no write; third public predicate) | **Agree** — spec L154, L411-418; computes `SalienceScore.suggested_state` only |

No additional lifecycle-state writers found in `src/hooks/` or integration tests beyond fixtures.

---

## Findings

- **[MED]** `lifecycle_config` public metadata is not in the contract-cleanup list (spec L284-292, L419-423). `src/mcp/tools/registry.rs:1267-1268` and `docs/MCP_TOOLS.md:1366-1367` still advertise `min_importance` / `min_access_count` as lifecycle thresholds; the handler is a stub (`lifecycle.rs:270-285`) but clients may treat it as authoritative. Add to the implementation-plan doc sweep or deprecate those fields explicitly.

- **[MED]** `memory_archive_old` disarm is decision-clear (spec L105-111) but less implementation-explicit than `compress_old_memories`. Current code at `summarize.rs:261-269` has **no** `lifecycle_state` filter (unlike `retention.rs:241` which requires `active`). Implementers must **add** `lifecycle_state = 'archived'` to candidate selection, not only remove the UPDATE at L329, or the tool will keep creating summaries for active rows without archiving them.

- **[MED]** Retention auto-delete boundary is documented and testable (spec L316-323, test #10 at L382-385), not a spec blocker. Verified `retention.rs:202-206`: soft-delete uses `created_at < cutoff`, not time-since-archived. Once `lifecycle_run` starts producing `Archived` rows, old-by-creation memories can be soft-deleted on the next `retention_policy_apply` — preexisting semantics, but operators should be warned in rollout notes.

- **[MED]** Public MCP contracts still advertise old lifecycle side effects today; spec correctly mandates updates in the implementation plan (`docs/MCP_TOOLS.md:825`, `1284`, `1339`, `1492`; `registry.rs:642`, `1186`, `1239`, `1404`). Safe to plan — not a spec gap — but HIGH priority during implementation.

- **[LOW]** `memory_decay` write site is `memory_policy.rs:143-152`; predicate is `352-357`. Spec table cites `352-354` (spec L45) — minor line imprecision, same file/function.

- **[LOW]** Hybrid predicate arithmetic verified: `archive_days_base=90 × max_importance_mult=4.0 = 360 < hard_idle_cap_days=365` (spec L240-242, L246-247). Cap is dormant under defaults; parametrized test at L341 proves dominance when `cap=300`.

- **[LOW]** Monotonicity without `lifecycle_changed_at` is sound for this spec: pure `decide_lifecycle_state` with terminal `Archived` guard (spec L305-309, L198) and idempotence test #3 (L362). Direct `active→archived` avoids run-frequency dependence.

- **[LOW]** Zero-migration claim holds: predicate uses existing columns (spec L258-260); `normalized_importance` handles raw `f32` at storage boundaries (spec L216-227); creation-age auto-delete accepted as behavior, not schema.

---

## Validation of review questions

### 2. `compress_old_memories` / scheduler decision
The conservative "already `Archived` only" rule is correct and precise. Allowing `retention_policy_apply` or the `ENGRAM_COMPRESSION_INTERVAL` scheduler to call `decide_lifecycle_state` would recreate a **second automatic decay writer** with a divergent candidate predicate (`created_at` + importance + access_count vs recency-primary hybrid), violating invariant #1 (spec L296-301). Implementation: change L241 from `= 'active'` to `= 'archived'`, remove L312 UPDATE (or guard it), rename return/log semantics from "archived" to "compressed" (`server.rs:753`).

### 4. `memory_archive_old` convergence
Correct to disarm. Delegating to `decide_lifecycle_state` from this tool would make it a second lifecycle engine reachable via MCP (`summarize.rs:225`). Compression-only on already-archived rows preserves single-writer invariant while keeping summarization UX deferrable to a follow-up `compress_archived_memories` surface (spec L110-111).

### 5. `memory_decay` decision
Sufficient for this spec. Disarming the `lifecycle_target` write (spec L98-104) removes the divergent retention-score writer; feeding retention into `decide_lifecycle_state` is correctly deferred as a product follow-up (spec L433) rather than smuggled in as a parallel writer.

### 6. Retention auto-delete boundary
Safe to document and accept for zero-migration (spec L316-323). Not a spec blocker because behavior is preexisting, explicit, and covered by integration test #10. Operational caveat only (see MED finding above).

### 7. Advisory state + importance normalization
Sufficient and testable. `suggest_lifecycle_state` delegation to `decide_lifecycle_state` (spec L154, L379-381, L411-418) eliminates the third public predicate. `normalized_importance` with NaN/∞ handling (spec L220-227) plus table tests for clamp/NaN (L346-348) are concrete acceptance criteria.

---

**Bottom line:** The latest draft fixes the re-review v2 blocker (optional compression scheduler + both `compress_old_memories` callers are in scope and disarmed). Writer enumeration matches code. Remaining issues are implementation-plan hygiene (docs/metadata, `memory_archive_old` candidate-filter addition, operator notes on auto-delete), not spec-level blockers.
