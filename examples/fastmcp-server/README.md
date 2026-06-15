# FastMCP Server Memory Pattern

Engram is not a FastMCP plugin. A FastMCP project can call Engram as a separate
MCP or HTTP memory service.

## Run Engram

```bash
ENGRAM_HTTP_API_KEY="change-me" engram-server --transport http --http-port 8080
```

## Runnable Example

Dry-run without installing FastMCP:

```bash
python examples/fastmcp-server/server.py
```

Run the FastMCP server:

```bash
ENGRAM_HTTP_API_KEY=change-me \
ENGRAM_URL=http://localhost:8080/mcp \
uv run examples/fastmcp-server/server.py --live
```

The FastMCP server exposes two tools:

- `remember_project_decision(content: str)` -> Engram `memory_create`
- `search_project_memory(query: str)` -> Engram `memory_search`

## Limitation

Keep FastMCP responsible for your domain tools. Use Engram only for durable
memory, search, decisions, and provenance.

See [MCP memory server guide](../../docs/integrations/mcp-memory-server.md).
