---
name: engram-onboarding
description: Bootstrap an AI agent's engram memory session in a new repository. Guides through loading prior context, project scan, workspace setup, first memory creation, search, and session close. Use when starting work in a repo that has engram configured, when setting up memory for the first time, or when a new agent needs to establish context. Skip if the session is already active and workspaces are already configured.
---

# Engram Onboarding

Use this skill to establish an engram memory session in a repository from scratch. Follow the steps in order. Each step builds on the previous one.

## When to Use

- First agent session in a repo that has engram running
- Resuming work after a long gap (re-scan and reload context)
- New agent picking up work mid-project

## Prerequisites

- Engram MCP server is reachable (check `ENGRAM_URL` or the MCP transport config)
- You have a `path` (absolute path to the repo root)

---

## Step 1 — Bootstrap: Load Prior Context

Call `memory_build_context` to retrieve relevant memories and establish prior context before doing any work.

```json
{
  "name": "memory_build_context",
  "arguments": {
    "query": "project context decisions architecture setup",
    "strategy": "balanced",
    "limit": 20,
    "depth": 2,
    "timeframe": "all"
  }
}
```

**On success:** you receive a structured prompt context from stored memories. Read it before proceeding — it tells you what was done in prior sessions.

**On failure:**
- `connection refused` → engram server is not running. Start it or check `ENGRAM_URL`.
- Empty result → no prior memories exist. This is a fresh start. Continue to Step 2.
- Any other error → report to the user and stop. Do not proceed without confirming server connectivity.

---

## Step 2 — Scan: Discover Existing Memories

Call `memory_scan_project` to index AI instruction files (CLAUDE.md, AGENTS.md, .cursorrules, etc.) in the repo and surface any memories already stored.

```json
{
  "name": "memory_scan_project",
  "arguments": {
    "path": "/absolute/path/to/repo",
    "extract_sections": true
  }
}
```

**Interpret the result:**
- Memories created → read the returned summaries. Note workspaces already in use.
- No files found → this is a fresh start. Continue to workspace setup.
- `error` → log the error and continue. A scan failure does not block the session.

---

## Step 3 — Workspace Setup

List existing workspaces to avoid creating duplicates.

```json
{
  "name": "workspace_list",
  "arguments": {}
}
```

For a typical engineering repo, ensure these workspaces exist. Workspaces are created implicitly when you write a memory with `"workspace": "<name>"` — there is no separate creation call.

| Workspace | Purpose |
|-----------|---------|
| `decisions` | Architecture and design choices |
| `bugs` | Known issues, root causes, workarounds |
| `architecture` | System design, data flow, component map |
| `tasks` | Active work, blockers, carry-overs |
| `onboarding` | Newcomer context, setup notes |

If a workspace from the table above is missing from `workspace_list` output, it will be created automatically when you write the first memory into it in Step 4.

---

## Step 4 — First Memory: Record the Onboarding Event

Store a memory documenting that onboarding ran and what you found.

```json
{
  "name": "memory_create",
  "arguments": {
    "content": "Engram onboarding completed. Scanned project, found N existing memories across M workspaces. Workspaces initialized: decisions, bugs, architecture, tasks, onboarding.",
    "memory_type": "episodic",
    "workspace": "onboarding",
    "importance": 0.7,
    "tags": ["onboarding", "session-start", "setup"]
  }
}
```

Replace `N` and `M` with the actual counts from Step 2.

**Field guidance:**
- `memory_type`: use `"episodic"` for events, `"decision"` for choices made, `"procedural"` for how-to knowledge, `"note"` for general observations, `"context"` for background information, `"learning"` for lessons learned
- `importance`: `0.9–1.0` for critical decisions, `0.6–0.8` for context, `0.3–0.5` for low-priority notes
- `tags`: always include at least one domain tag and one lifecycle tag (e.g. `"session-start"`)

---

## Step 5 — Search: Verify Retrieval Works

Confirm that the memory you just created (and any prior ones) are retrievable.

```json
{
  "name": "memory_search",
  "arguments": {
    "query": "onboarding session setup workspaces",
    "workspace": "onboarding",
    "rerank": true,
    "limit": 5
  }
}
```

**Expected result:** the memory from Step 4 appears in the top results.

**If nothing is returned:**
- Wait 1–2 seconds and retry (indexing may be async).
- Try without `workspace` to search across all scopes.
- If still empty, report to the user — retrieval may need investigation.

**Cross-workspace search example:**

```json
{
  "name": "memory_search",
  "arguments": {
    "query": "architecture decisions database",
    "rerank": true,
    "limit": 10
  }
}
```

---

## Step 6 — Session Close: Handoff Summary

When the session ends, call `harness_handoff` to persist a structured handoff for the next agent. Then call `session_land` to generate a checkpoint memory with session summary.

**harness_handoff** (structured handoff packet):

```json
{
  "name": "harness_handoff",
  "arguments": {
    "current_goal": "Completed initial onboarding — workspaces set up, first memories stored",
    "next_steps": [
      "Continue storing decisions and findings as memories",
      "Run memory_search before starting each new task",
      "Check decisions workspace before making architectural choices"
    ]
  }
}
```

**session_land** (checkpoint memory for continuity):

```json
{
  "name": "session_land",
  "arguments": {
    "session_id": "<session_id from your session context>",
    "workspace": "onboarding",
    "summary": "Onboarding complete. Workspaces initialized: decisions, bugs, architecture, tasks, onboarding. No prior memories found — fresh install.",
    "next_session_hints": [
      "Run memory_build_context before starting work",
      "Check decisions workspace before making architectural choices"
    ]
  }
}
```

Write the summary in imperative past tense: what was done, what was found, what the next agent needs to know.

---

## Quick Reference: Common Follow-Up Calls

**Store a decision:**
```json
{
  "name": "memory_create",
  "arguments": {
    "content": "Chose PostgreSQL over SQLite for multi-writer workload support.",
    "memory_type": "decision",
    "workspace": "decisions",
    "importance": 0.9,
    "tags": ["decision", "database", "architecture"]
  }
}
```

**Record a bug:**
```json
{
  "name": "memory_create",
  "arguments": {
    "content": "Race condition in batch processor when concurrency > 4. Workaround: set BATCH_CONCURRENCY=4 until #123 is fixed.",
    "memory_type": "note",
    "workspace": "bugs",
    "importance": 0.85,
    "tags": ["bug", "concurrency", "workaround"]
  }
}
```

**Resume from prior session:**
```json
{
  "name": "memory_search",
  "arguments": {
    "query": "last session carry-overs blockers",
    "workspace": "tasks",
    "rerank": true,
    "limit": 5
  }
}
```

---

## Error Handling Summary

| Error | Action |
|-------|--------|
| `memory_build_context` fails | Check server connectivity. If empty result, treat as fresh start. |
| `memory_scan_project` fails | Log and continue. Non-blocking. |
| `workspace_list` fails | Continue — workspaces are created on first write. |
| `memory_create` fails | Retry once. If still failing, report to user. |
| `memory_search` returns empty | Retry without `workspace` filter. Check engram logs. |
| `harness_handoff` fails | Log the summary to `tasks` workspace as a fallback memory. |
| `session_land` fails | Non-blocking — the harness_handoff already captured the essentials. |
