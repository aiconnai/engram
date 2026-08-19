"""Realtime and SSE streaming resource mixin."""

from __future__ import annotations

import json
from typing import Any, AsyncIterator

import httpx

from ..errors import EngramError
from .base import ResourceMixin


class EventsMixin(ResourceMixin):
    """Realtime Server-Sent Events (SSE) streaming and progress notification operations."""

    def parse_sse_event(self, event_chunk: str) -> dict[str, Any] | None:
        """Parse a raw SSE event payload into a dictionary."""
        lines = event_chunk.strip().split("\n")
        event_type = "memory_created"
        data_str = ""
        seq_id = None

        for line in lines:
            if line.startswith("event:"):
                event_type = line[6:].strip()
            elif line.startswith("id:"):
                raw_id = line[3:].strip()
                try:
                    seq_id = int(raw_id)
                except ValueError:
                    pass
            elif line.startswith("data:"):
                data_str = line[5:].strip()

        if not data_str:
            return None

        try:
            parsed = json.loads(data_str)
            if not isinstance(parsed, dict):
                return None
            return {
                "seq_id": seq_id or parsed.get("seq_id"),
                "type": event_type,
                "timestamp": parsed.get("timestamp"),
                "memory_id": parsed.get("memory_id"),
                "preview": parsed.get("preview"),
                "changes": parsed.get("changes"),
                "data": parsed.get("data", parsed),
            }
        except Exception:
            return None

    async def stream_events(
        self,
        *,
        workspace: str | None = None,
        event_types: str | list[str] | None = None,
        last_event_id: int | str | None = None,
    ) -> AsyncIterator[dict[str, Any]]:
        """Stream Server-Sent Events from `GET /v1/events` as an async generator.

        Args:
            workspace: Optional workspace filter.
            event_types: Event type or list of event types to subscribe to.
            last_event_id: Optional last sequence ID for resumable stream replay.
        """
        client = getattr(self, "_client", None)
        if client is None:
            raise EngramError("EngramClient is closed")

        params: dict[str, str] = {}
        if workspace is not None:
            params["workspace"] = workspace
        if event_types is not None:
            if isinstance(event_types, list):
                params["event_types"] = ",".join(event_types)
            else:
                params["event_types"] = str(event_types)

        headers: dict[str, str] = {"Accept": "text/event-stream"}
        if last_event_id is not None:
            headers["Last-Event-Id"] = str(last_event_id)

        try:
            async with client.stream("GET", "/v1/events", params=params, headers=headers) as response:
                if response.status_code != 200:
                    raise EngramError(f"SSE stream failed with status {response.status_code}")

                buffer = ""
                async for line in response.aiter_lines():
                    if line == "":
                        if buffer:
                            event = self.parse_sse_event(buffer)
                            if event:
                                yield event
                            buffer = ""
                    else:
                        if not line.startswith(":"):
                            buffer += line + "\n"
        except httpx.HTTPError as err:
            raise EngramError(f"Streaming connection failed: {err}") from err

    async def watch_progress(
        self,
        token: str | int,
        *,
        workspace: str | None = None,
    ) -> AsyncIterator[dict[str, Any]]:
        """Stream progress notifications matching a specific progressToken.

        Args:
            token: The progressToken integer or string.
            workspace: Optional workspace filter.
        """
        str_token = str(token)
        async for event in self.stream_events(workspace=workspace, event_types="progress"):
            if event.get("type") == "progress":
                data = event.get("data", {})
                event_token = str(data.get("progress_token", data.get("progressToken", "")))
                if event_token == str_token:
                    yield {
                        "seq_id": event.get("seq_id"),
                        "type": "progress",
                        "timestamp": event.get("timestamp"),
                        "preview": event.get("preview"),
                        "data": {
                            "progress_token": event_token,
                            "progress": data.get("progress", 0),
                            "total": data.get("total"),
                            "message": data.get("message", event.get("preview")),
                            "workspace": data.get("workspace"),
                        },
                    }
