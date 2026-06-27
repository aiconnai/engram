# Cursor MCP Memory Example

This example configures Engram as a repository-local MCP memory server for
Cursor.

## `.cursor/mcp.json`

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

## Repo Ignore Rule

```gitignore
.engram/
```

## Limitation

Cursor gets access through MCP only after its MCP config loads successfully.
Engram does not change Cursor's file index or model context behavior.
