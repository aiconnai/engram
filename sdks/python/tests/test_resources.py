"""Tests for modular resource mixins and backward compatibility."""

from __future__ import annotations

import inspect
import pytest
from unittest.mock import AsyncMock

from engram_client import EngramClient
from engram_client.resources import (
    AuthMixin,
    ContextMixin,
    GraphMixin,
    MemoriesMixin,
    ResourceMixin,
    SearchMixin,
)
from engram_client.resources.auth import AuthMixin as DirectAuthMixin
from engram_client.resources.context import ContextMixin as DirectContextMixin
from engram_client.resources.graph import GraphMixin as DirectGraphMixin
from engram_client.resources.memories import MemoriesMixin as DirectMemoriesMixin
from engram_client.resources.search import SearchMixin as DirectSearchMixin


@pytest.fixture
def mock_client():
    """Create an EngramClient with mocked HTTP client."""
    client = EngramClient(
        base_url="https://test.engram.dev",
        api_key="test-key",
        tenant="test-tenant",
        timeout=10.0,
    )
    return client


def test_resource_imports():
    """Verify all resource mixins can be imported directly and from resources package."""
    assert AuthMixin is DirectAuthMixin
    assert ContextMixin is DirectContextMixin
    assert GraphMixin is DirectGraphMixin
    assert MemoriesMixin is DirectMemoriesMixin
    assert SearchMixin is DirectSearchMixin


def test_engram_client_inheritance():
    """Verify EngramClient inherits from all resource mixins."""
    assert issubclass(EngramClient, MemoriesMixin)
    assert issubclass(EngramClient, SearchMixin)
    assert issubclass(EngramClient, GraphMixin)
    assert issubclass(EngramClient, ContextMixin)
    assert issubclass(EngramClient, AuthMixin)
    assert issubclass(EngramClient, ResourceMixin)


def test_resource_mixin_not_implemented():
    """Verify base ResourceMixin._mcp_call raises NotImplementedError."""
    base = ResourceMixin()
    with pytest.raises(NotImplementedError):
        import asyncio
        asyncio.run(base._mcp_call("test"))


@pytest.mark.asyncio
async def test_memories_mixin_methods(mock_client):
    """Test MemoriesMixin dispatch methods."""
    mock_client._mcp_call = AsyncMock(return_value={"status": "ok"})

    await mock_client.create("test content", tags=["t1"])
    mock_client._mcp_call.assert_awaited_with(
        "memory_create",
        {"content": "test content", "memory_type": "note", "tags": ["t1"]},
    )

    await mock_client.get(42)
    mock_client._mcp_call.assert_awaited_with("memory_get", {"id": 42})

    await mock_client.update(42, content="new")
    mock_client._mcp_call.assert_awaited_with(
        "memory_update", {"id": 42, "content": "new"}
    )

    await mock_client.delete(42)
    mock_client._mcp_call.assert_awaited_with("memory_delete", {"id": 42})

    await mock_client.list(filter_={"status": "active"})
    mock_client._mcp_call.assert_awaited_with(
        "memory_list",
        {"limit": 50, "offset": 0, "filter": {"status": "active"}},
    )

    await mock_client.stats()
    mock_client._mcp_call.assert_awaited_with("memory_stats", {})

    await mock_client.compress(42)
    mock_client._mcp_call.assert_awaited_with("memory_compress", {"id": 42})

    await mock_client.decompress(42)
    mock_client._mcp_call.assert_awaited_with("memory_decompress", {"id": 42})

    await mock_client.garden(workspace="ws", dry_run=True)
    mock_client._mcp_call.assert_awaited_with(
        "memory_garden", {"workspace": "ws", "dry_run": True}
    )

    await mock_client.lifecycle_update(42, action="promote")
    mock_client._mcp_call.assert_awaited_with(
        "memory_lifecycle_update", {"id": 42, "action": "promote"}
    )


@pytest.mark.asyncio
async def test_search_mixin_methods(mock_client):
    """Test SearchMixin dispatch methods."""
    mock_client._mcp_call = AsyncMock(return_value={"status": "ok"})

    await mock_client.search("query string", limit=5)
    mock_client._mcp_call.assert_awaited_with(
        "memory_search", {"query": "query string", "limit": 5}
    )

    await mock_client.search_by_image("path/to/img.png")
    mock_client._mcp_call.assert_awaited_with(
        "memory_search_by_image", {"image_path": "path/to/img.png", "limit": 10}
    )

    await mock_client.cache_stats()
    mock_client._mcp_call.assert_awaited_with("memory_cache_stats", {})

    await mock_client.feedback("query", 42, "relevant")
    mock_client._mcp_call.assert_awaited_with(
        "memory_feedback", {"query": "query", "memory_id": 42, "signal": "relevant"}
    )

    await mock_client.federation_search("global query")
    mock_client._mcp_call.assert_awaited_with(
        "memory_federation_search", {"query": "global query", "limit": 10}
    )


@pytest.mark.asyncio
async def test_graph_mixin_methods(mock_client):
    """Test GraphMixin dispatch methods."""
    mock_client._mcp_call = AsyncMock(return_value={"status": "ok"})

    await mock_client.related(42)
    mock_client._mcp_call.assert_awaited_with("memory_related", {"id": 42})

    await mock_client.link(1, 2, "depends_on")
    mock_client._mcp_call.assert_awaited_with(
        "memory_link", {"from_id": 1, "to_id": 2, "edge_type": "depends_on"}
    )

    await mock_client.query_triplets(subject="Agent")
    mock_client._mcp_call.assert_awaited_with(
        "memory_query_triplets", {"subject": "Agent"}
    )

    await mock_client.temporal_create("A", "B", "relates")
    mock_client._mcp_call.assert_awaited_with(
        "memory_temporal_create",
        {"from_entity": "A", "to_entity": "B", "relation": "relates", "confidence": 1.0},
    )

    await mock_client.graph_query(action="relations", id=42)
    mock_client._mcp_call.assert_awaited_with(
        "graph_query", {"action": "relations", "id": 42}
    )


@pytest.mark.asyncio
async def test_context_mixin_methods(mock_client):
    """Test ContextMixin dispatch methods."""
    mock_client._mcp_call = AsyncMock(return_value={"status": "ok"})

    await mock_client.extract_facts(42)
    mock_client._mcp_call.assert_awaited_with("memory_extract_facts", {"id": 42})

    await mock_client.build_context("user query", token_budget=2048)
    mock_client._mcp_call.assert_awaited_with(
        "memory_build_context",
        {"query": "user query", "strategy": "balanced", "token_budget": 2048},
    )

    await mock_client.token_estimate("some text")
    mock_client._mcp_call.assert_awaited_with(
        "memory_token_estimate", {"content": "some text"}
    )

    await mock_client.block_get("persona", "user_profile")
    mock_client._mcp_call.assert_awaited_with(
        "memory_block_get", {"block_type": "persona", "label": "user_profile"}
    )


@pytest.mark.asyncio
async def test_auth_mixin_methods(mock_client):
    """Test AuthMixin dispatch methods."""
    mock_client._mcp_call = AsyncMock(return_value={"status": "ok"})

    await mock_client.create_identity("user-1", "User One")
    mock_client._mcp_call.assert_awaited_with(
        "identity_create", {"canonical_id": "user-1", "display_name": "User One"}
    )

    await mock_client.resolve_identity("alias-1")
    mock_client._mcp_call.assert_awaited_with(
        "identity_resolve", {"alias": "alias-1"}
    )

    await mock_client.scope_set(42, "tenant/workspace/team")
    mock_client._mcp_call.assert_awaited_with(
        "memory_scope_set", {"id": 42, "scope_path": "tenant/workspace/team"}
    )

    await mock_client.check_access("agent-1", "tenant/workspace", permission="write")
    mock_client._mcp_call.assert_awaited_with(
        "memory_check_access",
        {"agent_id": "agent-1", "scope_path": "tenant/workspace", "permission": "write"},
    )
