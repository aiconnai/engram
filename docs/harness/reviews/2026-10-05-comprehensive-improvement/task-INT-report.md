# INT report: integrate lanes, Q1b and integration TODOs

Status: DONE_WITH_CONCERNS (concerns below: real-CI behaviour of the new workflow is unverified;
same-user trust limit of the candidate lane; floors still pending human review).

Branch `claude/engram-improvement-plan-edb43d`, start HEAD `77741c3`, final HEAD `4868d49`.
Local commits only: no push, no PR, no merge to main, no history rewrite.

## Commits (first-parent order)

| SHA | Subject | Step |
|---|---|---|
| `ee8c4ea` | chore(harness): merge improvement lane P into lane R | 1 |
| `a2fd341` | ci(ci): wire report-only candidate quality lane (Q1b) | 2 |
| `b8e693a` | ci(ci): wire SDK contract drift ratchet into required lanes | 3 |
| `a231cdb` | fix(infra): reconcile advisory exceptions after dependency updates | 4 |
| `e121894` | docs(embedding): fix private and ambiguous rustdoc intra-doc links | 5 |
| `33cf8a1` | chore(harness): ignore C4 spike build output and reconcile coverage map | 6 |
| `11a48a1` | fix(ci): map consumer exit 3 to a warning under bash -e | 2 (bug found by simulation) |
| `4868d49` | docs(harness): record lane integration progress (INT) | 8 |

Every commit: Conventional Commit validated by `check-commit-msg.sh`, explicit pathspecs, pre-commit hook
(fmt + clippy all-targets all-features) passed, no `--no-verify`, no Co-Authored-By.

## 1. Merge

`git merge --no-ff claude/improvement-lane-p` (lane P `3a373e7`). One conflict, as predicted:
`docs/quality/retrieval-performance-policy.md`. Resolved by keeping both: Q1a "Historical baseline
integrity lane" first, then the Q7 "Candidate evidence runner" sections (its intro now refers to the lane
above). No test dropped. `docs/harness/progress.md` now links the lane R log
(`progress/2026-10-05-improvement-lane-r.md`); `check-doc-links.py` PASS, and the H6 test
`test_live_summary_itself_links_to_nothing_pending` passes (the three DoctorLiveStateAncestry tests that
failed before the merge commit only did so because they clone committed HEAD, which still had lane R's
doctor; they pass after the merge).

## 2. Q1b (candidate lane wiring)

- New separate workflow `.github/workflows/quality-candidate.yml`, job "Candidate Quality Evidence
  (report-only, not required)". Not in `ci.yml`, in no `needs:` chain, not a required context. No
  `continue-on-error` anywhere (pinned by a test).
- Steps: checkout candidate to `cand/` (fetch-depth 0, persist-credentials false); anchor =
  `git merge-base HEAD origin/main`, or first parent when that equals HEAD (push to main);
  `git worktree add trusted <anchor>`; capture, runner and consumer run from `trusted/`, with
  `--candidate-dir cand --require-supervisor`, `ENGRAM_QUALITY_SUPERVISOR=${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}`
  and `--floors-anchor <anchor>`; output under `$RUNNER_TEMP`; report + Criterion capture uploaded.
- Deviation from Q7's snippet (deliberate): the feature list is read as data from
  `trusted/scripts/ci-required-features.env` (sed), not `source cand/...` (that would execute
  candidate-controlled shell in the supervisor shell).
- New consumer `scripts/consume-quality-candidate.py` + `scripts/quality_candidate/consume.py`: implements
  the documented Q1 consumption rule. Exit 0 accepted, 1 rejected (status/SHA/supervisor/marker binding,
  supervisor_required, missing sections, unreadable or NaN JSON), 3 intact but `floors.accepted` not true,
  2 usage. Writes a step-summary block. Workflow maps 3 -> `::warning::` + exit 0 (expected report-only
  state), anything else fails the job.
- Bootstrap: if the anchor lacks the runner/consumer (true for today's `main`), the job fails with an
  explicit `::error::` rather than running candidate-provided verifier code.
- Blocking unit tests: required `Test (ubuntu-latest)` job step "Fast offline Python contract tests ..." and
  new `scripts/ci.sh` step `[5/7]` run `python3 -m unittest scripts/test_run_quality_candidate.py` and
  `scripts/test_quality_candidate_consume.py` (ci.sh renumbered to 7 steps; nothing pinned the old numbering).
- Q1a historical integrity lane untouched (same steps/argv; `test_check_quality_ci_contract.py` 27 OK).
- Docs: policy doc gains the consumer contract and a "CI wiring (Q1b)" section (incl. promotion
  prerequisites and the same-user trust limit).
- H5 merge_gate_ci drift test did NOT need changes: the new Test-job references are all under the
  protected `scripts/` prefix; `test_merge_gate.py` 47/47 OK; the CI-policy fingerprint covers constants
  only, unchanged.

TDD evidence:
- RED: `python3 -m unittest scripts/test_quality_candidate_consume.py` -> `ImportError: cannot import name
  'consume'` (module absent; new behaviour). GREEN: 19 tests OK.
- Mutation check (scratch copy, 6 mutations: accepted check loosened, marker supervisor, supervisor_required,
  status, candidate SHA checks disabled, summary truncating): 6/6 KILLED.
- Local workflow simulation (scratchpad `sim-quality-candidate.py`, executes the YAML's own `run:` blocks
  with emulated GitHub env):
  - bootstrap vs the real `main` (`1952f3b`): Resolve step exit 1, `::error::trusted anchor 1952f3b... has
    no scripts/capture-criterion-candidate.py ...` (as designed).
  - full path, candidate `33cf8a1`, `origin/main` set to `a2fd341`: capture exit 0, runner exit 0
    (`status: pass`, supervisor `424242-1`, `supervisor_required: true`, `anchored: true`, `accepted: false`
    "review status is 'proposed-pending-independent-review'"); metrics lexical_fts5 recall@10 0.857 / mrr
    0.878 / ndcg@10 0.872, hybrid_tfidf 0.949 / 0.910 / 0.917; Criterion `extract_mixed` 4.96 us, ratio
    0.289; `entity_extractor_new/default` ratio 0.0002 (default Criterion params, macOS M5 Pro).
  - **Bug found**: the verdict step exited 3 and failed the job: GitHub runs `run:` with `bash -e`, so the
    bare `rc=$?` never executed. Fixed in `11a48a1` (`rc=0; ... || rc=$?`). RED: new
    `WorkflowVerdictStep` tests (run the real step text under `bash -e` with a fake consumer) ->
    `AssertionError: 3 != 0`; GREEN 23 tests OK; re-running the fixed step text on the simulated report:
    exit 0 with `::warning::candidate evidence is intact but NOT ACCEPTED ...`.
- actionlint clean on quality-candidate.yml, ci.yml, both SDK workflows; `check-workflow-supply-chain.py`
  PASS; `test_check_security_ci_contract.py`, `test_check_security_gate.py`,
  `test_check_workflow_supply_chain.py` OK.

## 3. SDK drift ratchet wiring

`python-sdk-live.yml` and `typescript-sdk-live.yml` `paths:` (pull_request and push) now include
`docs/quality/sdk-contract-drift-baseline.json` and `scripts/check-sdk-contract-alignment.py`. Required Test
job + ci.sh step 5 run `python3 -m unittest scripts/test_check_sdk_contract_alignment.py` (28 OK) and
`python3 scripts/check-sdk-contract-alignment.py --only typescript` (OK, 81 known, none new).
Finding: the unit tests import the Python SDK (2 errors without httpx, verified with `python3 -S`), so the
Test job gained a step `sudo apt-get install -y --no-install-recommends python3-httpx` (noble ships 0.26;
SDK requires >=0.25). Locally ci.sh needs httpx in python3 (present here).

## 4. Exception reconciliation

Lane P had already removed RUSTSEC-2026-0235 from `.cargo/audit.toml` and the manifest, and had corrected
versions (0.102.8 / 0.3.27), `engram-core 0.23.0`, `feature_gated=true`, `default_graph=false`,
`as_of=2026-10-05`. 0221 was not present. Changes in `a231cdb`:
- header rewritten to the post-Q6 state; 0258 `feature`/`exposure` corrected to the libsql-only path
  (it still claimed an AWS SDK h2 0.3 path; `cargo tree --all-features -i h2@0.3.27` shows libsql only);
- new governed record RUSTSEC-2026-0253 (lru 0.16.4 via aws-sdk-s3 1.120.0, `cloud` feature only,
  `cargo-audit:allowed-warning`, owner Ronaldo, expires 2026-12-31 = 87 days, follow-up: pin
  `BehaviorVersion` (src/sync/cloud.rs:56, src/storage/image_storage.rs:754 use `latest()`) then bump
  aws-sdk-s3 + aws-config with a bucket canary);
- `deny.toml` ignore comment corrected (no AWS SDK path; listed because deny runs all-features); ignore list
  unchanged (0049/0098/0099/0104/0258) and consistent with the manifest;
- `governance/exceptions.toml`: EXC-0001 removed (no subject: Cargo.lock has only reqwest 0.12.28; nothing
  cited the id as P-A.2.1 requires); the file now records why and points to the advisory manifest;
- `docs/security/security-gate-evidence.md`: post-integration update paragraph.

## 5. Docs gate

`src/embedding/queue/drain.rs:26` (private `super::jobs` link -> plain code) and `src/observability/mod.rs:7`
(ambiguous `counters` -> `mod@counters`). Doc comments only. Before: exit 101 with these 2 errors.

## 6. Housekeeping

`.gitignore` += `docs/decisions/assets/2026-10-05-c4-spike/target/` (135 MB build cache left in place, now
ignored). Coverage map: G-1 marked fixed by G1 (`974b7a6`, `9712ade`, `bbe9203`) with a new 3.4 row citing
`storage_posix_lock_regression_tests::{committed_writes_survive_another_process_open_and_close,
second_in_process_open_does_not_drop_the_first_handles_locks, mcp_paths::*}`; the C1 HTTP row no longer says
its checks are vacuous; G-3 marked fixed by Q2F (`98103b0`) with a new 3.3 row
(`contract_matrix::memory_search_limit_survives_the_result_cache`,
`memory_search_cache_distinguishes_workspaces_filter_and_min_score`,
`result_cache::tests::cache_key_distinguishes_*`).

## 7. Final gate results (verbatim key lines)

Full runs on code HEAD `11a48a13f20a3e8df2bcb9aac1b39129562eceaa` (the later `4868d49` changes only
progress docs; cheap checks re-run on it below). An earlier identical full pass was also recorded on
`33cf8a1` (make ci exit 0, nextest 2234/2 skipped; sensors full pass 2026-10-05T17:51:22Z, 294s).

| Command | Exit | Key output |
|---|---|---|
| `make ci` (-> `scripts/ci.sh`) | 0 | fmt/clippy ok; `Ran 27 tests ... OK` (quality contract); `OFFLINE_LANE: PASS components=12 checks=527`; `Summary [ 4.387s] 2234 tests run: 2234 passed, 2 skipped`; step 5: `Ran 63 tests OK`, `Ran 23 tests OK`, `Ran 28 tests OK`, `sdk contract alignment: OK (81 known drift entries, none new)`; WASM ok; `docs/MCP_TOOLS.md is up to date`; `Ran 15 tests OK`; `OK PDF worker packaging contract`; rustdoc `-D warnings --document-private-items` ok; `Required CI gates passed locally.` |
| `bash docs/harness/bin/sensors.sh` (full, no args) | 0 | `PASS (all deterministic gates green)`; `.sensors-last`: `status=pass ci_status=pass doctor_status=pass mode=full timestamp=2026-10-05T18:11:26Z duration_sec=314` (receipts saved to scratchpad and reverted, not committed, per E0 convention). The two `FAIL: PR title must not contain [codex]` lines are the expected negative self-tests. |
| `cargo audit` (fresh DB fetch) | 0 | `Loaded 1290 security advisories`, `Scanning Cargo.lock ... (647 crate dependencies)`, 0 vulnerabilities, `warning: 2 allowed warnings found` (RUSTSEC-2026-0192, RUSTSEC-2026-0253). DB HEAD still `ef6173c` after fetch. |
| `cargo deny check advisories bans licenses sources` | 0 | `advisories ok, bans ok, licenses ok, sources ok` (57 `duplicate` warnings, non-fatal) |
| `python3 scripts/check-security-exceptions.py --config ... --audit-config ... --deny-config ...` | 0 | `security exception check: PASS (10 governed records)`; `--self-test-expired` / `--self-test-missing-owner` PASS; `test_check_security_exceptions.py` OK |
| `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-default-features --features "$CI_REQUIRED_FEATURES" --no-deps` (with and without `--document-private-items`) | 0 / 0 | Finished, no warnings |
| On final HEAD `4868d49`: `bash docs/harness/bin/run-offline-lane.sh` | 0 | `OFFLINE_LANE: PASS components=12 checks=527` |
| `bash docs/harness/bin/doctor.sh` | 0 | `OK harness doctor` + expected `WARN: no review artifact found for active task` |
| `bash docs/harness/bin/check-live-state.sh --progress docs/harness/progress.md` (strict) | 0 | `head=4868d49 worktree_status=clean PASS live state matches current repository facts` |
| `git diff --check 77741c3 HEAD` | 0 | clean |
| `test_context_budget.py` | 0 | 49 OK |

Load-flaky multimodal process tests: not observed (both full nextest runs green), nothing modified.

## 8. Progress

`docs/harness/progress.md`: Last commit `11a48a1` (first parent of the progress commit, accepted by the
strict live-state check), Last sensors = the INT full run, lane R log linked, INT row in the task table,
D2 marked resolved (Q6). PT-BR INT entry appended to `docs/harness/progress/2026-10-05-improvement-lane-p.md`.

## NOT RUN / concerns

- Real GitHub CI (no push): neither the new Test-job steps (python3-httpx apt install, Python suites on
  ubuntu's python3.12) nor `quality-candidate.yml` ran on a GitHub runner. The simulation emulated the
  action steps (checkout, toolchain, rust-cache, protoc, disk cleanup, upload) and ran on macOS.
- On the first PR the candidate lane will fail by design (bootstrap: `main` lacks the runner) until this
  branch reaches the protected base; it is non-required, so it does not block.
- Trust limit (documented): candidate build scripts/tests run in the same job and user as the trusted
  verifier; the trusted worktree protects against verifier edits in the diff, not a malicious build script
  at runtime. Acceptance stays human.
- Floors and v2 corpus labels remain `proposed-pending-independent-review`; every report is
  `accepted: false` (NOT ACCEPTED warning) until a human accepts, seals and merges them.
- No independent review of INT itself (controller's job). No sdk live scripts run (`test-*-sdk-live.sh` need
  registry installs, not authorized).
- Network used: crates.io/RustSec only (cargo audit/deny DB fetch). Nothing else.
- `.superpowers/` artefacts (this report) are untracked, not committed.
