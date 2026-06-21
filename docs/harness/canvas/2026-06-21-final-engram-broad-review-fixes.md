# Review Canvas: final Engram broad review fixes

Date: 2026-06-21
Owner: Ronaldo / agent
Scope: Align Engram PR-title policy scripts after the final broad review found inconsistent gate wiring.

## Trigger

| Trigger | Evidence |
|---|---|
| Final broad review BLOCKER/HIGH | Review found `check-pr-title.sh` and `pr-title-policy.sh` had divergent exit contracts and doctor validated the wrong script. |
| Harness script change | Updates `check-pr-title.sh` and `doctor.sh`. |
| Harness policy docs change | Updates `GATES.md` and `README.md` to identify the canonical policy and compatibility wrapper. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Make `check-pr-title.sh` delegate to `pr-title-policy.sh` | Accepted | Keeps backward-compatible CLI while using one canonical implementation and exit contract. |
| Delete `check-pr-title.sh` | Rejected | Existing Engram harness docs and workflows reference it; deletion is broader than needed. |
| Teach `doctor.sh` to test only `check-pr-title.sh` | Rejected | `sensors.sh` uses `pr-title-policy.sh`, so doctor must require and test the canonical script directly. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| `check-pr-title.sh --title` | O(n) in title length | O(1) | Thin wrapper around `pr-title-policy.sh`. |
| `doctor.sh` PR-title checks | O(1) extra commands | O(1) | Adds exact exit-code checks for canonical policy. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| `pr-title-policy.sh` rejects `[codex]` with exit 4 | Run direct `--title` check and assert exit 4. |
| `pr-title-policy.sh` rejects `[ CoDeX ]` with exit 4 | Run direct spaced/mixed-case check and assert exit 4. |
| `check-pr-title.sh` wrapper shares canonical exit code | Run wrapper `--title "[codex] ..."` and assert exit 4. |
| Doctor validates canonical script and sensors wiring | `bash docs/harness/bin/doctor.sh` exits 0. |

## Breakage Risk

| Risk | Impact | Mitigation | Rollback | Verification |
|---|---|---|---|---|
| Existing caller expects `check-pr-title.sh` rejection exit 1 | Caller may need to accept canonical exit 4 | Document wrapper as compatibility interface with canonical policy exit codes | Revert this commit | Wrapper exit-code test confirms new behavior. |
| Missing `pr-title-policy.sh` not caught by doctor | Sensors fail later instead of doctor | Doctor now requires and tests the canonical script directly | Revert this commit | Doctor exits 0 only when canonical script is present/executable. |

## Decision

Proceed.

Reason: One canonical policy script removes fake parity while preserving the older wrapper interface.
