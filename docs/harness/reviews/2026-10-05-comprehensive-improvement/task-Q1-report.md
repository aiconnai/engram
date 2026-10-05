# Task Q1 — stage Q1a report (CI/local quality-budget contract)

Worktree: `engram-improvement-lane-p`, branch `claude/improvement-lane-p`. Q1b is NOT done (pending Q7).

## Commits
- `1addb94` fix(ci): restore --criterion in historical baseline integrity lane
- `96dfa45` docs(harness): record Q1a quality-budget contract repair in lane P log

## Implemented
- `.github/workflows/ci.yml`: budget step restores `--criterion benches/results/benchmark_results.txt`
  (same value as origin/main:154); new second step runs the same argv + `--self-test-degraded`.
  Both steps are named "Historical baseline integrity ... not candidate performance" with a comment.
- `scripts/ci.sh`: new lane `run_quality_budget_lane` (runs from repo root, same argv as CI,
  full call then self-test, fails closed when python3 is missing); runs as step 3/6 of the normal flow
  (mirrors CI order) and standalone via `./scripts/ci.sh quality-budgets`; unknown arg exits 2.
  `make ci` / `just ci` delegate to ci.sh unchanged.
- `scripts/ci-parity-check.sh`: removed stale Makefile/justfile string greps (they already failed before
  this task, unrelated to required lane) and the stale ci.sh/ci-features check; now checks the required
  lane env wiring + delegation, and *executes* the argv-level contract tests (classes
  WorkflowArgvContract, LocalLaneContract, LaneAgreement, DocsNaming; non-recursive, no git needed).
- `scripts/test_check_quality_ci_contract.py` (new, 20 tests, stdlib only): checker contract via
  subprocess (missing `--criterion` rc 2; missing file, invalid unit, 116% regression non-zero; valid
  temp file / repo snapshot / self-test rc 0), workflow argv extraction (exact argv, parses with the
  checker's own argparse, literal real run), local lane argv recorded via a fake python3 on PATH,
  local==CI agreement, fail-closed on checker failure and on missing python3, `make -n ci` delegation,
  lane naming, policy doc naming, parity script pass + negative (copy of tree with `--criterion` removed).
  No skips: missing make/python3 is a failure.
- `docs/quality/retrieval-performance-policy.md`: new section "Historical baseline integrity lane"
  (CI and local; not candidate performance; `--criterion` mandatory; missing tool/file = failure; Q1b after Q7).
- Lane P docs: Q1a entry (PT-BR) in `docs/harness/progress/2026-10-05-improvement-lane-p.md`;
  `docs/harness/progress.md` D3 marked resolved, Last commit = `1addb94`.
- `scripts/check-quality-budgets.py` reviewed, unchanged (`--criterion` stays required; floors/1.15 untouched).

## TDD evidence
RED (before implementation), `python3 scripts/test_check_quality_ci_contract.py -v`:
20 tests, 10 failures + 1 error. Key failure (WorkflowRealRun, workflow argv executed literally):
```
AssertionError: 2 != 0 : usage: check-quality-budgets.py [-h] --budgets BUDGETS --retrieval RETRIEVAL
                                --criterion CRITERION [--self-test-degraded]
check-quality-budgets.py: error: the following arguments are required: --criterion
```
plus the argparse error in `test_workflow_argv_satisfy_checker_parser`, argv mismatch in the workflow and
local-lane tests (ci.sh had no lane), naming/doc failures. Checker-contract tests (rc 2 without
`--criterion`, etc.) passed already, as they characterise the unchanged checker. Expected: the defect is
the workflow wiring, not the checker.

GREEN: `rtk proxy python3 -m unittest discover -s scripts -p 'test_check_quality_ci_contract.py'` ->
`Ran 20 tests ... OK`.

## Other verification
- `python3 scripts/check-quality-budgets.py --budgets docs/quality/budgets.json --retrieval tests/fixtures/retrieval_quality/baseline.json --criterion benches/results/benchmark_results.txt --self-test-degraded` -> pass (4 degraded cases blocked).
- `bash scripts/ci-parity-check.sh` -> PASS (inner 10 tests OK). It failed before the change (pre-existing stale checks).
- `bash scripts/ci.sh quality-budgets` -> both calls pass; `bash scripts/ci.sh bogus` -> exit 2.
- Full `bash scripts/ci.sh` exit 0: fmt, clippy (required features), lane, nextest 1927 passed / 1 skipped, wasm, doc + MCP reference.
- `bash docs/harness/bin/doctor.sh` -> OK (1 expected WARN: no review artifact for active task), re-run after commit 2 -> OK.
- `bash docs/harness/bin/sensors.sh quick` -> PASS (quick lane green). `.sensors-last`/`.sensors-log` were modified by the run and reverted (not committed, per E0 convention).
- `check-live-state.sh` PASS after commit 2 (worktree clean).
- Pre-commit hook (fmt + clippy) passed on both commits.

## Deviations / concerns
1. The brief's acceptance command uses `--criterion benches/results/benchmark_baseline.txt`. That file has the
   `Baseline:` / `name: value` format with no `time:` lines, so the checker exits 1 ("missing named hot path(s)"),
   independent of this task. origin/main used `benches/results/benchmark_results.txt`, which is what I use
   everywhere. Controller may want to correct the brief/plan text.
2. Added the self-test as an extra step in the required job (small added CI time, no network); `Test (ubuntu-latest)`
   keeps its name. Lane agreement is enforced by tests.
3. `ci-parity-check.sh` required more than the lane change: its legacy greps (Makefile `cargo test --features ...`,
   `ci.sh` loading `ci-features.env`) were already failing on HEAD; replaced by required-lane wiring checks as the
   brief asked ("sem depender de strings legadas de Makefile"). The script is not wired into any workflow.
4. Used EnterWorktree to switch worktrees because the Write hook blocks cross-worktree writes (lane-p for the
   implementation, then the plan worktree to write this report at the requested path).

## NOT RUN
- GitHub CI Linux run (no push, local-only). Current-run performance measurement (Q7/Q1b); this lane is
  historical baseline integrity only and must not be cited as performance verified.

---

## Fix report — review round 1

Commits: `06e9005` fix(ci): run quality-budget contract suite from required job and ci.sh;
docs commit after it (lane P log + progress.md Last commit = `06e9005`).

### Changes
- **Important (suite not wired)**: `.github/workflows/ci.yml` — new step
  "Historical baseline integrity contract tests ..." running `python3 scripts/test_check_quality_ci_contract.py`
  in the required Test job, right after the budget/self-test steps (same job, fail-closed). `scripts/ci.sh` —
  top-level `python3 "$SCRIPT_DIR/test_check_quality_ci_contract.py"` right after `run_quality_budget_lane`
  (step 3/6; `set -e` fail-closed). It is deliberately outside the lane function: the suite calls
  `ci.sh quality-budgets`, so putting it inside would recurse.
- **Detection tests** (new class `GateWiring`, 7 tests; helpers `workflow_steps`, `workflow_runs_contract_suite`,
  `ci_sh_runs_contract_suite`): real workflow and ci.sh are wired; mutated in-memory copies are detected when the
  step/line is removed, replaced by another script (other step), suffixed with `|| true`, or moved inside the lane
  function in ci.sh. `GateWiring` is also added to the classes `ci-parity-check.sh` executes.
- **Minor (labels)**: label tests now also `assertIn("not candidate performance")` for every workflow budget step name
  and both `==>` headers of the local lane output (exactly 2 headers asserted); step names and ci.sh echo for the
  self-test were reworded to contain it.
- **Side fix needed for the new wiring**: `ci-parity-check.sh` used `rg`; the suite now runs on stock runners
  (and ParityScript tests invoke it), so it uses `grep -F/-E` instead (no ripgrep dependency).

### RED evidence (pre-fix files from `HEAD` 96dfa45 fed to the new detectors)
```
HEAD workflow runs suite: False
HEAD ci.sh runs suite: False
working workflow: True ci.sh: True
```

### GREEN / covering tests
- `python3 scripts/test_check_quality_ci_contract.py -v` -> `Ran 27 tests ... OK`
- `python3 -m unittest discover -s scripts -p 'test_check_quality_ci_contract.py'` -> `Ran 27 tests ... OK`
- `bash scripts/ci-parity-check.sh` -> inner `Ran 17 tests ... OK`, `CI parity checks passed.`
- `bash scripts/ci.sh quality-budgets` -> both calls `"status": "pass"`, headers contain "not candidate performance"
- Full `bash scripts/ci.sh` -> exit 0 (suite ran in step 3/6: 27 tests OK; nextest 1927 passed / 1 skipped; wasm; doc)
- `bash docs/harness/bin/doctor.sh` -> OK (same 1 expected WARN); `check-live-state.sh` PASS; worktree clean.
- Pre-commit hook (fmt + clippy) passed.

### Not run
- GitHub CI (no push). Note: on the runner the new step needs bash, make and git history (the Test job already uses fetch-depth 0).
