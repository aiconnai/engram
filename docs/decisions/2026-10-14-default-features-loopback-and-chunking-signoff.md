# ADR: Default Features, Loopback Tool Scope, and Document Chunking Sign-off

- **Status:** Accepted
- **Date:** 2026-10-14
- **Decider:** Ronaldo (Repository Owner)
- **Review basis:** Architecture and security reviews across lanes P, Q, and R (2026-10 improvement plan)

## Context

Following the comprehensive improvement baseline (2026-10-02) and subsequent stabilization tasks, three architectural and security boundaries remained open for explicit repository owner confirmation:

1. **D7 (`Cargo.toml` default features):** Whether `default = ["openai"]` should remain the standard build configuration without `cloud` (`aws-sdk-s3`).
2. **C1 (Anonymous HTTP Loopback & Restricted Principal Scope):** The authorization boundary between full administrative access (`ProcessBearer`, stdio sessions) and restricted principals (such as unauthenticated loopback HTTP requests).
3. **#152 (Document Ingestion Chunk Boundaries):** Confirmation of character-based chunking (`chars()` boundaries) versus introducing token-based chunking on the ingestion hot path.

## Decisions

### 1. D7 — `Cargo.toml` Default Features (`default = ["openai"]`)

- **Ruling:** Confirmed as the intentional public build contract.
- **Rationale:**
  - `default = ["openai"]` ensures lightweight, fast local developer builds and test execution without compiling heavy cloud SDK dependencies (`aws-sdk-s3`).
  - Cloud synchronization remains an explicit opt-in compilation feature via `--features cloud`.
  - CI pipelines and cloud deployments explicitly pass `--features cloud` when cloud storage integration is required.

### 2. C1 — Anonymous HTTP Loopback & Restricted Principal Scope

- **Ruling:** Confirmed as the intended anti-IDOR security boundary.
- **Rationale:**
  - Restricted principals—including unauthenticated loopback HTTP callers—are strictly denied access to the 65 by-ID unscoped tools.
  - By-ID unscoped tools operate across workspace boundaries without an explicit workspace context and are reserved exclusively for administrative principals holding a valid `ProcessBearer` token or running via direct `stdio` local-owner sessions.
  - Anonymous loopback callers cannot bypass workspace isolation or access foreign workspace resources through by-ID tool invocations.

### 3. #152 — Document Ingestion Chunk Boundaries (Character-based Chunking)

- **Ruling:** Confirmed as the production contract.
- **Rationale:**
  - Document ingestion in `src/intelligence/document_ingest.rs` retains character-based chunking (`DEFAULT_CHUNK_SIZE = 1200`, `DEFAULT_OVERLAP = 200` in characters, sliced cleanly on `char` boundaries).
  - `TokenChunker` is not integrated into the hot ingestion path, avoiding tokenizer overhead, model dependency coupling, and runtime latency during ingestion.
  - Character boundary slicing ensures UTF-8 safety while keeping ingestion predictable and model-agnostic. Any future token-budgeting requirements must be designed as dedicated, opt-in post-processing stages rather than hard requirements in the core ingestion pipeline.

## Consequences

- Discrepancy **D7** from [`../harness/audits/2026-10-02-improvement-baseline.md`](../harness/audits/2026-10-02-improvement-baseline.md) is closed as Accepted.
- The security model for **C1** is formally codified: unauthenticated loopback HTTP requests are classified as restricted principals subject to unscoped-tool denial.
- The controller ruling for issue **#152** documented in [`../harness/context-budget.md`](../harness/context-budget.md) is confirmed by repository owner Ronaldo.
