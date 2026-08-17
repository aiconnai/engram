import { describe, it, expect, vi, beforeEach } from "vitest";
import {
  CouncilSkill,
  EngramClient,
  EngramError,
  MemoriesResource,
  SearchResource,
  GraphResource,
  ContextResource,
  AuthResource,
  AdminResource,
} from "./index";

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

describe("EngramClient", () => {
  let client: EngramClient;

  beforeEach(() => {
    client = new EngramClient(config);
    mockFetch.mockReset();
  });

  describe("constructor & sub-resources", () => {
    it("should instantiate all modular sub-resources", () => {
      expect(client).toBeDefined();
      expect(client.memories).toBeInstanceOf(MemoriesResource);
      expect(typeof client.search).toBe("function");
      expect(typeof client.search.search).toBe("function");
      expect(client.graph).toBeInstanceOf(GraphResource);
      expect(client.context).toBeInstanceOf(ContextResource);
      expect(client.auth).toBeInstanceOf(AuthResource);
      expect(client.admin).toBeInstanceOf(AdminResource);
    });

    it("should strip trailing slash from baseUrl", async () => {
      const slashClient = new EngramClient({
        ...config,
        baseUrl: "https://test.engram.dev/",
      });
      mockFetch.mockResolvedValueOnce(okResponse());

      await slashClient.get(123);

      expect(mockFetch).toHaveBeenCalledWith(
        "https://test.engram.dev/v1/mcp",
        expect.objectContaining({ method: "POST" })
      );
    });
  });

  describe("mcpCall", () => {
    it("should make POST request with correct headers", async () => {
      mockFetch.mockResolvedValueOnce(
        okResponse({ id: 123, content: "Test" })
      );

      const result = await client.create("test");

      expect(mockFetch).toHaveBeenCalledWith(
        "https://test.engram.dev/v1/mcp",
        expect.objectContaining({
          method: "POST",
          headers: expect.objectContaining({
            Authorization: "Bearer test-key",
            "X-Tenant-Slug": "test-tenant",
            "Content-Type": "application/json",
          }),
        })
      );
      expect(result).toEqual({ id: 123, content: "Test" });
    });

    it("should handle HTTP errors", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 404,
        statusText: "Not Found",
        text: () => Promise.resolve("Not Found"),
      });

      await expect(client.get(999)).rejects.toThrow("HTTP 404");
    });

    it("should handle JSON-RPC errors", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () =>
          Promise.resolve({
            jsonrpc: "2.0",
            id: 1,
            error: { message: "Invalid params", code: -32602 },
          }),
      });

      await expect(client.get(999)).rejects.toThrow("Invalid params");
    });

    it("should increment request IDs", async () => {
      mockFetch.mockResolvedValue(okResponse());

      await client.stats();
      await client.stats();
      await client.stats();

      expect([requestBody(0).id, requestBody(1).id, requestBody(2).id]).toEqual([
        1, 2, 3,
      ]);
    });
  });

  describe("memories resource", () => {
    it("should support modular memories.create() and direct create() / memoryCreate()", async () => {
      mockFetch.mockResolvedValue(okResponse({ id: 101 }));

      await client.memories.create("Modular memory", { workspace: "ws1" });
      expect(requestMethod(0)).toBe("memory_create");
      expect(requestArguments(0).content).toBe("Modular memory");
      expect(requestArguments(0).workspace).toBe("ws1");

      await client.memoryCreate("Direct alias memory");
      expect(requestMethod(1)).toBe("memory_create");
      expect(requestArguments(1).content).toBe("Direct alias memory");
    });

    it("should support get, update, delete, and list on memories resource", async () => {
      mockFetch.mockResolvedValue(okResponse());

      await client.memories.get(42);
      expect(requestMethod(0)).toBe("memory_get");
      expect(requestArguments(0).id).toBe(42);

      await client.memories.update(42, { content: "new content" });
      expect(requestMethod(1)).toBe("memory_update");
      expect(requestArguments(1).content).toBe("new content");

      await client.memories.delete(42);
      expect(requestMethod(2)).toBe("memory_delete");
      expect(requestArguments(2).id).toBe(42);

      await client.memories.list({ limit: 10 });
      expect(requestMethod(3)).toBe("memory_list");
      expect(requestArguments(3).limit).toBe(10);
    });

    it("should support compression, synthesis, utility, sentiment, and replay", async () => {
      mockFetch.mockResolvedValue(okResponse());

      await client.memories.createDaily("Daily note", { ttlSeconds: 3600 });
      expect(requestMethod(0)).toBe("memory_create_daily");
      expect(requestArguments(0).ttl_seconds).toBe(3600);

      await client.memories.compress(10);
      expect(requestMethod(1)).toBe("memory_compress");

      await client.memories.decompress(10);
      expect(requestMethod(2)).toBe("memory_decompress");

      await client.memories.compressForContext([1, 2], 500);
      expect(requestMethod(3)).toBe("memory_compress_for_context");
      expect(requestArguments(3).token_budget).toBe(500);

      await client.memories.consolidate("ws", { threshold: 0.9 });
      expect(requestMethod(4)).toBe("memory_consolidate");
      expect(requestArguments(4).threshold).toBe(0.9);

      await client.memories.synthesis([1, 2]);
      expect(requestMethod(5)).toBe("memory_synthesis");

      await client.memories.detectUpdates(10);
      expect(requestMethod(6)).toBe("memory_detect_updates");

      await client.memories.utilityScore(10, { signal: "upvote" });
      expect(requestMethod(7)).toBe("memory_utility_score");
      expect(requestArguments(7).signal).toBe("upvote");

      await client.memories.sentimentAnalyze(10);
      expect(requestMethod(8)).toBe("memory_sentiment_analyze");

      await client.memories.sentimentTimeline({ workspace: "ws" });
      expect(requestMethod(9)).toBe("memory_sentiment_timeline");

      await client.memories.reflect(10);
      expect(requestMethod(10)).toBe("memory_reflect");

      await client.memories.replayAtTime(10, "2026-01-01T00:00:00Z");
      expect(requestMethod(11)).toBe("memory_replay_at_time");
    });
  });

  describe("search resource", () => {
    it("should support client.search() direct and client.search.search() modular and client.memorySearch()", async () => {
      mockFetch.mockResolvedValue(okResponse());

      await client.search("query 1");
      expect(requestMethod(0)).toBe("memory_search");
      expect(requestArguments(0).query).toBe("query 1");

      await client.search.search("query 2");
      expect(requestMethod(1)).toBe("memory_search");
      expect(requestArguments(1).query).toBe("query 2");

      await client.memorySearch("query 3");
      expect(requestMethod(2)).toBe("memory_search");
      expect(requestArguments(2).query).toBe("query 3");
    });

    it("should support council, explain, feedback, feedbackStats", async () => {
      mockFetch.mockResolvedValue(okResponse());

      await client.search.council("Question?", { timeoutSeconds: 30 });
      expect(requestMethod(0)).toBe("memory_council");
      expect(requestArguments(0).timeout_seconds).toBe(30);

      await client.search.explain([{ id: 1 }]);
      expect(requestMethod(1)).toBe("memory_explain_search");

      await client.search.feedback("query", 1, "relevant");
      expect(requestMethod(2)).toBe("memory_feedback");

      await client.search.feedbackStats({ workspace: "ws" });
      expect(requestMethod(3)).toBe("memory_feedback_stats");
    });
  });

  describe("graph resource", () => {
    it("should support related, link, query, mutate, conflicts, triplets, temporal", async () => {
      mockFetch.mockResolvedValue(okResponse({ contradictions: [] }));

      await client.graph.related(5);
      expect(requestMethod(0)).toBe("memory_related");

      await client.graph.link(5, 6, "references");
      expect(requestMethod(1)).toBe("memory_link");
      expect(requestArguments(1).edge_type).toBe("references");

      await client.graph.query({ action: "traverse", depth: 2 });
      expect(requestMethod(2)).toBe("graph_query");
      expect(requestArguments(2).depth).toBe(2);

      await client.graph.mutate({ action: "link", fromId: 1, toId: 2 });
      expect(requestMethod(3)).toBe("graph_mutate");

      await client.graph.detectConflicts("ws");
      expect(requestMethod(4)).toBe("memory_detect_conflicts");

      await client.graph.resolveConflict("c1", "keep_first");
      expect(requestMethod(5)).toBe("memory_resolve_conflict");

      await client.graph.coactivationReport({ limit: 10 });
      expect(requestMethod(6)).toBe("memory_coactivation_report");

      await client.graph.queryTriplets({ subject: "Alice" });
      expect(requestMethod(7)).toBe("memory_query_triplets");

      await client.graph.addKnowledge("Alice", "knows", "Bob");
      expect(requestMethod(8)).toBe("memory_add_knowledge");

      await client.graph.temporalCreate("Alice", "Bob", "collaborates");
      expect(requestMethod(9)).toBe("memory_temporal_create");

      await client.graph.temporalInvalidate("e1", { reason: "outdated" });
      expect(requestMethod(10)).toBe("memory_temporal_invalidate");

      await client.graph.temporalSnapshot({ workspace: "ws" });
      expect(requestMethod(11)).toBe("memory_temporal_snapshot");

      await client.graph.temporalContradictions({ workspace: "ws" });
      expect(requestMethod(12)).toBe("memory_temporal_contradictions");

      await client.graph.temporalEvolve("Alice");
      expect(requestMethod(13)).toBe("memory_temporal_evolve");
    });
  });

  describe("context resource", () => {
    it("should support facts, context building, prompt templates, and blocks", async () => {
      mockFetch.mockResolvedValue(okResponse());

      await client.context.extractFacts(10);
      expect(requestMethod(0)).toBe("memory_extract_facts");

      await client.context.listFacts({ limit: 5 });
      expect(requestMethod(1)).toBe("memory_list_facts");

      await client.context.factGraph({ workspace: "ws" });
      expect(requestMethod(2)).toBe("memory_fact_graph");

      await client.context.build("summary query", { tokenBudget: 2000 });
      expect(requestMethod(3)).toBe("memory_build_context");
      expect(requestArguments(3).token_budget).toBe(2000);

      await client.context.promptTemplate("standard", { memories: [] });
      expect(requestMethod(4)).toBe("memory_prompt_template");

      await client.context.tokenEstimate("sample text");
      expect(requestMethod(5)).toBe("memory_token_estimate");

      await client.context.blockCreate("system", "main", "System prompt text");
      expect(requestMethod(6)).toBe("memory_block_create");

      await client.context.blockGet("system", "main");
      expect(requestMethod(7)).toBe("memory_block_get");

      await client.context.blockEdit("system", "main", "Updated text");
      expect(requestMethod(8)).toBe("memory_block_edit");

      await client.context.blockList({ blockType: "system" });
      expect(requestMethod(9)).toBe("memory_block_list");
    });
  });

  describe("auth resource", () => {
    it("should support identities, scopes, grants, and access checks", async () => {
      mockFetch.mockResolvedValue(okResponse());

      await client.auth.createIdentity("id-1", "Agent One", { aliases: ["one"] });
      expect(requestMethod(0)).toBe("identity_create");
      expect(requestArguments(0).display_name).toBe("Agent One");

      await client.auth.resolveIdentity("one");
      expect(requestMethod(1)).toBe("identity_resolve");

      await client.auth.scopeSet(10, "/workspaces/proj1");
      expect(requestMethod(2)).toBe("memory_scope_set");

      await client.auth.scopeGet(10);
      expect(requestMethod(3)).toBe("memory_scope_get");

      await client.auth.scopeList("/workspaces", { recursive: true });
      expect(requestMethod(4)).toBe("memory_scope_list");

      await client.auth.scopeInherit("/workspaces/sub", "/workspaces");
      expect(requestMethod(5)).toBe("memory_scope_inherit");

      await client.auth.scopeIsolate("/workspaces/sub");
      expect(requestMethod(6)).toBe("memory_scope_isolate");

      await client.auth.grantAccess("agent-x", "/workspaces/proj1", { permissions: "write" });
      expect(requestMethod(7)).toBe("memory_grant_access");

      await client.auth.revokeAccess("agent-x", "/workspaces/proj1");
      expect(requestMethod(8)).toBe("memory_revoke_access");

      await client.auth.listGrants("agent-x");
      expect(requestMethod(9)).toBe("memory_list_grants");

      await client.auth.checkAccess("agent-x", "/workspaces/proj1");
      expect(requestMethod(10)).toBe("memory_check_access");
    });
  });

  describe("admin resource", () => {
    it("should support stats, agent, gardener, cache, embeddings, federation, lifecycle", async () => {
      mockFetch.mockResolvedValue(okResponse());

      await client.admin.stats();
      expect(requestMethod(0)).toBe("memory_stats");

      await client.admin.agentStart({ workspace: "ws" });
      expect(requestMethod(1)).toBe("memory_agent_start");

      await client.admin.agentStop();
      expect(requestMethod(2)).toBe("memory_agent_stop");

      await client.admin.agentStatus();
      expect(requestMethod(3)).toBe("memory_agent_status");

      await client.admin.agentMetrics();
      expect(requestMethod(4)).toBe("memory_agent_metrics");

      await client.admin.agentConfigure({ max_concurrent: 4 });
      expect(requestMethod(5)).toBe("memory_agent_configure");

      await client.admin.garden({ dryRun: true });
      expect(requestMethod(6)).toBe("memory_garden");
      expect(requestArguments(6).dry_run).toBe(true);

      await client.admin.gardenPreview({ workspace: "ws" });
      expect(requestMethod(7)).toBe("memory_garden_preview");

      await client.admin.gardenUndo("op-123");
      expect(requestMethod(8)).toBe("memory_garden_undo");

      await client.admin.suggestAcquisition();
      expect(requestMethod(9)).toBe("memory_suggest_acquisition");

      await client.admin.proactiveScan();
      expect(requestMethod(10)).toBe("memory_proactive_scan");

      await client.admin.cacheStats();
      expect(requestMethod(11)).toBe("memory_cache_stats");

      await client.admin.cacheClear();
      expect(requestMethod(12)).toBe("memory_cache_clear");

      await client.admin.embeddingProviders();
      expect(requestMethod(13)).toBe("memory_embedding_providers");

      await client.admin.embeddingMigrate({ fromProvider: "openai", toProvider: "voyage" });
      expect(requestMethod(14)).toBe("memory_embedding_migrate");

      await client.admin.federationAddPeer("https://peer.com", "pk_123", { name: "peer-1" });
      expect(requestMethod(15)).toBe("memory_federation_add_peer");

      await client.admin.federationRemovePeer("peer-1");
      expect(requestMethod(16)).toBe("memory_federation_remove_peer");

      await client.admin.federationListPeers();
      expect(requestMethod(17)).toBe("memory_federation_list_peers");

      await client.admin.federationSearch("peer query");
      expect(requestMethod(18)).toBe("memory_federation_search");

      await client.admin.federationShare(10, "peer-1");
      expect(requestMethod(19)).toBe("memory_federation_share");

      await client.admin.federationSyncStatus();
      expect(requestMethod(20)).toBe("memory_federation_sync_status");

      await client.admin.lifecycleUpdate(10, { action: "promote" });
      expect(requestMethod(21)).toBe("memory_lifecycle_update");
    });
  });

  describe("CouncilSkill", () => {
    it("should delegate to client.memoryCouncil", async () => {
      mockFetch.mockResolvedValueOnce(okResponse({ decision: "redis" }));
      const skill = new CouncilSkill(client, { defaultWorkspace: "arch" });

      const res = await skill.ask("Redis vs PG?");
      expect(requestMethod(0)).toBe("memory_council");
      expect(requestArguments(0).prompt).toBe("Redis vs PG?");
      expect(requestArguments(0).workspace).toBe("arch");
      expect(res).toEqual({ decision: "redis" });
    });

    it("should handle askWithPersistence", async () => {
      mockFetch.mockResolvedValueOnce(okResponse({ decision: "pg" }));
      const skill = new CouncilSkill(client);

      await skill.askWithPersistence("Query");
      expect(requestArguments(0).persist).toBe(true);
    });

    it("should return error on empty prompt", async () => {
      const skill = new CouncilSkill(client);
      const res = await skill.ask("   ");
      expect(res).toEqual({ error: "prompt must be a non-empty string" });
    });
  });
});
