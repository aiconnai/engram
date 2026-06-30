# MCP Memory Server for AI Agents

Use this guide when an AI agent or MCP client needs durable memory across
sessions, projects, or teammates. Engram runs as a local-first Rust MCP server
and stores the canonical memory in SQLite.

## When to Use

- You use Claude Code, Cursor, VS Code MCP clients, or another MCP host.
- Agents need to search prior decisions, transcripts, notes, or project policy.
- You want local storage by default, with optional shared or hosted deployment.
- You need hybrid retrieval: BM25, vector search, fuzzy search, and graph links.

## Quick Command

```bash
cargo install engram-core
ENGRAM_DB_PATH="$HOME/.local/share/engram/memories.db" engram-server --transport stdio
```

For HTTP MCP transport:

```bash
ENGRAM_HTTP_API_KEY="change-me" engram-server --transport http --http-port 8080
```

Then call JSON-RPC at `POST /mcp`:

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Authorization: Bearer $ENGRAM_HTTP_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory_search","arguments":{"query":"architecture decisions"}}}'
```

## What to Review

- The MCP config points at the intended `engram-server` binary.
- `ENGRAM_DB_PATH` is explicit for repo-specific or shared team memory.
- Workspaces are named consistently across agents and repositories.
- Secrets and credentials are not stored as memory content.
- `docs/MCP_TOOLS.md` matches the server version you are running.

## Real Limitations

- Engram is a memory server, not an autonomous agent framework.
- Browser automation, code execution, and UI control belong to other MCP
  servers; Engram stores and retrieves the resulting knowledge.
- Local-first SQLite is the default. Shared/team deployments need explicit
  hosting, auth, and backup choices.
- Optional search and embedding backends require their feature flags and
  environment variables.
