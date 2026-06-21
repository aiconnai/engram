---
name: loop-engineering
description: Use when designing, implementing, auditing, hardening, or operating agent loops in this repository, including Claude Code/Grok Build/Codex style `/loop` workflows, explicit `/goal` runs, Git worktree orchestration, MCP and memory automation, sub-agent execution, STATE.md handoffs, cost controls, maker/checker verification, and the Engram security gate (full `sensors.sh` + AgentShield scan + `just ci`). Loops default to L1 report-only; autonomous execution against Engram requires a reviewed ADR per the harness security contract.
---

# Loop Engineering

Use this skill when work is repeatable, observable, and can be resumed safely
across runs. Treat loops as small control systems: they discover work, hand it
to agents, verify results, persist state, and decide whether to continue,
revise, stop, or escalate.

In Engram the canonical worked examples are the `agentshield-scan` loop
(`docs/loops/agentshield-scan/`, a bounded weekly security scan) and the
`engram-council` review loop. Use them as references for cadence, state shape,
and hard-stop discipline.

Per `docs/harness/security/anthropic-reference-harness.md`, harness work — loops
included — is static/read-only by default. A loop starts at L1 report-only;
autonomous execution against Engram (write-path MCP, storage, sync, attestation,
release, or any network/code-execution step) requires a reviewed ADR, a sandbox
boundary, and an explicit target/egress contract first. This skill never implies
autonomy is allowed by default.

## Use This Only If

Loop only when all four conditions are true:

- **Repeatable objective:** the work can be expressed as iterations toward a
  stable outcome.
- **Observable oracle:** each iteration has evidence from commands, tests,
  artifacts, reviews, scans, or a manual QA surface.
- **Bounded action space:** allowed tools, files, MCP operations, costs, and
  side effects are explicit enough to stop unsafe drift.
- **Durable state:** progress can be persisted in a `STATE.md`-style file
  without relying on hidden chat context.

Do not start a loop if a required secret is missing, the next step is
destructive or irreversible, the acceptance oracle is unclear, or the loop would
write to live external systems (MCP write-path, storage, sync, attestation,
release) without a read-only probe and an idempotent write plan.

## What This Skill Does Not Do

- It does not justify uncertain actions.
- It does not perform high-risk refactors on first contact.
- It does not bypass human gates on risky paths.
- It does not make a loop safe just because an agent can run it repeatedly.

## Quick Bootstrap

A new loop starts at L1 report-only. Scaffold it by hand from the
`agentshield-scan` loop (`docs/loops/agentshield-scan/`): copy its `STATE.md`
shape, set an explicit objective, scope, non-goals, stop condition, and a hard
iteration cap, and keep the first runs read-only.

Do not introduce loop scaffolding by executing unpinned third-party tooling
(for example `npx <package>` runners) from inside the repository. Per
`docs/harness/security/anthropic-reference-harness.md`, harness work is
static/read-only by default, and any autonomous or network/code-execution step
against Engram requires an accepted ADR, a sandbox boundary, and an explicit
target/egress contract first. Treat external loop-scaffolding CLIs as reference
material to read, not commands to run by default.

Upgrade beyond report-only only after at least one stable week of accurate
reports, low noise, clear cost behavior, and no untriaged safety findings.

## Required Scaffolding

Every production loop needs:

- `STATE.md` or a pattern-specific state file (Engram keeps these under
  `docs/loops/<loop-name>/`, as `agentshield-scan` does).
- `loop-budget.md` with daily caps, attempt caps, and a kill switch.
- `loop-run-log.md` as append-only run history.
- `LOOP.md` with cadence, budgets, ownership, escalation rules, and allowed
  actions.
- A verifier role separated from the implementer role; no self-approval.
- A path/API denylist and an explicit auto-merge allowlist.
- The Engram security gate (AgentShield scan + `just ci`) and policy for the
  repository surfaces the loop can touch.

Use `docs/loops/<loop-name>/` for repository-wide loops that should survive
merge. Use `.worktrees/<loop-name>/` only for disposable, worktree-local state.

## 14-Step Execution Loop

1. **Name the loop:** write the objective, owner, scope, and non-goals.
2. **Apply the 4-condition test:** record why the task is loopable or stop.
3. **Define the success oracle:** list the exact commands, artifacts, manual QA,
   and review criteria that prove completion.
4. **Map the action boundary:** identify files, directories, MCP operations,
   external services, and permissions the loop may touch.
5. **Choose the orchestration primitive:** select `/loop`, `/goal`, worktrees,
   skills, MCP tools, sub-agents, or a combination.
6. **Create isolation:** use a Git worktree or branch when parallel work,
   risky edits, or long-running changes could collide with the main checkout.
7. **Initialize `STATE.md`:** create or update the loop state before starting
   implementation.
8. **Capture baseline signal:** record status, existing failures, prior run
   timestamp, and active branch/worktree before making changes.
9. **Plan one smallest next action:** define the next observable change and its
   expected evidence.
10. **Run the budget guard:** check `loop-budget.md`, attempt count, cost cap,
    pause flags, and kill switch before write actions.
11. **Execute at the allowed level:** L1 is report-only, L2 is minimal-fix
    proposals in an isolated worktree plus verifier, and L3 requires explicit
    mature safety gates.
12. **Record evidence:** write command evidence, files changed, outcomes, and
    artifacts to `STATE.md` and `loop-run-log.md`.
13. **Run mandatory verification:** execute relevant tests/lint/scans, review
    scope diff, and check the denylist.
14. **Decide continue, revise, or stop:** compare evidence to the oracle and the
    loop readiness rubric; close with final gate results or escalate clearly.

## Execution Levels

- **L1 report-only:** triage, classify, summarize, and recommend. No writes
  outside state/log files unless the user explicitly asks.
- **L2 assisted fixes:** create minimal diffs in an isolated branch or worktree,
  then require an independent verifier before merge, PR, or external write.
- **L3 mature automation:** allow narrow automated actions only after a reviewed
  ADR (required by the harness security contract for any autonomous execution
  against Engram), stable L1 and L2 metrics, explicit allowlists, budget caps,
  denylist enforcement, the full sensor gate, and human-approved escalation
  paths are all in place.

## Building Blocks

Use `/loop` for bounded iterative work where the user expects repeated
execution until a named stop condition is met. State the stop condition before
the first iteration and re-check it after every gate.

Use `/goal` only when the user explicitly asks for a goal-style long-running
objective. Keep the objective concrete, do not mark it complete until the
matching surface has been exercised, and do not use `/goal` as a substitute for
clear acceptance criteria.

Treat any external loop-scaffolding CLI as reference material, not a default
command to execute (see Quick Bootstrap). If a reviewed ADR ever authorizes
running one, review every generated file before trusting it, then adapt its
budgets, cadence, and denylist to the Engram harness.

Use Git worktrees when two agents may touch different ownership boundaries or
when a loop needs isolation from the user's active checkout:

```bash
git worktree add .worktrees/<loop-name> -b <branch-name>
```

Do not create nested worktrees. Check the current worktree first with:

```bash
git rev-parse --git-dir
git rev-parse --git-common-dir
git branch --show-current
```

Use repository skills before generic reasoning. Load only the skill files needed
for the active task, then follow the repository-specific rules for MCP tools,
storage, search, the SDKs (Python/TypeScript), Rust, and release work.

Use connectors for systems of record such as GitHub, Linear, Slack, or Vercel
only after identifying whether the operation is read-only or write-path. For
writes, probe the target first, make the operation idempotent, and never print
tokens or full environment values.

Use sub-agents for parallel exploration, independent review, or isolated
implementation on disjoint files. Give each sub-agent a worktree, scope,
allowed files, expected evidence, and return format. Do not use sub-agents to
hide missing acceptance criteria or to bypass connector permissions.

## Maker / Checker Split

The implementer produces the smallest diff that satisfies the current
iteration. The verifier independently checks scope, tests, security, and intent.

Never combine implementer and verifier in the same loop iteration. If a separate
agent is unavailable, require a human verifier or stop at L1/L2 proposal status.
Default verifier stance is **reject until proven safe**. This mirrors the
harness Single-Process Judgment rule: the agent that wrote a change is not the
final judge of it — route post-review to a different CLI/model via
`docs/harness/bin/review-gate.sh`.

The verifier must check:

- The diff stays inside the declared file and API boundary.
- The change matches the user's intent and the loop oracle.
- Tests, lint, AgentShield scan, and manual QA evidence are relevant.
- Denylisted paths and external systems were not touched.
- Connector and MCP writes were preceded by read-only probes and are idempotent.

## Safety And Cost Controls

Abort or pause the loop when:

- The attempt cap is reached; default to 3 attempts per item.
- Budget is at or above 100 percent.
- `loop-pause-all` or the configured kill switch is present.
- No actionable items are found.
- Verification is inconclusive after the allowed attempts.
- Uncertainty is higher than confidence.
- Risk is medium or high without explicit human approval.

Never auto-merge without an explicit allowlist. Enforce denylist rules for
secrets, auth, migrations, infrastructure, production data, release publishing,
and any repo-specific high-risk path.

In Engram, treat at least these paths/actions as human-gated unless the loop
charter explicitly allows them:

- `.env`, credential files, tokens, signing keys, and release secrets.
- `.github/workflows/release.yml`, `.github/workflows/ci.yml`,
  `.github/workflows/nightly.yml`, the `release/` directory, crates.io / npm /
  PyPI publishing, installers, and release artifacts.
- Attestation and snapshot code, sync write-path, and MCP server write tools.
- Storage migrations, the `proto/` protocol surface, auth flows, and production
  connector or MCP writes.

## Structured State

Keep loop state in a version-controlled `STATE.md` near the work:

- Repository-wide or release loops: `docs/loops/<loop-name>/STATE.md`
- Feature worktrees: `.worktrees/<loop-name>/STATE.md` when the state is local
  to that worktree, or `docs/loops/<loop-name>/STATE.md` when it should survive
  merge.
- Short repository skills or docs loops: `skills/<skill-name>/STATE.md` only if
  the state is part of the skill authoring record; otherwise use `docs/loops`.

Use this minimal structure:

```markdown
# <Loop Name> State

Objective:
Scope:
Non-goals:
Stop condition:
Level: L1 | L2 | L3
Owner:
Last run:

## Current Iteration
- Number:
- Planned change:
- Expected evidence:

## High Priority

## Watch

## Noise

## Attempts
| Item | Attempts | Current status |
|---|---:|---|

## Evidence
| Time | Command or action | Result | Notes |
|---|---|---|---|

## Decisions
| Time | Decision | Reason |
|---|---|---|

## Risks
| Risk | Mitigation | Status |
|---|---|---|

## Human Escalations
| Time | Issue | Outcome |
|---|---|---|

## Run Log Summary
- Append one line per run to `loop-run-log.md`.

## Next Step
```

Update `STATE.md` before handoff, before context-heavy pauses, and after every
iteration gate. Record facts, not private chain-of-thought. Do not store secrets,
access tokens, full environment dumps, or unredacted customer data.

`loop-budget.md` should include daily spend cap, per-run cap, attempt cap,
pause flag name, kill switch path, and the human escalation contact or channel.

`loop-run-log.md` should be append-only:

```markdown
| Time | Loop | Level | Result | Cost/Budget | Evidence | Next |
|---|---|---|---|---|---|---|
```

## Security Gate

The deterministic gate for loop work is the full `bash docs/harness/bin/sensors.sh`
run, which wraps `just ci` (fmt + clippy + tests + docs + MCP reference) and the
harness doctor checks. Run it — plus an AgentShield scan — after changing agent
code, skills, MCP servers, connector scripts, tool schemas, runtime policies, CI
workflows, or anything that affects execution, data, or network surfaces.

The `agentshield-scan` loop (`docs/loops/agentshield-scan/`) is the worked
example: it runs a bounded scan via `scripts/run-agentshield-loop.sh`, caps
iterations with `LOOP_MAX_ITERATIONS` (default `1`, hard cap `5`), gates on
`--fail-on high`, and only accepts pre-existing findings through a reviewed
`.agentshield-baseline.json`.

Recommended local gate:

```bash
agentshield scan . --ignore-tests --fail-on high --explain
just ci
```

Recommended machine-readable artifacts:

```bash
mkdir -p target/agentshield
agentshield scan . --ignore-tests --format json --output target/agentshield/loop-scan.json
agentshield scan . --ignore-tests --format sarif --output target/agentshield/loop-scan.sarif
```

For an established repository with accepted findings, create a baseline once,
review it, commit it, and then gate only new findings:

```bash
agentshield scan . --write-baseline .agentshield-baseline.json
agentshield scan . --ignore-tests --baseline .agentshield-baseline.json --fail-on high --explain
```

Do not rewrite `.agentshield-baseline.json` automatically inside a loop. Update
it only after intentional review of the findings being accepted — the
`agentshield-scan` STATE.md records this rule explicitly.

The canonical deterministic gate is the no-argument full sensor run. It is the
hard gate for any commit, handoff, or completion claim:

```bash
bash docs/harness/bin/sensors.sh        # full canonical gate (== just ci + harness checks)
```

`sensors.sh quick` (fmt + check + doctor) and the other optional lanes
(`docs`, `mcp`, `baseline`) are development preflights only. Per
`docs/harness/GATES.md`, the optional lanes do not replace the full gate for
merge, handoff, or completion — never substitute `quick` for the full run when
closing loop work.

If a test, parser, detector, policy, baseline, scan, or sensor check fails,
rerun the specific command directly before making code changes based on the
failure. Do not rely on filtered output for security decisions.

## Anti-Patterns

- Endless retry loops.
- Narrative-only triage with no actionable evidence.
- Shared unstructured state across multiple loops.
- Auto-fix beyond L1 before evidence and metrics are stable.
- Silent baseline rewrites that hide new findings.
- Self-approval by the implementer.
- Connector or MCP writes without read-only probes and idempotency checks.
- Manual QA claims without exercising the matching surface.

## Acceptance Checklist

Before calling a loop complete, verify:

- The 4-condition test still holds or the loop has been stopped deliberately.
- Evidence and verification match the oracle.
- `STATE.md` and `loop-run-log.md` contain the latest objective, evidence,
  attempts, decisions, risks, human escalations, and next step or closeout.
- Budget, L1/L2/L3 safety rules, pause flags, and escalation rules were honored.
- All connector and MCP writes were preceded by read-only probes and are
  idempotent.
- Sub-agent outputs were reviewed in the coordinating worktree.
- Maker/checker roles were separated, or the loop stopped at proposal status.
- The relevant build, lint, test, and manual QA surfaces have been exercised.
- The full canonical gate `bash docs/harness/bin/sensors.sh` has passed for the
  commit/handoff/completion (not just the `quick` preflight lane).
- The AgentShield scan and `just ci` have passed, or every remaining finding is
  explicitly triaged with a reviewed baseline or suppression.
- Human-required items are escalated clearly.
- No generated JSON, SARIF, HTML, console report, baseline, egress policy, or
  attestation was altered by output filtering.
