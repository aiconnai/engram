# Task H5 report — SHA-bound review and read-only merge-policy evaluator

Worktree: `~/Projects/_aiconnai/engram/.claude/worktrees/engram-improvement-lane-p`
(branch `claude/improvement-lane-p`, base `f96b200`; O2 committed `a40430d`/`e6807e3` concurrently, untouched).

Commits:
- `8950f47` feat(harness): add read-only merge-policy evaluator and workflow
- `7e4a2c9` docs(harness): refresh progress last commit after H5

Nothing was pushed, merged, approved, deployed or published. No GitHub call was made.

## What was implemented

### `docs/harness/bin/merge-gate.py` (NEW)
Read-only, deterministic evaluator. Output: one JSON document on stdout
`{evaluator, merge_policy, decision: eligible|refused, reasons:[{section, code, detail}], sections{...},
bound:{task, base, head, tree, policy[, integrated]}, authority, recheck_before_merge}`; stderr line
`MERGE_GATE: ELIGIBLE|REFUSED head=... reasons=...`. Exit 0 eligible / 1 refused / 2 usage.
`eligible` requires every section to be `pass` (an unevaluated section is refused: `incomplete_evaluation`);
any unexpected exception in the CLI becomes `refused` (`evaluator_error`), never eligible.

Sections and sources of trust:
- **refs**: head/base re-resolved from git via `--head-ref`/`--base-ref` (never from a payload); optional
  `--expect-head/--expect-base` cross-check; expected policy version must equal the trusted registry's
  `policy_version`. Evaluator, registry/catalog, review-v2 schema and Q5 policy come from the checkout the
  script lives in (run from base/trusted copy; candidate is `--repo`).
- **evidence**: H4 `record-evidence.verify()` for `expect_candidate = current head`; then task id, `stale_base`
  (evidence base != current base), tree, evidence policy.
- **gate_history**: any other H4 gate receipt for the same task + candidate with verdict != pass → refused
  ("later FAIL" of the runner).
- **review_receipt**: operator receipt in the H1 KEY=VALUE format, **`RECEIPT_VERSION=2`** = the 10 H1 keys +
  `REVIEWER`, `POLICY_VERSION`, `REVIEW_CONTEXT=fresh-readonly` (operator attests the reviewer got
  task/diff/files/evidence without the writer transcript). Strict parse (unknown/duplicate/missing key, bad
  formats). v1 receipt (marker flow) → `reviewer_unavailable` (history only). Task/base/head/tree/policy/context
  must match.
- **review_scope**: trusted `review-gate.sh` copy (`--review-gate` + `--expect-gate-sha256`, must equal the
  receipt `GATE_SCRIPT_SHA256`) runs `scope` against the candidate; only the fixed header
  (SCOPE_MODE..DIFF_SHA256, printed before any writer-controlled path line) is parsed; `DIFF_SHA256` must equal
  the receipt.
- **review**: artifact at `REVIEW_ARTIFACT` must hash to `REVIEW_SHA256`; the exact hashed bytes are validated
  (temp copy, no TOCTOU). `REVIEW_VERDICT` marker → `review_legacy_marker`; non-JSON/prose → `review_malformed`;
  non-v2 → `review_not_v2`; H2 `validate_file_detailed("review-v2")` with expected head/base/policy, complete
  expectations (catches PASS with blocking finding, wrong SHA) → `review_invalid`; verdict != pass →
  `review_not_pass`.
- **reviewer**: operator-owned allowlist (`--identities`, `merge-gate-identities-v1`:
  `operators[]`, `reviewers{id:{lineage}}`, `writers{adapter:{lineage}}`). Receipt OPERATOR and REVIEWER must be
  listed; the free-text `reviewer` field must equal the receipt REVIEWER (`reviewer_claim_mismatch`) but never
  authenticates.
- **lineage**: reviewer lineage from the allowlist; writer lineage from the allowlist entry of the
  `writer_adapter` that the RUNNER recorded in the H4 evidence (`fake_writer`); missing → `lineage_unavailable`
  (never inferred from a name), equal → `lineage_not_independent`, payload `metadata.lineage` contradicting →
  `lineage_claim_mismatch`.
- **ci**: operator-saved `gh api repos/<o>/<r>/commits/<head>/check-runs` (object, or `--slurp` page list);
  `total_count` must equal saved runs; any run for another head → refused; only app `github-actions` counts
  (other apps / commit statuses can be posted by any token); latest run per required context (in-progress rerun
  = newest) mapped and judged by Q5 `check-security-gate.verdict()` with no allowed skips. Required contexts:
  Format, Clippy, Test (ubuntu-latest), Documentation, Security Audit, Cargo Deny, Security Gate.
- **integrated**: optional `--integrated-ref` (merge queue / synthetic merge) must have the same tree as the
  verified head, else `integrated_tree_unverified` (needs its own evidence; H4 cannot record merge commits
  because it requires parent == base). Without it, `stale_base` guarantees a merge onto the current base
  reproduces the head tree.
- **head_recheck**: head and base refs re-resolved immediately before emitting; moved → refused.
- Operator-owned files (receipt, allowlist, CI JSON, gate copy): absolute, regular, no symlink, nlink == 1,
  owned by us, no group/other write (file and directory), ACL check (`ls -lde` on macOS; ACL on Linux → refuse),
  and no ancestor is the same filesystem object as a worktree / git dir of the judged repo (`os.path.samestat`).

### `.github/workflows/agent-evidence.yml` (NEW)
`on: pull_request` (branches main) only; top-level `permissions: contents: read`; no secrets; two
`actions/checkout` pinned by SHA with `persist-credentials: false`: base sha → `trusted/`, head sha →
`candidate/` (fetch-depth 0). Steps: fail if the base has no evaluator; run `test_merge_gate.py` from
`trusted/`; run `python3 trusted/docs/harness/bin/merge-gate.py --repo candidate --head-ref $HEAD_SHA
--base-ref refs/remotes/origin/$BASE_REF --expect-head --expect-base --expect-policy-version <trusted registry>`;
all `${{ }}` values go through `env:`. The job succeeds when the decision is well formed (eligible↔0,
refused↔1) and writes it to the step summary. Without operator inputs the decision is `refused` by design; the
job result is not a merge signal and the context is not required. Covered by Q5
`check-workflow-supply-chain.py` (PASS).

### Wiring
- `run-offline-lane.sh`: new component `merge_gate` (`python3 -m unittest discover -s docs/harness/tests -p
  'test_merge_gate.py' -v`), exact floor `FLOOR_MERGE_GATE=36`, component count 9 → 10.
- `test_offline_lane.py`: stub for `test_merge_gate.py`, `components=10`, new test that skip / below floor (35) /
  zero / missing file fail the lane (15 tests).
- `doctor.sh`: `require_exec merge-gate.py`, `require_file test_merge_gate.py`, `require_grep` for the lane
  command, and `merge_gate_workflow:read_only` (no `pull_request_target`/`secrets.`/`write` in non-comment
  lines, permissions block exactly `contents: read`, evaluator invoked from `trusted/`). Verified to FAIL when
  the workflow gains an extra permission line (then restored).
- `GATES.md` (unlisted file, needed for the consumer migration note): new section "Merge-policy read-only (H5)"
  and lane items 9 (H4 runner tests — was missing from the numbered list) and 10 (H5).
- Lane P log entry (PT-BR) and progress.md table row + Last commit `8950f47`.

## TDD evidence

Deviation: the evaluator was drafted before the test file. To obtain a real behavioral RED, the tests were run
against a deliberately permissive stub evaluator (same API, always `eligible`), then the real file was restored.

RED — `python3 -m unittest discover -s docs/harness/tests -p 'test_merge_gate.py'` (stub in place):
```
FAIL: test_wrong_policy_version_is_refused (...)
FAIL: test_wrong_task_is_refused (...)
Ran 36 tests in 23.279s
FAILED (failures=47, errors=3)
```
Expected: every refusal case asserts `decision == refused` + a specific reason code; the stub accepts everything.

First real run: 2 errors in the TESTS (fixture helper param name collision `identities`, and loading
check-workflow-supply-chain.py without registering it in `sys.modules` for its dataclass) — fixed in the tests;
the hard-link test was also corrected (the helper rewrote the receipt, dropping the extra link) to reuse one
`Inputs` via `dataclasses.replace`.

GREEN — `python3 -m unittest discover -s docs/harness/tests -p 'test_merge_gate.py' -v`:
```
Ran 36 tests in 36.357s
OK
```
(36 `... ok` lines, no skips, no warnings.)

Brief case → test mapping (all refused): wrong task (`test_wrong_task_is_refused`), wrong SHA
(`test_wrong_head_sha_is_refused`), wrong policy (`test_wrong_policy_version_is_refused`), reviewer unavailable
(`test_reviewer_unavailable_is_refused`: no allowlist, no receipt, v1 receipt), malformed/prose PASS
(`test_malformed_and_prose_pass_reviews_are_refused`), PASS with blocking (`test_pass_with_a_blocking_finding_is_refused`),
log missing (`TamperedEvidence.test_missing_log_is_refused`), later FAIL (`test_a_later_failing_gate_for_the_same_candidate_is_refused`,
`test_later_fail_wins_and_a_later_rerun_success_is_the_latest_result`), stale base (`test_stale_base_is_refused`),
head swapped during evaluation (`test_head_or_base_swapped_during_evaluation_is_refused`), approval of previous
candidate (`test_replayed_approval_of_a_previous_candidate_is_refused`), forged/unauthorized reviewer
(`test_forged_reviewer_field_does_not_authenticate`, `test_unauthorized_reviewer_and_operator_are_refused`),
lineage swap (`test_lineage_comes_from_the_trusted_allowlist_never_from_names`), merge queue
(`test_merge_queue_commit_needs_its_own_evidence_unless_its_tree_is_the_head_tree`), trusted locations,
CI provenance, read-only (no ref/status/runs-root change), workflow contract + untrusted variants, CLI exit codes.

## Other verification (argv, exit, counts)
- `rtk proxy bash docs/harness/bin/run-offline-lane.sh` → exit 0, `OFFLINE_LANE: PASS components=10 checks=438`
  (validator_unit 76, validator_self 18, fixtures 39, live_state 40, review_gate 36, lane_contract 15,
  sandbox_unit 54, context_budget 49, runner_unit 75, merge_gate 36); 3m33s wall.
- `python3 -m unittest discover -s docs/harness/tests -p 'test_offline_lane.py'` → 15 tests OK.
- `rtk proxy bash docs/harness/bin/doctor.sh` → `OK harness doctor` (before and after commits); JSON shows the 4
  new checks `pass`.
- `rtk proxy python3 scripts/check-workflow-supply-chain.py` → `workflow supply-chain: PASS`;
  `scripts/test_check_workflow_supply_chain.py`, `test_check_security_ci_contract.py`, `test_check_security_gate.py` → OK.
- `python3 docs/harness/bin/check-doc-links.py docs/harness/progress.md docs/harness/GATES.md` → PASS;
  `test_context_budget.py` OK; `git diff --check` clean; pre-commit (fmt+clippy) passed;
  `check-live-state.sh --progress docs/harness/progress.md` → PASS.

## NOT RUN
- Real GitHub execution of `agent-evidence.yml` (no push/PR by mandate); real CI; `sensors.sh` full/quick
  (cross-lane telemetry, as in previous lane-P tasks); `gh api` fetch of real check runs; independent review
  (controller's job).

## Concerns / notes for review
1. New receipt version: H5 consumes the H1 format with `RECEIPT_VERSION=2` (+3 keys). `review-gate.sh post`
   still only accepts v1 (and marker reviews); v2 receipts are for review-v2 JSON reviews, which H1 cannot read.
   This is the "consumer migration": the legacy marker stays history and never satisfies the merge gate.
2. Required CI contexts and the `github-actions` app slug are constants in the evaluator (a policy change is a
   change to the trusted base revision). `Security Gate` is required by the evaluator although it is not a live
   branch-protection context.
3. The review-gate `scope` subprocess runs git against the writer-editable candidate repo (`.git/config`
   filter/fsmonitor) — same honest limit documented for H1; in CI the candidate is a fresh checkout.
4. Linux ACLs are not inspected: any `+` ACL marker refuses (fail closed).
5. The workflow's job result is "decision produced", not "eligible"; on the PR that introduces it the base has
   no evaluator and the job fails by design. `refs/remotes/origin/<base_ref>` relies on `fetch-depth: 0`.
6. CI "latest wins" lets a later successful rerun supersede an earlier failure for the same context (an
   in-progress rerun blocks). If the owner wants "any failure on this head blocks", it is a one-line policy change.
7. O2 left "wire `retention` into run-offline-lane.sh" pending; whoever does it must bump the component count
   to 11 and keep the `merge_gate` stub in `test_offline_lane.py`.
8. The lane grows by ~36 s (one real H4 stub run per test class + review-gate scope per evaluation).

---

# Fix round 1 (review findings + controller rulings a/b)

Commits (lane-p, local only):
- `bcbb568` fix(harness): bind merge-gate CI to one ci.yml suite and v2 receipts
- `0d8c7c1` chore(harness): raise H1/H5 lane floors and guard workflow expressions
- `5b2fa0f` docs(harness): record H5 fix round 1 progress
- `280698c`, `403bf3e` docs(harness): progress Last commit refresh. My first refresh pointed at the wrong
  commit and the strict check-live-state failed. `403bf3e` fixes it; the check now passes.

O2 and O4 committed in this worktree while I worked. I waited for their staged files to land and committed only my
own paths (`git commit -- <paths>`).

## Changes
1. **[Critical] Cross-suite CI laundering.** New module `docs/harness/bin/merge_gate_ci.py`, imported by merge-gate.py.
   - `ci_policy_paths_changed`: base..head touches any of `.github/workflows/**`, `.github/actions/**`,
     `docs/harness/{bin,tests,checks,schemas}/**`, `docs/security/**`, `scripts/check-*.py`,
     `scripts/test_check_*.py`, `scripts/ci.sh`, `scripts/ci-*.env` or `tests/fixtures/security_gate_matrix.json`.
     This covers check-security-gate/findings, workflow-supply-chain, quality-budgets, run-offline-lane.sh and its
     components, review-gate.sh and merge-gate.py.
   - `ci_workflow_changed`: the candidate's `.github/workflows/ci.yml` blob differs from the base blob, or the
     file is missing.
   - Check runs now need `check_suite.id`. Each required context must resolve to exactly one `github-actions`
     suite; more than one gives `ci_context_multiple_suites` (ruling a).
   - The suite must map, in the new operator-saved `--workflow-runs` file, only to path `.github/workflows/ci.yml`;
     otherwise `ci_workflow_unbound`. That file must be complete, for this head only and well formed
     (`workflow_runs_*` codes).
   - Re-runs count only inside that one suite: latest run wins, an in-progress re-run blocks.
   - The gh api commands the operator runs are documented in GATES.md and in the module docstring.
2. **[Important] Receipt v2 producer and consumers (ruling b).**
   - `review-gate.sh` now accepts `RECEIPT_VERSION=2` as a superset of v1. v1 is still accepted. A v2-only key in a
     v1 receipt is malformed. In v2, REVIEWER, POLICY_VERSION and REVIEW_CONTEXT are format-checked.
   - Any `<placeholder>` value is now `receipt-malformed`, in review-gate.sh and in merge-gate.py.
   - New producer mode `review-gate.sh receipt-template <task> --repo --base --head [--review-file]` prints a v2
     receipt. All bound values are recomputed; OPERATOR, REVIEWER and POLICY_VERSION are left as placeholders.
     Without a final scope it exits 2 (usage).
   - The merge-gate test fixtures now build every receipt from the producer's output. New test
     `test_one_template_receipt_is_accepted_by_both_consumers`: one producer receipt is accepted by H1 `post`
     (PASS) and passes merge-gate's `review_receipt` and `review_scope` sections.
   - New H1 tests: `test_receipt_v2_is_a_superset_of_v1` and `test_receipt_template_round_trips_through_post`.
   - GATES.md documents the operator procedure and the fresh-context reviewer bundle: task, diff/files from
     scope, head files and H4 evidence, never the writer transcript.
3. **[Important] Rerun test rewritten** per ruling a:
   - `test_reruns_count_only_inside_the_single_ci_suite`
   - `test_a_same_named_job_in_another_suite_cannot_launder_a_failure`. A failure in suite 1 plus a success in
     suite 2 is refused, and so are two successful suites.
4. **Minor items folded in:**
   - **Policy binding:** `MERGE_POLICIES = {harness-hardening-v1: merge-policy-v1}`; an unknown policy version gives
     `policy_unsupported`. `bound` now also carries `merge_policy`, `ci_policy_sha256` (hash of the CI policy
     constants) and `evaluator_sha256`.
   - **trusted_input:** opens the file once with `O_NOFOLLOW|O_NONBLOCK` and validates it with `fstat` (regular,
     one link, ours, not group/world writable), then reads that same descriptor. The parent directory must be
     private. Every ancestor must be ours or root, and not writable by others unless sticky. ACL and
     worktree-identity checks are kept.
   - **Review artifact and gate copy:** the review artifact is opened with O_NOFOLLOW. The review-gate copy is
     executed from its verified bytes in a private temporary copy.
   - **Head re-check:** it now also re-checks `--integrated-ref` (`integrated_changed_during_evaluation`). The
     docstring, workflow comment and GATES.md state that the re-check is vacuous for a raw SHA in CI.
   - **Injection guard:** the workflow-contract test flags any `${{` inside a `run:` body, with a new "inline
     expression" variant. A new doctor check `merge_gate_workflow:no_inline_expressions` does the same; I confirmed
     it fails on a mutated copy, then restored the file.
   - **Workflow:** the trust comment no longer claims a modified copy is harmless; it says the result is untrusted
     and privileges are minimised. The policy-version read moved into `env:` to keep lines short.
   - **Line length:** all H5 files have lines of 100 characters or fewer.
   - **File split:** the test fixtures moved to `docs/harness/tests/merge_gate_test_support.py`, so the test file
     stays under 800 lines (test_merge_gate.py 593, support 283, merge-gate.py 787, merge_gate_ci.py 155). Doctor
     now requires both new files.
5. **Lane:** `FLOOR_REVIEW_GATE` 30 → 38 and `FLOOR_MERGE_GATE` 36 → 43, both exact counts; `test_offline_lane`
   stubs updated, plus a new review_gate below-floor test.

## TDD evidence
- **RED, behavioural probe:** I ran the previous evaluator (`8950f47`) on the new fixtures.
  - Format failure in suite 1 plus a later Format success in suite 2: `OLD laundering probe: eligible []`.
  - A head that changes ci.yml: the old code had no CI-policy refusal; its CI section failed only by accident
    (`ci_results_head_mismatch`).
- **RED, new merge-gate suite on the old evaluator:** `Ran 43 tests ... FAILED (errors=73)`. These are mostly API
  errors (no `workflow_runs`), not behavioural failures, so the probe above is the meaningful RED.
- **RED, H1:** `REVIEW_GATE_UNDER_TEST=<HEAD review-gate.sh> bash docs/harness/bin/test-review-gate.sh` →
  `Tests: 38 ... Assertions failed: 12`. Failures: template mode exited 2; v2 receipt PENDING; no
  `needs RECEIPT_VERSION=2` error.
- **GREEN:**
  - `python3 -m unittest discover -s docs/harness/tests -p 'test_merge_gate.py' -v` → `Ran 43 tests`, `OK`
    (43 `... ok` lines).
  - `bash docs/harness/bin/test-review-gate.sh` → `Tests: 38  Assertions passed: 245  Assertions failed: 0  Not run: 0`.
  - `python3 -m unittest discover -s docs/harness/tests -p 'test_offline_lane.py'` → 18 tests, OK.
  - `rtk proxy bash docs/harness/bin/run-offline-lane.sh` → exit 0, `OFFLINE_LANE: PASS components=12 checks=523`
    (review_gate 38/38, merge_gate 43/43, retention 21, standing_checks 52).
  - `doctor.sh` → OK (new merge_gate checks pass).
  - `check-live-state.sh --progress docs/harness/progress.md` → PASS.
  - `scripts/check-workflow-supply-chain.py` → PASS. `git diff --check` is clean and pre-commit passed.

## Not run
GitHub execution of agent-evidence.yml, real `gh api` output (the field shapes `check_suite.id`,
`workflow_runs[].check_suite_id` and `path` come from the public API docs and were not checked against live data),
`sensors.sh`, and independent review.

## Concerns
- The protected-path set is broad: any PR touching `docs/harness/bin/**` or `scripts/check-*.py` can never be
  `eligible` and is judged by a human only, by design. The owner may want to adjust the set.
- GitHub's workflow-run `path` field is assumed to be exactly `.github/workflows/ci.yml`. If the API returns a
  suffixed form such as `...@refs/...`, the result is `ci_workflow_unbound` (fails closed). Check this against real
  data before the first operator run.
- Re-runs are allowed only within one suite, which matches GitHub "re-run jobs" behaviour (same suite, new run
  attempt). A full "re-run all" that creates a new suite would be refused.

---

# Fix round 2

Commits (lane-p, local only):
- `cd5e7f3` fix(harness): protect every input of required CI jobs in merge-gate
- `29d4f7e` docs(harness): record H5 fix round 2 progress
- `3a373e7` docs(harness): refresh progress last commit after H5 fix 2

## Changes
1. **[Important] CI protected-path set**, in `docs/harness/bin/merge_gate_ci.py`.
   - **Static list.**
     - Protected prefixes: `.github/` (all of it), `.cargo/`, `.config/`, `scripts/`,
       `docs/harness/{bin,tests,checks,schemas}/`, `docs/security/`, `docs/quality/`, `benches/results/`,
       `tests/fixtures/retrieval_quality/`.
     - Protected files: `deny.toml`, `.gitleaks.toml`, `.gitleaksignore`, `.semgrepignore`, `rust-toolchain(.toml)`,
       `(.)rustfmt.toml`, `(.)clippy.toml`, `Makefile`, `justfile`, `tests/fixtures/security_gate_matrix.json`.
     - The fnmatch globs are gone.
   - **Derived list.**
     - `required_jobs(ci_text)` starts from the jobs named after a required context and adds their transitive
       `needs:`. For the real ci.yml that is fmt, clippy, test, docs, audit, deny, security-gate and its seven
       constituents.
     - `required_job_references(ci_text, exists)` collects every repository file named in those jobs' lines
       (`run:`, `with:`, `config-file: ./...`). Today that is 26 files.
     - `base_references()` applies this to the **base** ci.yml and base tree at evaluation time.
       `protected_changes` refuses anything that is statically protected or base-referenced.
   - **Tests.**
     - `test_every_file_a_required_job_references_is_protected` is the drift guard on the real ci.yml: every
       referenced file must be covered by the static list. It asserts that `scripts/generate-mcp-reference.sh` and
       `.github/codeql/codeql-config.yml` are among them.
     - `test_tool_configs_and_indirect_gate_inputs_are_protected` covers 17 paths, including deny.toml,
       .cargo/audit.toml, codeql config, .gitleaks.toml, .semgrepignore, the docs/security exception TOMLs,
       ci-*.env, ci.sh, generate_mcp_reference.py, rust-toolchain.toml and CODEOWNERS.
     - `test_files_referenced_by_the_base_ci_yml_are_protected_dynamically` uses a synthetic base ci.yml whose
       Format job reads `tools/fmt.json`. A change to that file is refused although it is not in the static list.
     - `ordinary_source_changes`: `scripts/bench-compare.sh` is now protected, so that test checks `benches/search.rs`
       instead.
   - **GATES.md** now states the broader set and the dynamic derivation.
2. **[Minor]**
   - **Fresh-context attestation:** `receipt-template` prints `REVIEW_CONTEXT=<fresh-readonly once you checked the
     reviewer got no writer transcript>`. The H1 test and the merge-gate fixtures fill it explicitly.
   - **Renamed test:** `test_template_parses_in_both_consumers_but_each_needs_its_own_review`. It checks that the
     template never pre-fills REVIEW_CONTEXT. A marker review plus its receipt passes H1 `post`; in merge-gate the
     same receipt passes review_receipt and review_scope but is refused as `review_legacy_marker`. A separate
     review-v2 JSON plus its own receipt is `eligible`. GATES.md states that H1 and H5 need separate review
     artifacts and receipts.
   - **Second suite on the same head:** GATES.md and the merge_gate_ci docstring say that a reopen or
     workflow_dispatch creating a second ci.yml suite leaves the head permanently `ci_context_multiple_suites`.
     Recovery is a new commit (new head, new evidence, review and receipt), never deleting runs from the saved file.
   - **Pinned CI policy:** `MERGE_POLICIES = {harness-hardening-v1: {id: merge-policy-v1, ci_policy_sha256:
     763198f2...}}`. A different fingerprint gives `policy_mismatch`. Tested by patching the pin; the pin also
     equals the live fingerprint.
   - **"Private" wording:** the parent-directory check is "ours and not group/world writable". The GATES.md
     wording, the docstring and the error message now say exactly that.
   - **Line length:** I wrapped the >100-char lines I had added in review-gate.sh, doctor.sh (fail messages now go
     through a variable), test-review-gate.sh, run-offline-lane.sh, test_offline_lane.py, merge-gate.py and
     merge_gate_ci.py. Long lines that O2/O4 added in those files were left alone.
3. **Lane:** `FLOOR_MERGE_GATE` 43 → 47 (exact). The `test_offline_lane` stub and below-floor case are 47 / 46.

## TDD evidence
- **RED:** `merge_gate_ci.py` from HEAD `403bf3e`, run with
  `-k test_tool_configs -k test_every_file -k dynamically`, gives `Ran 3 tests`, `FAILED (failures=13, errors=1)`.
  The failures include `[] != ['deny.toml']`, `['.cargo/audit.toml']`, `['.github/codeql/codeql-config.yml']`,
  `['scripts/generate-mcp-reference.sh']`, `['.gitleaks.toml']` and `['.semgrepignore']`; the error is the
  missing `required_jobs`.
- **GREEN:**
  - `python3 -m unittest discover -s docs/harness/tests -p 'test_merge_gate.py' -v` → `Ran 47 tests`, `OK`
    (47 `... ok` lines).
  - `bash docs/harness/bin/test-review-gate.sh` → `Tests: 38  Assertions passed: 245  Assertions failed: 0  Not run: 0`.
  - `test_offline_lane.py` → OK.
  - `rtk proxy bash docs/harness/bin/run-offline-lane.sh` → exit 0, `OFFLINE_LANE: PASS components=12 checks=527`
    (review_gate 38/38, merge_gate 47/47).
  - `doctor.sh` → OK with no failing checks.
  - `check-live-state.sh --progress` → PASS.
  - `check-workflow-supply-chain.py` → PASS. `git diff --check` is clean.

## Not run
Real workflow and `gh api`, `sensors.sh`, independent review.

## Concerns
- With all of `scripts/**` and `.github/**` protected, any PR that touches repo tooling, issue templates or
  CODEOWNERS is never `eligible`; it is human-only by design.
- The reference derivation is a token scan of the job lines. It over-approximates, which only adds protection, and
  it cannot see files that scripts read indirectly. The static `scripts/` prefix covers the known cases, such as
  generate_mcp_reference.py.
