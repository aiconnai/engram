# Engram

**MCP memory server for Claude Code, Cursor, and AI agents.**

[![Crates.io](https://img.shields.io/crates/v/engram-core)](https://crates.io/crates/engram-core)
[![docs.rs](https://img.shields.io/docsrs/engram-core)](https://docs.rs/engram-core)
[![CI](https://github.com/aiconnai/engram/actions/workflows/ci.yml/badge.svg)](https://github.com/aiconnai/engram/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Engram is a Rust, local-first memory layer for teams that need agents to
remember proprietary project context across sessions. It ingests meetings,
docs, transcripts, and decisions; stores them in SQLite; indexes them with
hybrid BM25/vector/fuzzy search and knowledge graph links; and exposes the same
source of truth through MCP, HTTP JSON-RPC, CLI, and Python/TypeScript SDKs.

Use Engram when coding agents, research crews, or internal AI tools need
durable memory with provenance instead of rebuilding context from chat history.

---

## Quick Start

```bash
# Install with Homebrew (tracks the GitHub release artifacts)
brew install aiconnai/engram/engram

# Or install from crates.io (may lag the latest GitHub/Homebrew release)
cargo install engram-core

# Or from source
git clone https://github.com/aiconnai/engram.git
cd engram && cargo install --path .
```

Run as an MCP server:

```bash
# stdio transport (Claude Code, Cursor, VS Code MCP clients, etc.)
engram-server --transport stdio

# HTTP JSON-RPC transport
engram-server --transport http --http-port 8080

# Both (default)
engram-server --transport both --http-port 8080
```

### MCP Configuration

Add to your MCP config (e.g. `~/.claude/mcp.json`, `.cursor/mcp.json`, or your
VS Code MCP extension config):

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram-server",
      "args": [],
      "env": {
        "ENGRAM_DB_PATH": "~/.local/share/engram/memories.db"
      }
    }
  }
}
```

By default, `tools/list` exposes a focused Essential profile plus
`discover_tools`. Set `ENGRAM_TOOL_TIER=standard` in `env` when an MCP host
needs the broader pre-0.23 tool surface on first connect, or
`ENGRAM_TOOL_TIER=all` for every compiled tool.

If you built from source, use the full path to the binary
(e.g. `/path/to/engram/target/release/engram-server`).

First calls over HTTP:

```bash
# Store a memory
curl -X POST localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"memory_create","arguments":{"content":"User prefers dark mode"}}}'

# Hybrid search
curl -X POST localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"memory_search","arguments":{"query":"user preferences"}}}'
```

For a full repository setup (repo-local databases, agent instructions, CLI,
local embeddings), see
[Using Engram From Another Repository](docs/USING_ENGRAM_IN_A_REPO.md).

---

## Why Engram

Agents forget between sessions. Context windows overflow. Important knowledge
gets buried in chat logs and meeting notes. Engram turns scattered artifacts
into a structured memory layer that agents query directly from the source:

| Problem | Engram Solution |
|---------|-----------------|
| Knowledge is spread across meetings, docs, and chats | **Structured ingestion workflows** into one memory layer |
| Search misses exact terms or related concepts | **Hybrid search**: BM25 + vectors + fuzzy, fused and ranked |
| Context disappears between sessions | **Persistent memory** on SQLite + WAL |
| Teams need a private source of truth | **Local-first** with optional sync and shared workspaces |
| Agents need direct access to the same facts | **MCP-native** tools for read/write/search workflows |
| No project awareness | **Project Context Discovery** (CLAUDE.md, AGENTS.md, .cursorrules, etc.) |

How it works:

1. **Ingest** meetings, docs, transcripts, and notes.
2. **Organize** — normalize, tag, and store with durable provenance.
3. **Index** — combine exact, fuzzy, and semantic retrieval.
4. **Expose via MCP** — let Claude Code and other agents query the same
   knowledge base.

Wondering how Engram compares to Mem0, Zep/Graphiti, Cognee, or simpler MCP
memory servers? See the honest comparison in
[docs/COMPARISON.md](docs/COMPARISON.md).

---

## Core Features

### Hybrid Search

```bash
# Handles typos, semantic matches, and exact keywords in one query
engram-cli search "asynch awiat rust"
# → Returns: "Use async/await for I/O-bound work in Rust"
```

### Multi-Workspace Support

Isolate memories by project or context through the MCP tools:

```json
{
  "name": "memory_create",
  "arguments": {
    "content": "API keys are stored in Vault",
    "workspace": "my-project",
    "memory_type": "decision"
  }
}
```

### Memory Tiering & Lifecycle

Two tiers for different retention needs:

- **Permanent**: important knowledge and decisions (never expires) —
  `memory_create`
- **Daily**: session context and scratch notes (auto-expire after 24h) —
  `memory_create_daily`

Salience scoring prioritizes memories by recency, frequency, importance, and
feedback (`salience_top`, `salience_boost`). Salience decays over time;
lifecycle transitions (`Active -> Stale -> Archived`) are decided by
`lifecycle_run`.

### Session Transcript Indexing

Store conversation transcripts with `session_index` and search them with
`memory_search` (`"include_transcripts": true`).

### Knowledge Graph & Identity Links

Entity extraction (`memory_extract_entities`) links memories through shared
entities; `identity_create` unifies different mentions under canonical
identities. Multi-hop traversal and shortest-path are available via
`memory_traverse` and `memory_find_path`, and the graph can be exported:

```bash
engram-cli graph --format json --output graph.json
```

### Context Quality

5-component quality assessment (clarity, completeness, freshness, consistency,
source trust) via `quality_report`, plus duplicate detection
(`quality_find_duplicates`) and conflict detection/resolution workflows for
contradictions between memories.

### Project Context Discovery

Ingest and query repo instruction and policy files with `memory_scan_project`
and `memory_get_project_context`. Supported patterns include `CLAUDE.md`,
`AGENTS.md`, `.cursorrules`, `.github/copilot-instructions.md`,
`.aider.conf.yml`, `CONVENTIONS.md`, and `CODING_GUIDELINES.md` (when present).

### MCP Resources & Prompts

**Resources** — query-only URI templates: `engram://memory/{id}`,
`engram://workspace/{name}`, `engram://workspace/{name}/memories`,
`engram://stats`, `engram://entities`.

**Prompts** — guided workflows for agents: `create-knowledge-base`,
`daily-review`, `search-and-organize`, `seed-entity`.

### Optional Meilisearch Backend

Offload search to Meilisearch for larger-scale deployments (feature-gated).
SQLite remains the source of truth; the indexer syncs changes in the background:

```bash
cargo build --features meilisearch
engram-server --meilisearch-url http://localhost:7700 --meilisearch-indexer
```

### Dream Snapshot Review Pipeline

RFC 0007 defines an implemented reviewable pipeline for derived memory
proposals. Dream output is candidate memory until reviewed and explicitly
applied with confirmation; it is not canonical memory by default. See the
[contract](docs/rfcs/0007-dream-snapshot-review-pipeline.md) and
[eval scaffold](docs/DREAM_SNAPSHOT_EVALS.md).

### Available MCP Tools

The MCP tool reference is generated from the source of truth
(`src/mcp/tools/registry.rs`) and tracked in
[docs/MCP_TOOLS.md](docs/MCP_TOOLS.md).

- By default, `tools/list` exposes only the Essential profile plus
  `discover_tools`; set `ENGRAM_TOOL_TIER=standard` or `all` for broader
  profiles.
- Generated count, tier, group, feature requirement, and schema are in that
  reference (single source of truth).
- Regenerate with: `./scripts/generate-mcp-reference.sh`

---

## Interfaces & Integrations

- **MCP** over stdio and HTTP for Claude Code, Cursor, VS Code MCP clients, and
  other Model Context Protocol hosts.
- **HTTP JSON-RPC 2.0** at `POST /mcp` (`POST /v1/mcp` as compatibility alias),
  with optional Bearer token auth via `ENGRAM_HTTP_API_KEY` — see
  [MCP HTTP Authentication](docs/MCP_AUTH.md).
- **WebSocket** event streaming, opt-in via `ENGRAM_WS_PORT`.
- **CLI** (`engram-cli`) over the same memory store.
- **Python and TypeScript SDKs** for application code and hosted deployments.

| Ecosystem | How Engram helps |
|-----------|------------------|
| Claude Code / Cursor / VS Code MCP clients | Native MCP server for durable project memory, decision search, and repo context retrieval. |
| CrewAI | Python SDK adapters for short-term, long-term, and entity memory. |
| LangChain | Python SDK chat history and vector-store-style adapters over hybrid search. |
| LlamaIndex | Python SDK document store, vector store, and chat store adapters. |
| OpenAI Assistants API / Threads | Python adapter syncs thread messages into searchable session memory. |
| OpenAI Agents SDK, LangGraph, FastMCP, Playwright MCP, Browser Use | No first-party adapters; integrate via MCP, HTTP JSON-RPC, or the SDKs — see the runnable examples below. |

**Runnable examples:**

- [Claude MCP](examples/claude-mcp/) — Claude Code MCP config plus a seed/search smoke test.
- [OpenAI Agents SDK](examples/openai-agents-sdk/) — function tools that call Engram over HTTP JSON-RPC.
- [FastMCP server](examples/fastmcp-server/) — FastMCP tools backed by Engram memory calls.
- [LangGraph tool](examples/langgraph-tool/) — graph nodes that search and store Engram memory.

### Council Skill (SDKs + MCP)

Engram exposes the `memory_council` MCP tool for structured multi-perspective
consensus, wrapped by both SDKs (`engram_client.integrations.CouncilSkill` in
Python, `CouncilSkill` from `engram-client` in TypeScript). A ready-to-use
Claude skill lives in [`skills/engram-council/`](skills/engram-council/) —
install the folder in your agent's skills environment and keep your Engram MCP
server configured as usual.

---

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `ENGRAM_DB_PATH` | SQLite database path | `~/.local/share/engram/memories.db` |
| `ENGRAM_TOOL_TIER` | MCP tool surface (`essential`, `standard`, `all`) | `essential` |
| `ENGRAM_STORAGE_URI` | S3/R2 URI for cloud sync | - |
| `ENGRAM_CLOUD_ENCRYPT` | AES-256-GCM encryption | `false` |
| `ENGRAM_EMBEDDING_MODEL` | Embedding model (`tfidf`, `local`, `openai`) | `tfidf` |
| `ENGRAM_ONNX_MODEL_DIR` | Local embedding model directory (`model.onnx` + `tokenizer.json`) | platform data dir |
| `ENGRAM_CLEANUP_INTERVAL` | Expired memory cleanup interval (seconds) | `3600` |
| `ENGRAM_HTTP_PORT` | HTTP transport port | `3100` |
| `ENGRAM_HTTP_BIND_ADDRESS` | HTTP transport bind address | `127.0.0.1` |
| `ENGRAM_HTTP_API_KEY` | Bearer token for the HTTP transport | - |
| `ENGRAM_WS_PORT` | WebSocket server port (0 = disabled) | `0` |
| `ENGRAM_WS_BIND_ADDRESS` | WebSocket server bind address | `127.0.0.1` |
| `ENGRAM_WS_AUTH_KEY` | Bearer token for WebSocket upgrade authentication | - |
| `ENGRAM_WS_ALLOWED_ORIGINS` | Comma-separated exact browser origins allowed for WebSocket upgrades (no wildcard) | - |
| `ENGRAM_WS_MAX_CONNECTIONS` | Maximum simultaneous upgraded WebSocket connections | `128` |
| `ENGRAM_WS_MAX_MESSAGE_BYTES` | Maximum bytes in one inbound WebSocket frame or message | `65536` |
| `ENGRAM_WS_READ_IDLE_TIMEOUT_SECONDS` | Maximum post-handshake silence between inbound frames | `60` |
| `ENGRAM_GRPC_PORT` | gRPC transport port (requires `--features grpc`) | `50051` |
| `ENGRAM_GRPC_BIND_ADDRESS` | gRPC transport bind address (requires `--features grpc`) | `127.0.0.1` |
| `ENGRAM_GRPC_API_KEY` | Bearer token for the gRPC transport | - |
| `OPENAI_API_KEY` | OpenAI API key (for `openai` embeddings) | - |
| `MEILISEARCH_URL` | Meilisearch URL (requires `--features meilisearch`) | - |
| `MEILISEARCH_API_KEY` | Meilisearch API key | - |
| `MEILISEARCH_INDEXER` | Enable background sync to Meilisearch | `false` |
| `MEILISEARCH_SYNC_INTERVAL` | Sync interval in seconds | `60` |

### Local embeddings

Local sentence-transformer embeddings are opt-in and keep the default binary
small:

```bash
cargo build --features local-embeddings
./target/debug/engram-cli model download minilm-l6-v2
ENGRAM_EMBEDDING_MODEL=local ./target/debug/engram-server
```

This backend uses ONNX Runtime with `all-MiniLM-L6-v2` (384 dimensions). The
model is downloaded explicitly and is not bundled into the binary.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Engram Server                           │
├─────────────────────────────────────────────────────────────────┤
│  MCP stdio    │  HTTP MCP     │  WebSocket* │  CLI / SDKs       │
├─────────────────────────────────────────────────────────────────┤
│                    Intelligence Layer                           │
│  • Salience scoring  • Quality assessment  • Entity extraction  │
│  • Context compression  • Lifecycle management                  │
├─────────────────────────────────────────────────────────────────┤
│                      Search Layer                               │
│  • BM25 (FTS5)  • Vectors (cosine)  • Fuzzy  • RRF fusion       │
│  • Optional Meilisearch backend for scaled deployments          │
├─────────────────────────────────────────────────────────────────┤
│                     Storage Layer                               │
│  • SQLite + WAL (local-first)  • Connection pooling             │
│  • S3/R2 Snapshot Attestation & Cloud Sync                      │
└─────────────────────────────────────────────────────────────────┘
```

\* WebSocket event streaming is opt-in via `ENGRAM_WS_PORT`; HTTP, WebSocket,
and gRPC network listeners bind `127.0.0.1` by default. Explicit public binds
require the operator to configure the relevant bind-address variable and deploy
the matching authentication control; public WebSocket binds require
`ENGRAM_WS_AUTH_KEY` even behind a trusted proxy. MCP stdio remains the
default agent interface. WebSocket subscriptions select one workspace with
`/ws?workspace=<name>` (default: `default`); browser clients must send an Origin
listed exactly in `ENGRAM_WS_ALLOWED_ORIGINS`, while non-browser clients may
omit the Origin header. Resource limits default to 128 simultaneous connections,
64 KiB per inbound frame/message, and a 60-second read-idle timeout. A saturated
listener rejects the upgrade with HTTP `503`; oversized clients close with
WebSocket code `1009`, and read-idle clients close with code `1008`. Aggregate
resource counters are exposed by the WebSocket listener's `/health` response;
they never include principals, credentials, workspaces, or message content.

---

## Documentation

- [Quickstart](docs/QUICKSTART.md) · [Getting Started](docs/GETTING_STARTED.md) · [User Guide](docs/USER_GUIDE.md)
- [Using Engram From Another Repository](docs/USING_ENGRAM_IN_A_REPO.md)
- [Engram vs alternatives](docs/COMPARISON.md)
- [MCP memory server guide](docs/integrations/mcp-memory-server.md)
- [Claude Code MCP memory guide](docs/integrations/claude-code-mcp-memory.md)
- [Cursor MCP memory guide](docs/integrations/cursor-mcp-memory.md)
- [OpenAI Agents memory guide](docs/integrations/openai-agents-memory.md)
- [Architecture](docs/ARCHITECTURE.md) · [MCP tool reference](docs/MCP_TOOLS.md) · [Roadmap](docs/ROADMAP.md)

---

## Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for conventions.

```bash
cargo test           # Run all tests
cargo clippy         # Lint
cargo fmt            # Format
```

---

## License

MIT License — see [LICENSE](LICENSE) for details.
