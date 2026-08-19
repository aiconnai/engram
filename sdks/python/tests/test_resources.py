"""Tests for modular resource mixins and backward compatibility."""

from __future__ import annotations

import inspect
import pytest
from unittest.mock import AsyncMock

from engram_client import EngramClient
from engram_client.resources import (
    AuthMixin,
    ContextMixin,
    DreamMixin,
    GraphMixin,
    MemoriesMixin,
    ResourceMixin,
    SearchMixin,
)
from engram_client.resources.auth import AuthMixin as DirectAuthMixin
from engram_client.resources.context import ContextMixin as DirectContextMixin
from engram_client.resources.dream import DreamMixin as DirectDreamMixin
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
    assert DreamMixin is DirectDreamMixin
    assert GraphMixin is DirectGraphMixin
    assert MemoriesMixin is DirectMemoriesMixin
    assert SearchMixin is DirectSearchMixin


def test_engram_client_inheritance():
    """Verify EngramClient inherits from all resource mixins."""
    assert issubclass(EngramClient, MemoriesMixin)
    assert issubclass(EngramClient, SearchMixin)
    assert issubclass(EngramClient, DreamMixin)
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


@pytest.mark.asyncio
async def test_dream_mixin_methods(mock_client):
    """Test DreamMixin dispatch methods."""
    mock_client._mcp_call = AsyncMock(return_value={"status": "ok"})

    await mock_client.dream_create(workspace="ws", instructions="Consolidate")
    mock_client._mcp_call.assert_awaited_with(
        "dream_create",
        {
            "workspace": "ws",
            "run": True,
            "instructions": "Consolidate",
            "max_memories": 50,
            "max_candidates": 25,
            "summary_min_memories": 2,
        },
    )

    await mock_client.dream_get("job-123")
    mock_client._mcp_call.assert_awaited_with("dream_get", {"id": "job-123"})

    await mock_client.dream_list(status="completed", limit=10)
    mock_client._mcp_call.assert_awaited_with(
        "dream_list", {"status": "completed", "limit": 10}
    )

    await mock_client.dream_cancel("job-123")
    mock_client._mcp_call.assert_awaited_with("dream_cancel", {"id": "job-123"})

    await mock_client.dream_archive("job-123")
    mock_client._mcp_call.assert_awaited_with("dream_archive", {"id": "job-123"})

    await mock_client.dream_candidates_list(review_state="pending")
    mock_client._mcp_call.assert_awaited_with(
        "dream_candidates_list", {"review_state": "pending"}
    )

    await mock_client.dream_candidate_get("cand-1")
    mock_client._mcp_call.assert_awaited_with("dream_candidate_get", {"id": "cand-1"})

    await mock_client.dream_candidate_review("cand-1", "accepted", notes="Good")
    mock_client._mcp_call.assert_awaited_with(
        "dream_candidate_review",
        {"id": "cand-1", "review_state": "accepted", "notes": "Good"},
    )

    await mock_client.dream_candidate_apply("cand-1", confirm=True)
    mock_client._mcp_call.assert_awaited_with(
        "dream_candidate_apply", {"id": "cand-1", "confirm": True}
    )

    await mock_client.dream_eval_run(lane="freshness_temporal")
    mock_client._mcp_call.assert_awaited_with(
        "dream_eval_run", {"lane": "freshness_temporal"}
    )

    await mock_client.dream_run_now(workspace="ws")
    mock_client._mcp_call.assert_awaited_with("dream_run_now", {"workspace": "ws"})


@pytest.mark.asyncio
async def test_search_digest(mock_client):
    """Test SearchMixin.digest method."""
    mock_client._mcp_call = AsyncMock(return_value={"topic": "auth", "digest": {}})

    res = await mock_client.digest(
        "authentication rules",
        workspace="prod",
        mode="standard",
        limit=15,
        related_depth=1,
    )
    mock_client._mcp_call.assert_awaited_with(
        "memory_digest",
        {
            "topic": "authentication rules",
            "workspace": "prod",
            "mode": "standard",
            "limit": 15,
            "related_depth": 1,
        },
    )
    assert res == {"topic": "auth", "digest": {}}


def test_parse_sse_event(mock_client):
    """Test parse_sse_event helper."""
    raw = 'id: 99\nevent: progress\ndata: {"progress_token":"pt-1","progress":3,"total":5,"message":"Step 3"}'
    event = mock_client.parse_sse_event(raw)
    assert event is not None
    assert event["seq_id"] == 99
    assert event["type"] == "progress"
    assert event["data"]["progress_token"] == "pt-1"
    assert event["data"]["progress"] == 3
    assert event["data"]["total"] == 5

    # Empty or invalid
    assert mock_client.parse_sse_event("") is None
    assert mock_client.parse_sse_event("data: invalid-json") is None


def test_events_mixin_attributes(mock_client):
    """Test that stream_events and watch_progress are exposed."""
    assert hasattr(mock_client, "stream_events")
    assert hasattr(mock_client, "watch_progress")
    assert hasattr(mock_client, "parse_sse_event")


@pytest.mark.asyncio
async def test_mcp_resources_mixin_methods(mock_client):
    """Test McpResourcesMixin dispatch methods."""
    mock_client._mcp_call = AsyncMock(return_value={"status": "ok"})

    await mock_client.resource_list()
    mock_client._mcp_call.assert_awaited_with("resources/list")

    await mock_client.resource_read("engram://stats")
    mock_client._mcp_call.assert_awaited_with("resources/read", {"uri": "engram://stats"})

    await mock_client.resource_subscribe("engram://workspace/dev/memories")
    mock_client._mcp_call.assert_awaited_with(
        "resources/subscribe", {"uri": "engram://workspace/dev/memories"}
    )

    await mock_client.resource_unsubscribe("engram://workspace/dev/memories")
    mock_client._mcp_call.assert_awaited_with(
        "resources/unsubscribe", {"uri": "engram://workspace/dev/memories"}
    )


@pytest.mark.asyncio
async def test_multimodal_mixin_methods(mock_client):
    """Test MultimodalMixin dispatch methods."""
    mock_client._mcp_call = AsyncMock(return_value={"status": "ok"})

    await mock_client.describe_image("/path/to/img.png", prompt="Describe this")
    mock_client._mcp_call.assert_awaited_with(
        "memory_describe_image", {"image_path": "/path/to/img.png", "prompt": "Describe this"}
    )

    await mock_client.transcribe_audio("/path/to/voice.mp3")
    mock_client._mcp_call.assert_awaited_with(
        "memory_transcribe_audio", {"audio_path": "/path/to/voice.mp3"}
    )

    await mock_client.capture_screenshot(display_index=1, delay_seconds=2)
    mock_client._mcp_call.assert_awaited_with(
        "memory_capture_screenshot", {"display_index": 1, "delay_seconds": 2}
    )

    await mock_client.process_video("/path/to/vid.mp4", max_frames=5)
    mock_client._mcp_call.assert_awaited_with(
        "memory_process_video",
        {"video_path": "/path/to/vid.mp4", "extract_frames": True, "max_frames": 5},
    )

    await mock_client.list_media(media_type="image", limit=25)
    mock_client._mcp_call.assert_awaited_with(
        "memory_list_media", {"media_type": "image", "limit": 25}
    )

    await mock_client.search_by_image("/path/to/img.png", limit=5, workspace="dev")
    mock_client._mcp_call.assert_awaited_with(
        "memory_search_by_image",
        {"image_path": "/path/to/img.png", "limit": 5, "workspace": "dev"},
    )

    await mock_client.ingest_media(
        "/path/to/chart.png", media_type="image", workspace="analytics"
    )
    mock_client._mcp_call.assert_awaited_with(
        "memory_ingest_media",
        {"media_path": "/path/to/chart.png", "media_type": "image", "workspace": "analytics"},
    )

    await mock_client.sync_media(dry_run=True)
    mock_client._mcp_call.assert_awaited_with("memory_sync_media", {"dry_run": True})


