# Chair Synthesis — Lifecycle Predicate Unification Council

Date: 2026-06-27
Scope: `docs/superpowers/specs/2026-06-27-lifecycle-predicate-unification-design.md`
Mode: Level 1 council (isolated subagents + Codex cross-model)
Provenance tokens: `[prov:agent:First-Principles]`, `[prov:agent:Skeptic]`, Codex (`codex-cli 0.142.0`, in-repo, file:line verified)

## Verdict

REVIEW_VERDICT: FAIL lifecycle predicate spec is not ready for implementation planning

Confidence: high — three independent voices converged on the same blocker class.

## Blocking findings synthesized

- [BLOCKER] Writer enumeration was still incomplete. The prior spec listed three decay engines plus four domain writers, but review found additional lifecycle writers: `memory_archive_old` (`src/mcp/handlers/summarize.rs:329`), conflict-resolution archive writes (`src/intelligence/context_quality.rs:730,737`), and retention compression archive writes (`src/storage/queries/retention.rs:312`).
- [BLOCKER] `memory_archive_old` is not a simple domain writer. It archives by `max_age_days`, `max_importance`, and `min_access_count` (`src/mcp/handlers/summarize.rs:231-269`), making it a disguised decay/compression engine that must converge with the canonical predicate or be disarmed as a lifecycle writer.
- [HIGH] `retention_policy_apply` auto-delete is creation-age based (`src/storage/queries/retention.rs:198-205`), not time-since-archived. Canonical lifecycle archiving can feed rows into this path if `auto_delete_after_days` is configured; the spec must explicitly accept this preexisting semantic or add a lifecycle timestamp migration.
- [MED] The predicate assumed `importance ∈ [0,1]`, but storage paths can carry raw `f32`; the lifecycle predicate must normalize finite values and handle non-finite values before computing the multiplier.
- [LOW] Disarming `memory_decay` is acceptable for this spec only if retention-score-as-lifecycle-input becomes an explicit follow-up, because retention score carries reinforcement/durability/centrality signals.
- [LOW] `SalienceScore.suggested_state` cannot keep using `suggest_lifecycle_state`; it must be removed or delegated to `decide_lifecycle_state`.

## Decision after chair synthesis

Correct the spec before any implementation plan. The core hybrid predicate remains sound, but the writer taxonomy and boundary conditions must be fixed first.

## Reversal triggers

A future PASS requires:

1. Exhaustive writer inventory in the spec, including `memory_archive_old`, `context_quality`, and retention compression.
2. `memory_archive_old` classified as a decay/compression engine, not a normal domain writer.
3. Explicit cap × retention auto-delete semantics, preserving zero-migration only if creation-age deletion is accepted.
4. Mandatory tests for disarming `memory_decay` and `memory_archive_old` lifecycle writes.
5. A concrete choice for `SalienceScore.suggested_state`.
6. Importance normalization inside `decide_lifecycle_state`.
