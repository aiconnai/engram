# Review Canvas — Required Aggregate Security Gate

| Field | Value |
|-------|-------|
| Task | `engram-10-of-10-todo-18` |
| Date | `2026-07-12` |
| Scope | GitHub Actions security jobs, aggregate contract checker, fixture, and gate documentation |
| Out of scope | Branch-protection writes, advisory-policy changes, product code, publication |

## Problem

Security and supply-chain scans were split across advisory or independent
workflow signals. A failure could therefore remain disconnected from the
already-required branch-protection contexts, and there was no deterministic
proof of the dependency chain.

## Approaches Considered

- Write branch protection directly: rejected because repository automation is
  not authorized to mutate that external security boundary.
- Treat independent workflow statuses as the aggregate: rejected because
  GitHub Actions cannot express cross-workflow `needs` dependencies.
- Run PR-visible constituent jobs in CI, aggregate them, and make the existing
  required test context depend on the aggregate: selected because it fails
  closed without an external write and remains statically verifiable.

## Failure Semantics

`security-gate` uses `if: always()` so a failed prerequisite cannot silently
skip the aggregate. It accepts only successful constituents on current PR
events. The matrix checker proves every constituent failure is red, an
explicit event-policy skip can be neutral, and removal of the required-context
dependency is rejected.

## Hot-Path / Critical-Path Complexity

The required Ubuntu test now waits for seven independent security constituents
and their small aggregate job. Those constituents run in parallel, so the PR
critical path grows by the slowest scan (normally CodeQL), not by the sum of all
seven jobs. This intentionally trades additional Actions time and merge latency
for a single fail-closed security boundary. The aggregate itself only checks a
bounded seven-entry JSON object and does not compile or scan the repository.

The standalone scanner workflows remain available for GitHub Security/SARIF
visibility, while the CI copies are what create a deterministic `needs` graph.
Version pins and the contract checker keep those duplicated definitions
reviewable; future consolidation must preserve both SARIF visibility and the
required transitive chain.

## Edge Cases

- **Cancelled, timed-out, or missing constituent:** GitHub reports a non-success
  result (or omits it). The checker treats every state other than `success` or
  an explicitly event-allowed `skipped` as failure, so `Security Gate` and the
  dependent required test remain red/skipped rather than passing silently.
- **Fork pull request has downgraded token permissions:** if CodeQL or SARIF
  upload cannot run with the fork token, that constituent fails and the
  aggregate blocks merge. It never falls back to a green advisory result or
  exposes a repository secret.
- **Workflow event legitimately excludes a future constituent:** a skip is
  neutral only after that exact job/event pairing is recorded in the versioned
  matrix. Current PR, push, schedule, and dispatch policies allow no skips.
- **Branch-protection context is renamed or removed:** the read-only live
  context receipt no longer matches a job that reaches `security-gate`, and
  `check-security-gate.py` rejects the chain.

## Breakage-Risk Table

| Risk | Impact | Mitigation |
|------|--------|------------|
| Scan failure silently skips aggregate | Required test might still run | Aggregate uses `if: always()` and inspects every `needs` result |
| Aggregate is not merge-blocking | Red security result remains advisory | Required `Test (ubuntu-latest)` transitively needs `security-gate`; checker validates live contexts read-only |
| Mutable action tag changes executed code | Supply-chain drift | Third-party actions in owned workflows are pinned to immutable commits with version comments |
| Legitimate event exclusion is treated as green accidentally | False success | Skips require explicit matrix policy; current PR jobs have no skip allowance |
| Duplicate standalone scans drift | Conflicting signals | Standalone workflows use the same pinned scanner/action versions as aggregate jobs |

## Reviewer Checklist

- Confirm all seven constituents are direct dependencies of `security-gate`.
- Confirm `test` depends on `security-gate` and its live name remains required.
- Confirm constituent failures and unexpected skips fail closed.
- Confirm no branch-protection or publication write occurs.
- Confirm the checker negative self-tests fail for a constituent failure and an
  unrequired aggregate.
