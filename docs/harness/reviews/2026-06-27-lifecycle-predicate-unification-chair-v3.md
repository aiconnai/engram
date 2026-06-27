REVIEW_VERDICT: PENDING Codex PASS only; Grok artifact invalid because CLI was not authenticated

# Chair note — lifecycle predicate unification re-review v3 provenance correction

Date: 2026-06-27
Artifacts:
- Codex: `docs/harness/reviews/2026-06-27-lifecycle-predicate-unification-codex-v3.md`
- Invalid/non-review artifact: `docs/harness/reviews/2026-06-27-lifecycle-predicate-unification-grok-v3.md`
- Prompt: `docs/harness/reviews/2026-06-27-lifecycle-predicate-unification-prompt.md`
- Spec: `docs/superpowers/specs/2026-06-27-lifecycle-predicate-unification-design.md`

## Provenance correction

The earlier version of this Chair artifact incorrectly counted the Grok output as a
second reviewer. That was wrong: the local `grok` CLI existed on PATH, but it was
not authenticated/covered by an active subscription in this environment. Therefore
`docs/harness/reviews/2026-06-27-lifecycle-predicate-unification-grok-v3.md` is an
invalid review artifact and must not be used as evidence that an independent
reviewer confirmed the design.

## Valid evidence retained

- Codex v3 returned **PASS**: the scheduler/compression blocker is fixed and the
  spec is safe for implementation planning with one metadata cleanup gap.
- The Codex MED finding about `lifecycle_config` public metadata was incorporated
  into the spec draft after the Codex review.

## Current gate state

- **Not a completed multi-review council.** Only Codex v3 is valid evidence.
- The spec is improved and has a valid Codex PASS, but any process requirement for
  two independent reviewers remains open until a real second reviewer (for example
  Fugu/Sakana, Gemini, or another authenticated reviewer) runs the prompt.
- Do not summarize this state as "PASS in three axes" or "Codex + Grok confirmed".

## Load-bearing Codex-confirmed points

1. Writer inventory now includes all converging decay/compression lifecycle paths:
   `lifecycle_run`, `run_salience_decay`, `memory_decay`, `memory_archive_old`, and
   `compress_old_memories` via both `retention_policy_apply` and the optional
   `ENGRAM_COMPRESSION_INTERVAL` scheduler.
2. `compress_old_memories` is constrained to compression-only behavior: it may
   operate on already-Archived rows or be split/redesigned later, but it must not
   transition Active/Stale rows to Archived.
3. The single-writer invariant remains coherent: only `lifecycle_run` applies
   decay-derived lifecycle transitions through `decide_lifecycle_state`.
4. Retention auto-delete remains an accepted explicit-policy boundary with
   creation-age semantics and no zero-migration break.

## Minor findings incorporated after Codex PASS

- Added `lifecycle_config` public metadata cleanup for `min_importance` and
  `min_access_count` alongside `lifecycle_run.min_importance`.
- Made the `memory_archive_old` implementation requirement explicit: candidate
  selection must require already-Archived rows before summarizing; removing only
  the final lifecycle UPDATE is insufficient.
- Added rollout-note requirement for the retention auto-delete interaction.

Next step: run a real second reviewer if the process still requires cross-model
council parity; otherwise treat the current evidence as Codex-only PASS plus owner
approval.
