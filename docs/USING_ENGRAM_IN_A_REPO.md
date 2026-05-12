# Using Engram From Another Repository

This guide shows how to connect any project repository to Engram so coding agents and application code can store and retrieve persistent project memory.

## 1. Install Engram

From crates.io:

```bash
cargo install engram-core
```

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

Engram works best with coding agents through MCP. Add an MCP config to the repository or to the agent's global config.

### Cursor

Create `.cursor/mcp.json` in your project repo:

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram-server",
      "args": ["--mcp"],
      "env": {
        "ENGRAM_DB_PATH": ".engram/memories.db",
        "ENGRAM_EMBEDDING_MODEL": "tfidf"
      }
    }
  }
}
```

### Claude Code

Add this server to your Claude Code MCP config:

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram-server",
      "args": ["--mcp"],
      "env": {
        "ENGRAM_DB_PATH": "/absolute/path/to/your/repo/.engram/memories.db",
        "ENGRAM_EMBEDDING_MODEL": "tfidf"
      }
    }
  }
}
```

Use an absolute path for global configs so the server starts with the intended database.

## 4. Add Repo Instructions for Agents

Add this section to your repository's `AGENTS.md`, `CLAUDE.md`, or equivalent agent instruction file:

```markdown
## Persistent Memory

Use Engram for project memory.

- Before major work, search Engram for relevant prior decisions, bugs, and conventions.
- Store durable decisions, architecture notes, integration details, and recurring gotchas.
- Do not store secrets, API keys, personal data, or transient command output.
- Prefer workspace names that match this repository, for example `my-org/my-repo`.

Suggested MCP tools:
- `memory_search` before implementation or debugging
- `memory_create` for new durable knowledge
- `memory_list` to inspect recent memories
```

## 5. Store Initial Project Context

From your project repo:

```bash
engram-cli create "Repository uses Rust 1.75+ and SQLite WAL for local storage" \
  --type note \
  --tags "repo,context"

engram-cli create "Run cargo fmt --check, cargo clippy, and cargo test before PRs" \
  --type decision \
  --tags "workflow,verification"
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

## 7. Use the HTTP API From App Code

Start Engram as an HTTP service:

```bash
engram-server --http --port 8080
```

Create a memory:

```bash
curl -X POST http://localhost:8080/v1/memories \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Payments service retries Stripe webhooks with exponential backoff",
    "memory_type": "note",
    "tags": ["payments", "stripe"]
  }'
```

Search:

```bash
curl "http://localhost:8080/v1/search?q=stripe+webhook+retry&limit=10"
```

## 8. Optional: Local Semantic Embeddings

By default, Engram uses TF-IDF embeddings, which require no model download. For local semantic embeddings:

```bash
cargo install --path /path/to/engram --features local-embeddings
engram-cli model download minilm-l6-v2
ENGRAM_EMBEDDING_MODEL=local engram-server --mcp
```

If the model lives outside the default cache:

```bash
export ENGRAM_ONNX_MODEL_DIR="/path/to/minilm-l6-v2"
```

## 9. Recommended Repository Setup

For a repo-local setup, commit only the config and ignore the data:

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

## Safety Rules

- Do not store secrets or credentials.
- Do not commit Engram SQLite database files unless you intentionally want shared memory artifacts.
- Use repo-local databases for isolated experiments.
- Use one shared database if you want cross-repo memory and conventions.
