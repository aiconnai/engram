---
adr: ADR-STORAGE-20260714-1
track: storage-architecture
service: engram-core
status: Accepted
owner: Engram Architecture Team
created: 2026-07-14
---

# Turso / libSQL Backend Architectural Status and Production Roadmap

## Context & Problem Statement

Engram is a persistent memory system designed for AI agents and engineering teams, prioritizing low latency, local-first operation, deterministic reproducibility, and zero-network overhead on core agent execution loops.

During Phase 6 (ENG-54), support for **Turso / libSQL** was introduced under the optional `turso` Cargo feature (`src/storage/turso_backend/`). The objective was to explore distributed edge replicas, cloud-synchronized SQLite storage, and multi-node read replication.

While the basic CRUD and health-check operations for `TursoBackend` are functional and verified in isolated integration tests (`tests/turso_backend_tests.rs`), architectural tensions and feature gaps prevent Turso/libSQL from being treated as a production-grade default:

1. **Transaction Wrapping & Trait Model**:
   - `StorageBackend::with_transaction` expects a synchronous execution closure receiving `&dyn StorageBackend`.
   - In `TursoBackend`, libSQL's asynchronous handle architecture cannot safely pass a transaction-scoped `&dyn StorageBackend` without an async transaction refactor. As a result, calling `with_transaction` on `TursoBackend` returns an explicit unsupported storage error.
2. **C-ABI & Linker Collisions**:
   - Under combined compilation (e.g. `--all-features`), static linking of both `libsqlite3-sys` (used by `rusqlite`) and `libsql-sys` (used by `libsql`) can cause symbol collision and global state initialization conflicts in the SQLite C runtime unless strictly partitioned.
3. **Vector Search Parity**:
   - Engram's fast vector search pipeline utilizes `sqlite-vec` and embedded SIMD cosine similarity. LibSQL handles vector indexing differently across server and edge modes, creating indexing divergence.
4. **Sync Delta Extensions**:
   - Cloud sync primitives such as `sync_delta` and `sync_state` are currently stubbed as unsupported on `TursoBackend`.

## Decision

1. **Canonical Engine**:
   - **Local SQLite with Write-Ahead Logging (WAL)** via `rusqlite` is and remains the **canonical, default, and primary production storage engine** for Engram.
   - All standard deployments, CLI invocations, and MCP transport servers default to SQLite WAL mode (`ENGRAM_DB_PATH`).

2. **Turso / libSQL Classification**:
   - `TursoBackend` is officially designated as an **Experimental / Opt-in Feature** guarded by `feature = "turso"`.
   - It is not included in default workspace builds and will not be enabled in release binaries until all graduation criteria are satisfied.

3. **Graduation Criteria for Production Readiness**:
   To graduate `TursoBackend` from experimental status to a fully supported production tier, the following engineering gates must be met:
   - **Gate 1 (Transaction Parity)**: Implement transaction-scoped storage handles or async transaction semantics allowing safe transactional execution.
   - **Gate 2 (Vector Search Parity)**: Implement complete vector similarity search and metadata filtering parity with Engram's embedding engine.
   - **Gate 3 (Linker & ABI Safety)**: Ensure clean C-ABI separation and unified SQLite runtime initialization to prevent symbol collisions when compiled alongside `rusqlite`.
   - **Gate 4 (End-to-End Replication CI)**: Introduce automated CI workflows testing live replica synchronization against a Turso/libSQL cloud endpoint or emulator.

## Consequences

### Positive
- Clear separation of concerns and explicit operational expectations for users and contributors.
- Zero risk of regressions in local-first production workloads.
- Targeted roadmap for contributors interested in distributed edge storage.

### Negative / Trade-offs
- Users requiring cloud synchronization must rely on Engram's S3/R2 snapshot sync or run Turso in experimental mode with awareness of transaction limitations.
