# Engram

**Persistent, local-first memory engine for AI agents across Frontier LLMs.**

[![CI](https://github.com/aiconnai/engram/actions/workflows/ci.yml/badge.svg)](https://github.com/aiconnai/engram/actions/workflows/ci.yml)
[![Release](https://img.shields.io/badge/Release-v0.23.0%20GA-success.svg)](https://github.com/aiconnai/engram/releases)
[![Website](https://img.shields.io/badge/Website-aiconnai.github.io%2Fengram-cyan.svg)](https://aiconnai.github.io/engram/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/engram-core.svg)](https://crates.io/crates/engram-core)
[![docs.rs](https://img.shields.io/docsrs/engram-core)](https://docs.rs/engram-core)
[![Python SDK](https://img.shields.io/badge/Python%20SDK-PyPI-blue?logo=python)](sdks/python)
[![TypeScript SDK](https://img.shields.io/badge/TypeScript%20SDK-npm-purple?logo=typescript)](sdks/typescript)
[![MCP Server](https://img.shields.io/badge/MCP%20Server-243%20Tools-emerald)](docs/MCP_TOOLS.md)

- [Website & Interactive Playground](https://aiconnai.github.io/engram/)
- [MCP Tools Catalog](https://aiconnai.github.io/engram/mcp-tools.html)
- [System Architecture](https://aiconnai.github.io/engram/architecture.html)
- [Quickstart Guide](docs/QUICKSTART.md)
- [Using Engram in a Repo](docs/USING_ENGRAM_IN_A_REPO.md)
- [Comparison Guide](docs/COMPARISON.md)
- [Contributing](CONTRIBUTING.md)
- [Security Policy](SECURITY.md)

---

Engram is an offline, high-performance Rust memory engine for teams shipping autonomous AI agents across the modern AI stack. Native adapters and first-class Model Context Protocol (MCP) support cover **Claude Code, Cursor & Cursor Rules, Google Antigravity, OpenAI Agents SDK, Gemini 3.7 Flash, DeepSeek-V4-Pro, Grok 4.6, GPT-5.6 Sol, LangGraph, CrewAI, LlamaIndex, and FastMCP**.

It organizes decisions, transcripts, architecture notes, and repos into SQLite with sub-millisecond retrieval, **3-way hybrid search** (BM25 + Vectors + Fuzzy), **dynamic knowledge graphs with identity resolution**, **multi-tier temporal salience decay**, and **contradiction detection**.

Engram operates 100% offline, keeping your proprietary context on your machine with zero telemetry, and exposes unified access via MCP stdio, HTTP JSON-RPC 2.0 (`POST /mcp`), WebSocket event streaming, CLI, and Python/TypeScript SDKs.

---

## At a Glance

| Area | What Engram does |
|------|------------------|
| **Storage Layer** | Local-first SQLite + WAL with connection pooling, sub-millisecond ACID writes, and optional AES-256-GCM encrypted S3/R2 cloud backup sync. |
| **Hybrid Search** | 3-way retrieval fusing BM25 (FTS5), dense cosine vector embeddings (MiniLM ONNX / OpenAI), and fuzzy Levenshtein via Reciprocal Rank Fusion (RRF). |
| **Knowledge Graph** | Automatic entity extraction, bidirectional relationship edges, canonical identity resolution, and multi-hop shortest path graph traversal. |
| **Salience & Lifecycle** | Multi-tier retention distinguishing permanent decisions from 24h daily scratchpads; dynamic exponential decay scoring. |
| **Context Quality** | 5-component quality scoring (clarity, completeness, freshness, consistency, trust) + automated contradiction and conflict detection. |
| **Project Discovery** | Auto-ingests and indexes `CLAUDE.md`, `AGENTS.md`, `.cursorrules`, `copilot-instructions.md`, and `.aider.conf.yml`. |
| **Universal Transports** | MCP over stdio, HTTP JSON-RPC 2.0 (`/mcp`), WebSocket real-time subscription streaming, and gRPC. |
| **Developer Surface** | Native Rust crate (`engram-core`), async Python SDK (`engram-client`), TypeScript SDK (`@aiconnai/engram-client`), and CLI (`engram-cli`). |

---

## Works With

Engram provides durable, structured memory anywhere an AI agent reasons, plans, or codes:

| Ecosystem | How Engram helps |
|-----------|------------------|
| **Claude Code & Claude Desktop** | Native MCP memory server for durable cross-session decisions, transcript search, and repo context retrieval. |
| **Cursor & Cursor Rules** | Ingests `.cursorrules`, provides instant semantic search over architecture patterns and coding conventions. |
| **Google Antigravity** | Persistent multi-agent memory and shared workspace context for complex agentic refactors and long-running swarms. |
| **OpenAI Agents SDK & ChatGPT** | Callable tools for durable thread state, entity lookup, and fact verification over HTTP JSON-RPC. |
| **Gemini 3.7 Flash & DeepSeek-V4** | Sub-millisecond memory lookups matching high-throughput reasoning loops and accurate code snippet indexing. |
| **LangGraph & LangChain** | Python SDK adapters for stateful graph nodes that search, store, and compact memory across multi-agent workflows. |
| **CrewAI Swarms** | Python SDK adapters for short-term, long-term, and entity memory across specialized agent crews. |
| **LlamaIndex** | Python SDK document store, vector store, and chat store adapters backed by hybrid retrieval. |
| **FastMCP & Custom MCP Hosts** | Ready-to-use MCP tools for seamless plug-and-play memory in any MCP client. |

Runnable examples live under [examples/](examples/README.md), with focused guides for [Claude MCP memory](docs/integrations/claude-code-mcp-memory.md), [Cursor MCP memory](docs/integrations/cursor-mcp-memory.md), and [OpenAI Agents memory](docs/integrations/openai-agents-memory.md).

---

## Why Engram?

AI agents without persistent memory suffer from critical operational flaws:
- **Context Loss Between Sessions**: Agents forget architectural decisions, conventions, and user preferences as soon as a chat closes.
- **Context Window Exhaustion**: Stuffing full histories into prompts wastes tokens, degrades reasoning, and causes context overflow.
- **Keyword vs Concept Blindspots**: Pure vector search misses exact symbol names and error codes; pure keyword search fails on semantic concepts.
- **Contradictory Directives**: Outdated notes conflict with new requirements, causing agents to oscillate between conflicting code styles.

Engram solves this by providing a unified, local-first intelligence layer:

```
┌─────────────────────────────────────────────────────────────────┐
│                          AI Agent Hosts                         │
│   Claude Code │ Cursor │ Antigravity │ OpenAI Agents │ CrewAI   │
└────────────────────────────────┬────────────────────────────────┘
                                 │ MCP stdio / HTTP / WS / gRPC
┌────────────────────────────────▼────────────────────────────────┐
│                         Engram Server                           │
├─────────────────────────────────────────────────────────────────┤
│                     Intelligence Layer                          │
│  • Salience scoring  • Quality assessment  • Entity extraction  │
│  • Contradiction detection  • Identity alias resolution         │
├─────────────────────────────────────────────────────────────────┤
│                       Search Layer                              │
│  • BM25 (FTS5)  • Vectors (Cosine)  • Fuzzy  • RRF fusion       │
├─────────────────────────────────────────────────────────────────┤
│                      Storage Layer                              │
│  • SQLite + WAL (local-first)  • Multi-workspace isolation      │
│  • Optional S3/R2 Cloud Backup Sync (AES-256-GCM)               │
└─────────────────────────────────────────────────────────────────┘
```

---

## How It Compares

| Feature | Engram | Mem0 | Zep (Graphiti) | Cognee | Generic MCP Memory |
|---------|:------:|:----:|:--------------:|:------:|:------------------:|
| **Core Runtime** | **Rust (Single binary)** | Python | Python / Cloud | Python | Node.js / Python |
| **100% Offline / Local-First** | **Yes (SQLite+WAL)** | Partial | Cloud service | Partial | Yes |
| **3-Way Hybrid Search** | **BM25 + Vector + Fuzzy** | Vector only | Vector + Graph | Graph + Vector | Keyword only |
| **Knowledge Graph & Identity** | **Yes (Built-in)** | Basic | Yes (Graphiti) | Yes | None |
| **Memory Tiering & Salience** | **Permanent + 24h Daily** | Manual | Single tier | Manual | None |
| **Contradiction Detection** | **Yes (Temporal)** | Overwrite only | Edge invalidation | None | None |
| **Transports Supported** | **stdio, HTTP, WS, gRPC** | REST API | REST / GraphQL | REST API | stdio only |
| **Active MCP Tools** | **243 tools** | ~6 tools | ~8 tools | ~10 tools | ~4 tools |
| **Zero Cloud Lock-in** | **Yes** | No | No | Partial | Yes |

For a comprehensive breakdown, see [docs/COMPARISON.md](docs/COMPARISON.md).

---

## Quick Start

### Installation

```bash
# 🍺 Install with Homebrew (macOS & Linux)
brew install aiconnai/engram/engram

# 🦀 Or install from crates.io
cargo install engram-core

# ⚡ Or build from source
git clone https://github.com/aiconnai/engram.git
cd engram && cargo build --release
```

### Run as an MCP Server

```bash
# stdio transport (for Claude Code, Cursor, Antigravity, VS Code)
engram-server --transport stdio

# HTTP JSON-RPC transport (POST /mcp)
engram-server --transport http --http-port 8080

# Both stdio + HTTP (default)
engram-server --transport both --http-port 8080
```

### MCP Configuration

Add to your MCP config (`~/.claude/mcp.json`, `.cursor/mcp.json`, or Antigravity):

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram-server",
      "args": ["--transport", "stdio"],
      "env": {
        "ENGRAM_DB_PATH": "~/.local/share/engram/memories.db",
        "ENGRAM_TOOL_TIER": "standard"
      }
    }
  }
}
```

---

## SDK Quickstarts

### Python SDK (`sdks/python/`)

```bash
pip install engram-client
```

```python
import asyncio
from engram_client import EngramClient

async def main():
    async with EngramClient(base_url="http://localhost:8080") as client:
        # Create a persistent decision memory
        memory = await client.create_memory(
            content="Use AES-256-GCM for cloud backup encryption",
            workspace="security",
            memory_type="decision"
        )
        print(f"Stored memory: {memory.id}")

        # 3-way hybrid search with typo tolerance
        results = await client.search_memory(
            query="aes encryption cloud backup",
            workspace="security"
        )
        for hit in results.memories:
            print(f"[{hit.score:.2f}] {hit.content}")

if __name__ == "__main__":
    asyncio.run(main())
```

### TypeScript SDK (`sdks/typescript/`)

```bash
npm install @aiconnai/engram-client
```

```typescript
import { EngramClient } from '@aiconnai/engram-client';

const client = new EngramClient({ baseUrl: 'http://localhost:8080' });

async function run() {
  const memory = await client.memories.create({
    content: 'PostgreSQL connection pool sized to 25 with 5s timeout',
    workspace: 'backend',
    memoryType: 'permanent'
  });

  const searchResults = await client.search.query({
    query: 'postgre pool size',
    workspace: 'backend'
  });

  console.log('Match:', searchResults.matches[0]?.content);
}

run().catch(console.error);
```

### CLI (`engram-cli`)

```bash
# Store a memory
engram-cli store "API keys are rotated daily via Vault" --workspace backend --type decision

# Hybrid search with explanation
engram-cli search "vault key rotat" --workspace backend --explain

# Knowledge graph traversal
engram-cli graph traverse --entity "AuthService" --depth 2

# Scan project context
engram-cli project-context scan .
```

---

## Core Capabilities

### 1. 3-Way Hybrid Search
Fuses exact keyword matches with dense semantic vectors and fuzzy typo correction:
```bash
engram-cli search "asynch awiat rust"
# → Returns: "Use Tokio async/await for all I/O-bound workers; reserve std::thread for CPU compute."
```

### 2. Knowledge Graph & Identity Resolution
Extracts entities automatically, unifies aliases under canonical identities, and allows multi-hop graph traversals:
```bash
engram-cli graph --format json --output graph.json
```

### 3. Multi-Tier Retention & Salience Decay
- **Permanent**: Critical architecture choices and invariants (never expires).
- **Daily**: Session context and scratch notes (auto-expires after 24h).
- **Salience Scoring**: Ranks memories by recency, access frequency, importance, and user feedback.

### 4. Contradiction Detection & Quality Assessment
Detects conflicting directives across sessions and assesses memory quality across 5 dimensions (clarity, completeness, freshness, consistency, trust).

### 5. Council Skill for Consensus Review
Structured multi-perspective review via `memory_council` tool before committing critical decisions to canonical memory.

---

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `ENGRAM_DB_PATH` | SQLite database path | `~/.local/share/engram/memories.db` |
| `ENGRAM_TOOL_TIER` | MCP tool surface (`essential`, `standard`, `all`) | `essential` |
| `ENGRAM_STORAGE_URI` | S3/R2 URI for cloud sync backup | - |
| `ENGRAM_CLOUD_ENCRYPT` | AES-256-GCM encryption for cloud backup | `false` |
| `ENGRAM_EMBEDDING_MODEL` | Embedding model (`tfidf`, `local`, `openai`) | `tfidf` |
| `ENGRAM_ONNX_MODEL_DIR` | Local embedding model dir (`model.onnx` + `tokenizer.json`) | platform data dir |
| `ENGRAM_HTTP_PORT` | HTTP transport port | `3100` |
| `ENGRAM_HTTP_BIND_ADDRESS` | HTTP transport bind address | `127.0.0.1` |
| `ENGRAM_HTTP_API_KEY` | Bearer token for HTTP transport authentication | - |
| `ENGRAM_WS_PORT` | WebSocket server port (0 = disabled) | `0` |
| `ENGRAM_WS_AUTH_KEY` | Bearer token for WebSocket upgrade authentication | - |
| `ENGRAM_WS_ALLOWED_ORIGINS` | Allowed browser origins for WebSocket upgrades | - |
| `ENGRAM_GRPC_PORT` | gRPC transport port (requires `--features grpc`) | `50051` |
| `MEILISEARCH_URL` | Meilisearch URL (requires `--features meilisearch`) | - |

---

## Documentation

- [Quickstart Guide](docs/QUICKSTART.md) · [Getting Started](docs/GETTING_STARTED.md) · [User Guide](docs/USER_GUIDE.md)
- [Using Engram in Another Repository](docs/USING_ENGRAM_IN_A_REPO.md)
- [Engram vs Alternatives (Honest Comparison)](docs/COMPARISON.md)
- [MCP Memory Server Guide](docs/integrations/mcp-memory-server.md)
- [Claude Code Integration Guide](docs/integrations/claude-code-mcp-memory.md)
- [Cursor Integration Guide](docs/integrations/cursor-mcp-memory.md)
- [OpenAI Agents Integration Guide](docs/integrations/openai-agents-memory.md)
- [System Architecture](docs/ARCHITECTURE.md) · [MCP Tool Reference](docs/MCP_TOOLS.md) · [Roadmap](docs/ROADMAP.md)

---

## Contributing

Contributions are warmly welcomed! See [CONTRIBUTING.md](CONTRIBUTING.md) and [STANDARDS.md](STANDARDS.md) for development workflows.

```bash
cargo test           # Run unit and integration tests
cargo clippy         # Lint with Clippy
cargo fmt --check    # Check code formatting
```

---

## License

MIT License — see [LICENSE](LICENSE) for details.
