# O4 report - standing checks read-only com ownership (Lane P, worktree engram-improvement-lane-p)

Status: DONE_WITH_CONCERNS (see concerns)

## Commits (local, branch claude/improvement-lane-p)
- a9809d1 feat(harness): add read-only alert-only standing checks with ownership
- 1f5879d chore(harness): wire standing-checks tests into lane and doctor
- c8581cc docs(harness): record O4 standing checks and refresh last commit

## Implemented
- docs/harness/schemas/goal-v1.schema.json, docs/harness/goals/registry.json (1 goal: pr-title-policy-daily, owner harness-maintainers, daily), docs/harness/goals/README.md (runbook named by alerts).
- docs/harness/bin/run-standing-checks.py: validate / run --mode manual|dispatched|scheduled. Strict validation (H2 parse_json_strict + PureSchemaValidator + jsonschema cross-check), H3 registry cross-check (check exists, timeout <= min(check timeout, sandbox max), policy equal), per-goal flock under <git-common-dir>/engram-standing-checks, export of run SHA, run only via sandbox-adapter.run_isolated (read-only caps, no tcb_files, no writer), argv-equals-registry check, outcome.json/log existence + hash verification, before/after checkout snapshot (mutation = fail, no git object writes), receipt (SHA, tree, policy, toolchain, input/TCB hashes, outcome/log hashes), LOCAL alert for any non-pass (owner + runbook, delivery.sent=false). Kill switch ENGRAM_STANDING_CHECKS_DISABLED=1. Module imports no subprocess/network (AST-tested).
- .github/workflows/standing-checks.yml: schedule + workflow_dispatch only, contents: read, no secrets, SHA-pinned actions (same pins as repo), dispatch input via env only, artifact upload excluding exported checkouts. scripts/check-workflow-supply-chain.py: PASS.
- Wiring (done last, after re-reading): run-offline-lane.sh component standing_checks floor 52 (components 11->12), test_offline_lane.py (+1 test, counts), doctor.sh (files, lane grep, standing_checks_workflow:read_only).
- Lane P log entry (PT-BR) and progress.md Last commit refreshed.

## TDD evidence
- Honest note: implementation was written before the tests (needed to read H3/H4 first); RED was proven with a permissive mutant of the runner in the scratchpad (classify always PASS, lock disabled, unknown-check/timeout checks off), loaded via ENGRAM_STANDING_RUNNER_UNDER_TEST:
  `Ran 52 tests`, `FAILED (failures=19, errors=2)`.
- GREEN: `python3 -m unittest discover -s docs/harness/tests -p test_standing_checks.py` -> `Ran 52 tests ... OK` (~34s).

## Other verification
- `bash docs/harness/bin/run-offline-lane.sh` -> `OFFLINE_LANE: PASS components=12 checks=523` (standing_checks 52, floor 52).
- `test_offline_lane.py` OK (17 at my commit; 18 now after H5 additions); `doctor.sh` OK (only the pre-existing review-artifact WARN); `check-live-state.sh` PASS; `check-workflow-supply-chain.py` PASS; `run-standing-checks.py validate` -> `GOALS: OK goals=1 owners=1`.

## Deviations / concerns
- goal has an extra required field `runbook` (brief lists owner but alerts must name a runbook); runbook points to new docs/harness/goals/README.md.
- Q3 lanes (fuzz/miri/mutants) NOT registered as goals: they need a Rust toolchain; the H3 sandbox image is python-only with no network, and the H3 registry has only pr_title_policy. Goals can only reference H3 registry check IDs, so they cannot be added without an H3 registry/image change. Only pr_title_policy goal exists.
- Owner handle `harness-maintainers` is a placeholder handle; owner should set the real one.
- Workflow runs `docker pull` of the digest-pinned sandbox image (adapter itself never pulls). Not executed in real GitHub.
- GATES.md not touched (H5 section being edited concurrently); O4 documentation lives in goals/README.md and the lane log.
- progress.md Last commit is 1f5879d (H5's bcbb568 landed in between).
- Test suite takes ~34s (git repo + stub docker per test).

## NOT RUN
Real scheduled/dispatched GitHub run, real Docker smoke of a goal, real alert delivery channel (needs human authorization), sensors.sh, independent review (controller).
