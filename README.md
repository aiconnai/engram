# Engram

**Memory for production AI agents — built for predictable latency.**  
Hybrid search, knowledge graphs, and optional cloud sync — shipped as a single Rust binary.

[![Crates.io](https://img.shields.io/crates/v/engram-core)](https://crates.io/crates/engram-core)
[![docs.rs](https://img.shields.io/docsrs/engram-core)](https://docs.rs/engram-core)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## Choose Your Path

<table>
<tr>
<td width="50%" valign="top">

### Production LLM Apps (Primary)

A persistent memory layer designed for real deployments: fast, stable, and easy to ship.

```bash
# Store a memory
curl -X POST localhost:8080/v1/memories \
  -d '{"content": "User prefers dark mode"}'

# Hybrid search
curl localhost:8080/v1/search?q=user+preferences
```

**What you get:**
- Hybrid search (BM25 + vectors + fuzzy) in one call
- MCP / REST / WebSocket / CLI
- Predictable p95 latency (no runtime, no reindex loops)

</td>
<td width="50%" valign="top">

### Dev Workflow (Bonus)

Capture project context and decision trails so your coding agents stop repeating the same questions.

```bash
# Search decisions
engram-cli search "why did we choose postgres"
```

**What you get:**
- Project Context Discovery (CLAUDE.md, .cursorrules, etc.) via MCP tools
- Decision trails with tags + metadata
- Local-first by default, sync optional

</td>
</tr>
</table>

---

## Use Engram In Your Repository

See [Using Engram From Another Repository](docs/USING_ENGRAM_IN_A_REPO.md) for a practical setup guide covering MCP config, repo-local databases, agent instructions, CLI usage, HTTP access, and local embeddings.

---

### Quick Start

```bash
# Install from crates.io
cargo install engram-core

# Or from source
git clone https://github.com/aiconnai/engram.git
cd engram && cargo install --path .

# Run as MCP server (Claude Code, Cursor, VS Code MCP clients, etc.)
engram-server --mcp

# Or run as HTTP API
engram-server --http --port 8080
```

## Why Engram

Agents forget between sessions. Context windows overflow. Important knowledge gets buried in chat logs.

Engram turns that into a fast, queryable memory system with stable latency and zero runtime dependencies.

| Problem | Engram Solution |
|---------|-----------------|
| Vector search misses exact keywords | **Hybrid search**: BM25 + vectors + fuzzy, fused + ranked |
| Context disappears between sessions | **Persistent memory** on SQLite + WAL |
| Cloud-only products | **Local-first**, optional S3/R2 sync |
| Python/Docker required | **Single Rust binary** (no runtime stack) |
| No project awareness | **Project Context Discovery** (CLAUDE.md, AGENTS.md, .cursorrules, etc.) |

---

## How It Compares

| Feature | Engram | Mem0 | Zep | Letta |
|---------|--------|------|-----|-------|
| Language | Rust | Python | Python | Python |
| MCP Native | Yes | Plugin | No | No |
| Single Binary | Yes | No | No | No |
| Local-first | Yes | Optional | Cloud-first | Optional |
| Hybrid Search | BM25+Vec+Fuzzy | Vec+KV | Vec+Graph | Vec |
| Project Context | Yes | No | No | No |
| Edge-Native Latency | Yes | No | No | No |

> "Edge-native" here means runs beside the agent, with predictable p95 latency and no dependency chain.

---

## Core Features

### Hybrid Search

```bash
# Handles typos, semantic matches, and exact keywords in one query
engram-cli search "asynch awiat rust"
# → Returns: "Use async/await for I/O-bound work in Rust"
```

### Multi-Workspace Support

Isolate memories by project or context:

```bash
# Create memory in a specific workspace
engram-cli create "API keys stored in Vault" --workspace my-project

# List workspaces
engram-cli workspace list
```

### Memory Tiering

Two tiers for different retention needs:

- **Permanent**: Important knowledge, decisions (never expires)
- **Daily**: Session context, scratch notes (auto-expire after 24h)

```bash
# Create a daily memory (expires in 24h)
engram-cli create "Current debugging task" --tier daily
```

### Session Transcript Indexing

Store and search conversation transcripts:

```bash
# Index a conversation session
engram-cli session index --session-id chat-123 --messages messages.json

# Search within transcripts
engram-cli session search "error handling"
```

### Identity Links (Entity Unification)

Link different mentions to canonical identities:

```bash
# Create identity with aliases
engram-cli identity create user:ronaldo --alias "Ronaldo" --alias "@ronaldo"
```

### Knowledge Graph

```bash
# Export the graph
engram-cli graph --format json --output graph.json
```

Entity extraction (`memory_extract_entities`) links memories through shared entities.  
Multi-hop traversal and shortest-path are available via MCP tools:
- `memory_traverse`
- `memory_find_path`

### Multiple Interfaces

- **MCP**: Native Model Context Protocol for Claude Code, Cursor, VS Code MCP clients
- **REST**: Standard HTTP API for any client
- **WebSocket**: Real-time updates
- **CLI**: Developer-friendly commands

### Salience Scoring

Dynamic memory prioritization based on recency, frequency, importance, and feedback:

```bash
# Get top memories by salience
engram-cli salience top --limit 10

# Boost a memory's salience
engram-cli salience boost 42
```

Salience decays over time, transitioning memories through lifecycle states: Active -> Stale -> Archived.

### Context Quality

5-component quality assessment (clarity, completeness, freshness, consistency, source trust):

```bash
# Quality report for a workspace
engram-cli quality report --workspace my-project

# Find near-duplicate memories
engram-cli quality duplicates
```

Includes conflict detection for contradictions between memories and resolution workflows.

### Optional Meilisearch Backend

Offload search to Meilisearch for larger-scale deployments (feature-gated):

```bash
# Build with Meilisearch support
cargo build --features meilisearch

# Run with Meilisearch indexer
engram-server --meilisearch-url http://localhost:7700 --meilisearch-indexer
```

SQLite remains the source of truth. MeilisearchIndexer syncs changes in the background.

### MCP Resources & Prompts (v0.6.0)

Engram exposes MCP Resources and Prompts for richer agent integration:

**Resources** — Query-only URI templates:
- `engram://memory/{id}` — Get specific memory
- `engram://workspace/{name}` — Get workspace statistics
- `engram://workspace/{name}/memories` — List workspace memories
- `engram://stats` — Global statistics
- `engram://entities` — Extracted entities

**Prompts** — Guided workflows for agents:
- `create-knowledge-base` — Steps to build a new knowledge base
- `daily-review` — Daily memory review and archival workflow
- `search-and-organize` — Search results with suggested tags
- `seed-entity` — Initialize entity graph from project

### Streamable HTTP Transport (v0.6.0)

Run Engram as HTTP server with JSON-RPC 2.0 support:

```bash
# HTTP-only server (port 8080)
engram-server --transport http --port 8080

# Both HTTP and stdio (default)
engram-server --transport both --port 8080

# Bearer token authentication
ENGRAM_BEARER_TOKEN=secret-token-here engram-server --transport http
```

Clients connect via HTTP with JSON-RPC 2.0 at `/v1/mcp` endpoint.

### Project Context Discovery

Ingest and query instruction and policy files using MCP tools:
- `memory_scan_project`
- `memory_get_project_context`

**Supported patterns:**
- CLAUDE.md
- AGENTS.md (agora com documentação útil para agentes)
- .cursorrules
- .github/copilot-instructions.md
- .aider.conf.yml (se existir)
- CONVENTIONS.md, CODING_GUIDELINES.md (se existirem)

---

## MCP Configuration

Add to your MCP config (for example: `~/.claude/mcp.json`, `.cursor/mcp.json`, or your VS Code MCP extension config):

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

If you built from source instead of installing via Homebrew, use the full path to the binary (e.g. `/path/to/engram/target/release/engram-server`).

### Available MCP Tools

The MCP tool reference is generated from source of truth (`src/mcp/tools.rs`) and tracked in `docs/MCP_TOOLS.md`.

- Full reference: [docs/MCP_TOOLS.md](docs/MCP_TOOLS.md)
- Generated count and schema are in that reference (single source of truth).
- Regenerate with: `./scripts/generate-mcp-reference.sh`

---

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `ENGRAM_DB_PATH` | SQLite database path | `~/.local/share/engram/memories.db` |
| `ENGRAM_STORAGE_URI` | S3/R2 URI for cloud sync | - |
| `ENGRAM_CLOUD_ENCRYPT` | AES-256-GCM encryption | `false` |
| `ENGRAM_EMBEDDING_MODEL` | Embedding model (`tfidf`, `local`, `openai`) | `tfidf` |
| `ENGRAM_ONNX_MODEL_DIR` | Local embedding model directory (`model.onnx` + `tokenizer.json`) | platform data dir |
| `ENGRAM_CLEANUP_INTERVAL` | Expired memory cleanup interval (seconds) | `3600` |
| `ENGRAM_WS_PORT` | WebSocket server port (0 = disabled) | `0` |
| `OPENAI_API_KEY` | OpenAI API key (for `openai` embeddings) | - |
| `MEILISEARCH_URL` | Meilisearch URL (requires `--features meilisearch`) | - |
| `MEILISEARCH_API_KEY` | Meilisearch API key | - |
| `MEILISEARCH_INDEXER` | Enable background sync to Meilisearch | `false` |
| `MEILISEARCH_SYNC_INTERVAL` | Sync interval in seconds | `60` |

### Local embeddings

Local sentence-transformer embeddings are opt-in and keep the default binary small:

```bash
cargo build --features local-embeddings
./target/debug/engram-cli model download minilm-l6-v2
ENGRAM_EMBEDDING_MODEL=local ./target/debug/engram-server
```

This backend uses ONNX Runtime with `all-MiniLM-L6-v2` (384 dimensions). The model is downloaded explicitly and is not bundled into the binary.

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Engram Server                           │
├─────────────────────────────────────────────────────────────────┤
│  MCP (stdio)  │  REST (HTTP)  │  WebSocket  │  CLI              │
├─────────────────────────────────────────────────────────────────┤
│                    Intelligence Layer                           │
│  • Salience scoring  • Quality assessment  • Entity extraction  │
│  • Context compression  • Lifecycle management                  │
├─────────────────────────────────────────────────────────────────┤
│                      Search Layer                               │
│  • BM25 (FTS5)  • Vectors (sqlite-vec)  • Fuzzy  • RRF fusion  │
│  • Optional Meilisearch backend for scaled deployments          │
├─────────────────────────────────────────────────────────────────┤
│                     Storage Layer                               │
│  • SQLite + WAL  • Turso/libSQL  • Connection pooling           │
│  • Optional S3/R2 sync with AES-256 encryption                  │
└─────────────────────────────────────────────────────────────────┘
```

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
