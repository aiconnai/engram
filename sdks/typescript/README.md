# engram-client

[![npm](https://img.shields.io/npm/v/engram-client)](https://www.npmjs.com/package/engram-client)
[![TypeScript](https://img.shields.io/badge/TypeScript-5.3+-blue.svg)](https://www.typescriptlang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

TypeScript client for [Engram Cloud](https://github.com/aiconnai/engram-cloud) - AI memory infrastructure for agents.

## Installation

```bash
npm install engram-client
```

## Quick Start

```typescript
import { CouncilSkill, EngramClient } from "engram-client";

const client = new EngramClient({
  baseUrl: "https://your-engram-api.fly.dev",
  apiKey: "ek_...",
  tenant: "my-tenant",
});

// Create a memory
const memory = await client.create("User prefers dark mode", {
  tags: ["preferences", "ui"],
  workspace: "my-project",
});

// Search (hybrid: BM25 + vector + fuzzy)
const results = await client.search("user preferences");

// Run a council consensus session
const council = await client.memoryCouncil(
  "Should we use Redis or Postgres for caching?",
  {
    timeoutSeconds: 120,
    persist: true,
    workspace: "architecture",
  }
);

const councilSkill = new CouncilSkill(client, {
  defaultWorkspace: "architecture",
  defaultTimeoutSeconds: 120,
  defaultIncludeRawStages: true,
});

const councilWithSkill = await councilSkill.askWithPersistence(
  "Should we use Redis or Postgres for caching?"
);

// List with filters
const memories = await client.list({ limit: 20, workspace: "my-project" });

// Get by ID
const mem = await client.get(42);

// Update
await client.update(42, {
  content: "User prefers light mode",
  tags: ["preferences"],
});

// Delete
await client.delete(42);

// Stats
const stats = await client.stats();
```

## API Reference

### `new EngramClient(config)`

```typescript
interface EngramConfig {
  baseUrl: string;   // Engram Cloud URL
  apiKey: string;    // API key (ek_...)
  tenant: string;    // Tenant slug
}
```

### Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `create(content, options?)` | `Promise<Memory>` | Create a memory |
| `get(id)` | `Promise<Memory>` | Get memory by ID |
| `update(id, options)` | `Promise<Memory>` | Update a memory |
| `delete(id)` | `Promise<void>` | Delete a memory |
| `list(options?)` | `Promise<Memory[]>` | List memories with filters |
| `search(query, options?)` | `Promise<SearchResult[]>` | Hybrid search |
| `memoryCouncil(prompt, options?)` | `Promise<unknown>` | Run a prompt through llm-council |
| `memoryReplayAtTime(memoryId, timestamp, options?)` | `Promise<unknown>` | Replay a memory state at a point in time |
| `new CouncilSkill(client, options?)` | `CouncilSkill` | Reusable wrapper for repeated council sessions |
| `stats()` | `Promise<Stats>` | Storage statistics |

### `CouncilSkill` methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| `ask(prompt, options?)` | `Promise<unknown>` | Run a council session with skill defaults |
| `askWithPersistence(prompt, options?)` | `Promise<unknown>` | Force `persist: true` with defaults |

### Options

**CreateOptions:** `tags`, `workspace`, `memoryType`, `importance`, `metadata`, `mediaUrl`

**ListOptions:** `limit`, `offset`, `workspace`, `workspaces`, `memoryType`, `tags`, `tier`, `sortBy`, `sortOrder`, `filter`

- **SearchOptions:** `limit`, `workspace`, `workspaces`, `tags`, `memoryType`, `tier`, `includeArchived`, `filter`, `global`
- **MemoryCouncilOptions:** `conversationId`, `councilUrl`, `timeoutSeconds`, `includeRawStages`, `persist`, `workspace`, `memoryTags`
- **MemoryReplayAtTimeOptions:** `eventType`, `includeEvents`, `includeFailed`, `includeDryRuns`, `eventLimit`
- **CouncilSkillOptions:** `defaultWorkspace`, `defaultTimeoutSeconds`, `defaultIncludeRawStages`
- **CouncilSkillAskOptions:** `persist`, `workspace`, `timeoutSeconds`, `includeRawStages`, `conversationId`, `councilUrl`, `memoryTags`

## Error Handling

```typescript
import { EngramError } from "engram-client";

try {
  await client.get(999);
} catch (e) {
  if (e instanceof EngramError) {
    console.error(`${e.status}: ${e.message}`);
  }
}
```

## Requirements

- TypeScript >= 5.3
- Node.js >= 18 (uses native `fetch`)

## Related

- [Engram](https://github.com/aiconnai/engram) - Core memory engine (Rust)
- [Engram Cloud](https://github.com/aiconnai/engram-cloud) - Multi-tenant SaaS gateway
- [engram-client](https://pypi.org/project/engram-client/) - Python client

## License

MIT
