# Review canvas — keyless-http-authz-2026-11-10

Base: main `85502ad8`. Candidate: `7a572e15` on `claude/revise-e-avalie-62b8e4`.
Scope: findings A-1 to A-4 of the 2026-11-10 read-only review (keyless HTTP
boundary, read-only authorization gaps). No storage schema, SDK or tool-name
change. Triggers: MCP surface change (`palace_visualize` permission and
`output_path` description), new transport middleware, more than 200 lines.

| Alternative | Decision |
|---|---|
| A-1: keep `palace_visualize` read-only and only harden the path check | Rejected: saving to a server path is the same filesystem write as a Markdown export |
| A-1: reuse `markdown_required_mode()` (`scoped_write` with `ENGRAM_EXPORT_BASE_DIR`, else `maintenance`) | Selected: one rule for every server-side file write |
| A-2: reject `/v1/events` without `workspace` for restricted principals | Rejected: breaks the loopback default client that names no workspace |
| A-2: confine an unnamed stream to `default` unless the principal reads all workspaces | Selected: matches `allows_workspace(None)` for the anonymous loopback principal |
| A-3: filter expanded crossrefs by searched workspace and principal | Selected; `valid_to` filtering and `include_graph` neighbour IDs left out of scope |
| A-4: require an API key for every HTTP server | Rejected: breaks the documented keyless loopback mode |
| A-4: on keyless servers only, require `Host`/`Origin` to name this machine | Selected; `ENGRAM_HTTP_ALLOWED_HOSTS` and `ENGRAM_CORS_ORIGINS` extend the allowlist |

Hot path: A-4 adds one header parse and a short list comparison per request on
keyless servers only; keyed servers skip it. A-3 adds one comparison per
expanded memory. A-2 is per event, unchanged complexity. A-1 adds one
`canonicalize` of the nearest existing ancestor per save.

| Edge / breakage risk | Evidence / containment |
|---|---|
| DNS rebinding: `Host: attacker.example` resolving to 127.0.0.1 | `auth_router` tests: 403 on foreign `Host` and on foreign `Origin` |
| `http://localhost.attacker.example` as Origin | Exact host comparison after URI parse; prefix test not used in the new path |
| Duplicate or malformed `Host`/`Origin` headers | Refused with a distinct reason string; `keyless_boundary_refuses_ambiguous_or_malformed_headers` (also `Origin: null`) |
| Non-browser client sending neither header | Passes, same as before |
| Keyless server reached through a local proxy or container name | Behaviour change: 403 until `ENGRAM_HTTP_ALLOWED_HOSTS` or an API key is set; documented in `docs/MCP_AUTH.md` and `CHANGELOG.md` |
| 403 body shape on `/mcp` | JSON-RPC error `-32003`, same code `mcp_handler.rs` already returns for forbidden calls |
| Path outside export base creates directories before refusal | Containment checked before `create_dir_all`; child-process test asserts no directory is created |
| `..` or final-component symlink under export base | Refused; test covers both |
| stdio without `ENGRAM_PERMISSION_MODE` | Unchanged: tool still writes anywhere (local-first, deliberate) |
| Check-then-write race on the final path | Not closed: no descriptor-bound open (same class as RISK-0007) |
| Crossref into a foreign workspace during `depth` expansion | `handlers::context` test; neighbour IDs via `include_graph` still returned |

Required before upstream: clippy and tests with `CI_REQUIRED_FEATURES`, full
`sensors.sh`, independent post-review and the owner's H1 receipt. Push, PR and
merge remain separate owner decisions.
