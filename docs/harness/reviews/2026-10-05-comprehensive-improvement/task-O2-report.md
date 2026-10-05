# Report — O2 Git hygiene and recoverable retention (Lane P)

Worktree/branch: engram-improvement-lane-p / claude/improvement-lane-p. Local commits only.

## Commits
- a40430d feat(harness): add git retention policy, manifest and restore tool
- e6807e3 / 695b553 docs(harness): O2 progress + last-commit refreshes
- 980737f chore(harness): wire retention tests into the offline lane (after H5 committed; re-read first)
- 19a6218 docs(harness): record O2 lane wiring and refresh last commit

## What was done
- Read-only inventory (`retention-manifest.py inventory`, 2026-10-05T15:07:31Z, HEAD f96b200): 1219 tracked files / 31,366,068 B;
  review-raw 73 files / 4,249,732 B at HEAD, 57 distinct blobs in history, 739,705 B packed; count-objects: 5862 loose,
  36.06 MiB, size-pack 13.47 MiB, prune-packable 4464, 1 garbage (64 B, worktree refs dir; recorded only); refs shared:
  59 heads/59 remotes/10 tags/76 other, 13 worktrees, 1640 reflog entries; dev/bench* (gh-pages) history 145 MB uncompressed / 1.6 MB packed.
  Sensitivity: 7/73 .raw flagged by flag name only (4 e-mail, 3 abs home path, 1 credential heuristic); no provider tokens/keys. Content never printed.
- docs/OPERATIONS_GIT_RETENTION.md (PT-BR): per-class owner/duration/storage+hash/lookup/restore; O1 operational logs = "pending O1 implementation";
  destructive ops listed as prohibited without separate authorization; no MB promises.
- docs/harness/bin/retention-manifest.py (stdlib): inventory, generate, verify (0/1/4), backup (sha256 content-addressed), verify-backup,
  restore (--backup/--from-git, hash before write, atomic, never overwrites differing file), untrack (dry-run default, needs verified backup,
  only `git rm --cached` + ignore rule, no delete/commit/history change). Manifest committed: docs/harness/retention/review-raw.manifest.json (73 entries).
- docs/harness/tests/test_retention.py (21 tests, hermetic) wired as lane component `retention` (floor 21) + doctor wiring + test_offline_lane (+1 test).

## Class migrated (disposable clones only): review-raw (.raw prompt dumps)
Clone A: verify --strict OK, backup (63 distinct objects for 73 files), verify-backup OK, untrack --apply, commit. Fresh clone B (no .raw):
verify=MISSING x73 exit 4; doctor OK; check-doc-links PASS; run-offline-lane PASS (components=9 checks=401 at f96b200 base; again components=9 checks=401 on HEAD a40430d-era clone);
test_retention 21 OK without the .raw files; restore --backup -> 73 restored, verify OK, `diff -r docs/harness/reviews` identical to source, git status clean;
second clone restore --from-git identical. Measurement: checkout -4,190,592 B (-4,249,732 raw +tool/manifest); size-pack 6290->6178 KiB is packing noise,
not attributable and not promised (blobs remain in history). REAL REPO: nothing untracked, .gitignore unchanged, no object/ref touched.

## TDD
RED: `python3 -m unittest discover -s docs/harness/tests -p test_retention.py` before the tool existed: Ran 21, FAILED (failures=18, errors=2) (script missing, exit 2).
GREEN: Ran 21 tests OK (also OK inside a clone without .raw files).

## Final verification (real worktree, after H5 landed)
- `bash docs/harness/bin/doctor.sh` -> OK harness doctor (usual WARN: no review artifact for active task).
- `bash docs/harness/bin/run-offline-lane.sh` -> OFFLINE_LANE: PASS components=11 checks=460 (retention 21/21, merge_gate 36/36, lane_contract 16).
- check-doc-links PASS for OPERATIONS_GIT_RETENTION.md, context-budget.md, progress.md, lane-p log (lane R log allowed-missing as in H6).

## Open / NOT RUN
- Owner decision pending: apply untrack + `docs/harness/reviews/*.raw` ignore rule on the real branch (changes what review-gate.sh leaves for commit;
  stop concurrent writers first). No persistent real backup was created (proof backup was in a temp dir).
- Durations in the policy (e.g. backup >= 12 months) are proposals for owner ratification.
- sensors.sh, real CI, independent review (controller): NOT RUN. `check-live-state.sh` strict still fails on Last sensors (pre-existing, unrelated).
- Manual review of 7 flagged .raw paths recommended before any public release.
- O1 log class: pending O1 implementation.
