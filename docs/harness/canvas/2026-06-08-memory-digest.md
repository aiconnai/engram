# Review Canvas: memory-digest

Date: 2026-06-08
Owner: Codex
Scope: Add the `memory_digest` contract and prepare a read-only MCP
implementation plan for `ENGRA-103`.

## Trigger

| Trigger | Evidence |
|---|---|
| Future MCP surface change | RFC 0008 proposes new tool `memory_digest` |
| More than trivial product behavior | Tool orchestrates search, graph, context, and Operational Context |
| Provenance-sensitive output | Digest claims must cite source memory/context IDs |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Thin orchestrator over existing tools | Accepted | Highest ROI, no schema, no parallel index, matches existing handler style |
| New digest storage table | Rejected for v1 | Would turn a read-only UX feature into a persistence/schema change |
| LLM-generated summaries | Rejected for v1 | Adds network/provider variance and makes deterministic tests harder |
| Reuse only `memory_build_context` | Rejected | Does not expose source IDs, relationships, next actions, or Operational Context as first-class fields |
| Reuse only `context_build_bundle` | Rejected | Covers operational resume context but not general memory/topic digest |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| Initial smart retrieval | Same order as `memory_smart_retrieve` | Bounded by `limit` | Existing handler already serially composes strategies |
| Context build | Same order as `memory_build_context` | Bounded by `total_budget` | Should forward existing budget/type/timeframe filters |
| Relationship lookup | O(selected memory IDs * local edge lookup) | Bounded by selected IDs | Keep `related_depth <= 2` in v1 |
| Operational Context bundle | Bounded by `max_results`/section limits | Bounded arrays | Raw artifacts remain excluded |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Empty or whitespace topic | Tool returns structured validation error, no panic |
| No matching memories | Tool returns valid empty digest with warning/omitted reason |
| Duplicate memory IDs from multiple strategies | Deduplicate while preserving first-ranked order |
| Graph disabled | No `relationships` rows returned when `include_graph=false` |
| Operational Context absent | Empty operational arrays, not an error |
| Malformed enum/numeric inputs | Structured validation error or bounded clamping per RFC |
| Source-less derived text | Test requires source IDs or explicit warning |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| MCP registry drift | Tool advertised incorrectly or docs stale | Generate `docs/MCP_TOOLS.md` from code | `./scripts/generate-mcp-reference.sh --check` |
| Handler panic on adversarial input | MCP client sees runtime failure | Parse at boundary and avoid unwrap in production path | Unit/protocol tests + clippy |
| Digest implies canonical truth | Agents over-trust derived summary | Source IDs on all durable claims and warnings for low-confidence text | Behavioral tests inspect provenance |
| Hidden mutation | Violates read-only contract | No storage writes; read-only annotation | Code review + tests around no schema/write calls |
| Response too large | Agent context bloat | `limit`, `related_depth`, and `total_budget` bounds | Tests for bounds and truncation behavior |

## Decision

Proceed as a two-step change:

1. Land RFC 0008, this canvas, and the implementation plan.
2. Implement `memory_digest` in a separate MCP-surface PR for `ENGRA-103`.

Reason: the product contract has enough hidden design choices to justify a
contract-first PR, while the implementation can remain small and reviewable once
the response shape is accepted.
