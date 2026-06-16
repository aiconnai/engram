# Engram Hosted/Cloud Architecture

**Date:** 2026-05-27 *(updated to reflect v32–v39 migration phases)*  
**Goal:** Provide a managed source of truth for team memory without forcing every project to build its own ingestion, indexing, and retrieval stack.

**Status:** This document is an architecture reference for hosted deployments
and managed-service planning. It is not a public availability, enterprise SKU,
or production SLO claim unless those services are verified separately.

## Product Line

- **Engram OSS (MIT):** single-tenant, BYO storage, self-hosted memory for teams and agents.
- **Engram Cloud / hosted deployment:** multi-tenant managed hosting pattern with shared workspaces, quotas, metering, and a dashboard.
- **Engram Enterprise:** hosted deployment plus SSO/SCIM, audit, governance, and SLA requirements when contracted and implemented.

## High-level System

Clients:
- MCP agents (JSON-RPC)
- REST clients (future SDKs)
- Web console (dashboard)

Control Plane (Neon Postgres):
- tenants, members, invites
- API keys, usage events, quotas/plans
- subscriptions (Stripe)

Data Plane (per-tenant):
- SQLite DB per tenant workspace on Fly volume
- Optional object storage (Cloudflare R2): backups + images + exports

Edge:
- Cloudflare (CDN/WAF) -> Gateway

## Request Flow (Auth -> Tenant -> Quota -> Proxy)

1) **Auth**
- JWT (Neon Auth) OR API key
- Validate JWT via JWKS cache (refresh on kid miss)
- Validate API key via prefix lookup + argon2/bcrypt verify

2) **Tenant Resolution**
- Preferred: `{tenant-slug}.engram.cloud` -> slug from host
- Fallback: `api.engram.cloud` requires header `X-Engram-Tenant: {slug}`
- If ambiguous/missing -> 400

3) **Membership & Status**
- If JWT: enforce membership `(tenant_id, user_id)` in `tenant_members`
- If API key: key is tenant-bound, still enforce `tenants.status = active`

4) **Quota & Rate Limit**
- Rate limit: token bucket per tenant + route (Redis or in-memory + sticky routing for MVP)
- Quotas: check plan limits (memories, workspaces, API calls/day, storage bytes, etc.)

5) **Proxy to Tenant Engine**
- Gateway proxies to tenant engine instance
- Engine owns tenant SQLite and implements MCP + REST resources

6) **Usage Metering**
- Record a usage event per request (buffered)
- Flush batches to control plane (idempotent via `request_id`)

## Tenant Isolation Strategy

### MVP: SQLite-per-tenant
- File layout: `/data/tenants/{tenant_id}/engram.db`
- Strong isolation, simple backup/restore, predictable performance

### Runtime Model (explicit)
Gateway needs a deterministic way to route requests.

**Recommended MVP:** shared engine process, tenant DB selected by `tenant_id`  
(You can move to process-per-tenant later.)

- `workspaces.db_path` points to the tenant db file
- Gateway injects `tenant_id` into request context
- Engine opens SQLite connection for that tenant (pool per tenant, capped)

**Future:** process-per-tenant
- Add runtime registry: `machine_id`, `internal_url`, `state`, `heartbeat`

## Storage & Backups

### SQLite
- WAL mode on
- Backups must be consistent:
  - snapshot using SQLite backup API or `VACUUM INTO`
  - upload snapshot to R2 with metadata (`tenant_id`, `schema_version`, `ts`)

### R2
- Backups: `r2://engram-backups/tenants/{tenant_id}/db/{ts}.sqlite`
- Images: `r2://engram-images/tenants/{tenant_id}/images/{memory_id}/{ts}_{idx}_{hash}.{ext}`
- Exports: `r2://engram-exports/tenants/{tenant_id}/{ts}.json`

## Observability (M1 minimum)

Structured log fields:
- request_id, tenant_id, tenant_slug
- user_id (if JWT)
- route, method, status_code
- latency_ms
- error_code (auth/quota/mcp)

Metrics:
- requests_total{route,status}
- latency_ms p50/p95/p99
- auth_failures_total
- rate_limited_total
- quota_exceeded_total
- usage_flush_failures_total

## Security Baseline

- TLS only + HSTS
- Body size limits (2-5MB)
- CORS restricted (dashboard + SDK origins)
- API keys hashed at rest; prefix stored for lookup
- Key rotation + revoke immediately
- Encrypted backups in R2

---

## Storage Backends (OSS)

The open-source core supports multiple pluggable storage backends:

| Backend | Feature flag | Use case |
|---------|-------------|----------|
| **SQLite** (default) | *(always on)* | Primary embedded store; BM25 FTS via SQLite FTS5 |
| **Turso / libSQL** | `turso` | Distributed SQLite for cloud deployments (Phase 6 / ENG-54) |
| **Meilisearch** | `meilisearch` | Full-text search complement/replacement for SQLite FTS (Phase 7 / ENG-58) |
| **Image Storage** | `cloud` | R2 or local storage for multimodal memory assets (`media_url` on memories, v34) |

---

## Embedding Backends (OSS)

All providers live in `src/embedding/` and share a common async queue + cache layer (`queue.rs`, `cache.rs`):

| Provider | Feature flag | Notes |
|----------|-------------|-------|
| TF-IDF | *(default)* | No API key required |
| OpenAI | `openai` | Requires `OPENAI_API_KEY` |
| ONNX / local | `onnx-embed` / `local-embeddings` | Runs sentence-transformers locally; requires `ENGRAM_ONNX_MODEL_DIR` |
| CLIP | `multimodal` | Image + text embeddings for multimodal memories |
| Ollama | `ollama` | Local LLM-backed embeddings via Ollama |
| Cohere | `cohere` | Cohere Embed API |
| Voyage AI | `voyage` | Voyage AI embeddings |

---

## Core Subsystems Added in v32–v39

### Scoping & Access Control (v24/v31)
`src/storage/scoping.rs`, `src/storage/scope_grants.rs`

Hierarchical memory scoping with `global/org/project/agent` path structure. Per-agent grants with `read`/`write` permissions. Memories inherit scope from their workspace; cross-scope access requires an explicit grant.

### Agent Portability (`agent-portability` feature, v32)
Snapshot provenance columns on memories (`snapshot_origin`, `snapshot_loaded_at`). `attestation_log` table provides a tamper-evident audit trail for knowledge ingestion from external agents.

### DuckDB Graph (`duckdb-graph` feature, v33)
Analytical graph queries over memory relationships. `graph_entities` table; `scope_path` on `temporal_edges` for tenant isolation. Enables SPARQL-like traversal at scale.

### Dream Phase (`dream-phase` feature, v35/v36)
Periodic background consolidation during idle time. `dream_runs` + `dream_locks` tables; advisory locking prevents concurrent runs. Merges duplicates, prunes stale memories, archives low-salience content.

### Auto-Consolidation Audit (v37)
`consolidation_runs` table logs every merge/conflict-resolve/summarize pass per workspace with token savings metrics.

### Pending Injections (v38)
`src/storage/pending_injections.rs` — FIFO queue written by `SessionEnd` hook, consumed by `SessionStart` hook. Workspace-keyed; enables memories queued during a session to be injected into the next context window automatically.

### Auto-Linker (`emergent-graph` feature)
`src/storage/auto_linker.rs` — Hebbian-style automatic relationship detection. `coactivation_edges` table (v30) tracks co-accessed memory pairs; strength decays over time and increases on repeated co-access.

### Lifecycle Hooks (`hooks` feature)
Hook points: `SessionStart`, `SessionEnd`, `PostToolUse`, `Stop`. Used by agent runtimes to trigger consolidation, inject pending memories, and record tool-use provenance.
