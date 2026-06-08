# Review Canvas — ENGRA-84 MCP HTTP Rate-Limit Hardening

## Intent

Close the remaining ENGRA-84 hardening gap after ENGRA-58/59/60 landed:
preserve the documented auth contract while keeping token-bucket protection
bounded and observable.

## Approaches Considered

| Approach | Decision | Rationale |
|---|---|---|
| Leave rate limit before auth | Rejected | Missing or invalid Bearer could return `429` after bucket exhaustion, contradicting `docs/MCP_AUTH.md` and hiding auth failures from operators. |
| Move auth before rate limit in `handle_mcp` | Chosen | Minimal behavior change; unauthorized requests do not spend tokens and keep returning `401`. |
| Add only router-level tests | Rejected | Stale cleanup and bucket eviction require `RATE_LIMIT_MAX_BUCKETS` pressure, which is too large and slow to exercise through Axum requests. |
| Extract pure bucket mutation helper | Chosen | Keeps production behavior unchanged except ordering; makes stale cleanup and eviction deterministic and cheap to test. |

## Hot-Path Complexity

The request path still does one auth check, one optional mutex lock, and one
hash-map bucket update per authorized MCP HTTP request. Unauthorized requests
now skip the mutex entirely when an API key is configured.

The extracted helper is synchronous and does not add allocation beyond the
existing bucket-key string and optional oldest-key clone during eviction.

## Edge Cases

- Missing Bearer after an authorized request exhausted the bucket must still
  return `401 Unauthorized`, not `429 Too Many Requests`.
- Missing Bearer must not consume the single-token burst, so the next authorized
  request can still succeed.
- When a custom key header is disabled or absent, `x-forwarded-for` is preferred
  and `x-real-ip` is the fallback.
- Under `max_buckets` pressure, stale buckets are removed before evicting the
  oldest still-fresh bucket.

## Breakage Risk

| Area | Risk | Mitigation |
|---|---|---|
| Auth semantics | Low | Behavior now matches existing docs: invalid/missing Bearer returns `401`. |
| Abuse protection | Low | Authorized requests are still rate-limited; unauthorized requests are counted as auth failures rather than rate-limit failures. |
| Metrics | Medium | Unauthorized traffic no longer increments rate-limit counters; this is intentional and documented by contract. |
| Concurrency | Low | Existing mutex boundary is unchanged for authorized requests; unauthorized requests skip it. |
| Tests | Low | Added focused regressions for interaction, fallback keying, stale cleanup, and eviction. |

## Verification

- `cargo test http_transport --lib`
- `cargo clippy --lib --tests -- -D warnings`
- `cargo fmt --all -- --check`
- `bash docs/harness/bin/doctor.sh`
- `git diff --check`
