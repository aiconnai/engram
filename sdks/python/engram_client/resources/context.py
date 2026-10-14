"""Context resource mixin."""

from __future__ import annotations

from typing import Any

from .base import ResourceMixin


class ContextMixin(ResourceMixin):
    """Context engineering, atomic fact extraction, prompt templates, and memory blocks."""

    # -- Context Engineering --

    async def extract_facts(self, memory_id: int) -> dict[str, Any]:
        """Extract atomic facts from a memory."""
        return await self._mcp_call("memory_extract_facts", {"memory_id": memory_id})

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
        return await self._mcp_call("memory_list_facts", params)

    async def fact_graph(
        self,
        subject: str = "default",
        *,
        workspace: str | None = None,
    ) -> dict[str, Any]:
        """Export a graph of extracted facts and their relationships."""
        return await self._mcp_call("memory_fact_graph", {"subject": subject or "default"})

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
            "total_budget": token_budget,
        }
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_build_context", params)

    async def prompt_template(
        self,
        template_name: str = "default",
        *,
        query: str | None = None,
        workspace: str | None = None,
        token_budget: int | None = None,
        include_types: list[str] | None = None,
        memories: list[Any] | None = None,
    ) -> dict[str, Any]:
        """Render a named prompt template populated with memories."""
        resolved_query = query or template_name or "default"
        params: dict[str, Any] = {"query": resolved_query}
        if workspace is not None:
            params["workspace"] = workspace
        if token_budget is not None:
            params["token_budget"] = token_budget
        if include_types is not None:
            params["include_types"] = include_types
        return await self._mcp_call("memory_get_injection_prompt", params)

    async def token_estimate(
        self,
        content: str = "",
        *,
        ids: list[int] | None = None,
        token_budget: int | None = None,
    ) -> dict[str, Any]:
        """Estimate the token count for the given content."""
        params: dict[str, Any] = {"ids": ids if ids is not None else [1]}
        if token_budget is not None:
            params["token_budget"] = token_budget
        return await self._mcp_call("memory_compress_for_context", params)

    async def block_get(
        self,
        block_type: str = "persona",
        label: str = "default",
        *,
        workspace: str | None = None,
        name: str | None = None,
    ) -> dict[str, Any]:
        """Retrieve a named memory block by type and label."""
        return await self._mcp_call("memory_block_get", {"name": name or label or "default"})

    async def block_edit(
        self,
        block_type: str = "persona",
        label: str = "default",
        content: str = "",
        *,
        workspace: str | None = None,
        reason: str | None = None,
        name: str | None = None,
    ) -> dict[str, Any]:
        """Edit the content of an existing memory block."""
        params: dict[str, Any] = {
            "name": name or label or "default",
            "content": content,
        }
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
        return await self._mcp_call("memory_block_list", {})

    async def block_create(
        self,
        block_type: str = "persona",
        label: str = "default",
        content: str = "",
        *,
        workspace: str | None = None,
        max_tokens: int | None = None,
        name: str | None = None,
    ) -> dict[str, Any]:
        """Create a new named memory block."""
        params: dict[str, Any] = {
            "name": name or label or "default",
            "content": content,
        }
        if max_tokens is not None:
            params["max_tokens"] = max_tokens
        return await self._mcp_call("memory_block_create", params)
