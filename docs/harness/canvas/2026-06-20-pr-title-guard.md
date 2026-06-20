# Review Canvas: pr-title-guard

Date: 2026-06-20
Owner: Codex
Scope: Add a harness gate that prevents the `[codex]` marker in automated PR titles.

## Trigger

| Trigger | Evidence |
|---|---|
| Harness gate/script change | Adds `docs/harness/bin/check-pr-title.sh` and wires `doctor.sh` self-tests. |
| Harness policy/invariant change | Updates `INVARIANTS.md`, `GATES.md`, and `README.md` with the PR title contract. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Rely on PR template text only | Rejected | Templates are advisory and do not fail automation before PR creation or edit. |
| Add a standalone title checker | Accepted | Smallest enforceable boundary, usable before both `gh pr create` and `gh pr edit`. |
| Add a GitHub Actions-only check | Rejected | CI would catch the issue after the bad title is already published. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| Local `--title` validation | O(title length) | O(title length) | Only trims, lowercases, and checks for the forbidden marker. |
| `--pr` validation | One `gh pr view` call | O(title length) | Used for existing PRs or preflight checks against GitHub state. |
| `doctor.sh` self-test | Two local checker invocations | O(title length) | Keeps harness consistency fast and read-only. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| Plain PR title should remain valid | `bash docs/harness/bin/check-pr-title.sh --title "align lifecycle hook contracts"`. |
| Forbidden marker appears in the title | Negative test wraps `check-pr-title.sh --title "[codex] align lifecycle hook contracts"` and requires non-zero exit. |
| Existing PR title fetched from GitHub | `bash docs/harness/bin/check-pr-title.sh --pr 91`. |
| Option-like PR identifier | `bash -c 'if docs/harness/bin/check-pr-title.sh --pr --help; then exit 1; else exit 0; fi'`. |
| Trailing help after validation arguments | Negative tests wrap `--title "[codex] ... " --help` and `--pr 91 --help`; both must exit non-zero. |
| Duplicate validation arguments | Negative test wraps `--title "[codex] ..." --title "clean title"` and requires non-zero exit. |
| Guard script drift | `bash docs/harness/bin/doctor.sh` self-tests the allow and block paths. |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| Automation forgets to call the new checker | Bad title can still be created manually | README and GATES document the required preflight; doctor enforces script presence. | `doctor.sh` cross-reference checks. |
| Checker blocks unrelated title text | False PR-title failure | Only rejects the exact case-insensitive marker and empty titles. | Positive title test and existing PR #91 lookup. |
| Option-like PR identifier reaches `gh` | Guard validates help or another target instead of a PR title | `--pr` accepts digits only before invoking `gh`. | Option-like negative test and doctor self-test. |
| Trailing `--help` bypasses validation | Guard exits successfully after a complete but invalid validation request | `--help` is accepted only as the sole argument. | Trailing-help negative tests and doctor self-tests. |
| Duplicate validation argument overwrites forbidden title | A later clean argument hides an earlier forbidden marker | Validation modes are mutually exclusive and cannot repeat. | Duplicate-title negative test and doctor self-test. |
| `gh` missing for `--pr` mode | Existing PR validation cannot run | `--title` mode has no GitHub dependency; `--pr` exits with usage/env error. | Syntax check and direct title tests. |

## Decision

Proceed.

Reason: the requested rule is title-specific, low risk, and best enforced before publication. A small local checker plus doctor self-test prevents future harness-driven PRs from reintroducing the marker while keeping manual title validation simple.
