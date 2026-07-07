# Using Engram From Another Repository

This guide shows how to connect any project repository to Engram so coding agents and application code can store and retrieve persistent project memory. Use it when you want one shared source of truth for decisions, conventions, and recurring project context instead of leaving that knowledge scattered across chats and meeting notes.

## Deployment Modes

Use one of these modes:

- **Local mode**: each machine runs `engram-server` and stores memory in a local SQLite database.
- **Hosted mode**: repositories and agents connect to a shared private Engram server over HTTPS.

For personal or internal project memory, hosted mode is often simpler: one
persistent memory service, separated by workspaces, without requiring a public
multi-tenant SaaS gateway. Local mode is better when the repository should own
its own context store and keep the data beside the codebase.

## 1. Install Engram

From crates.io:

```bash
cargo install engram-core
```

The crates.io package can lag the latest GitHub/Homebrew release. Check the
crate badge or release page if you need an exact version.

Or from this repository:

```bash
git clone https://github.com/aiconnai/engram.git
cd engram
cargo install --path .
```

Verify the binaries are available:

```bash
engram-server --version
engram-cli --version
```

## 2. Choose a Database Path

Skip this section if you use the shared cloud server.

For one shared local memory store:

```bash
export ENGRAM_DB_PATH="$HOME/.local/share/engram/memories.db"
```

For a repo-specific memory store, run this from your project repo:

```bash
mkdir -p .engram
export ENGRAM_DB_PATH="$PWD/.engram/memories.db"
```

If you use a repo-local database, add it to `.gitignore`:

```gitignore
.engram/
```

## 3. Connect Coding Agents With MCP

Engram works best with coding agents through MCP because it gives the agent direct access to the source of truth instead of forcing it to reconstruct context from chat history.

### Option A: Local MCP Server

Use this when each repo or machine should run its own local Engram process.

#### Cursor

Create `.cursor/mcp.json` in your project repo:

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

#### Claude Code

Add this server to your Claude Code MCP config:

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram-server",
      "args": ["--transport", "stdio"],
      "env": {
        "ENGRAM_DB_PATH": "/absolute/path/to/your/repo/.engram/memories.db",
        "ENGRAM_EMBEDDING_MODEL": "tfidf"
      }
    }
  }
}
```

Use an absolute path for global configs so the server starts with the intended database.

### Option B: Private Hosted MCP Server

Use this when all repos should share the same private Engram instance.

Example private hosted endpoint:

```text
https://engram.example.com/mcp
```

The server requires:

```http
Authorization: Bearer <ENGRAM_HTTP_API_KEY>
```

Store the token outside the repository. For example:

```bash
mkdir -p ~/.config/engram
printf "%s\n" "your-token-here" > ~/.config/engram/engram-mcp-http-api-key
chmod 600 ~/.config/engram/engram-mcp-http-api-key
```

Do not commit this token.

MCP clients differ in how they support remote HTTP MCP servers. If your client supports remote MCP directly, configure:

```json
{
  "mcpServers": {
    "engram-cloud": {
      "url": "https://engram.example.com/mcp",
      "headers": {
        "Authorization": "Bearer ${ENGRAM_HTTP_API_KEY}"
      }
    }
  }
}
```

If your MCP client only supports local command servers, run a small local proxy or use the local `engram-server` mode instead. Application code can still call the hosted server directly over HTTP as shown below.

## 4. Add Repo Instructions for Agents

Add this section to your repository's `AGENTS.md`, `CLAUDE.md`, or equivalent agent instruction file:

```markdown
## Persistent Memory

Use Engram for project memory.

- Before major work, search Engram for relevant prior decisions, bugs, and conventions.
- Store durable decisions, architecture notes, integration details, and recurring gotchas.
- Do not store secrets, API keys, personal data, or transient command output.
- Prefer workspace names that match this repository, for example `my-org/my-repo`.
- When using cloud Engram, always set the workspace explicitly.

Suggested MCP tools:
- `memory_search` before implementation or debugging
- `memory_create` for new durable knowledge
- `memory_list` to inspect recent memories
- `memory_score` before promoting uncertain context
- `memory_explain` before trusting a surprising result
- `memory_reconcile_conflict` when a newer correction contradicts older memory
- `memory_promote_to_permanent` only when you intend to change canonical retention tier
```

## 5. Store Initial Project Context

### Local CLI

From your project repo:

```bash
engram-cli create "Repository uses Rust 1.75+ and SQLite WAL for local storage" \
  --type note \
  --tags "repo,context"

engram-cli create "Run cargo fmt --check, cargo clippy, and cargo test before PRs" \
  --type decision \
  --tags "workflow,verification"
```

### Hosted MCP

Call the hosted MCP endpoint with the project workspace:

```bash
TOKEN=$(cat ~/.config/engram/engram-mcp-http-api-key)

curl -X POST https://engram.example.com/mcp \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "memory_create",
      "arguments": {
        "content": "Run cargo fmt --check, cargo clippy, and cargo test before PRs",
        "memory_type": "decision",
        "tags": ["workflow", "verification"],
        "workspace": "my-org/my-repo"
      }
    }
  }'
```

For MCP clients, you can ask the agent:

> Remember this repository's verification flow: run cargo fmt --check, cargo clippy, and cargo test before PRs.

## 6. Search During Development

CLI:

```bash
engram-cli search "verification flow"
engram-cli search "database migration gotchas"
engram-cli search "why did we choose sqlite"
```

MCP prompt:

> Search Engram for prior decisions about database migrations before changing the schema.

Hosted MCP search:

```bash
TOKEN=$(cat ~/.config/engram/engram-mcp-http-api-key)

curl -X POST https://engram.example.com/mcp \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "memory_search",
      "arguments": {
        "query": "database migration gotchas",
        "limit": 10,
        "workspace": "my-org/my-repo"
      }
    }
  }'
```

## 7. Use Engram From App Code

### Local HTTP

Start Engram as an HTTP service:

```bash
engram-server --transport http --http-port 8080 --http-api-key "$ENGRAM_HTTP_API_KEY"
```

Engram's core HTTP transport exposes MCP over HTTP at `/mcp` (`/v1/mcp` is a
compatibility alias). Create a memory:

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Authorization: Bearer $ENGRAM_HTTP_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "memory_create",
      "arguments": {
        "content": "Payments service retries Stripe webhooks with exponential backoff",
        "memory_type": "note",
        "tags": ["payments", "stripe"],
        "workspace": "my-org/my-repo"
      }
    }
  }'
```

Search:

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Authorization: Bearer $ENGRAM_HTTP_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "memory_search",
      "arguments": {
        "query": "stripe webhook retry",
        "limit": 10,
        "workspace": "my-org/my-repo"
      }
    }
  }'
```

### Private Hosted HTTP

Use the shared hosted endpoint:

```bash
TOKEN=$(cat ~/.config/engram/engram-mcp-http-api-key)

curl -X POST https://engram.example.com/mcp \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "memory_stats",
      "arguments": {}
    }
  }'
```

## 8. Optional: Local Semantic Embeddings

By default, Engram uses TF-IDF embeddings, which require no model download. For local semantic embeddings:

```bash
cargo install --path /path/to/engram --features local-embeddings
engram-cli model download minilm-l6-v2
ENGRAM_EMBEDDING_MODEL=local engram-server --transport stdio
```

If the model lives outside the default cache:

```bash
export ENGRAM_ONNX_MODEL_DIR="/path/to/minilm-l6-v2"
```

## 9. Recommended Repository Setup

### Local Mode

```text
your-repo/
├── .cursor/
│   └── mcp.json
├── AGENTS.md
├── .gitignore
└── .engram/          # ignored, local SQLite data
```

Recommended `.gitignore` entry:

```gitignore
.engram/
```

### Hosted Mode

Commit only documentation and non-secret config:

```text
your-repo/
├── AGENTS.md
├── .env.example
└── docs/
    └── MEMORY.md
```

Example `.env.example`:

```bash
ENGRAM_MCP_URL=https://engram.example.com/mcp
ENGRAM_WORKSPACE=my-org/my-repo
# Set locally, never commit:
# ENGRAM_HTTP_API_KEY=...
# Optional: widen first-connect MCP exposure beyond the Essential profile.
# ENGRAM_TOOL_TIER=standard
# Optional MCP HTTP rate limiting:
# ENGRAM_HTTP_RATE_LIMIT_RPS=120
# ENGRAM_HTTP_RATE_LIMIT_BURST=240
# ENGRAM_HTTP_RATE_LIMIT_KEY=x-tenant-id
```

## 10. Tool Exposure Profiles

Engram keeps `tools/list` small by default so MCP hosts do not load the full
registry on first connect. The default Essential profile includes the core
create/recall/context tools plus `discover_tools`.

Use `ENGRAM_TOOL_TIER=standard` when an agent or MCP host needs the broader
common workflow surface immediately, or `ENGRAM_TOOL_TIER=all` for every
compiled tool. Tools outside the current profile remain discoverable through
`discover_tools`, including feature-disabled tools with their build flag.

## 11. Reusable Council Workflow (`engram-council` Skill)

For architecture decisions or design discussions, use the MCP `memory_council` tool or the new reusable SDK wrappers:

- Python: `CouncilSkill` in `engram_client.integrations`
- TypeScript: `CouncilSkill` in `engram-client`

Python:

```python
from engram_client import EngramClient
from engram_client.integrations import CouncilSkill


async def run_consensus() -> None:
    async with EngramClient(
        base_url="https://engram.example.com",
        api_key="ek_...",
        tenant="my-tenant",
    ) as client:
        skill = CouncilSkill(
            client,
            default_workspace="architecture",
            default_timeout_seconds=120,
        )
        result = await skill.ask_with_persistence(
            "Evaluate tradeoffs: Postgres HA vs MySQL Galera"
        )
        print(result)
```

TypeScript:

```typescript
import { CouncilSkill, EngramClient } from "engram-client";

const client = new EngramClient({
  baseUrl: "https://engram.example.com",
  apiKey: "ek_...",
  tenant: "my-tenant",
});

const skill = new CouncilSkill(client, {
  defaultWorkspace: "architecture",
  defaultTimeoutSeconds: 120,
});

const result = await skill.askWithPersistence(
  "Evaluate tradeoffs: Postgres HA vs MySQL Galera"
);
```

Direct MCP fallback:

```bash
curl -X POST https://engram.example.com/mcp \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "tools/call",
    "params": {
      "name": "memory_council",
      "arguments": {
        "prompt": "Should we switch primary DB from SQLite to Postgres?",
        "workspace": "my-org/my-repo",
        "persist": true
      }
    }
  }'
```

### Repo Skill: `engram-council`

If this repository is also used with Claude agents, install the reusable skill at
`skills/engram-council/SKILL.md` so future prompts reuse a consistent consensus
workflow.

Recommended when:

- you want the same architecture-review flow across projects
- you want decision checkpoints persisted as memories
- you want the same structure for tradeoff prompts every time

The skill uses the same underlying tools shown above:

- Python/TypeScript `CouncilSkill`
- MCP `memory_council` tool as fallback

## Safety Rules

- Do not store secrets or credentials.
- Do not commit `ENGRAM_HTTP_API_KEY`.
- Do not commit Engram SQLite database files unless you intentionally want shared memory artifacts.
- Use repo-local databases for isolated experiments.
- Use one shared database if you want cross-repo memory and conventions.
- In cloud mode, always pass `workspace` to keep project memories separated.

## Dream Snapshot Review Pipeline

RFC 0007 defines an implemented Dream Snapshot Review Pipeline for repositories that
want agents to synthesize project memory without silently changing canonical facts.
The key rule is that dream output is a review candidate until a user or agent
explicitly reviews and applies it with confirmation.

The shipped behavior is controlled by the `dream-phase` feature flag. If that
feature is not enabled, `dream_*` tools will not be listed and MCP calls to them
should not be used.

Use the eval scaffold in
[`docs/DREAM_SNAPSHOT_EVALS.md`](DREAM_SNAPSHOT_EVALS.md) when validating this
workflow in a repository harness. It covers carry-forward context,
preferences/constraints, freshness, provenance, unsafe raw-log rejection, and
the no-mutation-before-apply boundary.

Use these tools only if they appear in the generated reference at
[`docs/MCP_TOOLS.md`](MCP_TOOLS.md), which is also gated by the feature flag.
