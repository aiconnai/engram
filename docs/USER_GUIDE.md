# Engram User Guide

This guide is for someone who wants to use Engram as persistent memory for an
AI tool, a project repository, or application code.

Use Engram when you want decisions, notes, transcripts, project conventions, and
recurring context to survive across agent sessions instead of living only in
chat history.

## 1. Install Engram

On macOS with Homebrew:

```bash
brew install aiconnai/engram/engram
```

From crates.io:

```bash
cargo install engram-core
```

To enable PDF ingestion from crates.io on Linux, install the feature-gated
worker with the server and CLI:

```bash
cargo install engram-core --features pdf
```

From this repository:

```bash
git clone https://github.com/aiconnai/engram.git
cd engram
cargo install --path .
```

Verify the install:

```bash
engram-server --version
engram-cli --version
```

PDF ingestion requires `engram-pdf-worker` in the same directory as the
running Engram executable. Official GitHub release archives ship it on Linux.
PDF ingestion is unavailable on macOS because a hard process memory boundary
cannot be enforced there. Custom library deployments must package the matching
worker themselves. Missing workers and platforms without enforceable resource
limits fail closed; Engram never falls back to in-process PDF parsing.

## 2. Choose Where Memories Live

For one personal memory database:

```bash
export ENGRAM_DB_PATH="$HOME/.local/share/engram/memories.db"
```

For memory tied to one repository:

```bash
mkdir -p .engram
export ENGRAM_DB_PATH="$PWD/.engram/memories.db"
```

If you use `.engram/` inside a repository, add it to `.gitignore` unless you
intentionally want to version memory artifacts:

```gitignore
.engram/
```

## 3. Connect an AI Tool With MCP

Most users should start with MCP. Add Engram to your MCP client config.

For Claude Code or another global MCP config, use an absolute database path:

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

For Cursor, put the same server block in `.cursor/mcp.json` in the repository.
If the config is repository-local, a relative path is fine:

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

After restarting the MCP client, ask the agent to run `memory_stats`. If that
works, the client can reach Engram.

## 4. Create and Search Memories

Create a memory from the CLI:

```bash
engram-cli create "Deploy to staging before production" \
  --type decision \
  --tags "deploy,process"
```

Search from the CLI:

```bash
engram-cli search "staging deploy"
```

From an MCP-enabled agent, use natural language:

```text
Remember this as a decision: deploy to staging before production.
```

```text
Search Engram for prior decisions about deploys.
```

The agent should call tools such as `memory_create`, `memory_search`, and
`memory_list`.

## 5. Use Workspaces

Use workspaces to keep projects or teams separate.

The current CLI stores in the default workspace. For explicit workspace
selection, use an MCP-enabled agent or the HTTP MCP transport and pass
`workspace` to the memory tools:

```text
Create a memory in workspace backend-api: API uses JWT access tokens.
```

```json
{
  "name": "memory_search",
  "arguments": {
    "query": "jwt tokens",
    "workspace": "backend-api",
    "limit": 10
  }
}
```

For hosted or shared Engram instances, include a workspace name in MCP requests
so one project does not mix with another.

## 6. Use Daily vs Permanent Memory

Use permanent memory for durable decisions and project facts:

```bash
engram-cli create "Architecture decision: SQLite is the local source of truth"
```

Use daily memory for temporary session context through MCP:

```json
{
  "name": "memory_create",
  "arguments": {
    "content": "Currently debugging auth middleware ordering",
    "tier": "daily"
  }
}
```

Promote a daily memory when it becomes durable:

```json
{
  "name": "memory_promote_to_permanent",
  "arguments": {
    "id": 42
  }
}
```

## 7. Use Engram Over HTTP

Start the HTTP MCP transport:

```bash
export ENGRAM_HTTP_API_KEY="change-me"
engram-server --transport http --http-port 8080 --http-api-key "$ENGRAM_HTTP_API_KEY"
```

Call `memory_search` over JSON-RPC:

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Authorization: Bearer $ENGRAM_HTTP_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "memory_search",
      "arguments": {
        "query": "deploy process",
        "limit": 10
      }
    }
  }'
```

Use HTTP when application code, tests, or another service needs to call Engram
directly.

## 8. What to Store

Good memories:

- Architecture decisions and why they were made.
- Project conventions and verification commands.
- Recurring bugs, fixes, and operational gotchas.
- Meeting notes, summaries, transcripts, and handoff notes.
- Stable preferences that future agents should respect.

Do not store:

- Secrets, API keys, tokens, passwords, or private keys.
- Raw command logs with credentials or customer data.
- Temporary noise that will not help a future search.
- Personal data unless your deployment and retention policy allow it.

## 9. Recommended Agent Instructions

Add this to a repository's `AGENTS.md`, `CLAUDE.md`, or equivalent:

```markdown
## Persistent Memory

Use Engram for project memory.

- Search Engram before major implementation, debugging, or architecture work.
- Store durable decisions, conventions, integration details, and recurring gotchas.
- Do not store secrets, credentials, raw logs, or sensitive personal data.
- Use the workspace for this repository when creating or searching memories.

Useful tools:
- `memory_search` for prior decisions and context
- `memory_create` for durable new knowledge
- `memory_list` for recent memories
- `memory_stats` to verify the server is connected
```

## 10. Troubleshooting

If `engram-server` is not found, check that Cargo or Homebrew installed binaries
are on `PATH`.

If the agent cannot see Engram tools, restart the MCP client and verify the MCP
config path is correct.

If a global MCP config opens the wrong database, replace relative paths with an
absolute `ENGRAM_DB_PATH`.

If HTTP calls return `401`, check that the `Authorization: Bearer ...` header
matches `ENGRAM_HTTP_API_KEY`.

For the complete MCP tool list, see [`MCP_TOOLS.md`](MCP_TOOLS.md).
