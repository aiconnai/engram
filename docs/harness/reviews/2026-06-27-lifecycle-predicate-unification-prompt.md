# Cross-Model Re-Review Prompt — Lifecycle Predicate Unification Spec

> Paste this into Codex (GPT/o-series) and Fugu/Sakana independently.
> Reviewer must read the spec file AND verify claims against the actual code.
> This is a SPEC design re-review, not a code-diff review — there is no
> implementation yet. The prior councils returned FAIL because the writer
> enumeration repeatedly missed lifecycle-state writers/entry points. Validate
> that the latest draft actually fixes the re-review v2 blocker.

## Spec under review

`docs/superpowers/specs/2026-06-27-lifecycle-predicate-unification-design.md`.
Read it fully before judging.

## Context (why this re-review exists)

Earlier review found that the spec missed `memory_archive_old`, conflict-resolution
writers, retention compression, retention auto-delete semantics, importance
normalization, and the `memory_decay` lifecycle transition. Those were fixed in the
previous draft. Re-review v2 then found a remaining BLOCKER: the spec listed
`compress_old_memories` / `retention.rs:312`, but classified it only as explicit
retention compression and missed the **optional automatic server scheduler**:

- `src/bin/server.rs:122-136` configures `ENGRAM_COMPRESSION_INTERVAL` and related
  age/importance/access parameters;
- `src/bin/server.rs:726-749` starts the scheduler when the interval is enabled;
- the scheduler calls `engram::storage::queries::compress_old_memories`;
- `src/storage/queries/retention.rs:237` selects active old/low-importance/low-access rows;
- `src/storage/queries/retention.rs:312` sets `lifecycle_state='archived'`.

The latest draft now claims:

- there are **four MCP-facing decay/compression tools plus one optional server
  compression scheduler / retention compression path** that must converge;
- `compress_old_memories` is in scope and must be disarmed of lifecycle
  transitions for both callers: `retention_policy_apply(compress_after_days)` and
  `ENGRAM_COMPRESSION_INTERVAL` scheduler;
- after the spec, `compress_old_memories` may only summarize/compress rows already
  `Archived`, or a follow-up must redesign/split the compression surface;
- there is still **no new lifecycle scheduler** in this spec;
- public MCP contracts/metadata must be updated for changed tool behavior:
  `memory_decay`, `memory_archive_old`, `lifecycle_run.min_importance`, and
  `salience_decay_run`.

## What to validate (verify each against code; do not take the spec's word)

### 1. Writer enumeration and classification (HIGHEST PRIORITY)

Grep the codebase yourself for every lifecycle-state write and related visibility
write. At minimum, run an equivalent of:

```bash
rg -n "SET lifecycle_state|UPDATE memories SET lifecycle_state|update_memory_lifecycle_state\(|compress_old_memories\(|auto_delete_after_days|valid_to = .*archived" src
```

Then include in your review a compact inventory of every write site / entry point
you found: `file:line → decay-engine/domain/helper/irrelevant → agree/disagree
with spec`.

The spec should now classify:

- converging decay/compression lifecycle paths:
  - `lifecycle_run`
  - `run_salience_decay`
  - `memory_decay`
  - `memory_archive_old`
  - `compress_old_memories` via both `retention_policy_apply` and optional server
    compression scheduler (`ENGRAM_COMPRESSION_INTERVAL`)
- preserved domain writers:
  - consolidation
  - retention max-count
  - retention auto-delete (`valid_to`, creation-age semantics)
  - context-quality conflict resolution
  - dream-approved actions
  - manual lifecycle
- helpers/initializers/tests/bench fixtures as non-engines.

If this inventory is still incomplete, return FAIL.

### 2. `compress_old_memories` and compression scheduler decision

The latest draft chooses the conservative fix: `compress_old_memories` must stop
changing lifecycle state for **all callers**. It may only create summaries for rows
already `Archived`, or be replaced/split in a follow-up. This removes the optional
compression scheduler as a hidden lifecycle writer while preserving zero-migration
and the single-writer invariant.

Validate the strongest counter-case: should explicit retention compression or the
optional scheduler be allowed to call `decide_lifecycle_state` and still archive?
Would that preserve or violate the spec's single-writer invariant? Is the chosen
"already Archived only" rule precise enough to plan implementation?

### 3. Public MCP contract cleanup

Validate the spec now explicitly requires implementation-plan updates for all
public surfaces whose advertised behavior changes:

- `docs/MCP_TOOLS.md`
- `src/mcp/tools/registry.rs`
- `src/mcp/tools/memory.rs`

Focus on:

- `memory_decay` no longer performs active lifecycle transitions;
- `memory_archive_old` no longer moves originals to archived state;
- `salience_decay_run` no longer updates lifecycle states;
- `lifecycle_run.min_importance` is no longer a candidate-selection filter and
  must not exclude memories from the canonical predicate (remove from metadata or
  accept as deprecated/no-op only).

If the public contract cleanup remains under-specified, return FAIL only if it is
unsafe to turn into an implementation plan; otherwise mark HIGH/MED.

### 4. `memory_archive_old` convergence decision

The spec treats `memory_archive_old` as a decay/compression engine and disarms
lifecycle transitions from its age/importance/access-count predicate. It may only
compress rows already `Archived`, or a follow-up must introduce a replacement
compression surface.

Is this the right convergence, or should it keep writing lifecycle if it delegates
to `decide_lifecycle_state`? Explain whether that would create a second lifecycle
writer.

### 5. `memory_decay` decision

The spec disarms `memory_decay`'s lifecycle transition while preserving policy-score
updates, and makes retention-score-as-lifecycle-input a mandatory follow-up if the
product wants reinforcement/durability/centrality to affect lifecycle.

Is this sufficient, or should retention score feed `decide_lifecycle_state` in this
spec rather than being deferred?

### 6. Retention auto-delete boundary

The spec explicitly accepts the preexisting `retention_policy_apply` behavior:
`auto_delete_after_days` soft-deletes archived rows by `created_at`, not time since
archived. This preserves zero-migration; changing semantics requires a follow-up
with `lifecycle_changed_at` or equivalent.

Is documenting/accepting this boundary safe, or is it a blocker because canonical
lifecycle will produce more `Archived` rows that can be immediately soft-deleted
under existing retention policies?

### 7. Public/advisory state and importance normalization

Validate both:

- `SalienceScore.suggested_state` must not keep using legacy `suggest_lifecycle_state`;
  the spec chooses compatibility by keeping the field but delegating to the canonical
  predicate.
- `importance` must be normalized inside the predicate before the multiplier because
  storage paths can carry raw `f32` values.

Are these requirements sufficient and testable?

### Also check (lower priority)

- The hybrid predicate arithmetic: does importance-as-multiplier + terminal cap
  close the high-importance-abandoned hole? Is the cap dormant under defaults
  (`archive_base 90 × max_mult 4.0 = 360 < cap 365`)?
- Monotonicity/idempotence: is direct `active→archived` sound without
  `lifecycle_changed_at`?
- Zero-migration claim: after accepting creation-age retention auto-delete and
  normalizing importance in code, does the spec truly need no new column?

## Output format (REQUIRED)

First line of your verdict MUST be exactly one of:

```text
REVIEW_VERDICT: PASS <one-line summary>
REVIEW_VERDICT: FAIL <one-line summary>
```

Then a bullet list of findings, each prefixed `[BLOCKER]` / `[HIGH]` / `[MED]` /
`[LOW]`. For each finding cite the spec line and/or the code file:line you verified
against. Do not reward the spec; report only real issues introduced or unaddressed.
PASS means "this spec is safe to turn into an implementation plan".
