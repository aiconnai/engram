# Engram Examples

These examples show how Engram fits into common agent ecosystems without
overstating support.

## Deterministic smoke test

From a clean checkout, run:

```bash
bash scripts/test-examples.sh
```

The aggregate test uses a disposable database and a loopback-only real Engram
HTTP server. It never contacts an external service and requires no API keys.
The ecosystem-specific Python packages are intentionally not installed: their
dependency-free Engram adapters are syntax checked and exercised directly.

| Family | Classification | Smoke coverage |
| --- | --- | --- |
| Rust library demos | Runnable | All Cargo examples compile |
| Claude MCP | Runnable | Real `memory_create` and `memory_search` HTTP round trip |
| Cursor MCP | Illustrative configuration | JSON block and stdio arguments validate |
| CrewAI memory | Illustrative adapter sketch | SDK adapter imports/classes validate |
| OpenAI Agents SDK | Runnable integration pattern | Dry run plus real Engram adapter round trip |
| FastMCP server | Runnable integration pattern | Dry run plus real Engram adapter round trip |
| LangGraph tool | Runnable integration pattern | Dry run plus real Engram adapter round trip |

## Native Examples

- [Claude MCP](claude-mcp/) - configure Engram as a Claude Code MCP memory server.
- [Cursor MCP](cursor-mcp/) - use Engram from Cursor through `.cursor/mcp.json`.
- [CrewAI memory](crewai-memory/) - use the Python SDK's CrewAI memory adapters.

## Integration Patterns

These ecosystems can benefit from Engram through MCP, HTTP JSON-RPC, or the SDKs,
but they do not have first-party adapters in this repository today.

- [OpenAI Agents SDK](openai-agents-sdk/) - persist agent facts and decisions via HTTP.
- [FastMCP server](fastmcp-server/) - call Engram as a separate memory service.
- [LangGraph tool](langgraph-tool/) - use Engram search/create calls from graph nodes.

## What These Examples Are Not

- They are not a promise that every ecosystem has a native adapter.
- They are not production auth, tenancy, or backup templates.
- They are starting points for wiring Engram into an agent workflow.
