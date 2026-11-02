# Markdown portability core — evidence log

## Authorization and baseline

User approved compatibility, both workflows, shared auth distribution and final
implementation summary on 2026-11-02. Base 60ed4eec, clean new worktree. Root
checkout and integration candidate left intact. Bootstrap and doctor executed;
doctor exit 0 with existing missing-review warning. Plan/canvas recorded.

The prescribed lazycodex-ai doctor unexpectedly delegated to a Codex process with
a broader source-sync diagnosis. That invocation was stopped before implementation;
no lazycodex run was used. Continue directly under the approved local scope.

## Implementation and focused checks

Workflow implementation moved into portability/markdown with a Storage+principal
context and typed requests/outcomes. MCP adapters retain JSON decoding, aliases
and error envelopes. The public Rust facade directly projects typed reports,
including empty-export success and existing error messages. Shared mode/env and
persisted-row checks moved to auth/access; old MCP imports remain via reexports.

Baseline:103 Markdown tests PASS,1 manual probe ignored. Current markdown filter:
112 PASS,1 same ignored probe; includes4 new seam tests. Environment case children
select exactly1 test and have20s deadlines. Integration suites CLI/vault/permission/
G1 passed; exact counts recorded below after log reconciliation. Seven moved
filesystem/frontmatter/status/format files are byte-identical after namespace
replacement; import/files.rs differs additionally only by rustfmt import wrapping.

An initial command used non-existent feature watch, corrected to CI_REQUIRED_FEATURES.
Intermediate compile errors/private visibility and a new fixture expecting index.md
to be excluded from import were corrected; index import behavior was preserved.
No production behavior fix was bundled. Clippy/full gate and independent review pending.

Integration count reconciliation: CLI3 + permission/routing3 + G1 8 (1 helper
ignored but invoked by parents) + vault6 =20 PASS. The markdown filter's ignored
probe is the pre-existing manual case, not a new exclusion. Doctor exit0 after
plan/canvas/live-state updates. No harness scripts or policies modified.

## Frozen implementation and final evidence

Implementation commit:0da56ffcbdff29960158ff9c4e59b0476ea0284d. Pre-commit hooks
were enabled: fmt + clippy all-targets/all-features PASS (20.48s). Separate
CI_REQUIRED_FEATURES clippy PASS (16.82s). Public Markdown types, parsing and
Default implementations are byte-identical; signatures, portability reexports,
CLI, Cargo.toml/Cargo.lock and MCP reference remain unchanged. No crate::mcp
or HandlerContext references remain anywhere in portability, including tests.

`sensors.sh` full PASS:2026-11-02T16:51:50Z,410s, no sensor exclusions. Steps:
fmt, clippy, library/integration tests, server/watcher tests, WASM host/target,
rustdoc, MCP reference, mandatory offline lane, PR-title positive/negative cases,
and doctor. Negative PR-title fixtures intentionally print FAIL; aggregate PASS
is the final result. The only doctor warning is the existing missing authoritative
review for active remediation task. This is local verification, not remote CI.

[Independent static review](../reviews/2026-11-02-markdown-portability-core-static.md):
two fresh-context native reviewers, Standards0 / Spec0 substantive findings on
60ed4eec..0da56ffc. Both terminal; no code changes followed their frozen target.
Not a canonical Sonnet review, cross-model assertion or formal acceptance.

Formal post-gate: trusted gate extracted from60ed4eec outside every worktree,
executed with --repo and exact base/head. Exit3, reason review-missing. Prompt:
`docs/harness/reviews/2026-11-02-markdown-portability-core-v2-post.md.raw` (ignored).
No operator receipt, reviewer response or authoritative post-gate PASS fabricated.
The final handoff commit contains evidence/docs only; formal acceptance remains
pending the separate Sonnet review and human receipt for the final exact range.

Logs: `target/markdown-portability-core/` in the primary checkout (baseline,
focused, clippy, integration, commit, full, pre, post and intermediate compile
logs). These logs are ignored artifacts; versioned summaries and sensor receipts
preserve the observed results. Root planning checkout and integrated safety
worktree retained. No push, PR, merge, tag, publish or production access.

Handoff validation caught an over-abbreviated Last live-state check field: it
must include the complete checker CLI invocation. Restored that literal command;
this correction changes documentation only, not the tested/reviewed source.
