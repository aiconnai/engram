REVIEW_VERDICT: FAIL corrected lifecycle predicate spec is not ready for implementation planning

# Chair synthesis — lifecycle predicate unification re-review v2

Date: 2026-06-27
Artifacts:
- Codex: `docs/harness/reviews/2026-06-27-lifecycle-predicate-unification-codex-v2.md`
- Grok substitute: `docs/harness/reviews/2026-06-27-lifecycle-predicate-unification-grok-v2.md`

## Council result

- Codex: **FAIL** — BLOCKER: spec still misses the optional automatic compression scheduler path.
- Grok substitute: **PASS with HIGH finding** — found the same scheduler path but judged it plan-actionable rather than plan-blocking.
- Chair decision: **FAIL**. The two reviewers independently converge on the same unmodeled path: `server.rs` optional compression scheduler calls `compress_old_memories`, which writes `lifecycle_state='archived'` through `retention.rs:312` using the same age/importance/access-count predicate class this spec is trying to eliminate from parallel decay engines.

## Load-bearing finding

The corrected spec enumerates `compress_old_memories` / `retention.rs:312`, but only as retention compression/domain. It does not model the **automatic server entry point**:

- `src/bin/server.rs:122` / config: optional compression interval
- `src/bin/server.rs:726` / scheduler start when interval is enabled
- `src/bin/server.rs:746-747` / scheduler calls `compress_old_memories`
- `src/storage/queries/retention.rs:237` / candidate predicate
- `src/storage/queries/retention.rs:312` / lifecycle write to `archived`

This means the current prose claim that the remaining engines are manual MCP tools / no scheduler is false when compression scheduling is enabled.

## Secondary finding

Codex also flagged public contract cleanup as under-specified: docs and registry metadata still advertise old behavior for `memory_decay`, `memory_archive_old`, `lifecycle_run` (`min_importance`), and `salience_decay_run`. This should be explicit implementation-plan scope, not implied cleanup.

## Dissent / caveat

Grok judged the scheduler issue non-blocking because the write site is already listed and the scheduler defaults to disabled. Chair keeps it BLOCKER because the spec’s central premise is writer topology; an optional scheduler is still an execution path, and this exact project has repeatedly failed on incomplete writer enumeration.

## Required corrections before next review

1. Add `compression scheduler -> compress_old_memories -> retention.rs:312` as an explicit automatic compression/lifecycle writer path.
2. Decide whether scheduled compression is:
   - disarmed for lifecycle transitions in this spec,
   - constrained to already-Archived rows, or
   - treated as a domain exception with its own documented risk.
3. Remove/qualify the claim that all noncanonical decay/compression engines are manual MCP-only.
4. Add implementation-plan scope for MCP registry/schema/reference docs updates: `memory_decay`, `memory_archive_old`, `lifecycle_run.min_importance`, `salience_decay_run`.

No commit and no `writing-plans` until the spec is patched and re-reviewed.
