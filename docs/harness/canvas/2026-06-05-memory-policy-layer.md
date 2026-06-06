# Review Canvas: memory-policy-layer

## Intent

Add an auditable memory policy layer that computes salience, retention, retrieval priority, promotion, decay, and conflict explanations over explicit Engram memories.

This implementation intentionally does not auto-write synthesized memories. Synthesis v1 is reviewable candidates only; this policy layer provides scoring and explanations that future candidates can reuse.

## Approaches Considered

| Approach | Outcome | Reason |
|---|---|---|
| Store learned state in model weights | Rejected | Breaks deletion, review, provenance, and reproducibility. |
| Auto-write synthesized memories | Rejected for v1 | Highest risk of stale or wrong memory pollution; reviewable candidates must come first. |
| Auto-write high-confidence synthesized memories | Deferred | Better UX later, but requires calibrated confidence, conflict handling, and review history. |
| Extend existing salience only | Rejected | Salience lacks retention and retrieval-priority policy records. |
| Add separate policy table keyed by memory ID | Accepted | Keeps canonical facts explicit and makes policy debuggable. |

## Hot Path Complexity

Search adds one optional rerank pass over returned candidates. Phase 1 must not add a full-table scan to default retrieval.

## Edge Cases

| Case | Expected Behavior |
|---|---|
| Memory has no policy row | Compute transient default and optionally upsert when mutating. |
| Memory is archived | Keep it excluded unless caller opts into archived results. |
| Memory has contradictions | Demote confidence and retrieval priority, do not delete. |
| Existing DB at schema 40 | Migration creates policy table and backfills lazily. |
| Synthesizer proposes stale project state | Candidate remains review-only; policy can explain freshness risk but cannot mutate canonical memory automatically. |

## Breakage Risk

| Surface | Risk | Mitigation |
|---|---|---|
| SQLite schema | High | Bump `SCHEMA_VERSION`, migration tests, schema version tests. |
| MCP tools | High | Protocol tests, generated reference update. |
| Search ranking | Medium | Add explicit `policy_rerank` parameter first; avoid default behavior flip. |
| Hooks | Medium | Best-effort policy events only; never abort user flow. |
| Future synthesis | High | Keep candidates separate from canonical memories and require explicit approval in v1. |
