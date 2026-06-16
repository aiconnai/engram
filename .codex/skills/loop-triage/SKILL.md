---
name: loop-triage
description: Triage recent changes, CI failures, issues, and conversations into concise loop-consumable state updates. Use for L1 report-only daily triage.
user_invocable: true
---

# Loop Triage

Produce a clean, prioritized report that a Codex loop can append to `STATE.md`.

## Inputs

- Current `STATE.md`
- Current `loop-budget.md`
- Current `loop-run-log.md`
- Recent CI/test failures if available
- Recent commits, issues, PRs, or team threads if available
- Project instructions from `AGENTS.md`, skills, or repo docs

## Output Sections

### High Priority

Include only items a reasonable engineer should know about today.

Each item must include:

- One-line description
- Why it matters
- Suggested loop action
- Risk level: low, medium, or high
- Owner or escalation target when known

### Watch List

Include lower-urgency items, ambiguous signals, or items waiting for more evidence.

### Noise / Ignore

Briefly record checked signals that do not need action.

### State Updates

Record facts the next run should remember, such as closed PRs, resolved failures, stale items pruned, or owner decisions.

## Rules

- Be concise and structured.
- Do not invent work.
- Do not propose architecture changes during triage.
- Do not edit source files.
- Do not write to connectors in L1.
- Put ambiguous items in Watch or Escalate, not High Priority.
- Escalate anything involving auth, billing, payments, secrets, migrations, production infra, or unclear ownership.
