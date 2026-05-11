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
        mock_response = MagicMock()
        mock_response.status_code = 404
        mock_response.text = "Not Found"
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
