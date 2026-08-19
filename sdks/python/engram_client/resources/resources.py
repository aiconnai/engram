"""MCP Resource discovery, reading, and dynamic subscription resource mixin."""

from __future__ import annotations

from typing import Any

from .base import ResourceMixin


class McpResourcesMixin(ResourceMixin):
    """MCP Resource discovery and dynamic live subscription operations."""

    async def resource_list(self) -> dict[str, Any]:
        """List all resource templates exposed by the MCP server."""
        return await self._mcp_call("resources/list")

    async def resource_read(self, uri: str) -> dict[str, Any]:
        """Read an MCP resource by URI.

        Args:
            uri: The resource URI (e.g. `engram://stats`, `engram://memory/1`, `engram://workspace/dev/memories`).
        """
        return await self._mcp_call("resources/read", {"uri": uri})

    async def resource_subscribe(self, uri: str) -> dict[str, Any]:
        """Subscribe to live updates for an MCP resource URI.

        Args:
            uri: The resource URI to subscribe to.
        """
        return await self._mcp_call("resources/subscribe", {"uri": uri})

    async def resource_unsubscribe(self, uri: str) -> dict[str, Any]:
        """Unsubscribe from updates for an MCP resource URI.

        Args:
            uri: The resource URI to unsubscribe from.
        """
        return await self._mcp_call("resources/unsubscribe", {"uri": uri})
