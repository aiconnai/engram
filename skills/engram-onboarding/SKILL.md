---
name: engram-onboarding
description: Bootstrap an AI agent's engram memory session in a new repository. Guides through session_land, project scan, workspace setup, first memory creation, search, and session close. Use when starting work in a repo that has engram configured, when setting up memory for the first time, or when a new agent needs to establish context. Skip if the session is already active and workspaces are already configured.
---

# Engram Onboarding

Use this skill to establish an engram memory session in a repository from scratch. Follow the steps in order. Each step builds on the previous one.

## When to Use

- First agent session in a repo that has engram running
- Resuming work after a long gap (re-land and re-scan)
- New agent picking up work mid-project

## Prerequisites

- Engram MCP server is reachable (check `ENGRAM_URL` or the MCP transport config)
- You have a `project_path` (absolute path to the repo root)
- You have an `agent_id` (e.g. `"claude-code"`, `"codex"`, or a task ID like `"T42"`)

---

## Step 1 — Bootstrap: Land the Session

Call `session_land` to register your agent and load prior session context.

```json
{
  "name": "session_land",
  "arguments": {
    "project_path": "/absolute/path/to/repo",
    "agent_id": "claude-code",
    "context": "Starting onboarding session for initial workspace setup"
  }
}
```

**On success:** you receive a `session_id`. Save it — some tools accept it as a filter.

**On failure:**
- `connection refused` → engram server is not running. Start it or check `ENGRAM_URL`.
- `project_path not found` → use the absolute path, not a relative one.
- Any other error → report to the user and stop. Do not proceed without a session.

---

## Step 2 — Scan: Discover Existing Memories

Call `memory_scan_project` to index the repo and surface any memories already stored.

```json
{
  "name": "memory_scan_project",
  "arguments": {
    "project_path": "/absolute/path/to/repo",
    "include_archived": false
  }
}
```

**Interpret the result:**
- `memories_found > 0` → read the returned summaries. Note workspaces already in use.
- `memories_found = 0` → this is a fresh start. Continue to workspace setup.
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

For a typical engineering repo, ensure these workspaces exist. Create any that are missing:

| Workspace | Purpose |
|-----------|---------|
| `decisions` | Architecture and design choices |
| `bugs` | Known issues, root causes, workarounds |
| `architecture` | System design, data flow, component map |
| `tasks` | Active work, blockers, carry-overs |
| `onboarding` | Newcomer context, setup notes |

Create a missing workspace with `scope_set`:

```json
{
  "name": "scope_set",
  "arguments": {
    "workspace": "decisions",
    "description": "Architecture and design choices for this repository"
  }
}
```

Repeat for each missing workspace. Skip workspaces that already exist.

---

## Step 4 — First Memory: Record the Onboarding Event

Store a memory documenting that onboarding ran and what you found.

```json
{
  "name": "memory_create",
  "arguments": {
    "content": "Engram onboarding completed. Scanned project, found N existing memories across M workspaces. Created workspaces: decisions, bugs, architecture, tasks, onboarding.",
    "memory_type": "episodic",
    "workspace": "onboarding",
    "importance": 0.7,
    "tags": ["onboarding", "session-start", "setup"]
  }
}
```

Replace `N` and `M` with the actual counts from Step 2.

**Field guidance:**
- `memory_type`: use `"episodic"` for events, `"semantic"` for facts, `"procedural"` for how-to knowledge
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

When the session ends, call `harness_handoff` to persist a summary for the next agent.

```json
{
  "name": "harness_handoff",
  "arguments": {
    "session_summary": "Onboarding complete. Workspaces created: decisions, bugs, architecture, tasks, onboarding. No prior memories found — fresh install. Next agent should run memory_scan_project and check decisions workspace before making architectural choices.",
    "agent_id": "claude-code",
    "project_path": "/absolute/path/to/repo"
  }
}
```

Write the summary in imperative past tense: what was done, what was found, what the next agent needs to know. Do not include session IDs or timestamps — engram records those.

---

## Quick Reference: Common Follow-Up Calls

**Store a decision:**
```json
{
  "name": "memory_create",
  "arguments": {
    "content": "Chose PostgreSQL over SQLite for multi-writer workload support.",
    "memory_type": "semantic",
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
    "memory_type": "semantic",
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
| `session_land` fails | Stop. Fix connectivity before continuing. |
| `memory_scan_project` fails | Log and continue. Non-blocking. |
| `workspace_list` fails | Attempt `scope_set` directly; list may not be implemented. |
| `memory_create` fails | Retry once. If still failing, report to user. |
| `memory_search` returns empty | Retry without `workspace` filter. Check engram logs. |
| `harness_handoff` fails | Log the summary to `tasks` workspace as a fallback memory. |
