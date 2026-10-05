# Task O1 report — observability without proprietary content

Status: DONE_WITH_CONCERNS. Branch claude/engram-improvement-plan-edb43d (Lane R).

## Commit situation (read first)
The O1 work is committed in `35a1592` ("fix(infra): update event-listener to 5.4.2 ...").
Another lane committed through the shared index while my commit ran its pre-commit hook; my
`git add` of 51 explicit O1 paths was swept into their commit, and my own `git commit` then failed
with "cannot lock ref HEAD". I did not rewrite history (concurrent lanes). Intended message was
`feat(mcp): redact logs and instrument critical paths` (validated with check-commit-msg.sh; the
scope `observability` is rejected by that validator). Follow-up `9a6ed82` records this in the
progress file. The controller may reword/split 35a1592 if it matters; Cargo.lock in that commit is
the other lane's change, the other 51 files are O1.
Pre-commit hook (fmt + clippy --all-targets --all-features) ran and passed on my attempts; I never
used --no-verify and the hook did not fail on another lane's WIP.

## What was implemented
New `src/observability/`:
- `redact.rs`: `redacted(&EngramError)` (error class only), `path_label`, `opaque`, `io_class`,
  `join_error_class`, timeout/busy classification, opt-in `ENGRAM_LOG_ERROR_DETAIL=1`, redacting
  panic hook (installed in engram-server main).
- `operation.rs`: `OperationEvent` (operation, correlation_id, outcome, duration_ms, retry_after_secs,
  bounded method/tool/status/route/error_class) on target `engram::ops`; `CorrelationId` (caller
  `x-request-id` accepted only if 1-64 chars [A-Za-z0-9._-]).
- `counters.rs`: bounded-label counters: permission_denied by C1 reason (9 reasons), sqlite_busy,
  embedding provider timeout/failure, recovery succeeded/failed; snapshot with `uptime_seconds` and
  `not_instrumented` list.
- `alerts.rs`: 9 local alert rules (owner, runbook heading, rollback), `no_data` instead of `ok`
  when samples are missing; test enforces every rule id + `#### Runbook: <id>` heading exist in docs.
Instrumentation: permission denials (permission.rs, workspace_guard.rs), SQLITE_BUSY (manual
`From<rusqlite::Error> for EngramError`, error.rs), provider failure/timeout (embedding drain, worker,
memory_create), recovery (`observe_recovery` around `replication_recover` in dispatch, not in
handlers/sync.rs to avoid the sync lane's files). Gate mismatch: no Rust-side signal; harness gates are
shell/Python -> explicit non-goal in docs and `not_instrumented`.
HTTP transport (#161/#178, #162/#179 reconciliation): `metrics.rs` (moved + extended), `request_id.rs`;
parse errors outside the handler (JsonRejection kept as the same 400/415/413 response, now counted),
MCP-error / tool-error vs HTTP 200 classification, correlation id on every response, bounded
method/tool labels (`src/mcp/log_labels.rs`), SSE active gauge, latency histogram per outcome
(no samples -> null), transport-timeout outcome and in-flight gauge release on drop (existing leak on
timeout/disconnect fixed), health `observability` block with `ENGRAM_OBSERVABILITY_EXPORT=off` rollback.
Log redaction applied to: HTTP transport, workspace_guard, embedding create/drain/worker, server
background threads, dream, project-context scan (paths), attestation hooks, vision/CLIP fallbacks,
meilisearch, enrichment/audit emitters, active_artifacts.
Docs: `docs/OPERATIONS.md` (logging contract, operation record, counters, #178/#179 table with explicit
non-goals, alert catalog + 9 runbooks, local incident exercise with real sample output, SLO caveat).
`docs/harness/risk-register.yaml` RISK-0002 (unswept log sites).

## Files changed
51 files in 35a1592 (see `git show --stat 35a1592`). Notable: src/observability/*, src/mcp/http_transport/
{metrics,request_id,mcp_handler,router,events,mod}.rs and tests/{redaction,observability,incident_exercise,
support,mod}.rs, src/mcp/{log_labels,redaction_tests,observability_tests,permission,workspace_guard}.rs,
src/embedding/queue/{drain,worker,mod,redaction_tests}.rs, src/error.rs, src/bin/server.rs, tests/
cli_observability_tests.rs, docs/OPERATIONS.md. Unlisted-in-brief files touched (reason): src/error.rs
(central SQLITE_BUSY hook), src/bin/server.rs (drain loop + panic hook + 2 log sites), src/dream/*,
src/mcp/handlers/*, src/storage/{active_artifacts,enrichment_events,meilisearch_indexer,operational_context}.rs,
src/intelligence/project_context.rs (log redaction on those modules' own sites).
Not touched: storage/connection.rs, sync/*, hooks/*, multimodal/*, intelligence/entities.rs, benches,
storage/queries/core/* (other lanes).

## TDD evidence
RED (behavioral, before implementation), argv:
`cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --lib mcp::redaction_tests`
```
log output leaked "provider-body-sentinel-8e61"; matching lines:
WARN engram::mcp::handlers::memory_crud::create: Immediate embedding failed; ... error=Embedding error:
Embedding API error 500: provider-body-sentinel-8e61 input=private-memory-sentinel-4b7d
test result: FAILED. 1 passed; 1 failed
```
`... --lib http_transport::tests::redaction`
```
log output leaked "panic-payload-sentinel-5e6f"; ERROR ...mcp_handler: MCP handler task failed or panicked
error=task 1 panicked with message "handler failed while processing panic-payload-sentinel-5e6f"
test result: FAILED. 3 passed; 1 failed
```
Expected: logs interpolated `%e` of provider/JoinError text. The other redaction tests (bearer token,
malformed body, rate-limit key header, query, recovery path) passed before the change: they are
regression guards (no log existed), not RED. The HTTP observability tests (#179 criteria), counters,
alerts and incident exercise were written after the implementation (no RED captured): honest gap.
GREEN: same argv, 99/99 in `observability redaction http_transport` filter, 6 consecutive runs stable;
full run below.

## Verification
- `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests --no-fail-fast`:
  exit 0, 58 binaries, 2226 passed, 0 failed, 2 ignored (log: <session-tmp>
  First full run had 1 flake in my own test (a parallel test's drain event matched my `find`); fixed to
  match on class, re-ran.
- `cargo clippy --no-default-features --features "$CI_REQUIRED_FEATURES" --all-targets -- -D warnings`: exit 0.
- `cargo clippy --all-targets --all-features -- -D warnings`: exit 0 (also what the hook ran).
  Note: it returned in 0.23s (warm, other lane compiled the same tree), so attestation-gated edits
  (handlers/project_context.rs, snapshot.rs, document_ingest.rs) are covered by that run's cache, not
  by a fresh compile I observed.
- Real-process smoke (engram-server --transport http on 127.0.0.1:18947, RUST_LOG=trace): create + wrong
  key + malformed body; x-request-id echoed, `engram::ops` lines present, 0 occurrences of the content
  or either key in stderr, /health showed outcomes/p95/observability keys.
- Local incident exercise: `... --lib local_incident_exercise -- --nocapture` prints the in-memory alert
  sink (sample pasted in docs/OPERATIONS.md). No external messages sent.
- `harness-risk-register.sh validate` fails on ANY version of risk-register.yaml in this environment
  (Ruby 4.0 Psych disallows Time); verified the HEAD original fails identically. PENDING/not mine.

## Deviations and concerns
1. Commit mix-up described at the top.
2. `mcp_success_total` semantics changed: JSON-RPC errors and `isError` tool results (HTTP 200) are no
   longer counted as success (that is the #179 "MCP error vs HTTP 200" requirement). Wire format
   unchanged except an added `x-request-id` response header. Counters/health JSON additive otherwise.
3. Redaction sweep is not exhaustive (~270 log sites). Known remaining path-printing sites in other
   lanes' files: `src/storage/connection.rs` (2 warnings) and `src/sync/wal_recovery_staging.rs` (2).
   RISK-0002 + docs say so. Hooks/multimodal subsystem internals were not audited.
4. Panic hook is only unit-adjacent: no process-level test that a handler panic payload stays off
   stderr (the tracing path is tested). Test runs with panicking handlers still print the default panic
   message to stderr (test noise).
5. The CLI installs no tracing subscriber, so its stderr is empty by construction; the CLI test is a
   regression guard for stdout correctness + no log leak if a subscriber is added.
6. Counters are since-process-start, not rates; the alert thresholds are local evaluation only. No SLO,
   dashboard or pager integration claimed. `reqwest::Client::new()` has no default timeout in the
   embedding providers, so provider timeouts may be rare in practice (observed, not changed).
7. Operation event timing for pre-auth 401 measures only the auth middleware.

---
# Fix report, review round 1

## Changes
1. Alert rules gated on denominators (alerts.rs `gated_signal`): zero failures with zero attempts is
   `no_data`; a non-zero failure count still fires. Denominators: `mcp_requests_total` (handler panic,
   timeouts), new `observability.storage_operations_total` (sqlite_busy; counted in `Storage`
   with_connection/with_transaction and the pool variant), new `provider_calls_total.embedding` (drain,
   worker, create), new `authorization_checks_total` (permission_denied_burst), succeeded+failed
   recoveries. Incident exercise now asserts EVERY rule is `no_data` on a fresh router (HTTP half is the
   router's own fresh metrics; the process-wide counters are shared by the lib test binary, so a fresh
   process' counters are simulated with `CriticalCounters::new()`), and also with the export removed.
   Unit tests: `zero_failures_without_attempts_is_no_data_for_every_rule`,
   `zero_failures_with_attempts_is_ok_...`. docs/OPERATIONS.md matches (denominators, counters table,
   regenerated sample sink output, exercise description).
2. Recovery redaction test is now real: `file_backed_recovery_succeeds_and_logs_no_filesystem_path`
   (file-backed Storage under a sentinel dir, real `recover_active_database`, asserts success, target
   written, counter, and no path/content in logs). Because a successful recovery prints no path, the
   RED-able coverage is at the sites that did: `redaction_tests` in `sync/wal_recovery_staging.rs`
   (remove_quietly + sync_parent_dir warnings) and `storage/redaction_tests.rs` (permissive parent dir
   warning). RED shown by temporarily restoring `path.display()`: both failed with
   `log output leaked "staging-path-sentinel-3c4d"` / `"db-parent-sentinel-7a8b"`; restored, GREEN.
   The old half (in-memory storage, `contains(path) || error.is_some()`) was removed.
   Side finding, fixed: the old `observe_recovery` treated any `error` key as failure, but a successful
   recovery report contains `"error": null` and `success: true`, so every success would have been
   counted as failed. Classification now comes from the engine `Result`.
3. Minor:
   - Recovery: `replication_recover_classified` (handlers/sync.rs) returns the outcome; input validation
     errors are `rejected` (own counter/label, not an attempt, INFO event with error_class
     invalid_input); only an engine error is `failed`. Test `rejected_recovery_input_is_not_counted_...`.
   - Timeout before handler start: `RequestCounted` request extension; `on_mcp_timeout(latency,
     already_counted)` counts the request itself when no stage did, exports
     `mcp_timeouts_before_handler_total`; invariant `requests_total = completed + inflight + abandoned +
     timeouts_before_handler` asserted in `assert_balanced` (two timeout tests, one with a stalled body).
   - Client disconnect / handler drop now emits an `abandoned` operation event (WARN, with correlation id
     and route), asserted in the timeout test; docs list it.
   - server.rs storage_mode_warning -> `opaque`; connection.rs path warnings -> `path_label`;
     wal_recovery_staging.rs (2) and sync/cloud.rs fsync warning (1) -> `path_label` + `io_class`.
   - CLI test renamed `cli_stdout_has_the_content_and_stderr_has_none_of_it_even_at_trace` with an honest
     module doc (CLI installs no subscriber; regression guard only).
   - `is_sqlite_busy` deduplicated: connection.rs `is_busy` removed, uses `observability::redact`.

## Verification
- `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests --no-fail-fast`: exit 0,
  58 binaries, 2232 passed, 0 failed, 2 ignored (<session-tmp>
- `cargo clippy --no-default-features --features "$CI_REQUIRED_FEATURES" --all-targets -- -D warnings`: exit 0.
- `cargo clippy --all-targets --all-features -- -D warnings`: exit 0.
- 4 consecutive runs of the `observability redaction http_transport` lib filter: 105/105 each.

## Concerns / not done
- Remaining unaudited log sites: rest of src/sync (cloud bucket/key ids, worker errors), hooks, multimodal,
  watcher (RISK-0002 text updated; docs say so).
- The exercise's "fresh process" half simulates fresh critical counters rather than spawning a process,
  because startup itself legitimately records storage operations (a real fresh server would show
  `sqlite_busy` as ok, not no_data, once it has done DB work).
- Commit made with add+commit in one command on explicit paths.
