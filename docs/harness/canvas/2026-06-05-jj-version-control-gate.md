# Review Canvas — jj Version-Control Gate

## Task

Add an optional harness gate for issue-level version-control discipline, with
support for Jujutsu (`jj`) as a local workflow layer while keeping Git canonical
for release tags and Cargo publishing.

## Problem

Issue work is accumulating in a dirty worktree while release tags and Cargo
versions remain separate. This makes it easy to lose the boundary between
finished issue work, release commits, and immutable crates.io publishes.

## Approach

Add `docs/harness/bin/vc-gate.sh` as a small explicit gate:

- `status` reports Git and optional jj state.
- `start ISSUE` blocks starting a new issue from an unexplained dirty tree.
- `done ISSUE` requires clean state and recent issue evidence in Git or jj.
- `release VERSION` requires clean state, Cargo version alignment, and release
  tag alignment.

The script is observational and blocking only. It does not create commits, run
`jj new`, move tags, or publish.

## Alternatives Considered

1. Put dirty-worktree checks inside `doctor.sh`.
   Rejected: dirty worktrees are normal during implementation, and `doctor.sh`
   is a harness integrity check rather than an issue-boundary gate.

2. Auto-run `jj new` in the gate.
   Rejected: the harness should not silently mutate version-control topology.
   Agents and humans should choose when to create, split, squash, or describe
   changes.

3. Replace Git release flow with jj tags.
   Rejected: Cargo publishing and GitHub release workflows still expect Git as
   canonical, and jj lightweight tags are not a drop-in replacement for annotated
   release tags.

## Edge Cases

- Existing tag points away from `HEAD`: `release VERSION` fails.
- Cargo manifest version differs from requested version: release gate fails.
- jj is installed but the repo is not colocated: status prints a hint only.
- Worktree is dirty at issue start/done/release: gate fails unless an explicit
  allow flag is used.

## Breakage Risk

| Area | Risk | Mitigation |
|------|------|------------|
| Existing bootstrap | Low | No bootstrap changes. |
| Existing sensors | Low | No default sensor changes. |
| Git release flow | Low | Git remains canonical for tags and publish. |
| jj adoption | Medium | Optional only; script detects availability. |

## Review Notes

This is a process harness change, not a product behavior change. Post-review
should focus on false confidence, accidental mutation, and release-safety gaps.

