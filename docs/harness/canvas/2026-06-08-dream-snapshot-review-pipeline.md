# Review Canvas: dream-snapshot-review-pipeline

## Intent

Define the contract for a non-destructive Dream Snapshot Review Pipeline before
storage, MCP, or intelligence code changes begin.

The pipeline should synthesize review candidates from existing memories,
Operational Context, memory policy, temporal graph, enrichment events, and
harness records. It must not mutate canonical memories until a user or agent
explicitly reviews and applies a candidate.

## Approaches Considered

| Approach | Outcome | Reason |
|---|---|---|
| Reuse `memories` for unaccepted candidates | Rejected | Proposed facts would be confused with accepted canonical memory. |
| Auto-apply high-confidence candidates | Rejected for v1 | Confidence is not calibrated enough to mutate memory without review. |
| Store candidates in dedicated tables | Accepted | Keeps provenance, review state, and rollback clear. |
| Extend existing `dream_runs` only | Rejected | Run reports do not model candidate review or source provenance. |
| Require external LLM synthesis | Rejected for v1 | Adds cost, privacy, latency, and reproducibility risk. |
| Local deterministic generator first | Accepted | Matches current harness gates and local-first architecture. |

## Hot Path Complexity

Default search and memory writes must not depend on dream candidates.

The new hot paths are isolated:

- creating a dream job;
- listing candidates;
- applying a reviewed candidate.

Dream generation may scan memories and context records, but it runs as an
explicit job, not inside ordinary `memory_search`, `memory_create`, or
`context_build_bundle` paths.

## Hooks And Intelligence Impact

Existing lifecycle hooks may record evidence that later dream jobs read, but
hooks must not trigger candidate application. Hook output remains source
material, not a review decision.

The first intelligence layer is deterministic and local. External LLM synthesis,
automatic scheduling, and auto-acceptance are outside this phase. This keeps the
contract reproducible enough for harness review and makes failures auditable.

## Cross-SDK Impact

The first implementation is MCP-first. Python and TypeScript SDK changes are
not required until the MCP review tools stabilize, but generated MCP reference
docs must name the new tools and preserve the distinction between candidate
proposals and canonical memories.

SDK follow-up work should add typed wrappers for job creation, candidate
inspection, review, and confirmed application. SDK helpers must keep
`confirm=true` explicit for mutating apply calls.

## Edge Cases

1. **Partial job failure:** a job emits three candidates and then fails. The
   emitted candidates remain inspectable, the job status becomes `failed`, and
   the error is stored. No canonical memories are mutated.

2. **Stale planned event:** a memory says a deployment is planned for a future
   date. After that date passes, the dream job emits a `stale_fact` or
   `temporal_update` candidate with source memory id and event timestamp. It
   does not rewrite the original memory automatically.

3. **Conflicting facts:** two source memories disagree. The job emits a
   `contradiction` candidate with both source ids, confidence, and reason codes.
   It does not delete either source.

4. **Missing source artifact:** a context summary references an artifact that no
   longer has retained raw content. The candidate can still reference the
   artifact pointer but must mark evidence as incomplete or low-trust.

5. **Repeated rejection:** a user rejects a candidate. Later dream jobs should
   retain that review signal so the same proposal is not repeatedly surfaced
   without new evidence.

## Breakage Risk

| Area | Risk | Mitigation |
|---|---|---|
| Storage schema | Version drift or migration failure | Dedicated migration tests and schema docs. |
| MCP surface | Tool contract drift | Protocol tests and generated MCP reference check. |
| Canonical memory | Silent mutation by synthesis | Review-only candidates; `confirm=true` for apply. |
| Search/context | Proposals confused with facts | Default search/context excludes unaccepted candidates. |
| Provenance | Candidate without evidence | Require candidate sources or mark low-trust scratch. |
| Performance | Full scans in user-facing paths | Run synthesis as explicit jobs, not search hot path. |
| Security | Raw logs or secrets captured | Reuse Operational Context redaction and raw artifact policy. |
| Concurrency | Overlapping dream jobs | Reuse advisory lock/job lifecycle and idempotent cancellation. |

## Required Follow-Up Review

Storage, MCP, generator, freshness, harness dogfooding, and eval phases all need
independent post-review. This canvas is evidence for the product contract only;
it does not approve later code changes.

## Verification For This Contract Change

```bash
bash docs/harness/bin/doctor.sh
git diff --check
```
