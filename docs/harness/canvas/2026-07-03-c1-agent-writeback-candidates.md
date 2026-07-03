# Review Canvas: c1-agent-writeback-candidates

Date: 2026-07-03
Owner: Ronaldo + Codex
Scope: Add pending agent writebacks as `agent_writeback` dream candidates without a new writeback table.

## Trigger

| Trigger | Evidence |
|---|---|
| Storage migration | `SCHEMA_VERSION` moves to 45 so the existing `dream_candidates.kind` CHECK accepts `agent_writeback`. |
| MCP surface change | New Advanced-tier `memory_agent_writeback` tool creates pending candidates behind `dream-phase`. |
| Generated memory governance | Agent-generated memory must remain pending/evidence-only until review and explicit apply. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| New writeback table | Rejected | Violates the approved condition to reuse the dream candidate review/apply pipeline first. |
| Store agent writebacks as another existing kind plus metadata | Rejected | Avoids migration but hides semantics and makes kind-level review/reporting weaker. |
| Add `agent_writeback` to `dream_candidates` | Accepted | Minimal schema change, keeps review/apply/dry-run semantics, and preserves one pending-candidate path. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| `memory_agent_writeback` dry-run | O(number of sources) | Small JSON response only | Default path writes nothing. |
| confirmed pending candidate creation | O(number of sources) inserts | One candidate row plus source rows | No canonical memory row is created before review/apply. |
| `dream_candidate_apply` | Existing behavior | Existing behavior plus `agent_writeback` kind | Applies only accepted/edited candidates. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Live writeback without `confirm=true` | Integration test expects an error. |
| No evidence sources | Integration test expects an error before candidate creation. |
| Pending candidate applied before review | Integration test expects `dream_candidate_apply` to reject pending state. |
| Schema CHECK rejects new kind | Migration test inserts `agent_writeback` candidate after v45. |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| MCP tools/list advertises unavailable feature-gated tool | Agents call unknown tools | Add `memory_agent_writeback` to dream-phase feature filter. | Protocol test with `dream-phase`; MCP reference check. |
| v45 table rebuild drops existing candidates | Data loss | Copy all `dream_candidates` columns and keep sources table intact. | Migration tests plus review of SQL column list. |
| Agent-created candidate mutates canonical memory too early | Trusted generated memory bypasses governance | Handler writes only `dream_candidates` and sources; canonical apply remains existing review/apply path. | Integration test compares memory count before and after pending creation. |

## Decision

Proceed.

Reason: The accepted approach is the smallest durable change that satisfies the governance requirement: pending generated writebacks reuse the existing dream candidate review/apply path, dry-run is the default, and canonical memory changes only after review plus explicit apply.
