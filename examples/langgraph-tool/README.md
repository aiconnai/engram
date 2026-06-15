# LangGraph Tool Memory Pattern

Engram does not currently ship a native LangGraph adapter. Use this pattern when
a graph node or tool needs durable memory.

## Run Engram

```bash
ENGRAM_HTTP_API_KEY="change-me" engram-server --transport http --http-port 8080
```

## Runnable Example

Dry-run without installing LangGraph:

```bash
python examples/langgraph-tool/memory_graph.py
```

Run the live LangGraph flow after starting Engram HTTP transport:

```bash
ENGRAM_HTTP_API_KEY=change-me \
ENGRAM_URL=http://localhost:8080/mcp \
uv run examples/langgraph-tool/memory_graph.py --live
```

The graph has two nodes:

- `search_memory` calls Engram `memory_search` before work uses memory.
- `remember_decision` calls Engram `memory_create` after a durable decision.

## Limitation

LangGraph remains responsible for graph state and control flow. Engram provides
external memory and retrieval, not graph orchestration.

See [MCP memory server guide](../../docs/integrations/mcp-memory-server.md).
