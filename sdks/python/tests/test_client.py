"""Tests for EngramClient - Direct HTTP mock tests."""

from __future__ import annotations

import os
import sys
from unittest.mock import AsyncMock, MagicMock, patch

import httpx
import pytest

# Add parent directory to path for imports
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from engram_client.client import EngramClient, EngramError
from engram_client.integrations.council import CouncilSkill


@pytest.fixture
def mock_client():
    """Create a client with mocked httpx.AsyncClient."""
    client = EngramClient(
        base_url="https://test.engram.dev",
        api_key="test-key",
        tenant="test-tenant",
        timeout=10.0,
    )
    # Mock the _client attribute
    client._client = AsyncMock(spec=httpx.AsyncClient)
    return client


@pytest.fixture
def mock_response():
    """Create a mock httpx response."""
    response = MagicMock()
    response.status_code = 200
    response.text = "OK"
    response.raise_for_status.return_value = None
    response.json.return_value = {
        "jsonrpc": "2.0",
        "id": 1,
        "result": {"id": 123, "content": "Test memory", "memory_type": "note"},
    }
    return response


class TestEngramClientInit:
    """Test client initialization."""

    def test_init_stores_correct_values(self):
        client = EngramClient("https://example.com", "key123", "tenant1")
        assert client.base_url == "https://example.com"
        assert client.tenant == "tenant1"
        assert client._id_counter is not None

    def test_init_strips_trailing_slash(self):
        client = EngramClient("https://example.com/", "key", "tenant")
        assert client.base_url == "https://example.com"


class TestEngramClientContextManager:
    """Test async context manager behavior."""

    @pytest.mark.asyncio
    async def test_aenter_returns_self(self):
        client = EngramClient("https://example.com", "key", "tenant")
        with patch.object(client, "_client", AsyncMock()):
            result = await client.__aenter__()
            assert result is client

    @pytest.mark.asyncio
    async def test_aexit_closes_client(self):
        client = EngramClient("https://example.com", "key", "tenant")
        mock_http_client = AsyncMock()
        client._client = mock_http_client
        await client.__aexit__(None, None, None)
        mock_http_client.aclose.assert_called_once()


class TestMCPCall:
    """Test the _mcp_call helper method."""

    @pytest.mark.asyncio
    async def test_mcp_call_success(self, mock_client, mock_response):
        mock_client._client.post.return_value = mock_response

        result = await mock_client._mcp_call("memory_create", {"content": "test"})

        assert result["id"] == 123
        assert result["content"] == "Test memory"
        mock_client._client.post.assert_called_once_with(
            "/v1/mcp",
            json={
                "jsonrpc": "2.0",
                "id": 1,
                "method": "tools/call",
                "params": {
                    "name": "memory_create",
                    "arguments": {"content": "test"},
                },
            },
        )

    @pytest.mark.asyncio
    async def test_mcp_call_http_error(self, mock_client):
        request = httpx.Request("POST", "https://test.engram.dev/v1/mcp")
        mock_response = httpx.Response(404, text="Not Found", request=request)
        mock_client._client.post.return_value = mock_response

        with pytest.raises(EngramError, match="HTTP 404"):
            await mock_client._mcp_call("memory_get", {"id": 999})

    @pytest.mark.asyncio
    async def test_mcp_call_jsonrpc_error(self, mock_client):
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "jsonrpc": "2.0",
            "id": 1,
            "error": {"message": "Invalid params", "code": -32602},
        }
        mock_client._client.post.return_value = mock_response

        with pytest.raises(EngramError, match="Invalid params"):
            await mock_client._mcp_call("memory_get", {"id": "invalid"})

    @pytest.mark.asyncio
    async def test_mcp_call_increments_id(self, mock_client, mock_response):
        mock_client._client.post.return_value = mock_response

        await mock_client._mcp_call("test", {})
        await mock_client._mcp_call("test", {})
        await mock_client._mcp_call("test", {})

        # Check that post was called 3 times with different IDs
        calls = mock_client._client.post.call_args_list
        ids = [call.kwargs["json"]["id"] for call in calls]
        assert ids == [1, 2, 3]


class TestCreate:
    """Test memory creation."""

    @pytest.mark.asyncio
    async def test_create_basic(self, mock_client, mock_response):
        mock_client._client.post.return_value = mock_response

        result = await mock_client.create("Hello world")

        assert result["id"] == 123
        call_args = mock_client._client.post.call_args.kwargs["json"]["params"][
            "arguments"
        ]
        assert call_args["content"] == "Hello world"
        assert call_args["memory_type"] == "note"  # default

    @pytest.mark.asyncio
    async def test_create_with_all_params(self, mock_client, mock_response):
        mock_client._client.post.return_value = mock_response

        result = await mock_client.create(
            "Test content",
            memory_type="image",
            tags=["test", "example"],
            workspace="my-workspace",
            metadata={"source": "test"},
            importance=0.8,
            media_url="https://example.com/image.jpg",
        )

        args = mock_client._client.post.call_args.kwargs["json"]["params"]["arguments"]
        assert args["content"] == "Test content"
        assert args["memory_type"] == "image"
        assert args["tags"] == ["test", "example"]
        assert args["workspace"] == "my-workspace"
        assert args["metadata"] == {"source": "test"}
        assert args["importance"] == 0.8
        assert args["media_url"] == "https://example.com/image.jpg"


class TestList:
    """Test memory listing."""

    @pytest.mark.asyncio
    async def test_list_default_params(self, mock_client, mock_response):
        mock_client._client.post.return_value = mock_response

        await mock_client.list()

        args = mock_client._client.post.call_args.kwargs["json"]["params"]["arguments"]
        assert args["limit"] == 50
        assert args["offset"] == 0
        assert "filter_" not in args  # Should be mapped to "filter" in API

    @pytest.mark.asyncio
    async def test_list_with_filter_(self, mock_client, mock_response):
        mock_client._client.post.return_value = mock_response

        filter_dict = {"field": "value"}
        await mock_client.list(filter_=filter_dict)

        args = mock_client._client.post.call_args.kwargs["json"]["params"]["arguments"]
        assert args["filter"] == filter_dict  # Mapped to "filter" for API


class TestSearch:
    """Test memory search."""

    @pytest.mark.asyncio
    async def test_search_basic(self, mock_client, mock_response):
        mock_client._client.post.return_value = mock_response

        await mock_client.search("test query")

        args = mock_client._client.post.call_args.kwargs["json"]["params"]["arguments"]
        assert args["query"] == "test query"
        assert args["limit"] == 10

    @pytest.mark.asyncio
    async def test_search_with_filter_(self, mock_client, mock_response):
        mock_client._client.post.return_value = mock_response

        filter_dict = {"workspace": "test"}
        await mock_client.search("query", filter_=filter_dict)

        args = mock_client._client.post.call_args.kwargs["json"]["params"]["arguments"]
        assert args["filter"] == filter_dict


class TestMemoryCouncil:
    """Test council orchestration helper."""

    @pytest.mark.asyncio
    async def test_memory_council_basic(self, mock_client, mock_response):
        mock_client._client.post.return_value = mock_response

        await mock_client.memory_council("What is the plan?")

        args = mock_client._client.post.call_args.kwargs["json"]["params"]["arguments"]
        assert args["prompt"] == "What is the plan?"
        assert args["include_raw_stages"] is True
        assert args["persist"] is False

    @pytest.mark.asyncio
    async def test_memory_council_with_options(self, mock_client, mock_response):
        mock_client._client.post.return_value = mock_response

        await mock_client.memory_council(
            "What should we do?",
            conversation_id="conv-1",
            council_url="http://127.0.0.1:8001",
            timeout_seconds=120,
            include_raw_stages=False,
            persist=True,
            workspace="project-a",
            memory_tags=["llm", "consensus"],
        )

        args = mock_client._client.post.call_args.kwargs["json"]["params"]["arguments"]
        assert args["conversation_id"] == "conv-1"
        assert args["council_url"] == "http://127.0.0.1:8001"
        assert args["timeout_seconds"] == 120
        assert args["include_raw_stages"] is False
        assert args["persist"] is True
        assert args["workspace"] == "project-a"
        assert args["memory_tags"] == ["llm", "consensus"]


class TestCouncilSkill:
    """Test council skill helper wrapper."""

    @pytest.mark.asyncio
    async def test_council_skill_ask_uses_defaults(self, mock_client):
        mock_client.memory_council = AsyncMock(return_value={"result": "ok"})

        skill = CouncilSkill(
            mock_client,
            default_workspace="project-a",
            default_timeout_seconds=120,
            default_include_raw_stages=True,
        )

        await skill.ask("How should we proceed?")

        mock_client.memory_council.assert_awaited_once_with(
            "How should we proceed?",
            conversation_id=None,
            council_url=None,
            timeout_seconds=120,
            include_raw_stages=True,
            persist=False,
            workspace="project-a",
            memory_tags=None,
        )

    @pytest.mark.asyncio
    async def test_council_skill_ask_with_override_and_persistence(
        self,
        mock_client,
    ):
        mock_client.memory_council = AsyncMock(return_value={"result": "ok"})

        skill = CouncilSkill(mock_client)
        await skill.ask_with_persistence(
            "  summarize last meeting  ",
            workspace="planning",
            timeout_seconds=45,
            include_raw_stages=False,
            conversation_id="conv-42",
            council_url="http://127.0.0.1:8001",
            tags=("decisions", "architecture"),
        )

        mock_client.memory_council.assert_awaited_once_with(
            "  summarize last meeting  ",
            conversation_id="conv-42",
            council_url="http://127.0.0.1:8001",
            timeout_seconds=45,
            include_raw_stages=False,
            persist=True,
            workspace="planning",
            memory_tags=["decisions", "architecture"],
        )

    @pytest.mark.asyncio
    async def test_council_skill_rejects_empty_prompt(self, mock_client):
        mock_client.memory_council = AsyncMock(return_value={"result": "ok"})

        skill = CouncilSkill(mock_client)
        result = await skill.ask("  ")

        assert result == {"error": "prompt must be a non-empty string"}
        mock_client.memory_council.assert_not_awaited()


class TestGetUpdateDelete:
    """Test get, update, and delete operations."""

    @pytest.mark.asyncio
    async def test_get(self, mock_client, mock_response):
        mock_client._client.post.return_value = mock_response

        await mock_client.get(123)

        args = mock_client._client.post.call_args.kwargs["json"]["params"]["arguments"]
        assert args["id"] == 123

    @pytest.mark.asyncio
    async def test_update(self, mock_client, mock_response):
        mock_client._client.post.return_value = mock_response

        await mock_client.update(123, content="Updated content")

        args = mock_client._client.post.call_args.kwargs["json"]["params"]["arguments"]
        assert args["id"] == 123
        assert args["content"] == "Updated content"

    @pytest.mark.asyncio
    async def test_delete(self, mock_client, mock_response):
        mock_client._client.post.return_value = mock_response

        await mock_client.delete(123)

        args = mock_client._client.post.call_args.kwargs["json"]["params"]["arguments"]
        assert args["id"] == 123


class TestClose:
    """Test client close behavior."""

    @pytest.mark.asyncio
    async def test_close_when_client_exists(self):
        client = EngramClient("https://example.com", "key", "tenant")
        mock_http_client = AsyncMock()
        client._client = mock_http_client

        await client.close()

        mock_http_client.aclose.assert_called_once()
        assert client._client is None

    @pytest.mark.asyncio
    async def test_close_when_no_client(self):
        client = EngramClient("https://example.com", "key", "tenant")
        client._client = None

        # Should not raise
        await client.close()


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
