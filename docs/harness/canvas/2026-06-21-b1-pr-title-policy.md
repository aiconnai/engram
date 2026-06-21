# Review Canvas: b1-pr-title-policy

Date: 2026-06-21
Owner: Codex
Scope: Port the AgentShield PR title policy gate into Engram and wire it into deterministic harness checks.

## Trigger

| Trigger | Evidence |
|---|---|
| Harness script change | Adds `docs/harness/bin/pr-title-policy.sh` and wires it into `sensors.sh`. |
| Harness gate/policy documentation change | Updates `GATES.md` and `doctor.sh` cross-references so the new gate is auditable. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Copy AgentShield `pr-title-policy.sh` verbatim and add deterministic harness checks | Accepted | Keeps parity with the accepted source repo and gives Engram the same title contract. |
| Implement only as documentation guidance | Rejected | Documentation alone would not prevent `[codex]` titles from passing local gates. |
| Add PR title validation into `review-gate.sh` | Rejected | Review gate is for independent diff review; title validation is a small deterministic sensor. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| `pr-title-policy.sh --title` | O(n) in title length | O(1) | Single regex scan over one title. |
| `sensors.sh quick/full` | Constant extra work | O(1) | Runs one accepting title and two expected rejection cases. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Clean title is accepted | `bash docs/harness/bin/pr-title-policy.sh --title "fix: clean title"` exits 0. |
| Plain `[codex]` marker is rejected | `bash docs/harness/bin/pr-title-policy.sh --title "[codex] fix: bad title"` exits 4. |
| Spaced/mixed-case marker is rejected | `bash docs/harness/bin/pr-title-policy.sh --title "[ CoDeX ] fix: bad title"` exits 4. |
| `--stdin` uses the same parser | `printf '%s' "feat: clean" \| bash docs/harness/bin/pr-title-policy.sh --stdin` exits 0. |

## Breakage Risk

| Risk | Impact | Mitigation | Rollback | Verification |
|---|---|---|---|---|
| New sensor check false-positives on normal PR titles | Blocks local harness runs unnecessarily | Regex only rejects bracketed `codex` marker, not ordinary words | Revert this commit to remove the policy gate | Positive title verification exits 0. |
| `sensors.sh` expected-failure handling masks real policy failure | False green on broken rejection path | `run_expected_exit` compares the exact exit code 4 | Revert this commit or remove the PR title policy calls from `sensors.sh` | Negative title verifications exit exactly 4. |
| `doctor.sh` requires a script that is not executable | Harness doctor fails after checkout | Commit the script executable and require it in doctor | Revert this commit | `bash docs/harness/bin/doctor.sh` exits 0. |

## Decision

Proceed.

Reason: This is a small deterministic harness gate copied from the accepted AgentShield implementation, with cheap local verification and a direct rollback path.
