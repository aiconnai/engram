# RFC 0009: MCP Contract Validation JSON Report

Status: proposed

Date: 2026-06-09

Related issue: ENGRA-110

## Summary

Add an offline MCP contract validator that emits the common harness JSON
envelope. The first implementation should be a local script, not a new MCP tool.
Its job is to catch drift between tool definitions, handler dispatch, generated
docs, annotations, and degraded-mode expectations before MCP-related changes
land.

## Problem

Engram exposes a large MCP surface. Today we have useful tests and a generated
reference, but agents still need one deterministic command that answers:

- Does `tools/list` advertise tools that `tools/call` can dispatch?
- Are required schema fields present and parseable?
- Are read-only tools annotated with `readOnlyHint`?
- Are mutating/destructive tools classified consistently?
- Is `docs/MCP_TOOLS.md` in sync with the registry?
- Are optional feature surfaces reported as optional/degraded instead of silent
  failures?

The validator must be local, read-only, deterministic, and safe to run in CI or
agent loops without network access.

## Decision

Implement the first surface as:

```bash
python3 scripts/validate_mcp_contract.py --json
```

This script should read repository files only. It should not start an MCP server,
open a database, call provider APIs, or mutate generated docs. A future MCP
read-only tool may wrap the same logic after the offline contract is stable, but
that is not part of the first implementation.

## Validation Scope

The validator should produce checks for these categories.

### Tool Registry

- Parse `src/mcp/tools/registry.rs` using the same assumptions as
  `scripts/generate_mcp_reference.py`.
- Confirm every `ToolDef` has:
  - non-empty `name`
  - non-empty `description`
  - parseable JSON schema
  - `ToolTier`
  - `ToolAnnotations`
- Confirm tool names are unique.

### `tools/list` Shape

- Confirm generated tool definitions include `name`, `description`,
  `inputSchema`, and `annotations`.
- Confirm selected critical tools are present in the default compiled surface:
  - `memory_create`
  - `memory_get`
  - `memory_search`
  - `memory_list`
  - `context_search`
  - `context_build_bundle`
  - `harness_status`

### Required Schema Fields

- Confirm every schema is a JSON object with `type: "object"` when parameters
  are object-shaped.
- Confirm `required` is an array when present.
- Confirm every field listed in `required` exists in `properties`.
- For selected critical tools, assert the expected required fields:
  - `memory_create`: content-bearing input must remain required by the schema
    used by the current registry.
  - `memory_get`: memory identifier must remain required.
  - `context_search`: query input must remain required.
  - `context_build_bundle`: request scope or equivalent selector must remain
    explicit.

The exact field names should be read from the current schema during
implementation and then locked in focused tests. If current schemas use legacy
aliases, the validator may allow documented aliases but must report them.

### Annotation Consistency

- Read-only tools must have `readOnlyHint: true`.
- Destructive tools must have `destructiveHint: true`.
- Idempotent maintenance tools should have `idempotentHint: true`.
- Mutating tools must not be accidentally classified as read-only.

Initial critical annotation checks:

- read-only: `memory_get`, `memory_list`, `memory_search`, `context_search`,
  `context_build_bundle`, `harness_status`
- destructive: `memory_delete`, `memory_cleanup_expired`,
  `embedding_cache_clear`
- idempotent: `lifecycle_run`, `retention_policy_apply`,
  `memory_rebuild_embeddings`

### Handler Dispatch

- Parse string-literal match arms in `src/mcp/handlers/mod.rs`.
- For every registry tool that is available in the default feature set, confirm a
  dispatch arm exists.
- For feature-gated tools, report `missing_optional` when the registry documents
  the tool but the current build intentionally excludes it.
- Unknown dispatch arms that are not in the registry should report `invalid`
  unless explicitly documented as compatibility aliases.

Compatibility aliases currently allowed:

- `memory_seed` dispatching through `context_seed`

### Docs Coverage

- Confirm `docs/MCP_TOOLS.md` exists.
- Confirm it carries the generated-file marker.
- Confirm every default-surface tool has a docs entry.
- Confirm stale docs are caught by the existing generator check:

```bash
./scripts/generate-mcp-reference.sh --check
```

The validator may either call the generator in `--check` mode or reuse its parser
and compare generated markdown in memory. It must not rewrite
`docs/MCP_TOOLS.md`.

### Degraded-Mode Warnings

Optional compiled features should not fail the validator when absent. They should
produce structured degraded-mode checks when relevant:

- `missing_optional`: optional tool or feature surface absent by build feature.
- `degraded`: required surface exists, but optional metadata or docs are
  incomplete.
- `unavailable`: validator could not inspect a source file or generated docs.

Required surfaces must still fail with `invalid`.

## Degraded-Mode Vocabulary

Use this vocabulary inside check objects and top-level `degraded_mode`:

| Value | Meaning | Top-level status |
|-------|---------|------------------|
| `ok` | Required and optional validation passed. | `pass` |
| `missing_optional` | Optional feature surface is absent as expected. | `warn` |
| `degraded` | Required validation passed, but optional docs or metadata are incomplete. | `warn` |
| `invalid` | Required contract is broken. | `fail` |
| `unavailable` | Validator could not inspect a required source artifact. | `fail` |

## JSON Envelope

The validator must use `docs/harness/JSON_OUTPUTS.md`.

Example:

```json
{
  "schema_version": "harness-json-v1",
  "tool": "mcp_contract_validator",
  "mode": "offline",
  "status": "pass",
  "exit_code": 0,
  "timestamp": "2026-06-09T15:45:00Z",
  "summary": "MCP contract validation passed",
  "warnings": [],
  "failures": [],
  "checks": [
    {
      "id": "mcp_registry:unique_names",
      "status": "pass",
      "message": "all tool names are unique",
      "path": "src/mcp/tools/registry.rs"
    }
  ],
  "artifacts": [
    {
      "path": "docs/MCP_TOOLS.md",
      "kind": "mcp_reference",
      "format": "markdown"
    }
  ],
  "degraded_mode": "ok",
  "counts": {
    "tools": 0,
    "checks": 1,
    "warnings": 0,
    "failures": 0
  }
}
```

Check id families:

- `mcp_registry`
- `mcp_tools_list`
- `mcp_schema`
- `mcp_annotation`
- `mcp_dispatch`
- `mcp_docs`
- `mcp_degraded_mode`

Exit-code mapping:

- `pass` -> `0`
- `warn` -> `0`
- `fail` -> `1`
- usage/setup errors -> `2`

## Initial Implementation Plan

1. Add `scripts/validate_mcp_contract.py`.
2. Reuse or import parser logic from `scripts/generate_mcp_reference.py` where
   possible.
3. Add focused tests under `scripts/` for parser and negative fixture behavior.
4. Wire docs to mention the command, but do not add it to full `make ci` until
   runtime and stability are observed.
5. Keep `cargo test --test mcp_protocol_tests` as the runtime protocol safety
   net; the validator is a static contract check, not a replacement.

## Non-Goals

- No network calls.
- No provider credentials.
- No new MCP tool in the first implementation.
- No database startup.
- No generated-doc rewrites in validation mode.
- No replacement for existing MCP protocol tests.

## Open Questions

- Whether the script should share code with `generate_mcp_reference.py` directly
  or keep a small duplicated parser to avoid coupling two commands too tightly.
- Whether feature-gated tool availability should be inferred statically from
  `tool_feature_available` or supplied by a fixture generated from a default
  build.
- Whether alias handling should live in the validator script or in a small
  versioned metadata file.
