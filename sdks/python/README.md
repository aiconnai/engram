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
import asyncio
from engram_client import EngramClient


async def example() -> None:
    async with EngramClient(
        base_url="https://your-engram-api.fly.dev",
        api_key="ek_...",
        tenant="my-tenant",
    ) as client:
        # Create a memory
        memory = await client.create(
            "User prefers dark mode",
            tags=["preferences", "ui"],
            workspace="my-project",
        )

        # Search (hybrid: BM25 + vector + fuzzy)
        results = await client.search("user preferences")

        # Run a council consensus session
        council = await client.memory_council(
            "Should we use Redis or Postgres for caching?",
            timeout_seconds=120,
            persist=True,
            workspace="architecture",
        )

        print(memory, results, council)


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
async with EngramClient(base_url="...", api_key="...", tenant="...") as client:
    # List with workspace and metadata filters
    memories = await client.list(
        limit=20,
        workspace="my-project",
        filter_={"metadata.source": {"eq": "support"}},
    )

    # Get by ID
    memory = await client.get(42)

    # Update content and multimodal media URL
    await client.update(
        42,
        content="User prefers light mode",
        tags=["preferences"],
        media_url="https://example.com/preference.png",
    )

    # Delete
    await client.delete(42)

    # Stats
    stats = await client.stats()
```

## Context Manager

```python
async with EngramClient(base_url="...", api_key="...", tenant="...") as client:
    await client.create("Hello from Python SDK")
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

**create kwargs:** `memory_type`, `tags`, `workspace`, `metadata`, `importance`, `media_url`

**update kwargs:** `content`, `tags`, `metadata`, `importance`, `media_url`

**list kwargs:** `limit`, `offset`, `workspace`, `memory_type`, `tags`, `filter_`, `sort_by`, `sort_order`

**search kwargs:** `limit`, `workspace`, `filter_`

`filter_` is sent to the MCP API as `filter` and supports the same AND/OR and comparison-operator syntax as the server.

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
