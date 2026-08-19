"""Engram Cloud HTTP client."""

from __future__ import annotations

from itertools import count
from typing import Any

import httpx

from .errors import EngramError
from .mcp_result import post_tool_call
from .resources.auth import AuthMixin
from .resources.context import ContextMixin
from .resources.dream import DreamMixin
from .resources.events import EventsMixin
from .resources.graph import GraphMixin
from .resources.memories import MemoriesMixin
from .resources.resources import McpResourcesMixin
from .resources.search import SearchMixin

# Re-export for callers that historically imported from client.
__all__ = ["EngramClient", "EngramError"]


class EngramClient(
    MemoriesMixin,
    SearchMixin,
    DreamMixin,
    EventsMixin,
    GraphMixin,
    ContextMixin,
    AuthMixin,
    McpResourcesMixin,
):
    """Async Engram Cloud client over authenticated MCP-HTTP."""

    def __init__(
        self,
        base_url: str,
        api_key: str,
        tenant: str,
        timeout: float = 30.0,
    ):
        self.base_url = base_url.rstrip("/")
        self.tenant = tenant
        self._client: httpx.AsyncClient | None = httpx.AsyncClient(
            base_url=self.base_url,
            headers={
                "Authorization": f"Bearer {api_key}",
                "X-Tenant-Slug": tenant,
                "Content-Type": "application/json",
            },
            timeout=timeout,
        )
        self._id_counter = count(1)  # JSON-RPC id (thread/coroutine-safe)

    async def close(self) -> None:
        client = self._client
        if client is None:
            return
        await client.aclose()
        self._client = None

    async def __aenter__(self) -> "EngramClient":
        return self

    async def __aexit__(self, *args: Any) -> None:
        await self.close()

    async def _mcp_call(
        self, method: str, params: dict[str, Any] | None = None
    ) -> Any:
        """Execute an MCP tool call over HTTP."""
        client = self._client
        if client is None:
            raise EngramError("EngramClient is closed")
        return await post_tool_call(
            client,
            request_id=next(self._id_counter),
            method=method,
            params=params,
        )
