import { describe, it, expect, vi, beforeEach } from "vitest";
import { EngramClient } from "./index";

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

describe("EngramClient", () => {
  let client: EngramClient;

  beforeEach(() => {
    client = new EngramClient(config);
    mockFetch.mockReset();
  });

  describe("constructor", () => {
    it("should store config values", () => {
      expect(client).toBeDefined();
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

  describe("create", () => {
    it("should call mcpCall with correct params", async () => {
      mockFetch.mockResolvedValueOnce(okResponse({ id: 123 }));

      const result = await client.create("Hello world");

      const args = requestArguments();
      expect(args.content).toBe("Hello world");
      expect(args.memory_type).toBe("note");
      expect(result).toEqual({ id: 123 });
    });

    it("should pass all optional params", async () => {
      mockFetch.mockResolvedValueOnce(okResponse());

      await client.create("Test content", {
        memoryType: "image",
        tags: ["tag1", "tag2"],
        workspace: "my-workspace",
        metadata: { source: "test" },
        importance: 0.8,
        mediaUrl: "https://example.com/img.jpg",
      });

      const args = requestArguments();
      expect(args.content).toBe("Test content");
      expect(args.memory_type).toBe("image");
      expect(args.tags).toEqual(["tag1", "tag2"]);
      expect(args.workspace).toBe("my-workspace");
      expect(args.metadata).toEqual({ source: "test" });
      expect(args.importance).toBe(0.8);
      expect(args.media_url).toBe("https://example.com/img.jpg");
    });
  });

  describe("list", () => {
    it("should use default params", async () => {
      mockFetch.mockResolvedValueOnce(okResponse());

      await client.list();

      const args = requestArguments();
      expect(args.limit).toBe(50);
      expect(args.offset).toBe(0);
      expect(args.filter).toBeUndefined();
    });

    it("should pass advanced filters using the MCP filter field", async () => {
      mockFetch.mockResolvedValueOnce(okResponse());

      const filter = { field: "value" };
      await client.list({
        limit: 25,
        offset: 5,
        workspace: "workspace-a",
        workspaces: ["workspace-a", "workspace-b"],
        memoryType: "decision",
        tags: ["tag1"],
        tier: "permanent",
        sortBy: "importance",
        sortOrder: "asc",
        filter,
      });

      const args = requestArguments();
      expect(args.limit).toBe(25);
      expect(args.offset).toBe(5);
      expect(args.workspace).toBe("workspace-a");
      expect(args.workspaces).toEqual(["workspace-a", "workspace-b"]);
      expect(args.memory_type).toBe("decision");
      expect(args.tags).toEqual(["tag1"]);
      expect(args.tier).toBe("permanent");
      expect(args.sort_by).toBe("importance");
      expect(args.sort_order).toBe("asc");
      expect(args.filter).toEqual(filter);
    });
  });

  describe("search", () => {
    it("should call with query and default limit", async () => {
      mockFetch.mockResolvedValueOnce(okResponse());

      await client.search("test query");

      const args = requestArguments();
      expect(args.query).toBe("test query");
      expect(args.limit).toBe(10);
    });

    it("should pass search filters and optional scopes", async () => {
      mockFetch.mockResolvedValueOnce(okResponse());

      const filter = { workspace: { eq: "test" } };
      await client.search("query", {
        limit: 7,
        workspace: "test",
        workspaces: ["test", "archive"],
        tags: ["planning"],
        memoryType: "note",
        tier: "daily",
        includeArchived: true,
        filter,
        global: true,
      });

      const args = requestArguments();
      expect(args.query).toBe("query");
      expect(args.limit).toBe(7);
      expect(args.workspace).toBe("test");
      expect(args.workspaces).toEqual(["test", "archive"]);
      expect(args.tags).toEqual(["planning"]);
      expect(args.memory_type).toBe("note");
      expect(args.tier).toBe("daily");
      expect(args.include_archived).toBe(true);
      expect(args.filter).toEqual(filter);
      expect(args.global).toBe(true);
    });
  });

  describe("get/update/delete", () => {
    it("should get memory by id", async () => {
      mockFetch.mockResolvedValueOnce(okResponse());

      await client.get(123);

      const args = requestArguments();
      expect(args.id).toBe(123);
    });

    it("should update memory", async () => {
      mockFetch.mockResolvedValueOnce(okResponse());

      await client.update(123, {
        content: "Updated content",
        mediaUrl: null,
      });

      const args = requestArguments();
      expect(args.id).toBe(123);
      expect(args.content).toBe("Updated content");
      expect(args.media_url).toBeNull();
    });

    it("should delete memory", async () => {
      mockFetch.mockResolvedValueOnce(okResponse());

      await client.delete(123);

      const args = requestArguments();
      expect(args.id).toBe(123);
    });
  });
});
