# Engram Cloud Operations

This document covers the operating model for a managed or private hosted Engram
memory service: availability targets, latency targets, backups, metering, and
recovery for the team knowledge source of truth.

These targets are planning baselines for hosted deployments. Do not publish
them as public SLOs unless the specific deployment, monitoring, and incident
process have been verified.

## Target SLOs (M1 planning baseline)
- Gateway availability: 99.9%
- p95 latency (gateway + storage, excluding model calls): < 250ms
- Error rate: < 0.5%
- Auth failure rate: < 2%

None of these is measured by this repository. The server only exports
process-lifetime counters and a coarse latency histogram (see
"Observability without proprietary content"); an SLO needs a deployment, a
scrape/retention stack and rate windows that only the operator has. Local
availability checks (the incident exercise below, `/health`) say whether one
process is alive and counting, never whether a deployment met its SLO.

## Alerts (minimum)
- 5xx_rate > 1% for 5m
- p95_latency > 500ms for 10m
- auth_failures spike (3x baseline)
- usage_flush_failures > 0 for 5m
- db_open_failures > 0 for 1m
- rate_limited_total anomaly (abuse)

Mapping to what this repository exports (`/health`, see the alert catalog
below): `5xx_rate` alone under-counts handler failures because MCP errors travel
as HTTP 200, so use `mcp_outcomes` (`handler_panic`, `protocol_error`,
`tool_error`). `p95_latency` maps to `mcp_latency.all.p95_ms` (bucket upper
bound). `auth_failures` maps to `mcp_outcomes.unauthorized`. `rate_limited_total`
maps to `mcp_outcomes.rate_limited`. `usage_flush_failures` and
`db_open_failures` belong to the hosted gateway/control plane and have no
counterpart in this repository's code: not instrumented, not "zero".

## Backup & Restore

### Backup (recommended)
1) Create consistent snapshot:
   - SQLite backup API OR `VACUUM INTO '/tmp/{tenant_id}_{ts}.sqlite'`
2) Upload to R2:
   - `tenants/{tenant_id}/db/{ts}_v{schema_version}.sqlite`
3) Store backup metadata row in control plane:
   - tenant_id, ts, schema_version, size_bytes, sha256

### Restore
1) Download snapshot to `/data/tenants/{tenant_id}/restore.sqlite`
2) Stop writes (maintenance lock for tenant)
3) Replace db file atomically:
   - move current -> `.bak`
   - move restore -> active
4) Run integrity check + schema version verify
5) Release lock

### Restore Tests
- weekly: restore a random tenant into a staging sandbox and run smoke tests

## Usage Metering Pipeline

### Why
Writing to Postgres on every request throttles the gateway.

### Design
- Generate `request_id` at gateway edge
- Produce usage event `{request_id, tenant_id, route, status, counters, ts}`
- Buffer in memory
- Flush batches every N seconds or M events
- Control plane insert is idempotent (unique(request_id))

### Counters
- api_calls = 1 per request
- search_queries = 1 per /v1/search or MCP memory_search
- memories_created for create endpoints
- storage_bytes delta (optional in M1)

## Incident Playbook (fast actions)

### Auth outage
- check JWKS refresh
- force JWKS refresh on kid miss
- verify issuer/audience config

### Tenant DB unavailable
- check Fly volume health
- verify path permissions
- restart engine process
- restore from last good backup if corruption

### Embedding queue backlog or stale jobs
Owner: the embedding drain thread of `engram-server` (every
`ENGRAM_EMBEDDING_DRAIN_INTERVAL` seconds, default 30; 0 disables it).
- Symptom: `maintenance status` shows the `embeddings` index as `backlogged`
  (jobs pending/processing, expected while `defer_embedding=true` memories wait),
  or `degraded` (stale/failed jobs, flag without row, row without flag, orphaned
  rows, memories with no embedding and no job, completed job without row).
- Automatic recovery: on start and on every cycle the drain thread moves a
  `processing` job whose lease expired (15 minutes since `started_at`) back to
  `pending` (`retry_count + 1`), or to `failed` once the retry budget (3) is
  exhausted. A crash or restart therefore never strands a job forever, and until
  the lease expires the job stays visible as `processing`.
- `failed` jobs are never retried automatically. After fixing the cause
  (provider outage, credentials), run
  `maintenance queue-hygiene --requeue-failed --apply` (omit `--apply` for a
  dry-run). Retry is idempotent per memory: one embedding row and one job.
  Note `retry_count` is incremented both when a job fails and when it is
  requeued, so the default budget of 3 allows about 2 provider attempts.
- `missing_embedding_unqueued`, `complete_job_without_embedding_row` or
  `flagged_without_embedding_row` > 0: the memory has no embedding row and
  nothing will produce one (legacy data, or an interrupted older write). Repair
  with `maintenance rebuild --embeddings --apply` (dry-run without `--apply`).
  In one transaction it selects every live memory whose `has_embedding` flag is
  0, or is 1 without an `embeddings` row; sets such flags to 0; and creates or
  resets a `pending` job (`retry_count = 0`) for each, replacing a `complete`,
  `failed` or stale `processing` job. The drain then re-embeds them. Memories
  that have both a flag and an embedding row are untouched, and orphaned rows
  (`orphaned_count`, rows of superseded memories) are not repaired by it.
  CLI `create`, snapshot load and dream promotion now enqueue their jobs themselves.

### High latency
- inspect top routes p95
- reduce burst / tighten rate limit
- cap per-tenant SQLite pool
- shed load on expensive routes

## Observability without proprietary content

Task O1. Goal: diagnostics that are useful without leaking memory text,
queries, credentials, filesystem paths or provider error bodies, and signals
that never mistake "no samples" for "no errors".

### Logging contract

Logs may say what kind of failure happened and how long it took. By default
they never contain:

| Never logged | Instead |
|---|---|
| Bearer tokens, API keys, rate-limit key header values | nothing (not even the outcome's credential) |
| Memory content, search/query text, request bodies, JSON parse messages | nothing; the HTTP parse rejection logs only status class and outcome |
| Filesystem paths | `<path>` |
| Provider/HTTP error bodies (they can echo the prompt, and `reqwest` errors embed the request URL) | the error class: `embedding`, `timeout`, `http`, `database_busy`, ... |
| Panic payloads (handler tasks, dream pass, process panic hook) | task class `task_panicked`; location only |
| Caller-supplied JSON-RPC method and tool names | bounded labels: a defined method or catalog tool name, else `other` / `unknown` |

Memory ids and workspace names may still appear in diagnostic logs (they
locate a problem); they are never counter labels. Operators who need the full
text for one debugging session can set `ENGRAM_LOG_ERROR_DETAIL=1`; that is an
explicit opt-in and must not be used for shared or retained logs.

Unchanged by design: MCP/HTTP protocol payloads (error envelopes still carry
their message), CLI stdout, the `error` column of `embedding_queue` jobs and the
error returned to the caller. Only the log stream is redacted. Code:
`src/observability/redact.rs`; tests: `src/mcp/redaction_tests.rs`,
`src/mcp/http_transport/tests/redaction.rs`,
`src/embedding/queue/redaction_tests.rs`, and
`scan_logs_do_not_contain_the_project_path` in `src/intelligence/project_context.rs`.

Scope of the sweep. About 270 log sites exist; this task redacted the ones on
the critical paths (HTTP transport, authorization, embedding create/drain/worker,
the server's background threads, dream, project-context scan, attestation hooks,
vision/CLIP fallbacks, Meilisearch sync, enrichment/audit emitters). It is not an
audit of every site. Also redacted in the review round: the database-path warnings in
`src/storage/connection.rs`, the staging-file warnings in
`src/sync/wal_recovery_staging.rs`, the download-directory warning in
`src/sync/cloud.rs` and the storage-mode warning printed at server start (tests:
`src/storage/redaction_tests.rs`, `redaction_tests` in `wal_recovery_staging.rs`,
`file_backed_recovery_succeeds_and_logs_no_filesystem_path`). Still not audited:
the remaining sites in `src/sync` (cloud bucket/key identifiers, worker errors),
`src/hooks`, `src/multimodal` and the watcher binaries; they are tracked as open
follow-ups (RISK-0002), not as fixed.

### Operation record

Every finished HTTP MCP/SSE request, embedding drain failure and
`replication_recover` call emits one event on target `engram::ops`:

| Field | Meaning |
|---|---|
| `operation` | `http.mcp`, `http.events`, `embedding.drain`, `replication_recover` |
| `correlation_id` | Same value as the `x-request-id` response header. A caller `x-request-id` of 1-64 chars `[A-Za-z0-9._-]` is kept, anything else is replaced by a generated id. Present on every response, including 401, 408, 413 and parse rejections |
| `outcome` | `success`, `unauthorized`, `forbidden`, `rate_limited`, `parse_error`, `body_too_large`, `timeout`, `protocol_error`, `tool_error`, `handler_panic` (HTTP MCP), plus `abandoned` (the handler future was dropped before answering: client disconnect, or the follow-up of a `timeout`; logged at WARN, not a response class); `stream_opened` (SSE). `replication_recover`: `succeeded`, `failed`, `rejected` |
| `duration_ms` | Wall time of the operation |
| `retry_after_secs` | Set when the caller is told to retry (HTTP 429 sends `Retry-After: 1`) |
| `method`, `tool`, `status`, `route`, `notification`, `error_class` | Bounded labels, optional |

WARN level means failure, INFO means success. Code: `src/observability/operation.rs`.

### Counters

`GET /health` carries `transport.http` (per-request signals) and, unless
`ENGRAM_OBSERVABILITY_EXPORT=off`, an `observability` block (critical paths).
All counters restart at zero when the process starts; the block reports
`uptime_seconds` and a `not_instrumented` list so an absent series is never read
as zero. Labels are fixed enums, never ids, tenants, workspaces, tool names,
queries or error text.

| Signal | Series | Labels |
|---|---|---|
| Permission denied | `observability.permission_denied_total` | `reason`: `mode_insufficient`, `workspace_not_allowed`, `global_scope_requested`, `tool_not_workspace_scoped`, `workspace_claim_missing`, `scope_grant_missing`, `foreign_or_missing_memory`, `authorization_check_failed`, `invalid_mode_config` (the C1 reasons in `docs/security/workspace-operation-matrix.md`; foreign and missing stay indistinguishable) |
| SQLITE_BUSY / LOCKED | `observability.sqlite_busy_total` | none. Counted where a rusqlite error converts into `EngramError`; busy retries handled inside `src/storage/connection.rs` are not counted |
| Embedding provider timeout / failure | `observability.provider_timeout_total`, `provider_failure_total` | `embedding`. Timeout = `reqwest` timeout, `io::TimedOut`, or a provider message saying it timed out (heuristic, text is inspected, never logged) |
| Recovery | `observability.recovery_total` | `succeeded`, `failed`, `rejected`. `rejected` is a caller input error (missing/invalid path, unsupported option): the engine never ran, so it is not an attempt. Only `failed` = an error from the recovery engine itself |
| Denominators | `observability.authorization_checks_total`, `storage_operations_total`, `provider_calls_total.embedding` | none. One per authorization check, per `Storage` connection/transaction use, per embedding provider call. The alert rules need them to tell "no failures" from "nothing happened" |
| HTTP outcomes | `transport.http.mcp_outcomes` | outcome class (above) |
| HTTP latency | `transport.http.mcp_latency.all` and `.by_outcome.<class>` | fixed buckets 1, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000 ms and overflow; `p50_ms/p95_ms/p99_ms` are bucket upper bounds (at or above 5000 reports 5000); `samples` = 0 means no data and quantiles are `null` |
| Transport timeouts / abandoned | `mcp_timeouts_total`, `mcp_timeouts_before_handler_total`, `mcp_abandoned_total` | none. Abandoned = handler dropped before answering (client gone or timeout); it releases `mcp_inflight_total` and logs an `abandoned` event. `mcp_timeouts_before_handler_total` = timeouts that fired while the body was still being read; invariant: `mcp_requests_total = mcp_requests_completed + mcp_inflight_total + mcp_abandoned_total + mcp_timeouts_before_handler_total` (asserted by tests) |
| SSE | `events_active` (gauge), `events_streams_total` | none |

Rollback of the extra export: set `ENGRAM_OBSERVABILITY_EXPORT=off` (or
`0/false/no/disabled`). The `observability` block disappears from `/health`;
counters keep counting, and logs, `x-request-id`, `transport.http` and the
structured error envelopes are untouched, so no error message is lost.

### Reconciliation with issues #161/#178 and #162/#179

Both issues were closed before O1. This table is the residual check against
the code, not a claim that the issues' prose was implemented verbatim.

| Criterion | Status |
|---|---|
| #178 rate limiting rejects bursts on `/mcp` and `/v1/mcp` with 429 and `Retry-After: 1`, auth failures do not consume the bucket | Existing, with regressions in `src/mcp/http_transport/tests/rate_limit_basic.rs` and `rate_limit_keying.rs`; O1 adds `rate_limited_requests_log_retry_and_keep_the_429_contract` (429, body `-32005`, `Retry-After: 1`, WARN `retry_after_secs=1`) |
| #178 loopback bypass | Not implemented and not added: limits apply on loopback unless rate limiting is set to 0 (disabled). Explicit non-goal |
| #178 adaptive `Retry-After` | Non-goal: it is the constant `1` |
| #179 request duration and status visible | Added: operation event (`duration_ms`, `status`, `outcome`) and the latency histogram |
| #179 auth failure rate and parse error rate queryable | Added: `mcp_outcomes.unauthorized`, `mcp_outcomes.parse_error` (parse rejections happen in the body extractor, outside the handler; before O1 they were uncounted) |
| #179 MCP error vs HTTP 200 | Added: `protocol_error` (JSON-RPC `error` object) and `tool_error` (`isError: true`) are failures, no longer counted as `mcp_success_total` |
| #179 correlation id / method | Added: `x-request-id` and bounded `method` / `tool` |
| #179 active SSE | Added: `events_active` |
| #179 p95/error-rate dashboards can separate auth, transport and handler failures | Added: `mcp_latency.by_outcome` and `mcp_outcomes` |
| #179 no production-only dependency | Held: no dependency added |
| Parse errors answered as JSON-RPC `-32700` | Non-goal: axum's 400/415/413 text response is unchanged (wire format) |
| Prometheus exposition, OpenTelemetry/trace spans, JSON log format | Non-goals: the JSON in `/health` is the export; events, not spans |
| 404/405 on unknown routes, per-tool metrics | Non-goals (unbounded or low value); listed in `not_instrumented` where relevant |
| Dashboards | None presumed or shipped |

### Alert catalog

Local alert rules live in `src/observability/alerts.rs` and evaluate the
`/health` JSON; they send nothing. A rule without enough samples answers
`no_data`, never `ok`: every rule is gated on a denominator, so zero failures
out of zero attempts is `no_data` (requests for the HTTP rules, authorization
checks for `permission_denied_burst`, storage operations for `sqlite_busy`,
provider calls for `embedding_provider_timeout`, succeeded + failed recoveries
for `replication_recover_failed`). A non-zero failure count fires regardless
of the denominator. Thresholds: auth rejections over 2% and rate-limited
requests over 5% of at least 50 finished requests; p95 bucket over 500 ms on at
least 20 samples; any handler panic, transport timeout, SQLITE_BUSY conversion,
embedding provider timeout or failed recovery since start; more than 25
permission denials since start. Counts are since process start: turn them into
rates with the monitoring stack that scrapes `/health`.

| Alert id | Owner | Runbook | Rollback |
|---|---|---|---|
| `mcp_handler_panic` | http-transport (`src/mcp/http_transport`) | Runbook: mcp_handler_panic | previous binary |
| `mcp_timeouts` | http-transport | Runbook: mcp_timeouts | previous `ENGRAM_HTTP_REQUEST_TIMEOUT_MS` |
| `mcp_auth_failure_rate` | security (`src/auth`) | Runbook: mcp_auth_failure_rate | revert key/proxy change |
| `mcp_rate_limited_rate` | http-transport (`rate_limit.rs`) | Runbook: mcp_rate_limited_rate | previous rate-limit flags |
| `mcp_latency_p95` | http-transport | Runbook: mcp_latency_p95 | roll back the release |
| `sqlite_busy` | storage (`src/storage`) | Runbook: sqlite_busy | stop the second writer |
| `embedding_provider_timeout` | embeddings (`src/embedding`) | Runbook: embedding_provider_timeout | previous provider or `tfidf` |
| `replication_recover_failed` | recovery (`src/sync/wal_recovery*.rs`) | Runbook: replication_recover_failed | discard staging target |
| `permission_denied_burst` | security (`src/mcp/permission.rs`) | Runbook: permission_denied_burst | revert permission config |

Gate mismatch (listed in the O1 brief) has no Rust-side signal: harness gates
are shell/Python and report through `.sensors-log` and `bin/harness-stats.sh`.
It is in `not_instrumented`, an explicit non-goal here.

#### Runbook: mcp_handler_panic
Signal: `mcp_outcomes.handler_panic` greater than 0. Find the request by its
`x-request-id` in the `engram::ops` WARN event (`outcome="handler_panic"`) and
the `MCP handler task failed or panicked` ERROR line (class only; the payload is
redacted, set `ENGRAM_LOG_ERROR_DETAIL=1` locally to see it). Mitigation: roll
back to the previous binary. The client already received JSON-RPC `-32603`.

#### Runbook: mcp_timeouts
Signal: `mcp_timeouts_total` greater than 0 (HTTP 408, JSON-RPC `-32008`).
Check `mcp_latency.by_outcome.timeout`, `mcp_inflight_total` (must return to 0)
and `mcp_abandoned_total`. Mitigation: shed or rate-limit the slow caller; raise
`ENGRAM_HTTP_REQUEST_TIMEOUT_MS` only with a reason. The blocking handler thread
keeps running after the timeout; it is not cancelled.

#### Runbook: mcp_auth_failure_rate
Signal: unauthorized share above 2% of at least 50 requests. Check whether a
client rotated its key or a proxy strips `Authorization`; compare with
`mcp_outcomes.forbidden`. A sustained spike from one source is abuse: keep the
key, tighten the rate limit.

#### Runbook: mcp_rate_limited_rate
Signal: `mcp_outcomes.rate_limited` above 5% of requests. Layer: HTTP per-key/IP
bucket (`--http-rate-limit-rps`, `--http-rate-limit-burst`, `--http-rate-limit-key`).
The 429 carries `Retry-After: 1`. Do not raise the limit blindly; size it from
the caller's expected sustained rate.

#### Runbook: mcp_latency_p95
Signal: `mcp_latency.all.p95_ms` above 500 on at least 20 samples. Use
`mcp_latency.by_outcome` to see whether successes or failures are slow, and the
`engram::ops` `duration_ms` plus `tool` to find the slow tool. Mitigation: roll
back the release that moved it; tighten the rate limit.

#### Runbook: sqlite_busy
Signal: `observability.sqlite_busy_total` greater than 0. Cause: a second writer
(another process) held the write lock past `busy_timeout` (30 s). Check that only
one `engram-server` owns the database (singleton storage lock) and that no
maintenance command runs against it. Mitigation: stop the second writer.

#### Runbook: embedding_provider_timeout
Signal: `observability.provider_timeout_total.embedding` greater than 0. Memories
stay durable with `pending` jobs; see "Embedding queue backlog or stale jobs".
Mitigation: switch `ENGRAM_EMBEDDING_MODEL` to the last working provider (or
`tfidf`); after the provider is back, `maintenance queue-hygiene --requeue-failed --apply`.

#### Runbook: replication_recover_failed
Signal: `observability.recovery_total.failed` greater than 0 (input errors are
counted separately as `rejected` and never fire this alert). The caller received
the error text in the tool result. The active database is never opened by raw
file handles (G1): recovery writes a staging target. Mitigation: leave the active
database alone, remove the staging target, retry from a closed copy.

#### Runbook: permission_denied_burst
Signal: more than 25 denials since start. Read `permission_denied_total` by
`reason`: `mode_insufficient` after a mode change means clients need the new mode
or the change is wrong; `tool_not_workspace_scoped` / `workspace_claim_missing`
mean a restricted principal calls tools it cannot scope; `foreign_or_missing_memory`
means probing or stale ids. Denials are fail-closed, so nothing needs data rollback.

### Local incident exercise

`cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --lib local_incident_exercise -- --nocapture`
(`src/mcp/http_transport/tests/incident_exercise.rs`) runs a local router, checks
that on a fresh router EVERY rule is `no_data` (also with the observability
export off; the process-wide counters are shared by the whole test binary, so
a fresh process' counters are simulated), injects real failures
(handler panics, a transport timeout, 60 wrong keys, a SQLITE_BUSY conversion, a
failed recovery), reads `/health`, evaluates the rules and prints what an
on-call would receive into an in-memory sink. Sample output (owner, runbook and
rollback are part of every line; nothing is sent to any external system):

```
ALERT mcp_handler_panic | owner: http-transport (src/mcp/http_transport) | runbook: docs/OPERATIONS.md#runbook-mcp_handler_panic | rollback: Redeploy the previous binary; the failing request is identified by its x-request-id. | why: handler panics = 3 since process start
ALERT mcp_timeouts | owner: http-transport (src/mcp/http_transport) | runbook: docs/OPERATIONS.md#runbook-mcp_timeouts | rollback: Restore the previous ENGRAM_HTTP_REQUEST_TIMEOUT_MS and shed the slow tool's callers. | why: transport timeouts = 1 since process start
ALERT mcp_auth_failure_rate | owner: security (src/auth, src/mcp/http_transport) | runbook: docs/OPERATIONS.md#runbook-mcp_auth_failure_rate | rollback: Revert the API key / proxy change that started the rejections; keep the key as is if it is an attack. | why: unauthorized rate 0.682 over 0.02 (60/88)
ALERT sqlite_busy | owner: storage (src/storage) | runbook: docs/OPERATIONS.md#runbook-sqlite_busy | rollback: Stop the second writer; confirm the singleton storage lock is held by one server. | why: SQLITE_BUSY conversions = 1 since process start
ALERT replication_recover_failed | owner: recovery (src/sync/wal_recovery*.rs) | runbook: docs/OPERATIONS.md#runbook-replication_recover_failed | rollback: Keep the active database untouched; discard the staging target and retry from a closed copy. | why: failed recoveries = 1 since process start
```

This demonstrates local availability signalling only. It does not establish an
SLO, a paging integration or any production dashboard.

## Environments

### Dev
- local gateway + local sqlite
- Neon branch for control plane dev
- relaxed CORS for localhost

### Staging
- separate Neon branch
- real rate limits (soft)
- usage metering can be dry-run

### Prod
- Neon main branch with backups
- strict CORS, TLS/HSTS
- quotas enforced
- alerts + dashboards live
