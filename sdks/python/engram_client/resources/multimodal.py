"""Multimodal vision, audio, video, screenshot, and media asset operations (RFC 0009)."""

from __future__ import annotations

from typing import Any

from .base import ResourceMixin


class MultimodalMixin(ResourceMixin):
    """Multimodal operations for images, diagrams, audio, and video assets."""

    async def describe_image(
        self,
        image_path: str,
        *,
        prompt: str | None = None,
        max_tokens: int | None = None,
    ) -> dict[str, Any]:
        """Describe an image file using the configured vision provider.

        Args:
            image_path: Local path to the image file.
            prompt: Optional prompt instructions for the vision model.
            max_tokens: Maximum tokens for the generated description.
        """
        params: dict[str, Any] = {"image_path": image_path}
        if prompt is not None:
            params["prompt"] = prompt
        if max_tokens is not None:
            params["max_tokens"] = max_tokens
        return await self._mcp_call("memory_describe_image", params)

    async def transcribe_audio(
        self,
        audio_path: str,
    ) -> dict[str, Any]:
        """Transcribe an audio file into text.

        Args:
            audio_path: Local path to the audio file.
        """
        return await self._mcp_call("memory_transcribe_audio", {"audio_path": audio_path})

    async def capture_screenshot(
        self,
        *,
        display_index: int = 0,
        delay_seconds: int = 0,
    ) -> dict[str, Any]:
        """Capture a desktop or display screenshot.

        Args:
            display_index: Index of the display to capture.
            delay_seconds: Delay before capture in seconds.
        """
        return await self._mcp_call(
            "memory_capture_screenshot",
            {"display_index": display_index, "delay_seconds": delay_seconds},
        )

    async def process_video(
        self,
        video_path: str,
        *,
        extract_frames: bool = True,
        max_frames: int = 10,
    ) -> dict[str, Any]:
        """Process a video file and extract key frames.

        Args:
            video_path: Local path to the video file.
            extract_frames: Whether to extract representative keyframes.
            max_frames: Maximum number of frames to extract.
        """
        return await self._mcp_call(
            "memory_process_video",
            {
                "video_path": video_path,
                "extract_frames": extract_frames,
                "max_frames": max_frames,
            },
        )

    async def list_media(
        self,
        *,
        media_type: str | None = None,
        limit: int = 50,
    ) -> dict[str, Any]:
        """List indexed media assets from storage.

        Args:
            media_type: Optional filter by "image", "audio", or "video".
            limit: Maximum number of records to return.
        """
        params: dict[str, Any] = {"limit": limit}
        if media_type is not None:
            params["media_type"] = media_type
        return await self._mcp_call("memory_list_media", params)

    async def search_by_image(
        self,
        image_path: str,
        *,
        limit: int = 10,
        workspace: str | None = None,
        min_score: float | None = None,
        strategy: str | None = None,
    ) -> dict[str, Any]:
        """Find memories semantically similar to an image.

        Args:
            image_path: Local path to the image file.
            limit: Maximum number of results.
            workspace: Optional workspace filter.
            min_score: Minimum similarity score filter.
            strategy: "clip", "description", or "auto".
        """
        params: dict[str, Any] = {"image_path": image_path, "limit": limit}
        if workspace is not None:
            params["workspace"] = workspace
        if min_score is not None:
            params["min_score"] = min_score
        if strategy is not None:
            params["strategy"] = strategy
        return await self._mcp_call("memory_search_by_image", params)

    async def ingest_media(
        self,
        media_path: str,
        *,
        media_type: str | None = None,
        content: str | None = None,
        workspace: str | None = None,
        tags: list[str] | None = None,
        importance: float | None = None,
    ) -> dict[str, Any]:
        """Ingest and index a media asset into a durable memory.

        Args:
            media_path: Path to the media file on disk.
            media_type: Optional explicit media type ("image", "audio", "video").
            content: Optional human or OCR description override.
            workspace: Target workspace name.
            tags: List of semantic tags.
            importance: Relative importance score (0.0 to 1.0).
        """
        params: dict[str, Any] = {"media_path": media_path}
        if media_type is not None:
            params["media_type"] = media_type
        if content is not None:
            params["content"] = content
        if workspace is not None:
            params["workspace"] = workspace
        if tags is not None:
            params["tags"] = tags
        if importance is not None:
            params["importance"] = importance
        return await self._mcp_call("memory_ingest_media", params)

    async def sync_media(
        self,
        *,
        dry_run: bool = False,
    ) -> dict[str, Any]:
        """Upload local media assets to S3/R2 cloud storage.

        Args:
            dry_run: If True, simulate upload without mutating remote storage.
        """
        return await self._mcp_call("memory_sync_media", {"dry_run": dry_run})
