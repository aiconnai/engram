# Migrating from Turso/libSQL after v0.24.0

Engram v0.24.0 removes the experimental in-tree Turso/libSQL backend, as decided
in [ADR-STORAGE-20260816-1](../adr/2026-08-16-turso-backend-deprecation-and-sunset.md).
SQLite WAL is the canonical operational store. Optional S3/R2 backup sync uses
the existing `cloud` feature.

## Compatibility impact

- Remove `turso` from Cargo dependency feature lists and build/test commands.
  `--features turso` no longer resolves; there is no `deprecated-turso` replacement.
- Rust imports of `engram::storage::{TursoBackend, TursoConfig}` and paths under
  `engram::storage::turso_backend` no longer compile. Replace backend construction
  with `engram::storage::SqliteBackend::new(StorageConfig)` when using
  `StorageBackend`, or `engram::storage::Storage::open(StorageConfig)` for the
  concrete SQLite API. `StorageConfig` is in `engram::types`.
- These constructors are synchronous. Review former async constructor/sync calls
  and transaction requirements; changing type names alone is insufficient.
  `Storage::with_transaction` is the concrete SQLite transaction API.
- Turso URLs, auth tokens, embedded replicas, and edge-replication semantics have
  no direct replacement in the local SQLite configuration. S3/R2 sync is a backup
  and synchronization path, not a Turso edge-replica service.

## Move existing data deliberately

The old Turso backend maintained its own schema initialization/migration runner,
separate from the canonical SQLite migrations. A Turso/libSQL database or local
replica is **not guaranteed to be compatible** with `Storage::open`, even if it
has SQLite format or records an Engram schema version. There is no automatic
Turso-to-Engram database conversion command.

1. Keep the original deployment and a verified backup until migration is validated.
   Quiesce writers and use the source system's supported export/backup mechanism.
2. Initialize a **new, separate** Engram SQLite database with v0.24.0. Do not point
   it at the old replica and rely on automatic migrations to repair the schema.
3. Build a logical transfer for the data you use, mapping source fields to current
   memory inputs. Validate workspace/scope, tags, metadata, tiers/expiration, and
   references. Re-map relationships if newly assigned memory IDs differ.
4. Compare counts and representative records, run search and relationship checks,
   and test backup/restore on the destination before cutting over. Preserve any
   source data the transfer does not cover.

The built-in generic JSON export/import is a partial memory transfer, not a
lossless full-store migration. Imports create new rows with newly allocated IDs
and creation/update timestamps, rather than restoring the exported values. The
format does not reconstruct the full identity, graph, version history, embedding,
or temporal state; plan and validate any required transfer of that state
separately. See the [export/import implementation](../../src/storage/queries/export.rs).

Never raw-copy or upload a live SQLite database or its `-wal`/`-shm` files from
inside a process that has it open. For an open Engram `Storage`, use
`Storage::snapshot_copy` / `Storage::vacuum_into`; with the `cloud` feature,
`CloudStorage::upload_snapshot` creates a consistent SQLite snapshot before upload.
Do not replace an open destination database. See [invariant 27](../../INVARIANTS.md).

## Start the SQLite destination

Point an installed v0.24.0 server at the new destination path:

```bash
engram-server --db-path /absolute/path/to/new-engram.db --transport stdio
```

`ENGRAM_DB_PATH` is the environment equivalent. Local SQLite needs no additional
backend feature or cloud credentials. For a separate CLI inspection after stopping
the server, use `engram-cli --db-path /absolute/path/to/new-engram.db stats`.

## Add optional S3/R2 backup sync

Build with the existing `cloud` feature if your binary does not include it:

```bash
cargo build --release --features cloud --bin engram-server
```

Configure AWS SDK credentials for the destination account, then start the built
server with the new local database and a **new remote object key**:

```bash
export ENGRAM_DB_PATH=/absolute/path/to/new-engram.db
export ENGRAM_STORAGE_URI=s3://my-bucket/engram/new-memories.db
./target/release/engram-server --transport stdio
```

`--cloud-uri` is the CLI equivalent of `ENGRAM_STORAGE_URI`. R2 uses the same
`s3://bucket/key` URI with the AWS SDK configured for its endpoint and credentials
(`AWS_ENDPOINT_URL`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION`).
A `libsql://` or `r2://` URI is not accepted by this sync implementation.

For encrypted cloud backups, set `ENGRAM_CLOUD_ENCRYPT=true` and configure exactly
one of `ENGRAM_CLOUD_ENCRYPTION_KEY` or `ENGRAM_CLOUD_ENCRYPTION_KEY_FILE`. The
value/file must contain a hex- or base64-encoded 32-byte key; keep the key available
for restore. Enabling encryption without a key fails configuration.

Library callers must wire cloud synchronization explicitly; constructing
`SqliteBackend` does not start a sync worker. Portable signed snapshots and
attestation use the separate `snapshot` / `attestation` features (combined by
`agent-portability`); the `cloud` feature alone does not enable signing.
