# Cross-Model Review Prompt — Lifecycle Predicate Unification Spec

> Paste this into Codex (GPT/o-series) and Fugu (Sakana) independently.
> Reviewer must read the spec file AND verify claims against the actual code.
> This is a SPEC design review, not a code-diff review — there is no
> implementation yet. The spec's central discovery changed after the first
> internal review (from "two writers" to "three decay engines + domain writers"),
> so the enumeration is the highest-priority thing to validate.

## Spec under review

`docs/superpowers/specs/2026-06-27-lifecycle-predicate-unification-design.md`
(334 lines). Read it fully before judging.

## Context (why this spec exists)

Engram's memory lifecycle (`active → stale → archived`) has never transitioned
anything in production (1,179 memories: `archived=0`, `stale=0`). Investigation
found multiple concurrent writers of `lifecycle_state` with divergent predicates,
none scheduled. This spec unifies the *decay-policy* writers into one canonical
predicate (`decide_lifecycle_state`), hybrid temporal model (recency primary +
absolute-idle terminal cap), importance as continuous modulator. Execution stays
manual (no scheduler — that's a deferred follow-up). The old stability spec
(`2026-06-26-stability-spacing-effect-design.md`) is superseded.

## What to validate (verify each against the code, do not take the spec's word)

### 1. The writer enumeration (HIGHEST PRIORITY)

The spec claims exactly **three decay-policy engines** that must converge:
- `lifecycle_run` (`src/mcp/handlers/lifecycle.rs:178,184`) — day-count predicate
- `run_salience_decay` (`src/intelligence/salience.rs:439-460`) — score predicate
- `memory_decay`/policy (`src/mcp/handlers/memory_policy.rs:352-354`) — retention-score predicate

and **four domain writers** that legitimately coexist (consolidation, retention
policy, dream-approved, manual). **Grep the codebase yourself** for every write to
`lifecycle_state`. Is the enumeration complete? Is any "domain writer" actually a
disguised decay engine (or vice-versa)? Is there a writer the spec missed?

Do not answer this from memory or from the spec alone. At minimum, run an
equivalent of:

```bash
rg -n "SET lifecycle_state|UPDATE memories SET lifecycle_state|update_memory_lifecycle_state\\(|auto_delete_after_days|valid_to = .*archived" src
```

Then include in your review a compact inventory of every write site you found:
`file:line → decay-engine/domain/helper/irrelevant → agree/disagree with spec`.
If you cannot inspect the repository/code, return `REVIEW_VERDICT: FAIL`.

### 2. The `memory_decay` decision (MOST CONTESTED)

The spec **disarms** `memory_decay`'s lifecycle transition
(`memory_policy.rs:352-357` returns `None`), keeping its score/policy writes but
removing the `Active → Stale` transition. Owner's stated preference: disarm now;
if retention-score should influence decay, make it a follow-up that feeds the
canonical predicate as an *input*, not a parallel writer. **Is disarming correct,
or should `memory_decay`'s retention signal feed `decide_lifecycle_state` in this
spec rather than being silenced?** Argue the strongest case against the owner's
choice.

### 3. The `suggested_state` requirement

`SalienceScore.suggested_state` (`src/intelligence/salience.rs:80,211,220`,
exposed via `salience_get`) currently calls the legacy `suggest_lifecycle_state`.
The spec requires removing it OR recomputing it via `decide_lifecycle_state`
(recommends the latter). **Is leaving it as-is a real divergence risk? Is the
recommendation (delegate to canonical predicate) right, or should it be removed
entirely?**

### 4. The boundary with `retention_policy_apply` auto-delete

The spec scopes "cap archives, never deletes" to *this predicate only*, and
acknowledges `retention_policy_apply` (`src/storage/queries/retention.rs:182,204`)
already soft-deletes archived rows via `auto_delete_after_days` as a separate
domain. **Is this boundary clean, or does the cap (which produces more `Archived`
rows) interact with retention auto-delete in a way the spec doesn't address?**
E.g. does archiving via the cap now feed rows into an auto-delete path that the
owner may not intend?

### Also check (lower priority)

- The hybrid predicate arithmetic (spec lines 139-189): does importance-as-multiplier
  + terminal cap actually close the "high-importance abandoned" hole? Is the cap
  genuinely dormant under defaults (archive_base 90 × max_mult 4.0 = 360 < cap 365)?
- The monotonicity + idempotence invariant (lines 218-240): is "direct
  active→archived allowed, no two-step required" sound without a `lifecycle_changed_at`
  column?
- Zero-migration claim: does the predicate truly need no new column?

## Output format (REQUIRED — parsed by review-gate.sh)

First line of your verdict MUST be exactly one of:

```
REVIEW_VERDICT: PASS <one-line summary>
REVIEW_VERDICT: FAIL <one-line summary>
```

Then a bullet list of findings, each prefixed `[BLOCKER]` / `[HIGH]` / `[MED]` /
`[LOW]`. For each finding cite the spec line and/or the code file:line you
verified against. Do not reward the spec; report only real issues introduced or
unaddressed. PASS means "this spec is safe to turn into an implementation plan".
