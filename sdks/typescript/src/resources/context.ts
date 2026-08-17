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
    return this.caller.mcpCall("memory_extract_facts", { id: memoryId });
  }

  /**
   * List facts with optional memory or workspace filters.
   */
  async listFacts(options?: ListFactsOptions): Promise<unknown> {
    const params: Record<string, unknown> = {
      limit: options?.limit ?? 50,
    };
    if (options?.memoryId !== undefined) params.memory_id = options.memoryId;
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_list_facts", params);
  }

  /**
   * Retrieve the fact graph for a workspace.
   */
  async factGraph(options?: FactGraphOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_fact_graph", params);
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
      token_budget: options?.tokenBudget ?? 4096,
    };
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_build_context", params);
  }

  /**
   * Format memories using a named prompt template.
   */
  async promptTemplate(
    templateName: string,
    options?: PromptTemplateOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = { template_name: templateName };
    if (options?.memories !== undefined) params.memories = options.memories;
    return this.caller.mcpCall("memory_prompt_template", params);
  }

  /**
   * Estimate the token count for given text content.
   */
  async tokenEstimate(content: string): Promise<unknown> {
    return this.caller.mcpCall("memory_token_estimate", { content });
  }

  /**
   * Retrieve a context block by type and label.
   */
  async blockGet(
    blockType: string,
    label: string,
    options?: BlockGetOptions
  ): Promise<unknown> {
    const params: Record<string, unknown> = {
      block_type: blockType,
      label,
    };
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_block_get", params);
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
      block_type: blockType,
      label,
      content,
    };
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    if (options?.reason !== undefined) params.reason = options.reason;
    return this.caller.mcpCall("memory_block_edit", params);
  }

  /**
   * List context blocks.
   */
  async blockList(options?: BlockListOptions): Promise<unknown> {
    const params: Record<string, unknown> = {};
    if (options?.blockType !== undefined)
      params.block_type = options.blockType;
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_block_list", params);
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
      block_type: blockType,
      label,
      content,
      max_tokens: options?.maxTokens ?? 2048,
    };
    if (options?.workspace !== undefined) params.workspace = options.workspace;
    return this.caller.mcpCall("memory_block_create", params);
  }
}
