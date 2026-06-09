# RFC 0010: Permission Modes for MCP and Harness Operations

Status: proposed

Date: 2026-06-09

Related issue: ENGRA-108

## Summary

Define Engram permission modes for MCP tools and operational commands:

- `read_only`
- `scoped_write`
- `maintenance`
- `admin`

The first implementation must not break current local-first workflows. Runtime
denial should be introduced only behind an explicit mode configuration after this
contract is reviewed.

## Goals

- Give agents a stable way to reason about which tools can mutate memory,
  delete data, run maintenance, or change access.
- Classify high-risk mutating surfaces before enforcement.
- Preserve current local developer behavior unless a caller explicitly requests a
  restrictive mode.
- Align with MCP annotations such as `readOnlyHint`, `destructiveHint`, and
  `idempotentHint`.
- Provide a structured denial shape for future enforcement.

## Non-Goals

- No auth system redesign.
- No tenant or workspace ACL implementation in this RFC.
- No default behavior change for existing MCP clients.
- No new network dependency.
- No claim that MCP annotations alone are authorization.

## Modes

### `read_only`

Allows inspection only. A `read_only` caller may list, search, retrieve, inspect
status, and run dry-run or preview operations that do not persist state.

Examples:

- `memory_get`
- `memory_list`
- `memory_search`
- `context_search`
- `context_build_bundle`
- `harness_status`
- `memory_garden_preview`
- `sync_status` / `memory_sync_status`

### `scoped_write`

Allows ordinary writes inside the caller's active workspace, scope, or session.
It does not allow destructive deletes, global maintenance, access changes, or
cross-agent administration.

Examples:

- `memory_create`
- `memory_update`
- `context_seed`
- `memory_link`
- `memory_unlink`
- `identity_create`
- `identity_update`
- `identity_add_alias`
- `session_index`
- `harness_record`
- `harness_verify`

### `maintenance`

Allows deterministic maintenance jobs that may rewrite derived state, rebuild
indexes, run lifecycle policies, or perform bulk cleanup according to existing
policy. Maintenance may be idempotent or destructive, but it is operationally
broader than a normal scoped write.

Examples:

- `lifecycle_run`
- `retention_policy_apply`
- `memory_rebuild_embeddings`
- `memory_rebuild_crossrefs`
- `meilisearch_reindex`
- `memory_embedding_migrate`
- `sync_cleanup`
- `memory_cleanup_expired`
- `pending_injections_cleanup`

### `admin`

Allows high-risk destructive operations, access-control changes, workspace-level
deletion, agent registration changes, and global configuration writes.

Examples:

- `memory_delete`
- `memory_delete_batch`
- `workspace_delete`
- `identity_delete`
- `session_delete`
- `retention_policy_delete`
- `memory_revoke_access`
- `memory_grant_access`
- `agent_deregister`
- `agent_register`
- `embedding_cache_clear`

## Classification Matrix

| Surface | Minimum mode | Rationale |
|---------|--------------|-----------|
| `readOnlyHint` tools | `read_only` | Must not mutate persisted state. |
| ordinary `mutating` memory writes | `scoped_write` | Writes scoped memory/session data. |
| identity create/update/link | `scoped_write` | Mutates scoped identity graph, but not access policy. |
| harness evidence writes | `scoped_write` | Records local operational evidence. |
| idempotent rebuilds/reindexing | `maintenance` | May be safe to rerun but can rewrite derived state. |
| lifecycle/retention apply | `maintenance` | Bulk policy execution over existing data. |
| destructive deletes | `admin` | Removes or irreversibly changes data. |
| access grant/revoke | `admin` | Changes authorization boundary. |
| workspace delete | `admin` | Broad destructive scope. |
| agent register/deregister | `admin` | Changes actor registry and trust boundary. |

## Harness Command Classification

| Command | Minimum mode | Notes |
|---------|--------------|-------|
| `bootstrap.sh` | `read_only` | Must stay fast and read-only. |
| `doctor.sh` | `read_only` | Validates harness consistency only. |
| `doctor.sh --json` | `read_only` | Same validation, parseable output. |
| `sensors.sh status --json` | `read_only` | Reads `.sensors-last`; does not run the gate. |
| `sensors.sh` full/default | `maintenance` | Runs CI and doctor; may write `.sensors-last`. |
| `baseline.sh` | `maintenance` | Writes `.baseline-last` evidence. |
| `quarterly-audit.sh` | `maintenance` | Writes audit evidence only. |
| `review-gate.sh pre/post` | `maintenance` | Writes review artifacts and may block handoff. |

## Runtime Contract

Future enforcement should compute an effective mode from explicit configuration,
for example:

- process environment: `ENGRAM_PERMISSION_MODE`
- MCP session metadata
- per-request override for local tools

Compatibility rule:

- If no mode is explicitly configured, existing local MCP behavior remains
  unchanged.
- Restrictive behavior starts only when an explicit mode is configured.
- `read_only` must fail closed for mutating, maintenance, and admin tools.
- `scoped_write` must deny maintenance and admin tools.
- `maintenance` must deny admin tools unless explicitly elevated.
- `admin` allows every existing surface, subject to future auth and audit
  policies.

## Denial Shape

Future denial responses should be structured and stable:

```json
{
  "error": {
    "code": "permission_denied",
    "tool": "memory_delete",
    "current_mode": "read_only",
    "required_mode": "admin",
    "message": "memory_delete requires admin mode",
    "audit_id": null
  }
}
```

Rules:

- Denials must not expose secrets or raw request payloads.
- Denials should include the tool name, current mode, required mode, and a short
  message.
- Denials should be testable without network access.
- Future audit IDs are optional until audit persistence is implemented.

## Audit Expectations

When enforcement is added, these events should be auditable:

- denied tool call
- allowed admin call
- allowed maintenance call
- permission mode override
- mode source, for example env or session metadata

Audit entries should include:

- timestamp
- tool
- current mode
- required mode
- decision: `allow` or `deny`
- reason code
- workspace/scope when available

Audit entries must not store secrets, auth headers, cookies, or full raw
payloads.

## First Implementation Boundary

The first implementation may add only low-risk metadata or opt-in guard behavior.
Acceptable first steps:

- expose a static classifier function for tool -> required mode;
- expose a JSON report listing critical tool classifications;
- add tests proving classifier coverage for selected high-risk tools;
- add an opt-in `read_only` deny test for one clearly administrative tool.

Do not add broad enforcement until review confirms:

- default behavior remains compatible;
- denial shape is stable;
- tests cover allow and deny cases;
- docs make the explicit mode source clear.

## Open Questions

- Should `memory_cleanup_expired` be `maintenance` or `admin` by default? It is
  destructive, but policy-driven and operational.
- Should `harness_record` be `scoped_write` or `maintenance` when it writes
  verification evidence?
- Should permission mode metadata live in `src/mcp/tools/mod.rs`, a separate
  policy module, or generated metadata consumed by validators?
- Should future audit entries live in storage, harness artifacts, or both?
