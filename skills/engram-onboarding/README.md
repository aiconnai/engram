# engram-onboarding

A Claude Code skill that guides an AI agent through bootstrapping an engram memory session in a new repository.

## What It Does

Walks the agent through six steps in order:

1. `session_land` — register the agent and load prior context
2. `memory_scan_project` — discover existing memories
3. `workspace_list` + `scope_set` — ensure standard workspaces exist
4. `memory_create` — record the onboarding event as the first memory
5. `memory_search` — verify retrieval works
6. `harness_handoff` — persist a session summary for the next agent

Each step includes concrete JSON examples and error-handling guidance.

## Install

```bash
cp -r skills/engram-onboarding ~/.claude/skills/
```

## When to Invoke

Invoke this skill (e.g. `/engram-onboarding`) at the start of a session when:

- You are a new agent in a repo that has engram configured
- You are resuming work after a long gap
- You need to verify that workspaces and retrieval are set up correctly

Skip it if the session is already active and workspaces are confirmed to exist.
