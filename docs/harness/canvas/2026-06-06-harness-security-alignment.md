# Review Canvas — Harness Security Alignment

| Field | Value |
|-------|-------|
| Task | `harness-security-alignment` |
| Date | `2026-06-06` |
| Scope | Docs, policy, tuning files, and harness script enforcement |
| Out of scope | `src/`, autonomous execution pipeline, canary fixture, C/C++/ASAN import |

## Problem

The harness docs referenced an Anthropic reference-harness adaptation and Claude tuning files, but those files were not present and the scripts did not enforce the security contract. That made the policy easy to drift.

## Approaches Considered

- Documentation only: lower risk, but leaves the contract unenforced and keeps drift possible.
- Script enforcement only: catches missing files, but does not give reviewers a canonical explanation of the boundary.
- Combined docs plus minimal enforcement: creates the missing source of truth and makes `doctor.sh` fail closed when wiring drifts.

Chosen approach: combined docs plus minimal enforcement.

## Hot-Path Complexity

The default harness loop remains unchanged. `bootstrap.sh` prints one extra read-next line. `sensors.sh` prints the security contract and tuning files before running the same deterministic gates. `doctor.sh` adds file, anchor, and cross-reference checks only.

## Edge Cases

- Missing security note or tuning files after docs reference them: `doctor.sh` should fail closed.
- Tuning files try to suppress broad categories or replace policy text: review-gate should flag this through CODE_REVIEW_POLICY.
- Future prompt or doc implies autonomous execution without ADR/sandbox: review-gate should flag boundary drift.

## Breakage-Risk Table

| Risk | Impact | Mitigation |
|------|--------|------------|
| Bootstrap output exceeds limit | Agents lose concise session start | Added only one read-next line |
| Doctor regex too brittle | False harness failure | Anchors use stable ASCII tokens |
| Sensors becomes a new security scanner | Gate gets slower/flaky | Sensors only surfaces contract; no new scan execution |
| Review prompt becomes noisy | Lower reviewer signal | Added only boundary checks tied to changed scope |

## Reviewer Checklist

- Confirm no `src/` changes.
- Confirm no autonomous execution pipeline was added.
- Confirm `.claude/scan-extras.txt` and `.claude/fp-rules.txt` are tuning files, not policy replacements.
- Confirm `doctor.sh` fails closed on missing contract/tuning files.
