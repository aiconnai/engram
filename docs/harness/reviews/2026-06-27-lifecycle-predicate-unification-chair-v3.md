REVIEW_VERDICT: PASS Codex and Claude Sonnet authenticated reviews passed; Grok artifact remains invalid

# Chair synthesis — lifecycle predicate unification re-review v3 provenance-corrected

Date: 2026-06-27
Artifacts:
- Codex: `docs/harness/reviews/2026-06-27-lifecycle-predicate-unification-codex-v3.md`
- Claude Sonnet: `docs/harness/reviews/2026-06-27-lifecycle-predicate-unification-claude-sonnet-v3.md`
- Invalid/non-review artifact: `docs/harness/reviews/2026-06-27-lifecycle-predicate-unification-grok-v3.md`
- Prompt: `docs/harness/reviews/2026-06-27-lifecycle-predicate-unification-prompt.md`
- Spec: `docs/superpowers/specs/2026-06-27-lifecycle-predicate-unification-design.md`

## Provenance correction

The earlier Chair artifact incorrectly counted the Grok output as a second
reviewer. That was wrong: the local `grok` CLI existed on PATH, but it was not
authenticated/covered by an active subscription. The Grok artifact remains invalid
and must not be used as evidence.

A valid second reviewer was then run through authenticated Claude Code Sonnet
(`claude --safe-mode --model sonnet --print`). Sonnet returned PASS. Its artifact
has a minor output-format defect (one introductory sentence before
`REVIEW_VERDICT`), but the verdict and findings are present and the review
independently grepped the codebase.

## Council result

- Codex v3: **PASS** — scheduler/compression blocker fixed; safe for implementation
  plan with one metadata cleanup gap.
- Claude Sonnet v3: **PASS** — writer inventory complete; v2 blockers resolved;
  findings are MED/LOW only.
- Chair decision: **PASS**. The re-review v2 blocker is fixed: the optional server
  compression scheduler path (`server.rs` -> `compress_old_memories` ->
  `retention.rs:312`) is explicitly modeled and disarmed as a lifecycle writer.

## Load-bearing confirmations

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

## Findings incorporated after PASS

- Codex MED: added `lifecycle_config` public metadata cleanup for `min_importance`
  and `min_access_count` alongside `lifecycle_run.min_importance`.
- Grok-invalid/Sonnet-confirmed implementation caution: `memory_archive_old`
  candidate selection must require already-Archived rows before summarizing;
  removing only the final lifecycle UPDATE is insufficient.
- Sonnet MED: implementation must explicitly replace `lifecycle_run`'s current
  `created_at`-based stale/archive SQL with the canonical `last_accessed_at
  .unwrap_or(created_at)` idle calculation; this is now called out in the spec and
  plan.
- Added rollout-note requirement for the retention auto-delete interaction.

## Residual implementation-plan cautions

- Public docs/registry metadata cleanup is high priority during implementation.
- `memory_archive_old` and `compress_old_memories` will shrink in behavior until a
  follow-up compression surface exists; this is accepted by the spec to preserve
  single-writer lifecycle semantics.
- Do not summarize the review set as Codex + Grok. Correct provenance is Codex +
  Claude Sonnet; Grok is invalid.

Next step: implementation can proceed from
`docs/superpowers/plans/2026-06-27-lifecycle-predicate-unification.md` if the owner
accepts Codex + Claude Sonnet as the required reviewer set.
