# Decision Canvas: mcp-export-code-api

Date: 2026-06-26
Owner: Ronaldo (decisions) + agent (drafting)
Scope: Decide the shape of a code-mode API generator that materializes Engram's
MCP tool surface as typed callable wrappers, per the Anthropic "code execution
with MCP" pattern — generator only, no runtime/executor.

## Trigger

| Trigger | Evidence |
|---|---|
| MCP tool surface is large; loading all definitions per call is costly | `get_tool_definitions()` exposes ~306 tools (235 active under default features: essential 24, standard 110, advanced 101) |
| Anthropic pattern: expose MCP tools as code-callable APIs with progressive disclosure | https://www.anthropic.com/engineering/code-execution-with-mcp |
| Progressive discovery already half-built; `discover_tools.detail=schema` now emits full schemas | `feature/discover-tools-detail` (commit `aeacb93`), `src/mcp/handlers/misc.rs` |
| Generator can reuse the existing source of truth without a second registry | `get_tool_definitions()` at `src/mcp/tools/mod.rs:102` already serializes `input_schema` (`src/mcp/protocol.rs:233`) |

## Problem Statement

Agents (Claude Code, Codex, Gemini) that want to compose multi-step work over
Engram today must either (a) call `tools/call` one tool at a time through the
model context, or (b) read tool schemas into context before each use. For
bulk/compositional work (filter N memories, join results, loop with retries),
this spends tokens on intermediate results the model never needs to see.

The Anthropic pattern: generate a file-per-tool typed API so an agent writes a
small script importing only the tools it needs; the runtime executes locally and
only the final summary returns to the model context. This canvas decides the
**generator** — the executor is explicitly out of scope (see below).

## Decisions (locked with owner 2026-06-26)

| # | Question | Decision | Reason |
|---|---|---|---|
| 1 | Target language(s) initially | **TypeScript first**; Python as fast-follow, not in first release | Matches the Anthropic article; prove the generator on one language before duplicating |
| 2 | Versioned in-repo vs generated on demand | **Generated on demand, gitignored** (`.engram/mcp-api/`) + CI test that the generator runs and matches the registry | 306 committed wrappers guarantee drift per new tool; derived artifact like `docs/MCP_TOOLS.md` (regenerated, not hand-edited) |
| 3 | Anti-drift with `ToolDef`/registry | **(a) The Engram binary emits the catalog as JSON** (reusing `get_tool_definitions()`); the generator consumes that JSON | Reuses the existing source of truth; parsing `registry.rs` textually would be fragile; the JSON already carries `input_schema` |
| 4 | Delivery form | **CLI subcommand only** (`engram mcp export-code-api`), first release; not an MCP tool | Generating a file tree is a build/dev operation, not an in-loop agent action; keeps the harness boundary (Engram exposes tools; the client generates/executes code) |
| 5 | Execution scope | **Generator only.** The sandboxed runner is OUT OF SCOPE and deferred to a separate security ADR | Runner needs sandbox/resource limits/credential isolation; bundling it would hold the easy decision hostage to the hard one |

## Architecture (consequence of decisions)

```
engram-cli  (existing bin, Cargo.toml:56; clap 4.4 already a dep)
  └── subcommand: mcp export-code-api --lang typescript --out .engram/mcp-api
        │
        ├─ step 1: enumerate tools via get_tool_definitions()  ← source of truth
        │           (emit JSON catalog: name, description, tier, input_schema, annotations)
        │
        └─ step 2: render one .ts file per tool + an index + a thin client
                   .engram/mcp-api/
                   ├── client.ts          (callEngramTool → tools/call transport)
                   ├── <domain>/<tool>.ts  (typed input iface + wrapper fn)
                   └── <domain>/index.ts
```

Each generated wrapper is a thin typed shim over `tools/call`:

```ts
// .engram/mcp-api/memory/search.ts   (GENERATED — do not edit)
import { callEngramTool } from "../client";
export interface MemorySearchInput { query: string; limit?: number; workspace?: string; }
export async function memorySearch(input: MemorySearchInput) {
  return callEngramTool("memory_search", input);
}
```

Input interfaces are derived from each tool's JSON Schema. The generator does NOT
hand-maintain types — they fall out of the schema the binary already serializes.

## Relationship to existing SDKs (NOT a collision)

| Surface | `sdks/typescript` (`@engram/client`) | `export-code-api` (this canvas) |
|---|---|---|
| Audience | Humans building Cloud apps | Agents composing scripts over the MCP surface |
| Transport | HTTP REST to Engram Cloud | MCP `tools/call` |
| Coverage | Hand-curated high-level methods (`create`, `search`) | All ~306 tools, mechanically |
| Maintenance | Hand-written, 1028 lines | Generated, zero hand-maintenance |
| Drift risk | Manual (accepted) | None (regenerated from source) |

They are orthogonal: different transport, different audience, different coverage.
The generator does not replace, wrap, or depend on the hand-written SDK.

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| `get_tool_definitions()` enumeration | One-shot at generate time; not a request path | None at runtime | Already called by `tools/list`; no new hot path |
| Generated wrappers at agent runtime | One `tools/call` per invocation (same as today) | Agent imports only needed files | No added latency vs direct `tools/call`; token win is in NOT loading all schemas |

This generator touches **no request hot path**. It is a dev-time codegen step.

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Tool name not a valid TS identifier (e.g. has `__`) | Generator must map to a safe identifier; snapshot test on a known-odd name |
| Feature-gated tools (langfuse, meilisearch, etc.) absent under default build | Catalog reflects compiled features; generator emits only what the binary reports — document that `--features` affects output |
| Tool schema is `{}` or malformed | Wrapper falls back to `input: Record<string, unknown>`; test with a schema-less tool |
| Registry changes but generator not re-run | CI test regenerates into a temp dir and diffs against a committed catalog hash; fails loudly (same discipline as `MCP_TOOLS.md` ref_check) |
| Duplicate tool names across registry fragments | Pre-existing orphan `discovery.rs` must be ignored; generator reads `get_tool_definitions()` (registry.rs only), not files — immune by construction. See [[mcp-tools-registry-is-registry-rs]] |
| `.engram/` not gitignored → generated tree committed by accident | Generator refuses to write unless `.engram/` is gitignored, or adds the entry; verify in the implementing task |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| Generated API diverges from live tool behavior | Agent calls a wrapper whose types lie | Single source of truth = `get_tool_definitions()`; CI drift test | Regenerate-and-diff in CI |
| New CLI subcommand expands `engram-cli` surface | Maintenance + test burden | Subcommand is additive, behind explicit invocation; no default behavior change | `cargo test` on engram-cli; subcommand has its own tests |
| Scope creep into the executor | Security surface (code exec) leaks into an "easy" change | Decision #5 fences it out; executor requires its own ADR | This canvas + reviewer checks no runner code lands |
| `.gitignore`/catalog-hash artifacts add noise to the gate | Sensors churn | Treat generated output as build artifact; only the generator + catalog snapshot are tracked | `sensors.sh` green with the new files |

## Out of Scope (explicit)

- **Sandboxed runner / code executor** — deferred to a separate security ADR. No
  process spawning, no `eval`, no credential mounts in this work.
- **Python generator** — fast-follow after TS proves the design.
- **MCP-tool delivery of the generator** — CLI only for now.
- **Promote-script-to-skill** (roadmap item) — unrelated; tracked separately.

## Decision

**Proceed — as a generator-only feature, TS-first, on-demand + gitignored,
catalog-from-binary (3a), CLI subcommand only.**

Reason: All five product questions are locked with the owner. The design reuses
the existing source of truth (`get_tool_definitions()`), adds no request hot
path, does not collide with the hand-written SDK, and fences out the one genuinely
risky piece (the executor) into a separate ADR. The remaining work is a
well-bounded codegen task suitable for TDD + the standard gate.

## Next steps (post-decision, not part of this canvas)

1. Implement `engram mcp export-code-api --lang typescript --out <dir>` (TDD; the
   first test is the catalog-emit step, then the renderer).
2. Add a CI drift test (regenerate → diff against committed catalog hash).
3. Defer: Python target and security ADR for a sandboxed runner. Bootstrap
   tool-count fix and orphan `discovery.rs` cleanup were closed separately in
   the MCP registry hygiene follow-up. See
   [[feedback-validate-param-type-not-just-value]] for the boundary-validation
   lesson to carry into the renderer's input handling.
