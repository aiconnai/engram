"""Context resource mixin."""

from __future__ import annotations

from typing import Any

from .base import ResourceMixin


class ContextMixin(ResourceMixin):
    """Context engineering, atomic fact extraction, prompt templates, and memory blocks."""

    # -- Context Engineering --

    async def extract_facts(self, memory_id: int) -> dict[str, Any]:
        """Extract atomic facts from a memory."""
        return await self._mcp_call("memory_extract_facts", {"id": memory_id})

    async def list_facts(
        self,
        *,
        memory_id: int | None = None,
        workspace: str | None = None,
        limit: int = 50,
    ) -> dict[str, Any]:
        """List extracted facts, optionally filtered by memory or workspace."""
        params: dict[str, Any] = {"limit": limit}
        if memory_id is not None:
            params["memory_id"] = memory_id
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_list_facts", params)

    async def fact_graph(self, *, workspace: str | None = None) -> dict[str, Any]:
        """Export a graph of extracted facts and their relationships."""
        params: dict[str, Any] = {}
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_fact_graph", params)

    async def build_context(
        self,
        query: str,
        *,
        strategy: str = "balanced",
        token_budget: int = 4096,
        workspace: str | None = None,
    ) -> dict[str, Any]:
        """Build an optimised context window for an LLM prompt."""
        params: dict[str, Any] = {
            "query": query,
            "strategy": strategy,
            "token_budget": token_budget,
        }
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_build_context", params)

    async def prompt_template(
        self,
        template_name: str,
        *,
        memories: list[Any] | None = None,
    ) -> dict[str, Any]:
        """Render a named prompt template populated with memories."""
        params: dict[str, Any] = {"template_name": template_name}
        if memories is not None:
            params["memories"] = memories
        return await self._mcp_call("memory_prompt_template", params)

    async def token_estimate(self, content: str) -> dict[str, Any]:
        """Estimate the token count for the given content."""
        return await self._mcp_call("memory_token_estimate", {"content": content})

    async def block_get(
        self,
        block_type: str,
        label: str,
        *,
        workspace: str | None = None,
    ) -> dict[str, Any]:
        """Retrieve a named memory block by type and label."""
        params: dict[str, Any] = {"block_type": block_type, "label": label}
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_block_get", params)

    async def block_edit(
        self,
        block_type: str,
        label: str,
        content: str,
        *,
        workspace: str | None = None,
        reason: str | None = None,
    ) -> dict[str, Any]:
        """Edit the content of an existing memory block."""
        params: dict[str, Any] = {
            "block_type": block_type,
            "label": label,
            "content": content,
        }
        if workspace is not None:
            params["workspace"] = workspace
        if reason is not None:
            params["reason"] = reason
        return await self._mcp_call("memory_block_edit", params)

    async def block_list(
        self,
        *,
        block_type: str | None = None,
        workspace: str | None = None,
    ) -> dict[str, Any]:
        """List memory blocks, optionally filtered by type or workspace."""
        params: dict[str, Any] = {}
        if block_type is not None:
            params["block_type"] = block_type
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_block_list", params)

    async def block_create(
        self,
        block_type: str,
        label: str,
        content: str,
        *,
        workspace: str | None = None,
        max_tokens: int = 2048,
    ) -> dict[str, Any]:
        """Create a new named memory block."""
        params: dict[str, Any] = {
            "block_type": block_type,
            "label": label,
            "content": content,
            "max_tokens": max_tokens,
        }
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_block_create", params)
