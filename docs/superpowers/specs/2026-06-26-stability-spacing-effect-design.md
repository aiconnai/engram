# Design: Spacing-Effect Stability for Memory Decay

**Status:** Approved (design)
**Date:** 2026-06-26
**Author:** Ronaldo Lima (with Claude)
**Cross-model review:** Codex/GPT (gpt-5.5) + Fugu/Sakana — both AGREE on all four decisions
**Origin:** Comparative analysis of MemPalace (github.com/MemPalace/mempalace), which
implements the spacing effect (Cepeda et al. 2006) on graph connections. This adapts
the idea to Engram's per-memory salience model.

---

## Problem

Engram's recency decay is uniform: every memory decays with a fixed 14-day half-life.

```rust
// src/intelligence/salience.rs (3 call sites: ~230, ~417, ~710)
recency = 0.5_f32.powf(days_since_access / 14.0)
```

This contradicts the cognitive reality that **distributed (spaced) reinforcement builds
more durable memory than massed (bursty) reinforcement** (Cepeda et al. 2006, "Distributed
practice in verbal recall tasks"). A memory accessed 50 times in 5 minutes is treated
identically to one accessed 50 times over 50 days. The first is a transient spike; the
second is genuinely important. The current model cannot tell them apart, so durability
is not earned — it is uniform.

## Goal

Introduce a per-memory `stability` factor that lengthens the effective half-life **only**
when reinforcement is genuinely spaced over time, with diminishing returns and a hard
ceiling (no immortal memories).

New decay formula:

```
recency = 0.5 ^ (days_since_access / (14 * stability))
```

`stability` defaults to `1.0` (identical to today's behavior) and grows toward a ceiling
of `4.0` (max effective half-life = 56 days) as the memory is used in a spaced manner.

## Non-goals

- **Graph connection stability** (halls/tunnels, as MemPalace does it). Engram does not
  currently use connection strength in memory decay, so it would be inert. Deferred to a
  future phase, best bundled with "coactivation as a search rank signal" (separate roadmap item).
- **Retroactive stabilization of existing memories.** See Migration below — existing rows
  start neutral. A retroactive re-stabilization command is explicitly out of scope here and,
  if ever built, must be a separate opt-in admin command with audit/provenance, never inside
  this migration.
- **Replacing `importance`.** Stability rewards *demonstrated spaced use*. Protecting an
  old high-value memory that hasn't been re-used is the job of the existing `importance`
  / `boost` fields, not stability. Clean separation of concerns.

---

## Design

### 1. Scope — per individual memory (Decision 1, both reviewers AGREE)

`stability` lives on the `Memory` struct and is applied in `salience.rs::calculate_recency`
and the two stats-loop call sites. This is where decay already runs and where the effect is
immediately observable: search reranking, stale/archive lifecycle decisions, context-budget
allocation, and priority-queue ordering all consume the salience score.

### 2. Reinforcement trigger — "was USED", not "was fetched" (Decision 2, both reviewers AGREE)

The signal that increments `stability` is **evidence of engagement, not exposure**. Fugu's
framing: *"impressions are not endorsements."*

**Increments stability** (subject to the temporal gate below):
- Explicit `memory_get` of a specific memory
- `memory_boost`
- Positive `memory_feedback` ("useful")
- A search result that is subsequently expanded/cited downstream (the "click", not the "impression")

**Updates recency only (NOT stability):**
- A memory merely appearing in a search result list (passive exposure)

**Rationale.** Salience drives the reranker. If passive search appearance fed back into
stability, high-ranked memories would rank ever higher in a positive-feedback loop —
entrenching winners regardless of real value (flagged independently by both reviewers).

**Known gap + mitigation (Fugu).** A memory that is genuinely useful but *only ever consumed
inside search result lists* (never a discrete `memory_get`) would never gain stability and
could decay out — a false negative masked by the recency bump. Mitigation: where downstream
expansion/citation can be captured, treat it as engagement. Where it cannot yet be captured,
**ship the explicit-only trigger now but log impression-vs-engagement separately** so the
threshold can be tuned with real data. (See "Phased delivery".)

### 3. Stability curve — conservative, diminishing returns, capped (Decision 3, both reviewers AGREE → A)

```
stability ∈ [1.0, 4.0],  default 1.0

per reinforcing use (when temporal gate passes):
    stability += 0.15 * (1 - stability / 4.0)

decay:
    recency = 0.5 ^ (days_since_access / (14 * stability))
```

- **Increment `0.15`** (Codex: lowered from an initial 0.2 — with hourly reinforcement
  allowed, 0.2 climbs too fast for automation loops).
- **Diminishing returns** `(1 - stability/4.0)`: early reinforcements buy the most durability,
  later ones almost none — the shape of the Cepeda retention curve. This is the property that
  rules out option C (flat linear `+= 0.1`, which makes every use worth the same and is just
  MemPalace mimicry) and option B (ceiling 10.0 / 140-day half-life — a "clutter generator"
  that keeps a finished project's memories near-full-salience for ~5 months).

**Numerical validation against existing lifecycle thresholds (Fugu).** The ceiling of 4.0
coheres cleanly with the existing `min_salience = 0.05` floor and the 30-day stale / 90-day
archive thresholds:

| Memory | recency @ 90 days | Outcome |
|---|---|---|
| Max-reinforced (stability=4) | `0.5^(90/56) ≈ 0.33` | Survives the 90-day archive window — correct: genuinely core memories should not auto-archive |
| Default (stability=1) | `0.5^(90/14) ≈ 0.012` | Below the 0.05 floor — archived as intended |

The two regimes separate cleanly, and even a maxed memory keeps decaying (half-life 56 < 90)
rather than going immortal. **The ceiling stays at 4.0 precisely for this reason.**

### 4. Temporal gate — 1h spacing AND a daily cap (Decision 3 refinement, Fugu)

A reinforcement increments stability only when **both** hold:

1. `>= 1h` since the last *reinforcing* use of this memory, AND
2. `<= 3` reinforcements counted in the rolling previous 24h.

**Rationale (Fugu).** The 1h gate alone is insufficient: a long agent session can fire
10–16 increments in a single day, rushing a memory to ceiling in 2–3 days — which is
*massed* practice, the exact opposite of the spacing effect we are paying for. The daily
cap blocks burst-gaming at trivial cost and preserves Cepeda's "spaced across distinct
time" intent.

To enforce the daily cap we need to know how many reinforcements happened in the last 24h.
Two implementation options (resolved in the plan, not here):
- (a) A `last_reinforced_at` timestamp + a small rolling counter (`reinforcement_count_24h`
  + window start) on the memory row, or
- (b) Derive the count from the existing `memory_events` audit table (EventType::Accessed),
  filtered to reinforcing event kinds in the last 24h.

Option (b) avoids new columns if `memory_events` already records enough to distinguish
reinforcing access from passive exposure; otherwise (a) is the fallback. This is the main
open implementation question for the plan.

### 5. Migration — neutral backfill (Decision 4, both reviewers AGREE → A)

Existing rows get `stability = 1.0`. Their decay behavior is **identical to today** until
they earn stability from real future use. A schema migration must not silently change the
observable decay behavior of existing data; `access_count` is an untimestamped scalar that
structurally cannot distinguish spaced from massed access, so seeding stability from it
(options B/C) would bake the very massed/spaced confound this feature exists to avoid into
the starting state.

**Schema (migration 45).** Both reviewers independently proposed the identical constraint:

```sql
stability REAL NOT NULL DEFAULT 1.0 CHECK (stability >= 1.0 AND stability <= 4.0)
```

**SQLite caveat (implementation note).** Engram's migrations use
`ALTER TABLE memories ADD COLUMN ...` (see `migrations.rs` precedent: `quality_score`,
`embedding_model`, `media_url`, etc.). SQLite's `ALTER TABLE ADD COLUMN` accepts a
`DEFAULT` and `NOT NULL`, but **does not enforce a `CHECK` added this way in older
versions / cannot always add table-level CHECKs via ADD COLUMN**. Resolution for the plan:
- Apply `... ADD COLUMN stability REAL NOT NULL DEFAULT 1.0` in migration 45 (additive,
  idempotent DDL), and
- Enforce the `[1.0, 4.0]` bound at the **application layer** in the single update path
  (clamp before write), since all stability writes funnel through one reinforcement function.
- Optionally add a `CHECK` via a column-level constraint if the project's minimum SQLite
  version supports it cleanly; do not block the migration on it.

The clamp-at-write-site is the load-bearing invariant; the DB CHECK is defense-in-depth.

---

## Affected code

| File | Change |
|---|---|
| `src/types.rs` | Add `stability: f32` to `Memory` (default 1.0 via serde) |
| `src/storage/migrations.rs` | Migration 45: `ADD COLUMN stability REAL NOT NULL DEFAULT 1.0` |
| `src/storage/queries/core.rs` | `memory_from_row` reads `stability`; insert/update paths write it |
| `src/intelligence/salience.rs` | `calculate_recency` + 2 stats loops use `14 * stability`; new reinforcement fn with the temporal gate + diminishing-returns increment + clamp |
| Reinforcement call sites (`memory_get`, `memory_boost`, `memory_feedback` handlers) | Invoke the reinforcement fn (explicit-use path only) |
| `SalienceConfig` | New tunables: `stability_max` (4.0), `stability_increment` (0.15), `reinforcement_min_gap_hours` (1.0), `reinforcement_daily_cap` (3) |

The three duplicated decay formulas should be collapsed into one shared helper as part of
this work (they must stay in sync once `stability` is a factor).

---

## Testing

- **Unit (salience):** default stability ⇒ identical curve to current (regression guard);
  stability=4 ⇒ 56-day half-life; the 90-day threshold-coherence table above as explicit assertions.
- **Unit (reinforcement gate):** <1h gap ⇒ no increment; >1h gap ⇒ increment; 4th use in
  24h ⇒ capped; diminishing returns shrink each step; clamp holds at 4.0.
- **Unit (trigger):** explicit get/boost/feedback increment; simulated passive search
  appearance does not.
- **Migration:** existing rows get exactly 1.0; idempotent re-run; no NULLs; out-of-range
  write is clamped (and rejected by CHECK if present).
- **Property:** stability is monotonically non-decreasing under reinforcement and never
  exceeds 4.0 nor drops below 1.0.

## Phased delivery

1. **Phase 1 (this spec):** schema + decay formula + explicit-use reinforcement with the
   1h/daily-cap gate + neutral migration. Log impression-vs-engagement counts for tuning.
2. **Phase 2 (later, separate):** downstream expansion/citation as an engagement signal
   (closes Fugu's search-native-but-used gap); graph-connection stability.

## Open questions for the implementation plan

1. Daily-cap bookkeeping: new columns (`last_reinforced_at` + rolling counter) vs. deriving
   from `memory_events`. Prefer deriving if the event log already distinguishes reinforcing
   access from passive exposure.
2. Exact set of handlers that count as "explicit use" vs. "passive" — enumerate against the
   current call sites that bump `access_count` / `last_accessed_at`.
3. Whether to expose stability in `memory_get` / stats output for observability.
