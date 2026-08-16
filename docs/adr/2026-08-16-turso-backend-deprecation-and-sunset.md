---
adr: ADR-STORAGE-20260816-1
track: storage-architecture
service: engram-core
status: Accepted
owner: Engram Architecture Team
created: 2026-08-16
supersedes: ADR-STORAGE-20260714-1
---

# Turso / libSQL Backend Deprecation and Sunset Strategy

## Context & Problem Statement

Engram is a persistent memory system for AI agents and engineering teams, built on the core values of **low latency, local-first execution, deterministic reproducibility, and zero-network overhead on hot agent execution loops**.

In Phase 6 (ENG-54), support for **Turso / libSQL** was introduced under the optional `turso` Cargo feature (`src/storage/turso_backend/`) to explore distributed edge replicas and cloud-synchronized SQLite storage.

Subsequent architectural reviews, concurrency audits, and empirical verification revealed severe systemic limitations that make long-term maintenance of the in-tree Turso backend untenable:

1. **Async-over-Sync Runtime Nesting Hazard**:
   - The canonical `StorageBackend` trait is synchronous (`fn create_memory(&self, ...) -> Result<Memory>`).
   - `libSQL` is fundamentally asynchronous (`libsql::Connection`).
   - The in-tree `TursoBackend` bridges this mismatch using `tokio::task::block_in_place(|| rt.block_on(async { ... }))` across all 20+ trait methods.
   - When invoked from a `tokio::task::spawn_blocking` thread pool or a single-threaded runtime (e.g. CLI or lightweight embedding worker), `block_in_place` causes runtime panics (`Cannot start a runtime from within a runtime` or worker thread assertion failures).

2. **Transaction Impossibility**:
   - `StorageBackend::with_transaction` accepts a synchronous closure `FnOnce(&dyn StorageBackend) -> Result<R>`.
   - libSQL's asynchronous RAII transaction handles (`libsql::Transaction`) cannot be passed as `&dyn StorageBackend` in synchronous closures. Consequently, `with_transaction` returns an explicit runtime error on `TursoBackend`.

3. **C-ABI & Linker Collisions**:
   - Static linking of `libsql-sys` (bundled with `libsql`) alongside `libsqlite3-sys` (bundled with `rusqlite`) leads to symbol collision and global state initialization conflicts in the SQLite C runtime unless strictly partitioned in separate processes. This prevents combined feature compilation (`--all-features`) from safely running SQLite and Turso in the same test process.

4. **Schema Drift & High Maintenance Friction**:
   - Engram's schema evolves rapidly in `src/storage/migrations.rs`. Because `TursoBackend` cannot execute `rusqlite` migrations, it maintains an independent manual migration runner (`turso_backend/core.rs`), which continuously drifts from the canonical schema.

5. **Architectural Redundancy with Snapshot Attestation**:
   - Engram has since established **S3 / Cloudflare R2 Snapshot Attestation and Synchronization** (`src/storage/dream_snapshots.rs`, `tests/snapshot_attestation.rs`). This architecture provides cryptographic provenance, disaster recovery, and multi-machine sync directly over local SQLite WAL databases with zero network hops in the hot path.

## Considered Alternatives

### Alternative 1: Full Async Refactoring of `StorageBackend`
Convert all `StorageBackend` trait methods to `async fn` or `#[async_trait]`.
- *Pros*: Aligns with libSQL's native async API.
- *Cons*: Forces asynchronous scheduling overhead onto local SQLite WAL queries; introduces massive churn across MCP handlers, CLI commands, and lifecycle hooks; complicates embedded and WASM targets.

### Alternative 2: Extract to an Out-of-Tree Crate (`engram-storage-turso`)
Move `src/storage/turso_backend/` into a separate external crate.
- *Pros*: Isolates dependencies and eliminates C-ABI linker collisions in `engram-core`.
- *Cons*: Still suffers from the synchronous trait impedance mismatch unless the external crate defines its own distinct async API; requires ongoing community maintenance for an experimental backend.

### Alternative 3 (Chosen): Formal Deprecation & Scheduled Sunset (v0.24.0)
Formally deprecate the in-tree `turso` feature in v0.22.x, freeze its schema, and schedule its complete removal in v0.24.0 in favor of SQLite WAL + S3/R2 Snapshot Attestation.
- *Pros*: Eliminates code rot and C-ABI hazards; simplifies build matrices and CI parallelization; focuses engineering resources on local-first SQLite performance and snapshot sync.
- *Cons*: Users relying on Turso edge replicas must migrate to local SQLite WAL with S3/R2 backup or community-maintained external storage shims.

## Decision

1. **Formal Deprecation (v0.22.x)**:
   - Mark the `turso` Cargo feature and `TursoBackend` struct as **Deprecated**.
   - Freeze schema additions in `src/storage/turso_backend/`.
   - Document migration path to SQLite WAL + S3/R2 Cloud Sync.

2. **Sunset Timeline**:
   - **v0.22.x (Current)**: Announce deprecation via this ADR, documentation updates, and `#[deprecated]` attributes.
   - **v0.23.0**: Gate remaining code behind explicit `deprecated-turso` feature flag.
   - **v0.24.0**: Completely remove `src/storage/turso_backend/`, `tests/turso_backend_tests.rs`, and the `libsql` dependency from `Cargo.toml`.

3. **Canonical Architecture Confirmation**:
   - **Local SQLite with Write-Ahead Logging (WAL)** remains the single, primary, and canonical storage engine for Engram.
   - Cloud synchronization and multi-machine persistence are provided exclusively via **S3 / R2 Signed Snapshot Attestation**.

## Consequences

### Positive
- **Guaranteed Stability**: Eliminates runtime panic vectors caused by nested Tokio runtimes and `block_in_place`.
- **Clean Toolchain & Build**: Removes C-ABI collisions and reduces dependency footprint.
- **Unified Schema Evolution**: All storage migrations remain strictly centralized in `src/storage/migrations.rs`.
- **Architectural Clarity**: Reinforces Engram's identity as a blazing-fast, local-first persistent memory engine.

### Negative / Migration Impact
- Teams currently experimenting with Turso/libSQL edge replicas must transition to SQLite WAL with background snapshot push/pull.
