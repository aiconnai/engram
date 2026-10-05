# Task H4 report: runner and post-commit external evidence (Lane P, Wave 4, fake writer only)

**Status:** DONE_WITH_CONCERNS (concerns are listed at the end; none blocks acceptance)
**Worktree:** `~/Projects/_aiconnai/engram/.claude/worktrees/engram-improvement-lane-p`, branch `claude/improvement-lane-p`, base `96c82c5` (H6 committed its work before I touched the shared files)
**Commits:**
- `2bdf942` feat(harness): add trusted runner, scope check and evidence recorder
- `1501600` docs(harness): refresh progress last commit after H4

## What was implemented

### New files
| File | Role |
|---|---|
| `docs/harness/bin/run-task.py` | Trusted runner (fake writer only). Has `run` and `cleanup` subcommands and the in-process `run_task()`. |
| `docs/harness/bin/check-scope.py` | Scope checker. Has the CLI and `check_scope()`. |
| `docs/harness/bin/record-evidence.py` | `record()` is in-process only; there is no record CLI. `verify` has a CLI. |
| `docs/harness/bin/harness_git.py` | **Not in the brief.** Shared neutralized git access plus byte-exact export and re-hash. All three tools above use it, which is why it is a separate module (same reason H3 split out `sandbox_registry.py`). |
| `docs/harness/bin/run-runner-smoke.sh` | Separate real-Docker smoke, a copy of the `run-sandbox-smoke.sh` pattern. It is not in the lane, sensors or CI. |
| `docs/harness/tests/test_runner.py` (23), `test_scope.py` (24), `test_evidence_integrity.py` (15) | The offline tests the brief asks for. |
| `docs/harness/tests/runner_test_support.py` | **Not in the brief.** Holds `TempRepo` (commits built with plumbing, so it can create newline, dash and non-UTF-8 names, gitlinks, symlinks and mode changes), `RunnerStub` and `RunnerFixture`. `RunnerStub` wraps the H3 stub docker: on `start` it *interprets* the fake writer's file effects on the /work mount and runs per-gate scripts. It never executes the container argv on the host. |
| `docs/harness/tests/test_runner_smoke.py` (4) | Real Docker, production registry and catalog unchanged, fake writer. |

### Modified files (minimal edits)
- `tests/fake_writer.py`: added the `write_file`, `delete_file` and `claim_pass` behaviors. Existing behaviors are unchanged, and the H3 smoke still passes 14/14.
- `bin/run-offline-lane.sh`: added the `runner_unit` component, run as one `python3 -m unittest -v` over the three files. Its floor is exactly 62. The lane is now 9 components.
- `tests/test_offline_lane.py`: the count goes from 8 to 9, and a new test makes `runner_unit` mandatory. Skip, below-floor, zero and a missing file (for each of the 3 files) all fail.
- `bin/doctor.sh`: `require_exec`/`require_file` for the H4 files, plus a `require_grep` for the lane wiring.
- `progress.md`: new H4 row, and `Last commit` refreshed. The lane-p log has a PT-BR H4 entry.

### Design decisions
- **The writer workspace is a plain directory, not a git worktree.** The base is exported byte-exactly with `ls-tree` + `cat-file --batch` and has no `.git`, so the writer cannot reach git state.
- **Building the candidate is plumbing only.** The workspace is hashed with `hash-object --no-filters` + `update-index --index-info` + `write-tree`. In-tree `.gitattributes`, `.gitignore`, eol and filter drivers therefore cannot change the judged bytes. The workspace is refused if it has a `.git` entry (any case) or special files.
- **Scope is checked before the commit exists.** The scope check runs `base..TREE`. Only on pass does the supervisor run `commit-tree` (fixed identity `engram-runner`, parent = base) and create the ref `refs/engram-runner/candidates/<task>/<run>/a<N>r<M>`. The ref is create-only (`update-ref` with old value = zero OID). A scope refusal stops the run (exit 4) with no retry and no commit.
- **Gates run on a separate clean checkout** (`a<N>/c<M>`) built by exporting the candidate commit. The gate task-v2 has `target_sha = candidate`, so the runner now binds the candidate that H3 left `target_sha_bound:false`. After the gate the checkout is re-hashed; any tracked or untracked change becomes `gate_mutated_checkout`, which is never a pass.
- **H3 TCB finding (the `pr_title_policy` script is writer-writable):** I resolved it by making `docs/harness/` (the whole TCB) a built-in protected path in the scope check. The gate therefore always runs the base copy of the script. I did not switch it to running from `/tcb`, because that would change the H3 registry argv. This is documented in the `check-scope.py` docstring.
- **Git neutralization** follows H1. There is a fresh environment with no inherited `GIT_*`, `GIT_CONFIG_NOSYSTEM`, `GIT_CONFIG_GLOBAL=/dev/null`, `--no-replace-objects`, `GIT_NO_REPLACE_OBJECTS`, `GIT_GRAFT_FILE=/dev/null`, `GIT_ATTR_NOSYSTEM`, `core.attributesFile=/dev/null` and `core.hooksPath=/dev/null` (`update-ref` runs the reference-transaction hook), with literal pathspecs and `--end-of-options`. A test with a `git replace` ref shows that real objects are what gets judged.
- **Scope rules and test-weakening heuristic.** These are defined in the `check-scope.py` docstring:
  - Path rules: allowed paths, built-in protected paths (case-folded and NFC-normalized), lockfiles unless `allow_lockfiles`, symlinks that escape or point to protected or `.git` paths, submodules, mode and type changes, and unsafe filenames (control characters, newline, non-UTF-8).
  - Test-weakening findings: `test_deleted`, `test_lines_removed` (any removed line in an existing test file), `test_skip_added` (`#[ignore]`, `unittest.skip`, `pytest.mark.skip`/`xfail`, `it/test/describe.skip(`, `xit(`, …), and `assertion_removed` (net removal of assertion lines in non-test source, which covers Rust inline tests). `.expect(` is deliberately not counted as an assertion.
- **Evidence.**
  - `record()` reads only supervisor-owned inputs: the adapter's `outcome.json`, raw docker-attach logs (not RTK-filtered), and the staged gate task, registry and catalog. It recomputes every hash and refuses if the bytes on disk differ from the hash the adapter streamed.
  - Each evidence-v2 check uses the stdout log as `log_path`; the stderr logs and all inputs are in `sha256_manifest`, whose keys are paths relative to the gate root.
  - The `environment` field records the following. The full detail is in `run-record.json`, whose hash is in the manifest.
    - candidate ref and post-gate tree;
    - image, image id, docker endpoint and whether limits were verified;
    - writer adapter and harness version, CLI version ("unavailable"), model and effort requested, and reported identity ("unavailable");
    - log capture mode;
    - hashes of the registry, catalog, gate task, scope report and run record.
  - A truncated log turns pass into `warn`.
- **`verify()` is the only path that accepts evidence.** It checks:
  - **Runs root:** it must be trusted — absolute, owned by us, not group/world-writable, carrying the runner marker `.engram-runs-root`, not nested inside another runs root, and outside every worktree and git dir.
  - **Receipt location:** exactly `<root>/<task>/<run>/a<N>/g<M>/receipt.json`, with no symlink component.
  - **Candidate binding:** the receipt's candidate equals the operator's expected candidate. In git, the candidate commit exists, its tree equals the recorded tree, its parents are exactly `[base]`, and the runner ref points at it.
  - **H2 validation:** the H2 validator passes with every expectation supplied (`expectations_complete`, the equivalent of `--require-expectations`), and the gate task is validated against the trusted catalog.
  - **Integrity:** the whole `sha256_manifest` is recomputed; the staged registry and catalog match the trusted copies (otherwise `policy_drift`); the recorder id and verdict are pass; and the post-gate tree matches.
- **Budgets.** The pilot defaults are also the maximum a request can ask for.
  - `wall_seconds` 2700 (supervisor deadline; each sandbox phase gets the remaining time), `attempts` 2, `repair` 1 and `writer_concurrency` 1 (exclusive lock on `<runs_root>/.writer.lock`) are marked `enforced`. The check timeout is marked `enforced` by the adapter.
  - `turn_cap` 20 is recorded as **not enforced**, because the fake writer has no observable turns.
  - `cost_cap_usd` is **not enforceable**. A request that sets it, or lists `turn_cap`/`cost_cap_usd` in `require_enforced`, is refused (`budget_not_enforceable`).
  - A repair round that reproduces the previous tree counts as `writer_no_changes`.
- **Refusals before anything runs:** kill switch `ENGRAM_RUNNER_DISABLED=1` (rollback without deleting receipts), invalid request, real writer adapter (`writer_not_granted`), fake-writer behaviors the runner does not expose (e.g. `network`), budgets above the caps, policy mismatch, wrong repo, base ref mismatch, dirty base (tracked changes), runs root inside or overlapping a worktree (checked before anything is created), and a second writer.
- **Exit codes:** 0 passed / 1 failed / 2 refused / 3 unavailable / 4 scope_refused. The final line is `RUN_STATUS: ...`.
- **Failure path:** workspaces, writer and gate logs, scope reports, every candidate ref (failed ones included) and the failed evidence are all kept. `cleanup` removes only one run's `ws/` and `c<M>/`, never refs, evidence or logs, and never another task's directories.

## TDD evidence
- **Scope (proper RED):** I wrote `test_scope.py` first against a skeleton whose `check_scope` raised `NotImplementedError`.
  - RED: `python3 -m unittest discover -s docs/harness/tests -p 'test_scope.py'` gave `Ran 24 tests … FAILED (errors=33)`. All 24 errored on `NotImplementedError: check_scope`; 33 errors because subTests count separately. This was expected because nothing was implemented.
  - GREEN: the same command gave `Ran 24 tests … OK`.
- **Runner and evidence (deviation):** the implementation draft came before these two test files. To show the tests are real, I demonstrated RED by mutation, then restored the files from a scratchpad backup:
  - **Mutant A:** `verify` skips `_verify_manifest`, validates without expectations, and the scope refusal in the runner is disabled. Result: `test_evidence_integrity.py` FAILED (4 failures, 1 error): one byte in a log, deleted log, tampered hash with a re-sealed receipt, tampered manifest, and the symlink/group-writable case. `test_runner.py` FAILED (4): protected change, writer-claimed PASS (both tests), and the CLI exit codes.
  - **Mutant B:** the receipt path-shape check and the nested-runs-root check are removed. Result: FAILED (3): bundle in the gate checkout, runs root pointed at the writer workspace, and writer-claimed PASS in the workspace.
  - After restoring, all three files pass.

## Verification (all on the final committed tree, `git status` clean)
| Command | Result |
|---|---|
| `python3 -W error::ResourceWarning -m unittest -v docs/harness/tests/test_runner.py docs/harness/tests/test_scope.py docs/harness/tests/test_evidence_integrity.py` | `Ran 62 tests … OK` (about 57–75 s) |
| `bash docs/harness/bin/run-offline-lane.sh` | exit 0, `OFFLINE_LANE: PASS components=9 checks=388` (runner_unit 62/62 floor, lane_contract 14, sandbox_unit 54, context_budget 49) |
| `bash docs/harness/bin/doctor.sh` | exit 0, `OK harness doctor` (1 pre-existing WARN: no review artifact for the active task) |
| `bash docs/harness/bin/run-runner-smoke.sh` (real Docker 29.4.0, pinned `python@sha256:02108f5d…`) | `RUNNER_SMOKE: PASS tests=4`; 0 leftover `engram.sandbox.run` containers |
| `DOCKER_HOST=unix:///nonexistent/docker.sock bash docs/harness/bin/run-runner-smoke.sh` | exit 3, `RUNNER_SMOKE: UNAVAILABLE reason=docker daemon unreachable…` |
| `bash docs/harness/bin/run-sandbox-smoke.sh` (H3 regression after the fake writer edits) | `SANDBOX_SMOKE: PASS tests=14` |
| `python3 -m unittest discover -s docs/harness/tests -p 'test_sandbox_adapter.py'` | OK |
| `python3 -m unittest discover -s docs/harness/tests -p 'test_context_budget.py'` | OK |
| `shellcheck -x run-runner-smoke.sh run-offline-lane.sh` | clean |
| `check-doc-links.py docs/harness/progress.md` | `LINK_CHECK: PASS` |
| `check-live-state.sh --progress docs/harness/progress.md` | `PASS` |
| `check-commit-msg.sh` | OK for both commits |
| pre-commit (fmt + clippy) | passed |

Brief cases and where each is covered:

| Brief case | Tests |
|---|---|
| Dirty base, wrong repo and wrong ref | `test_runner` InputsAndEnvironment, `test_scope` InputsAndCli |
| Protected rename and delete | covered |
| Symlink escape | covered |
| Submodule | covered |
| Mode change | covered |
| Newline, dash and non-UTF-8 filenames | covered |
| Lockfile | covered |
| Existing-test weakening (4 variants) | covered |
| Wall, attempt and repair caps | `test_runner` Budgets; nonzero exit, logs survive |
| Turn cap | refused when it is required to be enforced |
| Check timeout | `test_runner` Budgets; nonzero exit, logs survive |
| One byte changed after the gate (log and candidate) | `test_evidence_integrity` |
| Deleted log | `test_evidence_integrity` |
| Tampered hash | `test_evidence_integrity` |
| Writer-claimed PASS | `test_runner` WriterClaims and `test_evidence_integrity` ForgedEvidence |
| Cleanup keeps candidate refs and other tasks | `test_runner` Cleanup |

## Deviations from the brief
1. I added two files outside the brief's list: `harness_git.py` and `runner_test_support.py`, for the reasons given in the table above. I also added the Docker smoke pair (`run-runner-smoke.sh`, `test_runner_smoke.py`), which the dispatch asked for.
2. `sha256_manifest` keys are relative to the gate root (e.g. `run/logs/00-pr_title_policy.stdout.log`, `inputs/registry.json`). The schema text says "repo-relative", but H2 validates structure only, and I did not change the schema (TCB).
3. **"Dirty base"** is implemented as: refuse when the repository checkout has uncommitted *tracked* changes, regardless of `--base-ref`. Untracked files are ignored. This is strict on purpose.
4. **Attempt vs repair semantics** I chose: a writer failure ends the attempt and starts a fresh one; a gate failure starts a repair round in the same workspace while repairs remain; a scope refusal ends the run.
5. The runner's TDD was demonstrated by mutation, not by test-first (see above).

## Concerns / open items
- Evidence integrity is not a signature. A host process running as the same uid can rewrite a whole bundle and its receipt. The protection covers the sandboxed writer, gate checks and post-gate drift. This is stated in the `record-evidence.py` docstring.
- The test-weakening heuristic is deliberately strict: any removed line in an existing test file is refused, so a test refactor always needs a human. It does not understand semantics (e.g. weakening a matcher on an added line).
- Gate checks must not write into the checkout. A check that writes build artifacts (e.g. a future cargo check without a separate target dir) will always end as `gate_mutated_checkout`. Registry checks need `/tmp` outputs.
- `runner_unit` adds about 60–75 s to the offline lane, because the wall-budget and check-timeout tests sleep on real clocks.
- The registry argv validator (H3) applies to writer invocations as well, so a fake-writer argument such as `-cfoo` is refused as inline code. This only affects the fake writer.

## NOT RUN
- `sensors.sh` quick/full, because they rewrite the tracked `.sensors-last` and would conflict with the other lane's telemetry. Doctor and the lane were run separately.
- Real CI.
- Independent review (the controller's job).

---

# Fix round 1 (review findings)

**Commits:**
- `e0021c9` fix(harness): resolve runner symlinks and lock writers per repository
- `240d0a4` docs(harness): refresh progress last commit after H4 fix (also marks the H4 row "+ correção")

## Changes
1. **[Important] Symlinks are resolved, not checked lexically.** In `check-scope.py`, `_LinkResolver` resolves each target against the candidate tree (`ls-tree -r`), following the tree's own symlinks, with a 40-hop cap. A loop or too many hops is `symlink_escape`, and so is an absolute target anywhere in the chain. `_symlink_findings` now runs over the whole candidate tree. It checks every new or changed link, and every unchanged link whose resolution differs from the base, so a new link can no longer redirect an old one. Links whose resolution did not change are skipped, so links already in the base cause no false refusals. Both review probes are now refused:
   - `src/a/d -> ../..` + `src/a/x -> d/../../outside` → `symlink_escape`
   - `src/a/d -> ../..` + `src/a/x -> d/docs/harness/bin` → `symlink_to_protected`
2. **[Important] Rust inline test weakening.** New finding code `rust_test_attr_removed`. It fires for any modified or deleted `.rs` file that loses, or edits, a line carrying `#[test]`, `#[tokio::test]` or any `path::test`, `#[rstest]`, `#[test_case]`, or a `#[cfg(...test...)]` gate. `#[ignore]` additions in `.rs` stay under `test_skip_added`.
   - Tested: (a) `#[cfg(test)]`→`#[cfg(any())]`; (b) a removed `#[test]` and a removed `#[tokio::test]`; `#[ignore]` added; adding new tests still passes.
   - (c) Limit, documented in the docstring: replacing an assertion with a weaker one on the same line count is not detected.
3. **[Important] The writer concurrency lock is now per repository.** The lock is `<git-common-dir>/engram-runner.lock`, resolved with `git rev-parse --path-format=absolute --git-common-dir`. It is taken after the repo checks and before the runs root is touched, and it is never writer-reachable because the writer only ever sees its plain workspace. Test: while the repo lock is held, runs from two different runs roots are both refused with `writer_concurrency_cap` and make no docker calls; after unlock, the run passes. `ENFORCEMENT` and the docstring are updated.
4. **[Minor items]**
   - The runs-root marker is only created in a directory the runner just created or in an empty one. A non-empty existing directory is refused (`untrusted_runs_root`) and no marker is written there.
   - `cleanup` uses `trusted_runs_root` directly and never creates anything. A missing runs root is refused and is still absent afterwards.
   - `harness_git` has `MAX_WORKSPACE_FILES` (100k) and `MAX_WORKSPACE_BYTES` (2 GiB), giving `workspace_too_large`, plus a `deadline` on `export_tree` and `build_tree` (scan, hash and git timeouts), giving `deadline_exceeded`. The runner passes `run.deadline` and maps `deadline_exceeded` to `failed: wall_budget_exhausted`. Other workspace errors map to: candidate side → scope_refused; base export → refused; gate checkout → `gate_mutated_checkout`.
   - New protected entries:
     - basenames anywhere: `.gitleaksignore`, `.pre-commit-config.yaml`, `codecov.yml`, `.codecov.yml`, `tarpaulin.toml`, `.tarpaulin.toml`, `.coveragerc`, `nextest.toml`;
     - prefixes: `.config/`, `docs/quality/`, `scripts/quality_baseline/`;
     - files: `scripts/check-quality-budgets.py`, `scripts/check-quality-baseline.py`.
   - `_verify_manifest` locally refuses absolute keys, backslashes, and `..`, `.` or empty components (`manifest_mismatch`), in addition to the H2 schema pattern.
   - **Real-clock sleeps:** I profiled with `--durations`. The wall-budget and check-timeout tests are not among the 8 slowest; runtime is dominated by git and stub-docker process overhead (the slowest tests take 3–5.5 s each, mostly pipeline setup). I kept the sleeps and updated the exact floor to 71.

## TDD
- **RED (tests added first, before any fix):**
  - `test_scope.py`: `Ran 29 tests … FAILED (failures=10)`. The 6 new protected-path subtests failed, plus rust attrs, chained symlink (escape and TCB), unchanged-link redirect, and loop.
  - `test_runner.py` + `test_evidence_integrity.py`: `Ran 42 … FAILED (failures=5, errors=2)`. Failures: concurrency across runs roots (3 subtest failures), non-empty dir adopted, cleanup created a dir; errors: the two workspace-bounds tests (no `max_files`/`deadline` parameters yet).
  - Then the corrected manifest-key test failed alone (`failures=1, errors=1`), because `_verify_manifest` accepted `../…/outside.txt`.
- **GREEN:** `python3 -W error::ResourceWarning -m unittest -v docs/harness/tests/test_runner.py docs/harness/tests/test_scope.py docs/harness/tests/test_evidence_integrity.py` gave `Ran 71 tests … OK` (scope 29, runner 26, evidence 16).

## Verification (on the committed tree, `git status` clean)
| Command | Result |
|---|---|
| `bash docs/harness/bin/run-offline-lane.sh` | exit 0, `OFFLINE_LANE: PASS components=9 checks=397` (`runner_unit` 71, floor 71) |
| `bash docs/harness/bin/doctor.sh` | exit 0, `OK harness doctor` |
| `bash docs/harness/bin/run-runner-smoke.sh` | `RUNNER_SMOKE: PASS tests=4`; 0 leftover containers |
| `check-live-state.sh --progress docs/harness/progress.md` | PASS |
| `check-commit-msg.sh` | OK; pre-commit passed |

**NOT RUN:** `sensors.sh` (cross-lane telemetry), real CI, independent re-review.

---

# Fix round 2 (review findings)

**Commits:**
- `26d43ea` fix(harness): refuse additive Rust cfg gates and lock failures
- `f96b200` docs(harness): refresh progress last commit after H4 fix 2

## Changes
1. **[Important] Tests switched off by an added cfg gate are now refused.** This is the new finding code `cfg_gate_added` in `check-scope.py`.
   - **What it checks:** every added line in a *modified* `.rs` file, test or non-test, that opens `#[cfg(`, `#![cfg(` or `#[cfg_attr(`.
   - **What is allowed:** a predicate that only combines plain atoms, using non-empty `all(...)` / `any(...)`. The atoms are `test`, `unix`, `windows`, `debug_assertions`, `doc`, `miri`, and `key = "value"` where the key is `feature`, any `target_*` key, or `panic`.
   - **What is refused:**
     - an empty `any()` / `all()`;
     - `not(...)`;
     - a bare identifier (`cfg(FALSE_FEATURE)`);
     - every added `cfg_attr`, because it can inject `#[ignore]`;
     - an attribute that does not close on the same line.
   - **Related changes:** `RUST_TEST_ATTR_RE` now also matches inner `#![…]` attributes. `tests.rs` and `test.rs` count as test files, and anything under a `tests/` directory already did. `_test_findings` also runs the Rust attribute and cfg checks for modified `.rs` test files. `#[ignore]` additions stay under `test_skip_added`.
   - **Known limit (documented in the docstring):** a multi-line attribute is judged by its first line only. Continuation lines added under an unchanged `#[cfg(` opener are not parsed. An added opener that does not close on its line is refused.
   - **Probes tested:**
     - `#[cfg(any())]` above `#[cfg(test)] mod`;
     - `#[cfg(FALSE_FEATURE)]` above `#[test]`;
     - `#[cfg(not(test))]`;
     - a multi-line `#[cfg(\n any()\n)]`;
     - an additive `#![cfg(any())]` in `tests/it.rs`;
     - `#![cfg(test)]`→`#![cfg(any())]` in `src/tests.rs`, which gives `rust_test_attr_removed` + `test_lines_removed` + `cfg_gate_added`.
   - **No over-refusal:** a test checks that adding `#[cfg(feature = "extra")]`, `#[cfg(target_os = "linux")]` and `#[cfg(all(test, unix))]` still passes.
2. **[Minor] Lock problems are refusals, not exceptions.** In `run-task.py`, an `OSError` or `GitError` from `git_common_dir` or from `os.open` of the lock, and a non-`BlockingIOError` `OSError` from `flock`, now become `refused: runner_lock_unavailable`. Test: with the git common dir chmod'ed to 0500, the run returns a refusal and makes no docker calls.

## TDD
- **RED (new tests first):** `python3 -m unittest -v docs/harness/tests/test_scope.py docs/harness/tests/test_runner.py` gave `Ran 59 tests … FAILED (failures=6, errors=1)`. The failures were the 4 additive-cfg subtests plus the additive inner-attribute case, and the inner-attribute edit in `src/tests.rs`. The error was the unusable lock raising `PermissionError` out of `run_task`.
- **GREEN:** `python3 -W error::ResourceWarning -m unittest -v docs/harness/tests/test_runner.py docs/harness/tests/test_scope.py docs/harness/tests/test_evidence_integrity.py` gave `Ran 75 tests … OK` (scope 32, runner 27, evidence 16).

## Verification (on the committed tree, `git status` clean)
| Command | Result |
|---|---|
| `bash docs/harness/bin/run-offline-lane.sh` | exit 0, `OFFLINE_LANE: PASS components=9 checks=401` (`runner_unit` 75; floor updated to exactly 75 in the lane and in `test_offline_lane.py`) |
| `bash docs/harness/bin/doctor.sh` | `OK harness doctor` |
| `bash docs/harness/bin/run-runner-smoke.sh` | `RUNNER_SMOKE: PASS tests=4` |
| `check-live-state.sh --progress docs/harness/progress.md` | PASS |
| `check-commit-msg.sh` | OK; pre-commit passed |

**NOT RUN:** `sensors.sh` (cross-lane telemetry), real CI, independent re-review.
