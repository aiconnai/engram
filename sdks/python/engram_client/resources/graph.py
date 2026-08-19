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
    ) -> dict[str, Any]:
        """Detect conflicting or contradictory memories."""
        params: dict[str, Any] = {}
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_detect_conflicts", params)

    async def resolve_conflict(
        self,
        conflict_id: str,
        resolution: str,
    ) -> dict[str, Any]:
        """Resolve a detected memory conflict."""
        return await self._mcp_call(
            "memory_resolve_conflict",
            {"conflict_id": conflict_id, "resolution": resolution},
        )

    async def coactivation_report(
        self,
        *,
        workspace: str | None = None,
        limit: int = 50,
    ) -> dict[str, Any]:
        """Report memories that are frequently co-accessed."""
        params: dict[str, Any] = {"limit": limit}
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_coactivation_report", params)

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
        subject: str,
        predicate: str,
        object: str,
        *,
        confidence: float = 1.0,
    ) -> dict[str, Any]:
        """Add a knowledge triplet to the graph."""
        return await self._mcp_call(
            "memory_add_knowledge",
            {
                "subject": subject,
                "predicate": predicate,
                "object": object,
                "confidence": confidence,
            },
        )

    # -- Temporal Graph --

    async def temporal_create(
        self,
        from_entity: str,
        to_entity: str,
        relation: str,
        *,
        valid_from: str | None = None,
        confidence: float = 1.0,
    ) -> dict[str, Any]:
        """Create a time-bounded edge in the temporal knowledge graph."""
        params: dict[str, Any] = {
            "from_entity": from_entity,
            "to_entity": to_entity,
            "relation": relation,
            "confidence": confidence,
        }
        if valid_from is not None:
            params["valid_from"] = valid_from
        return await self._mcp_call("memory_temporal_create", params)

    async def temporal_invalidate(
        self,
        edge_id: str,
        *,
        reason: str | None = None,
    ) -> dict[str, Any]:
        """Mark a temporal graph edge as no longer valid."""
        params: dict[str, Any] = {"edge_id": edge_id}
        if reason is not None:
            params["reason"] = reason
        return await self._mcp_call("memory_temporal_invalidate", params)

    async def temporal_snapshot(
        self,
        *,
        timestamp: str | None = None,
        workspace: str | None = None,
    ) -> dict[str, Any]:
        """Get a snapshot of the knowledge graph at a specific point in time."""
        params: dict[str, Any] = {}
        if timestamp is not None:
            params["timestamp"] = timestamp
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_temporal_snapshot", params)

    async def temporal_contradictions(
        self,
        *,
        workspace: str | None = None,
    ) -> dict[str, Any]:
        """Find temporal contradictions in the knowledge graph."""
        params: dict[str, Any] = {}
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_temporal_contradictions", params)

    async def temporal_evolve(self, entity: str) -> dict[str, Any]:
        """Trace how an entity's relationships have evolved over time."""
        return await self._mcp_call("memory_temporal_evolve", {"entity": entity})

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

