# ADR: Agent harness hardening v1

- **Status:** Accepted — 2026-10-05, by the decision owner (see "Acceptance record")
- **Date:** 2026-07-21 (proposed); 2026-10-05 (accepted)
- **Decision owner:** Ronaldo
- **Review basis:** The pre-change Engram harness gates and review contract
- **Status history:** Proposed on 2026-07-21 (Wave 0 governance proposal,
  commit `6064847`) → Accepted on 2026-10-05. The text of the Decision, Phased
  rollout, Failure and rollback, and Consequences sections is unchanged by the
  acceptance.

## Context

The current harness provides deterministic Rust/CI sensors, live-state checks,
review artifacts, and operational history. It does not yet provide a trusted
task runner whose evidence is bound to an exact candidate commit. In
particular, operational sensor records and agent-writable memory are useful
telemetry, but are not merge authorization.

The operator supplied the *Agent Harness Hardening Specification* v1.0 as an
implementation blueprint. Engram will adapt its trust-boundary principles
without treating that document, its example model names, or its example
commands as authoritative over local invariants.

## Decision

Engram will add the hardening capability in separately reviewed phases. This
ADR authorizes governance design only. It does **not** authorize autonomous
execution, production access, deployment, merge, or a host-only fallback.

### Trust boundaries

| Component | Code writes | Trusted evidence | Merge authority |
|---|---:|---:|---:|
| Writer adapter | Task-scoped worktree only | No | No |
| Reviewer | No | No | No |
| Trusted runner | Controls isolated run | Local candidate evidence | No |
| CI | No discretionary edits | CI evidence | Policy input only |
| Merge gate | No | Reads trusted evidence | No direct mutation |
| Human owner | Through a new revision | Approval record | Yes |

### Trusted computing base

The TCB is limited to the manifest validator, approved-check registry, sandbox
adapter, runner, scope checker, evidence recorder, structured-review validator,
merge-policy evaluator, and protected CI configuration. The writer must not be
able to modify the active copies of these components.

### Isolation and target contract

- A Git worktree is revision isolation, **not** a security sandbox.
- Writer execution requires a reviewed sandbox adapter. The first supported
  adapter should be a pinned container image with network disabled, no
  credential mounts, a read-only manifest/policy mount, only the task worktree
  writable, dropped capabilities, `no-new-privileges`, and process/resource
  caps.
- If required isolation is unavailable, execution fails closed. There is no
  silent host-execution fallback.
- The runner executes only fixed argument vectors selected by approved check
  IDs. It never uses `bash -c`, `sh -c`, `eval`, or model-authored predicates.
- Initial runner tests use a fake writer. Enabling a real coding-agent adapter
  requires a later approved task and evidence that the sandbox contract holds.

### Credentials and network

- Network is disabled by default. Allowlisted egress requires explicit task and
  policy authorization and is recorded in evidence.
- Ambient developer credentials, `HOME`, SSH agent sockets, Git askpass,
  provider tokens, cloud credentials, cookies, and production secrets are not
  inherited by writer processes.
- Any unavoidable credential is short-lived, task-scoped, brokered explicitly,
  and absent from prompts and logs.

### Evidence and review

- `.sensors-last` and `.sensors-log` remain operational telemetry.
- Trusted evidence is a separate immutable or content-addressed bundle bound to
  base SHA, candidate SHA, tree hash, policy version, exact argv, exit status,
  time bounds, toolchain, environment, log hashes, and recorder identity.
- Evidence is created outside the writer-writable tree and only after a
  committed-tree verification run.
- Every byte change creates a new candidate SHA and invalidates earlier test
  and review evidence.
- New review artifacts will migrate to strict SHA-bound JSON. Historical
  marker-based reviews remain historical records but will never satisfy the
  future merge gate.
- The marker-based gate remains the approving gate for this ADR and its Wave 0
  governance PR; the harness must not approve its own replacement.

### Merge and production authority

The merge evaluator is read-only and deterministic. High-risk, critical,
security, authentication, migration, infrastructure, CI/policy, test-weakening,
and production-boundary changes require human approval. Deployment and
production mutation remain outside this decision.

## Phased rollout

1. **Wave 0 — governance:** this ADR, active plan, Canvas, reference intake,
   and synchronized live-state metadata.
2. **Wave 1 — contracts:** strict JSON task/evidence/review schemas,
   dependency-free semantic validators, and adversarial fixtures.
3. **Wave 2 — runner MVP:** fake-writer execution through the approved sandbox,
   scope enforcement, post-commit verification, and external evidence bundle.
4. **Wave 3 — structured review:** compatibility migration followed by
   fail-closed SHA-bound review.
5. **Wave 4 — merge-policy CI:** read-only merge decision over trusted CI
   artifacts and exact PR head.

Each wave starts from a clean branch after the prior wave is accepted. A wave
may not inherit evidence or approval from an older candidate SHA.

## Failure and rollback

- Invalid input, unavailable isolation, timeout, unknown state, missing log,
  missing reviewer, scope violation, or evidence mismatch is a failure.
- Rollback for governance changes is a normal revert to the previous policy.
- Rollback for later implementation waves disables the new policy version and
  restores the last accepted pre-change gate; it never converts incomplete new
  evidence into a pass.

## Consequences

The rollout is slower than adding a shell loop, but it minimizes false-green
outcomes and prevents a writer from manufacturing completion. Container
availability becomes an explicit prerequisite rather than an implicit host
fallback.

## Acceptance

Original acceptance condition (as proposed on 2026-07-21): this ADR becomes
**Accepted** only after the owner explicitly approves it and the dedicated Wave 0
PR passes bootstrap, doctor, full sensors, diff checks, and independent review
using the pre-change gates.

### Acceptance record (2026-10-05)

| Field | Value |
|---|---|
| Previous status | Proposed (2026-07-21) |
| New status | Accepted |
| Date | 2026-10-05 |
| Decision owner | Ronaldo |
| Source | The owner's chat answer of 2026-10-05, in which he authorized all waves of the comprehensive improvement plan including Wave 4 (his words in chat, "Tudo, inclusive Onda 4", are **not** text of the plan). The plan records the resulting authorization note (`docs/harness/plans/2026-10-02-engram-comprehensive-improvement-plan.md`, authorization note at the top: execution of waves 0-4 authorized, "incluindo aceite do ADR"). Recorded by task E0; attribution corrected by task H6 (the quoted phrase had been attributed to the plan note). |
| Scope of acceptance | Phased rollout Waves 0-4 as written above. Wave 4 runner/sandbox work uses a **fake writer only, offline**. |
| Not granted by this acceptance | Real coding-agent adapter; autonomous execution against Engram; host-execution fallback; production or cloud access; deployment; auto-merge; branch-protection changes; push, PR, or package publication. Human merge authority is unchanged. |
| Pre-acceptance gate evidence | The Wave 0 governance content (commit `6064847`) and its marker-based independent review `docs/harness/reviews/2026-07-22-agent-harness-hardening-v1-v3-post.md` (PASS) predate this acceptance and are already in `main`'s history. E0 baseline gate evidence (bootstrap, doctor, sensors) is recorded in `docs/harness/audits/2026-10-02-improvement-baseline.md`. |
| Deviation from the text above | The original acceptance sentence assumed a dedicated Wave 0 PR. The owner accepted directly in chat; no PR was opened, and this program delivers local commits only (no push). The owner's explicit approval is the acceptance; gate results are recorded as evidence, not as a substitute for it. |

Acceptance authorizes the **phased work** described in this ADR. It does not
retroactively approve code that landed earlier (for example the Wave 1 schemas
and validator introduced in commit `2313d8a`): those artifacts stay
unactivated and outside the trusted computing base until the corresponding
wave task (H2 onward) hardens and the owner approves them, as inventoried in
the E0 baseline audit.

Rollback: revert the commit that records this acceptance; the status returns to
Proposed and no executable work is authorized.
