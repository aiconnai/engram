REVIEW_VERDICT: FAIL writer enumeration is incomplete and the retention-delete interaction is under-specified

# Codex cross-model verdict (extracted from codex exec,   507016 bytes raw)
# Provenance: codex-cli 0.142.0, ran in-repo, verified against code file:line

- [BLOCKER] Spec lines 28-54 claim a full writer enumeration, but grep/code inspection found additional production writers that the spec does not classify. Compact inventory:
  - `src/mcp/handlers/lifecycle.rs:178,184` → decay-engine → agree.
  - `src/intelligence/salience.rs:450` → decay-engine → agree.
  - `src/mcp/handlers/memory_policy.rs:143-150`, `352-357` → decay-engine → agree.
  - `src/storage/queries/lifecycle.rs:28`, `39` → helper → agree; callers decide engine/domain.
  - `src/mcp/handlers/lifecycle.rs:239` → manual domain writer → agree.
  - `src/intelligence/consolidation_offline.rs:568` → consolidation domain writer → agree.
  - `src/storage/queries/retention.rs:182` → retention max-count domain writer → agree.
  - `src/storage/queries/retention.rs:312` → retention compression domain writer → spec missed this write site.
  - `src/storage/queries/retention.rs:202-205` → retention soft-delete, not lifecycle write → boundary relevant.
  - `src/mcp/handlers/dream.rs:377` → dream-approved expire domain writer → agree.
  - `src/intelligence/context_quality.rs:730,737` → conflict-resolution domain writer → spec missed.
  - `src/mcp/handlers/summarize.rs:329` → `memory_archive_old` compression/archive writer → spec missed; it is also the strongest “disguised decay engine” candidate because it archives by `max_age_days`, `max_importance`, and `min_access_count` at `src/mcp/handlers/summarize.rs:231-269`.
  - `src/storage/turso_backend.rs:651-656`, `675` → initial-create/default `active` writer → irrelevant to transitions, but still a lifecycle-state write.
  - `src/storage/migrations.rs:926` → schema default initializer → irrelevant.
  - `src/mcp/handlers/lifecycle.rs:488-490`, `src/storage/queries/tests.rs:1543` → test fixtures/helper tests → irrelevant.
  The spec must either add the missed domain writers or explicitly fold `memory_archive_old` into the decay-engine convergence decision before implementation.

- [HIGH] Spec lines 236-240 and 315 say the cap “archives, never deletes,” but `retention_policy_apply` deletes archived rows using `created_at < cutoff`, not “time since archived” (`src/storage/queries/retention.rs:198-205`). A newly archived 400-day-old memory from the canonical cap/normal archive path can be soft-deleted on the next retention apply if `auto_delete_after_days` is configured. With no `lifecycle_changed_at`, this boundary is not clean unless the spec explicitly accepts creation-age-based deletion semantics.

- [MED] Spec line 155 assumes `importance ∈ [0,1]`, but core storage paths accept `Option<f32>` without visible clamping: `CreateMemoryInput.importance` at `src/types.rs:875`, `UpdateMemoryInput.importance` at `src/types.rs:917`, create uses `unwrap_or(0.5)` at `src/storage/turso_backend.rs:623`, and SQLite update writes raw importance at `src/storage/queries/core.rs:901-904`. The predicate should clamp/normalize finite importance before computing `mult`, or the spec should cite the actual boundary that enforces the range.

- [LOW] The strongest case against disarming `memory_decay` is real but not enough to keep it as a parallel writer: `retention_score` already incorporates reinforcement, durability, source confidence, graph centrality, contradiction risk, and age decay (`src/intelligence/memory_policy/scoring.rs:30-38`; feature inputs in `src/intelligence/memory_policy/features.rs:20-35`). Silencing the lifecycle transition at spec lines 67-73 drops those non-temporal signals from lifecycle decisions. I still agree with disarming for this spec, but the follow-up should be mandatory/explicit if retention is intended to influence lifecycle.

- [LOW] Leaving `SalienceScore.suggested_state` as-is would be a real public divergence risk: `salience_get` exposes it via `src/mcp/handlers/quality.rs:217-235`, and it is currently computed by legacy `suggest_lifecycle_state` at `src/intelligence/salience.rs:210-211`, `254-278`. Spec lines 296-306 correctly require removal or recomputation via the canonical predicate; recomputation is the safer compatibility choice because removing the field would change the response shape.


hook: Stop
hook: Stop Completed
tokens used
174,476
REVIEW_VERDICT: FAIL writer enumeration is incomplete and the retention-delete interaction is under-specified

- [BLOCKER] Spec lines 28-54 claim a full writer enumeration, but grep/code inspection found additional production writers that the spec does not classify. Compact inventory:
  - `src/mcp/handlers/lifecycle.rs:178,184` → decay-engine → agree.
  - `src/intelligence/salience.rs:450` → decay-engine → agree.
  - `src/mcp/handlers/memory_policy.rs:143-150`, `352-357` → decay-engine → agree.
  - `src/storage/queries/lifecycle.rs:28`, `39` → helper → agree; callers decide engine/domain.
  - `src/mcp/handlers/lifecycle.rs:239` → manual domain writer → agree.
  - `src/intelligence/consolidation_offline.rs:568` → consolidation domain writer → agree.
  - `src/storage/queries/retention.rs:182` → retention max-count domain writer → agree.
  - `src/storage/queries/retention.rs:312` → retention compression domain writer → spec missed this write site.
  - `src/storage/queries/retention.rs:202-205` → retention soft-delete, not lifecycle write → boundary relevant.
  - `src/mcp/handlers/dream.rs:377` → dream-approved expire domain writer → agree.
  - `src/intelligence/context_quality.rs:730,737` → conflict-resolution domain writer → spec missed.
  - `src/mcp/handlers/summarize.rs:329` → `memory_archive_old` compression/archive writer → spec missed; it is also the strongest “disguised decay engine” candidate because it archives by `max_age_days`, `max_importance`, and `min_access_count` at `src/mcp/handlers/summarize.rs:231-269`.
  - `src/storage/turso_backend.rs:651-656`, `675` → initial-create/default `active` writer → irrelevant to transitions, but still a lifecycle-state write.
  - `src/storage/migrations.rs:926` → schema default initializer → irrelevant.
  - `src/mcp/handlers/lifecycle.rs:488-490`, `src/storage/queries/tests.rs:1543` → test fixtures/helper tests → irrelevant.
  The spec must either add the missed domain writers or explicitly fold `memory_archive_old` into the decay-engine convergence decision before implementation.

- [HIGH] Spec lines 236-240 and 315 say the cap “archives, never deletes,” but `retention_policy_apply` deletes archived rows using `created_at < cutoff`, not “time since archived” (`src/storage/queries/retention.rs:198-205`). A newly archived 400-day-old memory from the canonical cap/normal archive path can be soft-deleted on the next retention apply if `auto_delete_after_days` is configured. With no `lifecycle_changed_at`, this boundary is not clean unless the spec explicitly accepts creation-age-based deletion semantics.

- [MED] Spec line 155 assumes `importance ∈ [0,1]`, but core storage paths accept `Option<f32>` without visible clamping: `CreateMemoryInput.importance` at `src/types.rs:875`, `UpdateMemoryInput.importance` at `src/types.rs:917`, create uses `unwrap_or(0.5)` at `src/storage/turso_backend.rs:623`, and SQLite update writes raw importance at `src/storage/queries/core.rs:901-904`. The predicate should clamp/normalize finite importance before computing `mult`, or the spec should cite the actual boundary that enforces the range.

- [LOW] The strongest case against disarming `memory_decay` is real but not enough to keep it as a parallel writer: `retention_score` already incorporates reinforcement, durability, source confidence, graph centrality, contradiction risk, and age decay (`src/intelligence/memory_policy/scoring.rs:30-38`; feature inputs in `src/intelligence/memory_policy/features.rs:20-35`). Silencing the lifecycle transition at spec lines 67-73 drops those non-temporal signals from lifecycle decisions. I still agree with disarming for this spec, but the follow-up should be mandatory/explicit if retention is intended to influence lifecycle.

- [LOW] Leaving `SalienceScore.suggested_state` as-is would be a real public divergence risk: `salience_get` exposes it via `src/mcp/handlers/quality.rs:217-235`, and it is currently computed by legacy `suggest_lifecycle_state` at `src/intelligence/salience.rs:210-211`, `254-278`. Spec lines 296-306 correctly require removal or recomputation via the canonical predicate; recomputation is the safer compatibility choice because removing the field would change the response shape.
