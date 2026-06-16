# LOOP - Daily Triage

## Scope

- Pattern: daily-triage
- Rollout level: L1 report-only
- Cadence: daily at 08:00 local
- Environment: Codex Automation or local Codex thread
- Owner: Ronaldo
- State file: `STATE.md`
- Run log: `loop-run-log.md`
- Budget file: `loop-budget.md`
- Non-goals: source edits, auto-remediation, dependency upgrades, production config changes, connector writes

## Objective

Produce a concise daily view of engineering signals that need attention, without modifying source files.

## Stop Conditions

- Success: `STATE.md` and `loop-run-log.md` are updated with current triage.
- Empty run: no actionable items after triage.
- Budget stop: `loop-budget.md` requires report-only or exit.
- Risk stop: finding touches denied paths or requires a medium/high-risk decision.
- Kill switch: `loop-pause-all` appears in state, budget, ticket label, or owner instruction.

## Run Steps

1. Run `$loop-budget`.
2. Run `$loop-triage`.
3. Read `STATE.md`, `loop-budget.md`, `loop-run-log.md`, and project instructions.
4. Prune resolved, closed, merged, or stale items.
5. Update `STATE.md` with High Priority, Watch List, Noise / Ignore, and State Updates.
6. Append one run entry to `loop-run-log.md`.
7. Exit without source edits.

## Safety Rules

- No source file edits in L1.
- No sub-agent spawns in L1.
- No connector writes in L1.
- Ambiguous findings go to Watch or Escalate, not High Priority.
- Denylist paths are never acted on by this loop.
- Budget >=80% keeps the run report-only with shallow discovery.
- Budget >=100% exits after recording a throttle entry.

## Denylist

- `.env`
- `.env.*`
- `**/secrets/**`
- `**/credentials/**`
- `**/*_key*`
- `**/*_secret*`
- `.terraform/**`
- `k8s/production/**`
- `**/migrations/**`
- `auth/**`
- `payments/**`
- `billing/**`

## Evidence Required

- Last run timestamp
- Item counts by section
- No-op reason when no actionable items exist
- Escalation target for human decisions
- Budget state if throttled

## Success Criteria

- `STATE.md` is current and scannable.
- `loop-run-log.md` has exactly one new entry for the run.
- No source files are changed.
- Human-required items are clearly marked.
