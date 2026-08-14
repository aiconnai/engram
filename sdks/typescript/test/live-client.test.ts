import { EngramClient, EngramError } from "engram-client";

export interface LiveClientConfig {
  baseUrl: string;
  apiKey: string;
  tenant: string;
  mode: "happy" | "wrong-bearer" | "missing-endpoint";
  marker: string;
}

type JsonObject = Record<string, unknown>;

function assert(condition: boolean, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

function object(value: unknown, label: string): JsonObject {
  assert(
    typeof value === "object" && value !== null && !Array.isArray(value),
    `${label} must be an object`
  );
  return value as JsonObject;
}

function array(value: unknown, label: string): unknown[] {
  assert(Array.isArray(value), `${label} must be an array`);
  return value;
}

function number(value: unknown, label: string): number {
  assert(typeof value === "number", `${label} must be a number`);
  return value;
}

function string(value: unknown, label: string): string {
  assert(typeof value === "string", `${label} must be a string`);
  return value;
}

function toolPayload(result: unknown): unknown {
  const envelope = object(result, "tool result");
  const content = array(envelope.content, "tool result content");
  assert(content.length > 0, "tool result content must not be empty");
  const first = object(content[0], "tool result content item");
  assert(first.type === "text", "tool result must contain text content");
  return JSON.parse(string(first.text, "tool result text")) as unknown;
}

function hasMemoryId(value: unknown, expectedId: number): boolean {
  if (Array.isArray(value)) {
    return value.some((entry) => hasMemoryId(entry, expectedId));
  }
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const record = value as JsonObject;
  return (
    record.id === expectedId ||
    Object.values(record).some((entry) => hasMemoryId(entry, expectedId))
  );
}

async function expectHttpError(
  client: EngramClient,
  expectedStatus: number
): Promise<void> {
  const started = Date.now();
  try {
    await client.stats();
    throw new Error(`request unexpectedly succeeded; wanted HTTP ${expectedStatus}`);
  } catch (error: unknown) {
    assert(error instanceof EngramError, "failure must be an EngramError");
    assert(
      error.message.includes(`HTTP ${expectedStatus}`),
      `expected HTTP ${expectedStatus}, received: ${error.message}`
    );
  }
  assert(Date.now() - started < 5_000, "error request exceeded the live-test bound");
}

export async function runLiveClientContract(
  config: LiveClientConfig
): Promise<void> {
  const client = new EngramClient({
    baseUrl: config.baseUrl,
    apiKey: config.apiKey,
    tenant: config.tenant,
    timeout: 2_500,
  });

  if (config.mode === "wrong-bearer") {
    await expectHttpError(client, 401);
    return;
  }
  if (config.mode === "missing-endpoint") {
    await expectHttpError(client, 404);
    return;
  }

  const workspace = "typescript-live";
  const original = `TypeScript live package ${config.marker}`;
  const updated = `${original} updated`;

  const created = object(toolPayload(await client.create(original, {
    workspace,
    tags: ["typescript-live"],
  })), "memory_create payload");
  const memoryId = number(created.id, "created memory id");
  assert(created.content === original, "create did not preserve content");

  const fetched = object(toolPayload(await client.get(memoryId)), "memory_get payload");
  assert(fetched.id === memoryId, "get returned the wrong memory");
  assert(fetched.content === original, "get returned the wrong content");

  const listed = toolPayload(await client.list({ workspace }));
  assert(hasMemoryId(listed, memoryId), "list did not contain the created memory");

  const searched = toolPayload(await client.search(config.marker, { workspace }));
  assert(hasMemoryId(searched, memoryId), "search did not contain the created memory");

  const changed = object(
    toolPayload(await client.update(memoryId, { content: updated })),
    "memory_update payload"
  );
  assert(changed.id === memoryId, "update returned the wrong memory");
  assert(changed.content === updated, "update did not persist content");

  const deleted = object(toolPayload(await client.delete(memoryId)), "memory_delete payload");
  assert(deleted.deleted === memoryId, "delete returned the wrong memory id");
}
