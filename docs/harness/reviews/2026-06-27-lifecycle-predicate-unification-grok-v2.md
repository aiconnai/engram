I'll run the harness bootstrap, read the spec, and grep the codebase for every lifecycle-state write site to verify the spec's claims.
Broadening the search for lifecycle-state writes and reading the key implementation sites.
```text
REVIEW_VERDICT: PASS Prior council FAIL items (writer enumeration + retention-delete boundary) are fixed and verified against code; remaining gaps are follow-up/operational, not plan-blocking.
```

## Compact write-site inventory (grep-verified)

| Site | Class (code reality) | Spec classification | Agree? |
|---|---|---|---|
| `lifecycle.rs:178,184` | decay engine (`lifecycle_run`) | decay engine | ✓ |
| `salience.rs:450` | decay engine (`run_salience_decay`) | decay engine | ✓ |
| `memory_policy.rs:146` (+ predicate `352-354`) | decay engine (`memory_decay` → `Stale`) | decay engine | ✓ |
| `summarize.rs:329` (+ filter `263-267`) | decay/compression engine (`memory_archive_old`) | decay engine | ✓ |
| `retention.rs:182` | domain (`max_memories` cap) | domain | ✓ |
| `retention.rs:312` (`compress_old_memories`) | domain (retention compression) | domain | ✓ |
| `retention.rs:202-205` | domain soft-delete (`valid_to`, not lifecycle write) | domain boundary (documented) | ✓ |
| `context_quality.rs:730,737` | domain (conflict `KeepA`/`KeepB`) | domain | ✓ |
| `consolidation_offline.rs:568` | domain (consolidation archive) | domain | ✓ |
| `dream.rs:377` (`expire` apply) | domain (approved dream action) | domain (`dream candidate apply`) | ✓ |
| `lifecycle.rs:239` | domain (manual `memory_set_lifecycle`) | domain | ✓ |
| `storage/queries/lifecycle.rs:28,39` | helper (`update_memory_lifecycle_state`) | helper | ✓ |
| `turso_backend.rs:675`, `migrations.rs:926` | initializer/default `active` | helper/init | ✓ |
| `storage/queries/tests.rs:1543`, `benches/search.rs:328` | test/bench fixtures | non-engine | ✓ |

**Not lifecycle writes:** `dream/candidates.rs:476` reads `lifecycle_state` and proposes `expire`; the actual write is `dream.rs:377`.

---

## Findings

- **[HIGH]** Spec omits the automatic compression entry point `server.rs:726-749`, which calls `compress_old_memories` (write at `retention.rs:312`) when `ENGRAM_COMPRESSION_INTERVAL > 0`. Default is `0` (disabled), but when enabled this is an undeclared automatic archiver using the same `created_at`/importance/access predicate being removed from `memory_archive_old`. Spec lines 49-59 / 64-68 list the write site but not this caller; implementation plan should either document it as an alternate domain entry point or require it to respect the disarmed-predicate rule. Verified: `server.rs:124-136` (defaults), `server.rs:747`.

- **[MED]** Retention auto-delete boundary is now explicit (spec lines 275-283, 337-340) and matches code: `retention.rs:198-205` filters `lifecycle_state = 'archived' AND created_at < cutoff` — creation-age, not time-since-archived. Documenting/accepting this is **not a spec blocker** for zero-migration, but the interaction is broader than the spec’s 400-day example: any memory archived by canonical `lifecycle_run` whose `created_at` is already older than `auto_delete_after_days` can be soft-deleted on the next `retention_policy_apply`. Operational risk is real; test #9 mitigates if implemented as written.

- **[MED]** `memory_archive_old` disarm (spec lines 86-92, 356-359) is the right convergence for single-writer invariant: it currently archives via `summarize.rs:329` using the same divergent predicate as `lifecycle_run` (`summarize.rs:263-267`). Preserving compression only for already-`Archived` rows will materially shrink the tool until `lifecycle_run` runs or a follow-up `compress_archived_memories` exists — acceptable tradeoff, explicitly acknowledged at spec line 378.

- **[MED]** `memory_decay` disarm (spec lines 79-85, 352-357) is sufficient for unification scope. Code confirms only `Active → Stale` when `new_retention < 0.25` (`memory_policy.rs:352-354`); deferring retention-score as `decide_lifecycle_state` input (spec line 377) is a product tradeoff, not an architectural hole.

- **[LOW]** Spec invariant line 262 cites `dream/candidates.rs expire path`, but the write is `dream.rs:377` on approved `expire` action. Enumeration is correct; reference is imprecise.

- **[LOW]** Hybrid predicate arithmetic checks out: `archive_base 90 × max_mult 4.0 = 360 < cap 365` (spec lines 212-218) — cap is dormant under defaults; parametrized cap test at spec line 301 is the right proof strategy.

- **[LOW]** `normalized_importance` inside `decide_lifecycle_state` (spec lines 177, 188-199) is necessary and testable: `core.rs:653` stores `input.importance.unwrap_or(0.5)` without clamp; `salience.rs:197` assumes importance is already `[0,1]` in comments but does not clamp on read. Delegating `SalienceScore.suggested_state` to `decide_lifecycle_state` (spec lines 127, 334-336, 360-367) closes the third public predicate divergence.

- **[LOW]** Monotonicity/idempotence without `lifecycle_changed_at` (spec lines 265-272) is sound for decay-forward-only semantics: direct `active → archived` when `idle_days >= effective_arch` follows from decision order (archive checked before stale at spec lines 182-183). No automatic reversal paths exist in the enumerated writers.

- **[LOW]** Zero-migration claim (spec lines 230-232, 281-283) holds given accepted creation-age retention delete and in-code importance normalization; `SCHEMA_VERSION` is 44 (`migrations.rs:8`).

---

## Validation of prior FAIL fixes

| Prior FAIL | Status |
|---|---|
| Incomplete writer enumeration (`memory_archive_old`, `context_quality`, retention compression) | **Fixed** — all three verified in code and listed in spec lines 36-59 |
| Under-specified retention-delete boundary | **Fixed** — spec lines 275-283 explicitly document `created_at`-based soft-delete at `retention.rs:204` |

---

## Answers to explicit review questions

1. **Writer enumeration:** Complete against independent grep; four decay engines + seven domain surfaces + helpers/tests match code.
2. **`memory_archive_old` convergence:** Correct — disarm lifecycle transitions, keep compression for already-archived rows or defer replacement tool; keeps single decay writer invariant.
3. **`memory_decay`:** Sufficient for this spec; retention-as-input correctly deferred with mandatory follow-up if product needs it.
4. **Retention auto-delete boundary:** Safe to document and accept for zero-migration; not a spec blocker, but ops should understand immediate-delete risk for old `created_at` rows.
5. **Advisory state + importance normalization:** Requirements are sufficient and testable via spec tests #8 and importance clamp cases (lines 306-308).
