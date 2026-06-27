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

### `EngramClient(base_url, api_key, tenant, timeout=30.0)`

#### Memory CRUD

| Method | Description |
|--------|-------------|
| `create(content, **kwargs)` | Create a memory |
| `get(id)` | Get memory by ID |
| `update(id, **kwargs)` | Update a memory |
| `delete(id)` | Delete a memory |
| `list(**kwargs)` | List memories with filters |
| `create_daily(content, **kwargs)` | Create a daily memory that auto-expires |

#### Search

| Method | Description |
|--------|-------------|
| `search(query, **kwargs)` | Hybrid search (BM25 + vector + fuzzy) |

#### Graph

| Method | Description |
|--------|-------------|
| `related(memory_id)` | Get related memories via knowledge graph |
| `link(from_id, to_id, edge_type="related_to")` | Create a link between two memories |
| `detect_conflicts(**kwargs)` | Detect conflicting or contradictory memories |
| `resolve_conflict(conflict_id, resolution)` | Resolve a detected memory conflict |
| `coactivation_report(**kwargs)` | Report frequently co-accessed memories |
| `query_triplets(**kwargs)` | Query knowledge graph triplets |
| `add_knowledge(subject, predicate, object, **kwargs)` | Add a knowledge triplet |

#### Temporal Graph

| Method | Description |
|--------|-------------|
| `temporal_create(from_entity, to_entity, relation, **kwargs)` | Create a time-bounded edge |
| `temporal_invalidate(edge_id, **kwargs)` | Mark a temporal edge as invalid |
| `temporal_snapshot(**kwargs)` | Get graph snapshot at a point in time |
| `temporal_contradictions(**kwargs)` | Find temporal contradictions |
| `temporal_evolve(entity)` | Trace entity relationship evolution |

#### Scopes & Access Control

| Method | Description |
|--------|-------------|
| `scope_set(memory_id, scope_path)` | Assign a scope path to a memory |
| `scope_get(memory_id)` | Get the scope path of a memory |
| `scope_list(scope_path, **kwargs)` | List memories in a scope |
| `scope_inherit(scope_path, parent_path)` | Inherit settings from a parent scope |
| `scope_isolate(scope_path)` | Isolate a scope from its parent |
| `grant_access(agent_id, scope_path, **kwargs)` | Grant agent access to a scope |
| `revoke_access(agent_id, scope_path)` | Revoke agent access |
| `list_grants(agent_id)` | List all scope grants for an agent |
| `check_access(agent_id, scope_path, **kwargs)` | Check agent permission on a scope |

#### Identity

| Method | Description |
|--------|-------------|
| `create_identity(canonical_id, display_name, **kwargs)` | Create or update an identity |
| `resolve_identity(alias)` | Resolve an alias to its canonical identity |

#### Council & Consensus

| Method | Description |
|--------|-------------|
| `memory_council(prompt, **kwargs)` | Run a prompt through llm-council |

#### Temporal Replay

| Method | Description |
|--------|-------------|
| `memory_replay_at_time(memory_id, timestamp, **kwargs)` | Replay memory state at a timestamp |

#### Agentic Evolution

| Method | Description |
|--------|-------------|
| `detect_updates(memory_id)` | Detect whether a memory may be outdated |
| `utility_score(memory_id, **kwargs)` | Compute or update memory utility score |
| `sentiment_analyze(memory_id)` | Run sentiment analysis on a memory |
| `sentiment_timeline(**kwargs)` | Retrieve sentiment scores over time |
| `reflect(memory_id)` | Trigger self-reflection on a memory |

#### Autonomous Agent

| Method | Description |
|--------|-------------|
| `agent_start(**kwargs)` | Start the autonomous memory gardening agent |
| `agent_stop()` | Stop the autonomous agent |
| `agent_status()` | Get current agent status |
| `agent_metrics()` | Get agent performance metrics |
| `agent_configure(config)` | Configure the autonomous agent |

#### Gardening

| Method | Description |
|--------|-------------|
| `garden(**kwargs)` | Run one gardening cycle (prune, merge, promote) |
| `garden_preview(**kwargs)` | Preview gardening without applying changes |
| `garden_undo(operation_id)` | Undo a previous gardening operation |
| `suggest_acquisition(**kwargs)` | Suggest topics to acquire knowledge about |
| `proactive_scan(**kwargs)` | Scan for gaps, staleness, or improvements |

#### Compression & Consolidation

| Method | Description |
|--------|-------------|
| `compress(memory_id)` | Compress a memory to reduce token footprint |
| `decompress(memory_id)` | Decompress a previously compressed memory |
| `compress_for_context(memory_ids, token_budget)` | Compress memories to fit a token budget |
| `consolidate(workspace, **kwargs)` | Consolidate similar memories |
| `synthesis(memory_ids)` | Synthesize multiple memories into one |

#### Cache & Embeddings

| Method | Description |
|--------|-------------|
| `cache_stats()` | Get cache statistics |
| `cache_clear(**kwargs)` | Clear the embedding and search cache |
| `embedding_providers()` | List available embedding providers |
| `embedding_migrate(**kwargs)` | Migrate embeddings between providers |

#### Retrieval Feedback

| Method | Description |
|--------|-------------|
| `explain_search(results)` | Explain why search results were returned |
| `feedback(query, memory_id, signal)` | Record relevance feedback |
| `feedback_stats(**kwargs)` | Get aggregated feedback statistics |

#### Context Engineering

| Method | Description |
|--------|-------------|
| `extract_facts(memory_id)` | Extract atomic facts from a memory |
| `list_facts(**kwargs)` | List extracted facts |
| `fact_graph(**kwargs)` | Export fact graph and relationships |
| `build_context(query, **kwargs)` | Build optimised context window for LLM |
| `prompt_template(template_name, **kwargs)` | Render a named prompt template |
| `token_estimate(content)` | Estimate token count for content |

#### Memory Blocks

| Method | Description |
|--------|-------------|
| `block_get(block_type, label, **kwargs)` | Retrieve a named memory block |
| `block_edit(block_type, label, content, **kwargs)` | Edit a memory block |
| `block_list(**kwargs)` | List memory blocks |
| `block_create(block_type, label, content, **kwargs)` | Create a named memory block |

#### Multimodal

| Method | Description |
|--------|-------------|
| `search_by_image(image_path, **kwargs)` | Find memories similar to an image |
| `sync_media(**kwargs)` | Upload local media assets to cloud storage |

#### Stats

| Method | Description |
|--------|-------------|
| `stats()` | Get memory statistics |

#### Federation

| Method | Description |
|--------|-------------|
| `federation_add_peer(url, api_key, **kwargs)` | Register a remote Engram instance |
| `federation_remove_peer(peer_id)` | Remove a federation peer |
| `federation_list_peers()` | List all federation peers |
| `federation_search(query, **kwargs)` | Search across federation peers |
| `federation_share(memory_id, peer_id)` | Share a memory with a peer |
| `federation_sync_status()` | Get federation sync status |

### Key Parameters

**create kwargs:** `memory_type`, `tags`, `workspace`, `metadata`, `importance`, `media_url`

**update kwargs:** `content`, `tags`, `metadata`, `importance`, `media_url`

**list kwargs:** `limit`, `offset`, `workspace`, `memory_type`, `tags`, `filter_`, `sort_by`, `sort_order`

**search kwargs:** `limit`, `workspace`, `filter_`

`filter_` is sent to the MCP API as `filter` and supports AND/OR and comparison-operator syntax:

```python
await client.list(filter_={
    "AND": [
        {"importance": {"gte": 0.8}},
        {"metadata.project": {"eq": "engram"}},
    ]
})
```

Supported operators: `eq`, `neq`, `gt`, `gte`, `lt`, `lte`, `contains`, `not_contains`, `exists`.

**memory_council kwargs:** `conversation_id`, `council_url`, `timeout_seconds`, `include_raw_stages`, `persist`, `workspace`, `memory_tags`

## Development

```bash
pip install -e ".[dev]"
pytest tests/
```

## Requirements

- Python >= 3.10
- httpx >= 0.25.0

## Related

- [Engram](https://github.com/aiconnai/engram) - Core memory engine (Rust)
- [Engram Cloud](https://github.com/aiconnai/engram-cloud) - Multi-tenant SaaS gateway
- [engram-client](https://www.npmjs.com/package/engram-client) - TypeScript client

## License

MIT
