# Progress Log — Harness live-state closeout

## Engram 10/10 Wave 2 closure — 2026-07-11

Tasks 11–16 are represented by `bbd49fc`, `fc37fff`, `73f4959`, `bc05e81`,
`815d1af`, and `ce292ac`. Barrier findings were remediated by `72bdf0b`
(WebSocket peer-task cancellation) and `92734bc` (isolated PDF worker release
packaging). The final SQLite remediation uses a no-follow open boundary before
SQLite configuration; its regression proves the symlink target hash, DELETE
journal mode, and sentinel row remain unchanged. Non-Unix platforms fail
closed for filesystem databases when an equivalent atomic no-follow boundary
is unavailable, while `:memory:` remains supported. Detailed command evidence
is kept in the orchestrator-owned `.omo/evidence/` Wave 2 records.

### Final barrier and independent review — 2026-07-12

- Candidate `98b844680ee569df5f9254e7f5e2e9e51e59c900`, based on
  `9b832146a3c3cc326aaa9e12d0e72de679e7f75f`, passed Wave 2 barrier v4.
- All 21 barrier gates passed without exclusions. Evidence is immutable under
  `.omo/evidence/wave-2-integration-barrier-v4.md`, with raw receipts and a
  SHA-256 manifest in the sibling directory.
- Independent review v2 reran storage Unix (7/7), HTTP security (5/5), listener
  configuration (9/9), gRPC (25/25), and PDF worker (1/1), then returned
  `REVIEW_VERDICT: PASS` in
  `docs/harness/reviews/2026-07-12-engram-10-of-10-wave2-v2-post.md`.
- The previous independent-review FAIL remains preserved in orchestration
  history; the v2 artifact supersedes its verdict only for the remediated SHA.
- FreeBSD/OpenBSD were not run on this macOS host. Their shared `/dev/fd`
  recognition logic has unit coverage, and this remains a non-blocking runtime
  limitation rather than inferred platform proof.
- No push, tag, release, or real publication occurred during barrier/review.
  `cargo publish` was invoked only with `--dry-run`.

**Sprint**: Harness maintenance — live-state closeout
**Task**: harness-live-state-closeout — close completed bootstrap sprint metadata
**Date started**: 2026-06-27
**Owner**: Ronaldo + agents (Claude Code + Claude Code Sonnet reviewer)

---

## 2026-06-27 — Close stale bootstrap live state

### Contexto

The repository is back on a single clean `main` line after the lifecycle
predicate follow-up:

- `HEAD` and `origin/main` are both `1aa14e5`.
- PR #108 landed as `e156810`, and that commit is contained in current `main`.
- The only local branch at handoff was `main`; this task started
  `chore/harness-live-state` as a fresh branch.
- `docs/harness/progress.md` still presented the original
  `harness-bootstrap` sprint as active and kept stale `Last commit` metadata.

### Ações realizadas

1. Started fresh branch `chore/harness-live-state`.
2. Updated `docs/harness/progress.md` so the live state points at this
   housekeeping task instead of the completed bootstrap sprint.
3. Updated `docs/harness/SPEC.md` with matching active sprint/task/plan fields
   to preserve the `doctor.sh` drift contract.
4. Recorded `Harness Engineering v0 — bootstrap & core gates` as completed.
5. Updated live metadata to current evidence:
   - last merged main commit: `1aa14e5`;
   - latest relevant post-review: harness live-state closeout v2 PASS;
   - latest full sensors timestamp: `2026-06-27T15:07:07Z`.

### Fora de escopo

- No changes to harness scripts, gates, invariants, review policy, CI workflows,
  MCP tools, storage, SDKs, or Rust code.
- No selection of the next product follow-up; this task only makes the
  canonical harness state ready for that next branch.

### Evidência

- `rtk bash docs/harness/bin/bootstrap.sh` — PASS before the edit; confirmed
  stale active sprint and stale commit metadata.
- `rtk bash docs/harness/bin/doctor.sh` — PASS before the edit.
- `rtk bash docs/harness/bin/doctor.sh` — PASS after updating
  `SPEC.md`/`progress.md`; confirmed no drift and active plan exists.
- `rtk bash docs/harness/bin/bootstrap.sh` — PASS after the edit; active work
  now points at `harness-live-state-closeout`.
- `rtk git diff --check` — PASS.
- `rtk bash docs/harness/bin/sensors.sh` — PASS, full canonical gate
  (`fmt + clippy + test_lib + test_integration + test_integration_watch +
  wasm_target + wasm_all_targets + wasm_wasm_target + doc + ref_check +
  pr-title-policy + harness doctor`), timestamp `2026-06-27T15:04:27Z`,
  `duration_sec=151`.
- Claude Code Sonnet post-review:
  `docs/harness/reviews/2026-06-27-harness-live-state-closeout-v2-post.md`
  — `REVIEW_VERDICT: PASS docs-only housekeeping with complete gate evidence
  and correct scope boundary`.
- `rtk bash docs/harness/bin/review-gate.sh post harness-live-state-closeout --review-file docs/harness/reviews/2026-06-27-harness-live-state-closeout-v2-post.md`
  — PASS.
- `rtk bash docs/harness/bin/bootstrap.sh` — PASS after cleanup; last review
  now resolves to the Sonnet PASS artifact, not a prompt-only pre-gate copy.
- `rtk bash docs/harness/bin/sensors.sh` — PASS after final metadata updates,
  full canonical gate, timestamp `2026-06-27T15:07:07Z`, `duration_sec=29`.

## 2026-07-10 — Make live state self-checking

### Contexto

The 10/10 harness maintenance wave requires live-state metadata to be
truthful at execution time, not only historically accurate for the old
bootstrap closeout. The stale `1aa14e5` commit marker is now a regression
fixture: current live progress records approved execution baseline `843fd52`
(`843fd520cbd0eb4c2b1885fe11c997198beb2ca1`). After later Wave 0 commits,
that baseline remains valid only when the checker can bind it to the direct
live-state snapshot commit, current review artifact, and Canvas metadata;
a temp fixture carrying `1aa14e5` must fail loudly.

### Ações realizadas

1. Added `docs/harness/bin/check-live-state.sh` with explicit
   `--progress PROGRESS_PATH` fixture support.
2. Added `docs/harness/bin/test-check-live-state.sh` to drive the checker CLI
   through happy, stale-SHA, malformed-input, dirty-worktree, repeated-run, and
   misleading-success-output probes.
3. Updated `docs/harness/progress.md` with execution-time HEAD, sensor snapshot,
   checker status, and required-versus-advisory workflow reconciliation.
4. Added the Review Canvas for `engram-10-of-10-live-state`.

### Required versus advisory gate reconciliation

- Live GitHub branch-protection API contexts currently list `Format`, `Clippy`,
  `Documentation`, `Test (ubuntu-latest)`, `Security Audit`, and `Cargo Deny`
  as required.
- `.github/workflows/harness-contract.yml` defines `Harness Contract`, but the
  live `required_status_checks.contexts` receipt does not include it; required
  status is therefore not inferred from workflow text.
- `Harness Doctor Advisory` remains a non-blocking workflow job.
- Unknowns are not inferred as branch-protection truth; the GitHub API receipt
  records the live `required_status_checks.contexts` result or labels it
  unknown if unavailable.

### Evidência esperada

- `rtk bash docs/harness/bin/test-check-live-state.sh` — checker CLI regression suite.
- `rtk bash docs/harness/bin/check-live-state.sh --progress docs/harness/progress.md` — current progress matches repository facts and accepts only current HEAD, first parent, or the approved live-state baseline bound to review/Canvas/snapshot metadata.
- `rtk bash docs/harness/bin/bootstrap.sh` — live state visible to future agents.
- `rtk bash docs/harness/bin/doctor.sh` — harness consistency.
- `rtk bash docs/harness/bin/sensors.sh quick` — quick deterministic lane, not a substitute for the full lane; latest observed timestamp `2026-07-10T09:00:50Z`.
- `rtk bash docs/harness/bin/review-gate.sh post engram-10-of-10-live-state --review-file docs/harness/reviews/2026-07-10-engram-10-of-10-live-state-v4-post.md` — PASS, independent post-review gate.

### Fora de escopo

- No change to GitHub branch-protection settings.
- No fabrication of required checks from workflow names; workflow mapping is
  documented separately from the GitHub API receipt.
- No product, MCP, storage, SDK, or Rust behavior changes.

### Post-review metadata fix — 2026-07-10

Independent follow-up review found that bootstrap still selected the old
2026-06-27 docs-only review because the active task id remained
`harness-live-state-closeout`. The live task is now
`engram-10-of-10-live-state`, and `progress.md` records
`docs/harness/reviews/2026-07-10-engram-10-of-10-live-state-v4-post.md` as the
authoritative current post-review artifact. `check-live-state.sh` now validates
the `Last review` field, artifact existence, `REVIEW_VERDICT: PASS`, and task
path alignment so stale review metadata fails mechanically.

### Superseding review metadata PASS — 2026-07-10

A follow-up independent review returned `REVIEW_VERDICT: PASS` for the stale-review metadata fixes: bootstrap now surfaces the 2026-07-10 `engram-10-of-10-live-state` artifact, the checker validates `Last review` mechanically, missing `--progress` operands fail with an actionable error, and the unused test helper was removed. The authoritative artifact is `docs/harness/reviews/2026-07-10-engram-10-of-10-live-state-v4-post.md`.

### Superseding dirty-probe review PASS — 2026-07-10

A second follow-up independent review returned `REVIEW_VERDICT: PASS` for the dirty-probe and SPEC gate-label fixes. The dirty probe now uses `docs/harness/check-live-state-dirty-probe.untracked`, which is not ignored by `.gitignore`; `SPEC.md` now names `engram-10-of-10-live-state` in the expected post gate. The authoritative artifact is `docs/harness/reviews/2026-07-10-engram-10-of-10-live-state-v4-post.md`.

## 2026-07-10 — Engram 10/10 Wave 2 Task 11: fail-closed HTTP listeners

### Domain change

- Public HTTP and SSE listeners refuse startup without an API key.
- Loopback HTTP retains anonymous development access.
- Bearer authentication and principal authorization precede rate limiting and
  MCP dispatch for requests, notifications, and SSE subscriptions.
- The HTTP contract distinguishes authentication failures (`401`) from scope
  or workspace authorization failures (`403`).

### Verification

- `cargo test http_transport --lib` — PASS.
- `cargo test --test http_transport_security -- --nocapture` — PASS against
  real `engram-server` processes.
- `cargo clippy -p engram-core --all-targets -- -D warnings` — PASS.
- `cargo fmt --all -- --check` — PASS.
- `bash docs/harness/bin/doctor.sh` — PASS.
- Manual real-binary HTTP/SSE QA observed loopback anonymous `200`, public
  no-key refusal before bind, keyed missing/wrong bearer `401`, valid MCP
  initialize `200`, missing SSE bearer `401`, and valid SSE `200` with
  `text/event-stream`.

Detailed command receipts are recorded in the orchestrator-owned Task 11
evidence file under `.omo/evidence/`.

## 2026-07-12 — PR #189 CI remediation

- Fixed three Linux Clippy errors in the descriptor-bound SQLite VFS using
  platform-gated imports and checked device-id comparison.
- Replaced four hard-coded cryptographic test salts and two fixed WebSocket
  handshake nonces with runtime `OsRng` values.
- No scanner allowlist, dependency, public API, lockfile, or `unsafe` surface
  was added. Focused tests, exact CI Clippy, clean-tree Gitleaks, doctor,
  formatting, diff-check, and independent review passed.

## 2026-07-12 — Harness Contract workflow YAML repair

- The `main` push after PR #187 produced a zero-job failure for
  `.github/workflows/harness-contract.yml`.
- A local YAML parser reproduced the syntax error at the unquoted `PR_TITLE`
  expression because its fallback contains `merge-group: no PR title`.
- Quoted the complete expression without changing its runtime behavior.
- Ruby YAML parse, harness doctor, fallback title policy, and quick sensors all
  passed. Independent Sonnet review returned `REVIEW_VERDICT: PASS`.
