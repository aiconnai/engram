# Design: Spacing-Effect Stability for Memory Retention

**Status:** Approved (design) — rewritten after full-spec cross-model review
**Date:** 2026-06-26
**Author:** Ronaldo Lima (with Claude)
**Cross-model review:** Codex/GPT (gpt-5.5) + Fugu/Sakana
- Four base decisions: both AGREE.
- Full-spec review: Codex returned NEEDS-REWORK (1 BLOCKER, 3 HIGH) — all valid, all addressed below.
- Revised scope (A3) + refinements: both independently PICK A3 with the same corrections.
**Origin:** Comparative analysis of MemPalace (github.com/MemPalace/mempalace), which
implements the spacing effect (Cepeda et al. 2006) on graph connections. This adapts the
idea to Engram's per-memory salience + lifecycle model.

---

## Problem

Engram's memory retention has two independent controls, and the decay control is uniform.

The recency component of the salience score decays with a fixed 14-day half-life for every
memory:

```rust
// src/intelligence/salience.rs:230 (and two stats-loop copies at ~417, ~710)
let decay = 0.5_f32.powf(days_since_access / 14.0);  // recency_half_life_days = 14.0
```

This contradicts the cognitive reality that **distributed (spaced) reinforcement builds more
durable memory than massed (bursty) reinforcement** (Cepeda et al. 2006). A memory accessed
50 times in 5 minutes is treated identically to one accessed 50 times over 50 days. The first
is a transient spike; the second is genuinely important. The current model cannot tell them
apart — durability is uniform, not earned.

### Critical finding: where retention is actually decided (verified against code)

`suggest_lifecycle_state` (`salience.rs:254-278`) gates archival on **two independent
variables joined by AND**:

```rust
let days_inactive = (now - last_access).num_days();   // pure wall-clock
if score < 0.2 && days_inactive >= 90 { return Archived; }   // archive_threshold_days = 90
if score < 0.4 || days_inactive >= 30 { return Stale; }      // stale_threshold_days = 30
```

The salience `score` is weighted and floored:
`score = recency*0.30 + frequency*0.20 + importance*0.30 + feedback*0.20`, then `.max(0.05)`.

**The consequence (both reviewers verified this against the code):** a default memory
(`importance=0.5`, `feedback=0.5`) already scores ~0.25 from importance+feedback *alone*
(0.15 + 0.10), before recency, and the never-accessed frequency base adds ~0.02 → ~0.27.
The `score < 0.2` archive gate is therefore **unreachable for any typical memory**. In
practice, **only the `days_inactive >= 90` gate drives archival.**

Since `stability` (as originally specced) only lengthens the *recency* component (weight
0.30) and does **not** touch `days_inactive` (wall-clock), a stability feature that changes
only the decay formula has **almost no effect on retention** — it would reorder search
results (ranking) without protecting durable memories from archival. That betrays the
feature's premise: Cepeda is about what *survives*, not display order.

> One precise caveat (Fugu + Codex): the score gate is not universally dead. A memory with
> `importance=0` AND `feedback=0` floors at 0.05 and *can* archive via score. "Only
> `days_inactive` matters" holds for **typical** memories (the ones that matter here).

## Goal

Introduce a per-memory `stability` factor, earned only through genuinely spaced reinforcement,
that does two things:

1. Lengthens the effective recency half-life: `recency = 0.5^(days_since_access / (14 * stability))`
   — improves **ranking** of durable memories (applies to all memories that gain stability).
2. Stretches the **archive** inactivity threshold: `effective_archive_days = min(180, 90 * stability)`
   — extends retention for memories that can actually archive.

`stability` defaults to `1.0` (behavior identical to today on both axes) and grows toward a
ceiling of `4.0`. With the cap, a memory that *can* archive survives up to 180 days of inactivity
before becoming archive-eligible (vs. 90 today), and ranks as if its half-life were 56 days.

**Scope honesty (established by the second review round — see §5):** effect #2 is narrow. The
archive gate is `score < 0.2 AND days_inactive >= N`, and a default memory (`importance = 0.5`)
floors at ~0.25, so it *never archives today regardless*. Since stability only grows for
`importance >= 0.3`, the archive-stretch bites only in the band `importance ∈ [0.3, 0.5)`. For
typical memories effect #2 is inert; effect #1 (ranking) is what they get. The bigger lifecycle
finding — high-importance memories never archive at all — is recorded in §5 for a separate pass.
This feature is a coherent, low-risk Phase-1 step, deliberately not framed as a retention overhaul.

## Non-goals

- **Scaling the stale threshold.** Stale stays fixed at 30 days. (Both reviewers, independently.)
  Stale is a cheap, reversible early-warning / GC / resurfacing signal; archive is the costly,
  near-terminal one. Scaling stale would suppress the very signal you want on a normal cadence
  and let stable-but-cold memories stay `Active` too long. Only **archive** stretches.
- **Stopping passive search from updating recency** (Codex HIGH). Today, a memory merely
  appearing in search results bumps `last_accessed_at`, and recency feeds ranking/lifecycle —
  a mild self-reinforcement loop. This is a **pre-existing, orthogonal** issue: it exists
  today regardless of this feature, and this feature does not make it worse (stability is
  explicitly NOT incremented by passive exposure — see §2). Fixing the recency-on-impression
  loop is a separate change to a hot path used system-wide; tracked as a follow-up, out of
  scope here.
- **Graph connection stability** (halls/tunnels, as MemPalace does it). Inert in Engram today
  (connection strength is not used in memory decay). Deferred to a future phase, best bundled
  with "coactivation as a search rank signal."
- **Retroactive stabilization of existing memories.** Existing rows start neutral (§6). A
  retroactive re-stabilization command, if ever wanted, must be a separate opt-in admin command
  with audit/provenance — never inside this migration.
- **Replacing `importance`.** Stability rewards *demonstrated spaced use*. Protecting an old
  high-value memory that has not been re-used is the job of `importance` / `boost`. Clean
  separation of concerns — reinforced by the eligibility gate in §5.

---

## Design

### 1. Scope — per individual memory (base Decision 1)

`stability` lives on the `Memory` struct. It is consumed in two places in `salience.rs`:
`calculate_recency` (decay formula) and `suggest_lifecycle_state` (archive threshold). The
salience surface already drives search reranking, lifecycle decisions, context-budget
allocation, and the priority queue.

### 2. Reinforcement trigger — "was USED", not "was fetched" (base Decision 2)

The signal that increments `stability` is **evidence of engagement, not exposure** — Fugu:
*"impressions are not endorsements."*

**Increments stability** (subject to the temporal gate, §4):
- Explicit `memory_get` of a specific memory
- `memory_boost`
- Positive `memory_feedback` ("useful")
- A search result subsequently expanded/cited downstream (the "click", Phase 2)

**Does NOT increment stability:**
- A memory merely appearing in a search result list (passive exposure)

**Rationale.** Salience drives the reranker. If passive search appearance fed stability,
high-ranked memories would rank ever higher in a positive-feedback loop, entrenching winners
regardless of value. (Independently flagged by both reviewers.)

**Known gap + mitigation (Fugu).** A memory that is genuinely useful but *only ever consumed
inside search result lists* (never a discrete `memory_get`) would not gain stability and
could eventually archive — a false negative masked by the recency bump. Mitigation: where
downstream expansion/citation can be captured, treat it as engagement (Phase 2). Until then,
**ship the explicit-only trigger and log impression-vs-engagement counts separately** so the
policy can be tuned with real data.

### 3. Stability curve — conservative, diminishing returns, capped (base Decision 3)

```
stability ∈ [1.0, 4.0],  default 1.0

per reinforcing use (when temporal gate passes AND eligibility holds):
    stability += 0.15 * (1 - stability / 4.0)
```

- **Increment `0.15`** (Codex: lowered from 0.2 — with hourly reinforcement allowed, 0.2
  climbs too fast for automation loops).
- **Diminishing returns** `(1 - stability/4.0)`: early reinforcements buy the most durability,
  later ones almost none — the shape of the Cepeda retention curve. This rules out option C
  (flat `+= 0.1`, every use worth the same — mere MemPalace mimicry) and option B (ceiling
  10.0 / 140-day half-life — a clutter generator keeping finished-project memories near-full
  salience for ~5 months).
- **Ceiling 4.0.** Bounds the recency half-life at 56 days and (with the §5 cap) archive at
  180 days. No immortal memories.

### 4. Temporal gate — 1h spacing AND a true rolling 24h cap (Decision 3 refinement, Fugu)

A reinforcement increments stability only when **all** hold:

1. `>= 1h` since the last *reinforcing* use of this memory, AND
2. `< 3` reinforcing uses in the **rolling** previous 24h, AND
3. eligibility: `importance >= 0.3` (see §5).

**Rationale (Fugu).** The 1h gate alone is insufficient: a long agent session can fire 10–16
increments in a single day, rushing a memory to ceiling in 2–3 days — *massed* practice, the
opposite of what we are paying for. The daily cap blocks burst-gaming.

**Bookkeeping — dedicated table (resolves the prior open question; Codex HIGH).** The earlier
draft considered (a) a counter + window-start column or (b) deriving from `memory_events`.
Both are rejected:
- A counter + fixed window-start is a **tumbling** window, not rolling — it permits bursts
  straddling the boundary (Codex).
- `memory_events` / the realtime `EventType` enum (`MemoryCreated/Updated/Deleted/Crossref*`)
  has **no `Accessed`/reinforcing variant**, and the sync event log is **clearable**
  (`memory_events_clear`) — deriving a durability signal from a clearable log that doesn't
  even distinguish reinforcing access from passive exposure is unsafe (verified in code).

Resolution: a dedicated append-only table, indexed for the rolling-window query:

```sql
CREATE TABLE memory_reinforcements (
    memory_id INTEGER NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
    reinforced_at TEXT NOT NULL          -- RFC3339 UTC
);
CREATE INDEX idx_reinforcements_mem_time ON memory_reinforcements(memory_id, reinforced_at);
```

The 1h-gap and rolling-24h-count checks are both exact `COUNT`/`MAX` queries over this table
for the given `memory_id`. Optionally cache `last_reinforced_at` on the memory row for the
hot 1h check; the table remains the source of truth. Old rows (> 24h) may be pruned by the
existing lifecycle/gardening pass — they no longer affect any gate.

### 5. Archive-threshold modulation — scoped retention (revised scope: A3, archive-only)

A3 modulates **only the archive inactivity threshold**, leaving stale at 30. Second-round
review (Codex + Fugu, both PICK ARCHIVE-ONLY) confirmed this with the AND/OR asymmetry made
explicit — see "Honest scope" below.

```rust
// in suggest_lifecycle_state, replacing the constant 90:
let effective_archive_days = (90.0 * memory.stability as f64).min(180.0) as i64;
let effective_stale_days   = 30;  // UNCHANGED — scaling it is defeated by the score arm (below)

if score < 0.2 && days_inactive >= effective_archive_days { return Archived; }  // AND
if score < 0.4 || days_inactive >= effective_stale_days   { return Stale; }      // OR
```

**Why A3 over A1 (both reviewers PICK A3).** A1 (modulate only the recency component of the
score) is a ranking feature with near-zero archival effect — it does not honor the durability
premise. A3 acts on `days_inactive`, the gate that actually governs archival.

**Why archive-only, not scaling stale too (both reviewers, with the AND/OR asymmetry).**
`Stale` is an **OR** (`score < 0.4 OR days_inactive >= 30`); `Archived` is an **AND**. Scaling
the day-arm of the stale-OR to `30 * stability` is **defeated by the `score < 0.4` arm**: once
recency (weight 0.30) decays after a few weeks, a default memory's composite falls to ~0.19–0.25
< 0.4, so Stale fires on the score arm regardless of any day threshold. Scaling stale "buys
almost nothing and adds two moving parts" (Fugu). So only `archive` scales.

**Net behavior:** a maxed-stability memory = "Stale from ~day 30, Archived at day 180". This is
coherent **on a condition Fugu raised and we verified**: Stale must stay *retrievable*. Confirmed
in code — search only excludes `lifecycle_state = 'archived'` (`search/hybrid.rs:170`,
`search/bm25.rs:164`); `Stale` memories remain fully searchable and re-access lifts them back to
Active. So stability here means **"resist death (archive), not resist demotion (stale)"** —
exactly what it should. If a future change ever made Stale suppress retrieval, this feature would
fail silently and must be re-evaluated.

**Honest scope (both reviewers, Q3 — this narrows the feature; do not oversell it).** Because the
archive score-gate is `score < 0.2` and archive is AND:
- A default memory (`importance = 0.5`) floors at ~0.25 and **can never satisfy `score < 0.2`** —
  so it **never archives today, with or without A3**. Day-scaling is irrelevant to it.
- Stability only grows for `importance >= 0.3`. A memory at exactly `0.3` floors at
  `0.3*0.30 + 0.5*0.20 = 0.09 + 0.10 = 0.19`, which *can* dip below 0.2.
- **Therefore A3's archive protection bites only in the narrow band `importance ∈ [0.3, 0.5)`.**
  For the typical 0.5 memory it is inert. A3 is a real but **small-blast-radius** Phase-1 step.

**The larger finding (both reviewers ranked this above the day-count):** high-importance memories
**never archive at all** under the current predicate (dead score-gate + AND). If the product wants
eventual terminal cleanup of genuinely abandoned high-importance memories, that needs a **separate
hard-idle cap or an adjusted archive predicate** — out of scope here, but recorded as the finding
that matters more than A3's threshold tweak. Tracked for a future lifecycle pass.

**Perverse effects identified and mitigated (both reviewers):**

| Risk | Mitigation in this design |
|---|---|
| **Transient reinforcement → false durability.** One spaced burst lifts stability, then the memory goes cold but resists archive longer. (Fugu: "sharpest problem".) | (1) **Cap at 180 days** (Codex) is a hard ceiling — a reinforced-once-then-abandoned memory still archives at 180 days (given `score < 0.2`). Bounded delay, not immortality; the 4.0-ceiling fear was *unbounded* growth, and the cap closes it. (2) **Stale stays at 30** — still flagged Stale on the normal cadence for GC/resurfacing even while archival is deferred. |
| **Stability ⊥ importance** — scaling `days_inactive` grants longevity to low-value memories whose score gate *is* live. | **Eligibility gate `importance >= 0.3`** (Fugu): stability cannot grow for low-importance memories, so spacing can't buy longevity for junk. |
| **Double protection** — stability in both recency-score and archive-threshold. | In practice not double: the recency path is inert for archival (score gate near-dead), so only the threshold path acts — and only in the `importance ∈ [0.3, 0.5)` band (see Honest scope). Documented, not hidden. |
| **Near-dead score gate left in place** — `score < 0.2` rarely fires; high-importance memories never archive. | Left as-is (changing weights/predicate is out of scope and risky). Recorded as the larger finding (see §5) for a future lifecycle pass — it matters more than the day-count. |

**Optional extra guard (Codex, Phase 1).** Require at least one *post-creation* spaced
reinforcement event before stability may exceed 1.0 — i.e. initial `importance >= 0.3` alone does
not lift stability; a real reinforcement must occur. Cheap, tightens transient-durability further.
Recommended but not mandatory.

**Phase-2 consideration (deferred).** A `last_reinforced` recency term feeding **stability decay**
(so transient durability erodes when a memory goes cold) is the principled fix for risk #1, but
adds a second decay process — out of A3's scope. The 180-cap + stale-at-30 + eligibility gate are
sufficient for Phase 1.

### 6. Migration — neutral backfill (base Decision 4)

Existing rows get `stability = 1.0`: decay AND archive behavior **identical to today** until a
memory earns stability from real future use. A schema migration must not silently change the
observable lifecycle of existing data; `access_count` is an untimestamped scalar that cannot
distinguish spaced from massed access, so seeding from it would bake the massed/spaced confound
into the starting state.

**Schema (migration 45).** Engram links SQLite via `rusqlite 0.31` with the `bundled`
feature (`libsqlite3-sys 0.28.0` ⇒ SQLite ~3.45.x) — the bundled version is what runs, not
the system CLI. Column-level `CHECK` added via `ALTER TABLE ADD COLUMN` is long-standing
SQLite functionality (not a recent addition), so the bundled version supports it. Confirmed
empirically on the system CLI (3.51): an out-of-range insert is rejected with "CHECK
constraint failed". The earlier "SQLite can't do this" caveat was wrong (Codex HIGH). The DB
constraint is therefore the **default**, not optional:

```sql
ALTER TABLE memories
  ADD COLUMN stability REAL NOT NULL DEFAULT 1.0 CHECK (stability >= 1.0 AND stability <= 4.0);
```

Plus the `memory_reinforcements` table from §4. Migration is additive and idempotent.

---

## Affected code

| File | Change |
|---|---|
| `src/types.rs` | Add `stability: f32` to `Memory` (serde default 1.0); clamp on deserialize/parse boundary |
| `src/storage/migrations.rs` | Migration 45: `ADD COLUMN stability ... CHECK (...)` + create `memory_reinforcements` table & index |
| `src/storage/queries/core.rs` | `memory_from_row` reads `stability`; **all** insert/update SELECT/INSERT column lists include it; clamp before write |
| `src/intelligence/salience.rs` | `calculate_recency` uses `14 * stability`; `suggest_lifecycle_state` uses `effective_archive_days`; new reinforcement fn (gate + eligibility + diminishing-returns increment + clamp); **collapse the 3 duplicated decay formulas into one shared helper** so they can't drift |
| `salience.rs` stats loops (`get_salience_stats*`, ~417, ~710) | Their inline `SELECT` must add `stability`; use the shared decay helper |
| `get_memory_salience_with_feedback` (minimal `Memory` construction) | Must populate `stability` or it silently reads 1.0 |
| Reinforcement call sites (`memory_get`, `memory_boost`, `memory_feedback` handlers) | Invoke the reinforcement fn (explicit-use path only); append to `memory_reinforcements` |
| Alternate backends (`turso_backend.rs`, `meilisearch_backend.rs`) + snapshot/import/export + test fixtures | Map/round-trip the new column |
| `memory_get_public` | Decide whether a public read counts as reinforcement (proposed: **no** — public reads are impressions) |
| `SalienceConfig` | New tunables: `stability_max` (4.0), `stability_increment` (0.15), `reinforcement_min_gap_hours` (1.0), `reinforcement_daily_cap` (3), `archive_days_cap` (180), `stability_min_importance` (0.3), `require_post_creation_reinforcement` (true — Codex optional guard: stability > 1.0 needs ≥1 reinforcement event, not just initial importance) |

## Testing

- **Regression (load-bearing):** default `stability=1.0` ⇒ recency curve AND archive behavior
  **byte-identical** to current. This is the guard that the migration changes nothing for
  existing data.
- **Lifecycle (weighted, not raw recency)** (Codex MED): assert on the composite `score` +
  `days_inactive` AND-gate, e.g. a `stability=2` memory at 120 days inactive is NOT archived
  (effective 180), a `stability=1` memory at 95 days IS archive-eligible by days but only
  Stale because score ≥ 0.2 for default importance — encode the Critical Finding as tests.
- **Archive cap:** `stability=4` ⇒ `effective_archive_days = 180` (not 360).
- **Stale unchanged:** stale at 30 days regardless of stability.
- **Eligibility:** `importance < 0.3` ⇒ stability never increments.
- **Reinforcement gate:** <1h gap ⇒ no increment; ≥1h ⇒ increment; 3rd within rolling 24h ⇒
  capped; rolling window (not tumbling) verified with timestamps straddling a boundary.
- **Curve:** diminishing returns shrink each step; clamp holds at 4.0; monotonic non-decreasing.
- **Trigger:** explicit get/boost/feedback increment AND append a reinforcement row; simulated
  passive search appearance does neither.
- **Migration:** existing rows get exactly 1.0; idempotent re-run; no NULLs; out-of-range write
  rejected by the DB CHECK and by the app clamp (defense-in-depth).
- **Concurrency** (Codex MED): two concurrent reinforcements of the same memory don't double-
  count past the cap / corrupt the rolling window.

## Phased delivery

1. **Phase 1 (this spec):** `stability` column + reinforcements table + decay-formula use +
   archive-threshold modulation (capped) + eligibility gate + neutral migration. Log
   impression-vs-engagement counts.
2. **Phase 2 (later, separate):** downstream expansion/citation as an engagement signal
   (closes the search-native-but-used gap); optional cold-decay of stability; graph-connection
   stability; revisit the near-dead score gate and the recency-on-impression loop.

## Resolved review items (audit trail)

- **[BLOCKER]** Numeric lifecycle table used raw recency; archival is a weighted-score AND
  wall-clock-days gate, and the importance floor makes the score gate near-dead for typical
  memories → **scope changed A1→A3**; misleading table removed; structural justification added.
- **[HIGH]** Passive search → recency loop → documented as pre-existing/orthogonal **non-goal**.
- **[HIGH]** Tumbling-window / clearable-`memory_events` bookkeeping → **dedicated
  `memory_reinforcements` table** with exact rolling-window queries.
- **[HIGH]** SQLite CHECK caveat too soft → **verified CHECK works on ADD COLUMN (3.51)**; DB
  constraint is now the default, app clamp is defense-in-depth.
- **[MED]** "single write path" fragile → clamp at parse/write boundaries; full backend/fixture
  list in Affected code.
- **[MED]** Affected code incomplete → stats SELECTs, `get_memory_salience_with_feedback`,
  `memory_get_public`, alternate backends added.
- **[MED]** Tests raw-recency-only → weighted lifecycle + passive non-mutation + concurrency
  tests added.

### Second review round (AND/OR asymmetry + exact floors)

After the rewrite, three code-precision facts were surfaced (user) and taken back to both
reviewers, who had validated a simpler model:

- **Feedback floor exact:** feedback defaults to 0.5 (`salience.rs:293`), so default composite
  floor is `0.15 + 0.10 = 0.25` — the `score < 0.2` archive gate is unreachable for default memories.
- **AND/OR asymmetry:** archive is AND, stale is OR → both reviewers **PICK ARCHIVE-ONLY**; scaling
  stale is defeated by its `score < 0.4` arm. (Confirms §5; recorded the reasoning.)
- **Accidental immortality:** both confirm the **180-day cap + `importance >= 0.3` gate** bound it
  sufficiently for Phase 1 (no stability decay needed yet). Codex's optional extra guard
  (require a post-creation reinforcement before stability > 1.0) added as recommended-not-mandatory.
- **Verified Fugu's condition:** Stale must stay retrievable for stability to mean "resist death";
  confirmed search only excludes `archived` (`hybrid.rs:170`, `bm25.rs:164`), Stale stays searchable.
- **Honest scope correction:** A3 bites only in `importance ∈ [0.3, 0.5)`; for typical 0.5 memories
  the archive-stretch is inert. Goal + §5 reframed; "deliver retention" overstatement removed.
- **Larger finding recorded (both ranked above the day-count):** high-importance memories never
  archive under the current predicate; terminal cleanup needs a separate hard-idle cap / adjusted
  predicate — deferred, tracked in §5.
- Fixed a stale number ("~270 days" → 180-cap) that survived from a pre-cap draft.
