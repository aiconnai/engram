#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

CHECKER="docs/harness/bin/check-live-state.sh"
REAL_PROGRESS="docs/harness/progress.md"
TMP_DIR="$(mktemp -d)"
PROGRESS="$TMP_DIR/progress-synced.md"
DIRTY_PROBE="docs/harness/check-live-state-dirty-probe.untracked"
ASSERTIONS=0

cleanup() {
  rm -rf "$TMP_DIR"
  rm -f "$DIRTY_PROBE"
}
trap cleanup EXIT

assert_contains() {
  ASSERTIONS=$((ASSERTIONS + 1))
  local haystack="$1"
  local needle="$2"
  local label="$3"

  case "$haystack" in
    *"$needle"*) ;;
    *)
      printf 'FAIL: %s\nmissing: %s\noutput:\n%s\n' "$label" "$needle" "$haystack" >&2
      exit 1
      ;;
  esac
}

assert_not_contains() {
  ASSERTIONS=$((ASSERTIONS + 1))
  local haystack="$1"
  local needle="$2"
  local label="$3"

  case "$haystack" in
    *"$needle"*)
      printf 'FAIL: %s\nunexpected: %s\noutput:\n%s\n' "$label" "$needle" "$haystack" >&2
      exit 1
      ;;
  esac
}

run_expect_success() {
  local label="$1"
  shift
  local output

  if ! output="$("$@" 2>&1)"; then
    printf 'FAIL: %s\noutput:\n%s\n' "$label" "$output" >&2
    exit 1
  fi
  printf '%s' "$output"
}

run_expect_failure() {
  local label="$1"
  shift
  local output

  set +e
  output="$("$@" 2>&1)"
  local status=$?
  set -e

  if [ "$status" -eq 0 ]; then
    printf 'FAIL: %s unexpectedly succeeded\noutput:\n%s\n' "$label" "$output" >&2
    exit 1
  fi
  printf '%s' "$output"
}

# The suite also must not depend on telemetry freshness: every sensors.sh run rewrites the tracked
# docs/harness/.sensors-last, and the real progress.md "Last sensors" row only matches it right after
# the progress file was refreshed. $PROGRESS is therefore a copy of the live progress file whose
# "Last sensors" row is synced to the current .sensors-last; all other rows are the real ones.
python3 - "$REAL_PROGRESS" "$PROGRESS" docs/harness/.sensors-last <<'PY'
import re
import sys
from pathlib import Path

source = Path(sys.argv[1]).read_text()
fields = dict(
    line.split("=", 1) for line in Path(sys.argv[3]).read_text().splitlines() if "=" in line
)
row = "| Last sensors | `status={status} mode={mode} timestamp={timestamp} (fixture synced to .sensors-last)` |".format(
    status=fields["status"], mode=fields["mode"], timestamp=fields["timestamp"]
)
synced, count = re.subn(r"(?m)^\| Last sensors \|.*\|$", lambda _m: row, source)
assert count == 1, "Last sensors row not found exactly once"
Path(sys.argv[2]).write_text(synced)
PY

# The regression suite must not depend on which commit docs/harness/progress.md currently names:
# on a CI pull_request checkout HEAD is a synthetic merge commit. The happy path therefore runs on
# a copy of the live progress file whose Last commit is the checked-out HEAD; every other
# live-state row (reconciliation table, workflow names, sensors, review, plan) is the real one.
CURRENT_FIXTURE="$TMP_DIR/current-progress.md"
python3 - "$PROGRESS" "$CURRENT_FIXTURE" "$(git rev-parse --short HEAD)" <<'PY'
import re
import sys
from pathlib import Path

source = Path(sys.argv[1]).read_text()
current = re.sub(r"(\| Last commit \| `)[^`]+(` \|)", rf"\g<1>{sys.argv[3]}\2", source)
Path(sys.argv[2]).write_text(current)
PY

CURRENT_OUTPUT="$(run_expect_success "current progress passes" bash "$CHECKER" --progress "$CURRENT_FIXTURE")"
assert_contains "$CURRENT_OUTPUT" "head=$(git rev-parse --short HEAD)" "happy path reports the current head"
assert_contains "$CURRENT_OUTPUT" "PASS live state matches current repository facts" "happy path reports PASS"

# The approved-baseline path (Last commit is neither HEAD nor its parent) is exercised on a hermetic
# synthetic repository, so it no longer depends on which commit the live progress.md names.
FIXTURE_REPO="$TMP_DIR/baseline-repo"
build_approved_baseline_repo() {
  local repo="$1"
  local g=(git -C "$repo" -c user.name=fixture -c user.email=fixture@example.test -c commit.gpgsign=false -c core.hooksPath=/dev/null)
  local review="docs/harness/reviews/2026-07-10-engram-10-of-10-live-state-v4-post.md"
  local canvas="docs/harness/canvas/2026-07-09-engram-10-of-10-live-state.md"
  mkdir -p "$repo/docs/harness/bin" "$repo/docs/harness/reviews" "$repo/docs/harness/canvas" \
    "$repo/docs/harness/progress" "$repo/.github/workflows"
  git -C "$repo" init -q -b main
  cp "$REPO_ROOT/docs/harness/bin/check-live-state.sh" "$REPO_ROOT/docs/harness/bin/lib.sh" "$repo/docs/harness/bin/"
  printf 'plan\n' >"$repo/docs/harness/progress/plan.md"
  printf 'REVIEW_VERDICT: PASS fixture\n' >"$repo/$review"
  printf 'status=pass\nmode=quick\ntimestamp=2026-01-01T00:00:00Z\n' >"$repo/docs/harness/.sensors-last"
  printf 'name: Format\nname: Clippy\nname: Test (ubuntu-latest)\nname: Documentation\nname: Security Audit\nname: Cargo Deny\n' \
    >"$repo/.github/workflows/ci.yml"
  printf 'name: Harness Contract\nname: Harness Doctor Advisory\n' >"$repo/.github/workflows/harness-contract.yml"
  printf 'canvas\n' >"$repo/docs/harness/canvas/old.md"
  printf 'progress v0\n' >"$repo/docs/harness/progress.md"
  printf 'closeout\n' >"$repo/docs/harness/progress/2026-06-27-harness-live-state-closeout.md"
  printf '#!/usr/bin/env bash\n' >"$repo/docs/harness/bin/test-check-live-state.sh"
  "${g[@]}" add -A
  "${g[@]}" commit -q -m "baseline"
  local baseline
  baseline="$("${g[@]}" rev-parse HEAD)"
  printf 'baseline %s\n' "$baseline" >"$repo/$canvas"
  printf 'progress v1 %s\n' "$baseline" >"$repo/docs/harness/progress.md"
  printf 'closeout v1\n' >"$repo/docs/harness/progress/2026-06-27-harness-live-state-closeout.md"
  printf 'review v1\nREVIEW_VERDICT: PASS fixture\n' >"$repo/$review"
  printf '# touched\n' >>"$repo/docs/harness/bin/check-live-state.sh"
  printf '# touched\n' >>"$repo/docs/harness/bin/test-check-live-state.sh"
  "${g[@]}" add -A
  "${g[@]}" commit -q -m "docs(harness): make live state verifiable"
  local snapshot
  snapshot="$("${g[@]}" rev-parse HEAD)"
  cat >"$repo/docs/harness/progress.md" <<EOF
| Last review | \`2026-07-10 - pass: $review\` |
| Last sensors | \`status=pass mode=quick 2026-01-01T00:00:00Z\` |
| Last commit | \`$baseline\` |
| Last live-state check | \`check-live-state.sh --progress docs/harness/progress.md\` |
| Active plan | \`docs/harness/progress/plan.md\` |
- **Approved execution baseline**: \`$baseline\`
- **Approved live-state snapshot commit**: \`$snapshot\`
| \`Format\` | branch-required | \`.github/workflows/ci.yml\` |
| \`Clippy\` | branch-required | \`.github/workflows/ci.yml\` |
| \`Test (ubuntu-latest)\` | branch-required | \`.github/workflows/ci.yml\` |
| \`Documentation\` | branch-required | \`.github/workflows/ci.yml\` |
| \`Security Audit\` | branch-required | \`.github/workflows/ci.yml\` |
| \`Cargo Deny\` | branch-required | \`.github/workflows/ci.yml\` |
| \`Harness Contract\` | not in \`required_status_checks.contexts\` | \`.github/workflows/harness-contract.yml\` |
| \`Harness Doctor Advisory\` | advisory workflow job | \`.github/workflows/harness-contract.yml\` |
EOF
  "${g[@]}" add -A
  "${g[@]}" commit -q -m "docs(harness): record approved baseline"
  FIXTURE_BASELINE="$baseline"
  FIXTURE_SNAPSHOT="$snapshot"
}
mkdir -p "$FIXTURE_REPO"
build_approved_baseline_repo "$FIXTURE_REPO"
BASELINE_OUTPUT="$(cd "$FIXTURE_REPO" && run_expect_success "approved baseline fixture passes" bash docs/harness/bin/check-live-state.sh --progress docs/harness/progress.md)"
assert_contains "$BASELINE_OUTPUT" "approved_baseline=$FIXTURE_BASELINE" "approved baseline fixture reports the baseline"
assert_contains "$BASELINE_OUTPUT" "snapshot_commit=$(git -C "$FIXTURE_REPO" rev-parse --short "$FIXTURE_SNAPSHOT")" "approved baseline fixture reports the snapshot commit"
assert_contains "$BASELINE_OUTPUT" "PASS live state matches current repository facts" "approved baseline fixture reports PASS"

MISSING_OPERAND_OUTPUT="$(run_expect_failure "missing progress operand fails" bash "$CHECKER" --progress)"
assert_contains "$MISSING_OPERAND_OUTPUT" "ERROR --progress requires PROGRESS_PATH" "missing operand is actionable"

REPEAT_OUTPUT="$(run_expect_success "repeat check passes" bash "$CHECKER" --progress "$CURRENT_FIXTURE")"
assert_contains "$REPEAT_OUTPUT" "PASS live state matches current repository facts" "repeat run reports PASS"

PARENT_FIXTURE="$TMP_DIR/parent-progress.md"
PARENT_HEAD="$(git rev-parse HEAD^)"
python3 - "$PROGRESS" "$PARENT_FIXTURE" "$PARENT_HEAD" <<'PY'
import re
import sys
from pathlib import Path

source = Path(sys.argv[1]).read_text()
parent = re.sub(r"(\| Last commit \| `)[^`]+(` \|)", rf"\g<1>{sys.argv[3]}\2", source)
Path(sys.argv[2]).write_text(parent)
PY

PARENT_OUTPUT="$(run_expect_success "parent HEAD fixture passes" bash "$CHECKER" --progress "$PARENT_FIXTURE")"
assert_contains "$PARENT_OUTPUT" "PASS live state matches current repository facts" "parent fixture simulates post-commit pass"

STALE_FIXTURE="$TMP_DIR/stale-progress.md"
python3 - "$PROGRESS" "$STALE_FIXTURE" <<'PY'
import re
import sys
from pathlib import Path

source = Path(sys.argv[1]).read_text()
stale = re.sub(r"(\| Last commit \| `)[^`]+(` \|)", r"\g<1>1aa14e5\2", source)
Path(sys.argv[2]).write_text(stale)
PY

STALE_OUTPUT="$(run_expect_failure "stale HEAD fixture fails" bash "$CHECKER" --progress "$STALE_FIXTURE")"
assert_contains "$STALE_OUTPUT" "stale Last commit: found 1aa14e5" "stale fixture names stale SHA"
assert_contains "$STALE_OUTPUT" "remediation: update Last commit in $STALE_FIXTURE" "stale fixture gives remediation"
assert_not_contains "$STALE_OUTPUT" "PASS live state matches current repository facts" "failure does not print misleading PASS"

STALE_REVIEW_FIXTURE="$TMP_DIR/stale-review-progress.md"
python3 - "$PROGRESS" "$STALE_REVIEW_FIXTURE" <<'PY'
import re
import sys
from pathlib import Path

source = Path(sys.argv[1]).read_text()
stale = re.sub(
    r"(\| Last review \| `)[^`]+(` \|)",
    r"\g<1>2026-06-27 — pass: docs/harness/reviews/2026-06-27-harness-live-state-closeout-v2-post.md\2",
    source,
)
Path(sys.argv[2]).write_text(stale)
PY

STALE_REVIEW_OUTPUT="$(run_expect_failure "stale review fixture fails" bash "$CHECKER" --progress "$STALE_REVIEW_FIXTURE")"
assert_contains "$STALE_REVIEW_OUTPUT" "Last review artifact is stale for this task" "stale review is rejected"
assert_not_contains "$STALE_REVIEW_OUTPUT" "PASS live state matches current repository facts" "stale review failure does not print misleading PASS"

SUPERSEDED_REVIEW_FIXTURE="$TMP_DIR/superseded-review-progress.md"
python3 - "$PROGRESS" "$SUPERSEDED_REVIEW_FIXTURE" <<'PY'
import sys
from pathlib import Path

source = Path(sys.argv[1]).read_text()
superseded = source.replace(
    "docs/harness/reviews/2026-07-10-engram-10-of-10-live-state-v4-post.md",
    "docs/harness/reviews/2026-07-10-engram-10-of-10-live-state-v2-post.md",
)
Path(sys.argv[2]).write_text(superseded)
PY

SUPERSEDED_REVIEW_OUTPUT="$(run_expect_failure "superseded same-task review fixture fails" bash "$CHECKER" --progress "$SUPERSEDED_REVIEW_FIXTURE")"
assert_contains "$SUPERSEDED_REVIEW_OUTPUT" "Last review is not authoritative current review" "superseded same-task review is rejected"
assert_not_contains "$SUPERSEDED_REVIEW_OUTPUT" "PASS live state matches current repository facts" "superseded review failure does not print misleading PASS"

MALFORMED_FIXTURE="$TMP_DIR/malformed-progress.md"
printf '# malformed\n\nNo live-state table here.\n' > "$MALFORMED_FIXTURE"
MALFORMED_OUTPUT="$(run_expect_failure "malformed fixture fails" bash "$CHECKER" --progress "$MALFORMED_FIXTURE")"
assert_contains "$MALFORMED_OUTPUT" "missing required field: Last commit" "malformed fixture reports missing field"
assert_contains "$MALFORMED_OUTPUT" "remediation: restore the progress live-state field table" "malformed fixture gives remediation"
assert_not_contains "$MALFORMED_OUTPUT" "PASS live state matches current repository facts" "malformed failure does not print misleading PASS"

printf 'dirty probe\n' > "$DIRTY_PROBE"
DIRTY_OUTPUT="$(run_expect_success "dirty worktree probe remains diagnostic" bash "$CHECKER" --progress "$CURRENT_FIXTURE")"
assert_contains "$DIRTY_OUTPUT" "worktree_status=dirty" "dirty worktree is reported explicitly"
rm -f "$DIRTY_PROBE"

# --- Structural mode (task H6) -------------------------------------------------------------------
# `--structural` is what doctor.sh enforces on the real docs/harness/progress.md. It must fail closed on
# real drift (missing fields/plan/reconciliation rows, a Last commit that is not an ancestor of HEAD) and
# must NOT depend on per-commit or per-sensors-run churn (Last commit == HEAD/parent, .sensors-last
# timestamp). The strict mode above stays unchanged and is the one that closes tasks.
STRUCT_REPO="$TMP_DIR/structural-repo"
STRUCT_REVIEW="docs/harness/reviews/2026-07-10-engram-10-of-10-live-state-v4-post.md"
build_structural_repo() {
  local repo="$1"
  local g=(git -C "$repo" -c user.name=fixture -c user.email=fixture@example.test -c commit.gpgsign=false -c core.hooksPath=/dev/null)
  mkdir -p "$repo/docs/harness/bin" "$repo/docs/harness/reviews" "$repo/docs/harness/progress" "$repo/.github/workflows"
  git -C "$repo" init -q -b main
  cp "$REPO_ROOT/docs/harness/bin/check-live-state.sh" "$REPO_ROOT/docs/harness/bin/lib.sh" "$repo/docs/harness/bin/"
  printf 'plan\n' >"$repo/docs/harness/progress/plan.md"
  printf 'REVIEW_VERDICT: PASS fixture\n' >"$repo/$STRUCT_REVIEW"
  printf 'status=pass\nmode=quick\ntimestamp=2026-01-01T00:00:00Z\n' >"$repo/docs/harness/.sensors-last"
  printf 'name: Format\nname: Clippy\nname: Test (ubuntu-latest)\nname: Documentation\nname: Security Audit\nname: Cargo Deny\n' \
    >"$repo/.github/workflows/ci.yml"
  printf 'name: Harness Contract\nname: Harness Doctor Advisory\n' >"$repo/.github/workflows/harness-contract.yml"
  printf 'one\n' >"$repo/docs/harness/file.txt"
  "${g[@]}" add -A
  "${g[@]}" commit -q -m "c1"
  STRUCT_C1="$("${g[@]}" rev-parse HEAD)"
  printf 'two\n' >>"$repo/docs/harness/file.txt"
  "${g[@]}" commit -q -am "c2"
  printf 'three\n' >>"$repo/docs/harness/file.txt"
  "${g[@]}" commit -q -am "c3"
  git -C "$repo" checkout -q -b side "$STRUCT_C1"
  printf 'side\n' >"$repo/docs/harness/side.txt"
  "${g[@]}" add docs/harness/side.txt
  "${g[@]}" commit -q -m "side"
  STRUCT_SIDE="$("${g[@]}" rev-parse HEAD)"
  git -C "$repo" checkout -q main
}
write_structural_progress() {
  # $1 = output path, $2 = Last commit value, $3 = Last sensors value
  cat >"$1" <<EOF
| Last review | \`2026-07-10 - pass: $STRUCT_REVIEW\` |
| Last sensors | \`$3\` |
| Last commit | \`$2\` |
| Last live-state check | \`check-live-state.sh --progress docs/harness/progress.md\` |
| Active plan | \`docs/harness/progress/plan.md\` |
| \`Format\` | branch-required | \`.github/workflows/ci.yml\` |
| \`Clippy\` | branch-required | \`.github/workflows/ci.yml\` |
| \`Test (ubuntu-latest)\` | branch-required | \`.github/workflows/ci.yml\` |
| \`Documentation\` | branch-required | \`.github/workflows/ci.yml\` |
| \`Security Audit\` | branch-required | \`.github/workflows/ci.yml\` |
| \`Cargo Deny\` | branch-required | \`.github/workflows/ci.yml\` |
| \`Harness Contract\` | not in \`required_status_checks.contexts\` | \`.github/workflows/harness-contract.yml\` |
| \`Harness Doctor Advisory\` | advisory workflow job | \`.github/workflows/harness-contract.yml\` |
EOF
}
mkdir -p "$STRUCT_REPO"
build_structural_repo "$STRUCT_REPO"
SENSORS_OK="status=pass mode=quick 2026-01-01T00:00:00Z"

# The real live summary must pass structurally (this is the enforcement that doctor.sh runs).
REAL_STRUCT_OUTPUT="$(run_expect_success "structural mode passes on the real progress.md" bash "$CHECKER" --progress "$REAL_PROGRESS" --structural)"
assert_contains "$REAL_STRUCT_OUTPUT" "mode=structural" "structural output names its mode"
assert_contains "$REAL_STRUCT_OUTPUT" "PASS live state structure is consistent (structural mode" "structural PASS is labelled and never the strict PASS text"
assert_not_contains "$REAL_STRUCT_OUTPUT" "PASS live state matches current repository facts" "structural PASS cannot be mistaken for the strict PASS"

# An older ancestor is fine: no per-commit churn.
STRUCT_PROGRESS="$TMP_DIR/struct-ancestor.md"
write_structural_progress "$STRUCT_PROGRESS" "$STRUCT_C1" "$SENSORS_OK"
OUT="$(run_expect_success "older ancestor passes structurally" bash -c "cd '$STRUCT_REPO' && bash docs/harness/bin/check-live-state.sh --progress '$STRUCT_PROGRESS' --structural")"
assert_contains "$OUT" "ancestor_check=ancestor" "ancestor verdict is reported"
OUT="$(run_expect_failure "the same older ancestor is stale in strict mode" bash -c "cd '$STRUCT_REPO' && bash docs/harness/bin/check-live-state.sh --progress '$STRUCT_PROGRESS'")"
assert_contains "$OUT" "stale Last commit: found" "strict mode keeps rejecting non-HEAD/parent commits"

# Sensors telemetry drift does not matter structurally (it changes on every sensors run) but does strictly.
STRUCT_PROGRESS="$TMP_DIR/struct-sensors.md"
write_structural_progress "$STRUCT_PROGRESS" "$STRUCT_C1" "status=fail mode=full 1999-01-01T00:00:00Z"
OUT="$(run_expect_success "sensors telemetry drift is ignored structurally" bash -c "cd '$STRUCT_REPO' && bash docs/harness/bin/check-live-state.sh --progress '$STRUCT_PROGRESS' --structural")"
assert_contains "$OUT" "PASS live state structure is consistent" "sensors drift still passes structurally"

# A well-formed Last commit that HEAD cannot reach is NOT a structural failure: squash and rebase merges
# legitimately rewrite SHAs, and the required CI job runs this on the real progress.md. It is a visible
# warning instead (ancestor_check=unreachable); `--require-ancestor` makes it a hard failure (hermetic use).
STRUCT_PROGRESS="$TMP_DIR/struct-unknown.md"
write_structural_progress "$STRUCT_PROGRESS" "1aa14e5" "$SENSORS_OK"
OUT="$(run_expect_success "unknown but well-formed Last commit only warns structurally" bash -c "cd '$STRUCT_REPO' && bash docs/harness/bin/check-live-state.sh --progress '$STRUCT_PROGRESS' --structural")"
assert_contains "$OUT" "ancestor_check=unreachable" "unreachable commit is reported"
assert_contains "$OUT" "WARN Last commit 1aa14e5 is not reachable from HEAD" "unreachable commit prints a visible warning"
assert_contains "$OUT" "PASS live state structure is consistent" "unreachable commit does not fail the structural check"
OUT="$(run_expect_failure "unknown Last commit fails with --require-ancestor" bash -c "cd '$STRUCT_REPO' && bash docs/harness/bin/check-live-state.sh --progress '$STRUCT_PROGRESS' --structural --require-ancestor")"
assert_contains "$OUT" "Last commit 1aa14e5 is not an ancestor of HEAD" "hard ancestry mode names the failure"
assert_not_contains "$OUT" "PASS live state" "hard ancestry failure prints no PASS"

STRUCT_PROGRESS="$TMP_DIR/struct-diverged.md"
write_structural_progress "$STRUCT_PROGRESS" "$STRUCT_SIDE" "$SENSORS_OK"
OUT="$(run_expect_failure "diverged Last commit fails with --require-ancestor" bash -c "cd '$STRUCT_REPO' && bash docs/harness/bin/check-live-state.sh --progress '$STRUCT_PROGRESS' --structural --require-ancestor")"
assert_contains "$OUT" "is not an ancestor of HEAD" "a commit outside HEAD history is rejected in hard mode"

# Squash merge: the pre-squash commit still exists in the object database but is not in HEAD's history.
g_sq=(git -C "$STRUCT_REPO" -c user.name=fixture -c user.email=fixture@example.test -c commit.gpgsign=false -c core.hooksPath=/dev/null)
git -C "$STRUCT_REPO" checkout -q -b squashed main
"${g_sq[@]}" merge -q --squash side >/dev/null 2>&1
"${g_sq[@]}" commit -q -m "squash side"
STRUCT_PROGRESS="$TMP_DIR/struct-squash.md"
write_structural_progress "$STRUCT_PROGRESS" "$STRUCT_SIDE" "$SENSORS_OK"
OUT="$(run_expect_success "squash-merged history warns instead of failing" bash -c "cd '$STRUCT_REPO' && bash docs/harness/bin/check-live-state.sh --progress '$STRUCT_PROGRESS' --structural")"
assert_contains "$OUT" "ancestor_check=unreachable" "squash merge is reported as unreachable"
assert_contains "$OUT" "squash or rebase merges rewrite SHAs" "the warning explains the legitimate cause"
git -C "$STRUCT_REPO" checkout -q main

# A malformed id is still a hard failure, with or without full history.
STRUCT_PROGRESS="$TMP_DIR/struct-malformed.md"
write_structural_progress "$STRUCT_PROGRESS" "not-a-sha" "$SENSORS_OK"
OUT="$(run_expect_failure "malformed Last commit fails structurally" bash -c "cd '$STRUCT_REPO' && bash docs/harness/bin/check-live-state.sh --progress '$STRUCT_PROGRESS' --structural")"
assert_contains "$OUT" "is not a commit id" "malformed id is a hard failure on full history"

STRUCT_PROGRESS="$TMP_DIR/struct-noplan.md"
write_structural_progress "$STRUCT_PROGRESS" "$STRUCT_C1" "$SENSORS_OK"
sed -i.bak 's#docs/harness/progress/plan.md#docs/harness/progress/missing-plan.md#' "$STRUCT_PROGRESS"
OUT="$(run_expect_failure "missing active plan fails structurally" bash -c "cd '$STRUCT_REPO' && bash docs/harness/bin/check-live-state.sh --progress '$STRUCT_PROGRESS' --structural")"
assert_contains "$OUT" "active plan does not exist" "missing plan is reported"

STRUCT_PROGRESS="$TMP_DIR/struct-norow.md"
write_structural_progress "$STRUCT_PROGRESS" "$STRUCT_C1" "$SENSORS_OK"
grep -v 'Cargo Deny' "$STRUCT_PROGRESS" >"$STRUCT_PROGRESS.tmp" && mv "$STRUCT_PROGRESS.tmp" "$STRUCT_PROGRESS"
OUT="$(run_expect_failure "missing reconciliation row fails structurally" bash -c "cd '$STRUCT_REPO' && bash docs/harness/bin/check-live-state.sh --progress '$STRUCT_PROGRESS' --structural")"
assert_contains "$OUT" "missing progress reconciliation: Cargo Deny branch-required row" "dropped reconciliation row is reported"

STRUCT_PROGRESS="$TMP_DIR/struct-nofield.md"
write_structural_progress "$STRUCT_PROGRESS" "$STRUCT_C1" "$SENSORS_OK"
grep -v 'Last review' "$STRUCT_PROGRESS" >"$STRUCT_PROGRESS.tmp" && mv "$STRUCT_PROGRESS.tmp" "$STRUCT_PROGRESS"
OUT="$(run_expect_failure "missing required field fails structurally" bash -c "cd '$STRUCT_REPO' && bash docs/harness/bin/check-live-state.sh --progress '$STRUCT_PROGRESS' --structural")"
assert_contains "$OUT" "missing required field: Last review" "missing field is reported"

# A shallow clone (typical CI checkout) cannot prove ancestry: say so explicitly instead of passing silently.
SHALLOW_REPO="$TMP_DIR/shallow-repo"
git clone -q --depth 1 --no-local "file://$STRUCT_REPO" "$SHALLOW_REPO" 2>/dev/null
STRUCT_PROGRESS="$TMP_DIR/struct-shallow.md"
write_structural_progress "$STRUCT_PROGRESS" "$STRUCT_C1" "$SENSORS_OK"
OUT="$(run_expect_success "shallow clone skips ancestry visibly" bash -c "cd '$SHALLOW_REPO' && bash docs/harness/bin/check-live-state.sh --progress '$STRUCT_PROGRESS' --structural")"
assert_contains "$OUT" "ancestor_check=skipped-shallow" "shallow clone reports the skipped ancestry check"
assert_contains "$OUT" "WARN ancestry not verified (shallow clone)" "shallow clone prints a visible warning, never a silent pass"
STRUCT_PROGRESS="$TMP_DIR/struct-shallow-garbage.md"
write_structural_progress "$STRUCT_PROGRESS" "not-a-sha" "$SENSORS_OK"
OUT="$(run_expect_failure "shallow clone still rejects a malformed Last commit" bash -c "cd '$SHALLOW_REPO' && bash docs/harness/bin/check-live-state.sh --progress '$STRUCT_PROGRESS' --structural")"
assert_contains "$OUT" "is not a commit id" "shallow clone validates the commit id format"

echo "PASS check-live-state regression suite (assertions: $ASSERTIONS)"
