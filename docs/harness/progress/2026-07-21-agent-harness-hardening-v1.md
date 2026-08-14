# Agent harness hardening v1 — Wave 0 governance plan

## Objective

Authorize a bounded, auditable path from the existing Engram harness to
revision-specific trusted evidence without letting the new harness approve its
own policy changes.

## Scope of this wave

- Add the proposed trust-boundary ADR.
- Record the Review Canvas and external-reference intake.
- Make this plan the active harness task in `SPEC.md` and `progress.md`.
- Preserve the existing review parser, sensors, workflows, scripts, and product
  behavior unchanged.

## Explicitly out of scope

- Task/evidence/review schema implementation.
- Runner, sandbox adapter, scope checker, evidence recorder, or merge gate.
- Workflow or branch-protection changes.
- Model routing, subagent runtime, deployment, auto-merge, or production access.
- Changes to Rust, MCP, storage, SDKs, dependencies, or tests.

## Protected files authorized in Wave 0

- `docs/harness/SPEC.md`
- `docs/harness/progress.md`
- `docs/harness/progress/2026-07-21-agent-harness-hardening-v1.md`
- `docs/harness/canvas/2026-07-21-agent-harness-hardening-v1.md`
- `docs/decisions/2026-07-21-agent-harness-hardening-v1.md`
- Immutable prompt/review artifacts under `docs/harness/reviews/` for this task

No executable harness or CI file is authorized by this wave.

## Sequence

1. Land and human-accept Wave 0 under the previous gates.
2. Start Wave 1 on a new clean branch with strict JSON contracts and validators.
3. Prove validators against adversarial fixtures before adding execution.
4. Add a fake-writer runner behind a required sandbox adapter.
5. Migrate review consumers together; never allow legacy compatibility to
   produce merge-eligible evidence.
6. Add the read-only merge-policy job only after trusted CI artifact flow is
   defined and tested.

## Phase acceptance matrix

| Phase | Required proof | Must remain impossible |
|---|---|---|
| Wave 0 | doctor, full sensors, pre-change independent review, human ADR acceptance | Autonomous execution |
| Wave 1 | malformed/wrong-SHA/path/cap/check-ID fixtures fail closed | Agent-authored trusted evidence |
| Wave 2 | fake writer, sandbox refusal tests, post-commit evidence, scope adversarial tests | Host fallback or free-form shell |
| Wave 3 | exact SHA/policy matching and unavailable reviewer refusal | Marker/prose pass accepted for merge |
| Wave 4 | exact PR-head CI artifact validation and human-risk rules | Direct merge/deploy mutation |

## Rollback

Wave 0 is reverted as one governance revision. Later waves are separately
revertible and must retain the previous accepted gate until the replacement has
independent evidence.

## Required Wave 0 gates

- `bash docs/harness/bin/bootstrap.sh`
- `bash docs/harness/bin/doctor.sh`
- `bash docs/harness/bin/sensors.sh`
- `git diff --check`
- `bash docs/harness/bin/review-gate.sh post agent-harness-hardening-v1`
- Human acceptance of the ADR and merge decision

## Status

Implementation prepared on an isolated clean worktree. ADR remains proposed;
no autonomous runner or executable policy change is authorized yet.
