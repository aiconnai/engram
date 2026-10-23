PASS: the Turso/libSQL removal is complete and consistent with the ADR, and I found no substantive defects. This is an independent opinion only, not a receipt or merge authorization.

REVIEW_VERDICT: PASS Turso/libSQL sunset matches the ADR; no blocking defects found; sensor outcome unavailable when read, and cargo-deny attribution and the 23-failure equality are pending

## Scope and method
I read these in the worktree and `/tmp` packet:
- the trusted policy
- the accepted ADR
- the migration doc
- the non-deleted parts of `integrated.patch`
- the deleted `tests/turso_backend_tests.rs` hunk
- surrounding source for the claims the docs make

I did not read the other deleted `turso_backend/*.rs` bodies line by line. I did not verify the patch sha256 and ran no commands. All sensor, test, audit and deny results come from the integrator and are unverified by me.

## Findings
- [LOW] No issues found. Checked for bugs, regressions, security, tests for changed behavior, documentation, MCP protocol stability, SDK contracts, and scoped project instructions.

## What I checked

- **Cargo removal.** `turso`, the `full` membership and the optional `libsql` dependency are all gone from `Cargo.toml`.
  - The `src/storage/mod.rs` `cfg(feature = "turso")` module and re-export are removed.
  - Searches of `src/`, `tests/`, `scripts/`, `.github` and `*.toml` find no remaining `turso`/`libsql` reference. The exceptions are the intentional comments and the unrelated `libsqlite3-sys`.
- **Version churn.** The `Cargo.lock` diff has only removed `name`/`version` lines and no added ones. Removed packages include libsql*, tonic 0.11, h2 0.3.27, hyper 0.14, rustls 0.22.4, rustls-webpki 0.102.8, bincode 1.3.3 and rustls-pemfile.
  - The surviving lock still has h2 0.4.16, hyper 1.8.1, rustls 0.23.46, rustls-webpki 0.103.15, tonic 0.12.3 and prost 0.13.5. No other dependency was bumped.
- **Exception scope.** Every removed exception (0049, 0098, 0099, 0104, 0346, 0349, 0258, 2025-0141, 2025-0134) names only versions that no longer appear in the lock.
  - Retained: the `paste` ignore (RUSTSEC-2024-0436, `cargo-audit:ignore` only) and the `ttf-parser` allowed warning (RUSTSEC-2026-0192).
  - `.cargo/audit.toml` keeps only 2024-0436 and `deny.toml` has `ignore = []`, which matches what `scripts/check-security-exceptions.py` requires of the ledger.
- **Generic coverage.** The Turso-only tests were `test_turso_in_memory`, health check, CRUD, `with_transaction` unsupported, savepoint name validation and sync unsupported.
  - SQLite already has `test_crud_operations`, `test_savepoint_rejects_invalid_names_before_sql` (same five names), `test_with_transaction_reports_unsupported_wrapper_when_called`, and the health tests in `src/storage/sqlite_backend/health/tests.rs`.
  - `src/storage/sqlite_backend/mod.rs` now also asserts `!called` on the rejected closure. The Turso-specific sync-unsupported assertions are rightly dropped, since SQLite implements `sync_delta` and `sync_state`.
- **Schema and behavior.** No migrations, `SCHEMA_VERSION`, MCP or SDK files are in the diff. The only SQLite source change is the test hardening.
- **CI and inventory.** `scripts/ci-features.env` and `docs/contracts/advertised-surfaces.toml` drop only `turso`.
- **Migration doc.** The names it relies on exist in source:
  - `SqliteBackend::new(StorageConfig)` and `Storage::open`
  - `Storage::with_transaction`, `snapshot_copy` and `vacuum_into`
  - `CloudStorage::upload_snapshot`
  - the `ENGRAM_STORAGE_URI`, `ENGRAM_CLOUD_ENCRYPT` and `ENGRAM_CLOUD_ENCRYPTION_KEY(_FILE)` env vars
  - cloud URIs that accept only `s3://` (`src/sync/cloud.rs:42`)
  - The sync worker is cfg-gated on `cloud` and starts with `auto_sync: true`, so the "build with `cloud`" step is accurate.
- **Migration safety.** The doc says the transfer is non-lossless and that the new database must be separate. It forbids raw-copying a live DB, and it makes no replica-compatibility claim.

## Limitations and observations that are not findings
1. **Sensors.** When I read `sensors-full.log` it had reached `==> [harness] offline_lane` and stopped there, with no terminal status. It had passed fmt, clippy, test_lib, test_integration, test_integration_watch, wasm and doc, but I read no result lines, so I can't confirm they passed. I can't confirm full-gate success from this log.
2. **Cargo-deny and the 23 failures.** Neither the cargo-deny baseline attribution nor the 23-failure base/candidate equality is verifiable from my side. The lock diff is removal-only, so the diff itself can't have introduced a new vulnerable crate version. Confirm that the findings in surviving crates exist at base.
3. **External consumers.** Only 17 top-level sibling manifests were checked, and the progress doc says so. The breaking change is documented in `CHANGELOG.md`, the migration doc, `README.md` and `docs/ARCHITECTURE.md`.
4. **Pre-existing and out of scope, optional.**
   - `docs/quality/rust-risk-inventory.md` and `docs/security/security-gate-evidence.md` still reference the Turso stack. They are dated historical records, so leaving them is defensible. `docs/security/security-gate-evidence.md` is the only one that names the retired exceptions.
   - `fuzz/Cargo.lock` only contains `libsqlite3-sys`.
5. **Harness files.** `docs/harness/SPEC.md` and `progress.md` were switched to the new task. I did not treat them as review authority, as instructed. They look consistent with the active-plan pointer and the canvas file that the policy requires for storage changes.

REVIEW_VERDICT: PASS Turso/libSQL sunset matches the ADR; no blocking defects found; sensor outcome unavailable when read, and cargo-deny attribution and the 23-failure equality are pending
