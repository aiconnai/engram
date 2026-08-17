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
    return this.caller.mcpCall("memory_agent_configure", { config });
  }

  /**
   * Trigger the memory gardener maintenance pipeline.
   */
  async garden(options?: GardenOptions): Promise<unknown> {
    const params: Record<string, unknown> = {
      dry_run: options?.dryRun ?? false,
    };
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
  async gardenUndo(operationId: string): Promise<unknown> {
    return this.caller.mcpCall("memory_garden_undo", { operation_id: operationId });
  }

  /**
   * Suggest memories or topics for proactive acquisition.
   */
  async suggestAcquisition(
    options?: SuggestAcquisitionOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_suggest_acquisition", params);
  }

  /**
   * Trigger a proactive memory scan across workspaces.
   */
  async proactiveScan(options?: ProactiveScanOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_proactive_scan", params);
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
    if (options?.fromProvider !== undefined)
      params.from_provider = options.fromProvider;
    if (options?.toProvider !== undefined)
      params.to_provider = options.toProvider;
    return this.caller.mcpCall("memory_embedding_migrate", params);
  }

  /**
   * Add a federation peer node.
   */
  async federationAddPeer(
    url: string,
    apiKey: string,
    options?: FederationAddPeerOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = { url, api_key: apiKey };
    if (options?.name !== undefined) params.name = options.name;
    return this.caller.mcpCall("memory_federation_add_peer", params);
  }

  /**
   * Remove a federation peer node.
   */
  async federationRemovePeer(peerId: string): Promise<unknown> {
    return this.caller.mcpCall("memory_federation_remove_peer", { peer_id: peerId });
  }

  /**
   * List all registered federation peers.
   */
  async federationListPeers(): Promise<unknown> {
    return this.caller.mcpCall("memory_federation_list_peers", {});
  }

  /**
   * Search across federated peers.
   */
  async federationSearch(
    query: string,
    options?: FederationSearchOptions
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_federation_search", {
      query,
      limit: options?.limit ?? 10,
    });
  }

  /**
   * Share a memory with a federation peer.
   */
  async federationShare(memoryId: number, peerId: string): Promise<unknown> {
    return this.caller.mcpCall("memory_federation_share", {
      memory_id: memoryId,
      peer_id: peerId,
    });
  }

  /**
   * Check federation sync status.
   */
  async federationSyncStatus(): Promise<unknown> {
    return this.caller.mcpCall("memory_federation_sync_status", {});
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
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if (options?.dryRun !== undefined) params.dry_run = options.dryRun;
    return this.caller.mcpCall("memory_lifecycle_update", params);
  }
}
