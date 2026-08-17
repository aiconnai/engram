import { BaseResource } from "./base";
import type {
  CreateDailyOptions,
  CreateOptions,
  CompressForContextOptions,
  ConsolidateOptions,
  ListOptions,
  MemoryReplayAtTimeOptions,
  SentimentTimelineOptions,
  UpdateOptions,
  UtilityScoreOptions,
} from "../types";

export class MemoriesResource extends BaseResource {
  /**
   * Create a new memory.
   */
  async create(content: string, options?: CreateOptions): Promise<unknown> {
    const params: Record<string, unknown> = {
      content,
      memory_type: options?.memoryType ?? "note",
    };
    if (options?.tags) params.tags = options.tags;
    if (options?.workspace) params.workspace = options.workspace;
    if (options?.metadata) params.metadata = options.metadata;
    if (options?.importance !== undefined)
      params.importance = options.importance;
    if (options?.mediaUrl !== undefined) params.media_url = options.mediaUrl;
    return this.caller.mcpCall("memory_create", params);
  }

  /**
   * Retrieve a memory by its unique numeric ID.
   */
  async get(memoryId: number): Promise<unknown> {
    return this.caller.mcpCall("memory_get", { id: memoryId });
  }

  /**
   * Update an existing memory.
   */
  async update(memoryId: number, options: UpdateOptions): Promise<unknown> {
    const params: Record<string, unknown> = { id: memoryId };
    if (options.content !== undefined) params.content = options.content;
    if (options.tags !== undefined) params.tags = options.tags;
    if (options.metadata !== undefined) params.metadata = options.metadata;
    if (options.importance !== undefined)
      params.importance = options.importance;
    if (options.mediaUrl !== undefined) params.media_url = options.mediaUrl;
    return this.caller.mcpCall("memory_update", params);
  }

  /**
   * Delete a memory by its ID.
   */
  async delete(memoryId: number): Promise<unknown> {
    return this.caller.mcpCall("memory_delete", { id: memoryId });
  }

  /**
   * List memories with optional filtering and pagination.
   */
  async list(options?: ListOptions): Promise<unknown> {
    const params: Record<string, unknown> = {
      limit: options?.limit ?? 50,
      offset: options?.offset ?? 0,
    };
    if (options?.workspace) params.workspace = options.workspace;
    if (options?.workspaces) params.workspaces = options.workspaces;
    if (options?.memoryType) params.memory_type = options.memoryType;
    if (options?.tags) params.tags = options.tags;
    if (options?.tier) params.tier = options.tier;
    if (options?.sortBy) params.sort_by = options.sortBy;
    if (options?.sortOrder) params.sort_order = options.sortOrder;
    if (options?.filter) params.filter = options.filter;
    return this.caller.mcpCall("memory_list", params);
  }

  /**
   * Create an ephemeral daily memory with TTL.
   */
  async createDaily(
    content: string,
    options?: CreateDailyOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {
      content,
      ttl_seconds: options?.ttlSeconds ?? 86400,
    };
    if (options?.tags) params.tags = options.tags;
    if (options?.workspace) params.workspace = options.workspace;
    if (options?.metadata) params.metadata = options.metadata;
    return this.caller.mcpCall("memory_create_daily", params);
  }

  /**
   * Compress a memory.
   */
  async compress(memoryId: number): Promise<unknown> {
    return this.caller.mcpCall("memory_compress", { id: memoryId });
  }

  /**
   * Decompress a previously compressed memory.
   */
  async decompress(memoryId: number): Promise<unknown> {
    return this.caller.mcpCall("memory_decompress", { id: memoryId });
  }

  /**
   * Compress multiple memories to fit within a token budget.
   */
  async compressForContext(
    memoryIds: number[],
    tokenBudget: number
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_compress_for_context", {
      memory_ids: memoryIds,
      token_budget: tokenBudget,
    });
  }

  /**
   * Consolidate memories within a workspace exceeding similarity threshold.
   */
  async consolidate(
    workspace: string,
    options?: ConsolidateOptions
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_consolidate", {
      workspace,
      threshold: options?.threshold ?? 0.8,
    });
  }

  /**
   * Synthesize insights from multiple memories.
   */
  async synthesis(memoryIds: number[]): Promise<unknown> {
    return this.caller.mcpCall("memory_synthesis", { memory_ids: memoryIds });
  }

  /**
   * Detect potential updates for a memory.
   */
  async detectUpdates(memoryId: number): Promise<unknown> {
    return this.caller.mcpCall("memory_detect_updates", { id: memoryId });
  }

  /**
   * Calculate or update utility score for a memory.
   */
  async utilityScore(
    memoryId: number,
    options?: UtilityScoreOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = { id: memoryId };
    if (options?.signal !== undefined) params.signal = options.signal;
    return this.caller.mcpCall("memory_utility_score", params);
  }

  /**
   * Analyze sentiment for a memory.
   */
  async sentimentAnalyze(memoryId: number): Promise<unknown> {
    return this.caller.mcpCall("memory_sentiment_analyze", { id: memoryId });
  }

  /**
   * Retrieve sentiment timeline for memories.
   */
  async sentimentTimeline(
    options?: SentimentTimelineOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {
      limit: options?.limit ?? 50,
    };
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_sentiment_timeline", params);
  }

  /**
   * Generate a reflection based on a memory.
   */
  async reflect(memoryId: number): Promise<unknown> {
    return this.caller.mcpCall("memory_reflect", { id: memoryId });
  }

  /**
   * Replay memory events and state at a specific historical point in time.
   */
  async replayAtTime(
    memoryId: number,
    timestamp: string,
    options?: MemoryReplayAtTimeOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {
      memory_id: memoryId,
      timestamp,
      include_events: options?.includeEvents ?? true,
      include_failed: options?.includeFailed ?? false,
      include_dry_runs: options?.includeDryRuns ?? false,
    };

    if (options?.eventType !== undefined) params.event_type = options.eventType;
    if (options?.eventLimit !== undefined) params.event_limit = options.eventLimit;

    return this.caller.mcpCall("memory_replay_at_time", params);
  }
}
