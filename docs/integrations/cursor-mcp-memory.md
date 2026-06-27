# Cursor MCP Memory with Engram

Use this guide when Cursor should query durable project context through MCP
instead of relying only on the active chat or indexed files.

## When to Use

- Cursor needs prior architecture decisions or debugging history.
- Multiple repositories should share the same project memory conventions.
- You want Cursor, Claude Code, and scripts to read the same memory backend.

## Quick Command

Install Engram:

```bash
cargo install engram-core
```

Create `.cursor/mcp.json` in the repository:

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram-server",
      "args": ["--transport", "stdio"],
      "env": {
        "ENGRAM_DB_PATH": ".engram/memories.db",
        "ENGRAM_EMBEDDING_MODEL": "tfidf"
      }
    }
  }
}
```

Ignore the local database if it is repo-specific:

```gitignore
.engram/
```

## What to Review

- Cursor can start `engram-server` from the configured path.
- The repo's agent instructions describe what should become durable memory.
- The workspace name matches the repo or team convention.
- Large imports are intentional and do not include secrets.

## Real Limitations

- Engram does not change Cursor's editor behavior or indexing model.
- Local `.engram/` databases are machine-local unless you configure sync or a
  shared server.
- MCP client behavior differs between Cursor versions; verify the server starts
  before relying on memory in a workflow.
