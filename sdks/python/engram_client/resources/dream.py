"""Dream Phase and review pipeline resource mixin."""

from __future__ import annotations

from typing import Any

from .base import ResourceMixin


class DreamMixin(ResourceMixin):
    """Dream Phase job lifecycle, candidate review, evaluation, and background consolidation."""

    async def dream_create(
        self,
        *,
        workspace: str = "default",
        run: bool = True,
        job_id: str | None = None,
        instructions: str | None = None,
        max_memories: int = 50,
        max_candidates: int = 25,
        summary_min_memories: int = 2,
    ) -> dict[str, Any]:
        """Create and optionally run a reviewable dream snapshot job."""
        params: dict[str, Any] = {
            "workspace": workspace,
            "run": run,
            "max_memories": max_memories,
            "max_candidates": max_candidates,
            "summary_min_memories": summary_min_memories,
        }
        if job_id is not None:
            params["job_id"] = job_id
        if instructions is not None:
            params["instructions"] = instructions
        return await self._mcp_call("dream_create", params)

    async def dream_get(self, id: str) -> dict[str, Any]:
        """Inspect a dream job by ID."""
        return await self._mcp_call("dream_get", {"id": id})

    async def dream_list(
        self,
        *,
        workspace: str | None = None,
        status: str | None = None,
        limit: int | None = None,
    ) -> dict[str, Any]:
        """List dream jobs by workspace and status."""
        params: dict[str, Any] = {}
        if workspace is not None:
            params["workspace"] = workspace
        if status is not None:
            params["status"] = status
        if limit is not None:
            params["limit"] = limit
        return await self._mcp_call("dream_list", params)

    async def dream_cancel(self, id: str) -> dict[str, Any]:
        """Cancel a pending or running dream job."""
        return await self._mcp_call("dream_cancel", {"id": id})

    async def dream_archive(self, id: str) -> dict[str, Any]:
        """Archive a terminal dream job."""
        return await self._mcp_call("dream_archive", {"id": id})

    async def dream_candidates_list(
        self,
        *,
        workspace: str | None = None,
        job_id: str | None = None,
        review_state: str | None = None,
        kind: str | None = None,
        proposed_action: str | None = None,
        limit: int | None = None,
    ) -> dict[str, Any]:
        """List dream review candidates with optional filters."""
        params: dict[str, Any] = {}
        if workspace is not None:
            params["workspace"] = workspace
        if job_id is not None:
            params["job_id"] = job_id
        if review_state is not None:
            params["review_state"] = review_state
        if kind is not None:
            params["kind"] = kind
        if proposed_action is not None:
            params["proposed_action"] = proposed_action
        if limit is not None:
            params["limit"] = limit
        return await self._mcp_call("dream_candidates_list", params)

    async def dream_candidate_get(self, id: str) -> dict[str, Any]:
        """Inspect one candidate and its evidence sources."""
        return await self._mcp_call("dream_candidate_get", {"id": id})

    async def dream_candidate_review(
        self,
        id: str,
        review_state: str,
        *,
        edited_content: str | None = None,
        notes: str | None = None,
    ) -> dict[str, Any]:
        """Review a candidate (accept, edit, reject, archive)."""
        params: dict[str, Any] = {"id": id, "review_state": review_state}
        if edited_content is not None:
            params["edited_content"] = edited_content
        if notes is not None:
            params["notes"] = notes
        return await self._mcp_call("dream_candidate_review", params)

    async def dream_candidate_apply(
        self,
        id: str,
        *,
        confirm: bool = True,
        reviewer_notes: str | None = None,
    ) -> dict[str, Any]:
        """Apply an accepted or edited candidate to canonical memory.

        Requires ``confirm=True``.
        """
        params: dict[str, Any] = {"id": id, "confirm": confirm}
        if reviewer_notes is not None:
            params["reviewer_notes"] = reviewer_notes
        return await self._mcp_call("dream_candidate_apply", params)

    async def dream_eval_run(
        self,
        *,
        workspace: str | None = None,
        lane: str | None = None,
    ) -> dict[str, Any]:
        """Run deterministic local dream snapshot evaluation fixtures."""
        params: dict[str, Any] = {}
        if workspace is not None:
            params["workspace"] = workspace
        if lane is not None:
            params["lane"] = lane
        return await self._mcp_call("dream_eval_run", params)

    async def dream_run_now(
        self,
        *,
        workspace: str | None = None,
    ) -> dict[str, Any]:
        """Trigger an immediate background consolidation pass across all workspaces."""
        params: dict[str, Any] = {}
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("dream_run_now", params)
