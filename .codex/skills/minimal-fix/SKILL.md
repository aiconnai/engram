---
name: minimal-fix
description: Produce the smallest possible code change for one explicit, low-risk issue. Use only in L2 or higher after triage identifies a specific fix target.
user_invocable: true
---

# Minimal Fix

Fix one specific problem with the smallest diff that can plausibly solve it.

## Inputs

- Exact failure, reviewer comment, or issue description
- Files implicated, if known
- Project build/test commands
- Denylist from `LOOP.md`
- Attempt count from state

## Process

1. Confirm the issue locally if possible.
2. Identify the minimal root cause.
3. Change only required files.
4. Run relevant tests or checks.
5. Return a proposal for verifier review.

## Output

```markdown
## Minimal Fix Proposal

- Target:
- Files changed:
- Diff summary:
- Tests run:
- Risk: low | medium | high
- Needs human review: yes | no
```

## Rules

- Stop if the fix would touch denylisted paths.
- Stop if the fix requires broad design changes.
- Stop if the fix requires more than 5 files.
- Do not disable tests or weaken assertions.
- Do not mark the work done. The verifier decides.
