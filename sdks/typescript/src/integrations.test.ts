import { describe, it, expect, vi, beforeEach } from "vitest";
import { EngramClient, integrations } from "./index.js";
import {
  EngramChatMessageHistory as LangChainChatMessageHistory,
  EngramVectorStore as LangChainVectorStore,
} from "./integrations/langchain.js";
import {
  EngramVectorStore as LlamaIndexVectorStore,
  EngramChatStore as LlamaIndexChatStore,
  EngramDocumentStore as LlamaIndexDocumentStore,
} from "./integrations/llamaindex.js";
import {
  createEngramMemoryTool,
  engramChatHistory,
  EngramChatHistoryAdapter,
} from "./integrations/ai_sdk.js";

const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

const config = {
  baseUrl: "https://test.engram.dev",
  apiKey: "test-key",
  tenant: "test-tenant",
  timeout: 5000,
};

function okResponse(result: unknown = {}) {
  return {
    ok: true,
    status: 200,
    statusText: "OK",
    json: () =>
      Promise.resolve({
        jsonrpc: "2.0",
        id: 1,
        result,
      }),
  };
}

function requestBody(index = 0) {
  return JSON.parse(mockFetch.mock.calls[index][1].body);
}

function requestArguments(index = 0) {
  return requestBody(index).params.arguments;
}

function requestMethod(index = 0) {
  return requestBody(index).params.name;
}

describe("Integrations", () => {
  let client: EngramClient;

  beforeEach(() => {
    client = new EngramClient(config);
    mockFetch.mockReset();
  });

  describe("Re-exports", () => {
    it("should re-export langchain, llamaindex, and ai_sdk from integrations namespace", () => {
      expect(integrations).toBeDefined();
      expect(integrations.langchain).toBeDefined();
      expect(integrations.llamaindex).toBeDefined();
      expect(integrations.ai_sdk).toBeDefined();
      expect(integrations.aiSdk).toBeDefined();
      expect(integrations.LangChainChatMessageHistory).toBeDefined();
      expect(integrations.LangChainVectorStore).toBeDefined();
      expect(integrations.LlamaIndexVectorStore).toBeDefined();
      expect(integrations.LlamaIndexChatStore).toBeDefined();
      expect(integrations.createEngramMemoryTool).toBeDefined();
      expect(integrations.engramChatHistory).toBeDefined();
    });
  });

  describe("LangChain Adapter", () => {
    describe("EngramChatMessageHistory", () => {
      it("should construct with string sessionId and default workspace", () => {
        const history = new LangChainChatMessageHistory(client, "session-1");
        expect(history.sessionId).toBe("session-1");
        expect(history.workspace).toBe("langchain");
      });

      it("should construct with options object and custom workspace", () => {
        const history = new LangChainChatMessageHistory(client, {
          sessionId: "session-2",
          workspace: "custom-ws",
        });
        expect(history.sessionId).toBe("session-2");
        expect(history.workspace).toBe("custom-ws");
      });

      it("should add a message with role formatting and tags", async () => {
        mockFetch.mockResolvedValueOnce(okResponse({ id: 101 }));
        const history = new LangChainChatMessageHistory(client, "sess-1");

        await history.addMessage({ type: "human", content: "Hello assistant!" });

        expect(requestMethod(0)).toBe("memory_create");
        expect(requestArguments(0).content).toBe("[human] Hello assistant!");
        expect(requestArguments(0).tags).toEqual([
          "langchain",
          "chat-history",
          "session:sess-1",
          "role:human",
        ]);
        expect(requestArguments(0).workspace).toBe("langchain");
        expect(requestArguments(0).metadata).toEqual({
          sessionId: "sess-1",
          session_id: "sess-1",
          role: "human",
        });
      });

      it("should handle _getType() duck typing from LangChain BaseMessage", async () => {
        mockFetch.mockResolvedValueOnce(okResponse({ id: 102 }));
        const history = new LangChainChatMessageHistory(client, "sess-1");

        const lcMsg = {
          _getType: () => "ai",
          content: "Hello human!",
        };
        await history.addMessage(lcMsg);

        expect(requestArguments(0).content).toBe("[ai] Hello human!");
        expect(requestArguments(0).tags).toContain("role:ai");
      });

      it("should add multiple messages in sequence", async () => {
        mockFetch.mockResolvedValue(okResponse({ id: 103 }));
        const history = new LangChainChatMessageHistory(client, "sess-multi");

        await history.addMessages([
          { type: "human", content: "Hi" },
          { type: "ai", content: "Hey" },
        ]);

        expect(mockFetch).toHaveBeenCalledTimes(2);
        expect(requestArguments(0).content).toBe("[human] Hi");
        expect(requestArguments(1).content).toBe("[ai] Hey");
      });

      it("should support addUserMessage and addAIChatMessage convenience methods", async () => {
        mockFetch.mockResolvedValue(okResponse({ id: 104 }));
        const history = new LangChainChatMessageHistory(client, "sess-conv");

        await history.addUserMessage("User question");
        await history.addAIChatMessage("AI answer");

        expect(mockFetch).toHaveBeenCalledTimes(2);
        expect(requestArguments(0).content).toBe("[human] User question");
        expect(requestArguments(1).content).toBe("[ai] AI answer");
      });

      it("should retrieve and parse messages from session memories", async () => {
        mockFetch.mockResolvedValueOnce(
          okResponse({
            memories: [
              { id: 1, content: "[human] What is Engram?" },
              { id: 2, content: "[ai] Engram is an AI memory infrastructure." },
              { id: 3, content: "Unformatted note" },
            ],
          })
        );
        const history = new LangChainChatMessageHistory(client, "sess-read");

        const messages = await history.getMessages();

        expect(requestMethod(0)).toBe("memory_search");
        expect(requestArguments(0).query).toBe("session:sess-read");
        expect(requestArguments(0).workspace).toBe("langchain");
        expect(messages).toEqual([
          { type: "human", content: "What is Engram?" },
          { type: "ai", content: "Engram is an AI memory infrastructure." },
          { type: "unknown", content: "Unformatted note" },
        ]);

        // Also test .messages getter
        mockFetch.mockResolvedValueOnce(okResponse({ memories: [] }));
        const getterMsgs = await history.messages;
        expect(getterMsgs).toEqual([]);
      });

      it("should clear all messages for the session", async () => {
        mockFetch.mockResolvedValueOnce(
          okResponse({
            memories: [{ id: 10 }, { id: 11 }],
          })
        );
        mockFetch.mockResolvedValue(okResponse({}));

        const history = new LangChainChatMessageHistory(client, "sess-clear");
        await history.clear();

        expect(requestMethod(0)).toBe("memory_search");
        expect(requestMethod(1)).toBe("memory_delete");
        expect(requestArguments(1).id).toBe(10);
        expect(requestMethod(2)).toBe("memory_delete");
        expect(requestArguments(2).id).toBe(11);
      });
    });

    describe("EngramVectorStore", () => {
      it("should add texts with metadata and return string IDs", async () => {
        mockFetch.mockResolvedValueOnce(okResponse({ id: 201 }));
        mockFetch.mockResolvedValueOnce(okResponse({ memory_id: 202 }));

        const store = new LangChainVectorStore(client, { workspace: "docs-ws" });
        const ids = await store.addTexts(
          ["Doc 1 text", "Doc 2 text"],
          [{ topic: "tech" }, { topic: "finance" }]
        );

        expect(ids).toEqual(["201", "202"]);
        expect(requestMethod(0)).toBe("memory_create");
        expect(requestArguments(0).content).toBe("Doc 1 text");
        expect(requestArguments(0).tags).toEqual(["langchain", "vector-store"]);
        expect(requestArguments(0).workspace).toBe("docs-ws");
        expect(requestArguments(0).metadata).toEqual({ topic: "tech" });

        expect(requestArguments(1).metadata).toEqual({ topic: "finance" });
      });

      it("should add LangChain documents and vectors", async () => {
        mockFetch.mockResolvedValue(okResponse({ id: 301 }));

        const store = new LangChainVectorStore(client);
        const docs = [
          { pageContent: "Doc content", metadata: { category: "ml" } },
        ];

        const ids = await store.addDocuments(docs);
        expect(ids).toEqual(["301"]);
        expect(requestArguments(0).content).toBe("Doc content");
        expect(requestArguments(0).metadata).toEqual({ category: "ml" });

        await store.addVectors([[0.1, 0.2]], docs);
        expect(mockFetch).toHaveBeenCalledTimes(2);
      });

      it("should perform similarity search with k limit and metadata filter", async () => {
        mockFetch.mockResolvedValueOnce(
          okResponse({
            memories: [
              {
                id: 1,
                content: "Doc A",
                metadata: { source: "wiki", author: "alice" },
              },
              {
                id: 2,
                content: "Doc B",
                metadata: { source: "github", author: "bob" },
              },
            ],
          })
        );

        const store = new LangChainVectorStore(client);
        const results = await store.similaritySearch("search query", 2, {
          source: "wiki",
        });

        expect(requestMethod(0)).toBe("memory_search");
        expect(requestArguments(0).query).toBe("search query");
        expect(requestArguments(0).limit).toBe(2);
        expect(requestArguments(0).filter).toEqual({ source: "wiki" });

        expect(results).toEqual([
          {
            pageContent: "Doc A",
            metadata: { source: "wiki", author: "alice" },
          },
        ]);
      });

      it("should perform similarity search with score", async () => {
        mockFetch.mockResolvedValueOnce(
          okResponse({
            memories: [
              {
                id: 1,
                content: "Doc A",
                metadata: { source: "wiki" },
                score: 0.95,
              },
            ],
          })
        );

        const store = new LangChainVectorStore(client);
        const results = await store.similaritySearchWithScore("query", 1);

        expect(results).toEqual([
          [
            {
              pageContent: "Doc A",
              metadata: { source: "wiki" },
            },
            0.95,
          ],
        ]);
      });

      it("should delete documents by ID", async () => {
        mockFetch.mockResolvedValue(okResponse({}));

        const store = new LangChainVectorStore(client);
        await store.delete({ ids: ["10", "20"] });

        expect(mockFetch).toHaveBeenCalledTimes(2);
        expect(requestMethod(0)).toBe("memory_delete");
        expect(requestArguments(0).id).toBe(10);
        expect(requestMethod(1)).toBe("memory_delete");
        expect(requestArguments(1).id).toBe(20);
      });
    });
  });

  describe("LlamaIndex Adapter", () => {
    describe("EngramVectorStore", () => {
      it("should add nodes with metadata extraction and return IDs", async () => {
        mockFetch.mockResolvedValueOnce(okResponse({ id: 501 }));
        mockFetch.mockResolvedValueOnce(okResponse({ memory: { id: 502 } }));

        const store = new LlamaIndexVectorStore(client, {
          workspace: "llama-ws",
        });

        const nodes = [
          {
            id_: "node-1",
            text: "Node 1 text",
            metadata: { section: "intro" },
            ref_doc_id: "doc-100",
            hash: "h123",
          },
          {
            node_id: "node-2",
            getContent: () => "Node 2 text",
            metadata: { section: "body" },
          },
        ];

        const ids = await store.add(nodes);
        expect(ids).toEqual(["501", "502"]);

        expect(requestMethod(0)).toBe("memory_create");
        expect(requestArguments(0).content).toBe("Node 1 text");
        expect(requestArguments(0).tags).toEqual([
          "llamaindex",
          "vector-store",
          "node:node-1",
        ]);
        expect(requestArguments(0).workspace).toBe("llama-ws");
        expect(requestArguments(0).metadata).toEqual({
          node_id: "node-1",
          node_metadata: { section: "intro" },
          node_type: "Object",
          ref_doc_id: "doc-100",
          hash: "h123",
        });

        expect(requestArguments(1).content).toBe("Node 2 text");
        expect(requestArguments(1).tags).toContain("node:node-2");
      });

      it("should execute query in DEFAULT mode", async () => {
        mockFetch.mockResolvedValueOnce(
          okResponse({
            memories: [
              {
                id: 1,
                content: "Match 1",
                score: 0.88,
              },
            ],
          })
        );

        const store = new LlamaIndexVectorStore(client);
        const res = await store.query({
          queryStr: "test query",
          similarityTopK: 3,
        });

        expect(requestMethod(0)).toBe("memory_search");
        expect(requestArguments(0).query).toBe("test query");
        expect(requestArguments(0).limit).toBe(3);
        expect(res.ids).toEqual(["1"]);
        expect(res.similarities).toEqual([0.88]);
        expect(res.nodes.length).toBe(1);
      });

      it("should execute query in SPARSE mode", async () => {
        mockFetch.mockResolvedValueOnce(okResponse({ memories: [] }));

        const store = new LlamaIndexVectorStore(client);
        await store.query({
          query_str: "sparse test",
          mode: "SPARSE",
          similarity_top_k: 2,
        });

        expect(requestArguments(0).tier).toBe("sparse");
        expect(requestArguments(0).limit).toBe(2);
      });

      it("should delete node and batch delete nodes", async () => {
        mockFetch.mockResolvedValueOnce(
          okResponse({
            memories: [{ id: 99 }],
          })
        );
        mockFetch.mockResolvedValueOnce(okResponse({}));

        const store = new LlamaIndexVectorStore(client);
        await store.delete("node-abc");

        expect(requestMethod(0)).toBe("memory_search");
        expect(requestArguments(0).query).toBe("node:node-abc");
        expect(requestMethod(1)).toBe("memory_delete");
        expect(requestArguments(1).id).toBe(99);

        mockFetch.mockReset();
        mockFetch.mockResolvedValueOnce(okResponse({ memories: [{ id: 1 }] }));
        mockFetch.mockResolvedValueOnce(okResponse({}));
        mockFetch.mockResolvedValueOnce(okResponse({ memories: [{ id: 2 }] }));
        mockFetch.mockResolvedValueOnce(okResponse({}));

        await store.deleteNodes(["n1", "n2"]);
        expect(mockFetch).toHaveBeenCalledTimes(4);
      });
    });

    describe("EngramChatStore", () => {
      it("should add message with role and session tag", async () => {
        mockFetch.mockResolvedValueOnce(okResponse({ id: 601 }));
        const chatStore = new LlamaIndexChatStore(client);

        await chatStore.addMessage("user-1", {
          role: "user",
          content: "Hello llama",
        });

        expect(requestMethod(0)).toBe("memory_create");
        expect(requestArguments(0).content).toBe("[user] Hello llama");
        expect(requestArguments(0).tags).toEqual([
          "llamaindex",
          "chat-store",
          "session:user-1",
          "role:user",
        ]);
        expect(requestArguments(0).metadata).toEqual({
          session_key: "user-1",
          sessionId: "user-1",
          role: "user",
        });
      });

      it("should get messages in order", async () => {
        mockFetch.mockResolvedValueOnce(
          okResponse({
            memories: [
              { id: 1, content: "[user] Hi" },
              { id: 2, content: "[assistant] Hello" },
            ],
          })
        );

        const chatStore = new LlamaIndexChatStore(client);
        const msgs = await chatStore.getMessages("user-1");

        expect(requestMethod(0)).toBe("memory_search");
        expect(requestArguments(0).query).toBe("session:user-1");
        expect(msgs).toEqual([
          { role: "user", content: "Hi" },
          { role: "assistant", content: "Hello" },
        ]);
      });

      it("should set messages by clearing and appending", async () => {
        mockFetch.mockResolvedValueOnce(
          okResponse({ memories: [{ id: 10 }] })
        );
        mockFetch.mockResolvedValueOnce(okResponse({}));
        mockFetch.mockResolvedValueOnce(okResponse({ id: 20 }));

        const chatStore = new LlamaIndexChatStore(client);
        await chatStore.setMessages("user-2", [
          { role: "user", content: "New conversation" },
        ]);

        expect(requestMethod(0)).toBe("memory_search");
        expect(requestMethod(1)).toBe("memory_delete");
        expect(requestArguments(1).id).toBe(10);
        expect(requestMethod(2)).toBe("memory_create");
        expect(requestArguments(2).content).toBe("[user] New conversation");
      });

      it("should delete messages and return deleted list", async () => {
        mockFetch.mockResolvedValueOnce(
          okResponse({
            memories: [
              { id: 100, content: "[user] m1" },
              { id: 101, content: "[assistant] m2" },
            ],
          })
        );
        mockFetch.mockResolvedValue(okResponse({}));

        const chatStore = new LlamaIndexChatStore(client);
        const deleted = await chatStore.deleteMessages("user-3");

        expect(deleted).toEqual([
          { role: "user", content: "m1" },
          { role: "assistant", content: "m2" },
        ]);
        expect(requestMethod(1)).toBe("memory_delete");
        expect(requestArguments(1).id).toBe(100);
        expect(requestMethod(2)).toBe("memory_delete");
        expect(requestArguments(2).id).toBe(101);
      });

      it("should delete message at specific index", async () => {
        mockFetch.mockResolvedValueOnce(
          okResponse({
            memories: [
              { id: 10, content: "[user] first" },
              { id: 11, content: "[assistant] second" },
            ],
          })
        );
        mockFetch.mockResolvedValueOnce(okResponse({}));

        const chatStore = new LlamaIndexChatStore(client);
        const deleted = await chatStore.deleteMessage("user-idx", 1);

        expect(deleted).toEqual({ role: "assistant", content: "second" });
        expect(requestMethod(1)).toBe("memory_delete");
        expect(requestArguments(1).id).toBe(11);
      });

      it("should delete last message", async () => {
        mockFetch.mockResolvedValueOnce(
          okResponse({
            memories: [
              { id: 10, content: "[user] first" },
              { id: 11, content: "[assistant] last" },
            ],
          })
        );
        mockFetch.mockResolvedValueOnce(okResponse({}));

        const chatStore = new LlamaIndexChatStore(client);
        const deleted = await chatStore.deleteLastMessage("user-last");

        expect(deleted).toEqual({ role: "assistant", content: "last" });
        expect(requestArguments(1).id).toBe(11);
      });

      it("should get unique session keys", async () => {
        mockFetch.mockResolvedValueOnce(
          okResponse({
            memories: [
              { id: 1, tags: ["session:beta", "llamaindex"] },
              { id: 2, tags: ["session:alpha", "llamaindex"] },
              { id: 3, tags: ["session:beta"] },
            ],
          })
        );

        const chatStore = new LlamaIndexChatStore(client);
        const keys = await chatStore.getKeys();

        expect(keys).toEqual(["alpha", "beta"]);
      });
    });

    describe("EngramDocumentStore", () => {
      it("should add, get, check existence, and delete documents", async () => {
        mockFetch.mockResolvedValueOnce(okResponse({ id: 1 }));
        const docStore = new LlamaIndexDocumentStore(client);

        await docStore.addDocuments([{ id_: "doc-1", text: "Doc text" }]);
        expect(requestMethod(0)).toBe("memory_create");
        expect(requestArguments(0).tags).toContain("node:doc-1");

        mockFetch.mockResolvedValueOnce(
          okResponse({ memories: [{ id: 1, content: "Doc text" }] })
        );
        const doc = await docStore.getDocument("doc-1");
        expect(doc?.content).toBe("Doc text");

        mockFetch.mockResolvedValueOnce(
          okResponse({ memories: [{ id: 1 }] })
        );
        const exists = await docStore.documentExists("doc-1");
        expect(exists).toBe(true);

        mockFetch.mockResolvedValueOnce(
          okResponse({ memories: [{ id: 1 }] })
        );
        mockFetch.mockResolvedValueOnce(okResponse({}));
        await docStore.deleteDocument("doc-1");
        expect(requestMethod(3)).toBe("memory_search");
        expect(requestMethod(4)).toBe("memory_delete");
        expect(requestArguments(4).id).toBe(1);
      });
    });
  });

  describe("Vercel AI SDK Adapter", () => {
    describe("createEngramMemoryTool", () => {
      it("should create a tool definition conforming to AI SDK schema", () => {
        const tool = createEngramMemoryTool(client, {
          workspace: "custom-memory-ws",
          description: "Custom memory retrieval tool",
        });

        expect(tool.description).toBe("Custom memory retrieval tool");
        expect(tool.parameters.type).toBe("object");
        expect(tool.parameters.properties.query).toBeDefined();
        expect(tool.parameters.required).toEqual(["query"]);
        expect(typeof tool.execute).toBe("function");
      });

      it("should execute memory search through the tool execute function", async () => {
        mockFetch.mockResolvedValueOnce(
          okResponse({
            memories: [{ id: 1, content: "Found memory" }],
          })
        );

        const tool = createEngramMemoryTool(client, {
          workspace: "agent-memories",
        });

        const result = await tool.execute({
          query: "project roadmap",
          limit: 3,
        });

        expect(requestMethod(0)).toBe("memory_search");
        expect(requestArguments(0).query).toBe("project roadmap");
        expect(requestArguments(0).limit).toBe(3);
        expect(requestArguments(0).workspace).toBe("agent-memories");
        expect(result).toEqual({
          memories: [{ id: 1, content: "Found memory" }],
        });
      });
    });

    describe("engramChatHistory", () => {
      it("should instantiate EngramChatHistoryAdapter", () => {
        const history = engramChatHistory(client, {
          sessionId: "ai-sess-1",
          workspace: "ai-ws",
        });

        expect(history).toBeInstanceOf(EngramChatHistoryAdapter);
        expect(history.sessionId).toBe("ai-sess-1");
        expect(history.workspace).toBe("ai-ws");
      });

      it("should add and retrieve messages", async () => {
        mockFetch.mockResolvedValueOnce(okResponse({ id: 701 }));
        const history = engramChatHistory(client, "ai-sess-2");

        await history.addMessage({ role: "user", content: "Hello AI SDK" });

        expect(requestMethod(0)).toBe("memory_create");
        expect(requestArguments(0).content).toBe("[user] Hello AI SDK");
        expect(requestArguments(0).tags).toEqual([
          "ai-sdk",
          "chat-history",
          "session:ai-sess-2",
          "role:user",
        ]);

        mockFetch.mockResolvedValueOnce(
          okResponse({
            memories: [
              { id: 1, content: "[user] Hello AI SDK" },
              { id: 2, content: "[assistant] Hello human" },
            ],
          })
        );

        const msgs = await history.getMessages();
        expect(msgs).toEqual([
          { role: "user", content: "Hello AI SDK" },
          { role: "assistant", content: "Hello human" },
        ]);

        mockFetch.mockResolvedValueOnce(okResponse({ memories: [] }));
        const getterMsgs = await history.messages;
        expect(getterMsgs).toEqual([]);
      });

      it("should add multiple messages and save messages (replace history)", async () => {
        mockFetch.mockResolvedValue(okResponse({ id: 801 }));
        const history = engramChatHistory(client, "ai-sess-3");

        await history.addMessages([
          { role: "user", content: "Msg 1" },
          { role: "assistant", content: "Msg 2" },
        ]);
        expect(mockFetch).toHaveBeenCalledTimes(2);

        mockFetch.mockReset();
        // saveMessages: search to clear, delete, then add
        mockFetch.mockResolvedValueOnce(okResponse({ memories: [{ id: 801 }] }));
        mockFetch.mockResolvedValueOnce(okResponse({}));
        mockFetch.mockResolvedValueOnce(okResponse({ id: 901 }));

        await history.saveMessages([{ role: "user", content: "Replacement" }]);
        expect(requestMethod(0)).toBe("memory_search");
        expect(requestMethod(1)).toBe("memory_delete");
        expect(requestMethod(2)).toBe("memory_create");
        expect(requestArguments(2).content).toBe("[user] Replacement");
      });

      it("should clear session history", async () => {
        mockFetch.mockResolvedValueOnce(
          okResponse({ memories: [{ id: 999 }] })
        );
        mockFetch.mockResolvedValueOnce(okResponse({}));

        const history = engramChatHistory(client, "ai-sess-clear");
        await history.clear();

        expect(requestMethod(0)).toBe("memory_search");
        expect(requestArguments(0).query).toBe("session:ai-sess-clear");
        expect(requestMethod(1)).toBe("memory_delete");
        expect(requestArguments(1).id).toBe(999);
      });
    });
  });
});
