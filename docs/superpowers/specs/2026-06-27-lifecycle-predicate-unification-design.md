# Lifecycle Predicate Unification — Design Spec

Date: 2026-06-27
Owner: Ronaldo (decisions) + agent (drafting)
Status: Codex re-review v3 PASS — Grok artifact invalid; pending real second-reviewer decision
Supersedes: `docs/superpowers/specs/2026-06-26-stability-spacing-effect-design.md`

## Summary

Engram has **multiple writers** of `memories.lifecycle_state`. Review found
**four MCP-facing decay-policy/compression tools plus one optional server
compression scheduler path**, each currently capable of applying a divergent
lifecycle transition. In production, lifecycle-derived state has not transitioned
(1,179 memories: `archived=0`, `stale=0`) because the lifecycle tool is manual and
the compression scheduler defaults to disabled. Several other writers are
legitimate *domain* writers (retention max-count, conflict resolution,
consolidation, dream-approved actions, manual) that act on `lifecycle_state` for
non-decay reasons and must coexist. This spec unifies **decay/compression-derived
lifecycle transitions** into one canonical predicate and one lifecycle-state
writer for decay. It uses a hybrid temporal model (recency-primary + absolute-idle
terminal cap) with importance as a continuous resistance modulator (not a binary
gate). It does NOT touch legitimate domain writers except to draw the boundary
explicitly.

It is scoped to **fix what happens when lifecycle runs**, not to add a lifecycle
scheduler (that is a deferred follow-up). The preexisting optional compression
scheduler is not a lifecycle scheduler after this spec: it may remain as
compression-only work, but it must not turn active/stale memories into archived
ones. The `stability`/spacing-effect feature is **not** in this spec — it returns
as a follow-up that adds a recency multiplier on top of this corrected foundation.

## Problem (verified against code + production)

### The writer ecosystem (full enumeration, verified)

A `grep` for every `lifecycle_state` write across `src/` reveals **more than two**
writers. They split into two classes:

**Decay-policy/compression paths — divergent predicates, MUST converge (this spec):**

| Path | Predicate | Temporal var | Site |
|---|---|---|---|
| `lifecycle_run` MCP tool | `importance < X AND age AND access_count < 5` | `created_at` | `lifecycle.rs:178,184` |
| `run_salience_decay` MCP tool | `score < 0.2 AND days_idle >= 90` | `last_accessed_at` | `salience.rs:439-460` |
| `memory_decay` MCP tool (policy) | `Active AND new_retention < 0.25 → Stale` | retention score | `memory_policy.rs:352-354` |
| `memory_archive_old` MCP tool | `created_at < cutoff AND importance <= max AND access_count < min` | `created_at` | `summarize.rs:231-269,329` |
| `compress_old_memories` retention/scheduler path | `created_at < cutoff AND importance <= max AND access_count < min AND active → archive original` | `created_at` | `retention.rs:157,217-318`; `server.rs:122-136,726-749` |

Four paths are **manual MCP tools**. The fifth path is the optional server
compression scheduler (`ENGRAM_COMPRESSION_INTERVAL`, default `0`) and the
explicit retention-policy compression call (`compress_after_days`). It calls
`compress_old_memories`, creates a summary, and currently archives the original by
setting `lifecycle_state='archived'`. The prior "manual-only" framing was false:
there is no lifecycle scheduler, but there is an optional compression scheduler
that currently doubles as a lifecycle writer and therefore must converge too. The
original "two writers" framing missed `memory_decay`; the second framing missed
`memory_archive_old`; the third missed the scheduled `compress_old_memories` entry
point. All misses are part of the audit trail.

**Domain writers — legitimate non-decay reasons, COEXIST (out of scope):**

| Writer | Reason | Site |
|---|---|---|
| `consolidation_offline` | archives on consolidation/supersession | `consolidation_offline.rs:568` |
| `retention_policy_apply` max-count | explicit workspace cap policy | `retention.rs:182` |
| `retention_policy_apply` auto-delete | explicit retention soft-delete of already-archived rows | `retention.rs:204` |
| `context_quality::resolve_conflict` | explicit conflict resolution (`KeepA`/`KeepB`) | `context_quality.rs:730,737` |
| `dream` candidate apply | reviewed/approved action | `dream.rs:377` |
| `memory_set_lifecycle` | manual explicit | `lifecycle.rs:239` |

These are **not** concurrent decay engines. They are deliberate, non-decay
lifecycle/visibility mutations (auto-delete writes `valid_to`, not
`lifecycle_state`) and are explicitly preserved.

Additional write sites are helpers/initializers/tests, not lifecycle engines:
`storage/queries/lifecycle.rs:28,39` is the query-layer helper whose callers
decide domain/engine semantics; `turso_backend.rs:651-675` and
`migrations.rs:926` initialize/default `active`; handler/storage tests set up
fixtures.

`retention_policy_apply` compression is intentionally absent from the preserved
writer list while `compress_old_memories` writes `lifecycle_state`: that behavior
is in the convergence scope above. After implementation, retention compression may
remain as a domain operation only if it compresses/summarizes rows that are already
`Archived` and does not decide that a memory becomes archived.

> **Note (`memory_policy.rs:146`, `summarize.rs:329`, `retention.rs:312`):**
> `memory_decay`, `memory_archive_old`, and `compress_old_memories` currently write
> lifecycle state outside the canonical lifecycle decision. This spec's decisions
> for them (below) supersede those writes regardless of whether the caller is an
> MCP tool, an explicit retention policy apply, or the optional server scheduler.

#### Decision for the decay/compression paths

- `lifecycle_run` → becomes the **single canonical decay writer** (uses
  `decide_lifecycle_state`).
- `run_salience_decay` → **disarmed**, score-only (remove `salience.rs:439-460`).
- `memory_decay` (policy) → **disarmed** of its `lifecycle_target` transition
  (`memory_policy.rs:352-357` returns `None` for lifecycle; it keeps writing
  policy *scores* — retention/priority — but no longer transitions
  `lifecycle_state`). Rationale: its retention-score predicate is another
  divergent model; consolidating it into `decide_lifecycle_state` is the same
  unification. If a retention-score signal should influence decay, that is a
  mandatory follow-up that feeds the canonical predicate, not a parallel writer.
- `memory_archive_old` → **disarmed** of lifecycle transitions in this spec. Its
  current age/importance/access-count predicate is the same class of decay
  predicate being removed from `lifecycle_run`. To preserve the single-writer
  invariant, it must stop setting `lifecycle_state`; compression/summarization can
  continue only for rows already `Archived` by the canonical lifecycle path, or
  move to a follow-up that explicitly redesigns the tool as
  `compress_archived_memories`.
- `compress_old_memories` (retention compression + optional server scheduler) →
  **disarmed** of lifecycle transitions in this spec. This is the concrete
  resolution of the re-review v2 blocker. The function may create summaries only
  for rows already `Archived`, or it may be split/renamed in a follow-up, but it
  must not change `Active`/`Stale` rows to `Archived`. This applies equally to
  `retention_policy_apply` (`compress_after_days`) and to the background
  compression scheduler enabled by `ENGRAM_COMPRESSION_INTERVAL`. Any log/result
  wording that currently says "archived" must become "compressed" or equivalent.

### Why the salience predicate is structurally inert

Verified in `salience.rs:254-278`: archive requires `score < 0.2 AND
days_inactive >= 90` (ANDed). A default memory (importance=0.5, feedback default
0.5 at `salience.rs:293`) scores ~0.27 from importance+feedback alone, permanently
above 0.2. So the score gate is dead for typical memories; only the day-count
gate could ever drive archival — and stability (as the old spec specced) only
moved the recency component (weight 0.30), giving near-zero archival effect. The
old stability spec was built on this inert predicate; its review rounds (1-2)
added real rigor (the importance floor, the AND/OR asymmetry, the terminal
finding) which is carried forward here as context, but its *design* does not
survive.

### The terminal finding

Both cross-model reviewers (Codex + Fugu) ranked above all else: high-importance
abandoned memories would never be cleaned up without a hard-idle cap. This spec
closes that hole structurally.

## Decisions (locked with owner, 2026-06-26 → 2026-06-27)

| # | Question | Decision |
|---|---|---|
| Problem framing | What to fix | (A) Unify the predicates first, then a slice of (C) redesign |
| Temporal model | When does a memory decay | (3) Hybrid: recency primary (`last_accessed_at`) + absolute-idle terminal cap |
| Importance role | How importance interacts | (A) Protects from normal decay, NOT from the terminal cap — no immortality |
| Old spec relation | stability spec | (1) This spec supersedes it; stability returns as a follow-up |
| Architecture | Multiple decay/compression lifecycle writers | (C) Single automatic decay writer; `run_salience_decay`, `memory_decay`, `memory_archive_old`, and `compress_old_memories` stop writing lifecycle; domain writers untouched |
| Writer impl | How the predicate lives | (A) Canonical pure Rust fn `decide_lifecycle_state`; SQL is a permissive pre-filter only |
| Importance shape | gate vs modulator | Continuous multiplier `1.0 + imp*(MAX_MULT-1.0)`, not a binary threshold |
| Terminal cap | include now or defer | (i) Include now as a **dormant** composability guard-rail, proven by a parametrized test |
| Config | where parameters live | Dedicated `LifecycleConfig` (NOT folded into `SalienceConfig`) |
| Migration | schema change | **Zero-migration**; `SCHEMA_VERSION` stays 44; stability's migration 45 travels with the follow-up |
| Advisory state | `SalienceScore.suggested_state` | Keep response shape but compute it through `decide_lifecycle_state`; no legacy predicate survives |
| Execution | scheduler | No new lifecycle scheduler; existing optional compression scheduler is constrained to compression-only and cannot write lifecycle |

## Architecture

**Single automatic decay writer (C).** `run_salience_decay` stops writing
`lifecycle_state` (remove `salience.rs:~439-460` state block) and becomes
score-only. `memory_decay` keeps policy-score updates but stops lifecycle
transitions. `memory_archive_old` stops archiving by its own age/importance/access
predicate. `compress_old_memories` stops archiving originals by its own
age/importance/access predicate for both retention-policy and scheduler callers.
`lifecycle_run` becomes the only engine that applies decay-derived transitions.

**Canonical predicate (A).** Extract the decision into a pure function:

```rust
fn decide_lifecycle_state(
    memory: &Memory,
    now: DateTime<Utc>,
    cfg: &LifecycleConfig,
) -> LifecycleState
```

`lifecycle_run` uses SQL only as a **permissive candidate pre-filter** (cheap
selection); the final decision is always this function. **Safety invariant:** the
pre-filter must never be *more restrictive* than the function — it only narrows
the set to inspect, never excludes a candidate the function would transition.

```
lifecycle_run handler
  ├─ SQL pre-filter (permissive: e.g. lifecycle_state='active' OR 'stale')
  ├─ for each candidate: decide_lifecycle_state(m, now, cfg)
  └─ apply transitions (monotonic, idempotent)
```

## The hybrid predicate

```
idle_days = (now − last_accessed_at.unwrap_or(created_at)).num_days()
```

Decision order (first match wins):

```rust
if memory.lifecycle_state == Archived { return Archived; }   // terminal, never reverts here

// 1. TERMINAL CAP — composability guard-rail by absolute idle time.
//    Applies to ALL, including max-importance. Dormant under current defaults.
if idle_days >= cfg.hard_idle_cap_days { return Archived; }   // default 365

// 2. NORMAL DECAY by recency — importance grants resistance (extended windows).
let importance      = normalized_importance(memory.importance);                  // finite, clamped [0,1]
let mult            = 1.0 + importance * (cfg.max_importance_mult - 1.0);
let effective_stale = (cfg.stale_days_base   as f32 * mult) as i64;              // default 30×mult
let effective_arch  = (cfg.archive_days_base as f32 * mult) as i64;              // default 90×mult

if idle_days >= effective_arch  { return Archived; }
if idle_days >= effective_stale { return Stale;    }

Active
```

`normalized_importance` is deliberately part of the predicate, not assumed from
storage. Existing create/update paths accept raw `f32` importance in several
places, so the lifecycle decision normalizes defensively:

```rust
fn normalized_importance(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.5
    }
}
```

### Why the cap uses idle, not creation age

If the cap used creation age, a memory created 400 days ago but **accessed
yesterday** would archive — contradicting the chosen recency model (3). The cap
is "abandoned too long" (`idle_days >= 365`), not "old". Importance protects from
*normal* decay (up to 360 days at max), but genuine abandonment for a full year
is not survivable.

### The terminal hole is closed by normal decay, not the cap

With `archive_base=90 × MAX_MULT=4.0 = 360d`, a max-importance memory archives at
360 days idle **by the normal rule**. The terminal finding (high-importance never
archives) is closed by the **continuous modulation**, not by the cap.

### The cap is a dormant composability guard-rail

Under current defaults the cap (365) is **redundant** for importance-only memories
(normal archive maxes at 360 < 365). It is included now as a **safety invariant**,
not an optimization: it guarantees that when `stability` (the follow-up) adds a
*second* multiplier on top of importance, the effective window cannot exceed an
absolute ceiling and recreate "immortal by accident" (the exact failure the old
spec's 4.0 ceiling tried to avoid). The spec documents it as dormant under
defaults and **proves it dominates** when configured below the effective archive
window (test below). "Dormant" must be distinguished from "dead" — the inert
`suggest_lifecycle_state` is the cautionary precedent.

## Data model & config

**Zero-migration.** The predicate uses existing columns: `last_accessed_at`,
`created_at`, `importance`, `lifecycle_state`. No new column. `SCHEMA_VERSION`
stays **44**. The `stability` column + migration 45 belong to the follow-up.

**Dedicated `LifecycleConfig`** (NOT folded into `SalienceConfig`, because the
architectural decision is to *separate* salience-scoring from lifecycle):

```rust
pub struct LifecycleConfig {
    pub stale_days_base: i64,      // default 30
    pub archive_days_base: i64,    // default 90
    pub hard_idle_cap_days: i64,   // default 365 (dormant under defaults)
    pub max_importance_mult: f32,  // default 4.0
}
```

`SalienceConfig` keeps only weights / half-life / frequency / salience-history.
MCP-parameter compatibility (`stale_days`/`archive_days`) is handled by
translating at the handler boundary into `LifecycleConfig`, without mixing the
internal structs.

**Behavior migration (not schema):** `salience_decay_run`, `memory_decay`,
`memory_archive_old`, and `compress_old_memories` stop transitioning state. This
is documented in contract docs/tests; it is a behavior change, not a schema
migration.

**Public contract migration (docs/registry, not schema):** every public surface
whose advertised behavior changes must be updated in the implementation plan:
`docs/MCP_TOOLS.md`, `src/mcp/tools/registry.rs`, and the mirrored tool metadata in
`src/mcp/tools/memory.rs` must stop claiming that `salience_decay_run`,
`memory_decay`, or `memory_archive_old` perform lifecycle transitions. The
`lifecycle_run.min_importance` parameter must stop being advertised as a selection
filter; for backward compatibility the handler may accept it as a deprecated/no-op
or translate it only into reporting, but it must not exclude candidates from the
canonical predicate. The same cleanup applies to `lifecycle_config`: public
metadata/response docs must remove or deprecate `min_importance` and
`min_access_count` as lifecycle thresholds, because `LifecycleConfig` has only
`stale_days_base`, `archive_days_base`, `hard_idle_cap_days`, and
`max_importance_mult`.

## Invariants (canonical — go to the gate)

1. **Single automatic decay/lifecycle-policy writer.** Only `lifecycle_run`
   applies automatic transitions derived from `decide_lifecycle_state`.
   `run_salience_decay` is score-only; `memory_decay` is policy-score-only;
   `memory_archive_old` is not allowed to transition lifecycle by age/importance/
   access-count; `compress_old_memories` is not allowed to transition lifecycle
   for either retention-policy or compression-scheduler callers. Explicit /
   manual-domain writes (`memory_set_lifecycle`, consolidation, retention
   max-count, conflict resolution, confirmed expiration at `dream.rs:377`) are
   **not** concurrent decay engines and coexist legitimately.
2. **Monotonicity without automatic reversal.** `decide_lifecycle_state` only
   advances: `active → stale → archived` or `active → archived` directly. Never
   `archived → *`, never `stale → active` automatically. Direct `active →
   archived` is permitted when the archive predicate is already satisfied
   (avoids run-frequency dependence — no `lifecycle_changed_at` needed).
   Reactivation is explicit-only (out of scope).
3. **Idempotence under repetition.** Running `lifecycle_run` N times ≡ running
   once (consequence of monotonicity + pure function).
4. **Permissive pre-filter.** The SQL selection never excludes a candidate that
   `decide_lifecycle_state` would transition.
5. **Cap archives, never directly deletes.** *This spec's* cap never deletes —
   it only transitions to `Archived`. The preexisting `retention_policy_apply`
   auto-delete (`retention.rs:204`) is a separate, explicitly-configured
   retention domain. Its current semantics are **creation-age based**, not
   "time since archived": a newly archived 400-day-old memory can be soft-deleted
   by a later retention-policy apply if `auto_delete_after_days` is configured.
   This spec accepts and documents that preexisting behavior to preserve
   zero-migration; changing it requires a follow-up with `lifecycle_changed_at`
   or equivalent archival timestamp.

## Test strategy

**Principle:** every tested gate must prove it fires AND that it does not — the
inert `suggest_lifecycle_state` and the dormant cap share the failure mode that a
predicate never forced to fire can be dead without signal.

### Pure-function tests (`decide_lifecycle_state`, no I/O), parametrized

| Case | importance | idle_days | config | Expected | Proves |
|---|---|---|---|---|---|
| Fresh default | 0.5 | 5 | default | `Active` | nothing decays early |
| Stale by idle | 0.0 | 35 | default | `Stale` | stale gate fires (base 30) |
| Archive by idle | 0.0 | 95 | default | `Archived` | archive gate fires (base 90) |
| Importance protects from archive | 1.0 | 200 | default | `Stale` | mult 4.0 → eff_stale=120 (≥, so Stale), eff_arch=360 (not yet) |
| Calibration boundary − | 1.0 | 359 | default | `Stale` | regression-lock the numbers |
| Calibration boundary + | 1.0 | 360 | default | `Archived` | terminal hole closed by normal rule |
| **Cap FORCED** | 1.0 | 320 | `cap=300` | `Archived` | **cap dominates** when effective_arch=360 > cap=300, idle≥cap |
| Stale==threshold | 0.0 | 30 | default | `Stale` | exact boundary fires |
| Archive==threshold | 0.0 | 90 | default | `Archived` | exact boundary fires |
| Cap==threshold | 1.0 | 300 | `cap=300` | `Archived` | exact cap fires |
| One day before | 0.0 | 29 | default | `Active` | −1 does not fire |
| Importance > 1 clamps | 2.0 | 359 | default | `Stale` | max multiplier stays 4.0 |
| Importance < 0 clamps | -1.0 | 35 | default | `Stale` | min multiplier stays 1.0 |
| Importance NaN defaults | NaN | 95 | default | `Stale` | non-finite input cannot poison multiplier |
| Already archived | any | any | default | `Archived` | terminal no-revert |
| No last_accessed | 0.0 | (via created_at) | default | per age | `unwrap_or(created_at)` |

### Integration tests (`lifecycle_run`, real SQLite)

1. **Pre-filter excludes nothing valid (regression of the original bug):** a
   memory with `importance=1.0` AND high `access_count` AND idle ≥ archive must
   still be archived. If the SQL keeps any restrictive `importance`/`access_count`
   filter, this fails. *(This is the test that proves the original
   `importance < X AND access_count < 5` gate is gone.)*
2. **End-to-end transitions:** populate varied idle ages, run `lifecycle_run`,
   assert resulting `stale`/`archived` counts. *(The test that, if it had
   existed, would have caught `archived=0`.)*
3. **Idempotence:** run `lifecycle_run` twice; final state == running once.
4. **Dry-run parity:** `lifecycle_run(dry_run=true)` reports exactly the
   candidates that `dry_run=false` would apply.
5. **Behavior regression:** `salience_decay_run` does NOT change
   `lifecycle_state` (only score). Proves writer C was disarmed.
6. **Behavior regression:** `memory_decay(dry_run=false)` updates policy scores
   but does NOT change `lifecycle_state`. Proves the retention-score writer was
   disarmed without removing the policy-decay surface.
7. **Behavior regression:** `memory_archive_old(dry_run=false)` no longer changes
   `lifecycle_state` through its age/importance/access predicate. If compression
   remains, its candidate selection must explicitly require `lifecycle_state =
   'archived'` (or equivalent `LifecycleState::Archived`), not merely remove the
   final `UPDATE`; otherwise it would still summarize active rows via a divergent
   predicate. If that cannot be done cleanly, defer to a follow-up compression
   tool.
8. **Behavior regression:** `compress_old_memories` no longer changes
   `lifecycle_state` through its age/importance/access predicate, regardless of
   entry point (`retention_policy_apply` compression or `ENGRAM_COMPRESSION_INTERVAL`
   scheduler). It may create summaries only for rows already `Archived`, and its
   result/log wording must count compression rather than lifecycle archival.
9. **Advisory-state parity:** `salience_get`/`SalienceScore.suggested_state`
   matches `decide_lifecycle_state` for the same memory/config. The legacy
   `suggest_lifecycle_state` predicate cannot survive as a public second answer.
10. **Retention auto-delete boundary:** lifecycle cap/run alone never soft-deletes.
   A separate `retention_policy_apply` with `auto_delete_after_days` may soft-delete
   already-Archived rows by `created_at`; the test documents the preexisting
   explicit-policy boundary rather than silently relying on it. Implementation
   rollout notes must warn operators that newly archived old-by-creation memories
   can be soft-deleted by the next explicit retention-policy apply when that policy
   is configured.

### Out of scope for tests (YAGNI)

- stability (not in this spec)
- lifecycle scheduler (this spec adds none; compression scheduler mechanics are covered only through its called function)
- deletion by this predicate (cap only archives; retention auto-delete is separate domain)

## What is removed (not only added)

- The `lifecycle_state` write block in `run_salience_decay` (`salience.rs:~439-460`).
- The divergent SQL predicate in `lifecycle_run`
  (`importance < X AND access_count < 5 AND created_at`), replaced by a permissive
  pre-filter + `decide_lifecycle_state`.
- The `memory_decay` lifecycle transition (`memory_policy.rs:352-357`): keeps
  writing policy scores, stops transitioning `lifecycle_state`.
- The `memory_archive_old` lifecycle transition (`summarize.rs:231-269,329`):
  stops using its own age/importance/access-count predicate to archive memories.
  Compression of already-Archived rows is allowed; deciding a memory *becomes*
  archived remains `lifecycle_run`'s job. Its candidate selection must filter to
  already-Archived rows before summarizing; simply deleting the final lifecycle
  `UPDATE` is insufficient.
- The `compress_old_memories` lifecycle transition (`retention.rs:237-312`), for
  both `retention_policy_apply` (`retention.rs:157`) and the optional server
  compression scheduler (`server.rs:122-136,726-749`): stops selecting active
  memories by age/importance/access-count and setting them to `Archived`.
  Compression may operate only on rows already `Archived` unless a follow-up
  redesigns the compression surface.
- **The legacy `suggest_lifecycle_state` predicate, made non-divergent.**
  `SalienceScore.suggested_state` (`salience.rs:80,211,220`, exposed via
  `salience_get`/`salience_score`) currently calls `suggest_lifecycle_state` —
  a *third public* predicate that diverges from the canonical one. This spec
  chooses compatibility: keep `suggested_state`, but compute it via
  `decide_lifecycle_state(memory, now, &LifecycleConfig)` so there is exactly one
  predicate behind every surface. The `suggest_lifecycle_state` fn and its tests
  (`salience.rs:254-278,971-981`) are removed or rewritten accordingly.
- Public MCP contract metadata that advertises old lifecycle side effects:
  `docs/MCP_TOOLS.md`, `src/mcp/tools/registry.rs`, and
  `src/mcp/tools/memory.rs` must be updated for `memory_decay`,
  `memory_archive_old`, `lifecycle_run.min_importance`, `lifecycle_config`
  `min_importance`/`min_access_count`, and `salience_decay_run`.
- The old stability spec is marked **superseded** with a pointer to this one.

## Out of scope (explicit, with destination)

| Out of scope | Destination |
|---|---|
| `stability` column + migration 45 | stability follow-up (on this foundation) |
| Automatic lifecycle scheduler | execution follow-up (the "B"); this spec adds none and constrains the existing compression scheduler not to write lifecycle |
| Permanent deletion by timer (in *this* predicate) | never; the cap only archives. Preexisting `retention_policy_apply` auto-delete remains creation-age based and separate; lifecycle-age semantics require follow-up migration |
| Retention-score as lifecycle input | mandatory follow-up if product wants reinforcement/durability/centrality to influence `decide_lifecycle_state` |
| Redesigning compression UX/API beyond removing lifecycle writes | follow-up if `memory_archive_old`/`compress_old_memories` need a replacement compression surface |
| Reactivation by access (`stale → active`) | separate product decision; this spec is decay-forward only |
| Graph/coactivation as a signal | phase 2 of the old spec, already deferred |

## Honest consequence

After this spec, Engram has a **correct, unified** predicate, but lifecycle-driven
`archived` remains `0` in production until someone **calls** `lifecycle_run` (or a
follow-up adds a lifecycle scheduler). The existing optional compression scheduler
is not allowed to compensate by archiving active/stale memories; after this spec it
can only compress rows whose lifecycle was already decided elsewhere. This spec
fixes *what happens when lifecycle runs*, which is exactly the (A) the owner chose;
"make it run" (B) is a separate follow-up, as the original option-2 framing
anticipated.

## References

- Inert salience predicate: obs #14998, `salience.rs:254-278`
- No lifecycle scheduler / optional compression scheduler distinction: obs #15031,
  `dream/mod.rs:15`, `quality.rs:303`, `server.rs:122-136,726-749`
- Feedback default 0.5: obs #15007, `salience.rs:293`
- Superseded design: `2026-06-26-stability-spacing-effect-design.md`
- Boundary-validation discipline carried into the renderer/handler: see the
  Codex-gate lesson on validating param type, not just value.
