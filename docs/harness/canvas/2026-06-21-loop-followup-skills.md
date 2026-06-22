# Review Canvas: loop-followup-skills

Date: 2026-06-21
Owner: Codex
Scope: Promote the four AgentShield follow-up loop skills into Engram as
repo-local, report-first operational skills with Engram-specific guardrails.

## Trigger

| Trigger | Evidence |
|---|---|
| Harness operational-surface change | Adds four repo-local skills and promotes them into `docs/harness/SKILLS.md` Current Skills. |
| Loop behavior guidance changes | Adds triage loops for daily signals, CI failures, dependency advisories, and PR review queues. |
| Cross-repo port | Sources come from AgentShield skills and require Engram-specific adaptation. |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
| Keep the skills documented as follow-ups only | Rejected | The user requested the next follow-up work after B2; leaving them unported does not satisfy that request. |
| Port all four related follow-up skills together | Accepted | They share the same inventory/doctor surface and are all report-first loop triage skills; one canvas keeps their common safety review coherent. |
| Create four separate PRs | Rejected for this pass | It would repeat identical inventory/gate churn four times while the skills are small and share one policy boundary. |
| Copy AgentShield skills verbatim | Rejected | AgentShield references do not fully match Engram; content is adapted to Engram loop state paths, `sensors.sh`, `just ci`, MCP/connectors, attestation, storage, and release-risk boundaries. |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| `doctor.sh` skill validation | O(k) in number of repo-local skills | O(1) | Existing validation loops over `skills/*/SKILL.md`; this raises k from 4 to 8. |
| Agent skill discovery | O(k) metadata read by tooling | O(1) per skill | Each skill has concise frontmatter and no bundled resources. |
| Runtime product path | None | None | No Rust, storage, MCP, or runtime code changes. |

## Edge Cases

| Edge case | Verification plan |
|---|---|
| A promoted skill exists on disk but is missing from `Current Skills` | `bash docs/harness/bin/doctor.sh` fails via existing disk-to-inventory validation. |
| A promoted skill is listed in `Current Skills` but missing on disk | `bash docs/harness/bin/doctor.sh` fails via existing inventory-to-disk validation. |
| Frontmatter name drifts from directory name | `bash docs/harness/bin/doctor.sh` fails via existing `name:` validation. |
| Skill text keeps AgentShield-only assumptions | Manual review plus grep for AgentShield-specific commands verifies Engram wording uses `sensors.sh`, `just ci`, and Engram risk boundaries. |

## Breakage Risk

| Risk | Impact | Mitigation | Rollback | Verification |
|---|---|---|---|---|
| Skills imply unsafe autonomous writes | Agents may over-execute instead of report-only triage | Each skill defaults to L1/report-only or verifier-gated L2 and references `loop-engineering` for global controls | Revert this commit to remove the four skills and restore the follow-up table | Manual review of each `SKILL.md`; `doctor.sh` validates inventory. |
| AgentShield terminology leaks into Engram | Operators may run wrong commands or apply wrong denylist | Adapted full-gate language to `bash docs/harness/bin/sensors.sh`, `just ci`, Engram MCP/storage/attestation boundaries | Revert the skill files or patch the affected wording | Grep/manual read of the four skill files. |
| Doctor validation cost grows with skill count | Local harness checks get slower | Existing validation is bounded and linear over 8 small files | Revert the promotion | `bash docs/harness/bin/doctor.sh` and `sensors.sh quick` runtime remains small. |

## Decision

Proceed.

Reason: The four skills are the explicit B2 follow-ups, share one existing
inventory validation surface, and remain report-first by default. The change is
documentation/skill policy only, with a direct rollback path and deterministic
doctor coverage.
