# Review Canvas: agent-harness-hardening-v1

Date: 2026-07-21  
Owner: Ronaldo + agents  
Scope: Wave 0 governance for a SHA-bound, isolated agent harness.

## Trigger

| Trigger | Evidence |
|---|---|
| Harness policy change | The proposal affects trust, review, evidence, runtime, and merge boundaries. |
| Future autonomous execution | Invariant 23 requires ADR, sandbox, egress, credential, and target contracts first. |
| External blueprint | The operator supplied a detailed specification with volatile product/model references. |

## Approaches considered

| Approach | Decision | Reason |
|---|---|---|
| Implement runner immediately in the current checkout | Rejected | Current active scope excludes it; checkout is dirty; no accepted sandbox ADR exists. |
| Treat worktree isolation as a sandbox | Rejected | Worktrees do not restrict credentials, processes, filesystem reach, or network. |
| Land governance first, then isolated phases | Accepted | Preserves previous-gate review and makes each trust-boundary change independently reversible. |
| Replace current sensors/reviews in one change | Rejected | Would let the replacement participate in approving itself and break historical compatibility. |

## Hot-path complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
| Wave 0 bootstrap/doctor | Negligible | Five Markdown files | No executable path changes. |
| Future validation | Linear in manifest/artifact size | Bounded fixtures | Strict semantic validation, not runtime Markdown parsing. |
| Future verification | Dominated by existing `make ci` | Content-addressed logs | Post-commit rerun is intentionally mandatory. |

## Edge cases

| Edge case | Required handling |
|---|---|
| Pass evidence exists for SHA A and one byte changes to SHA B | Refuse merge; regenerate all evidence and review. |
| Required container/sandbox is unavailable | Fail closed; never fall back to host execution. |
| Rename crosses from allowed to protected path | Evaluate old and new names; block without explicit authorization. |
| Reviewer returns prose containing PASS or reviews an older SHA | Structured validator refuses it. |
| Writer creates a fake evidence file in its worktree | Ignore it; trusted recorder writes outside writer authority. |
| Network-off task requests a new domain during execution | Stop; authority cannot expand mid-run. |

## Breakage risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
| New policy approves itself | Invalid governance | Wave 0 uses pre-change gates; parser migration is later | Independent legacy post-review |
| Operational telemetry is mistaken for merge proof | Stale or forged success | Keep sensors unchanged and define a separate evidence artifact | Wave 1 semantic tests |
| Sandbox claim is weaker than reality | Credential/network exposure | Require pinned adapter and refuse unavailable isolation | Wave 2 adversarial tests |
| Historical review compatibility becomes fail-open | Unreviewed merge | Compatibility can display history but never satisfy merge gate | Wave 3 fixtures |

## External reference intake

| Field | Evidence |
|---|---|
| Source identity | Operator-supplied *Agent Harness Hardening Specification*, version 1.0, dated 2026-07-18, read 2026-07-21. URLs listed in its source-reference section were not used as implementation authority in Wave 0. |
| Source type | User-provided implementation blueprint and secondary synthesis. |
| License boundary | License not stated. No text, prompts, scripts, or checklists are vendored as an external dependency; local wording and structure are independently adapted. |
| Harness relevance | Governance, verifier, evidence, runtime control, security, review, and merge policy. |
| Placement | ADR for trust decisions; active plan for sequencing; Canvas for risks and intake. Executable details are deferred to separately reviewed waves. |
| Adaptation | Revision-specific evidence, bounded authority, no model-authored execution, fail-closed verification, and human production boundary. |
| Exclusions | No model IDs, prices, CLI flags, unofficial auth patterns, router, swarm, cron writer, auto-merge, or production deployment is adopted. |
| Verification | Wave 0 doctor, full sensors, diff check, pre-change independent review, and human acceptance. Volatile product claims must be re-verified from primary sources only when a later task depends on them. |

## Decision

Proceed with governance-only Wave 0. Do not add executable runner, schema,
review-parser, workflow, or merge-gate changes until the ADR is accepted.
