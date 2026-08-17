"""Auth and identity resource mixin."""

from __future__ import annotations

from typing import Any

from .base import ResourceMixin


class AuthMixin(ResourceMixin):
    """Identity resolution, hierarchical scopes, and access grant operations."""

    # -- Identity --

    async def create_identity(
        self,
        canonical_id: str,
        display_name: str,
        aliases: list[str] | None = None,
        metadata: dict[str, Any] | None = None,
    ) -> dict[str, Any]:
        """Create or update an identity with optional aliases.

        Maps to the ``identity_create`` MCP tool.
        """
        params: dict[str, Any] = {
            "canonical_id": canonical_id,
            "display_name": display_name,
        }
        if aliases:
            params["aliases"] = aliases
        if metadata:
            params["metadata"] = metadata
        return await self._mcp_call("identity_create", params)

    async def resolve_identity(self, alias: str) -> dict[str, Any]:
        """Resolve an alias to its canonical identity.

        Maps to the ``identity_resolve`` MCP tool.
        """
        return await self._mcp_call("identity_resolve", {"alias": alias})

    # -- Scope Management --

    async def scope_set(self, memory_id: int, scope_path: str) -> dict[str, Any]:
        """Assign a hierarchical scope path to a memory."""
        return await self._mcp_call(
            "memory_scope_set",
            {"id": memory_id, "scope_path": scope_path},
        )

    async def scope_get(self, memory_id: int) -> dict[str, Any]:
        """Get the scope path assigned to a memory."""
        return await self._mcp_call("memory_scope_get", {"id": memory_id})

    async def scope_list(
        self,
        scope_path: str,
        *,
        recursive: bool = False,
    ) -> dict[str, Any]:
        """List memories within a scope path."""
        return await self._mcp_call(
            "memory_scope_list",
            {"scope_path": scope_path, "recursive": recursive},
        )

    async def scope_inherit(
        self,
        scope_path: str,
        parent_path: str,
    ) -> dict[str, Any]:
        """Make a scope inherit settings and policies from a parent scope."""
        return await self._mcp_call(
            "memory_scope_inherit",
            {"scope_path": scope_path, "parent_path": parent_path},
        )

    async def scope_isolate(self, scope_path: str) -> dict[str, Any]:
        """Isolate a scope so it does not inherit from any parent."""
        return await self._mcp_call("memory_scope_isolate", {"scope_path": scope_path})

    # -- Scope Grants --

    async def grant_access(
        self,
        agent_id: str,
        scope_path: str,
        *,
        permissions: str = "read",
        granted_by: str | None = None,
    ) -> dict[str, Any]:
        """Grant an agent access to a scope path."""
        params: dict[str, Any] = {
            "agent_id": agent_id,
            "scope_path": scope_path,
            "permissions": permissions,
        }
        if granted_by is not None:
            params["granted_by"] = granted_by
        return await self._mcp_call("memory_grant_access", params)

    async def revoke_access(self, agent_id: str, scope_path: str) -> dict[str, Any]:
        """Revoke an agent's access to a scope path."""
        return await self._mcp_call(
            "memory_revoke_access",
            {"agent_id": agent_id, "scope_path": scope_path},
        )

    async def list_grants(self, agent_id: str) -> dict[str, Any]:
        """List all scope grants for an agent."""
        return await self._mcp_call("memory_list_grants", {"agent_id": agent_id})

    async def check_access(
        self,
        agent_id: str,
        scope_path: str,
        *,
        permission: str = "read",
    ) -> dict[str, Any]:
        """Check whether an agent has a specific permission on a scope path."""
        return await self._mcp_call(
            "memory_check_access",
            {"agent_id": agent_id, "scope_path": scope_path, "permission": permission},
        )
