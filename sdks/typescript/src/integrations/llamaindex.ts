/**
 * LlamaIndex.ts integration for Engram memory engine.
 *
 * Provides:
 * - EngramVectorStore: LlamaIndex BaseVectorStore-compatible class backed by Engram's hybrid search.
 * - EngramChatStore: LlamaIndex BaseChatStore-compatible class for persisting session chat history.
 * - EngramDocumentStore: LlamaIndex BaseDocumentStore-compatible class for node/document storage.
 *
 * Usage:
 * ```ts
 * import { EngramClient } from "engram-client";
 * import {
 *   EngramVectorStore,
 *   EngramChatStore,
 * } from "engram-client/integrations/llamaindex";
 *
 * const client = new EngramClient({ baseUrl: "...", apiKey: "...", tenant: "my-tenant" });
 *
 * // Vector store
 * const vectorStore = new EngramVectorStore(client);
 * const ids = await vectorStore.add([{ id_: "n1", text: "LlamaIndex node text" }]);
 * const res = await vectorStore.query({ queryStr: "node text", similarityTopK: 5 });
 *
 * // Chat store
 * const chatStore = new EngramChatStore(client);
 * await chatStore.setMessages("user-123", [{ role: "user", content: "Hello!" }]);
 * const messages = await chatStore.getMessages("user-123");
 * ```
 */

import type { EngramClient } from "../client.js";
import type { SearchOptions } from "../types.js";

/**
 * Duck-typed LlamaIndex node/document.
 */
export interface LlamaIndexNode {
  id_?: string;
  node_id?: string;
  id?: string;
  doc_id?: string;
  text?: string;
  getContent?: () => string;
  metadata?: Record<string, unknown>;
  ref_doc_id?: string;
  refDocId?: string;
  hash?: string;
  [key: string]: unknown;
}

/**
 * Duck-typed LlamaIndex VectorStoreQuery.
 */
export interface LlamaIndexQuery {
  queryStr?: string;
  query_str?: string;
  similarityTopK?: number;
  similarity_top_k?: number;
  mode?: string;
  [key: string]: unknown;
}

/**
 * Result structure returned by LlamaIndex VectorStore queries.
 */
export interface LlamaIndexQueryResult {
  nodes: Array<Record<string, unknown>>;
  similarities: number[];
  ids: string[];
}

/**
 * Duck-typed LlamaIndex ChatMessage.
 */
export interface LlamaIndexChatMessage {
  role: string;
  content: string;
  [key: string]: unknown;
}

/**
 * Options for configuring LlamaIndex adapters.
 */
export interface EngramLlamaIndexOptions {
  workspace?: string;
}

/**
 * LlamaIndex VectorStore implementation backed by Engram's hybrid search.
 *
 * Stores LlamaIndex nodes as Engram memories and delegates similarity queries
 * to Engram's built-in hybrid search (BM25 + vector).
 */
export class EngramVectorStore {
  public readonly client: EngramClient;
  public readonly workspace: string;
  public readonly storesText: boolean = true;
  public readonly isEmbeddingQuery: boolean = false;

  constructor(
    client: EngramClient,
    optionsOrWorkspace?: string | EngramLlamaIndexOptions
  ) {
    this.client = client;
    if (typeof optionsOrWorkspace === "string") {
      this.workspace = optionsOrWorkspace;
    } else {
      this.workspace = optionsOrWorkspace?.workspace ?? "llamaindex-vectors";
    }
  }

  /**
   * Add LlamaIndex nodes to Engram and return their memory IDs.
   *
   * @param nodes List of LlamaIndex nodes to store.
   * @returns Array of created memory IDs as strings.
   */
  async add(nodes: LlamaIndexNode[]): Promise<string[]> {
    const ids: string[] = [];
    for (const node of nodes) {
      const nodeId = getNodeDocId(node);
      const content = getNodeContent(node);
      const meta = buildNodeMetadata(node);

      const result = await this.client.create(content, {
        tags: ["llamaindex", "vector-store", `node:${nodeId}`],
        workspace: this.workspace,
        metadata: meta,
      });

      const memId = extractId(result);
      ids.push(memId !== undefined ? String(memId) : "");
    }
    return ids;
  }

  /**
   * Delete a node by ID from Engram.
   *
   * @param nodeId Node ID to delete.
   */
  async delete(nodeId: string): Promise<void> {
    const result = await this.client.search(`node:${nodeId}`, {
      workspace: this.workspace,
      limit: 10,
    });
    const memories = extractMemories(result);
    for (const mem of memories) {
      const memId = extractIdFromMemory(mem);
      if (memId !== undefined) {
        await this.client.delete(Number(memId));
      }
    }
  }

  /**
   * Batch delete nodes by their IDs.
   *
   * @param nodeIds List of node IDs to delete.
   */
  async deleteNodes(nodeIds: string[]): Promise<void> {
    for (const nodeId of nodeIds) {
      await this.delete(nodeId);
    }
  }

  /**
   * Execute a vector store query against Engram hybrid search.
   *
   * @param query Query object or query string.
   * @returns Query result with nodes, similarities, and ids.
   */
  async query(
    query: LlamaIndexQuery | string
  ): Promise<LlamaIndexQueryResult> {
    const queryStr =
      typeof query === "string"
        ? query
        : query.queryStr ?? query.query_str ?? "";
    const limit =
      typeof query === "object"
        ? query.similarityTopK ?? query.similarity_top_k ?? 4
        : 4;
    const mode = typeof query === "object" ? query.mode ?? "DEFAULT" : "DEFAULT";

    const searchOptions: SearchOptions = {
      workspace: this.workspace,
      limit,
    };

    if (String(mode).toUpperCase() === "SPARSE") {
      searchOptions.tier = "sparse";
    }

    const result = await this.client.search(queryStr, searchOptions);
    const memories = extractMemories(result);

    const nodes: Array<Record<string, unknown>> = [];
    const similarities: number[] = [];
    const ids: string[] = [];

    for (const mem of memories) {
      nodes.push(mem);
      const score =
        typeof mem.score === "number"
          ? mem.score
          : typeof mem.relevance_score === "number"
            ? mem.relevance_score
            : 0.0;
      similarities.push(Number(score));
      const memId = extractIdFromMemory(mem);
      ids.push(memId !== undefined ? String(memId) : "");
    }

    return { nodes, similarities, ids };
  }
}

/**
 * LlamaIndex ChatStore implementation backed by Engram.
 *
 * Persists chat messages as Engram memories with session key tags.
 */
export class EngramChatStore {
  public readonly client: EngramClient;
  public readonly workspace: string;

  constructor(
    client: EngramClient,
    optionsOrWorkspace?: string | EngramLlamaIndexOptions
  ) {
    this.client = client;
    if (typeof optionsOrWorkspace === "string") {
      this.workspace = optionsOrWorkspace;
    } else {
      this.workspace = optionsOrWorkspace?.workspace ?? "llamaindex-chat";
    }
  }

  /**
   * Replace the message list for a session key.
   */
  async setMessages(
    key: string,
    messages: LlamaIndexChatMessage[]
  ): Promise<void> {
    await this.deleteMessages(key);
    for (const msg of messages) {
      await this.addMessage(key, msg);
    }
  }

  /**
   * Retrieve all messages for a session key in chronological order.
   */
  async getMessages(key: string): Promise<LlamaIndexChatMessage[]> {
    const result = await this.client.search(`session:${key}`, {
      workspace: this.workspace,
      limit: 500,
    });
    const memories = extractMemories(result);
    const output: LlamaIndexChatMessage[] = [];
    for (const mem of memories) {
      const contentStr = typeof mem.content === "string" ? mem.content : "";
      const { role, text } = parseMessageContent(contentStr);
      output.push({ role, content: text });
    }
    return output;
  }

  /**
   * Append a single message to a session.
   */
  async addMessage(
    key: string,
    message: LlamaIndexChatMessage
  ): Promise<void> {
    const role = message.role ?? "user";
    const content = message.content ?? "";
    await this.client.create(`[${role}] ${content}`, {
      tags: [
        "llamaindex",
        "chat-store",
        `session:${key}`,
        `role:${role}`,
      ],
      workspace: this.workspace,
      metadata: {
        session_key: key,
        sessionId: key,
        role,
      },
    });
  }

  /**
   * Delete all messages for a session key and return the deleted list.
   */
  async deleteMessages(
    key: string
  ): Promise<LlamaIndexChatMessage[] | null> {
    const result = await this.client.search(`session:${key}`, {
      workspace: this.workspace,
      limit: 500,
    });
    const memories = extractMemories(result);
    if (memories.length === 0) return null;

    const deleted: LlamaIndexChatMessage[] = [];
    for (const mem of memories) {
      const contentStr = typeof mem.content === "string" ? mem.content : "";
      const { role, text } = parseMessageContent(contentStr);
      deleted.push({ role, content: text });

      const memId = extractIdFromMemory(mem);
      if (memId !== undefined) {
        await this.client.delete(Number(memId));
      }
    }
    return deleted;
  }

  /**
   * Delete the message at a specific index within a session.
   */
  async deleteMessage(
    key: string,
    idx: number
  ): Promise<LlamaIndexChatMessage | null> {
    const result = await this.client.search(`session:${key}`, {
      workspace: this.workspace,
      limit: 500,
    });
    const memories = extractMemories(result);
    if (idx < 0 || idx >= memories.length) return null;

    const mem = memories[idx];
    const contentStr = typeof mem.content === "string" ? mem.content : "";
    const { role, text } = parseMessageContent(contentStr);

    const memId = extractIdFromMemory(mem);
    if (memId !== undefined) {
      await this.client.delete(Number(memId));
    }

    return { role, content: text };
  }

  /**
   * Delete the last message in a session.
   */
  async deleteLastMessage(
    key: string
  ): Promise<LlamaIndexChatMessage | null> {
    const result = await this.client.search(`session:${key}`, {
      workspace: this.workspace,
      limit: 500,
    });
    const memories = extractMemories(result);
    if (memories.length === 0) return null;

    const mem = memories[memories.length - 1];
    const contentStr = typeof mem.content === "string" ? mem.content : "";
    const { role, text } = parseMessageContent(contentStr);

    const memId = extractIdFromMemory(mem);
    if (memId !== undefined) {
      await this.client.delete(Number(memId));
    }

    return { role, content: text };
  }

  /**
   * Return all unique session keys known to this chat store.
   */
  async getKeys(): Promise<string[]> {
    const result = await this.client.search("llamaindex chat-store", {
      workspace: this.workspace,
      limit: 1000,
    });
    const memories = extractMemories(result);
    const keys = new Set<string>();

    for (const mem of memories) {
      const tags = Array.isArray(mem.tags) ? (mem.tags as string[]) : [];
      for (const tag of tags) {
        if (typeof tag === "string" && tag.startsWith("session:")) {
          keys.add(tag.slice("session:".length));
        }
      }
    }

    return Array.from(keys).sort();
  }
}

/**
 * LlamaIndex DocumentStore implementation backed by Engram.
 */
export class EngramDocumentStore {
  public readonly client: EngramClient;
  public readonly workspace: string;

  constructor(
    client: EngramClient,
    optionsOrWorkspace?: string | EngramLlamaIndexOptions
  ) {
    this.client = client;
    if (typeof optionsOrWorkspace === "string") {
      this.workspace = optionsOrWorkspace;
    } else {
      this.workspace = optionsOrWorkspace?.workspace ?? "llamaindex-docs";
    }
  }

  async addDocuments(docs: LlamaIndexNode[]): Promise<void> {
    for (const node of docs) {
      const nodeId = getNodeDocId(node);
      const content = getNodeContent(node);
      const meta = buildNodeMetadata(node);
      await this.client.create(content, {
        tags: ["llamaindex", "docstore", `node:${nodeId}`],
        workspace: this.workspace,
        metadata: meta,
      });
    }
  }

  async getDocument(docId: string): Promise<Record<string, unknown> | null> {
    const result = await this.client.search(`node:${docId}`, {
      workspace: this.workspace,
      limit: 1,
    });
    const memories = extractMemories(result);
    return memories.length > 0 ? memories[0] : null;
  }

  async documentExists(docId: string): Promise<boolean> {
    const doc = await this.getDocument(docId);
    return doc !== null;
  }

  async deleteDocument(docId: string): Promise<void> {
    const result = await this.client.search(`node:${docId}`, {
      workspace: this.workspace,
      limit: 10,
    });
    const memories = extractMemories(result);
    for (const mem of memories) {
      const memId = extractIdFromMemory(mem);
      if (memId !== undefined) {
        await this.client.delete(Number(memId));
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

function getNodeDocId(node: LlamaIndexNode): string {
  for (const attr of ["node_id", "id_", "id", "doc_id"] as const) {
    const val = node[attr];
    if (val !== undefined && val !== null) {
      return String(val);
    }
  }
  return String(Math.random().toString(36).slice(2, 10));
}

function getNodeContent(node: LlamaIndexNode): string {
  if (typeof node.getContent === "function") {
    return String(node.getContent());
  }
  if (typeof node.text === "string") {
    return node.text;
  }
  return typeof node.content === "string" ? node.content : JSON.stringify(node);
}

function buildNodeMetadata(node: LlamaIndexNode): Record<string, unknown> {
  const meta: Record<string, unknown> = {};
  meta.node_id = getNodeDocId(node);
  if (node.metadata && typeof node.metadata === "object") {
    meta.node_metadata = node.metadata;
  }
  meta.node_type = node.constructor ? node.constructor.name : "Node";
  const refDocId = node.ref_doc_id ?? node.refDocId;
  if (refDocId !== undefined && refDocId !== null) {
    meta.ref_doc_id = String(refDocId);
  }
  if (node.hash !== undefined && node.hash !== null) {
    meta.hash = String(node.hash);
  }
  return meta;
}

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
