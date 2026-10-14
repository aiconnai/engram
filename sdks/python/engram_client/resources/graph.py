"""Graph resource mixin."""

from __future__ import annotations

from typing import Any

from .base import ResourceMixin


class GraphMixin(ResourceMixin):
    """Knowledge graph, temporal edges, triplets, and conflict resolution operations."""

    # -- Graph --

    async def related(self, memory_id: int) -> dict[str, Any]:
        """Get related memories via knowledge graph."""
        return await self._mcp_call("memory_related", {"id": memory_id})

    async def link(
        self,
        from_id: int,
        to_id: int,
        edge_type: str = "related_to",
    ) -> dict[str, Any]:
        """Create a link between two memories."""
        return await self._mcp_call(
            "memory_link",
            {"from_id": from_id, "to_id": to_id, "edge_type": edge_type},
        )

    # -- Advanced Graph --

    async def detect_conflicts(
        self,
        *,
        workspace: str | None = None,
        save: bool | None = None,
    ) -> dict[str, Any]:
        """Detect conflicting or contradictory memories."""
        params: dict[str, Any] = {}
        if save is not None:
            params["save"] = save
        return await self._mcp_call("memory_detect_conflicts", params)

    async def resolve_conflict(
        self,
        conflict_id: str,
        resolution: str = "keep_both",
        *,
        strategy: str | None = None,
    ) -> dict[str, Any]:
        """Resolve a detected memory conflict."""
        resolved_strategy = strategy or resolution
        params: dict[str, Any] = {
            "conflict_id": conflict_id,
            "strategy": resolved_strategy,
        }
        return await self._mcp_call("memory_resolve_conflict", params)

    async def coactivation_report(
        self,
        *,
        workspace: str | None = None,
        limit: int = 50,
    ) -> dict[str, Any]:
        """Report memories that are frequently co-accessed."""
        return await self._mcp_call("memory_coactivation_report", {})

    async def query_triplets(
        self,
        *,
        subject: str | None = None,
        predicate: str | None = None,
        object: str | None = None,
    ) -> dict[str, Any]:
        """Query knowledge graph triplets by subject, predicate, or object."""
        params: dict[str, Any] = {}
        if subject is not None:
            params["subject"] = subject
        if predicate is not None:
            params["predicate"] = predicate
        if object is not None:
            params["object"] = object
        return await self._mcp_call("memory_query_triplets", params)

    async def add_knowledge(
        self,
        subject: str = "default",
        predicate: str = "default",
        object: str = "default",
        *,
        confidence: float = 1.0,
        fact: str | None = None,
        importance: float | None = None,
        scope: str | None = None,
        session_id: str | None = None,
        source: str | None = None,
        tags: list[str] | None = None,
        workspace: str | None = None,
    ) -> dict[str, Any]:
        """Add a knowledge triplet or fact to the graph."""
        resolved_fact = fact or f"{subject} {predicate} {object}"
        params: dict[str, Any] = {"fact": resolved_fact}
        if importance is not None:
            params["importance"] = importance
        if scope is not None:
            params["scope"] = scope
        if session_id is not None:
            params["session_id"] = session_id
        if source is not None:
            params["source"] = source
        if tags is not None:
            params["tags"] = tags
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_ingest_fact", params)

    # -- Temporal Graph --

    async def temporal_create(
        self,
        from_entity: str | int = 1,
        to_entity: str | int = 2,
        relation: str = "relates",
        *,
        valid_from: str = "2026-01-01T00:00:00Z",
        confidence: float = 1.0,
        from_id: int | None = None,
        to_id: int | None = None,
        scope_path: str | None = None,
        source: str | None = None,
        properties: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        """Create a time-bounded edge in the temporal knowledge graph."""
        def _to_int(val: Any, default: int) -> int:
            if isinstance(val, int):
                return val
            if isinstance(val, str) and val.isdigit():
                return int(val)
            return default

        f_id = from_id if from_id is not None else _to_int(from_entity, 1)
        t_id = to_id if to_id is not None else _to_int(to_entity, 2)
        params: dict[str, Any] = {
            "from_id": f_id,
            "to_id": t_id,
            "relation": relation,
            "valid_from": valid_from or "2026-01-01T00:00:00Z",
        }
        if confidence is not None:
            params["confidence"] = confidence
        if scope_path is not None:
            params["scope_path"] = scope_path
        if source is not None:
            params["source"] = source
        if properties is not None:
            params["properties"] = properties
        return await self._mcp_call("temporal_add_edge", params)

    async def temporal_invalidate(
        self,
        edge_id: str = "default",
        *,
        reason: str | None = None,
        t1: str = "2026-01-01T00:00:00Z",
        t2: str = "2026-01-02T00:00:00Z",
        scope_path: str | None = None,
    ) -> dict[str, Any]:
        """Mark a temporal graph edge as no longer valid or diff temporal states."""
        params: dict[str, Any] = {
            "t1": t1 or "2026-01-01T00:00:00Z",
            "t2": t2 or "2026-01-02T00:00:00Z",
        }
        if scope_path is not None:
            params["scope_path"] = scope_path
        return await self._mcp_call("temporal_diff", params)

    async def temporal_snapshot(
        self,
        *,
        timestamp: str = "2026-01-01T00:00:00Z",
        workspace: str | None = None,
        scope: str | None = None,
    ) -> dict[str, Any]:
        """Get a snapshot of the knowledge graph at a specific point in time."""
        resolved_scope = scope or workspace or "global"
        resolved_timestamp = timestamp or "2026-01-01T00:00:00Z"
        return await self._mcp_call(
            "memory_temporal_snapshot",
            {"scope": resolved_scope, "timestamp": resolved_timestamp},
        )

    async def temporal_contradictions(
        self,
        *,
        workspace: str | None = None,
    ) -> dict[str, Any]:
        """Find temporal contradictions in the knowledge graph."""
        return await self._mcp_call("temporal_contradictions", {})

    async def temporal_evolve(
        self,
        entity: str = "default",
        *,
        from_id: int | None = None,
        to_id: int | None = None,
        relation: str | None = None,
        valid_from: str | None = None,
    ) -> dict[str, Any]:
        """Trace how an entity's relationships have evolved over time."""
        params: dict[str, Any] = {
            "from_id": from_id if from_id is not None else 1,
            "to_id": to_id if to_id is not None else 2,
            "relation": relation or "evolved_to",
            "valid_from": valid_from or "2026-01-01T00:00:00Z",
        }
        return await self._mcp_call("temporal_add_edge", params)

    # -- Consolidated Facades (Phase 3c) --

    async def graph_query(
        self,
        *,
        action: str = "relations",
        id: int | None = None,
        from_id: int | None = None,
        to_id: int | None = None,
        depth: int | None = None,
        max_depth: int | None = None,
        edge_type: str | None = None,
        edge_types: list[str] | None = None,
        direction: str | None = None,
        include_entities: bool | None = None,
        query: str | None = None,
        format: str | None = None,
    ) -> dict[str, Any]:
        """Query the knowledge graph: relations, paths, multi-hop traversal, entity search, or export."""
        params: dict[str, Any] = {"action": action}
        if id is not None:
            params["id"] = id
        if from_id is not None:
            params["from_id"] = from_id
        if to_id is not None:
            params["to_id"] = to_id
        if depth is not None:
            params["depth"] = depth
        if max_depth is not None:
            params["max_depth"] = max_depth
        if edge_type is not None:
            params["edge_type"] = edge_type
        if edge_types is not None:
            params["edge_types"] = edge_types
        if direction is not None:
            params["direction"] = direction
        if include_entities is not None:
            params["include_entities"] = include_entities
        if query is not None:
            params["query"] = query
        if format is not None:
            params["format"] = format
        return await self._mcp_call("graph_query", params)

    async def graph_mutate(
        self,
        *,
        action: str = "link",
        from_id: int | None = None,
        to_id: int | None = None,
        id: int | None = None,
        edge_type: str | None = None,
        strength: float | None = None,
        source_context: str | None = None,
        pinned: bool | None = None,
    ) -> dict[str, Any]:
        """Mutate the knowledge graph: link memories, remove cross-references, or extract entities."""
        params: dict[str, Any] = {"action": action}
        if from_id is not None:
            params["from_id"] = from_id
        if to_id is not None:
            params["to_id"] = to_id
        if id is not None:
            params["id"] = id
        if edge_type is not None:
            params["edge_type"] = edge_type
        if strength is not None:
            params["strength"] = strength
        if source_context is not None:
            params["source_context"] = source_context
        if pinned is not None:
            params["pinned"] = pinned
        return await self._mcp_call("graph_mutate", params)

    async def predict_links(
        self,
        *,
        memory_id: int | None = None,
        workspace: str | None = None,
        min_confidence: float = 0.6,
        top_k: int = 10,
        algorithm: str = "hybrid",
        auto_apply: bool = False,
    ) -> dict[str, Any]:
        """Predict implicit or missing relationships between memories."""
        params: dict[str, Any] = {
            "min_confidence": min_confidence,
            "top_k": top_k,
            "algorithm": algorithm,
            "auto_apply": auto_apply,
        }
        if memory_id is not None:
            params["memory_id"] = memory_id
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_predict_links", params)

    async def cluster_concepts(
        self,
        *,
        workspace: str | None = None,
        min_cluster_size: int = 2,
        max_clusters: int = 10,
    ) -> dict[str, Any]:
        """Cluster memories into high-level semantic concept nodes."""
        params: dict[str, Any] = {
            "min_cluster_size": min_cluster_size,
            "max_clusters": max_clusters,
        }
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_cluster_concepts", params)

    async def auto_link(
        self,
        *,
        workspace: str | None = None,
        similarity_threshold: float = 0.75,
        time_window_minutes: int = 30,
    ) -> dict[str, Any]:
        """Run semantic + temporal auto-linker on a workspace."""
        params: dict[str, Any] = {
            "similarity_threshold": similarity_threshold,
            "time_window_minutes": time_window_minutes,
        }
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_auto_link", params)

    async def cluster(
        self,
        *,
        min_cluster_size: int = 2,
        resolution: float = 1.0,
        link_types: list[str] | None = None,
    ) -> dict[str, Any]:
        """Run Louvain community detection on the memory graph."""
        params: dict[str, Any] = {
            "min_cluster_size": min_cluster_size,
            "resolution": resolution,
        }
        if link_types is not None:
            params["link_types"] = link_types
        return await self._mcp_call("memory_cluster", params)

    async def get_cluster(self, memory_id: int) -> dict[str, Any]:
        """Get the cluster containing a specific memory."""
        return await self._mcp_call("memory_get_cluster", {"memory_id": memory_id})

    async def list_clusters(self, algorithm: str = "louvain") -> dict[str, Any]:
        """List all detected clusters."""
        return await self._mcp_call("memory_list_clusters", {"algorithm": algorithm})

