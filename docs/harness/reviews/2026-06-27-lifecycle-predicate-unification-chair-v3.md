REVIEW_VERDICT: PASS corrected lifecycle predicate spec is safe to turn into an implementation plan

# Chair synthesis — lifecycle predicate unification re-review v3

Date: 2026-06-27
Artifacts:
- Codex: `docs/harness/reviews/2026-06-27-lifecycle-predicate-unification-codex-v3.md`
- Grok substitute: `docs/harness/reviews/2026-06-27-lifecycle-predicate-unification-grok-v3.md`
- Prompt: `docs/harness/reviews/2026-06-27-lifecycle-predicate-unification-prompt.md`
- Spec: `docs/superpowers/specs/2026-06-27-lifecycle-predicate-unification-design.md`

## Council result

- Codex: **PASS** — scheduler/compression blocker fixed; safe for implementation plan with one metadata cleanup gap.
- Grok substitute: **PASS** — writer inventory complete; convergence/disarm decisions safe to implement.
- Chair decision: **PASS**. The re-review v2 blocker is fixed: the optional server compression scheduler path (`server.rs` -> `compress_old_memories` -> `retention.rs:312`) is now explicitly modeled and disarmed as a lifecycle writer.

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

## Minor findings incorporated after PASS

- Added `lifecycle_config` public metadata cleanup for `min_importance` and
  `min_access_count` alongside `lifecycle_run.min_importance`.
- Made the `memory_archive_old` implementation requirement explicit: candidate
  selection must require already-Archived rows before summarizing; removing only
  the final lifecycle UPDATE is insufficient.
- Added rollout-note requirement for the retention auto-delete interaction.

## Residual implementation-plan cautions

- Public docs/registry metadata cleanup is high priority during implementation.
- `memory_archive_old` and `compress_old_memories` will shrink in behavior until a
  follow-up compression surface exists; this is accepted by the spec to preserve
  single-writer lifecycle semantics.
- Fugu/Sakana CLI was unavailable in this environment; Grok was used as the second
  local model-family reviewer, with Codex as the cross-model reviewer.

Next step: owner review/commit of the docs-only spec, then invoke `writing-plans`.
