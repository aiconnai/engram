# Review Canvas: zed-gemini-reviewer-path

Date: 2026-06-22
Owner: Codex
Scope: Clarify that the Gemini reviewer path means Zed's Gemini CLI agent, not the standalone terminal binary.

## Trigger

| Trigger | Evidence |
|---|---|
| User clarification | User specified the Gemini CLI agent in Zed's agent picker as the intended reviewer. |
| Harness script change | `docs/harness/bin/review-gate.sh` contains reviewer handoff instructions. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Keep terminal `gemini -m ...` example | Rejected | It points agents at the wrong surface and previously hit a license blocker. |
| Document both terminal Gemini and Zed Gemini equally | Rejected | The user explicitly identified Zed's Gemini CLI agent as the intended path. |
| Make Zed Gemini CLI the canonical handoff | Accepted | Matches the user's actual workflow and avoids repeated terminal Gemini calls. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| `review-gate.sh pre/post` prompt generation | None | None | Text-only output changes; verdict parsing is unchanged. |
| Manual post-review handoff | None | None | Reviewer is selected in Zed's agent picker before pasting the prompt. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Shell quoting or syntax regresses while changing echo text | Run `bash -n docs/harness/bin/review-gate.sh`. |
| Future agents confuse terminal Gemini with Zed Gemini | Grep active docs and script for terminal `gemini -m` examples. |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| Agents keep using the standalone terminal binary | Medium | Replace active examples with Zed agent-picker guidance. | Grep active docs and script. |
| Review gate behavior changes unintentionally | High | Do not alter prompt construction, verdict parsing, or exit behavior. | `bash -n`, `doctor.sh`, post-review gate. |
| Progress history becomes misleading | Low | Add a new clarification entry instead of deleting prior historical evidence. | Review diff. |

## Decision

Proceed.

Reason: This is a small but process-critical correction to the reviewer handoff
surface. The implementation should stay text-only and preserve the hard
post-review gate contract.
