# Review Canvas — harness-review-perspectives

| Field | Value |
|-------|-------|
| Date | 2026-06-26 |
| Task | `harness-review-perspectives` |
| Scope | Add Engram-local review perspectives to `CODE_REVIEW_POLICY.md` |
| Owner | Codex |

## Goal

Strengthen the local review policy with explicit review perspectives while
keeping the Engram harness authoritative and avoiding license-sensitive imports
from external workflow kits.

## Approaches Considered

| Approach | Decision | Rationale |
|----------|----------|-----------|
| Add a small local taxonomy to `CODE_REVIEW_POLICY.md` | Chosen | Directly affects the reviewer prompt source of truth with minimal surface area. |
| Add a new standalone external-patterns doc | Rejected for now | Larger process surface than needed for the first iteration. |
| Vendor or quote external plugin/checklist text | Rejected | Creates GPL/license risk and weakens the local harness boundary. |
| Change `review-gate.sh` prompt generation | Rejected | The script already injects `CODE_REVIEW_POLICY.md`; no process-critical script change is needed. |

## Hot-Path Complexity

No runtime hot path changes. The policy is read by reviewers and injected into
review prompts; complexity impact is limited to reviewer guidance.

## Edge Cases

1. **Docs-only harness change**: reviewer must still inspect process drift,
   negative scope, gate weakening and security boundary, not just code bugs.
2. **External pattern reuse**: reviewer must distinguish high-level inspiration
   from copying prompts, scripts, checklists or docs from a copyleft source.

## Breakage Risk

| Area | Risk | Mitigation |
|------|------|------------|
| Review signal | Taxonomy could make reviews verbose or generic | Policy keeps the existing max-findings bar and evidence requirement. |
| License boundary | External reference could invite copied text | Added explicit no-copy-without-license-review rule. |
| Harness process | Policy change could drift from gates | `doctor.sh` remains the consistency gate; no script behavior changed. |
| Scope creep | Broader external-kit adoption could sneak in | Canvas rejects vendoring, marketplace dependency and prompt/script import. |

## Verification Plan

- `rtk bash docs/harness/bin/doctor.sh`
- `rtk git diff --check`
- Review resulting diff for copied external prose or script behavior changes.
