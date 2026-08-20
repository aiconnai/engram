/**
 * Vercel AI SDK integration for Engram memory engine.
 *
 * Provides:
 * - createEngramMemoryTool: AI SDK-compatible tool for retrieval of persistent memories and facts.
 * - engramChatHistory: Session chat history management for AI SDK agents.
 *
 * Usage:
 * ```ts
 * import { generateText } from "ai";
 * import { openai } from "@ai-sdk/openai";
 * import { EngramClient } from "engram-client";
 * import {
 *   createEngramMemoryTool,
 *   engramChatHistory,
 * } from "engram-client/integrations/ai_sdk";
 *
 * const client = new EngramClient({ baseUrl: "...", apiKey: "...", tenant: "my-tenant" });
 *
 * // Memory retrieval tool for AI SDK
 * const memoryTool = createEngramMemoryTool(client);
 *
 * const { text } = await generateText({
 *   model: openai("gpt-4o"),
 *   tools: {
 *     engramMemory: memoryTool,
 *   },
 *   prompt: "What were the key decisions from the product meeting?",
 * });
 *
 * // Chat history
 * const history = engramChatHistory(client, { sessionId: "user-123" });
 * await history.addMessage({ role: "user", content: "Hello AI" });
 * const messages = await history.getMessages();
 * ```
 */

import type { EngramClient } from "../client.js";
import type { SearchOptions } from "../types.js";

/**
 * Duck-typed Vercel AI SDK core message.
 */
export interface CoreChatMessage {
  role: "system" | "user" | "assistant" | "tool" | string;
  content: string | unknown;
  [key: string]: unknown;
}

/**
 * Options for configuring the Engram memory tool for Vercel AI SDK.
 */
export interface EngramMemoryToolOptions {
  /**
   * Workspace to query. Defaults to server default.
   */
  workspace?: string;
  /**
   * Custom tool description shown to the LLM.
   */
  description?: string;
}

/**
 * Options for configuring EngramChatHistory.
 */
export interface EngramChatHistoryOptions {
  sessionId: string;
  workspace?: string;
}

/**
 * Vercel AI SDK tool interface.
 */
export interface EngramMemoryTool {
  description: string;
  parameters: {
    type: "object";
    properties: {
      query: {
        type: "string";
        description: string;
      };
      limit?: {
        type: "number";
        description: string;
      };
    };
    required: string[];
  };
  execute: (args: { query: string; limit?: number }) => Promise<unknown>;
}

/**
 * Create a Vercel AI SDK-compatible tool that retrieves relevant memories from Engram.
 *
 * @param client EngramClient instance.
 * @param options Configuration options for tool.
 * @returns AI SDK Tool definition with description, parameters schema, and execute handler.
 */
export function createEngramMemoryTool(
  client: EngramClient,
  options: EngramMemoryToolOptions = {}
): EngramMemoryTool {
  const workspace = options.workspace;

  return {
    description:
      options.description ??
      "Search and retrieve persistent long-term memories, decisions, facts, and context from Engram.",
    parameters: {
      type: "object",
      properties: {
        query: {
          type: "string",
          description: "Search query or topic to retrieve memories for.",
        },
        limit: {
          type: "number",
          description: "Maximum number of memories to return (default: 5).",
        },
      },
      required: ["query"],
    },
    execute: async ({ query, limit = 5 }: { query: string; limit?: number }) => {
      const searchOptions: SearchOptions = {
        limit,
      };
      if (workspace) {
        searchOptions.workspace = workspace;
      }
      return client.search(query, searchOptions);
    },
  };
}

/**
 * Adapter for managing Vercel AI SDK chat history in Engram.
 */
export class EngramChatHistoryAdapter {
  public readonly client: EngramClient;
  public readonly sessionId: string;
  public readonly workspace: string;

  constructor(
    client: EngramClient,
    optionsOrSessionId: string | EngramChatHistoryOptions,
    workspace: string = "ai-sdk"
  ) {
    this.client = client;
    if (typeof optionsOrSessionId === "string") {
      this.sessionId = optionsOrSessionId;
      this.workspace = workspace;
    } else {
      this.sessionId = optionsOrSessionId.sessionId;
      this.workspace = optionsOrSessionId.workspace ?? workspace;
    }
  }

  /**
   * Retrieve all messages for this session from Engram.
   */
  async getMessages(): Promise<Array<{ role: string; content: string }>> {
    const result = await this.client.search(`session:${this.sessionId}`, {
      workspace: this.workspace,
      limit: 500,
    });
    const memories = extractMemories(result);
    return memories.map((mem) => {
      const contentStr = typeof mem.content === "string" ? mem.content : "";
      const { role, text } = parseMessageContent(contentStr);
      return { role, content: text };
    });
  }

  /**
   * Getter property for messages.
   */
  get messages(): Promise<Array<{ role: string; content: string }>> {
    return this.getMessages();
  }

  /**
   * Append a single chat message to the session in Engram.
   */
  async addMessage(message: CoreChatMessage): Promise<unknown> {
    const role = message.role ?? "user";
    const content =
      typeof message.content === "string"
        ? message.content
        : JSON.stringify(message.content);

    return this.client.create(`[${role}] ${content}`, {
      tags: [
        "ai-sdk",
        "chat-history",
        `session:${this.sessionId}`,
        `role:${role}`,
      ],
      workspace: this.workspace,
      metadata: {
        sessionId: this.sessionId,
        session_id: this.sessionId,
        role,
      },
    });
  }

  /**
   * Append multiple chat messages to the session in Engram.
   */
  async addMessages(messages: CoreChatMessage[]): Promise<void> {
    for (const msg of messages) {
      await this.addMessage(msg);
    }
  }

  /**
   * Replace the full message history for this session.
   */
  async saveMessages(messages: CoreChatMessage[]): Promise<void> {
    await this.clear();
    await this.addMessages(messages);
  }

  /**
   * Clear all messages for this session from Engram.
   */
  async clear(): Promise<void> {
    const result = await this.client.search(`session:${this.sessionId}`, {
      workspace: this.workspace,
      limit: 500,
    });
    const memories = extractMemories(result);
    for (const mem of memories) {
      const id = extractIdFromMemory(mem);
      if (id !== undefined) {
        await this.client.delete(Number(id));
      }
    }
  }
}

/**
 * Helper to construct an EngramChatHistoryAdapter instance.
 *
 * @param client EngramClient instance.
 * @param optionsOrSessionId Session ID string or options object.
 * @param workspace Optional workspace name.
 */
export function engramChatHistory(
  client: EngramClient,
  optionsOrSessionId: string | EngramChatHistoryOptions,
  workspace?: string
): EngramChatHistoryAdapter {
  return new EngramChatHistoryAdapter(client, optionsOrSessionId, workspace);
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

function extractMemories(result: unknown): Array<Record<string, unknown>> {
  if (Array.isArray(result)) {
    return result as Array<Record<string, unknown>>;
  }
  if (result && typeof result === "object") {
    const res = result as Record<string, unknown>;
    for (const key of ["memories", "results", "items"]) {
      if (Array.isArray(res[key])) {
        return res[key] as Array<Record<string, unknown>>;
      }
    }
    if ("id" in res || "content" in res) {
      return [res];
    }
  }
  return [];
}

function extractIdFromMemory(mem: Record<string, unknown>): string | number | undefined {
  if (mem.id !== undefined) return mem.id as string | number;
  if (mem.memory_id !== undefined) return mem.memory_id as string | number;
  return undefined;
}

function parseMessageContent(content: string): { role: string; text: string } {
  if (content.startsWith("[") && content.includes("] ")) {
    const bracketEnd = content.indexOf("] ");
    const role = content.slice(1, bracketEnd);
    const text = content.slice(bracketEnd + 2);
    return { role, text };
  }
  return { role: "unknown", text: content };
}
