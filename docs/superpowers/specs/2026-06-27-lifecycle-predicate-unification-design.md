# Lifecycle Predicate Unification — Design Spec

Date: 2026-06-27
Owner: Ronaldo (decisions) + agent (drafting)
Status: Brainstorming complete — pending cross-model review before implementation plan
Supersedes: `docs/superpowers/specs/2026-06-26-stability-spacing-effect-design.md`

## Summary

Engram has **multiple writers** of `memories.lifecycle_state`. At least **three**
are concurrent *decay-policy* engines, each with a different divergent model, and
in production none has transitioned anything (1,179 memories: `archived=0`,
`stale=0`). Several other writers are legitimate *domain* writers (consolidation,
retention policy, dream-approved actions, manual) that act on `lifecycle_state`
for non-decay reasons and must coexist. This spec unifies the **decay-policy
engines** into a single canonical predicate with one automatic decay writer,
using a hybrid temporal model (recency-primary + absolute-age terminal cap) with
importance as a continuous resistance modulator (not a binary gate). It does NOT
touch the domain writers except to draw the boundary explicitly.

It is scoped to **fix what happens when lifecycle runs**, not to make it run
(that is a deferred follow-up). The `stability`/spacing-effect feature is
**not** in this spec — it returns as a follow-up that adds a recency multiplier
on top of this corrected foundation.

## Problem (verified against code + production)

### The writer ecosystem (full enumeration, verified)

A `grep` for every `lifecycle_state` write across `src/` reveals **more than two**
writers. They split into two classes:

**Decay-policy engines — divergent predicates, MUST converge (this spec):**

| Engine | Predicate | Temporal var | Site |
|---|---|---|---|
| `lifecycle_run` | `importance < X AND age AND access_count < 5` | `created_at` | `lifecycle.rs:178,184` |
| `run_salience_decay` | `score < 0.2 AND days_idle >= 90` | `last_accessed_at` | `salience.rs:439-460` |
| `memory_decay` (policy) | `Active AND new_retention < 0.25 → Stale` | retention score | `memory_policy.rs:352-354` |

All three are **manual MCP tools** with **no scheduler** (`dream/mod.rs:15` lists
decay as *future* work). This is why production shows `archived=0/stale=0`: nobody
has invoked any of them. The original "two writers" framing missed `memory_decay`
(retention-score based) — surfaced by code review.

**Domain writers — legitimate non-decay reasons, COEXIST (out of scope):**

| Writer | Reason | Site |
|---|---|---|
| `consolidation_offline` | archives on consolidation/supersession | `consolidation_offline.rs:568` |
| `retention_policy_apply` | explicit retention policy + soft-delete | `retention.rs:182,204` |
| `dream` candidate apply | reviewed/approved action | `dream.rs:377` |
| `memory_set_lifecycle` | manual explicit | `lifecycle.rs:239` |

These are **not** concurrent decay engines. They write `lifecycle_state` for a
deliberate, non-decay reason and are explicitly preserved.

> **Note (`memory_policy.rs:146`):** the `memory_decay` writer also bypasses the
> query layer with raw SQL (a known architectural violation, obs #13825). This
> spec's decision for it (below) supersedes the raw write regardless.

#### Decision for the three decay engines

- `lifecycle_run` → becomes the **single canonical decay writer** (uses
  `decide_lifecycle_state`).
- `run_salience_decay` → **disarmed**, score-only (remove `salience.rs:439-460`).
- `memory_decay` (policy) → **disarmed** of its `lifecycle_target` transition
  (`memory_policy.rs:352-357` returns `None` for lifecycle; it keeps writing
  policy *scores* — retention/priority — but no longer transitions
  `lifecycle_state`). Rationale: its retention-score predicate is a fourth
  divergent model; consolidating it into `decide_lifecycle_state` is the same
  unification. If a retention-score signal should influence decay, that is a
  follow-up that feeds the canonical predicate, not a parallel writer.

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
| Temporal model | When does a memory decay | (3) Hybrid: recency primary (`last_accessed_at`) + absolute-age terminal cap |
| Importance role | How importance interacts | (A) Protects from normal decay, NOT from the terminal cap — no immortality |
| Old spec relation | stability spec | (1) This spec supersedes it; stability returns as a follow-up |
| Architecture | Multiple decay writers | (C) Single automatic decay writer; `run_salience_decay` + `memory_decay` become score/policy-only; domain writers untouched |
| Writer impl | How the predicate lives | (A) Canonical pure Rust fn `decide_lifecycle_state`; SQL is a permissive pre-filter only |
| Importance shape | gate vs modulator | Continuous multiplier `1.0 + imp*(MAX_MULT-1.0)`, not a binary threshold |
| Terminal cap | include now or defer | (i) Include now as a **dormant** composability guard-rail, proven by a parametrized test |
| Config | where parameters live | Dedicated `LifecycleConfig` (NOT folded into `SalienceConfig`) |
| Migration | schema change | **Zero-migration**; `SCHEMA_VERSION` stays 44; stability's migration 45 travels with the follow-up |
| Execution | scheduler | Stays **manual**; "make it run" (B) is a separate follow-up |

## Architecture

**Single automatic decay writer (C).** `run_salience_decay` stops writing
`lifecycle_state` (remove `salience.rs:~439-460` state block) and becomes
score-only. `lifecycle_run` becomes the only engine that applies decay-derived
transitions.

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
let mult            = 1.0 + memory.importance * (cfg.max_importance_mult - 1.0); // imp∈[0,1]
let effective_stale = (cfg.stale_days_base   as f32 * mult) as i64;              // default 30×mult
let effective_arch  = (cfg.archive_days_base as f32 * mult) as i64;              // default 90×mult

if idle_days >= effective_arch  { return Archived; }
if idle_days >= effective_stale { return Stale;    }

Active
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

**Behavior migration (not schema):** `salience_decay_run` stops transitioning
state. This is documented in contract docs/tests; it is a behavior change, not a
schema migration.

## Invariants (canonical — go to the gate)

1. **Single automatic decay/lifecycle-policy writer.** Only `lifecycle_run`
   applies automatic transitions derived from `decide_lifecycle_state`.
   `run_salience_decay` is score-only. Explicit / manual-domain writes
   (`memory_set_lifecycle`, consolidation, confirmed expiration —
   `dream/candidates.rs` expire path) are **not** concurrent decay engines and
   coexist legitimately.
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
5. **Cap archives, never deletes.** *This spec's* cap never deletes — it only
   transitions to `Archived`. The preexisting `retention_policy_apply`
   auto-delete (`retention.rs:204`, soft-delete of archived rows via
   `auto_delete_after_days`) is a separate, explicitly-configured retention
   domain — out of this predicate, neither added nor removed here.

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

### Out of scope for tests (YAGNI)

- stability (not in this spec)
- automatic scheduler (lifecycle stays manual)
- deletion by this predicate (cap only archives; retention auto-delete is separate domain)

## What is removed (not only added)

- The `lifecycle_state` write block in `run_salience_decay` (`salience.rs:~439-460`).
- The divergent SQL predicate in `lifecycle_run`
  (`importance < X AND access_count < 5 AND created_at`), replaced by a permissive
  pre-filter + `decide_lifecycle_state`.
- The `memory_decay` lifecycle transition (`memory_policy.rs:352-357`): keeps
  writing policy scores, stops transitioning `lifecycle_state`.
- **The legacy `suggest_lifecycle_state` predicate, made non-divergent.**
  `SalienceScore.suggested_state` (`salience.rs:80,211,220`, exposed via
  `salience_get`/`salience_score`) currently calls `suggest_lifecycle_state` —
  a *third public* predicate that diverges from the canonical one. This spec
  requires one of: **(a)** remove `suggested_state` from `SalienceScore`
  entirely, or **(b)** compute it via `decide_lifecycle_state(memory, now,
  &LifecycleConfig)` so there is exactly one predicate behind every surface.
  The implementation plan picks one (recommendation: **(b)** — keep the
  advisory field useful, but make it tell the truth). The `suggest_lifecycle_state`
  fn and its tests (`salience.rs:254-278,971-981`) are removed or rewritten
  accordingly.
- The old stability spec is marked **superseded** with a pointer to this one.

## Out of scope (explicit, with destination)

| Out of scope | Destination |
|---|---|
| `stability` column + migration 45 | stability follow-up (on this foundation) |
| Automatic lifecycle scheduler | execution follow-up (the "B") |
| Permanent deletion by timer (in *this* predicate) | never; the cap only archives. Preexisting `retention_policy_apply` auto-delete is a separate domain, untouched here |
| Reactivation by access (`stale → active`) | separate product decision; this spec is decay-forward only |
| Graph/coactivation as a signal | phase 2 of the old spec, already deferred |

## Honest consequence

After this spec, Engram has a **correct, unified** predicate, but `archived` stays
`0` in production until someone **calls** `lifecycle_run` (or a follow-up adds a
scheduler). This spec fixes *what happens when it runs*, which is exactly the (A)
the owner chose; "make it run" (B) is a separate follow-up, as the original
option-2 framing anticipated.

## References

- Inert salience predicate: obs #14998, `salience.rs:254-278`
- Manual-only execution: obs #15031, `dream/mod.rs:15`, `quality.rs:303`
- Feedback default 0.5: obs #15007, `salience.rs:293`
- Superseded design: `2026-06-26-stability-spacing-effect-design.md`
- Boundary-validation discipline carried into the renderer/handler: see the
  Codex-gate lesson on validating param type, not just value.
