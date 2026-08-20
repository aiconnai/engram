"""Spatial navigation resource mixin (Method of Loci)."""

from __future__ import annotations

from typing import Any

from .base import ResourceMixin


class SpatialMixin(ResourceMixin):
    """Mnemonic spatial memory operations (Palace, Wings, Rooms, Drawers)."""

    async def palace_navigate(
        self,
        *,
        workspace: str = "default",
        wing: str | None = None,
    ) -> dict[str, Any]:
        """Navigate the Memory Palace to discover wings, rooms, and drawer counts."""
        params: dict[str, Any] = {"workspace": workspace}
        if wing is not None:
            params["wing"] = wing
        return await self._mcp_call("palace_navigate", params)

    async def room_search(
        self,
        wing: str,
        query: str,
        *,
        room: str | None = None,
        limit: int = 10,
        workspace: str | None = None,
    ) -> dict[str, Any]:
        """Search memories scoped within a specific wing and room."""
        params: dict[str, Any] = {
            "wing": wing,
            "query": query,
            "limit": limit,
        }
        if room is not None:
            params["room"] = room
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("room_search", params)

    async def drawer_open(self, memory_id: int) -> dict[str, Any]:
        """Open a memory drawer by ID to inspect full verbatim content and metadata."""
        return await self._mcp_call("drawer_open", {"id": memory_id})
