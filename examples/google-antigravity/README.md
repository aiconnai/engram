# Google Antigravity Memory Integration

This example demonstrates how **Google Antigravity** agents, subagents, and lifecycle hooks leverage **Engram** for persistent memory, decision tracking, and context distillation across coding tasks.

## Why Engram with Google Antigravity?

1. **Cross-Session Memory**: Preserve architectural decisions, technical constraints, and user preferences across distinct Antigravity conversations.
2. **Subagent Knowledge Sharing**: Multiple specialized subagents (e.g. `Codebase Researcher`, `Database Debugger`) read and write to the same structured workspace memory in `<0.8ms`.
3. **Context Distillation (`memory_digest`)**: Synthesize long memory timelines into compact, actionable context blocks before initiating major refactors.
4. **Local-First & Secure**: SQLite WAL storage with zero external cloud dependencies or telemetry.

---

## Configuration

In your workspace `.gemini/` directory or global Antigravity configuration:

```json
{
  "mcpServers": {
    "engram": {
      "command": "engram-server",
      "args": ["--transport", "stdio"],
      "env": {
        "ENGRAM_DB_PATH": "~/.local/share/engram/memories.db",
        "ENGRAM_TOOL_TIER": "standard",
        "ENGRAM_EMBEDDING_MODEL": "local"
      }
    }
  }
}
```

---

## Programmatic Usage (Python SDK)

Run the included standalone example:

```bash
python3 antigravity_memory_hook.py
```

### Key Workflow:

```python
import asyncio
from engram_client import EngramClient

async def main():
    async with EngramClient(
        base_url="http://localhost:8080",
        api_key="dev-token",
        tenant="default"
    ) as client:
        # 1. Store a decision made during an Antigravity planning phase
        mem = await client.create(
            content="Database migration: Sized connection pool to 25 with 5s timeout",
            workspace="backend",
            memory_type="decision",
        )
        print(f"Stored Antigravity Decision: {mem.id}")

        # 2. Retrieve relevant context for a new subagent
        results = await client.search(
            query="connection pool size",
            workspace="backend",
        )
        print(f"Top Match: {results.memories[0].content}")

        # 3. Synthesize an actionable digest for session bootstrap
        digest = await client.digest(
            topic="Database configuration",
            workspace="backend",
            limit=5,
        )
        print(f"Synthesized Digest:\n{digest.get('digest')}")

if __name__ == "__main__":
    asyncio.run(main())
```

