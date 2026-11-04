# Markdown portability core — independent static review

Target: base60ed4eec9390c9ef526411c9b64ea5f69bfbbcce →
head0da56ffcbdff29960158ff9c4e59b0476ea0284d. Two fresh-context native
reviewers, model inherited from the implementing host; not a cross-CLI Sonnet
post-gate, formal verdict or operator receipt. Reviewers read immutable commit
bytes and ran no candidate code/tests/network calls. Reports below preserve
both axes separately. The implementer separately validated the cited compatibility
and safety properties through source comparison and focused tests.

## Standards

No substantive Standards issues proven in60ed4eec…0da56ffc.

Coverage:

- Shared permission modes, environment policy, principal checks, and compatibility reexports.
- Stored-workspace authorization for export and integer-ID imports.
- Snapshot boundaries, complete-batch authorization, transaction ordering, and uncertain commit outcomes.
- Filesystem confinement, active SQLite protections, ACL handling, and moved reader/writer code.
- MCP response fields, aliases, error envelopes, and public Rust signatures/report behavior.
- Moved expiration/format/status tests and new typed-workflow/environment-policy coverage.
- Harness documentation changes and all twelve smell heuristics, subject to repository rules.

Checked candidate changes against trusted BASE standards and implementation.
No actionable new smell met the repository’s concrete-failure threshold.
Independent static opinion only; no tests or candidate code executed, and no
formal gate approval or receipt produced.

## Spec

No substantive Spec findings in60ed4eec9390c9ef526411c9b64ea5f69bfbbcce…0da56ffcbdff29960158ff9c4e59b0476ea0284d.

The implementation matches “Deepen export and import together” and “Keep public
signatures, reexports, JSON output, defaults, environment modes”: workflow ownership
moves into portability, both MCP handlers retain direct authorization checks, and
the public Rust facade keeps its distinct conversions. No HandlerContext or
crate::mcp references remain under Markdown portability.

Explicit coverage:

- Compared both adapters’ success/error shapes with BASE, including import aliases,
  optional per-file metadata, missing arguments, and uncertain post-commit outcomes.
- Compared Rust facade messages against BASE’s ToolError conversion, including
  empty-export success-zero and non-UTF-8 path fallback.
- Checked environment unset/empty/invalid/non-Unicode handling, configured-root
  mode requirements, per-call restrictions, and principal permission delegation.
- Checked persisted-ID authorization remains on the import mutation/snapshot
  connection, complete-batch authorization precedes writes, and export queries
  remain workspace confined.
- Inspected moved filesystem/reader diffs: pinned-directory, nofollow,
  ownership/ACL, hardlink, active-SQLite protections and quotas retain their implementation.
- Checked preserved formatting/status implementations, moved tests, added
  typed-seam tests, plan/canvas and scope documentation. No schema, embedding,
  cache, HNSW, dependency or remote-operation changes found.

Independent static opinion only. No tests, sensors, network calls or mutations,
and no formal gate verdict or receipt issued.

## Reconciliation

Standards:0 substantive findings. Spec:0 substantive findings. No candidate defect
required a repair or validator. Static review is evidence only; final executable
checks and the formal human-receipt gate are recorded separately in the task log.
