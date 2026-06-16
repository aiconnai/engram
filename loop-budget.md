# Loop Budget - Daily Triage

## Metadata

- Pattern: daily-triage
- Rollout level: L1 report-only
- State file: `STATE.md`
- Run log: `loop-run-log.md`
- Owner: Ronaldo
- Last reviewed: 2026-06-16

## Daily Limits

| Metric | Value |
|---|---:|
| Max runs/day | 2 |
| Max tokens/day | 100000 |
| Report-only threshold | 80% |
| Stop threshold | 100% |
| Max sub-agent spawns/run | 0 |
| Max attempts per item | 0 |

## Budget Policy

- Always run report-only in L1.
- If no actionable items exist, update state and exit quickly.
- If tokens reach 80%, skip connector-heavy discovery and write a budget note.
- If tokens reach 100%, append a throttle entry and exit.
- If `loop-pause-all` is present, exit immediately.

## Kill Switches

- `loop-pause-all` in `STATE.md`
- `loop-pause-all` in an owner instruction
- Budget at or above stop threshold
- Any request to write source files while still in L1
- Any attempted action touching the denylist in `LOOP.md`

## Alerts This Period

| Time | Trigger | Action |
|---|---|---|
|  |  |  |
