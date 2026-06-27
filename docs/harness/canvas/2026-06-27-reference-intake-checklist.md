# Review Canvas: reference-intake-checklist

Date: 2026-06-27
Owner: Codex
Scope: Add a lightweight intake contract for external harness references and wire it into harness review/gate policy.

## Trigger

| Trigger | Evidence |
|---|---|
| Harness policy change | `docs/harness/GATES.md` and `docs/harness/CODE_REVIEW_POLICY.md` are updated. |
| External-source governance | The change is based on reviewing `walkinglabs/awesome-harness-engineering` and local prior 12207/reference-policy work. |

## External Reference Intake

| Field | Evidence |
|---|---|
| Source identity | `walkinglabs/awesome-harness-engineering`, https://github.com/walkinglabs/awesome-harness-engineering, read on 2026-06-27. GitHub MCP root listing resolved default-branch content at commit `f84f1701974cf1ad67dd774b025b33e613275cee`; volatile repo status was rechecked on 2026-06-27 before final synthesis. |
| Source type | Awesome list / curated reference catalog for harness-engineering resources, plus lightweight contribution workflow and link-check CI. |
| License boundary | Repository license is CC0 1.0. Adaptation still uses Engram-local wording; no prompts, scripts, checklists, or prose are copied into Engram. |
| Harness relevance | Governance, taxonomy, exception discipline, curation review metadata, and future verifier opportunities for duplicate/entry-shape checks. |
| Placement | New canonical checklist in `docs/harness/REFERENCE_INTAKE.md`; enforcement summaries in `GATES.md` and `CODE_REVIEW_POLICY.md`; canvas records the decision. Adjacent placements rejected: `GATES.md` alone would scatter policy, and script automation is deferred. |
| Adaptation | Convert the external repo review into an Engram-local intake contract requiring source identity, source type, license boundary, relevance, placement, adaptation, exclusions, and verification evidence. |
| Exclusions | No vendor/import of the awesome list, no CI/link-check workflow copy, no issue/PR governance import, no release-policy import, no autonomous execution, and no new default sensor. |
| Verification | `doctor.sh`, markdown hygiene, `git diff --check`, `sensors.sh quick`, full `sensors.sh`, and independent post-review. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Add only a note in `GATES.md` | Rejected | Too easy to grow into scattered policy; the intake checklist deserves one canonical doc. |
| Add `REFERENCE_INTAKE.md` plus policy links | Accepted | Keeps the first slice documentation-only, auditable, and low risk. |
| Add scripts for duplicate/link/exception checks now | Rejected | Useful later, but script changes would expand scope and require stronger gate evidence. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| Agent/reviewer reading harness policy | Low | Low | Adds one small doc and short policy references. |
| Runtime/product paths | None | None | Documentation-only; no Rust, MCP, SDK, storage, or runtime behavior changes. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| The checklist becomes a hidden dependency that overrides local invariants | `REFERENCE_INTAKE.md` states local invariants, gates, security boundary, and negative scope still win. |
| External licensed text is copied through the checklist process | Policy requires license boundary and makes copied licensed prompts/scripts/text a blocker. |
| Simple product-doc links become overburdened | Applicability section excludes routine user-facing links unless they change process, gates, or agent instructions. |
| Ambiguous source placement causes taxonomy drift | Placement guidance requires the narrowest local home and rejected placements when ambiguous. |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| Harness docs drift or broken references | Medium | Cross-link from GATES and CODE_REVIEW_POLICY to the new canonical doc. | `doctor.sh`, `git diff --check`, grep for links. |
| Scope creep into automation | Medium | Explicitly rejects scripts in this slice. | Diff review confirms docs-only scope. |
| Reviewer burden increases too much | Low | Checklist applies only to process-affecting external references, not routine links. | Applicability section reviewed. |

## Decision

Proceed: documentation-only intake contract, cross-linked from gates and review policy, with automation deferred to a later scoped task.
