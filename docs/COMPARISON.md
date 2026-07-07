# Choosing a Memory Layer: Engram vs Alternatives

An honest comparison. Every factual claim about a competitor below was checked
against a primary source (official pricing page, GitHub repo, or official docs)
in July 2026. Competitors evolve quickly — treat this as a snapshot and check
their sites for current details.

## TL;DR

Engram is the **local-first, MCP-native memory server for teams and coding
agents working on proprietary context**. It stores memory in SQLite, searches
it with hybrid BM25/vector/fuzzy retrieval, and applies deterministic salience,
lifecycle, and quality rules — **without LLM-based fact extraction in the
default write path**. Writes cost no model tokens; behavior is auditable and
reproducible.

If you want a managed SaaS with automatic "magic" fact extraction, or a
state-of-the-art temporal knowledge graph, other tools fit better. That is the
point of this page.

## Choose Engram when…

- Memory must stay **local and private** (single Rust binary, SQLite + WAL,
  optional encrypted S3/R2 sync).
- **MCP is your primary interface** — Claude Code, Cursor, VS Code MCP clients
  query the same store your CLI, HTTP API, and SDKs use.
- You need **provenance and auditability**: what was stored, when, from where,
  and why it ranks the way it does (`memory_explain_search`).
- You want **deterministic memory policy** — salience, decay, retention, and
  lifecycle transitions computed by rules, not by a model call per write.
- Derived memory should be **reviewed before it becomes canonical** (the dream
  snapshot pipeline proposes; a human or agent confirms).
- Operating cost matters: no per-write LLM call, no managed database, no
  per-request pricing.

## Choose an alternative when…

### Mem0 — managed SaaS with automatic fact extraction

Mem0 pairs an Apache-2.0 OSS library with a hosted platform (Free tier;
Starter $19/mo, Growth $79/mo, Pro $249/mo, usage-based Enterprise). Its
`add()` pipeline runs an LLM to extract salient facts and decide ADD/UPDATE
operations — one model call per write in its current single-pass design. Graph
support is built-in entity linking (the earlier external graph-store
requirement was removed); it is not a full temporal knowledge graph. Mem0 also
offers a hosted MCP server.

**Pick Mem0 if** you want a mature managed service that turns raw conversation
into structured facts automatically and you accept per-write model cost and a
cloud dependency. **Pick Engram if** you want the extraction step under your
control (or skipped), local storage, and zero marginal cost per write.

### Zep / Graphiti — best-in-class temporal knowledge graph

Graphiti (Apache-2.0, the engine behind Zep's commercial platform) builds a
bi-temporal knowledge graph: facts carry validity windows, superseded facts are
invalidated rather than deleted, and retrieval is hybrid without LLM calls on
the read path. The write path requires an LLM for entity/edge extraction, and
self-hosting requires a graph database (Neo4j or FalkorDB, typically via
Docker).

**Pick Zep/Graphiti if** point-in-time reasoning over an evolving fact graph is
your core requirement and you can run the infra. **Pick Engram if** you need
durable operational memory with temporal edges and conflict detection at a
fraction of the operational weight.

### Cognee — corpus-to-knowledge-graph pipelines

Cognee is open source and self-hostable for free, with a usage-based cloud
(free tier with 1M tokens; Standard $2.50 per 1M tokens plus $5 per additional
workspace; Enterprise with BYO cloud). MCP integration is first-class, included
in the free tier. Its strength is transforming document corpora into knowledge
graphs ("cognify"), which uses LLM processing.

**Pick Cognee if** your problem is turning a large document corpus into a
semantic graph. **Pick Engram if** your problem is continuous operational
memory for agents — writes, decisions, handoffs, session context.

### Basic Memory — Markdown-first personal knowledge

Basic Memory (AGPL-3.0) is local-first with Markdown files as the source of
truth and an MCP server on top; LLMs write only when explicitly directed. It is
the simplest mental model in this list.

**Pick Basic Memory if** you want human-editable Markdown notes that an agent
can also read and write. **Pick Engram if** you need hybrid search, salience
and lifecycle policy, workspaces, identities, quality scoring, and a
multi-interface server — capabilities beyond a note store.

### Anthropic's reference memory MCP server

Explicitly a basic demo: a local knowledge graph persisted to a JSONL file, no
vector search, no LLM involvement. Good for learning MCP memory concepts; not a
production memory system, and not positioned as one.

## Scale honesty

Engram's vector search is an exact cosine scan with caching, not a native ANN
index. Internal measurements at ~100K memories: FTS5 lexical search ~7ms,
cosine scan ~543ms. For collections into the hundreds of thousands, hybrid
fusion and caching keep interactive latency; for millions of vectors with
strict latency targets, use the optional external indexing backend
(Meilisearch, feature-gated) or a dedicated vector database alongside Engram.
We publish these numbers because a memory layer you cannot size honestly is a
memory layer you cannot trust.

## Wire-level differences that matter

| | Engram | Mem0 | Zep/Graphiti | Cognee | Basic Memory |
|---|---|---|---|---|---|
| Deployment | Single binary, local-first | SaaS + OSS lib | SaaS + OSS (graph DB required) | SaaS + OSS | Local OSS |
| LLM fact extraction on write | No (default); reviewable dream pipeline opt-in | Yes (per-write LLM call) | Yes (entity/edge extraction) | Yes (cognify) | No (explicit writes only) |
| MCP | Primary interface | Hosted MCP available | MCP server add-on | First-class | Primary interface |
| Temporal graph | Temporal edges, snapshots, contradiction detection | Entity linking, time-aware retrieval | Bi-temporal with validity windows (strongest) | Graph from corpus | No |
| Storage | SQLite + WAL | Managed / configurable | Neo4j or FalkorDB | Configurable | Markdown files |
| Marginal cost per write | ~0 | One LLM call | LLM extraction | LLM processing | ~0 |

Sources: mem0.ai/pricing, github.com/mem0ai/mem0, arxiv.org/abs/2504.19413,
github.com/getzep/graphiti, cognee.ai/pricing,
github.com/basicmachines-co/basic-memory,
github.com/modelcontextprotocol/servers (memory reference server). Verified
July 2026.
