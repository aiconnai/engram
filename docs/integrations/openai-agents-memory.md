# OpenAI Agents Memory with Engram

Use this guide when OpenAI-based agent workflows need durable memory outside the
model context window. Engram does not currently include a first-party OpenAI
Agents SDK adapter, but agents can use Engram through MCP, HTTP JSON-RPC, or the
Python and TypeScript SDKs.

## When to Use

- Agent runs need to remember decisions, user preferences, or task outcomes.
- You want a searchable memory service shared with Claude Code, Cursor, or other
  MCP clients.
- You need audit-friendly memory rows instead of hidden model state.

## Quick Command

Run Engram over HTTP:

```bash
cargo install engram-core
ENGRAM_HTTP_API_KEY="change-me" engram-server --transport http --http-port 8080
```

Create a memory from an agent tool or hook:

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Authorization: Bearer $ENGRAM_HTTP_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory_create","arguments":{"content":"User prefers concise status updates","memory_type":"preference","workspace":"openai-agents"}}}'
```

Search before a run:

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Authorization: Bearer $ENGRAM_HTTP_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"memory_search","arguments":{"query":"status update preferences","workspace":"openai-agents"}}}'
```

## What to Review

- Treat Engram calls as explicit tool or lifecycle-hook calls in your agent.
- Use one workspace per product, tenant, or agent boundary.
- Persist only durable facts, decisions, and preferences.
- Propagate HTTP errors visibly so memory failures do not look like success.
- If you use OpenAI Assistants API threads, review the native Python
  `EngramThreadStore` adapter in `sdks/python/engram_client/integrations/`.

## Real Limitations

- There is no native OpenAI Agents SDK adapter in this repository today.
- Engram does not choose what an agent should remember; your workflow must call
  the memory tools deliberately.
- HTTP mode needs explicit auth and deployment hardening before shared use.
- OpenAI Assistants API thread sync is a separate adapter and should not be
  treated as full OpenAI Agents SDK support.
