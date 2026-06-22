# Review Canvas: reviewer-cli-gemini-substitution

Date: 2026-06-22
Owner: Codex
Scope: Replace the active Grok reviewer path with Gemini Flash 3.5 in harness workflow guidance.

## Trigger

| Trigger | Evidence |
|---|---|
| Reviewer CLI unavailable | User stated Grok is no longer available and asked to substitute Gemini Flash 3.5. |
| Harness script change | `docs/harness/bin/review-gate.sh` contains user-facing reviewer handoff text. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Update only local assistant behavior | Rejected | Future agents would still see Grok in active harness guidance. |
| Rewrite all dated historical Grok mentions | Rejected | Historical progress logs should not be falsified. |
| Update active workflow docs and script prompts | Accepted | Keeps current guidance correct while preserving audit history. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| `review-gate.sh pre/post` prompt generation | None | None | Text-only output changes; no new execution path. |
| Manual reviewer handoff | None | None | Reviewer CLI name changes from Grok to Gemini Flash 3.5. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Shell syntax regresses while changing prompt text | Run `bash -n docs/harness/bin/review-gate.sh`. |
| Gemini model ID differs by account or provider release | Keep exact Gemini model ID configurable in the CLI command example. |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| Agents keep calling Grok from current guidance | Medium | Replace active README/progress/review-gate references with Gemini Flash 3.5. | Grep active docs and script for Grok references. |
| Review gate weakens hard post-review enforcement | High | Do not alter verdict parsing or exit behavior. | `bash -n`, `doctor.sh`, and post-review gate. |
| Historical audit record is rewritten incorrectly | Low | Preserve dated historical notes and add a new substitution entry. | Review diff. |

## Decision

Proceed.

Reason: The change is text-only but process-critical because it updates the
reviewer handoff path used by future agents. It should stay scoped to active
workflow guidance and preserve the existing hard review gate contract.
