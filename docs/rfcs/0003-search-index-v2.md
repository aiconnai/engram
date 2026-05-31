# RFC 0003: Search Index v2 (local-first architecture and optional scalable backends)

## Status

Accepted

## Context

Engram currently uses SQLite + FTS5 as the default local search path.  
Meilisearch exists as a feature-gated optional backend, and there is recurring interest in evaluating additional engines (Tantivy, Manticore, ANN/HNSW) for performance or retrieval quality.

Before changing contracts for search behavior, harness context identified the need for a formal decision to avoid introducing parallel, undocumented search surfaces and index lifecycle ambiguity.

## Question

Should Engram adopt an index v2 now, and if so, what is the default search stack and which optional backends are supported?

## Requirements

- Preserve local-first behavior and operational simplicity for the default OSS runtime.
- Minimize data corruption risk and improve recoverability.
- Keep delete/re-index behavior explicit and auditable.
- Keep CI/build/test complexity manageable.
- Avoid choosing a backend whose operational blast radius is currently unproven.
- Ensure every index path is:
  - rebuildable,
  - health-checkable,
  - drift-detectable,
  - and disposable.

## Options compared

### 1) SQLite + FTS5 (current default)

- **Model:** local embedded full-text index in SQLite (FTS5), hybrid ranking already layered in `src/search`.
- **Local-first fit:** Excellent (no external service).
- **Rebuild strategy:** `REINDEX`/recreate path; local file backup/restore aligns with existing DB ops.
- **Disk growth:** Moderate and coupled to DB growth; predictable and co-located with core data.
- **Delete propagation:** Deterministic with DB transaction scope.
- **Ranking quality:** Current production baseline (BM25/fuzzy/hybrid stack).
- **CI/test burden:** Already covered in existing codebase and harness gates.
- **Operational weight:** Low (single process/storage).

### 2) Meilisearch (external, feature-gated option)

- **Model:** external index service synchronized as an optional mirror.
- **Local-first fit:** Medium (requires extra process/service dependency).
- **Rebuild strategy:** Re-sync/re-index path; operationally manageable, but external drift can occur.
- **Disk growth:** External disk + networked payload overhead.
- **Delete propagation:** Requires robust sync semantics and periodic drift reconciliation.
- **Ranking quality:** Strong for semantic-like product search; not always superior for exact local corpus workflows.
- **CI/test burden:** Higher than baseline (service dependency, docker/service setup).
- **Operational weight:** Medium.

### 3) Tantivy embedded

- **Model:** embedded index engine alternative.
- **Local-first fit:** Good, but with larger code and migration surface than FTS5.
- **Rebuild strategy:** Requires custom index lifecycle code (migration, repair, corruption handling).
- **Disk growth:** Medium/High depending on vector/segment model.
- **Delete propagation:** Manual and error-prone without deep integration.
- **Ranking quality:** Potentially strong, uncertain for current hybrid contract.
- **CI/test burden:** Medium/High (additional invariants, custom harness assertions).
- **Operational weight:** Medium.

### 4) Manticore (external)

- **Model:** external search platform.
- **Local-first fit:** Medium/Low for OSS default due to infra coupling.
- **Rebuild strategy:** Operationally heavy; external schema lifecycle required.
- **Disk growth:** External and service-driven.
- **Delete propagation:** Requires asynchronous cleanup contracts.
- **Ranking quality:** Potentially high but currently unproven in this repo’s contract.
- **CI/test burden:** High (external infra in CI scenarios).
- **Operational weight:** High.

### 5) ANN/HNSW direct path

- **Model:** approximate nearest-neighbor index as primary retrieval layer.
- **Local-first fit:** Variable (local possible, but current incident history raises guardrail concern).
- **Rebuild strategy:** Expensive rebuild/reindex cadence; index repair complexity.
- **Disk growth:** High relative to deterministic sparse indexes for current corpus sizes.
- **Delete propagation:** Known operational complexity; stale vectors and dangling nodes can amplify drift.
- **Ranking quality:** Good for vector semantics, weaker for strict text recall unless hybridized.
- **CI/test burden:** Medium/High (fidelity + recall drift under mutation).
- **Operational weight:** High.

## Decision

### Keep SQLite + FTS5 as default local-first stack

- Default behavior remains **SQLite + FTS5 + existing hybrid ranking**.
- Feature-gated Meilisearch remains accepted as **optional external mirror** where external infra is already justified.
- No adoption of Tantivy/Manticore/ANN/HNSW as default in this phase.

### Release posture

- Continue shipping only a single default local search surface for OSS baseline stability.
- Keep index optionality explicit and configuration-gated.
- Avoid adding ANN/HNSW (or similar) as default until measured need demonstrates acceptable:
  - rebuild + delete propagation cost,
  - test flakiness and CI burden,
  - operational recoverability profile,
  - and quality uplift not achieved by current hybrid stack.

## Guardrails (derived from Chroma/HNSW incident)

- **No derived index is source of truth.** Source of truth remains core memory rows.
- Every index backend must be:
  - rebuildable,
  - health-checkable (`derived_index` health contract),
  - drift-detectable (missing/orphaned entries surfaced),
  - and disposable (safe drop/recreate path).
- Any optional backend must have:
  - explicit out-of-band repair workflow,
  - bounded retention and observability,
  - and deterministic emergency fallback to source-of-truth rebuild.

## Migration path from current FTS5 behavior

1. Keep existing schema and health contract for FTS5 stable for local runtime.
2. Treat Meilisearch as opt-in feature:
   - enable explicit backend selection,
   - document required infra and synchronization semantics,
   - keep migration docs explicit that local REST-like behavior is not altered.
3. Defer ANN/HNSW/Tantivy/Manticore adoption to dedicated RFC + benchmark issue(s) before schema or API-contract changes.
4. Before any future backend switch, require:
   - a measured benchmark package (quality + latency + rebuild time + delete lag + disk growth),
   - health and drift tests in harness runbook,
   - rollback + cleanup path in maintenance contracts.

## Acceptance summary for issue #29

This RFC explicitly compares:
- FTS5, Tantivy, Meilisearch, Manticore, ANN/HNSW;
- operational weight, local-first fit, rebuild, disk growth, delete propagation, ranking quality, and CI/test burden;
- recommends default and optional backends;
- applies explicit guardrails and migration path from current behavior.

Decision outcome:
- **Default:** SQLite + FTS5 + hybrid ranking.
- **Optional:** Meilisearch feature-gated as external mirror.
- **Not yet adopted:** Tantivy/Manticore/ANN/HNSW until measured and bounded by dedicated benchmark/incident-ready RFCs.
