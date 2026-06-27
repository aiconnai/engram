# Lifecycle Predicate Unification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans (or the local execution-plan runner) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Do not implement from the design spec directly; implement from this plan.

**Goal:** Unify Engram lifecycle decay so `lifecycle_run` is the only decay-derived lifecycle writer, backed by a pure canonical Rust predicate, while salience/policy/compression surfaces become score-only or compression-only.

**Architecture:** Add a dedicated lifecycle-decision module with `LifecycleConfig`, `normalized_importance`, and `decide_lifecycle_state(memory, now, cfg)`. `lifecycle_run` uses a permissive SQL pre-filter and calls this function for the final state. `run_salience_decay`, `memory_decay`, `memory_archive_old`, and `compress_old_memories` stop changing `memories.lifecycle_state`. Domain writers remain untouched: manual lifecycle set, consolidation, retention max-count, retention auto-delete, conflict resolution, and approved dream expiration.

**Tech Stack:** Rust, rusqlite, chrono, serde. No schema migration; `SCHEMA_VERSION` remains 44.

**Spec:** `docs/superpowers/specs/2026-06-27-lifecycle-predicate-unification-design.md`

**Review Evidence:** `docs/harness/reviews/2026-06-27-lifecycle-predicate-unification-chair-v3.md` (`REVIEW_VERDICT: PASS`).

---

## Must NOT Have

- Do not add `stability`, `lifecycle_changed_at`, or any schema migration.
- Do not make `run_salience_decay`, `memory_decay`, `memory_archive_old`, `compress_old_memories`, retention compression, or the server compression scheduler call `decide_lifecycle_state` and write lifecycle. That would create a second lifecycle writer.
- Do not leave any SQL pre-filter in `lifecycle_run` that excludes by `importance`, `access_count`, or `created_at` in a way that can be stricter than `decide_lifecycle_state`.
- Do not change legitimate domain writers: `memory_set_lifecycle`, `consolidation_offline`, retention max-count, retention auto-delete, context-quality conflict resolution, or approved dream expiration.
- Do not permanently delete from this lifecycle predicate. The cap archives only.
- Do not edit `docs/MCP_TOOLS.md` by hand; regenerate it with `./scripts/generate-mcp-reference.sh` after registry/source metadata changes.

---

## File Map

| Action | Path | Responsibility |
|---|---|---|
| Create | `src/intelligence/lifecycle.rs` | `LifecycleConfig`, `normalized_importance`, `decide_lifecycle_state`, pure unit tests |
| Modify | `src/intelligence/mod.rs` | Export lifecycle decision types/functions |
| Modify | `src/mcp/handlers/lifecycle.rs` | Use canonical predicate in `lifecycle_run`; update `lifecycle_config`; add integration-style handler tests |
| Modify | `src/intelligence/salience.rs` | Remove lifecycle writes from salience decay; delegate `suggested_state` to canonical predicate |
| Modify | `src/mcp/handlers/quality.rs` | Ensure salience decay response remains accurate after score-only behavior |
| Modify | `src/mcp/handlers/memory_policy.rs` | Keep policy-score decay; remove lifecycle update path from `memory_decay` |
| Modify | `src/mcp/handlers/summarize.rs` | Make `memory_archive_old` compress only already-Archived rows; remove lifecycle update |
| Modify | `src/storage/queries/retention.rs` | Make `compress_old_memories` compress only already-Archived rows; remove lifecycle update; keep retention max-count and auto-delete unchanged |
| Modify | `src/bin/server.rs` | Rename compression scheduler result/log wording from archived to compressed |
| Modify | `src/mcp/tools/registry.rs` | Update public tool descriptions/schemas for changed behavior |
| Modify | `src/mcp/tools/memory.rs` | Mirror public tool descriptions/schemas for changed behavior |
| Regenerate | `docs/MCP_TOOLS.md` | Generated MCP reference after registry/source metadata changes |
| Modify | `tests/mcp_protocol_tests.rs` | Update any protocol expectations for changed tool schemas/metadata |

---

## Task 1: Add canonical lifecycle predicate module

**Files:**
- Create: `src/intelligence/lifecycle.rs`
- Modify: `src/intelligence/mod.rs`

- [ ] **Step 1.1: Create `src/intelligence/lifecycle.rs` with config defaults**

Add:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{LifecycleState, Memory};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LifecycleConfig {
    pub stale_days_base: i64,
    pub archive_days_base: i64,
    pub hard_idle_cap_days: i64,
    pub max_importance_mult: f32,
}

impl Default for LifecycleConfig {
    fn default() -> Self {
        Self {
            stale_days_base: 30,
            archive_days_base: 90,
            hard_idle_cap_days: 365,
            max_importance_mult: 4.0,
        }
    }
}
```

Do not place this config in `SalienceConfig`.

- [ ] **Step 1.2: Add `normalized_importance`**

In the same file:

```rust
pub fn normalized_importance(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.5
    }
}
```

- [ ] **Step 1.3: Add `decide_lifecycle_state`**

In the same file:

```rust
pub fn decide_lifecycle_state(
    memory: &Memory,
    now: DateTime<Utc>,
    cfg: &LifecycleConfig,
) -> LifecycleState {
    if memory.lifecycle_state == LifecycleState::Archived {
        return LifecycleState::Archived;
    }

    let last_access = memory.last_accessed_at.unwrap_or(memory.created_at);
    let idle_days = (now - last_access).num_days();

    if idle_days >= cfg.hard_idle_cap_days {
        return LifecycleState::Archived;
    }

    let importance = normalized_importance(memory.importance);
    let mult = 1.0 + importance * (cfg.max_importance_mult - 1.0);
    let effective_stale = (cfg.stale_days_base as f32 * mult) as i64;
    let effective_arch = (cfg.archive_days_base as f32 * mult) as i64;

    if idle_days >= effective_arch {
        return LifecycleState::Archived;
    }
    if idle_days >= effective_stale {
        return LifecycleState::Stale;
    }

    LifecycleState::Active
}
```

- [ ] **Step 1.4: Export the lifecycle module**

In `src/intelligence/mod.rs`, add `pub mod lifecycle;` and re-export:

```rust
pub use lifecycle::{decide_lifecycle_state, normalized_importance, LifecycleConfig};
```

Place exports near other public intelligence exports.

- [ ] **Step 1.5: Add pure unit tests for the predicate table**

In `src/intelligence/lifecycle.rs`, under `#[cfg(test)]`, add a helper that builds a `Memory` with controllable `importance`, `created_at`, `last_accessed_at`, and `lifecycle_state`. Cover these cases exactly:

| Case | Setup | Expected |
|---|---|---|
| Fresh default | importance `0.5`, idle 5d | `Active` |
| Stale by idle | importance `0.0`, idle 35d | `Stale` |
| Archive by idle | importance `0.0`, idle 95d | `Archived` |
| Importance protects | importance `1.0`, idle 200d | `Stale` |
| Boundary 359 | importance `1.0`, idle 359d | `Stale` |
| Boundary 360 | importance `1.0`, idle 360d | `Archived` |
| Forced cap | importance `1.0`, idle 320d, cap 300d | `Archived` |
| Exact stale | importance `0.0`, idle 30d | `Stale` |
| Exact archive | importance `0.0`, idle 90d | `Archived` |
| Exact cap | importance `1.0`, idle 300d, cap 300d | `Archived` |
| One day before | importance `0.0`, idle 29d | `Active` |
| Importance > 1 | importance `2.0`, idle 359d | `Stale` |
| Importance < 0 | importance `-1.0`, idle 35d | `Stale` |
| Importance NaN | importance `f32::NAN`, idle 95d | `Stale` |
| Already archived | current `Archived`, any idle | `Archived` |
| Missing `last_accessed_at` | `None`, created_at drives idle | per expected age |

- [ ] **Step 1.6: Verify Task 1**

```bash
rtk cargo test intelligence::lifecycle --lib
rtk cargo fmt --check
```

Expected: lifecycle unit tests pass; formatting is clean.

---

## Task 2: Rewrite `lifecycle_run` to use the canonical predicate

**Files:**
- Modify: `src/mcp/handlers/lifecycle.rs`

- [ ] **Step 2.1: Import canonical lifecycle functions**

Add imports from `crate::intelligence`:

```rust
use crate::intelligence::{decide_lifecycle_state, LifecycleConfig};
```

Keep `LifecycleState` from `crate::types`.

- [ ] **Step 2.2: Replace `min_importance` parsing with `LifecycleConfig` parsing**

Inside `lifecycle_run`, parse:

- `stale_days` → `LifecycleConfig.stale_days_base`, default `30`
- `archive_days` → `LifecycleConfig.archive_days_base`, default `90`
- `hard_idle_cap_days`, default `365`
- `max_importance_mult`, default `4.0`

Accept `min_importance` if present only as deprecated/no-op compatibility; do not use it to filter or decide.

- [ ] **Step 2.3: Replace stale/archive candidate SQL with one permissive candidate query**

Use a single query that selects all non-deleted non-archived candidates in the workspace:

```sql
SELECT id, content, memory_type, importance, access_count,
       created_at, updated_at, last_accessed_at, lifecycle_state,
       workspace, tier
FROM memories
WHERE valid_to IS NULL
  AND COALESCE(lifecycle_state, 'active') != 'archived'
  -- optional workspace clause only
```

Must not filter on `importance`, `access_count`, `created_at`, or `expires_at`. If any cheap time/expiration pre-filter is added later, it must be proven more permissive than `decide_lifecycle_state`; do not add it in this implementation.

- [ ] **Step 2.4: Convert rows into `Memory` values or a local equivalent**

Prefer constructing `Memory` so `decide_lifecycle_state(&memory, now, &cfg)` is used exactly. Parse dates using existing repo patterns (`DateTime::parse_from_rfc3339(...).with_timezone(&Utc)`). If a row has NULL/unknown lifecycle, treat it as `LifecycleState::Active`.

- [ ] **Step 2.5: Compute transitions in memory and preserve dry-run/apply parity**

For each candidate:

1. `let next = decide_lifecycle_state(&memory, now, &cfg);`
2. If `next != memory.lifecycle_state`, include it in the transition list.
3. In dry-run, return counts and candidate previews for `stale` and `archived` based on this same transition list.
4. In apply mode, update only the transition list with `update_memory_lifecycle_state`.

Direct `Active -> Archived` is allowed when `decide_lifecycle_state` returns `Archived`.

- [ ] **Step 2.6: Preserve enrichment events for apply mode only**

Keep event type `lifecycle_transition`, `triggered_by: "lifecycle_run"`, and outcome `{"new_state": "stale"}` or `{"new_state": "archived"}`. Emit only for applied transitions, not for dry-run.

- [ ] **Step 2.7: Update `lifecycle_config` handler**

In `src/mcp/handlers/lifecycle.rs`, update `lifecycle_config` to return only lifecycle config fields:

- `stale_days`
- `archive_days`
- `hard_idle_cap_days`
- `max_importance_mult`
- `lifecycle_enabled`

Remove `min_importance` and `min_access_count` from response. If input includes them, ignore them; do not echo them.

- [ ] **Step 2.8: Add handler tests for lifecycle_run**

In `src/mcp/handlers/lifecycle.rs` tests, add/replace tests for:

1. high-importance + high-access + idle ≥ 360 archives despite old restrictive filters;
2. dry-run and apply produce the same candidate IDs and target states;
3. running apply twice is idempotent;
4. `Active -> Archived` direct transition is allowed;
5. `min_importance` parameter does not exclude a candidate.

Use the existing `test_lifecycle_run_emits_enrichment_event` setup style as the template.

- [ ] **Step 2.9: Verify Task 2**

```bash
rtk cargo test mcp::handlers::lifecycle --lib
```

Expected: lifecycle handler tests pass, including enrichment event test.

---

## Task 3: Make salience decay score-only and advisory state canonical

**Files:**
- Modify: `src/intelligence/salience.rs`
- Modify: `src/mcp/handlers/quality.rs`

- [ ] **Step 3.1: Stop `run_salience_decay_in_workspace` from updating lifecycle**

Remove the block in `src/intelligence/salience.rs` that executes:

```sql
UPDATE memories SET lifecycle_state = ?, updated_at = ? WHERE id = ?
```

Keep salience score calculation and `salience_history` insertion behavior.

- [ ] **Step 3.2: Keep `DecayResult` shape but make lifecycle counts zero**

For compatibility, keep `marked_stale` and `suggested_archive` fields in `DecayResult`, but make them represent state transitions performed by salience decay. Since salience decay is now score-only, they must remain `0` in apply and dry-run.

- [ ] **Step 3.3: Delegate `SalienceScore.suggested_state` to canonical predicate**

In `SalienceCalculator::calculate`, replace the legacy `self.suggest_lifecycle_state(memory, score, now)` call with:

```rust
let suggested_state = decide_lifecycle_state(memory, now, &LifecycleConfig::default());
```

Import `decide_lifecycle_state` and `LifecycleConfig`. The salience score still computes `score`; it no longer owns lifecycle decision logic.

- [ ] **Step 3.4: Remove or rewrite `suggest_lifecycle_state` tests**

The test at `src/intelligence/salience.rs:965` currently validates the old score-gated predicate. Replace it with a parity test: a memory passed through `SalienceCalculator::calculate` must have `score.suggested_state == decide_lifecycle_state(memory, now-ish, &LifecycleConfig::default())`. Use dates stable enough that the day boundary cannot flake; prefer constructing `now` and helper APIs if needed.

- [ ] **Step 3.5: Update salience_decay_run handler response wording if needed**

In `src/mcp/handlers/quality.rs`, ensure `salience_decay_run` does not claim lifecycle states were updated. If it surfaces `marked_stale` / `suggested_archive`, those values should be `0` and docs should explain salience decay is score/history-only.

- [ ] **Step 3.6: Verify Task 3**

```bash
rtk cargo test intelligence::salience --lib
rtk cargo test mcp::handlers::quality --lib
```

Expected: salience tests pass; salience decay no longer writes lifecycle.

---

## Task 4: Disarm `memory_decay` lifecycle writes while preserving policy scores

**Files:**
- Modify: `src/mcp/handlers/memory_policy.rs`

- [ ] **Step 4.1: Remove lifecycle target calculation**

In `decay_candidates`, change `lifecycle_target` so it is always `None`. Keep current policy score fields (`new_salience_score`, `new_retention_score`, `new_retrieval_priority`) unchanged.

- [ ] **Step 4.2: Remove apply-mode lifecycle UPDATE**

In `memory_decay`, remove the block that executes raw SQL:

```sql
UPDATE memories
SET lifecycle_state = ?1
WHERE id = ?2
  AND valid_to IS NULL
  AND COALESCE(lifecycle_state, 'active') = 'active'
```

Keep `upsert_policy_record` and `emit_policy_event`.

- [ ] **Step 4.3: Keep compatibility response fields but make lifecycle updates zero**

Keep `lifecycle_updates` in the JSON response if tests/clients expect it, but set it to `0` and update `concern` to say only policy scores are updated.

- [ ] **Step 4.4: Add regression test**

Add a handler test that creates an Active memory whose decayed retention would previously cross `< 0.25`, runs `memory_decay(dry_run=false)`, and asserts:

- policy record was updated;
- `lifecycle_state` remains `active`;
- response `lifecycle_updates == 0`.

- [ ] **Step 4.5: Verify Task 4**

```bash
rtk cargo test mcp::handlers::memory_policy --lib
```

Expected: policy score tests pass and lifecycle state remains unchanged.

---

## Task 5: Disarm `memory_archive_old` lifecycle writes

**Files:**
- Modify: `src/mcp/handlers/summarize.rs`

- [ ] **Step 5.1: Filter candidates to already Archived rows**

In `memory_archive_old`, candidate filtering currently checks age, importance, access count, and type but no lifecycle state. Add an explicit check:

```rust
m.lifecycle_state == LifecycleState::Archived
```

Import `LifecycleState` if needed. This is required; simply removing the final update would still summarize active rows via the old divergent predicate.

- [ ] **Step 5.2: Remove final lifecycle UPDATE**

Delete the `conn.execute("UPDATE memories SET lifecycle_state = 'archived' ...")` block. After `create_memory(conn, &input)` succeeds, count the row as compressed/summarized without updating the original lifecycle.

- [ ] **Step 5.3: Rename response and event wording**

Change response keys from archive semantics to compression semantics:

- `would_archive` → `would_compress`
- `archived` → `compressed`

For compatibility, only keep old keys if existing protocol tests require them, and if kept, mark them deprecated in docs. Event outcome should not be `{"new_state": "archived"}`; use `{"compressed": true, "summary_created": true}` or equivalent.

- [ ] **Step 5.4: Add regression tests**

Add tests that:

1. create an old, low-importance, low-access Active memory and run `memory_archive_old(dry_run=false)`; assert no summary is created and lifecycle remains Active;
2. create an old, low-importance, low-access Archived memory and run apply; assert a Summary is created and original remains Archived.

- [ ] **Step 5.5: Verify Task 5**

```bash
rtk cargo test mcp::handlers::summarize --lib
```

Expected: summarize handler tests pass; `memory_archive_old` no longer archives originals.

---

## Task 6: Disarm `compress_old_memories` for retention and scheduler callers

**Files:**
- Modify: `src/storage/queries/retention.rs`
- Modify: `src/bin/server.rs`

- [ ] **Step 6.1: Change `compress_old_memories` candidate SQL to already Archived rows**

In `src/storage/queries/retention.rs`, change:

```sql
AND COALESCE(m.lifecycle_state, 'active') = 'active'
```

to:

```sql
AND COALESCE(m.lifecycle_state, 'active') = 'archived'
```

Keep `valid_to IS NULL`, expiration, type exclusions, and batch limit.

- [ ] **Step 6.2: Remove lifecycle UPDATE from `compress_old_memories`**

Delete the `UPDATE memories SET lifecycle_state = 'archived' ...` call inside `compress_old_memories`. Count a candidate only when summary creation succeeds.

- [ ] **Step 6.3: Rename local variables and comments**

Rename `archived` local counter to `compressed`. Update comments and doc comment:

- from “Auto-compress old, rarely-accessed memories by creating summaries and archiving originals”
- to “Compress already-archived memories by creating summary rows.”

Keep function name for compatibility unless the codebase already supports a safe rename.

- [ ] **Step 6.4: Keep retention max-count and auto-delete unchanged**

Do not change:

- `retention.rs:182` max-count archival;
- `retention.rs:202` auto-delete by `created_at` for already-Archived rows.

Those are domain writers/visibility changes accepted by the spec.

- [ ] **Step 6.5: Update server scheduler log wording**

In `src/bin/server.rs`, change:

```rust
tracing::info!("Compression scheduler archived {} memories", archived);
```

to compression wording, e.g.:

```rust
tracing::info!("Compression scheduler compressed {} archived memories", compressed);
```

Do not remove the scheduler; it remains optional and disabled by default.

- [ ] **Step 6.6: Add retention query tests**

Add tests for `compress_old_memories` that prove:

1. old Active candidates are not changed and not summarized;
2. old Archived candidates are summarized and remain Archived;
3. `apply_retention_policies` with `compress_after_days` follows the same behavior;
4. `auto_delete_after_days` still soft-deletes Archived rows by `created_at` when explicitly configured.

Place tests in the existing storage query test area (`src/storage/queries/tests.rs`) if that is where query tests live; otherwise add a local `#[cfg(test)]` module in `retention.rs` following repo convention.

- [ ] **Step 6.7: Verify Task 6**

```bash
rtk cargo test retention --lib
rtk cargo test storage::queries --lib
```

Expected: retention tests pass; no compression path archives Active/Stale rows.

---

## Task 7: Update MCP contracts and generated reference

**Files:**
- Modify: `src/mcp/tools/registry.rs`
- Modify: `src/mcp/tools/memory.rs`
- Modify: `tests/mcp_protocol_tests.rs`
- Regenerate: `docs/MCP_TOOLS.md`

- [ ] **Step 7.1: Update `memory_decay` metadata**

In both registry files, change the description/schema text for `memory_decay` so it says:

- updates memory policy scores;
- does not transition `lifecycle_state`;
- use `lifecycle_run` for lifecycle transitions.

- [ ] **Step 7.2: Update `memory_archive_old` metadata**

In both registry files, change the description/schema text for `memory_archive_old` so it says:

- compresses/summarizes rows already `Archived`;
- does not move originals to archived state;
- lifecycle archival is handled by `lifecycle_run`.

Update `min_access_count` wording if it remains: it is now a compression eligibility filter for already-Archived rows, not an archival filter.

- [ ] **Step 7.3: Update `salience_decay_run` metadata**

In both registry files, change `salience_decay_run` text so it says score/history decay only and explicitly does not update lifecycle state.

- [ ] **Step 7.4: Update `lifecycle_run` metadata**

Remove `min_importance` from `lifecycle_run` input schema if compatibility allows. If compatibility requires accepting it, leave handler compatibility but remove it from public schema or mark it deprecated/no-op. It must not be described as a candidate-selection filter.

- [ ] **Step 7.5: Update `lifecycle_config` metadata and handler schema**

Remove `min_importance` and `min_access_count` from `lifecycle_config` public schema and docs. Add `hard_idle_cap_days` and `max_importance_mult` if the tool is meant to describe lifecycle config. Keep `stale_days` and `archive_days`.

- [ ] **Step 7.6: Update MCP protocol tests**

Run protocol tests first to see failures:

```bash
rtk cargo test --test mcp_protocol_tests
```

Update expected schemas/descriptions in `tests/mcp_protocol_tests.rs` only to match the intentional public contract changes above. Do not weaken unrelated checks.

- [ ] **Step 7.7: Regenerate MCP reference**

```bash
rtk ./scripts/generate-mcp-reference.sh
rtk ./scripts/generate-mcp-reference.sh --check
```

Expected: first command writes `docs/MCP_TOOLS.md`; second command exits 0.

- [ ] **Step 7.8: Verify Task 7**

```bash
rtk cargo test --test mcp_protocol_tests
rtk ./scripts/generate-mcp-reference.sh --check
```

Expected: protocol tests pass; generated reference is up to date.

---

## Task 8: Add cross-surface regression tests for single-writer behavior

**Files:**
- Modify: relevant existing test modules in `src/mcp/handlers/lifecycle.rs`, `src/intelligence/salience.rs`, `src/mcp/handlers/memory_policy.rs`, `src/mcp/handlers/summarize.rs`, `src/storage/queries/tests.rs`

- [ ] **Step 8.1: Add a grep-based regression check to the plan execution notes**

After implementation, run:

```bash
rtk rg -n "SET lifecycle_state|UPDATE memories SET lifecycle_state|update_memory_lifecycle_state\(" src
```

Expected remaining lifecycle writers:

- `lifecycle_run` via `update_memory_lifecycle_state`;
- `memory_set_lifecycle` manual;
- `retention_policy_apply` max-count domain writer;
- `consolidation_offline` domain writer;
- `context_quality` conflict-resolution domain writer;
- `dream.rs` approved expiration domain writer;
- query-layer helper `storage/queries/lifecycle.rs`;
- initializers/tests/fixtures.

No remaining lifecycle writes in `run_salience_decay`, `memory_decay`, `memory_archive_old`, or `compress_old_memories`.

- [ ] **Step 8.2: Add dry-run/apply parity test for lifecycle**

If not already covered in Task 2, assert dry-run reports exactly the same IDs/target states that apply mode changes on a fresh DB clone/fixture.

- [ ] **Step 8.3: Add retention auto-delete boundary test**

Create a memory old by `created_at`, run `lifecycle_run` so it becomes Archived, assert `valid_to` remains NULL. Then configure/apply retention `auto_delete_after_days`, assert `valid_to` becomes non-NULL. This documents that deletion is explicit retention policy, not lifecycle cap behavior.

- [ ] **Step 8.4: Verify Task 8**

```bash
rtk cargo test lifecycle --lib
rtk cargo test salience --lib
rtk cargo test memory_policy --lib
rtk cargo test summarize --lib
rtk cargo test retention --lib
```

Expected: all targeted regression tests pass.

---

## Task 9: Full verification and review gate

**Files:**
- No additional source edits unless earlier verification finds a real bug.

- [ ] **Step 9.1: Format and lint**

```bash
rtk cargo fmt --check
rtk cargo clippy --all-targets --all-features -- -D warnings
```

Expected: both exit 0. If clippy fails on preexisting unrelated issues, capture exact output and do not hide it.

- [ ] **Step 9.2: Run core tests**

```bash
rtk cargo test
```

Expected: exits 0. If full test runtime is too high, run the targeted tests from Tasks 1-8 plus `rtk make ci` if that is the repo's current CI lane.

- [ ] **Step 9.3: Run MCP reference and harness checks**

```bash
rtk ./scripts/generate-mcp-reference.sh --check
rtk bash docs/harness/bin/doctor.sh
rtk bash docs/harness/bin/sensors.sh
```

Expected: all pass. If `sensors.sh` is too slow or fails for a known optional dependency, document the exact known-issue path before using an exclusion.

- [ ] **Step 9.4: Run final lifecycle writer inventory**

```bash
rtk rg -n "SET lifecycle_state|UPDATE memories SET lifecycle_state|update_memory_lifecycle_state\(" src
```

Expected: output contains only canonical/domain/helper/test write sites listed in Step 8.1. Explicitly inspect any new line before declaring success.

- [ ] **Step 9.5: Manual QA gate through MCP handler surface**

Drive the behavior through handlers, not only unit tests. Add or use an existing handler test/harness that creates memories and invokes:

1. `lifecycle_run(dry_run=true)`;
2. `lifecycle_run(dry_run=false)`;
3. `salience_decay_run`;
4. `memory_decay(dry_run=false)`;
5. `memory_archive_old(dry_run=false)`;
6. `retention_policy_apply` with `compress_after_days`;
7. `retention_policy_apply` with `auto_delete_after_days`.

Expected observations:

- only `lifecycle_run` performs decay-derived `Active/Stale -> Stale/Archived`;
- salience/policy/compression surfaces do not change lifecycle;
- retention auto-delete changes `valid_to` only after explicit retention apply;
- archived memories remain excluded from search through existing search filters.

- [ ] **Step 9.6: Prepare implementation review artifact**

After implementation and local verification, run the repo review gate appropriate for post-implementation:

```bash
rtk bash docs/harness/bin/review-gate.sh post lifecycle-predicate-unification
```

Expected: review artifact under `docs/harness/reviews/` with `REVIEW_VERDICT: PASS` before merge.

---

## Commit Plan

Use small, revertable commits. Suggested grouping:

1. `test(intelligence): cover canonical lifecycle predicate` — failing/passing pure predicate tests plus new module.
2. `fix(lifecycle): route lifecycle_run through canonical predicate` — handler rewrite and lifecycle tests.
3. `fix(salience): make salience decay score-only` — salience changes and tests.
4. `fix(memory): disarm policy and compression lifecycle writers` — `memory_decay`, `memory_archive_old`, `compress_old_memories`, server log wording, tests.
5. `docs(mcp): update lifecycle tool contracts` — registry/memory metadata, protocol tests, generated `docs/MCP_TOOLS.md`.

Do not batch unrelated dirty work into these commits.

---

## Rollback Plan

If implementation causes unacceptable behavior:

1. Revert the implementation commits in reverse order.
2. Keep the design spec commit unless the design itself is invalidated by a new writer discovery.
3. If only public metadata is wrong, revert just the docs/registry commit and regenerate `docs/MCP_TOOLS.md` from the restored registry.
4. If lifecycle transitions are wrong, revert the `lifecycle_run` commit first; this restores manual lifecycle behavior without touching domain writers.

---

## Acceptance Criteria

- `decide_lifecycle_state` is the only decay predicate that decides `Active/Stale -> Stale/Archived`.
- `lifecycle_run` is the only decay-derived lifecycle writer.
- `run_salience_decay` writes scores/history only.
- `memory_decay` writes policy scores only.
- `memory_archive_old` and `compress_old_memories` compress only already-Archived rows and never archive active/stale rows.
- Optional compression scheduler remains optional and compression-only.
- `SalienceScore.suggested_state` delegates to the canonical predicate.
- Public MCP metadata no longer advertises deprecated lifecycle side effects or `min_importance`/`min_access_count` lifecycle thresholds.
- `docs/MCP_TOOLS.md` is regenerated and `./scripts/generate-mcp-reference.sh --check` passes.
- No schema migration; `SCHEMA_VERSION` remains 44.
- Cross-model post-implementation review returns PASS.
