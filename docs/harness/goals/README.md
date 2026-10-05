# Standing checks (task O4) - runbook

Read-only standing checks with an accountable owner. **Alert-only**: a failure produces a local report; nothing is
remediated, committed, opened as a PR or messaged by the scheduler, and the scheduler never starts a writer.

- Goals: `docs/harness/goals/registry.json` (schema `docs/harness/schemas/goal-v1.schema.json`). A goal only selects an
  approved check by ID from `docs/harness/checks/registry.json` (H3); it never carries a command, argv or shell string.
- Runner: `python3 docs/harness/bin/run-standing-checks.py run --mode manual|dispatched|scheduled ...`
  (`validate` checks the registry offline). Runs go through the H3 sandbox adapter (docker, no network, no credentials);
  Docker missing is `unavailable`, never a pass.
- Scheduler: `.github/workflows/standing-checks.yml` (`schedule` + `workflow_dispatch`, `contents: read`, no secrets).
  It uploads the receipts and alerts as an artifact. Rollback: disable the workflow, or set
  `ENGRAM_STANDING_CHECKS_DISABLED=1`; receipts are preserved and no baseline changes.

## When an alert names you as owner

1. Open the alert JSON (`alerts/<time>-<goal>-<run>.json`) and the `receipt_path` it points to: run SHA, policy,
   toolchain, input hashes, outcome and log hashes.
2. Reproduce locally with `--mode manual --goal <id>` against the same SHA. Statuses: `fail` (check exited non-zero),
   `timeout`, `error` (artifact missing/inconsistent, checkout mutated), `unavailable` (Docker or image missing),
   `refused` (policy, registry, lock held by a concurrent run of the same goal).
3. Fix through the normal reviewed change path. The alert `delivery.sent` is always `false`: handing the failure to
   you through a channel needs a channel and a human authorization of its own.
