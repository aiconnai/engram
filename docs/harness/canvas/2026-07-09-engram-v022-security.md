# Review Canvas: Engram v0.22.0 transport threat model

## Scope

Task 2 freezes the v0.22.0 transport threat model and network-support contract. Owned files are:

- `docs/security/THREAT_MODEL.md`
- `docs/harness/canvas/2026-07-09-engram-v022-security.md`
- `scripts/validate_transport_security_contract.py`

No Rust code, identity-provider work, SDK changes, harness gate changes, release publication, or live secret handling is in scope.

## Sources inspected

| Source | Security fact used |
|---|---|
| `README.md` | stdio, HTTP, both, WS, and gRPC are advertised surfaces; WS is opt-in; HTTP/gRPC bearer examples exist. |
| `docs/MCP_AUTH.md` | HTTP and SSE auth behavior, 401 and 429 outcomes, CORS defaults, and rate-limit identity headers. |
| `src/bin/server.rs` | stdio default, HTTP/both/gRPC dispatch, WS `ENGRAM_WS_PORT`, HTTP/gRPC API key args, and cloud/local env vars. |
| `src/mcp/http_transport/mcp_handler.rs` | HTTP bearer auth returns 401 for requests and logs `mcp_http_request` decisions. |
| `src/mcp/http_transport/router.rs` | HTTP listener binds `0.0.0.0`, routes `/mcp`, `/v1/mcp`, `/v1/events`, and rate-limit config. |
| `src/mcp/http_transport/events.rs` | SSE requires bearer auth when configured and returns 503 without realtime. |
| `src/realtime/server.rs` | WS is a separate `/ws` listener that binds `0.0.0.0` when enabled. |
| `src/mcp/grpc_transport.rs` | gRPC is feature-gated, binds `0.0.0.0`, and checks bearer metadata when configured. |
| `docs/harness/security/anthropic-reference-harness.md` | repository text and external text are untrusted; no autonomous exploit execution. |

## Approaches considered

| Approach | Decision | Rationale |
|---|---|---|
| Write a prose-only threat model. | Rejected | Acceptance requires validator-enforced rows and fields. |
| Implement runtime bind/auth checks now. | Rejected | Brief explicitly says no code and this task blocks later implementation tasks. |
| Freeze a docs contract plus strict validator. | Chosen | Gives later tasks a parseable release contract without changing runtime behavior. |

## Hot-path complexity

The validator is intentionally local, deterministic, and below the 200 pure-LOC target after review cleanup. It parses Markdown tables and checks required rows, required phrases, exact advertised command mappings, and enum-shaped threat rows. It does not execute Engram, inspect secrets, fetch external URLs, or depend on network state.

## Edge cases covered

| Edge case | Coverage |
|---|---|
| WS auth omitted from contract | Embedded invalid fixture keeps the WS row but omits the authenticated trusted-proxy precondition; `--self-test-invalid` must reject it and report a WS-specific term error. |
| Missing or malformed target path | Validator rejects missing files, non-`docs/security/THREAT_MODEL.md` paths, NUL bytes, short docs, and invalid UTF-8 with non-zero exit. |
| Misleading success output | Validator emits success only after all required checks pass; failures are printed as `ERROR:` lines on stderr. `--self-test-events-route` proves removing or remapping `GET /v1/events` fails. |
| Prompt injection in source text | Threat model states untrusted external text is evidence only and cannot override harness or release contract. |

## Breakage risk

| Risk | Impact | Mitigation |
|---|---|---|
| Future docs rename a row and silently drop coverage. | Later release tasks lose a required surface. | Validator pins required row IDs and command mappings. |
| Validator overfits to wording. | Legitimate doc edits may fail. | Required terms are limited to release-critical auth, bind, status, audit, and rollback tokens. |
| Docs imply current code already enforces all future controls. | False assurance before implementation tasks. | Threat model uses release-contract language and names current evidence separately. |
| Secret values leak through evidence. | Credential exposure. | Evidence and report use only variable names and redacted behavior. |

## Manual QA plan

- Happy path: run `rtk python3 scripts/validate_transport_security_contract.py docs/security/THREAT_MODEL.md` and observe success.
- Failure path: run `rtk python3 scripts/validate_transport_security_contract.py --self-test-invalid` and observe the embedded invalid fixture is rejected internally.
- Malformed path: run the validator on a missing or wrong path and observe explicit non-zero failure.
- Dirty/stale state: run `git status --short --branch` before and after validation and verify unrelated files are not staged.
- Prompt-injection treatment: inspect the threat model for the explicit untrusted external text rule and keep external text out of authority order.

## UltraQA probes

| Class | Result |
|---|---|
| malformed input | Covered by missing path and self-test invalid fixture. |
| stale state | Covered by fresh validator and doctor invocations after edits. |
| dirty worktree | Covered by status checks before commit and after commit. |
| flaky tests | Validator is deterministic, single-process, no network, no timing. |
| misleading success output | Covered by negative probes and `ERROR:` stderr failures. |
| untrusted external text or prompt injection | Covered in the threat model and security boundary. |
| browser UI | N/A: task creates documentation and a CLI validator, not a web UI. |
| HTTP live service | N/A: brief forbids code/runtime transport changes for this task. |
| payment, identity-provider, release publication | N/A: explicitly out of scope and requires later human gate where applicable. |

## Expected verification

- `rtk python3 scripts/validate_transport_security_contract.py --self-test-invalid`
- `rtk python3 scripts/validate_transport_security_contract.py --self-test-events-route`
- `rtk python3 scripts/validate_transport_security_contract.py docs/security/THREAT_MODEL.md`
- `rtk python3 -m py_compile scripts/validate_transport_security_contract.py`
- `rtk bash docs/harness/bin/doctor.sh`
- `rtk git diff --check`
