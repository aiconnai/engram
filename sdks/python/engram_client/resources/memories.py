"""Memories resource mixin."""

from __future__ import annotations

from typing import Any

from .base import ResourceMixin


class MemoriesMixin(ResourceMixin):
    """Memory CRUD, lifecycle, compression, and evolution operations."""

    # -- Memory CRUD --

    async def create(
        self,
        content: str,
        *,
        memory_type: str = "note",
        tags: list[str] | None = None,
        workspace: str | None = None,
        metadata: dict[str, Any] | None = None,
        importance: float | None = None,
        media_url: str | None = None,
    ) -> dict[str, Any]:
        """Create a new memory.

        Supports multimodal types (``image``, ``audio``, ``video``) via the
        ``memory_type`` and ``media_url`` parameters.
        """
        params: dict[str, Any] = {
            "content": content,
            "memory_type": memory_type,
        }
        if tags is not None:
            params["tags"] = tags
        if workspace is not None:
            params["workspace"] = workspace
        if metadata is not None:
            params["metadata"] = metadata
        if importance is not None:
            params["importance"] = importance
        if media_url is not None:
            params["media_url"] = media_url
        return await self._mcp_call("memory_create", params)

    async def get(self, memory_id: int) -> dict[str, Any]:
        """Get a memory by ID."""
        return await self._mcp_call("memory_get", {"id": memory_id})

    async def update(
        self,
        memory_id: int,
        *,
        content: str | None = None,
        tags: list[str] | None = None,
        metadata: dict[str, Any] | None = None,
        importance: float | None = None,
        media_url: str | None = None,
    ) -> dict[str, Any]:
        """Update an existing memory."""
        params: dict[str, Any] = {"id": memory_id}
        if content is not None:
            params["content"] = content
        if tags is not None:
            params["tags"] = tags
        if metadata is not None:
            params["metadata"] = metadata
        if importance is not None:
            params["importance"] = importance
        if media_url is not None:
            params["media_url"] = media_url
        return await self._mcp_call("memory_update", params)

    async def delete(self, memory_id: int) -> dict[str, Any]:
        """Delete a memory."""
        return await self._mcp_call("memory_delete", {"id": memory_id})

    async def list(
        self,
        *,
        limit: int = 50,
        offset: int = 0,
        workspace: str | None = None,
        memory_type: str | None = None,
        tags: list[str] | None = None,
        filter_: dict[str, Any] | None = None,
        sort_by: str | None = None,
        sort_order: str | None = None,
    ) -> list[dict[str, Any]]:
        """List memories with optional filters.

        Advanced filtering is supported via the ``filter_`` parameter, which is
        sent to the MCP API as ``filter`` using AND/OR combinators and
        comparison operators::

            client.list(filter_={
                "AND": [
                    {"importance": {"gte": 0.8}},
                    {"metadata.project": {"eq": "engram"}},
                ]
            })

        Supported operators: ``eq``, ``neq``, ``gt``, ``gte``, ``lt``,
        ``lte``, ``contains``, ``not_contains``, ``exists``.
        """
        params: dict[str, Any] = {"limit": limit, "offset": offset}
        if workspace is not None:
            params["workspace"] = workspace
        if memory_type is not None:
            params["memory_type"] = memory_type
        if tags is not None:
            params["tags"] = tags
        if filter_ is not None:
            params["filter"] = filter_
        if sort_by is not None:
            params["sort_by"] = sort_by
        if sort_order is not None:
            params["sort_order"] = sort_order
        return await self._mcp_call("memory_list", params)

    async def memory_replay_at_time(
        self,
        memory_id: int,
        timestamp: str,
        *,
        event_type: str | None = None,
        include_events: bool = True,
        include_failed: bool = False,
        include_dry_runs: bool = False,
        event_limit: int | None = None,
    ) -> dict[str, Any]:
        """Replay memory state at a given RFC3339 timestamp and optional event trail."""
        params: dict[str, Any] = {
            "memory_id": memory_id,
            "timestamp": timestamp,
            "include_events": include_events,
            "include_failed": include_failed,
            "include_dry_runs": include_dry_runs,
        }
        if event_type is not None:
            params["event_type"] = event_type
        if event_limit is not None:
            params["event_limit"] = event_limit
        return await self._mcp_call("memory_replay_at_time", params)

    # -- Daily (ephemeral) memories --

    async def create_daily(
        self,
        content: str,
        *,
        tags: list[str] | None = None,
        workspace: str | None = None,
        ttl_seconds: int = 86400,
        metadata: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        """Create a daily memory that auto-expires after ``ttl_seconds``.

        Uses the ``memory_create_daily`` MCP tool which sets ``tier='daily'``
        and computes ``expires_at`` from ``ttl_seconds``.
        """
        params: dict[str, Any] = {
            "content": content,
            "ttl_seconds": ttl_seconds,
        }
        if tags is not None:
            params["tags"] = tags
        if workspace is not None:
            params["workspace"] = workspace
        if metadata is not None:
            params["metadata"] = metadata
        return await self._mcp_call("memory_create_daily", params)

    # -- Stats --

    async def stats(self) -> dict[str, Any]:
        """Get memory statistics."""
        return await self._mcp_call("memory_stats", {})

    # -- Compression --

    async def compress(self, memory_id: int) -> dict[str, Any]:
        """Compress a memory to reduce token footprint."""
        return await self._mcp_call("memory_compress", {"id": memory_id})

    async def decompress(self, memory_id: int) -> dict[str, Any]:
        """Decompress a previously compressed memory."""
        return await self._mcp_call("memory_decompress", {"id": memory_id})

    async def compress_for_context(
        self,
        memory_ids: list[int],
        token_budget: int,
    ) -> dict[str, Any]:
        """Compress a set of memories to fit within a token budget."""
        return await self._mcp_call(
            "memory_compress_for_context",
            {"memory_ids": memory_ids, "token_budget": token_budget},
        )

    async def consolidate(
        self,
        workspace: str,
        *,
        threshold: float = 0.8,
    ) -> dict[str, Any]:
        """Consolidate similar memories in a workspace above a similarity threshold."""
        return await self._mcp_call(
            "memory_consolidate",
            {"workspace": workspace, "threshold": threshold},
        )

    async def synthesis(self, memory_ids: list[int]) -> dict[str, Any]:
        """Synthesize multiple memories into a single distilled memory."""
        return await self._mcp_call("memory_synthesis", {"memory_ids": memory_ids})

    # -- Agentic Evolution --

    async def detect_updates(self, memory_id: int) -> dict[str, Any]:
        """Detect whether a memory's content may be outdated."""
        return await self._mcp_call("memory_detect_updates", {"id": memory_id})

    async def utility_score(
        self,
        memory_id: int,
        *,
        signal: str | None = None,
    ) -> dict[str, Any]:
        """Compute or update the utility score for a memory."""
        params: dict[str, Any] = {"id": memory_id}
        if signal is not None:
            params["signal"] = signal
        return await self._mcp_call("memory_utility_score", params)

    async def sentiment_analyze(self, memory_id: int) -> dict[str, Any]:
        """Run sentiment analysis on a memory."""
        return await self._mcp_call("memory_sentiment_analyze", {"id": memory_id})

    async def sentiment_timeline(
        self,
        *,
        workspace: str | None = None,
        limit: int = 50,
    ) -> dict[str, Any]:
        """Retrieve sentiment scores over time for memories in a workspace."""
        params: dict[str, Any] = {"limit": limit}
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_sentiment_timeline", params)

    async def reflect(self, memory_id: int) -> dict[str, Any]:
        """Trigger self-reflection on a memory to surface insights."""
        return await self._mcp_call("memory_reflect", {"id": memory_id})

    # -- Autonomous Agent --

    async def agent_start(self, *, workspace: str | None = None) -> dict[str, Any]:
        """Start the autonomous memory gardening agent."""
        params: dict[str, Any] = {}
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_agent_start", params)

    async def agent_stop(self) -> dict[str, Any]:
        """Stop the autonomous memory gardening agent."""
        return await self._mcp_call("memory_agent_stop", {})

    async def agent_status(self) -> dict[str, Any]:
        """Get the current status of the autonomous agent."""
        return await self._mcp_call("memory_agent_status", {})

    async def agent_metrics(self) -> dict[str, Any]:
        """Get performance metrics for the autonomous agent."""
        return await self._mcp_call("memory_agent_metrics", {})

    async def agent_configure(self, config: dict[str, Any]) -> dict[str, Any]:
        """Configure the autonomous memory agent."""
        return await self._mcp_call("memory_agent_configure", {"config": config})

    async def garden(
        self,
        *,
        workspace: str | None = None,
        dry_run: bool = False,
    ) -> dict[str, Any]:
        """Run one gardening cycle: prune, merge, and promote memories."""
        params: dict[str, Any] = {"dry_run": dry_run}
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_garden", params)

    async def garden_preview(self, *, workspace: str | None = None) -> dict[str, Any]:
        """Preview what a gardening cycle would do without applying changes."""
        params: dict[str, Any] = {}
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_garden_preview", params)

    async def garden_undo(self, operation_id: str) -> dict[str, Any]:
        """Undo a previous gardening operation."""
        return await self._mcp_call("memory_garden_undo", {"operation_id": operation_id})

    async def suggest_acquisition(
        self,
        *,
        workspace: str | None = None,
    ) -> dict[str, Any]:
        """Suggest topics or entities to acquire knowledge about."""
        params: dict[str, Any] = {}
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_suggest_acquisition", params)

    async def proactive_scan(
        self,
        *,
        workspace: str | None = None,
    ) -> dict[str, Any]:
        """Proactively scan memories for gaps, staleness, or improvement opportunities."""
        params: dict[str, Any] = {}
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_proactive_scan", params)

    # -- Consolidated Facades (Phase 3c) --

    async def lifecycle_update(
        self,
        id: int,
        *,
        action: str = "promote",
        canonical_tier: bool | None = None,
        ttl_seconds: int | None = None,
        state: str | None = None,
        reason: str | None = None,
        persist: bool | None = None,
        workspace: str | None = None,
        dry_run: bool | None = None,
    ) -> dict[str, Any]:
        """Update or transition a memory's lifecycle state, reinforcement score, or TTL."""
        params: dict[str, Any] = {"id": id, "action": action}
        if canonical_tier is not None:
            params["canonical_tier"] = canonical_tier
        if ttl_seconds is not None:
            params["ttl_seconds"] = ttl_seconds
        if state is not None:
            params["state"] = state
        if reason is not None:
            params["reason"] = reason
        if persist is not None:
            params["persist"] = persist
        if workspace is not None:
            params["workspace"] = workspace
        if dry_run is not None:
            params["dry_run"] = dry_run
        return await self._mcp_call("memory_lifecycle_update", params)
