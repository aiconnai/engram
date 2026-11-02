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
