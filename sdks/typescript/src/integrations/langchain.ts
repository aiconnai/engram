/**
 * LangChain.js integration for Engram memory engine.
 *
 * Provides:
 * - EngramChatMessageHistory: LangChain BaseChatMessageHistory interface with session isolation and role tagging.
 * - EngramVectorStore: LangChain VectorStore interface with hybrid similarity search and metadata filtering.
 *
 * Usage:
 * ```ts
 * import { EngramClient } from "engram-client";
 * import { EngramChatMessageHistory, EngramVectorStore } from "engram-client/integrations/langchain";
 *
 * const client = new EngramClient({ baseUrl: "...", apiKey: "...", tenant: "my-tenant" });
 *
 * // Chat history
 * const history = new EngramChatMessageHistory(client, { sessionId: "user-123" });
 * await history.addMessage({ type: "human", content: "Hello!" });
 * const msgs = await history.getMessages();
 *
 * // Vector store
 * const store = new EngramVectorStore(client);
 * await store.addTexts(["Paris is the capital of France."]);
 * const docs = await store.similaritySearch("capital of France");
 * ```
 */

import type { EngramClient } from "../client.js";
import type { SearchOptions } from "../types.js";

/**
 * Duck-typed LangChain BaseMessage interface.
 */
export interface LangChainMessage {
  type?: string;
  _getType?: () => string;
  role?: string;
  content: string;
  [key: string]: unknown;
}

/**
 * Options for configuring EngramChatMessageHistory.
 */
export interface EngramChatMessageHistoryOptions {
  sessionId: string;
  workspace?: string;
}

/**
 * Duck-typed LangChain Document interface.
 */
export interface LangChainDocument {
  pageContent: string;
  metadata?: Record<string, unknown>;
  id?: string;
}

/**
 * Options for configuring EngramVectorStore.
 */
export interface EngramVectorStoreOptions {
  workspace?: string;
}

/**
 * LangChain BaseChatMessageHistory implementation backed by Engram.
 *
 * Stores chat messages as memories with tags:
 * `['langchain', 'chat-history', 'session:<sessionId>', 'role:<role>']`.
 */
export class EngramChatMessageHistory {
  public readonly client: EngramClient;
  public readonly sessionId: string;
  public readonly workspace: string;

  constructor(
    client: EngramClient,
    optionsOrSessionId: string | EngramChatMessageHistoryOptions,
    workspace: string = "langchain"
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
   * Retrieve all messages for the current session from Engram.
   */
  async getMessages(): Promise<Array<{ type: string; content: string }>> {
    const result = await this.client.search(`session:${this.sessionId}`, {
      workspace: this.workspace,
      limit: 100,
    });
    const memories = extractMemories(result);
    return memories.map((mem) => {
      const contentStr = typeof mem.content === "string" ? mem.content : "";
      const { role, text } = parseMessageContent(contentStr);
      return { type: role, content: text };
    });
  }

  /**
   * Getter property mirroring Python SDK / LangChain messages getter.
   */
  get messages(): Promise<Array<{ type: string; content: string }>> {
    return this.getMessages();
  }

  /**
   * Add a message to Engram.
   *
   * @param message Message object with content and type/role attributes.
   */
  async addMessage(
    message: LangChainMessage | { type?: string; role?: string; content: string }
  ): Promise<void> {
    const maybeGetType =
      "_getType" in message && typeof (message as Record<string, unknown>)._getType === "function"
        ? ((message as Record<string, unknown>)._getType as () => string)()
        : undefined;
    const role =
      maybeGetType ??
      message.type ??
      message.role ??
      "user";
    const content = message.content ?? "";

    await this.client.create(`[${role}] ${content}`, {
      tags: [
        "langchain",
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
   * Add multiple messages to Engram in sequence.
   */
  async addMessages(
    messages: Array<LangChainMessage | { type?: string; role?: string; content: string }>
  ): Promise<void> {
    for (const msg of messages) {
      await this.addMessage(msg);
    }
  }

  /**
   * Convenience helper to add a user (human) message.
   */
  async addUserMessage(message: string): Promise<void> {
    await this.addMessage({ type: "human", content: message });
  }

  /**
   * Convenience helper to add an AI message.
   */
  async addAIChatMessage(message: string): Promise<void> {
    await this.addMessage({ type: "ai", content: message });
  }

  /**
   * Delete all messages for the current session from Engram.
   */
  async clear(): Promise<void> {
    const result = await this.client.search(`session:${this.sessionId}`, {
      workspace: this.workspace,
      limit: 100,
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
 * LangChain VectorStore implementation backed by Engram's hybrid search.
 *
 * Uses Engram's built-in embedding and hybrid BM25 + vector search.
 * Supports metadata filtering and Document conversion.
 */
export class EngramVectorStore {
  public readonly client: EngramClient;
  public readonly workspace: string;

  constructor(
    client: EngramClient,
    optionsOrWorkspace?: string | EngramVectorStoreOptions
  ) {
    this.client = client;
    if (typeof optionsOrWorkspace === "string") {
      this.workspace = optionsOrWorkspace;
    } else {
      this.workspace = optionsOrWorkspace?.workspace ?? "langchain-vectors";
    }
  }

  /**
   * Add text strings to Engram and return their memory IDs.
   *
   * @param texts Text strings to store.
   * @param metadatas Optional metadata objects, one per text.
   * @returns List of created memory IDs as strings.
   */
  async addTexts(
    texts: string[],
    metadatas?: Array<Record<string, unknown>>
  ): Promise<string[]> {
    const ids: string[] = [];
    for (let i = 0; i < texts.length; i++) {
      const text = texts[i];
      const meta = metadatas && i < metadatas.length ? metadatas[i] : {};
      const result = await this.client.create(text, {
        workspace: this.workspace,
        tags: ["langchain", "vector-store"],
        metadata: meta,
      });
      const id = extractId(result);
      ids.push(id !== undefined ? String(id) : "");
    }
    return ids;
  }

  /**
   * Add LangChain Document objects to Engram.
   *
   * @param documents List of documents to store.
   * @returns List of created memory IDs as strings.
   */
  async addDocuments(documents: LangChainDocument[]): Promise<string[]> {
    const texts = documents.map((doc) => doc.pageContent);
    const metadatas = documents.map((doc) => doc.metadata ?? {});
    return this.addTexts(texts, metadatas);
  }

  /**
   * Add vectors and documents (delegates to server-side embedding).
   */
  async addVectors(
    _vectors: number[][],
    documents: LangChainDocument[]
  ): Promise<string[]> {
    return this.addDocuments(documents);
  }

  /**
   * Search Engram using hybrid search and return Document objects.
   *
   * @param query Search query string.
   * @param k Maximum number of documents to return (default: 4).
   * @param filter Optional metadata filter object.
   * @returns List of matching LangChain documents.
   */
  async similaritySearch(
    query: string,
    k: number = 4,
    filter?: Record<string, unknown>
  ): Promise<LangChainDocument[]> {
    const searchOptions: SearchOptions = {
      workspace: this.workspace,
      limit: k,
    };
    if (filter) {
      searchOptions.filter = filter;
    }

    const result = await this.client.search(query, searchOptions);
    let memories = extractMemories(result);

    if (filter && Object.keys(filter).length > 0) {
      memories = memories.filter((mem) => matchesFilter(mem.metadata, filter));
    }

    return memories.slice(0, k).map((mem) => ({
      pageContent: typeof mem.content === "string" ? mem.content : "",
      metadata: (mem.metadata as Record<string, unknown>) ?? {},
    }));
  }

  /**
   * Search Engram and return Document objects along with relevance scores.
   *
   * @param query Search query string.
   * @param k Maximum number of documents to return (default: 4).
   * @param filter Optional metadata filter object.
   * @returns List of [Document, score] tuples.
   */
  async similaritySearchWithScore(
    query: string,
    k: number = 4,
    filter?: Record<string, unknown>
  ): Promise<Array<[LangChainDocument, number]>> {
    const searchOptions: SearchOptions = {
      workspace: this.workspace,
      limit: k,
    };
    if (filter) {
      searchOptions.filter = filter;
    }

    const result = await this.client.search(query, searchOptions);
    let memories = extractMemories(result);

    if (filter && Object.keys(filter).length > 0) {
      memories = memories.filter((mem) => matchesFilter(mem.metadata, filter));
    }

    return memories.slice(0, k).map((mem) => {
      const score =
        typeof mem.score === "number"
          ? mem.score
          : typeof mem.relevance_score === "number"
            ? mem.relevance_score
            : 0.0;
      const doc: LangChainDocument = {
        pageContent: typeof mem.content === "string" ? mem.content : "",
        metadata: (mem.metadata as Record<string, unknown>) ?? {},
      };
      return [doc, score];
    });
  }

  /**
   * Delete documents by ID from Engram.
   */
  async delete(params?: { ids?: string[] }): Promise<void> {
    if (!params?.ids) return;
    for (const idStr of params.ids) {
      const numId = Number(idStr);
      if (!Number.isNaN(numId)) {
        await this.client.delete(numId);
      }
    }
  }
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

function extractId(result: unknown): string | number | undefined {
  if (result && typeof result === "object") {
    const res = result as Record<string, unknown>;
    if (res.id !== undefined) return res.id as string | number;
    if (res.memory_id !== undefined) return res.memory_id as string | number;
    if (res.memory && typeof res.memory === "object") {
      const mem = res.memory as Record<string, unknown>;
      if (mem.id !== undefined) return mem.id as string | number;
    }
  }
  return undefined;
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

function matchesFilter(
  metadata: unknown,
  filter: Record<string, unknown>
): boolean {
  if (!metadata || typeof metadata !== "object") return false;
  const meta = metadata as Record<string, unknown>;
  for (const [key, value] of Object.entries(filter)) {
    if (meta[key] !== value) {
      return false;
    }
  }
  return true;
}
