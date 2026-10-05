"""Model Routing resource mixin (RFC 0011)."""

from __future__ import annotations

from typing import Any

from .base import ResourceMixin


class ModelRoutingMixin(ResourceMixin):
    """Model routing and provider resolution operations (RFC 0011)."""

    async def model_route_resolve(
        self,
        purpose: str,
        *,
        preferred_provider: str | None = None,
    ) -> dict[str, Any]:
        """Deterministically resolve the active or preferred model route for a given AI capability / purpose (RFC 0011).

        Args:
            purpose: Model purpose (e.g. 'embedding_text', 'rerank', 'vision_describe_image', 'audio_transcribe', 'llm_council', 'token_count', 'deterministic_eval').
            preferred_provider: Optional caller preference for provider (e.g. 'openai', 'voyage', 'cohere', 'tfidf', 'clip').

        Returns:
            Structured route resolution dictionary.
        """
        params: dict[str, Any] = {"purpose": purpose}
        if preferred_provider is not None:
            params["preferred_provider"] = preferred_provider
        return await self._mcp_call("model_route_resolve", params)

    async def model_routes_list(
        self,
        *,
        purpose: str | None = None,
    ) -> dict[str, Any]:
        """List all declared model routes and their capabilities, cost classes, and fallback policies (RFC 0011).

        Args:
            purpose: Optional purpose filter.

        Returns:
            Dictionary containing `routes_count` and list of `routes`.
        """
        params: dict[str, Any] = {}
        if purpose is not None:
            params["purpose"] = purpose
        return await self._mcp_call("model_routes_list", params)
