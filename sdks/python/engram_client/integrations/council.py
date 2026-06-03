"""Convenience skill wrapper for Engram llm-council workflows."""

from __future__ import annotations

from typing import Any, Iterable

from engram_client.client import EngramClient


class CouncilSkill:
    """Reusable helper to run consensus prompts through llm-council.

    The skill keeps the protocol details centralized and exposes a small
    opinionated API for projects:

    - fixed defaults (workspace, timeout, persistence behavior)
    - consistent return shape from the skill contract
    - optional structured memory tags
    """

    def __init__(
        self,
        client: EngramClient,
        *,
        default_workspace: str = "default",
        default_timeout_seconds: int = 90,
        default_include_raw_stages: bool = False,
    ) -> None:
        self.client = client
        self.default_workspace = default_workspace
        self.default_timeout_seconds = default_timeout_seconds
        self.default_include_raw_stages = default_include_raw_stages

    async def ask(
        self,
        prompt: str,
        *,
        persist: bool = False,
        workspace: str | None = None,
        timeout_seconds: int | None = None,
        include_raw_stages: bool | None = None,
        conversation_id: str | None = None,
        council_url: str | None = None,
        tags: Iterable[str] | None = None,
    ) -> dict[str, Any]:
        """Run the prompt and return the council response.

        Args:
            prompt: User question/task.
            persist: Store response as checkpoint memory.
            workspace: Optional workspace override.
            timeout_seconds: HTTP timeout for the request.
            include_raw_stages: Return raw stage data as returned by council.
            conversation_id: Optional existing council conversation id.
            council_url: Optional override for council base URL.
            tags: Extra tags used when persisting.
        """
        if not isinstance(prompt, str) or not prompt.strip():
            return {"error": "prompt must be a non-empty string"}

        if timeout_seconds is None:
            timeout_seconds = self.default_timeout_seconds

        if include_raw_stages is None:
            include_raw_stages = self.default_include_raw_stages

        memory_tags: list[str] | None = list(tags) if tags is not None else None

        return await self.client.memory_council(
            prompt,
            conversation_id=conversation_id,
            council_url=council_url,
            timeout_seconds=timeout_seconds,
            include_raw_stages=include_raw_stages,
            persist=persist,
            workspace=workspace or self.default_workspace,
            memory_tags=memory_tags,
        )

    async def ask_with_persistence(
        self,
        prompt: str,
        *,
        workspace: str | None = None,
        timeout_seconds: int | None = None,
        include_raw_stages: bool | None = None,
        conversation_id: str | None = None,
        council_url: str | None = None,
        tags: Iterable[str] | None = None,
    ) -> dict[str, Any]:
        """Run the prompt and force persistence of the result as checkpoint."""
        return await self.ask(
            prompt,
            persist=True,
            workspace=workspace,
            timeout_seconds=timeout_seconds,
            include_raw_stages=include_raw_stages,
            conversation_id=conversation_id,
            council_url=council_url,
            tags=tags,
        )
