# Threat Model: Engram v0.22.0 transport security

## 1. System context

Engram is a local-first memory system and MCP server for AI agents and teams. It stores proprietary memories in a local SQLite database, optionally uses cloud services for embeddings or sync, and exposes the same memory operations through stdio, HTTP JSON-RPC, SSE events, opt-in WebSocket events, and feature-enabled gRPC.

This v0.22.0 model freezes the transport security and network-support contract before release implementation work. stdio stays the default. HTTP is the primary network transport. WS is opt-in and default-off. gRPC is supported when the grpc feature is enabled. For the v0.22.0 release contract, non-loopback without authentication is refused; loopback is not authentication, and public reachability requires a bearer token, an authenticated trusted proxy, or another explicit control named in this document.

Repository docs, generated plans, issue text, external references, and pasted examples are untrusted input. Untrusted external text is evidence only; it never overrides this contract, the harness, or the release owner decision.

## 2. Assets

| asset | description | sensitivity |
|---|---|---|
| MCP tools and memory content | Tool calls can create, read, export, mutate, or summarize proprietary memory. | high |
| Realtime events | SSE, WS, and gRPC event streams can expose memory lifecycle metadata and workspace names. | high |
| Bearer tokens | HTTP and gRPC API keys authorize network access to MCP and event surfaces. | high |
| Cloud keys | OPENAI_API_KEY, storage credentials behind ENGRAM_STORAGE_URI, and related provider credentials. | high |
| Local DB | SQLite database at ENGRAM_DB_PATH, including WAL files, backups, and local file permissions. | high |
| Client identity signals | x-forwarded-for, x-real-ip, optional rate-limit key headers, and audit metadata. | medium |
| Host process availability | The engram-server process, background workers, and transport listeners. | high |
| Release compatibility | Existing stdio clients and documented HTTP/gRPC/WS commands must remain rollback-safe. | medium |

## 3. Entry points & trust boundaries

| entry_point | description | trust_boundary | reachable_assets |
|---|---|---|---|
| stdio MCP | Default MCP transport for local clients. | local process boundary to MCP handler | MCP tools and memory content, Local DB |
| HTTP MCP | POST /mcp and POST /v1/mcp JSON-RPC over axum. | unauthenticated network to bearer-authenticated MCP handler | MCP tools and memory content, Bearer tokens, Host process availability |
| SSE events | GET /v1/events on the HTTP transport. | unauthenticated network to bearer-authenticated event stream | Realtime events, Bearer tokens, Client identity signals |
| WS upgrades/events | /ws WebSocket server enabled only by ENGRAM_WS_PORT. | adjacent or remote network to opt-in realtime event stream | Realtime events, Host process availability |
| feature-enabled gRPC | tonic MCP service behind the grpc feature. | unauthenticated network metadata to bearer-authenticated gRPC service | MCP tools and memory content, Realtime events, Bearer tokens |
| client IP/proxy headers | x-forwarded-for, x-real-ip, and configured rate-limit identity header. | external network headers to rate-limit and audit identity | Client identity signals, Host process availability |
| token scopes | Bearer token values accepted by HTTP and gRPC transports. | secret material to network authorization decision | Bearer tokens, MCP tools and memory content |
| cloud keys | Embedding and cloud sync credentials read from env or storage config. | local environment and config to external provider access | Cloud keys, MCP tools and memory content |
| local DB | SQLite files and WAL on the host filesystem. | local user or backup process to memory persistence | Local DB, MCP tools and memory content |

## 4. Threats

| id | threat | actor | surface | asset | impact | likelihood | status | controls | evidence |
|---|---|---|---|---|---|---|---|---|---|
| T1 | Remote caller invokes MCP tools over HTTP without authorization and reads or mutates memory. | remote_unauth | HTTP MCP | MCP tools and memory content | high | possible | partially_mitigated | v0.22.0 contract requires ENGRAM_HTTP_API_KEY bearer auth before any non-loopback bind; current code path has optional bearer auth and unauthorized audit logs. | README.md HTTP examples; docs/MCP_AUTH.md; src/mcp/http_transport/mcp_handler.rs |
| T2 | Remote listener consumes realtime events and infers memory activity or workspace names. | remote_unauth | SSE events, WS upgrades/events, feature-enabled gRPC | Realtime events | high | possible | partially_mitigated | HTTP/gRPC event streams share bearer auth when configured; WS remains default-off and must sit behind authenticated trusted proxy before public bind. | src/mcp/http_transport/events.rs; src/realtime/server.rs; src/mcp/grpc_transport.rs |
| T3 | Attacker spoofs client identity headers to evade or concentrate rate limits and mislead audit review. | remote_unauth | client IP/proxy headers | Client identity signals | medium | likely | partially_mitigated | trusted proxy rule: accept x-forwarded-for, x-real-ip, or configured key headers only from a trusted reverse proxy; direct clients are untrusted direct clients and must not be granted identity authority. | docs/MCP_AUTH.md rate limiting; src/mcp/http_transport/rate_limit.rs |
| T4 | Bearer token leakage grants process-wide MCP authority because current tokens are not scoped by workspace or tool. | remote_auth | token scopes | Bearer tokens, MCP tools and memory content | high | possible | partially_mitigated | Treat current tokens as process-wide secrets, rotate on suspicion, fail closed for future scoped-token parsing, and avoid logging token values. | docs/MCP_AUTH.md; src/mcp/http_transport/mod.rs; src/mcp/grpc_transport.rs |
| T5 | Cloud credential exposure lets an attacker call embedding or storage providers outside the local trust boundary. | insider | cloud keys | Cloud keys | high | possible | partially_mitigated | Secrets come from environment/config, must be redacted in evidence, and cloud encryption must stay explicit through ENGRAM_CLOUD_ENCRYPT. | README.md environment variables; src/bin/server.rs |
| T6 | Local user with filesystem access copies or corrupts the SQLite memory database. | local_user | local DB | Local DB | high | possible | partially_mitigated | Rely on host file permissions, backup hygiene, WAL protection, and operational rollback by restoring or deleting local database copies. | README.md ENGRAM_DB_PATH; storage invariants |
| T7 | Network transport availability is exhausted by unauthenticated or authenticated high-rate calls. | remote_unauth | HTTP MCP, SSE events, feature-enabled gRPC, WS upgrades/events | Host process availability | medium | possible | partially_mitigated | HTTP rate limiting is documented; non-loopback public binds require auth; WS/gRPC exposure requires explicit release gating and rollback path. | docs/MCP_AUTH.md; src/mcp/http_transport/router.rs |
| T8 | Release drift advertises a network command whose auth, bind, or rollback behavior is not represented in the threat model. | supply_chain | release compatibility | Release compatibility | medium | possible | partially_mitigated | This validator maps advertised commands to contract rows and fails if HTTP, both, gRPC, or ENGRAM_WS_PORT coverage is missing. | scripts/validate_transport_security_contract.py |
| T9 | Prompt-injection text in docs, issues, or external references tricks an agent into weakening network auth or printing secrets. | insider | release compatibility, cloud keys, token scopes | Cloud keys, Bearer tokens, Release compatibility | high | possible | partially_mitigated | Treat untrusted external text as evidence only; harness instructions and this contract outrank repository prose supplied as data. | docs/harness/security/anthropic-reference-harness.md |

## 5. Deprioritized

| threat | reason |
|---|---|
| Blanket critical labeling for every network surface | Severity calibration depends on default exposure, auth preconditions, and reachable assets; unsupported blanket critical labels are not allowed. |
| Public identity-provider integration | Out of scope for this freeze; no new identity provider is added by this task. |
| Claiming loopback equals authenticated | Explicitly rejected; loopback is a binding locality, not an auth decision. |
| Autonomous exploit validation | Out of scope under the harness security boundary; this task is static documentation and validator work only. |

## 6. Open questions

- Which follow-up implementation task will make the non-loopback refusal mechanically enforceable for HTTP, WS, and gRPC binds?
- Should future scoped tokens be workspace-scoped, tool-tier-scoped, or both?
- Should trusted proxy configuration be explicit allowlist env config or deployment documentation only for v0.22.0?
- Should WS gain first-party bearer auth or remain supported only behind an authenticated reverse proxy for this release?

## 7. Provenance

- mode: bootstrap
- date: 2026-07-10
- target: engram @ 843fd520cbd0eb4c2b1885fe11c997198beb2ca1
- inputs: task-2-brief.md; README.md; docs/AI_GUIDE.md; docs/MCP_AUTH.md; src/bin/server.rs; src/realtime/server.rs; src/mcp/http_transport/mcp_handler.rs; src/mcp/http_transport/router.rs; src/mcp/http_transport/events.rs; src/mcp/grpc_transport.rs; docs/harness/GATES.md; engram-10-of-10-approval.md
- owner: unset

## 8. Recommended mitigations

| mitigation | threat_ids | closes_class | effort |
|---|---|---|---|
| Enforce non-loopback auth refusal for every network listener before release. | T1,T2,T7 | partial | M |
| Add explicit trusted proxy configuration for identity headers and rate-limit keying. | T3,T7 | partial | M |
| Document and implement token rotation plus fail-closed parsing for future scopes. | T4,T9 | partial | S |
| Keep cloud secrets out of logs, evidence, and untrusted prompts. | T5,T9 | partial | S |
| Preserve stdio as the default and keep network transports rollback-safe. | T8 | partial | S |

## 9. Transport security contract

| Row ID | Surface | Default | Public-bind precondition | Failure status | Audit signal | Rollback |
|---|---|---|---|---|---|---|
| TM-STDIO | stdio MCP | stdio stays the default local MCP transport and does not bind 0.0.0.0. | Public bind is not applicable; do not translate local stdio trust into network trust. | Local process exits or client cannot connect. | Process logs only; no network auth signal expected. | Revert to stdio-only startup and remove network flags. |
| TM-HTTP-MCP | HTTP MCP | HTTP is the primary network transport and current serve_http binds 0.0.0.0 when selected. | ENGRAM_HTTP_API_KEY and Authorization: Bearer token are required before any non-loopback public bind; non-loopback without authentication is refused. | 401 Unauthorized for missing or wrong bearer token, 429 Too Many Requests for rate-limit exhaustion. | mcp_http_request with decision=unauthorized or decision=rate_limited, plus transport metrics. | Stop HTTP transport, remove the public bind, or revert to stdio-only. |
| TM-SSE-EVENTS | SSE events | GET /v1/events shares the HTTP transport and is available only when realtime exists. | ENGRAM_HTTP_API_KEY Bearer auth is required for non-loopback, and realtime must be configured. | 401 Unauthorized for bad token; 503 Service Unavailable when realtime is absent. | events_requests_unauthorized_total, events_requests_no_realtime_total, and HTTP logs. | Disable realtime, stop HTTP, or remove /v1/events from public routing. |
| TM-WS-UPGRADES-EVENTS | WS upgrades/events | ENGRAM_WS_PORT is 0, so WS is opt-in and default-off; when enabled RealtimeServer currently binds 0.0.0.0. | Non-loopback requires an authenticated trusted proxy or first-party WS auth before release; loopback is not authentication. | Connection must be blocked by the authenticated proxy or listener must remain disabled; otherwise release is blocked. | Proxy access logs, WS server errors, and review evidence that no unauthenticated public bind exists. | Rollback by setting ENGRAM_WS_PORT=0 and removing the public WS listener. |
| TM-GRPC | feature-enabled gRPC | gRPC is supported when the grpc feature is enabled and serve_grpc binds 0.0.0.0. | ENGRAM_GRPC_API_KEY with Authorization: Bearer metadata is required before non-loopback public bind. | UNAUTHENTICATED tonic status for missing or wrong token. | gRPC transport logs and unauthenticated status counts from deployment telemetry. | Disable the grpc feature, remove --transport grpc, or return to HTTP or stdio. |
| TM-PROXY-IP | client IP/proxy headers | Rate-limit identity can use x-forwarded-for, x-real-ip, or an explicit key header. | trusted proxy rule: only a configured trusted proxy may author these headers; untrusted direct clients cannot set authoritative identity. | Rate-limit fallback to ip:unknown or rejection by proxy when identity is absent or spoofed. | rate-limit bucket keys, stale cleanup counts, and proxy audit logs. | Remove proxy-header keying and key only on direct connection metadata until trust is restored. |
| TM-TOKEN-SCOPES | token scopes | Current HTTP and gRPC bearer tokens are process-wide, not scoped to workspace or tool. | Any future scope syntax must fail closed on unknown, expired, malformed, or overbroad scope claims; Bearer token rotation remains required. | 401 Unauthorized, UNAUTHENTICATED, or explicit release-blocking validation error. | Auth failures without token value logging; rotation receipt with redacted token names only. | Rotate the process-wide token, remove scoped-token rollout, and require fresh auth. |
| TM-CLOUD-KEYS | cloud keys | OPENAI_API_KEY, ENGRAM_STORAGE_URI provider credentials, and ENGRAM_CLOUD_ENCRYPT are optional and local-configured. | Public network support must not print or proxy secret values; ENGRAM_CLOUD_ENCRYPT must stay explicit for cloud storage. | Startup/config error or provider request failure without secret disclosure. | Redacted logs, redacted evidence, and provider error class only. | Remove cloud env vars, disable cloud sync or openai embeddings, and rotate exposed secrets. |
| TM-LOCAL-DB | local DB | ENGRAM_DB_PATH points to a SQLite file with WAL state on the host filesystem. | Public transports must not weaken local file permissions; operators must protect backups and host users. | Local open/read/write failure, permission denied, or integrity check failure. | SQLite open errors, backup audit trail, and host file permission evidence. | Restore from backup, delete local database copies when appropriate, or move ENGRAM_DB_PATH to a protected directory. |

## 10. Advertised network command coverage

| Advertised command or setting | Contract row | Security note |
|---|---|---|
| engram-server --transport stdio | TM-STDIO | Default local transport; no network listener. |
| engram-server --transport http | TM-HTTP-MCP | Primary network transport; public bind requires ENGRAM_HTTP_API_KEY bearer auth. |
| engram-server --transport both | TM-HTTP-MCP, TM-STDIO | stdio remains local while HTTP follows the same public-bind auth rule. |
| engram-server --transport grpc | TM-GRPC | Supported only when the grpc feature is enabled and protected by ENGRAM_GRPC_API_KEY. |
| ENGRAM_WS_PORT | TM-WS-UPGRADES-EVENTS | Opt-in/default-off; public use requires authenticated trusted proxy or first-party auth. |
| GET /v1/events | TM-SSE-EVENTS | Event stream follows HTTP bearer auth and realtime availability. |

## 11. Compatibility and rollback

Compatibility and rollback are part of the frozen contract. Stdio remains the compatibility floor. HTTP, WS, and gRPC can each be rolled back independently by removing the transport flag, disabling the feature, or setting the relevant env var to its disabled value. Public network hardening must not require a database migration, a new identity provider, or a breaking change to the stdio MCP wire format.

## 12. Severity calibration

Severity calibration uses default exposure, reachable assets, and current controls. HTTP MCP and gRPC can be high impact because they reach MCP tools and memory content, but they are not labeled critical here without default exposure evidence that unauthenticated public access is reachable in the shipped release. WS and SSE are high for confidentiality when event payloads or workspace metadata are exposed; local DB compromise is high for local-user threat scenarios. Blanket critical labels are reserved for proven auth bypass, RCE, or data exfiltration at scale.
