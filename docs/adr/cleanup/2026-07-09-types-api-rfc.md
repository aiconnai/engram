---
adr: ADR-TYPES-20260709-1
track: public-api-safe refactor
service: engram-core
status: Implemented
owner: Codex
created: 2026-07-09
---

# Types API Refactor RFC (Row 12)

## Context
Row 12 in `docs/adr/cleanup/2026-07-08-remaining-oversized-files-inspect.md` explicitly defers `src/types.rs` because it is the global public API surface. A facade-only split is required to keep compatibility.

## Baseline public API inventory

- Baseline snapshot collected with `cargo public-api -p engram-core` and stored in `docs/api/types-baseline.txt`.
- Key exported API items defined in `src/types.rs` (as of this baseline):
  - `MemoryId`
  - `Memory`, `CrossReference`, `MemoryVersion`
  - `WorkspaceStats`, `StorageStats`, `CompactOp`, `CompactReport`, `RebuildReport`
  - `SyncStatus`, `EmbeddingStatus`
  - `StorageConfig`, `EmbeddingConfig`
  - `CreateMemoryInput`, `UpdateMemoryInput`, `CreateCrossRefInput`
  - `ListOptions`, `SearchOptions`, `SearchResult`, `MatchInfo`, `SearchStrategy`
  - `WorkspaceError`, `normalize_workspace`
  - `MAX_WORKSPACE_LENGTH`, `RESERVED_WORKSPACES`
  - `MemoryType`, `MemoryTier`, `Visibility`, `MemoryScope`, `LifecycleState`
  - `EdgeType`, `RelationSource`
  - `StorageMode`, `DedupMode`, `SortField`, `SortOrder`, `EmbeddingState`

## API stability classification

### 1.0-stable (contract-safe)
- DTOs and enums heavily used by SDK-facing flows and storage/search/mcp: all current items above are considered part of public contract and must remain semver-stable in name/module shape.
- All top-level `pub` items listed above should remain available at `engram::...` exactly.

### Internal (crate-private)
- No current public contract candidates were identified as internal-only while keeping `pub` in this file.
- For future growth, any non-contract helper moved out of this surface should go in `types::internal` and stay un-exported.

### Unstable / transitional
- None proposed for initial move. If we need migration helpers, keep them in non-re-exported internal submodule first.

## Physical organization (as built)

`src/types.rs` is the module facade: private submodule declarations plus
`pub use` re-exports only — no structural/business logic or additional public items.

- `src/types.rs` (facade; `mod` declarations + `pub use` re-exports)
- `src/types/`
  - `core.rs` (`MemoryId`, `Memory`, `LifecycleState`, `normalize_workspace`, `WorkspaceError`, `MAX_WORKSPACE_LENGTH`, `RESERVED_WORKSPACES`)
  - `memory.rs` (`MemoryType`, `MemoryTier`, `MemoryScope`, `Visibility`, `CrossReference`, `EdgeType`, `RelationSource`)
  - `stats.rs` (`WorkspaceStats`, `StorageStats`, `CompactOp`, `CompactReport`, `RebuildReport`, `SyncStatus`, `EmbeddingStatus`, `EmbeddingState`, `MemoryVersion`)
  - `config.rs` (`StorageConfig`, `EmbeddingConfig`, `StorageMode`, `DedupMode`, `CreateMemoryInput`, `UpdateMemoryInput`, `CreateCrossRefInput`, `ListOptions`)
  - `search.rs` (`SearchStrategy`, `SortField`, `SortOrder`, `SearchOptions`, `SearchResult`, `MatchInfo`)

Notes vs. the original proposal: no separate `mod.rs` (the facade lives at
`src/types.rs`), and the planned `inputs.rs`/`version.rs` were folded into
`config.rs` and `stats.rs` respectively to keep files cohesive and under size limits.

### Re-export mapping

- `src/types.rs` re-exports every public item from its submodules verbatim,
  so all paths `engram::types::X` (and `engram::X` via `pub use types::*`
  in `src/lib.rs`) are unchanged.

## Refactoring strategy

1. Create `src/types/` directory with the files above.
2. Move declarations verbatim from `src/types.rs` in logical groups (pure file moves + imports updates only).
3. Preserve derives, serde attrs, defaults, impl blocks, and method bodies.
4. Keep all `pub` names in the same paths through `types.rs` re-exports.
5. Run targeted checks:
   - `rtk cargo check --workspace --all-targets --locked`
   - `rtk cargo test --workspace --locked`
6. Compile-check dependency surfaces:
   - `cd engram-wasm && rtk cargo check --locked`
   - `cd sdks/python && rtk cargo check --manifest-path ../Cargo.toml --locked`
   - `cd sdks/typescript && rtk npm run -s build --if-present`
   
   *(Python/TypeScript SDK commands are repository-dependent; use closest available build/check command when available.)*

## Compatibility and versioning policy

- No API breaks: all identifiers from the public list above must remain reachable from `engram::...`.
- Any new public DTO introduced in this move must be documented as either unstable (internal module only) or 1.0-stable based on usage.
- Update `CONTRIBUTING.md` with a short policy section stating the 1.0 stability commitment and semver behavior for exported DTOs in `types.rs`.

## Rollout plan

1) Post plan to review. ✅
2) Implement mechanical split only. ✅
3) Run gates. ✅
4) Diff audit for public API equivalence using stored baseline in `docs/api/types-baseline.txt`. ✅
   Baseline captured post-split with `cargo public-api -p engram-core --simplified`
   (19,908 public items); future refactors must diff against this file.
