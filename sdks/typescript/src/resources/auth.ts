import { BaseResource } from "./base.js";
import type {
  CheckAccessOptions,
  CreateIdentityOptions,
  GrantAccessOptions,
  ScopeListOptions,
} from "../types.js";

export class AuthResource extends BaseResource {
  /**
   * Create an identity mapping with optional aliases and metadata.
   */
  async createIdentity(
    canonicalId: string,
    displayName: string,
    options?: CreateIdentityOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {
      canonical_id: canonicalId,
      display_name: displayName,
    };
    if (options?.aliases) params.aliases = options.aliases;
    if (options?.metadata) params.metadata = options.metadata;
    return this.caller.mcpCall("identity_create", params);
  }

  /**
   * Resolve an identity alias to its canonical record.
   */
  async resolveIdentity(alias: string): Promise<unknown> {
    return this.caller.mcpCall("identity_resolve", { alias });
  }

  /**
   * Set the scope path for a memory.
   */
  async scopeSet(memoryId: number, scopePath: string): Promise<void> {
    const params: Record<string, unknown> = {
      id: memoryId,
      scope_path: scopePath,
    };
    await this.caller.mcpCall("memory_scope_set", params);
  }

  /**
   * Retrieve the scope path of a memory.
   */
  async scopeGet(memoryId: number): Promise<unknown> {
    return this.caller.mcpCall("memory_scope_get", { id: memoryId });
  }

  /**
   * List memories or sub-scopes within a scope path.
   */
  async scopeList(
    scopePath: string,
    options?: ScopeListOptions
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_scope_list", {
      scope_path: scopePath,
      recursive: options?.recursive ?? false,
    });
  }

  /**
   * Inherit permissions and visibility from a parent scope path.
   */
  async scopeInherit(
    scopePath: string,
    parentPath: string
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_scope_inherit", {
      scope_path: scopePath,
      parent_path: parentPath,
    });
  }

  /**
   * Isolate a scope path from inheriting parent visibility.
   */
  async scopeIsolate(scopePath: string): Promise<unknown> {
    return this.caller.mcpCall("memory_scope_isolate", { scope_path: scopePath });
  }

  /**
   * Grant scope access permissions to an agent.
   */
  async grantAccess(
    agentId: string,
    scopePath: string,
    options?: GrantAccessOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {
      agent_id: agentId,
      scope_path: scopePath,
      permissions: options?.permissions ?? "read",
    };
    if (options?.grantedBy !== undefined) params.granted_by = options.grantedBy;
    return this.caller.mcpCall("memory_grant_access", params);
  }

  /**
   * Revoke scope access from an agent.
   */
  async revokeAccess(agentId: string, scopePath: string): Promise<unknown> {
    return this.caller.mcpCall("memory_revoke_access", {
      agent_id: agentId,
      scope_path: scopePath,
    });
  }

  /**
   * List all access grants for an agent.
   */
  async listGrants(agentId: string): Promise<unknown> {
    return this.caller.mcpCall("memory_list_grants", { agent_id: agentId });
  }

  /**
   * Check if an agent has permission on a scope path.
   */
  async checkAccess(
    agentId: string,
    scopePath: string,
    options?: CheckAccessOptions
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_check_access", {
      agent_id: agentId,
      scope_path: scopePath,
      permission: options?.permission ?? "read",
    });
  }
}
