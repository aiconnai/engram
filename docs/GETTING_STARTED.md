# Getting Started with Engram

A quick guide to installing Engram, connecting it to your AI tools, and turning scattered project context into a shared source of truth.

Use Engram when you need one place for meetings, decisions, transcripts, and recurring project knowledge so agents can query the same context instead of reconstructing it from chat history.

---

## Installation

### From Source (Recommended)

```bash
git clone https://github.com/aiconnai/engram.git
cd engram
cargo install --path .
```

This installs `engram-server` and `engram-cli`. PDF ingestion is optional and
currently supported only on Linux; to enable it there, install the `pdf`
feature, which also installs the required isolated worker:

```bash
cargo install --path . --features pdf
```

### Pre-built Binaries

Download platform archives from [GitHub Releases](https://github.com/aiconnai/engram/releases).
Release artifacts are tarballs named `engram-vX.Y.Z-<target>.tar.gz` and
contain `engram-server` and `engram-cli`. Linux archives also contain
`engram-pdf-worker`; keep all three in the same directory to use PDF ingestion.
The server deliberately fails PDF ingestion closed if the worker is absent.

```bash
# Replace VERSION and TARGET with the release tag and platform target you need.
VERSION=v0.22.0
TARGET=x86_64-unknown-linux-gnu
curl -L "https://github.com/aiconnai/engram/releases/download/${VERSION}/engram-${VERSION}-${TARGET}.tar.gz" -o engram.tar.gz
tar -xzf engram.tar.gz
chmod +x engram-server engram-cli
chmod +x engram-pdf-worker
sudo mv engram-server engram-cli /usr/local/bin/
sudo mv engram-pdf-worker /usr/local/bin/
```

### Homebrew (macOS)

```bash
brew install aiconnai/engram/engram
```

GitHub PDF support is available on Linux. PDF ingestion fails closed on macOS
because macOS does not provide the hard process memory boundary required by
Engram's parser threat model. The Linux worker is
a project-owned subprocess with bounded input, output, memory, CPU, file
descriptors, and wall time. Platforms without enforceable worker resource
limits reject PDF extraction rather than parsing in the server process.

Library embedders that enable the Cargo `pdf` feature must deploy the matching
`engram-pdf-worker` beside their executable. The library does not fall back to
in-process PDF parsing when the worker is missing or unsupported.

### Local Docker Build

A Dockerfile is included for local builds. A public GHCR image is not verified
as part of the current release channel. This local image does not advertise or
package the optional PDF worker; build and deploy the worker separately if you
extend that image with the Cargo `pdf` feature.

```bash
docker build -t engram:local .
docker run -v engram-data:/data engram:local
```

---

## Configure MCP for AI Tools

Engram speaks the [Model Context Protocol](https://modelcontextprotocol.io/) (MCP), so it integrates with Claude Code, Cursor, VS Code MCP clients (like Cline/Roo Code), and other MCP-compatible tools. That matters because MCP gives agents direct access to the organized source of truth instead of forcing them to infer it from old conversations.

### Claude Code (Example)

Add to `~/.claude/mcp.json`:

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

### Cursor

Add to `.cursor/mcp.json` in your project root:

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

### Other MCP Clients

Use the same `mcpServers.engram` JSON block in your client's MCP config location.

### Verify Connection

Once configured, your AI tool will have access to the MCP tools listed in [`docs/MCP_TOOLS.md`](docs/MCP_TOOLS.md). Ask it to run `memory_stats` to verify the connection is working and to confirm it can read the shared memory layer.

---

## Create Your First Memory

### Using the CLI

```bash
# Create a simple note
engram-cli create "The API uses JWT tokens for authentication" --type note

# Create with tags
engram-cli create "Deploy to staging before production" --type decision --tags "deploy,process"

# The CLI writes to the default workspace. Use MCP/HTTP examples below when
# you need an explicit workspace.
```

### Using MCP (Any MCP Client)

In Claude Code, Cursor, VS Code MCP clients, or any MCP-enabled assistant, you can use prompts like:

> "Remember that our API uses JWT tokens for authentication"

> "Store this as a decision memory: deploy to staging before production"

> "Search my memories for authentication notes"

The AI will call `memory_create` automatically.

### Using the HTTP MCP Transport

```bash
# Start the HTTP server
engram-server --transport http --http-port 8080

# Create a memory
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "memory_create",
      "arguments": {
        "content": "The API uses JWT tokens for authentication",
        "type": "note",
        "tags": ["auth", "api"]
      }
    }
  }'
```

---

## Search Your Memories

Engram uses hybrid search combining BM25 keyword matching, vector similarity, and fuzzy matching in a single query.

### CLI Search

```bash
# Basic search
engram-cli search "authentication"

# Search handles typos
engram-cli search "authentcation"

# The CLI searches the default workspace. Use MCP/HTTP examples below when
# you need an explicit workspace.
```

### MCP Search

Ask your AI assistant:

> "Search my memories for anything about authentication"

### HTTP Search

```bash
curl -X POST http://localhost:8080/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "memory_search",
      "arguments": {
        "query": "authentication",
        "limit": 10
      }
    }
  }'
```

---

## Organize with Workspaces

Workspaces isolate memories by project or context. The CLI uses the default
workspace; for explicit workspace control, call MCP tools:

```json
{
  "name": "memory_create",
  "arguments": {
    "content": "Use PostgreSQL for this project",
    "workspace": "backend-api",
    "memory_type": "decision"
  }
}
```

```json
{
  "name": "memory_create",
  "arguments": {
    "content": "React with TypeScript",
    "workspace": "frontend-app",
    "memory_type": "context"
  }
}
```

```json
{
  "name": "workspace_list",
  "arguments": {}
}
```

```json
{
  "name": "memory_search",
  "arguments": {
    "query": "database",
    "workspace": "backend-api"
  }
}
```

---

## Memory Tiers

Use tiers to control memory lifetime:

- **permanent** (default): Important knowledge that persists forever
- **daily**: Scratch notes that auto-expire after 24 hours

```bash
# Permanent memory in the default workspace
engram-cli create "Architecture: microservices with event sourcing"
```

Daily tier and promotion are MCP operations:

```json
{
  "name": "memory_create_daily",
  "arguments": {
    "content": "Currently debugging the auth flow"
  }
}
```

```json
{
  "name": "memory_promote_to_permanent",
  "arguments": {
    "id": 42
  }
}
```

---

## Cloud Sync (Optional)

Sync your memories to S3-compatible storage (AWS S3, Cloudflare R2, MinIO):

```bash
# Configure cloud sync
export ENGRAM_STORAGE_URI=s3://my-bucket/engram/memories.db
export ENGRAM_CLOUD_ENCRYPT=true  # AES-256-GCM encryption
export AWS_PROFILE=my-profile

# Start with cloud sync
engram-server
```

This enables cross-machine synchronization with encrypted storage.

---

## Key Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `ENGRAM_DB_PATH` | SQLite database path | `~/.local/share/engram/memories.db` |
| `ENGRAM_STORAGE_URI` | S3 URI for cloud sync | (local only) |
| `ENGRAM_CLOUD_ENCRYPT` | Enable AES-256 encryption | `false` |
| `ENGRAM_EMBEDDING_MODEL` | Embedding provider. Values: `tfidf` (default, no API key needed), `openai` (requires `OPENAI_API_KEY`), `local` (local ONNX model, requires `ENGRAM_ONNX_MODEL_DIR`; build with `--features onnx-embed`), `clip` (multimodal; `--features multimodal`), `ollama` (`--features ollama`), `cohere` (`--features cohere`), `voyage` (`--features voyage`) | `tfidf` |
| `ENGRAM_ONNX_MODEL_DIR` | Path to directory containing `model.onnx` and `tokenizer.json`. Required when `ENGRAM_EMBEDDING_MODEL=local`. | — |
| `OPENAI_API_KEY` | Required for OpenAI embeddings | - |

---

---

## Using the SDKs

Engram ships Python and TypeScript clients that wrap the MCP HTTP transport.

### Installation

```bash
pip install engram-client        # Python
npm install engram-client        # TypeScript / Node
```

### Python quickstart

```python
import asyncio
from engram_client import EngramClient

async def main():
    async with EngramClient(
        base_url="http://localhost:3100",
        api_key="your-api-key",
        tenant="default",
    ) as client:
        # Create a memory
        mem = await client.create(
            "Discovered that the auth bug is in middleware ordering",
            tags=["bug", "auth"],
            importance=0.8,
        )

        # Search memories
        results = await client.search("auth middleware", limit=5)
        for r in results:
            print(r["content"])

        # Find related memories
        related = await client.related(mem["id"], limit=3)

        # Create a daily note
        await client.create_daily("Debugging session: fixed auth ordering issue")

asyncio.run(main())
```

### TypeScript quickstart

```typescript
import { EngramClient } from 'engram-client';

const client = new EngramClient({
  baseUrl: 'http://localhost:3100',
  apiKey: 'your-api-key',
  tenant: 'default',
});

// Create a memory
const mem = await client.create(
  'Discovered that the auth bug is in middleware ordering',
  { tags: ['bug', 'auth'], importance: 0.8 },
);

// Search memories
const results = await client.search('auth middleware', { limit: 5 });
results.forEach(r => console.log(r.content));

// Find related memories
const related = await client.related(mem.id, { limit: 3 });

// Create a daily note
await client.createDaily('Debugging session: fixed auth ordering issue');
```

---

## Next Steps

- Read the full [README](../README.md) for feature details
- See [AGENTS.md](../AGENTS.md) for the complete MCP tool reference
- Check [SCHEMA.md](SCHEMA.md) for database schema details
- Explore the [architecture overview](../README.md#architecture) to understand how components fit together
