# Competitive Landscape

This note summarizes useful competitive research without preserving stale
claims from the old branch. Treat every named-project observation as a prompt
for fresh verification before public use.

## Evaluation Axes

Engram should be evaluated against peer memory systems on these axes:

| Axis | Why it matters |
|---|---|
| Install and first-use friction | Developers choose the memory layer they can try quickly. |
| Agent integration surface | MCP, SDKs, and framework adapters determine how easily agents can use memory. |
| Local-first operation | Proprietary context often needs offline, private, and inspectable storage. |
| Retrieval quality | Hybrid exact, semantic, fuzzy, graph, and temporal retrieval affect usefulness. |
| Memory governance | Retention, provenance, versioning, audit, and review boundaries reduce silent drift. |
| Human inspection | Dashboards, graph views, and readable exports help users trust stored memory. |
| Proactive intelligence | Extraction, consolidation, contradiction handling, and context push make memory active. |
| Team and enterprise readiness | RBAC, sync, audit trails, deployment, and compliance shape adoption beyond single users. |

## Peer Categories

| Category | Representative projects from source research | Takeaway for Engram |
|---|---|---|
| Developer-first memory APIs | Mem0, Memobase, Letta, Zep | Reduce setup friction and publish clear examples before adding more surface area. |
| Visual or consumer memory tools | Supermemory, Memora | Add human-facing inspection and capture flows where they help trust and debugging. |
| Research memory systems | A-Mem, GAM, ReMe, HiMem, CoALA-style systems | Translate useful mechanisms into deterministic, reviewable production workflows. |
| Multi-agent and federation platforms | Nexus Agents and adjacent orchestration systems | Keep Engram focused as the memory substrate, with explicit handoff and sharing contracts. |

## Opportunity Gaps

The old feature matrix surfaced several areas that remain useful to track:

- **Query expansion and rewriting**: Improve recall for underspecified user
  queries by generating or normalizing search variants before retrieval.
- **Structured user or entity profiles**: Build on identity resolution to
  summarize preferences, interests, roles, and recurring behavior with
  provenance.
- **Human-facing dashboard and graph inspection**: Provide a small visual
  surface for search, memory inspection, graph traversal, and operational
  debugging.
- **Framework adapters**: Make Engram easy to adopt from common agent stacks
  without making those adapters the canonical memory contract.
- **Proactive extraction and verification**: Improve write-path quality before
  optimizing retrieval over noisy memory.
- **Knowledge-base import**: Support migration from Markdown, Obsidian, Notion
  exports, browser bookmarks, and other common knowledge stores.
- **Tool evolution governance**: Version and deprecate public tool contracts
  deliberately as the MCP surface grows.

## Claims to Re-Verify Before Public Use

- Current feature support and public APIs for each named competitor.
- Current install paths, hosted/cloud requirements, and local/offline support.
- Current GitHub stars, package downloads, licenses, and development activity.
- Current benchmark results for LoCoMo, LongMemEval, HaluMem, or any other
  memory benchmark.
- Current Engram tool count from the generated MCP reference, not hand-written
  docs.
