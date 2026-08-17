"""Base resource mixin."""

from __future__ import annotations

from typing import Any


class ResourceMixin:
    """Base mixin providing protocol signature for MCP tool invocation."""

    async def _mcp_call(
        self, method: str, params: dict[str, Any] | None = None
    ) -> Any:
        raise NotImplementedError
