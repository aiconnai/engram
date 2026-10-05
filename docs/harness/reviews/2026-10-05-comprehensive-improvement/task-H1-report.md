# Task H1 report — fail-closed review gate with complete diff scope (Lane P)

Worktree `engram-improvement-lane-p`, branch `claude/improvement-lane-p`.

## Commits
- `dfd8681` feat(harness): make review gate fail-closed with explicit scope
- progress commit (lane-P log entry + `Last commit` refresh) follows; see `git log`.

## What was implemented
`docs/harness/bin/review-gate.sh` rewritten (772 lines; prompt text and doctor-grepped strings kept):
- Modes: `pre` (advisory), `scope` (new, read-only: prints shas, diff sha256, NUL-safe `CHANGED_PATH=%q`
  lines, `SKIP_ALLOWLIST`, gate script sha256), `post` (hard gate).
- Explicit diff source. Final = `--base/--head` or `--range A..B` (rejects `...`, extra `..`, empty
  side, reversed/diverged, empty range, tree-not-commit, missing commit; base must be an ancestor).
  `post` without a range is a usage error (exit 2). Preparation (`pre`, `scope --prepare`) =
  working tree vs HEAD + staged-only paths (also staged-then-deleted) + non-ignored untracked files.
- NUL-safe paths (`-z`, `--no-renames`: rename/delete list both sides). Diff bytes pinned
  (`--full-index`, myers, no ext-diff/textconv, fixed prefixes, `-O/dev/null`); sha256 of the reviewed diff.
- Any git failure -> exit 4 before the review file is opened; error text is never used as diff.
- Trusted provenance: receipt (`KEY=VALUE`, strict parser: version, task, base, head, tree, diff
  sha256, review path, review sha256, operator) via `--receipt` or `ENGRAM_REVIEW_RECEIPT`; must be an
  absolute, regular, non-symlink file owned by the operator, not group/other-writable, in a directory
  with the same property, and outside the repo, `.git`, git common dir and every `git worktree`.
  Gate recomputes base/head/tree/diff, compares with the receipt AND with operator-supplied
  `--expect-tree`, `--expect-diff-sha256`, `--operator`; review artifact path + sha256 also bound.
  Missing/untrusted/mismatching -> `PENDING`, exit 3, even with `REVIEW_VERDICT: PASS`.
- Legacy marker parser kept, slightly stricter: exactly one valid marker line; zero / several /
  prompt placeholder `<one-line summary>` -> `INVALID_REVIEW` exit 1. FAIL -> exit 1.
- Skip allowlist: only item 1 of GATES.md (`docs/**/*.md` outside `docs/harness/`, regular files,
  both sides of renames, judged on the unfiltered path list) is mechanised -> `SKIPPED_ALLOWLIST`
  (exit 0, distinct from PASS). Not widened; items 2-4 never auto-skipped. Any `docs/harness/bin/*`
  change is flagged and always needs a reviewer.
- `--repo DIR` lets the operator run a trusted copy of the gate taken from the base revision
  (a modified gate cannot authorize itself). Task id validated (`[A-Za-z0-9][A-Za-z0-9._-]{0,63}`);
  GIT_DIR/GIT_INDEX_FILE etc. unset; no /tmp path built from user input (mktemp + trap).
- Exit codes: 0 PASS/SKIPPED_ALLOWLIST/ADVISORY, 1 FAIL/INVALID_REVIEW, 2 usage, 3 PENDING,
  4 scope error. Last stdout line is `GATE_STATUS: <STATUS> reason=<code>`.

Docs: GATES.md (new subsection "Review gate fail-closed (H1)": semantics, exit-code table, operator
receipt runbook; skip-allowlist note; Camada 2 bullets), CODE_REVIEW_POLICY.md (marker contract +
"Semântica do post-gate"), README.md (modes/loop; not in the brief's file list, edited because it
documented the removed `--range=main..HEAD`/default behaviour).

`docs/harness/bin/test-review-gate.sh` (new, 24 tests / 181 assertions, mktemp caller-owned repos
and a separate operator dir, cleanup trap). Brief fixtures present verbatim: `test_stale_review_rejected`,
`test_invalid_range_rejected`, `test_missing_commit_rejected`; plus review missing, prose PASS without
marker, FAIL, marker ambiguity, staged-only (+untracked, ignored excluded, staged-then-deleted),
rename/delete both paths, odd file names (newline, space, leading dash, unicode), range covers all
commits (and ignores working-tree edits), diff hash vs independent git, docs-only skip + 14 non-skip
cases, script changes always require reviewer, modified gate cannot self-authorize, git diff failure
(blob deleted from object store) -> 4 before verdict, pre never approves, trusted vs untrusted receipt
(in repo, in .git, in sibling worktree, symlink, world-writable, relative path), env-var receipt, every
receipt field tampered, malformed receipts, operator expectations, task-id/usage validation.

## TDD evidence
- RED (test written first, old gate): `rtk proxy bash docs/harness/bin/test-review-gate.sh` -> rc=1,
  `Tests: 24  Assertions passed: 52  Assertions failed: 127`. Most failures were "unknown arg --base"
  (rc 2) because the old CLI had no explicit scope. Real behavioural RED from the old gate with the
  legacy interface: `review-gate.sh post T1 --range BASE..HEAD` on a `src/` change with no review file
  printed "For now, exiting with code 0 (no verdict available to enforce)." (fail-open), and the
  old gate's prose-PASS artifact path only failed via the legacy marker check.
- GREEN: same command after implementation -> rc=0, `Tests: 24  Assertions passed: 181  Assertions failed: 0`.
- Mutation probes on a scratch copy (all caught): pending->exit 0 (46 assertions fail), allowlist
  widened to `docs/*` (4), head-check removed (2), renames enabled (6 fail across 5 tests), ancestor check
  removed (3). Two probes (receipt-arg-empty check, marker-count check) were equivalent mutants
  (later checks still reject).

## Other verification (all after final edits)
- `rtk proxy bash docs/harness/bin/test-review-gate.sh` rc 0 (also under `/bin/bash` 3.2.57 with the
  inner `bash` forced to 3.2: 181/0).
- `rtk proxy bash docs/harness/bin/test-fixtures.sh`: Passed 15 Failed 0.
- `rtk proxy bash docs/harness/bin/test-check-live-state.sh`: PASS.
- `rtk proxy bash docs/harness/bin/doctor.sh`: `OK harness doctor`, 1 WARN (pre-existing: no review
  artifact for active task).
- `bash docs/harness/bin/check-live-state.sh --progress docs/harness/progress.md`: PASS.
- `shellcheck -x` on both scripts: clean. `check-commit-msg.sh`: OK.
- Smoke on the real repo: `review-gate.sh scope H1 --base cf6d969~2 --head cf6d969` printed a correct
  scope (5 paths, 1 excluded).

## Deviations / decisions
- README.md edited (unlisted) for accuracy. Old behaviour "post defaults to HEAD / dirty tree" and
  `post` without review exiting 0 are intentionally removed (breaking for scripts relying on the
  fail-open; none found in the repo).
- Receipt is `KEY=VALUE` (not JSON) so the gate needs no JSON parser; format is in GATES.md.
- The `--no-renames` diff shows renames as delete + add; reviewer prompt is larger for big renames.
- Allowlist is judged on the full changed-path list (including excluded bookkeeping paths), so a
  commit mixing docs md with `docs/harness/progress/*` is NOT skippable (strictest reading, not widened).
  A range whose every path is excluded (reviews/progress/target...) is an `empty-scope` scope error.
- Wiring into `sensors.sh`/`doctor.sh`/CI NOT done: the existing harness self-tests
  (`test-fixtures.sh`, `test-check-live-state.sh`) are not wired into any lane either (verified by
  grep), so it is recorded for H2's lane wiring.

## Open concerns
- Receipt is a procedural boundary: a same-uid writer can still read/write files the operator can.
  Real isolation arrives with H4/H5. The location/permission checks only catch the in-repo and
  group/other-writable cases.
- `docs/harness/reviews|progress` and `target/` etc. remain excluded from the reviewed diff
  (pre-existing); they are now listed as `EXCLUDED_PATH` in scope output and prompt but their bytes
  are not bound by the receipt.
- Gate file is 772 lines (<800) with a long prompt heredoc; splitting was avoided because doctor
  greps strings inside review-gate.sh.
- `doctor.sh` still treats any artifact with a marker as "pass" (legacy/history check); not changed.

## NOT RUN
- Independent review of H1 (controller's job); CI on GitHub; wiring into sensors/CI (H2).

---

# Fix report — review round 1 (base HEAD 48033da)

Commits: `16ecc5a` fix(harness): harden review gate against writer-controlled state;
`60343b3` docs(harness): log H1 review fix round 1 and refresh Last commit.
Files: `docs/harness/bin/review-gate.sh`, `docs/harness/bin/test-review-gate.sh`, `GATES.md`,
`CODE_REVIEW_POLICY.md`, `README.md` (+ lane-P log, `progress.md` Last commit in the second commit).

## Changes, per finding (each with fixtures written first)
1. **Case-variant receipt bypass.** Location is now compared by filesystem identity: every ancestor
   of the receipt directory (and of the gate script directory) is tested with `-ef` against the repo
   root, git dir/common dir and every `git worktree list -z` path (`inside_untrusted_root`). Also
   added: hard-linked receipt refused (`find -links +1`), and ACL handling (`ls -lde`: an ACL granting
   write/append/delete/add_* on the receipt or its directory is untrusted; deny-only ACLs, such as
   macOS home-directory defaults, are tolerated; ACLs that cannot be inspected fail closed).
   Fixtures: `test_receipt_case_variant_path_rejected` (detects a case-insensitive temp FS by
   probing; otherwise records `[NOT RUN]` with the reason; on this machine it ran: repo, `.GIT` and
   worktree case variants), `test_receipt_hardlink_into_repo_rejected`, `test_receipt_acl_rejected`
   (NOT RUN if no ACL can be created; it ran here via `chmod +a`).
2. **Writer-writable `.git` state.** `GIT_NO_REPLACE_OBJECTS=1`, `GIT_GRAFT_FILE=/dev/null`,
   `GIT_ATTR_NOSYSTEM=1` exported; `gitx` uses `--no-replace-objects`, `core.attributesFile=/dev/null`,
   `advice.graftFileDeprecated=false`; `DIFF_FLAGS` gains `--text` (defeats `-diff`/binary attributes,
   including `.git/info/attributes`) next to `--no-textconv --no-ext-diff`. Fixtures:
   `test_replace_ref_cannot_forge_scope` (replace evil->benign: scope must print the real tree/diff
   and a receipt forged from the benign values must be PENDING), `test_graft_cannot_alter_ancestry`
   (git 2.55 still honours `info/grafts`), `test_attributes_cannot_hide_diff` (info/attributes
   `* -diff`, in-tree `.gitattributes -diff`, textconv driver). The independent-hash test now includes `--text`.
3. **Gate identity.** Receipt has a new required key `GATE_SCRIPT_SHA256`; new
   `--expect-gate-sha256` (missing -> `operator-expectations-missing`; mismatch ->
   `receipt-gate-mismatch` / `operator-expectation-mismatch`, PENDING). A PASS from a gate script
   living inside any worktree of the judged repo is refused (`gate-untrusted-location`). The test
   helper `gate()` now runs a trusted copy from the operator dir with `--repo`; `gate_in_repo()` is the
   writer-side copy. `test_modified_gate_cannot_authorize_itself` rewritten: base holds the OLD gate,
   candidate an approve-all gate; in-repo copy approves itself (precondition), trusted base copy gives
   PENDING, a receipt binding the tampered gate's hash is PENDING, correct binding PASSes. New
   `test_gate_inside_worktree_cannot_pass`. GATES.md runbook now ALWAYS runs an external gate copy
   with `--repo` (step 0 copies it from the base and records its sha256; receipt and expectation
   carry it); README/POLICY updated. GATES.md also records that H1 itself is accepted by the
   independent SDD review + owner decision (base `cf6d969` has no receipts).
4. **Minor.** `node_modules/*` (root and `sdks/typescript`) and `sdks/python/__pycache__/*` dropped from
   exclusions (`test_node_modules_is_reviewed`); staged-only loop uses `":(literal)$p"`
   (`test_staged_only_pathspec_is_literal`: a glob-named ghost no longer pulls in `gx.rs`, whose diff
   previously appeared twice); `mkdir`/prompt write/`cp` failures exit 4 `io-failure`
   (`test_prompt_write_failure_is_nonzero`, NOT RUN when dir perms are not enforced, e.g. root); prep mode
   appends the staged diff (with a `# staged (index) content for ...` marker) when index differs from the
   working tree (`test_prepare_shows_staged_and_worktree_content`).

## TDD evidence
- RED #1 (new fixtures for items 1, 2, 4 only, pre-fix gate): `bash docs/harness/bin/test-review-gate.sh`
  -> rc=1, `Tests: 34  Assertions passed: 188  Assertions failed: 23  Not run: 0`. Failures, all behavioural:
  case-variant receipt, hard-linked receipt and ACL receipt returned rc 0 (PASS); replace-ref scope printed the
  benign tree/diff and the forged receipt passed; graft made a valid range rc 4; `-diff` attributes yielded
  "Binary files differ"; node_modules not in scope; staged content missing; prompt write failure rc 0; glob
  pathspec duplicated `gx.rs`.
- RED #2 (gate-identity tests, pre-implementation): rc=1, `Tests: 35 ... Assertions failed: 88` (the old gate
  rejects `--expect-gate-sha256` and the `GATE_SCRIPT_SHA256` key).
- GREEN: `bash docs/harness/bin/test-review-gate.sh` -> rc=0, `Tests: 35  Assertions passed: 224  Assertions failed: 0  Not run: 0`;
  same under `/bin/bash` 3.2.57 (inner `bash` forced to 3.2).
- Mutation probes (all caught): identity check disabled (7 failures: case-variant + in-worktree gate tests),
  `--text` removed (attributes test). `GIT_NO_REPLACE_OBJECTS` alone removed is covered by the redundant
  `--no-replace-objects` (equivalent mutant).

## Other verification (after final edits)
`shellcheck -x` on both scripts: clean. `rtk proxy bash docs/harness/bin/doctor.sh`: OK (same pre-existing WARN).
`test-fixtures.sh`: Passed 15, Failed 0. `test-check-live-state.sh`: PASS. `check-live-state.sh`: PASS.

## Remaining limits / notes
- The gate still runs `git` against a writer-editable `.git/config` (filter/fsmonitor commands); documented in GATES.md as an
  H4/H5 isolation item. `.gitignore`/`info/exclude` hide files from preparation scope (documented).
- `--text` makes binary changes appear as raw patch bytes in the prompt (visibility over readability).
- Wiring of `test-review-gate.sh` into sensors/CI still deferred to H2.
