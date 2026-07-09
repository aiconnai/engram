# Session Handoff Context Rotation Design

| Field | Value |
|-------|-------|
| Date | 2026-07-08 |
| Status | Approved design; implementation not started |
| Product area | MCP session continuity, Operational Context, CLI UX |
| Primary user | Advanced users who understand when to rotate AI sessions |
| Future user | Non-technical users who need automatic context-rotation guidance |

## Problem

Long AI sessions degrade as the active context window fills. Users who know this
problem can manually open a fresh session, but they still need an easy way to
carry over the useful state: current goal, what happened, what was decided, what
was verified, what remains risky, and what the next agent should do.

Engram already has the core primitives for this: session indexing, checkpoints,
Operational Context bundles, memory digests, and structured harness handoffs. The
missing product layer is a simple, memorable workflow for rotating sessions.

## Decision

Build a manual-first **Session Handoff** workflow that is MCP-first and later
exposed through the CLI.

The MVP should make one user action enough:

- in an MCP-capable agent: ask for a session handoff / session continuation;
- in the CLI: run `engram session handoff`.

Both surfaces must use the same shared handoff builder. The CLI is a wrapper, not
a second implementation.

The MVP should not attempt fully automatic session-rotation detection. It should
produce the same packet shape that a future automatic detector can trigger.

## Product Principles

1. **Continuation packet, not generic summary.** The output is written for the
   next AI session to act on, not for a human archive.
2. **One obvious action.** Advanced users should only need to remember: before
   opening a fresh session, run handoff.
3. **Evidence-aware.** Completion claims require verification evidence; missing
   verification is reported explicitly.
4. **Provenance-first.** The packet cites memory IDs, context event IDs, files,
   commands, or artifact pointers where available.
5. **Safe by default.** Raw transcripts, raw command output, secrets, and logs are
   not copied into the packet by default.
6. **Upgradeable to automatic.** The same builder should support future passive
   drafts and context-pressure warnings.

## Existing Surfaces To Reuse

- `session_land`: existing essential MCP entry point for landing a session and
  creating a checkpoint.
- `harness_handoff`: stricter structured handoff surface for goal, decisions,
  verification, risks, blockers, and next steps.
- `context_build_bundle`: compact Operational Context for resuming work.
- `memory_digest`: actionable topic digest with source IDs and warnings.
- `memory_checkpoint`: checkpoint memory creation.
- `session_index` and `session_index_delta`: session transcript indexing when a
  client has messages to provide.

The implementation should avoid a parallel handoff system. If a friendlier MCP
name or prompt is added for discoverability, it must delegate to the same builder
used by `session_land`.

## User Experience

### MCP Flow

The user or agent asks for a handoff in natural language, for example:

```text
Prepare a session handoff so I can continue in a new AI session.
```

The MCP surface should collect or infer the minimum inputs. `session_id` is
preferred but not required for the one-action UX:

```json
{
  "session_id": "optional-current-session-id",
  "workspace": "default",
  "summary": "optional human summary",
  "next_session_hints": ["optional next step"]
}
```

If richer explicit fields are available from the user, agent, or host, the
builder should accept them and prefer them over inferred summaries:

```json
{
  "current_goal": "Ship the session handoff MVP design",
  "files_touched": ["docs/superpowers/specs/..."],
  "decisions_made": ["MCP first; CLI wraps the same builder"],
  "tests_run": ["docs/spec self-review"],
  "tests_not_run": ["cargo test, because this is docs-only design"],
  "known_risks": ["automatic rotation threshold remains future work"],
  "blockers": [],
  "next_steps": ["Implement shared handoff builder", "Wire CLI wrapper"]
}
```

The response must include a copy-ready Markdown block:

```markdown
# Continue this work in a new AI session

## Essential context
...

## Current goal
...

## What changed
...

## Decisions
...

## Verification
...

## Risks and blockers
...

## Next steps
1. ...
2. ...

## Source references
- Memory IDs: ...
- Context event IDs: ...
- Files: ...
```

### CLI Flow

The CLI command should call the same builder and print the copy-ready block:

```bash
engram session handoff
```

Useful options:

```bash
engram session handoff --session <session-id>
engram session handoff --workspace <workspace>
engram session handoff --summary "..."
engram session handoff --next "..."
engram session handoff --no-persist
engram session handoff --json
```

Default CLI output is human-copyable Markdown. `--json` returns the structured
packet for automation. `persist` defaults to `true`; `--no-persist` disables
checkpoint creation.

When `--session` is omitted, the CLI should use the most recent session in the
selected workspace. If no session exists, it may still produce a workspace-level
handoff from recent memories and Operational Context, but it must include a
warning that no concrete session was resolved.

## Surface Ownership

`session_land`, `harness_handoff`, any friendlier MCP alias, and the CLI command
must not grow separate handoff assembly logic.

The shared builder owns normalization, section assembly, warning generation,
copy-block rendering, and checkpoint persistence. `session_land` remains the
simple session-continuation surface. `harness_handoff` remains the stricter
explicit surface for curated engineering handoffs, but it should migrate to the
same builder for rendering and persistence so that decisions, verification,
risks, blockers, and next steps have one product contract.

## Shared Builder Contract

Introduce or extract a shared internal builder, conceptually named
`SessionHandoffBuilder`, with one responsibility: assemble a session continuation
packet from existing memory and context surfaces.

Suggested input shape:

```rust
pub struct SessionHandoffRequest {
    pub session_id: Option<String>,
    pub workspace: Option<String>,
    pub summary: Option<String>,
    pub current_goal: Option<String>,
    pub next_session_hints: Vec<String>,
    pub files_touched: Vec<String>,
    pub decisions_made: Vec<String>,
    pub tests_run: Vec<String>,
    pub tests_not_run: Vec<String>,
    pub known_risks: Vec<String>,
    pub blockers: Vec<String>,
    pub next_steps: Vec<String>,
    pub persist: bool,
    pub include_operational_context: bool,
    pub include_digest: bool,
}
```

Suggested output shape:

```rust
pub struct SessionHandoffPacket {
    pub session_id: Option<String>,
    pub workspace: String,
    pub created_at: String,
    pub summary: String,
    pub current_goal: Option<String>,
    pub open_items: Vec<HandoffItem>,
    pub decisions: Vec<HandoffItem>,
    pub verification: Vec<HandoffItem>,
    pub risks: Vec<HandoffItem>,
    pub blockers: Vec<HandoffItem>,
    pub next_steps: Vec<String>,
    pub source_memory_ids: Vec<i64>,
    pub source_context_event_ids: Vec<i64>,
    pub warnings: Vec<String>,
    pub checkpoint_id: Option<i64>,
    pub copy_block: String,
}
```

The exact Rust type names may follow existing module conventions, but these
fields define the product contract. Memory IDs should use the actual storage ID
type used by the implementation; `i64` above reflects the current handler shape
and should be checked before coding.

## Data Flow

1. Validate the request. `workspace` defaults to `default`; empty optional
   strings are ignored.
2. Resolve the session. If `session_id` is provided, use it. If it is omitted,
   select the most recent session in the workspace via the same semantics as
   `session_list` and add a warning that fallback resolution was used. If no
   session exists, continue with workspace-level context and add a warning.
3. Read recent session, workspace, todo, issue, decision, verification, and
   Operational Context records.
4. Build a compact Operational Context section through `context_build_bundle`
   when enabled.
5. Build a topic digest through `memory_digest` when a goal or query is
   available.
6. Merge sections into one deterministic packet. Prefer explicit user-provided
   fields over inferred summaries.
7. Emit warnings for missing goal, missing next steps, missing verification,
   stale context, or omitted raw artifacts.
8. Persist a checkpoint memory when `persist=true`.
9. Return both structured JSON and a copy-ready Markdown block.

## Error Handling

- Missing `session_id` triggers fallback session resolution, not an immediate
  error. It becomes an error only when a caller explicitly requires a concrete
  session and no matching session can be found.
- Missing optional context is not fatal; it becomes a warning in the packet.
- Retrieval failures for optional sections degrade gracefully and add warnings.
- Checkpoint persistence failure returns the packet plus a persistence warning
  when the packet could still be assembled safely.
- The builder must not panic on malformed JSON, empty arrays, invalid optional
  fields, or absent memories.
- Errors must include operation context such as session ID and workspace.

## Safety And Privacy

- Do not include raw transcripts by default.
- Do not include raw command output or raw logs by default.
- Do not include environment dumps, credentials, tokens, cookies, or private
  keys.
- Artifact references are allowed; raw artifact content requires the existing
  explicit artifact retrieval path.
- Generated text must be labeled as a derived continuation packet, not source of
  truth.

For the MVP, "safe by default" is achieved by selection and existing policy
boundaries, not by a new full-secret scanner. The builder should render only
summaries, structured fields, metadata, memory previews, and artifact pointers;
it must not dereference raw artifact content or include raw transcripts/logs.
Operational Context rows should respect their existing redaction status and raw
artifact access policy. Memory content rendered into the packet should reuse the
existing private-content stripping behavior where applicable. If later work adds
active secret scrubbing, that is additive and must not be required for the manual
MVP to ship.

## Testing Strategy

Minimum implementation tests:

1. Unit test fallback session resolution: omitted `session_id` uses the most
   recent session in the workspace and emits a warning.
2. Unit test the shared builder with seeded memories for decisions, todos,
   verification records, risks, and next steps.
3. Unit test explicit rich fields such as files touched, decisions, verification,
   risks, blockers, and next steps overriding inferred text.
4. Unit test warning behavior for missing goal, missing verification, and empty
   optional sections.
5. MCP protocol test that calls the handoff surface and asserts the response has
   structured fields plus `copy_block`.
6. Persistence test that `persist=true` creates a checkpoint with the
   `session-handoff` tag.
7. CLI smoke test for `engram session handoff --json` using a test database,
   including the omitted-session fallback.
8. Generated MCP reference check if any MCP schema or description changes.

Docs-only design work does not require Rust tests. Implementation work must run
the relevant Rust, MCP reference, and CLI checks before completion.

## Non-Goals For The MVP

- Automatic context-window pressure detection.
- Background handoff drafts that update continuously.
- Opening or controlling the next AI session automatically.
- New storage schema unless implementation proves existing checkpoint/context
  storage cannot represent the packet.
- LLM-based summarization as a hard dependency.
- Raw transcript archival.

## Future Automatic Mode

After the manual MVP is stable, the same builder can support automatic UX:

1. Track context pressure or session length in clients that expose those signals.
2. Maintain a passive handoff draft using safe, summarized context.
3. Warn the user when rotation is recommended.
4. Offer one-click creation of the continuation packet.
5. Optionally hand the packet to the next session launcher where the host
   supports that integration.

The automatic mode should not change the packet contract. It should only change
when and how the builder is invoked.

## Acceptance Criteria For Implementation

- A user can generate a useful continuation packet through MCP with one obvious
  action.
- A user can generate the same packet through the CLI.
- MCP and CLI share the same internal builder.
- Omitting `session_id` falls back to the most recent session or a
  warning-marked workspace-level handoff.
- `harness_handoff` and `session_land` share the same builder contract instead
  of maintaining parallel rendering/persistence logic.
- The output includes a copy-ready Markdown block for the next AI session.
- Missing verification, stale context, and omitted raw artifacts are explicit.
- The packet can be persisted as a checkpoint.
- No raw logs, raw artifact content, or full transcripts are included by
  default; the MVP does not promise comprehensive active secret scrubbing beyond
  existing redaction/private-content mechanisms.
