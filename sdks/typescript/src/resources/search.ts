import { BaseResource } from "./base.js";
import type {
  FeedbackStatsOptions,
  MemoryCouncilOptions,
  MemoryDigestOptions,
  SearchOptions,
} from "../types.js";

export class SearchResource extends BaseResource {
  /**
   * Build an actionable, source-linked retrieval digest for a topic.
   */
  async digest(
    topic: string,
    options?: MemoryDigestOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {
      topic,
    };
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if (options?.mode !== undefined) params.mode = options.mode;
    if (options?.limit !== undefined) params.limit = options.limit;
    if (options?.relatedDepth !== undefined) params.related_depth = options.relatedDepth;
    if (options?.totalBudget !== undefined) params.total_budget = options.totalBudget;
    if (options?.includeTypes !== undefined) params.include_types = options.includeTypes;
    if (options?.timeframe !== undefined) params.timeframe = options.timeframe;
    if (options?.includeGraph !== undefined) params.include_graph = options.includeGraph;
    if (options?.includeOperationalContext !== undefined)
      params.include_operational_context = options.includeOperationalContext;
    if (options?.includeNextActions !== undefined)
      params.include_next_actions = options.includeNextActions;
    if (options?.currentGitBranch !== undefined)
      params.current_git_branch = options.currentGitBranch;
    if (options?.currentCommitHash !== undefined)
      params.current_commit_hash = options.currentCommitHash;

    return this.caller.mcpCall("memory_digest", params);
  }

  /**
   * Search memories using hybrid search (BM25 + vector + fuzzy).
   */
  async search(query: string, options?: SearchOptions): Promise<unknown> {
    const params: Record<string, unknown> = {
      query,
      limit: options?.limit ?? 10,
    };
    if (options?.workspace) params.workspace = options.workspace;
    if (options?.workspaces) params.workspaces = options.workspaces;
    if (options?.tags) params.tags = options.tags;
    if (options?.memoryType) params.memory_type = options.memoryType;
    if (options?.tier) params.tier = options.tier;
    if (options?.includeArchived !== undefined)
      params.include_archived = options.includeArchived;
    if (options?.filter) params.filter = options.filter;
    if (options?.global !== undefined) params.global = options.global;
    return this.caller.mcpCall("memory_search", params);
  }

  /**
   * Run a prompt through council consensus.
   */
  async council(
    prompt: string,
    options?: MemoryCouncilOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {
      prompt,
      include_raw_stages: options?.includeRawStages ?? true,
      persist: options?.persist ?? false,
    };

    if (options?.conversationId !== undefined)
      params.conversation_id = options.conversationId;
    if (options?.councilUrl !== undefined) params.council_url = options.councilUrl;
    if (options?.timeoutSeconds !== undefined)
      params.timeout_seconds = options.timeoutSeconds;
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if (options?.memoryTags !== undefined) params.memory_tags = options.memoryTags;

    return this.caller.mcpCall("memory_council", params);
  }

  /**
   * Explain search scoring and ranking for a set of results.
   */
  async explain(results: unknown[]): Promise<unknown> {
    return this.caller.mcpCall("memory_explain_search", { results });
  }

  /**
   * Submit relevance feedback for a search result.
   */
  async feedback(
    query: string,
    memoryId: number,
    signal: string
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_feedback", {
      query,
      memory_id: memoryId,
      signal,
    });
  }

  /**
   * Retrieve feedback statistics.
   */
  async feedbackStats(options?: FeedbackStatsOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_feedback_stats", params);
  }
}

export interface SearchCallableResource extends SearchResource {
  (query: string, options?: SearchOptions): Promise<unknown>;
}

export function createSearchCallable(
  resource: SearchResource
): SearchCallableResource {
  const callable = ((query: string, options?: SearchOptions) =>
    resource.search(query, options)) as SearchCallableResource;

  return new Proxy(callable, {
    get(target, prop, receiver) {
      if (prop in target) {
        return Reflect.get(target, prop, receiver);
      }
      const val = Reflect.get(resource, prop);
      if (typeof val === "function") {
        return val.bind(resource);
      }
      return val;
    },
  });
}
