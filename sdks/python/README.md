# engram-client

[![PyPI](https://img.shields.io/pypi/v/engram-client)](https://pypi.org/project/engram-client/)
[![Python](https://img.shields.io/pypi/pyversions/engram-client)](https://pypi.org/project/engram-client/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Python client for [Engram Cloud](https://github.com/aiconnai/engram-cloud) - AI memory infrastructure for agents.

## Installation

```bash
pip install engram-client
```

## Quick Start

```python
from engram_client import EngramClient

client = EngramClient(
    base_url="https://your-engram-api.fly.dev",
    api_key="ek_...",
    tenant="my-tenant",
)

# Create a memory
memory = client.create(
    "User prefers dark mode",
    tags=["preferences", "ui"],
    workspace="my-project",
)

# Search (hybrid: BM25 + vector + fuzzy)
results = client.search("user preferences")

# Run a council consensus session
import asyncio


async def example() -> None:
    async with EngramClient(base_url="https://your-engram-api.fly.dev", api_key="ek_...", tenant="my-tenant") as client:
        await client.create("User prefers dark mode")

        council = await client.memory_council(
            "Should we use Redis or Postgres for caching?",
            timeout_seconds=120,
            persist=True,
            workspace="architecture",
        )
        print(council)


asyncio.run(example())
```

### Council Skill (reusable in projects)

```python
import asyncio
from engram_client import EngramClient
from engram_client.integrations import CouncilSkill


async def example() -> None:
    async with EngramClient(
        base_url="https://your-engram-api.fly.dev",
        api_key="ek_...",
        tenant="my-tenant",
    ) as client:
        skill = CouncilSkill(
            client,
            default_workspace="my-project",
            default_timeout_seconds=120,
            default_include_raw_stages=True,
        )

        result = await skill.ask(
            "What is the best migration strategy?",
            persist=True,
        )
        print(result)


asyncio.run(example())
```

```python
# List with filters
memories = client.list(limit=20, workspace="my-project")

# Get by ID
memory = client.get(42)

# Update
client.update(42, content="User prefers light mode", tags=["preferences"])

# Delete
client.delete(42)

# Stats
stats = client.stats()
```

## Context Manager

```python
with EngramClient(base_url="...", api_key="...", tenant="...") as client:
    client.create("Hello from Python SDK")
```

## API Reference

### `EngramClient(base_url, api_key, tenant)`

| Method | Description |
|--------|-------------|
| `create(content, **kwargs)` | Create a memory |
| `get(id)` | Get memory by ID |
| `update(id, **kwargs)` | Update a memory |
| `delete(id)` | Delete a memory |
| `list(**kwargs)` | List memories with filters |
| `search(query, **kwargs)` | Hybrid search |
| `memory_council(prompt, **kwargs)` | Run a prompt through llm-council |
| `memory_replay_at_time(memory_id, timestamp, **kwargs)` | Replay memory state at a timestamp |
| `stats()` | Storage statistics |

### Parameters

**create / update kwargs:** `tags`, `workspace`, `memory_type`, `importance`, `metadata`, `tier`

**list kwargs:** `limit`, `offset`, `workspace`, `memory_type`, `tags`, `sort_by`, `sort_order`

**search kwargs:** `limit`, `workspace`, `tags`, `memory_type`, `include_archived`
**memory_council kwargs:** `conversation_id`, `council_url`, `timeout_seconds`, `include_raw_stages`, `persist`, `workspace`, `memory_tags`

## Requirements

- Python >= 3.9
- httpx >= 0.25.0

## Related

- [Engram](https://github.com/aiconnai/engram) - Core memory engine (Rust)
- [Engram Cloud](https://github.com/aiconnai/engram-cloud) - Multi-tenant SaaS gateway
- [engram-client](https://www.npmjs.com/package/engram-client) - TypeScript client

## License

MIT
