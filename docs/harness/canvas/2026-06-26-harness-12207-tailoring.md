# Review Canvas: harness-12207-tailoring

Date: 2026-06-26
Owner: Codex
Scope: Add an Engram-local tailoring checklist for future use of
`docs/ieee-12207.md` as a lifecycle-process pattern source.

## Trigger

| Trigger | Evidence |
|---|---|
| Harness gate/process change | `docs/harness/GATES.md` gains a new checklist that reviewers must enforce. |
| Review policy change | `docs/harness/CODE_REVIEW_POLICY.md` now tells reviewers how to apply the checklist. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Add scripts to enforce 12207 fields | Rejected | Too heavy for a first pass and risks false bureaucracy before the workflow proves useful. |
| Add a separate full 12207 mapping document | Rejected | More maintenance surface than needed; the immediate gap is a lightweight adoption guard. |
| Add a checklist to `GATES.md` and a reviewer hook in policy | Accepted | Keeps the rule close to existing gates and makes missing tailoring evidence reviewable. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| `doctor.sh` | None | None | No script or executable contract changed. |
| `sensors.sh` | None | None | Default full gate remains unchanged. |
| `review-gate.sh` prompt/policy | Negligible | None | Reviewer reads one additional policy section for applicable lifecycle/process changes. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| A future task cites 12207 only as background material | Checklist applies only when the diff changes local process or claims lifecycle adoption. |
| A future harness script weakens a gate while citing lifecycle process | Policy escalates missing tailoring and reversibility evidence to blocker. |
| A future docs-only change copies standards wording | Policy directs reviewers to flag copied licensed wording and require Engram-local language. |
| A local licensed copy is useful for analysis but unsafe to distribute | `.gitignore` keeps `docs/ieee-12207.md` local-only while committed docs refer to it as a private pattern source. |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| Checklist becomes bureaucratic noise | Agents may over-document simple tasks | Applicability is limited to cited lifecycle/process changes and gate-affecting work | Review this canvas and changed sections only |
| Review policy contradicts existing gates | Reviewers could apply inconsistent severity | Severity aligns with existing Review Canvas and process-critical script rules | `doctor.sh` plus scoped post-review |
| Implied standards conformance | Legal/process confusion | Text explicitly rejects conformance claims and drop-in adoption | Manual read of changed docs |
| Local reference accidentally committed | Licensed source text enters the public repo | Ignore the local 12207 path and require Engram-local summaries/checklists | `git status --short` should not show `docs/ieee-12207.md` |

## Decision

Proceed:

Reason: The change captures useful 12207 patterns as local evidence discipline
without adding new commands, copying standard text, or claiming conformance.
