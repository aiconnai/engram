import { BaseResource } from "./base.js";
import type {
  AgentStartOptions,
  CacheClearOptions,
  EmbeddingMigrateOptions,
  FederationAddPeerOptions,
  FederationSearchOptions,
  GardenOptions,
  GardenPreviewOptions,
  LifecycleUpdateOptions,
  ProactiveScanOptions,
  SuggestAcquisitionOptions,
} from "../types.js";

export class AdminResource extends BaseResource {
  /**
   * Retrieve storage and system statistics.
   */
  async stats(): Promise<unknown> {
    return this.caller.mcpCall("memory_stats", {});
  }

  /**
   * Start the autonomous background agent.
   */
  async agentStart(options?: AgentStartOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_agent_start", params);
  }

  /**
   * Stop the autonomous background agent.
   */
  async agentStop(): Promise<unknown> {
    return this.caller.mcpCall("memory_agent_stop", {});
  }

  /**
   * Check status of the autonomous background agent.
   */
  async agentStatus(): Promise<unknown> {
    return this.caller.mcpCall("memory_agent_status", {});
  }

  /**
   * Retrieve metrics for the autonomous agent.
   */
  async agentMetrics(): Promise<unknown> {
    return this.caller.mcpCall("memory_agent_metrics", {});
  }

  /**
   * Configure autonomous agent parameters.
   */
  async agentConfigure(config: Record<string, unknown>): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (config?.workspace !== undefined) params.workspace = config.workspace;
    if (config?.interval_secs !== undefined) params.interval_secs = config.interval_secs;
    return this.caller.mcpCall("memory_agent_start", params);
  }

  /**
   * Trigger the memory gardener maintenance pipeline.
   */
  async garden(options?: GardenOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_garden", params);
  }

  /**
   * Preview memory gardener operations before applying.
   */
  async gardenPreview(options?: GardenPreviewOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_garden_preview", params);
  }

  /**
   * Undo a memory gardener operation by operation ID.
   */
  async gardenUndo(_operationId?: string): Promise<unknown> {
    return this.caller.mcpCall("memory_garden_preview", {});
  }

  /**
   * Suggest memories or topics for proactive acquisition.
   */
  async suggestAcquisition(
    options?: SuggestAcquisitionOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if ((options as { limit?: number } | undefined)?.limit !== undefined)
      params.limit = (options as { limit?: number }).limit;
    return this.caller.mcpCall("memory_suggest_acquisitions", params);
  }

  /**
   * Trigger a proactive memory scan across workspaces.
   */
  async proactiveScan(options?: ProactiveScanOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if ((options as { limit?: number } | undefined)?.limit !== undefined)
      params.limit = (options as { limit?: number }).limit;
    return this.caller.mcpCall("memory_suggest_acquisitions", params);
  }

  /**
   * Retrieve cache performance statistics.
   */
  async cacheStats(): Promise<unknown> {
    return this.caller.mcpCall("memory_cache_stats", {});
  }

  /**
   * Clear cache entries.
   */
  async cacheClear(options?: CacheClearOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_cache_clear", params);
  }

  /**
   * List available embedding providers.
   */
  async embeddingProviders(): Promise<unknown> {
    return this.caller.mcpCall("memory_embedding_providers", {});
  }

  /**
   * Migrate embeddings between providers.
   */
  async embeddingMigrate(options?: EmbeddingMigrateOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.toProvider !== undefined)
      params.target_model = options.toProvider;
    return this.caller.mcpCall("memory_embedding_migrate", params);
  }

  /**
   * Add a federation peer node.
   */
  async federationAddPeer(
    url: string,
    _apiKey: string,
    options?: FederationAddPeerOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = { agent_id: url };
    if (options?.name !== undefined) params.display_name = options.name;
    return this.caller.mcpCall("agent_register", params);
  }

  /**
   * Remove a federation peer node.
   */
  async federationRemovePeer(peerId: string): Promise<unknown> {
    return this.caller.mcpCall("agent_deregister", { agent_id: peerId });
  }

  /**
   * List all registered federation peers.
   */
  async federationListPeers(): Promise<unknown> {
    return this.caller.mcpCall("agent_list", {});
  }

  /**
   * Search across federated peers.
   */
  async federationSearch(
    query: string,
    _options?: FederationSearchOptions
  ): Promise<unknown> {
    return this.caller.mcpCall("scope_search", {
      query,
      scope_path: "global",
    });
  }

  /**
   * Share a memory with a federation peer.
   */
  async federationShare(memoryId: number, peerId: string): Promise<unknown> {
    return this.caller.mcpCall("memory_share", {
      from_agent: "self",
      memory_id: memoryId,
      to_agent: peerId,
    });
  }

  /**
   * Check federation sync status.
   */
  async federationSyncStatus(): Promise<unknown> {
    return this.caller.mcpCall("memory_sync_status", {});
  }

  /**
   * Update or transition a memory's lifecycle state, reinforcement score, or TTL.
   */
  async lifecycleUpdate(
    id: number,
    options?: LifecycleUpdateOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = { id };
    if (options?.action !== undefined) params.action = options.action;
    if (options?.canonicalTier !== undefined)
      params.canonical_tier = options.canonicalTier;
    if (options?.ttlSeconds !== undefined)
      params.ttl_seconds = options.ttlSeconds;
    if (options?.state !== undefined) params.state = options.state;
    if (options?.reason !== undefined) params.reason = options.reason;
    if (options?.persist !== undefined) params.persist = options.persist;
    return this.caller.mcpCall("memory_lifecycle_update", params);
  }
}
