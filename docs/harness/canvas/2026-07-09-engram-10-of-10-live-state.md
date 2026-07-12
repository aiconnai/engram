# Review Canvas: engram-10-of-10-live-state

Date: 2026-07-10
Owner: Codex harness worker
Scope: Make harness live-state metadata verifiable against current repository facts without changing product behavior.

## Trigger

| Trigger | Evidence |
|---|---|
| Harness script change | New `docs/harness/bin/check-live-state.sh` and test driver under `docs/harness/bin/`. |
| Harness process verification | Progress live state now records execution HEAD, approved baseline/snapshot commit, sensor snapshot, checker status, and workflow enforcement mapping. |
| Approved execution baseline | `843fd520cbd0eb4c2b1885fe11c997198beb2ca1` is valid only when bound to snapshot commit `3586a40e7952a051181d162028927a40bd6292f6`, the current review artifact, and this Canvas. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Documentation-only update | Rejected | Would leave stale HEAD/sensor drift possible with no local assertion surface. |
| Add checker with `--progress` fixture support | Accepted | Smallest self-checking surface; supports current progress and temp stale-SHA fixtures. |
| Query GitHub branch protection inside checker | Rejected | Would make the local checker network/credential dependent and risk fabricating branch-protection state when unavailable. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| `check-live-state.sh --progress docs/harness/progress.md` | O(lines in progress + small workflow greps) | Constant | Read-only; no network; no generated artifacts. |
| `test-check-live-state.sh` | O(checker runs over small fixtures) | One temp dir | Cleans temp fixtures and dirty-worktree probe on exit. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Stale `Last commit` contains old `1aa14e5` | Temp fixture must exit non-zero and print remediation to update `Last commit`. |
| Malformed progress file lacks live-state table | Temp fixture must exit non-zero and print remediation to restore the table. |
| Dirty worktree during in-flight agent work | Checker must report `worktree_status=dirty` without pretending cleanliness. |
| Misleading success output on failure | Failure output must not contain the final PASS line. |
| Repeated checker runs | Test driver runs the current progress check twice to catch obvious nondeterminism. |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| Checker blocks valid in-progress work because worktree is dirty | Agents could not run acceptance before commit | Dirty state is diagnostic, not a hard failure | Dirty probe in `test-check-live-state.sh`. |
| Workflow table diverges from workflows | Future agents may infer wrong required/advisory status | Checker validates branch-required/non-inferred/advisory rows and workflow job names separately | `check-live-state.sh --progress docs/harness/progress.md`. |
| Branch protection state is fabricated from workflows | Incorrect governance claims | GitHub API receipt is separate; Security Audit/Cargo Deny are recorded as branch-required, and Harness Contract is not inferred from workflow text | Evidence receipt includes `gh api ...required_status_checks...` result or unknown. |

## Decision

Proceed.

Reason: the change is local, read-only, reversible, and adds a deterministic CLI surface for current and fixture progress validation while preserving network-dependent branch-protection truth as receipt evidence only.
