# Loop Run Log - Daily Triage

Append one entry per run. Retain the last 30 days or last 200 entries.

## Entry Schema

```json
{
  "run_id": "2026-06-16T12:00:00Z",
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

## Recent Runs

| run_id | level | items | actions | escalations | tokens | outcome |
|---|---|---:|---:|---:|---:|---|
| 2026-06-16T22:05:37Z | L1 | 3 | 0 | 0 | 9500 | report-only |
