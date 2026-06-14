# Positioning and Messaging

This note extracts reusable messaging ideas from the old marketing strategy
draft while removing stale counts, unverifiable rankings, and absolute
competitive claims.

## Positioning

Engram is a local-first memory layer for AI agents and teams working with
proprietary context. It organizes durable memories, decisions, documents,
sessions, and relationships so agents can retrieve context from an inspectable
source of truth instead of reconstructing it from chat history.

Use Engram when:

- agents repeatedly need prior project decisions, constraints, or conventions;
- teams need shared memory across tools and sessions;
- local ownership, provenance, and inspectability matter;
- hybrid retrieval and knowledge graph traversal are more useful than a plain
  vector store.

Avoid claiming that Engram is the only, largest, fastest, or most complete
memory system unless that claim is backed by a fresh public comparison and raw
evidence.

## Audience Notes

| Audience | Need | Message angle |
|---|---|---|
| Agent developers | Add memory without rebuilding storage, search, and retrieval | MCP and SDK access to a local-first memory substrate. |
| Engineering teams | Reduce repeated context transfer and decision drift | Shared, searchable project memory with provenance. |
| Platform teams | Control deployment, data locality, and operational behavior | Single Rust service, SQLite-first storage, optional integrations. |
| Enterprise evaluators | Inspectability, auditability, and governance | Explicit memory records, retention, access, and attestation surfaces. |

## Reusable Copy

Short description:

> Engram is a local-first memory layer for AI agents and teams. It stores
> proprietary context in an inspectable source of truth and exposes it through
> MCP, local tooling, and SDKs.

Developer-oriented description:

> Give MCP-compatible agents durable project memory without forcing every
> workflow to rebuild storage, hybrid search, knowledge graph links, and
> session continuity from scratch.

Team-oriented description:

> Turn recurring meetings, documents, transcripts, and decisions into shared
> memory that agents and humans can search with provenance.

## Comparison Framing

Public comparisons should use evidence-based categories, not broad attacks:

- Installation and first-use path.
- Local-first versus hosted-only operation.
- MCP, SDK, and framework integration surfaces.
- Search modes and retrieval explainability.
- Provenance, audit, retention, and review boundaries.
- Human inspection and dashboard capabilities.
- Team sharing, sync, and access control.

When naming competitors, prefer phrasing like:

- "Compared with cloud-first memory APIs, Engram emphasizes local-first
  operation and inspectable storage."
- "Compared with research prototypes, Engram emphasizes production packaging,
  operational gates, and explicit provenance."
- "Compared with visual memory products, Engram currently needs stronger
  human-facing inspection surfaces."

## Public-Use Fact Checks

Before using these notes in README, launch posts, docs homepages, or sales copy,
verify:

- The exact current Engram version and generated MCP tool count.
- Which transports, SDKs, adapters, and integrations are shipped versus planned.
- Current behavior for local, HTTP, cloud, sync, auth, and deployment paths.
- Competitor install requirements, local/offline support, hosted requirements,
  licenses, and public API surfaces.
- Any benchmark numbers, latency claims, binary size claims, or throughput
  claims against reproducible commands and raw artifacts.
- Any community metrics such as stars, downloads, Discord size, contributor
  counts, or customer/adoption numbers.
