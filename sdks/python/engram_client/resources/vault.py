"""Vault and Markdown portability resource mixin."""

from __future__ import annotations

from typing import Any

from .base import ResourceMixin


class VaultMixin(ResourceMixin):
    """Mixin for Markdown & Obsidian vault portability operations."""

    async def vault_export(
        self,
        output_dir: str = "./memories-export",
        workspace: str = "default",
        group: str = "flat",
        include_links: bool = True,
    ) -> dict[str, Any]:
        """Export memories to Markdown files with standardized YAML frontmatter.

        Args:
            output_dir: Path to directory where markdown files will be written.
            workspace: Target workspace to export.
            group: Grouping strategy ('flat', 'day', 'workspace', 'type', 'entity').
            include_links: Whether to generate [[wikilinks]] from cross-references.

        Returns:
            Dict containing export summary (files_written, output_dir, workspace).
        """
        params: dict[str, Any] = {
            "output_dir": output_dir,
            "workspace": workspace,
            "group": group,
            "include_links": include_links,
        }
        return await self._mcp_call("memory_export_markdown", params)

    async def vault_import(
        self,
        input_dir: str = "./memories-export",
        workspace: str | None = None,
        confirm: bool = True,
        force_version: bool = False,
    ) -> dict[str, Any]:
        """Import Markdown files into Engram with SHA-256 drift detection.

        Args:
            input_dir: Path to directory containing exported markdown files.
            workspace: Optional workspace override for imported memories.
            confirm: If True, applies changes to database. If False, performs a dry-run.
            force_version: If True, overwrites even when version conflict is detected.

        Returns:
            Dict containing import report (scanned, in_sync, new, pending, conflict, applied).
        """
        params: dict[str, Any] = {
            "input_dir": input_dir,
            "confirm": confirm,
            "force_version": force_version,
        }
        if workspace is not None:
            params["workspace"] = workspace
        return await self._mcp_call("memory_import_markdown", params)

    async def vault_preview(
        self,
        input_dir: str = "./memories-export",
        workspace: str | None = None,
    ) -> dict[str, Any]:
        """Preview Markdown import without mutating the database (dry-run review mode).

        Args:
            input_dir: Path to directory containing markdown files.
            workspace: Optional workspace override.

        Returns:
            Dict containing dry-run preview report.
        """
        return await self.vault_import(
            input_dir=input_dir,
            workspace=workspace,
            confirm=False,
            force_version=False,
        )
