# Review Canvas — ENGRA-111 Mock Parity Harness

## Change

Add a deterministic MCP parity test driven by a small JSON fixture.

## Approaches Considered

- Direct Rust assertions only: simpler, but harder for SDKs to reuse later.
- Snapshot full MCP responses: catches more drift, but unstable IDs, timestamps,
  scores, and generated values would create noisy failures.
- Fixture inputs plus normalized expectations: chosen because it exercises the
  real `tools/call` path while keeping the contract stable and portable.

## Hot Path Complexity

No production hot path changes. The new code runs only in
`tests/mcp_protocol_tests.rs` against in-memory SQLite.

## Edge Cases

- `memory_create` serializes the public memory type as `type`, while create
  input accepts `memory_type`; the normalizer accepts the public key.
- Unknown tools are returned as tool-result JSON errors, not MCP transport
  errors, so parity covers the existing public envelope.

## Breakage Risk

| Area | Risk | Mitigation |
| --- | --- | --- |
| MCP response shape | Medium | Fixture normalized expectations fail on missing public fields. |
| Determinism | Low | IDs, timestamps, scores, and UUID-like values are excluded. |
| SDK expansion | Low | Fixture README documents reuse of scenario names and normalized output. |
| Production behavior | Low | No production code or registry changes. |
