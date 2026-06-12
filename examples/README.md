# Engram Examples

These examples show how Engram fits into common agent ecosystems without
overstating support.

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
