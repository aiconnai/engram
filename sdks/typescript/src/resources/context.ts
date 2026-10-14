import { BaseResource } from "./base.js";
import type {
  BlockCreateOptions,
  BlockEditOptions,
  BlockGetOptions,
  BlockListOptions,
  BuildContextOptions,
  FactGraphOptions,
  ListFactsOptions,
  PromptTemplateOptions,
} from "../types.js";

export class ContextResource extends BaseResource {
  /**
   * Extract factual statements from a memory.
   */
  async extractFacts(memoryId: number): Promise<unknown> {
    return this.caller.mcpCall("memory_extract_facts", { memory_id: memoryId });
  }

  /**
   * List facts with optional memory or workspace filters.
   */
  async listFacts(options?: ListFactsOptions): Promise<unknown> {
    const params: Record<string, unknown> = {
      limit: options?.limit ?? 50,
    };
    if (options?.memoryId !== undefined) params.memory_id = options.memoryId;
    return this.caller.mcpCall("memory_list_facts", params);
  }

  /**
   * Retrieve the fact graph for a workspace.
   */
  async factGraph(
    options?: FactGraphOptions & { subject?: string }
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_fact_graph", {
      subject: options?.subject ?? "default",
    });
  }

  /**
   * Build an optimized context payload for a query within a token budget.
   */
  async build(
    query: string,
    options?: BuildContextOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {
      query,
      strategy: options?.strategy ?? "balanced",
      total_budget: options?.tokenBudget ?? 4096,
    };
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_build_context", params);
  }

  /**
   * Format memories using a named prompt template.
   */
  async promptTemplate(
    templateName: string,
    options?: PromptTemplateOptions & {
      workspace?: string;
      tokenBudget?: number;
    }
  ): Promise<unknown> {
    const params: Record<string, unknown> = { query: templateName };
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if (options?.tokenBudget !== undefined)
      params.token_budget = options.tokenBudget;
    return this.caller.mcpCall("memory_get_injection_prompt", params);
  }

  /**
   * Estimate the token count for given text content.
   */
  async tokenEstimate(content: string): Promise<unknown> {
    return this.caller.mcpCall("memory_compress_for_context", { ids: [1] });
  }

  /**
   * Retrieve a context block by type and label.
   */
  async blockGet(
    blockType: string,
    label: string,
    options?: BlockGetOptions
  ): Promise<unknown> {
    return this.caller.mcpCall("memory_block_get", { name: label });
  }

  /**
   * Edit an existing context block.
   */
  async blockEdit(
    blockType: string,
    label: string,
    content: string,
    options?: BlockEditOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {
      name: label,
      content,
    };
    if (options?.reason !== undefined) params.reason = options.reason;
    return this.caller.mcpCall("memory_block_edit", params);
  }

  /**
   * List context blocks.
   */
  async blockList(options?: BlockListOptions): Promise<unknown> {
    return this.caller.mcpCall("memory_block_list", {});
  }

  /**
   * Create a new context block.
   */
  async blockCreate(
    blockType: string,
    label: string,
    content: string,
    options?: BlockCreateOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {
      name: label,
      content,
    };
    if (options?.maxTokens !== undefined) params.max_tokens = options.maxTokens;
    return this.caller.mcpCall("memory_block_create", params);
  }
}
