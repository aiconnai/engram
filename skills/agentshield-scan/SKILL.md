---
name: agentshield-scan
description: Weekly or manual security triage loop that executes AgentShield with a hard iteration cap and records the result in a repository state file.
metadata:
  short-description: Automated AgentShield security scanning loop
---

# AgentShield Security Scan Skill Instructions

## Objective
Run a bounded security scan on the repository using AgentShield, enforce a
high-severity gate, and record outcomes so the loop can resume across sessions.
This loop is for static security triage only; it does not remediate findings by
itself.

## Always Do
- Check that the `agentshield` command is available on the path before running scans.
- Run `bash scripts/run-agentshield-loop.sh` so the scan uses the repository's bounded wrapper.
- Keep `LOOP_MAX_ITERATIONS` between 1 and 5; the default is 1.
- Run `agentshield scan . --ignore-tests --fail-on high --explain` through the wrapper's gate path.
- Use `.agentshield-baseline.json` only after explicit review; create it with `LOOP_WRITE_BASELINE=1`.
- Append a structured entry to `docs/loops/agentshield-scan/STATE.md` after every iteration.

## Never Do
- Never auto-fix, auto-commit, or auto-push scanner findings from this loop.
- Never disable the scanner or bypass high-severity findings without a documented safety exception.
- Never commit raw API credentials, private keys, or tokens to the repository.
- Never expose API secrets or tokens in execution logs.
- Never bypass the gate validation on failure.
- Never mount production credentials or run the loop with broader privileges than repository read access.

## State Handoff
- Read and update the local `STATE.md` file located at `docs/loops/agentshield-scan/STATE.md` after every iteration.
- Record whether the run used a baseline, the iteration cap, and any non-zero exit code.
