---
name: loop-budget
description: Check loop budget, pause flags, and run-log spend before and after a loop run. Enforces early exit when over budget or when there is no actionable work.
user_invocable: true
---

# Loop Budget Guard

Run at the start and end of every loop run.

## Start Of Run

1. Read `loop-budget.md`.
2. Read `loop-run-log.md` entries for the last 24 hours.
3. Check `STATE.md` for `loop-pause-all`.
4. Estimate tokens already spent today for this pattern.
5. If spend is 80% to 99%, run report-only with shallow discovery.
6. If spend is 100% or higher, append a throttle entry and exit.
7. If no actionable items exist after state read, exit quickly with a no-op log entry.

## End Of Run

Append exactly one run entry to `loop-run-log.md`.

```json
{
  "run_id": "<ISO8601>",
  "pattern": "daily-triage",
  "rollout_level": "L1",
  "duration_s": 0,
  "items_found": 0,
  "actions_taken": 0,
  "escalations": 0,
  "tokens_estimate": 0,
  "attempts_consumed": 0,
  "verifier_verdict": "none",
  "outcome": "no-op | report-only | escalated | throttled"
}
```

## Rules

- Never exceed `Max sub-agent spawns/run`.
- Never exceed `Max attempts per item`.
- In L1, do not spawn sub-agents.
- In L1, do not edit source files.
- On self-throttle, record the reason in `STATE.md` and `loop-run-log.md`.
