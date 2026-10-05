/**
 * Offline contract tests for the TypeScript SDK over a real local HTTP socket.
 *
 * `index.test.ts` stubs the global `fetch`. These tests do not: they run the real
 * `fetch` against a node `http` server on 127.0.0.1, so the request path, headers,
 * JSON-RPC body, status handling, timeout and connection failure are exercised end
 * to end. Response bodies are copies of what `engram-server` returns (see
 * `tests/canonical_journey/contract.rs`).
 *
 * This is the OFFLINE acceptance lane (serialization and error handling). It does
 * not prove compatibility with a running server; that is the live lane,
 * `scripts/test-typescript-sdk-live.sh`, against the packed tarball.
 */
import { execFileSync } from "node:child_process";
import {
  createServer,
  type IncomingMessage,
  type Server,
  type ServerResponse,
} from "node:http";
import type { AddressInfo } from "node:net";
import { fileURLToPath } from "node:url";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { EngramClient, EngramError } from "./index.js";

interface Recorded {
  path: string;
  method: string;
  headers: IncomingMessage["headers"];
  body: {
    jsonrpc: string;
    id: number;
    method: string;
    params: { name: string; arguments: Record<string, unknown> };
  };
}

interface Reply {
  status?: number;
  body?: unknown;
  contentType?: string;
  delayMs?: number;
}

function toolResult(payload: unknown, isError = false) {
  return {
    content: [{ type: "text", text: JSON.stringify(payload, null, 2) }],
    ...(isError ? { isError: true } : {}),
  };
}

function rpcResult(result: unknown, id = 1) {
  return { jsonrpc: "2.0", id, result };
}

class Stub {
  requests: Recorded[] = [];
  responder: (request: Recorded) => Reply = () => ({
    body: rpcResult(toolResult({})),
  });
  private server: Server;
  url = "";

  constructor() {
    this.server = createServer((req, res) => void this.handle(req, res));
  }

  async start(): Promise<void> {
    await new Promise<void>((resolve) =>
      this.server.listen(0, "127.0.0.1", resolve)
    );
    this.url = `http://127.0.0.1:${(this.server.address() as AddressInfo).port}`;
  }

  async stop(): Promise<void> {
    this.server.closeAllConnections();
    await new Promise<void>((resolve) => this.server.close(() => resolve()));
  }

  private async handle(req: IncomingMessage, res: ServerResponse) {
    const chunks: Buffer[] = [];
    for await (const chunk of req) chunks.push(chunk as Buffer);
    const recorded: Recorded = {
      path: req.url ?? "",
      method: req.method ?? "",
      headers: req.headers,
      body: JSON.parse(Buffer.concat(chunks).toString() || "{}"),
    };
    this.requests.push(recorded);
    const reply = this.responder(recorded);
    if (reply.delayMs) {
      await new Promise((resolve) => setTimeout(resolve, reply.delayMs));
    }
    const payload =
      typeof reply.body === "string"
        ? reply.body
        : JSON.stringify(reply.body ?? {});
    res.writeHead(reply.status ?? 200, {
      "Content-Type": reply.contentType ?? "application/json",
    });
    res.end(payload);
  }
}

let stub: Stub;

beforeEach(async () => {
  stub = new Stub();
  await stub.start();
});

afterEach(async () => {
  await stub.stop();
});

function makeClient(timeout = 5000): EngramClient {
  return new EngramClient({
    baseUrl: stub.url,
    apiKey: "contract-key",
    tenant: "contract-tenant",
    timeout,
  });
}

function args(index = 0): Record<string, unknown> {
  return stub.requests[index].body.params.arguments;
}

/** Decode the MCP envelope the way the live contract does; the SDK returns it raw. */
function decode(envelope: unknown): unknown {
  const content = (envelope as { content: { text: string }[] }).content;
  return JSON.parse(content[0].text);
}

describe("wire format", () => {
  it("posts a tools/call to /v1/mcp with bearer and tenant headers", async () => {
    stub.responder = () => ({ body: rpcResult(toolResult({ id: 7 })) });

    await makeClient().get(7);

    const request = stub.requests[0];
    expect(request.method).toBe("POST");
    expect(request.path).toBe("/v1/mcp");
    expect(request.headers.authorization).toBe("Bearer contract-key");
    expect(request.headers["x-tenant-slug"]).toBe("contract-tenant");
    expect(request.headers["content-type"]).toContain("application/json");
    expect(request.body.jsonrpc).toBe("2.0");
    expect(request.body.method).toBe("tools/call");
    expect(request.body.params).toEqual({
      name: "memory_get",
      arguments: { id: 7 },
    });
  });

  it("sends snake_case argument keys for camelCase options", async () => {
    const client = makeClient();
    await client.create("content", {
      memoryType: "issue",
      workspace: "ws",
      mediaUrl: "https://example.invalid/a.png",
    });
    await client.list({ workspace: "ws", memoryType: "issue", sortBy: "created_at" });

    for (const [index] of stub.requests.entries()) {
      const camel = Object.keys(args(index)).filter((key) => /[A-Z-]/.test(key));
      expect(camel, `camelCase keys on the wire: ${camel}`).toEqual([]);
    }
    expect(args(0)).toMatchObject({
      memory_type: "issue",
      media_url: "https://example.invalid/a.png",
    });
  });
});

describe("pagination", () => {
  it("defaults list to limit 50 offset 0 and forwards explicit paging", async () => {
    stub.responder = () => ({ body: rpcResult(toolResult([])) });
    const client = makeClient();

    await client.list();
    await client.list({ limit: 2, offset: 4, workspace: "ws" });

    expect(stub.requests[0].body.params.name).toBe("memory_list");
    expect(args(0)).toMatchObject({ limit: 50, offset: 0 });
    expect(args(1)).toMatchObject({ limit: 2, offset: 4, workspace: "ws" });
  });

  it("forwards the search limit", async () => {
    stub.responder = () => ({ body: rpcResult(toolResult([])) });

    await makeClient().search("query", { limit: 3, workspace: "ws" });

    expect(stub.requests[0].body.params.name).toBe("memory_search");
    expect(args(0)).toMatchObject({ limit: 3 });
  });
});

describe("envelopes and errors", () => {
  it("returns the raw MCP envelope on success; callers decode content[0].text", async () => {
    stub.responder = () => ({ body: rpcResult(toolResult({ id: 7, content: "x" })) });

    const result = await makeClient().get(7);

    expect(result).toEqual(toolResult({ id: 7, content: "x" }));
    expect(decode(result)).toEqual({ id: 7, content: "x" });
  });

  it("does NOT throw on a tool error envelope (isError); it is returned as data", async () => {
    // Characterization. The Python SDK raises EngramError here; this one does not
    // (coverage-map gap G-4). If the TypeScript SDK ever decodes isError, update this test.
    const normalized = {
      error: {
        code: "permission_denied",
        message: "memory_delete requires admin mode",
        tool: "memory_delete",
        current_mode: "read_only",
        required_mode: "admin",
      },
    };
    stub.responder = () => ({ body: rpcResult(toolResult(normalized, true)) });

    const result = (await makeClient().delete(9)) as { isError?: boolean };

    expect(result.isError).toBe(true);
    expect(decode(result)).toEqual(normalized);
  });

  it("throws EngramError with the JSON-RPC message on a JSON-RPC error", async () => {
    stub.responder = () => ({
      body: { jsonrpc: "2.0", id: 1, error: { code: -32601, message: "Method not found: x" } },
    });

    await expect(makeClient().get(1)).rejects.toThrow(EngramError);
    await expect(makeClient().get(1)).rejects.toThrow("Method not found: x");
  });

  it.each([
    [401, { jsonrpc: "2.0", error: { code: -32001, message: "Unauthorized" } }, "application/json"],
    [400, "Failed to parse the request body as JSON", "text/plain"],
    [500, "boom", "text/plain"],
  ])("throws EngramError 'HTTP %i' for a failing status", async (status, body, contentType) => {
    stub.responder = () => ({ status, body, contentType });

    const error = await makeClient().get(1).catch((caught: unknown) => caught);

    expect(error).toBeInstanceOf(EngramError);
    expect((error as Error).message).toContain(`HTTP ${status}`);
  });
});

describe("timeout, connection failure and lifecycle", () => {
  it("aborts a slow call within the timeout and stays usable afterwards", async () => {
    stub.responder = () =>
      stub.requests.length === 1
        ? { body: rpcResult(toolResult({ id: 1 })), delayMs: 1500 }
        : { body: rpcResult(toolResult({ id: 2 })) };
    const client = makeClient(200);

    const started = Date.now();
    const error = await client.get(1).catch((caught: unknown) => caught);

    expect(Date.now() - started).toBeLessThan(1200);
    // Characterization: the abort surfaces as the platform AbortError, not EngramError
    // (the Python SDK wraps it; coverage-map gap G-5).
    expect(error).toBeInstanceOf(Error);
    expect(error).not.toBeInstanceOf(EngramError);
    expect((error as Error).name).toBe("AbortError");
    expect(decode(await client.get(2))).toEqual({ id: 2 });
  });

  it("rejects when nothing listens on the port", async () => {
    const closed = new EngramClient({
      baseUrl: "http://127.0.0.1:1",
      apiKey: "k",
      tenant: "t",
      timeout: 1000,
    });

    await expect(closed.get(1)).rejects.toThrow();
  });

  it("numbers concurrent requests uniquely", async () => {
    stub.responder = (request) => ({
      body: rpcResult(toolResult({ echo: request.body.id }), request.body.id),
    });
    const client = makeClient();

    const results = await Promise.all(
      Array.from({ length: 25 }, (_, index) => client.get(index))
    );

    const sent = stub.requests.map((request) => request.body.id);
    expect(new Set(sent).size).toBe(25);
    expect(
      results.map((result) => (decode(result) as { echo: number }).echo).sort()
    ).toEqual([...sent].sort());
  });
});

describe("alignment with the MCP registry", () => {
  it("makes only calls that match the registry baseline", () => {
    const root = fileURLToPath(new URL("../../..", import.meta.url));
    // Throws (and fails the test) on a non-zero exit, printing the drift lines.
    const output = execFileSync(
      "python3",
      ["scripts/check-sdk-contract-alignment.py", "--only", "typescript"],
      { cwd: root, encoding: "utf8", timeout: 120_000 }
    );
    expect(output).toContain("sdk contract alignment: OK");
  });
});
