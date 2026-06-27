# Claude Code MCP Memory with Engram

Use this guide when Claude Code needs project memory that survives context
window resets, terminal sessions, or teammate handoffs.

## When to Use

- Claude Code repeats questions that were already answered in prior sessions.
- You want searchable decisions from `AGENTS.md`, `CLAUDE.md`, docs, meeting
  notes, and implementation handoffs.
- You need a private memory store beside a repository or a shared Engram server.

## Quick Command

Install and run the MCP server:

```bash
cargo install engram-core
ENGRAM_DB_PATH="$HOME/.local/share/engram/claude-code.db" engram-server --transport stdio
```

Add Engram to your Claude Code MCP config:

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram-server",
      "args": ["--transport", "stdio"],
      "env": {
        "ENGRAM_DB_PATH": "/absolute/path/to/.engram/memories.db",
        "ENGRAM_EMBEDDING_MODEL": "tfidf"
      }
    }
  }
}
```

Seed a decision from the CLI:

```bash
engram-cli create "Use SQLite + WAL for local Engram memory" \
  --type decision \
  --tags "claude-code,architecture"
```

## What to Review

- Use absolute paths in global Claude configs so the intended database opens.
- Keep `.engram/` out of Git unless you intentionally version memory artifacts.
- Add repo instructions telling Claude when to call `memory_search` and
  `memory_create`.
- Decide whether memory is per-repo, per-user, or shared by workspace.

## Real Limitations

- Engram does not replace Claude Code's own planning or editing workflow.
- Claude will only use Engram when the MCP server is configured and instructions
  tell it when memory should be searched or written.
- The optional `skills/engram-council/` pack is separate from the MCP server.
- Do not store secrets, API keys, private customer data, or raw command logs.
