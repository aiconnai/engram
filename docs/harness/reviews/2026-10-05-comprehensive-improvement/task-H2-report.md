# Task H2 report — harden evidence schemas/validator and persistent offline lane (Lane P)

Worktree `engram-improvement-lane-p`, branch `claude/improvement-lane-p` (base `60343b3`). Local commits only; nothing pushed.

## Commits
- `283fca2` fix(harness): inspect receipt ACLs when xattrs mask the plus flag  (H1 gate fail-open found while running its suite; see Deviations)
- `dfbc0c3` test(harness): make live-state suite hermetic for approved baselines  (suite was already RED at 60343b3)
- `4ab5e83` feat(harness): harden evidence validator and add v2 schemas
- `3796e26` test(harness): pin live-state suite to the checked-out HEAD
- `73ae4a6` feat(harness): add mandatory offline lane to sensors, doctor and CI
- `4b678ee` test(harness): report inapplicable review-gate scenarios separately
- `64ed24d` test(harness): sync live-state fixture to sensors telemetry
- `a545e92` docs(harness): refresh offline lane floor counts
- `df9e9c8` docs(harness): log H2 validator hardening and offline lane (lane-P log entry + `Last commit` refresh)

## What was implemented
### Validator (`docs/harness/bin/validate-evidence.py`, rewritten)
- Strict JSON loader: duplicate keys, NaN/Infinity/-Infinity/overflowing floats, invalid UTF-8/BOM, >1 MiB, nesting >32, symlink or non-regular input all fail (`duplicate_key`, `non_finite_number`, `invalid_utf8`, `file_too_large`, `nesting_too_deep`, `unsafe_input_file`).
- Schema selected only by `schema_version` (or a matching explicit `--schema`); no key-marker / file-name guessing (`missing_schema_version`, `schema_mismatch`, `unknown_schema`). Semantic rules keyed on the resolved schema, not on payload keys (old code ran task rules on any payload with `task_id`).
- Built-in validator with a fixed keyword set; unsupported keyword/type/dialect or malformed schema => `unsupported_schema_keyword|unsupported_schema_dialect|invalid_schema` in both jsonschema modes (old code silently ignored unknown keywords and accepted unknown type names). bool is never integer/number, `1.0` is not an integer, `enum`/`const`/`uniqueItems` do not conflate `True` with `1`, `$` no longer accepts a trailing newline (pattern-constrained strings with control chars are rejected). New caps keywords: `maxItems`, `minProperties`, `maxProperties`; `propertyNames` applies the full sub-schema.
- jsonschema, when importable, is a cross-check with the same strict-integer rule. It can only add `validator_divergence`; it never relaxes the built-in verdict, so present == absent by construction and by test. A jsonschema crash => `jsonschema_exception` (fail closed).
- Semantics: 40-hex SHAs (all-zero rejected), canonical timestamps with real calendar dates (Feb 30, hour 24, second 60, year < 2000, future beyond injected clock + 5 min), path rules (`scope_violation`, `broad_path`, `unauthorized_glob`, `unnormalized_path`, `invalid_path_char`, `path_conflict`), duplicate check/finding/log ids, verdict-vs-checks (`verdict_check_mismatch`, `skipped_check_in_pass`, `status_exit_mismatch`), verdict-vs-findings (`pass_with_blocking_finding`, `comment_with_blocking_finding`, `fail_without_findings`), v2 isolation (`missing_isolation_capability`: write/exec need `sandbox_container` + `network_none`).
- External verification (caller-supplied only): `Expectations` dataclass / flags `--expect-candidate-sha`, `--expect-base-sha`, `--expect-tree-sha`, `--expect-policy-version`, `--catalog`, `--expect-catalog-sha256`, `--task`, `--logs-dir`, `--require-expectations` (incomplete => exit 2), `--now`, `--no-jsonschema`. Codes: `expected_sha_mismatch`, `missing_expected_field`, `expected_policy_mismatch`, `catalog_sha256_mismatch`, `catalog_invalid`, `unknown_check_id`, `missing_required_check`, `required_check_not_passed`, `task_id_mismatch`, `task_sha_mismatch`, `check_exceeds_timeout`, `log_hash_mismatch`, `log_file_missing`, `log_path_unsafe`, `task_invalid`. Log hashes are recomputed from the bytes under `--logs-dir` (no symlinks, cannot escape the dir).
- Trust boundary: `validate_file`/`validate_file_detailed` are structure-only; results carry `trust: structure_only` and `expectations_applied`; `*-v1` artifacts get `NOTE[HISTORICAL_V1]`, cannot satisfy a policy expectation, never become trusted. Nothing reads expectations from the payload (tested: a payload with `expected_commit_sha`, self-consistent wrong SHA, etc.).

### Schemas / fixtures
- New `task-v2`, `evidence-v2`, `review-v2` schemas (explicit caps on every array/string/object/integer, enforced by a lint test) and `schemas/check-catalog-v1.json`. v1 schemas untouched (historical).
- Fixtures: `valid_{task,evidence,review}_v2.json`, `check-logs/*.txt` (synthetic; evidence recorder `synthetic-test-fixture`), 9 new adversarial `invalid_*.json`. `--self-test` now 18 checks and prints `SELF_TEST_RESULT: passed=N failed=0`.

### Offline mandatory lane
- `docs/harness/bin/run-offline-lane.sh` runs 6 components, fail-closed: validator unit tests, `--self-test`, `test-fixtures.sh`, `test-check-live-state.sh`, `test-review-gate.sh`, plus its own contract tests (`docs/harness/tests/test_offline_lane.py`, 11 tests with stub components: zero tests, skip, below floor, failing self-test, non-zero exit, missing summary, deleted component, NOT RUN, missing jsonschema, usage). A component fails on: exit != 0, summary missing, count 0, count below the tripwire floor, skip / NOT RUN above `HARNESS_LANE_MAX_NOT_RUN` (default 0), or missing jsonschema (never skipped).
- Wiring: `sensors.sh` quick (`run_offline_lane`) and full (granular step `offline_lane`, also in `ci_steps`/label), `scripts/ci.sh`, and an unconditional step in the required `Test (ubuntu-latest)` job of `.github/workflows/ci.yml` (+ apt install of `python3-jsonschema`, `acl`). **Choice documented:** E0 audit §4 shows the live required contexts are Format, Clippy, Documentation, Test (ubuntu-latest), Security Audit, Cargo Deny; `Harness Contract` is not required live, so the Test job carries the lane. No job was restructured.
- `doctor.sh` now fails if: the runner/suites are missing or non-executable, the runner drops a component, the lane is absent (active, non-comment line) from sensors quick, sensors full, `scripts/ci.sh`, or the CI Test job step, the Test job loses its required name, or the step is conditional / `continue-on-error`.
- Docs: GATES.md (new "Offline mandatory lane (H2)" section), README (structure table, sensor modes), JSON_OUTPUTS.md (validator envelope fields).

## TDD evidence
RED (before any validator change; `python3 -m unittest discover -s docs/harness/tests -p 'test_validate_evidence.py'`): `Ran 75 tests ... FAILED (failures=32, errors=305)`. Behavioral failures against the OLD validator included: `enum [1]` accepted `True` (`test_enum_and_const_do_not_conflate_bool_and_int`), `pattern "^[0-9a-f]{4}$"` accepted `"abcd\n"`, 16 unsupported keywords (`oneOf`, `$ref`, `format`, ...) silently ignored, unknown `type: "strng"` accepted, `propertyNames` only honoured `pattern`, `maxItems`/`maxProperties` ignored, `--self-test` printed no count. The remaining errors were the new API (`Expectations`, `validate_file_detailed`, `StrictJsonError`) not existing yet, expected for new surface.
GREEN: after implementation `Ran 75 tests ... OK`, later 76 (+1 CLI diagnostics test). `-W error` run clean (no warnings).
RED for the gate fix (`283fca2`): `test_receipt_acl_with_xattr_rejected` failed with `expected rc=3, got rc=0` (PASS receipt with ACL+xattr accepted); GREEN after the fix.

## Verification (final, on committed HEAD)
- `python3 -m unittest discover -s docs/harness/tests -p 'test_validate_evidence.py'`: 76 tests OK (110-case adversarial table; present vs absent corpus compared in separate processes with the import blocker).
- `python3 docs/harness/bin/validate-evidence.py --self-test`: 18 passed, 0 failed.
- `bash docs/harness/bin/test-fixtures.sh`: 39 passed, 0 failed.
- `bash docs/harness/bin/test-check-live-state.sh`: PASS (assertions: 19).
- `bash docs/harness/bin/test-review-gate.sh`: 36 tests, 226 assertions, 0 failed, Not run 0, Not applicable 0 (macOS).
- `python3 -m unittest ... -p 'test_offline_lane.py'`: 11 OK (also under bash 3.2.57).
- `bash docs/harness/bin/run-offline-lane.sh`: `OFFLINE_LANE: PASS components=6 checks=199`.
- `bash docs/harness/bin/doctor.sh`: OK (1 pre-existing WARN: no review artifact for the active task). 9 mutation checks (wiring removed from sensors quick/full/commented-out, ci.yml step removed/`continue-on-error`/`if:`, ci.sh, runner dropping the review-gate or fixtures component) each made doctor exit 1 with the expected message; restored afterwards.
- `bash docs/harness/bin/sensors.sh quick`: pass (offline lane inside). `bash docs/harness/bin/sensors.sh` (full): pass (2026-10-05T05:27:17Z, 107 s; `ci_steps` includes `offline_lane: pass`). The FIRST full run (before `64ed24d`) FAILED in the lane's `live_state` component because the suite read the real progress.md `Last sensors` row, which only matches `.sensors-last` right after a progress refresh (every sensors run rewrites the receipt); fixed by syncing that row in the test's progress copy. `.sensors-last`/`.sensors-log` restored, not committed.
- shellcheck clean on `run-offline-lane.sh`, `sensors.sh` (info only), `doctor.sh`, `test-check-live-state.sh`, `test-review-gate.sh`; `test-fixtures.sh` has one PRE-EXISTING SC2034 warning (line 82), untouched.
- Linux check in an `ubuntu:24.04` image (`pra-h2-tests:ubuntu24`, non-root, `--network none`, fresh `git clone` of the branch): `--self-test` 18/18, `test-fixtures.sh` 39/39, `test-check-live-state.sh` 19, `test-review-gate.sh` 36 tests / 217 assertions / 0 failed (Not run 1 = ACL, no `setfacl` in the image; the CI step installs `acl`), lane fails closed with `reason=jsonschema-missing` because the image has no jsonschema.

## Deviations / files outside the brief's list (all declared)
1. `docs/harness/bin/review-gate.sh` (H1 file): `ls -l` prints `@` instead of `+` when a path has xattrs AND an ACL, so the receipt-ACL check missed it (reproduced: PASS accepted for a receipt with `everyone allow write`). 2-line fix + regression test (`283fca2`). Needed because the lane runs the H1 suite, which failed in this sandbox for that reason.
2. `docs/harness/bin/test-check-live-state.sh`: the suite was already failing at `60343b3` (hard-coded old approved baseline `843fd52`/snapshot `3586a40`, coupled to the live progress.md). The approved-baseline path is now covered by a hermetic synthetic repository (same output assertions), and the happy path runs on a copy of progress.md with `Last commit` pinned to the checked-out HEAD (CI pull_request checkouts are merge commits). The `Last sensors` row of the progress copy is synced to the current `.sensors-last` (telemetry freshness is not what this regression suite tests). Verified with HEAD = a synthetic merge commit in the Ubuntu container. No assertion removed; one assertion counter added for the lane.
3. `docs/harness/bin/test-review-gate.sh`: new `not_applicable` bucket (case-variant path on a case-sensitive FS, xattr+ACL on non-macOS) printed separately as `Not applicable: N`; genuinely unavailable capabilities (ACL tools) stay `NOT RUN` and fail the lane.
4. 6th lane component (`test_offline_lane.py`) and `run-offline-lane.sh` itself are additions beyond the five required components, to make "lane missing / zero tests / failure => non-zero" demonstrable.
5. `GATES.md`, `README.md`, `JSON_OUTPUTS.md` edited (documentation of the lane and envelope). GATES' stale "Required checks no GitHub" section (D4) was NOT touched (H6).
6. v2 schemas instead of editing v1 in place: keeps v1 historical and byte-identical (brief: new versions must not reinterpret v1 as new trust; rollback disables the new consumer).

## Open concerns
- `validate-evidence.py` is ~1.3k lines and `test_validate_evidence.py` ~1k (over the 800-line guideline). Left as single files per the plan; a split (parser / schema engine / semantics / expectations) needs plan guidance.
- Not run: GitHub Actions (no push). The Linux ACL scenario of `test-review-gate.sh` (needs `setfacl`) was not executed anywhere; on GNU `ls -lde` is unsupported so the gate takes its fail-closed branch, expected to pass but unverified. `python3-jsonschema` from the Ubuntu runner apt (4.10.x) was not exercised (local jsonschema is 4.26.0); the built-in validator is canonical, jsonschema can only add failures, so the risk is a false `validator_divergence` on an old library.
- CI step uses apt (network) for `python3-jsonschema`/`acl`; unpinned apt versions.
- Tripwire floors in `run-offline-lane.sh` (60/15/30/15/30/8) are below current counts (76/18/39/19/36/11) on purpose; they must be raised, not lowered, as suites grow.
- `.sensors-last`/`.sensors-log` were modified by the sensors runs and restored (not committed), same policy as E0.
- Independent review of H2 is the controller's job; H3 must not start until the lane is approved and persistent.
