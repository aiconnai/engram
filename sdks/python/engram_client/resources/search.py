"""Search resource mixin."""

from __future__ import annotations

from typing import Any

from .base import ResourceMixin


class SearchMixin(ResourceMixin):
    """Hybrid search, council orchestration, cache, feedback, and federation operations."""

    # -- Search & Digest --

    async def digest(
        self,
        topic: str,
        *,
        workspace: str | None = None,
        mode: str | None = None,
        limit: int | None = None,
        related_depth: int | None = None,
        total_budget: int | None = None,
        include_types: list[str] | None = None,
        timeframe: str | None = None,
        include_graph: bool | None = None,
        include_operational_context: bool | None = None,
        include_next_actions: bool | None = None,
        current_git_branch: str | None = None,
        current_commit_hash: str | None = None,
    ) -> dict[str, Any]:
        """Build an actionable, source-linked retrieval digest for a topic.

        Args:
            topic: Natural language topic or question.
            workspace: Optional workspace filter.
            mode: Digest detail mode ('brief', 'standard', 'deep').
            limit: Maximum source memories to inspect (1..50).
            related_depth: Graph expansion depth (0..2).
            total_budget: Approximate token budget (512..12000).
            include_types: Optional list of memory types to filter.
            timeframe: Time window filter ('1h', '24h', '7d', '30d', 'all').
            include_graph: Whether to include graph relationships.
            include_operational_context: Whether to attach operational context.
            include_next_actions: Whether to extract recommended next actions.
            current_git_branch: Active git branch name for staleness checks.
            current_commit_hash: Active git commit hash for staleness checks.
        """
        params: dict[str, Any] = {"topic": topic}
        if workspace is not None:
            params["workspace"] = workspace
        if mode is not None:
            params["mode"] = mode
        if limit is not None:
            params["limit"] = limit
        if related_depth is not None:
            params["related_depth"] = related_depth
        if total_budget is not None:
            params["total_budget"] = total_budget
        if include_types is not None:
            params["include_types"] = include_types
        if timeframe is not None:
            params["timeframe"] = timeframe
        if include_graph is not None:
            params["include_graph"] = include_graph
        if include_operational_context is not None:
            params["include_operational_context"] = include_operational_context
        if include_next_actions is not None:
            params["include_next_actions"] = include_next_actions
        if current_git_branch is not None:
            params["current_git_branch"] = current_git_branch
        if current_commit_hash is not None:
            params["current_commit_hash"] = current_commit_hash
        return await self._mcp_call("memory_digest", params)

    async def search(
        self,
        query: str,
        *,
        limit: int = 10,
        workspace: str | None = None,
        filter_: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        """Hybrid search (BM25 + vector + fuzzy).

        Accepts the same ``filter_`` syntax as :meth:`list` for advanced
        metadata filtering on search results.
        """
        params: dict[str, Any] = {"query": query, "limit": limit}
        if workspace is not None:
            params["workspace"] = workspace
        if filter_ is not None:
            params["filter"] = filter_
        return await self._mcp_call("memory_search", params)

    async def memory_council(
        self,
        prompt: str,
        *,
        conversation_id: str | None = None,
        council_url: str | None = None,
        timeout_seconds: int | None = None,
        include_raw_stages: bool = True,
        persist: bool = False,
        workspace: str | None = None,
        memory_tags: list[str] | None = None,
    ) -> dict[str, Any]:
        """Run a prompt through llm-council and return the consolidated response.

        Useful when you want multi-agent consensus while keeping the result in Engram
        via ``persist=True``.
        """
        params: dict[str, Any] = {
            "prompt": prompt,
            "include_raw_stages": include_raw_stages,
            "persist": persist,
        }
        if conversation_id is not None:
            params["conversation_id"] = conversation_id
        if council_url is not None:
            params["council_url"] = council_url
        if timeout_seconds is not None:
            params["timeout_seconds"] = timeout_seconds
        if workspace is not None:
            params["workspace"] = workspace
        if memory_tags is not None:
            params["memory_tags"] = memory_tags
        return await self._mcp_call("memory_council", params)

    # -- Retrieval Excellence --

    async def cache_stats(self) -> dict[str, Any]:
        """Get embedding and search cache statistics."""
        return await self._mcp_call("memory_cache_stats", {})

    async def cache_clear(self, *, workspace: str | None = None) -> dict[str, Any]:
        """Clear the embedding and search cache."""
        params: dict[str, Any] = {}
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_cache_clear", params)

    async def embedding_providers(self) -> dict[str, Any]:
        """List available embedding providers and their status."""
        return await self._mcp_call("memory_embedding_providers", {})

    async def embedding_migrate(
        self,
        *,
        from_provider: str | None = None,
        to_provider: str | None = None,
    ) -> dict[str, Any]:
        """Migrate embeddings from one provider to another."""
        params: dict[str, Any] = {}
        if from_provider is not None:
            params["from_provider"] = from_provider
        if to_provider is not None:
            params["to_provider"] = to_provider
        return await self._mcp_call("memory_embedding_migrate", params)

    async def explain_search(self, results: list[Any]) -> dict[str, Any]:
        """Explain why specific search results were returned."""
        return await self._mcp_call("memory_explain_search", {"results": results})

    async def feedback(
        self,
        query: str,
        memory_id: int,
        signal: str,
    ) -> dict[str, Any]:
        """Record relevance feedback for a search result to improve future retrieval."""
        return await self._mcp_call(
            "memory_feedback",
            {"query": query, "memory_id": memory_id, "signal": signal},
        )

    async def feedback_stats(self, *, workspace: str | None = None) -> dict[str, Any]:
        """Get aggregated feedback statistics."""
        params: dict[str, Any] = {}
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_feedback_stats", params)

    # -- Federation --

    async def federation_add_peer(
        self,
        url: str,
        api_key: str,
        *,
        name: str | None = None,
    ) -> dict[str, Any]:
        """Register a remote Engram instance as a federation peer."""
        params: dict[str, Any] = {"url": url, "api_key": api_key}
        if name is not None:
            params["name"] = name
        return await self._mcp_call("memory_federation_add_peer", params)

    async def federation_remove_peer(self, peer_id: str) -> dict[str, Any]:
        """Remove a federation peer by ID."""
        return await self._mcp_call("memory_federation_remove_peer", {"peer_id": peer_id})

    async def federation_list_peers(self) -> dict[str, Any]:
        """List all registered federation peers."""
        return await self._mcp_call("memory_federation_list_peers", {})

    async def federation_search(
        self,
        query: str,
        *,
        limit: int = 10,
    ) -> dict[str, Any]:
        """Search memories across all federation peers."""
        return await self._mcp_call(
            "memory_federation_search",
            {"query": query, "limit": limit},
        )

    async def federation_share(self, memory_id: int, peer_id: str) -> dict[str, Any]:
        """Share a local memory with a specific federation peer."""
        return await self._mcp_call(
            "memory_federation_share",
            {"memory_id": memory_id, "peer_id": peer_id},
        )

    async def federation_sync_status(self) -> dict[str, Any]:
        """Get the synchronization status for all federation peers."""
        return await self._mcp_call("memory_federation_sync_status", {})
