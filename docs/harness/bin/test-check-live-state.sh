#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

CHECKER="docs/harness/bin/check-live-state.sh"
PROGRESS="docs/harness/progress.md"
TMP_DIR="$(mktemp -d)"
DIRTY_PROBE="docs/harness/check-live-state-dirty-probe.untracked"

cleanup() {
  rm -rf "$TMP_DIR"
  rm -f "$DIRTY_PROBE"
}
trap cleanup EXIT

assert_contains() {
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

CURRENT_OUTPUT="$(run_expect_success "current progress passes" bash "$CHECKER" --progress "$PROGRESS")"
assert_contains "$CURRENT_OUTPUT" "PASS live state matches current repository facts" "happy path reports PASS"

MISSING_OPERAND_OUTPUT="$(run_expect_failure "missing progress operand fails" bash "$CHECKER" --progress)"
assert_contains "$MISSING_OPERAND_OUTPUT" "ERROR --progress requires PROGRESS_PATH" "missing operand is actionable"

REPEAT_OUTPUT="$(run_expect_success "repeat check passes" bash "$CHECKER" --progress "$PROGRESS")"
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

MALFORMED_FIXTURE="$TMP_DIR/malformed-progress.md"
printf '# malformed\n\nNo live-state table here.\n' > "$MALFORMED_FIXTURE"
MALFORMED_OUTPUT="$(run_expect_failure "malformed fixture fails" bash "$CHECKER" --progress "$MALFORMED_FIXTURE")"
assert_contains "$MALFORMED_OUTPUT" "missing required field: Last commit" "malformed fixture reports missing field"
assert_contains "$MALFORMED_OUTPUT" "remediation: restore the progress live-state field table" "malformed fixture gives remediation"
assert_not_contains "$MALFORMED_OUTPUT" "PASS live state matches current repository facts" "malformed failure does not print misleading PASS"

printf 'dirty probe\n' > "$DIRTY_PROBE"
DIRTY_OUTPUT="$(run_expect_success "dirty worktree probe remains diagnostic" bash "$CHECKER" --progress "$PROGRESS")"
assert_contains "$DIRTY_OUTPUT" "worktree_status=dirty" "dirty worktree is reported explicitly"
rm -f "$DIRTY_PROBE"

echo "PASS check-live-state regression suite"
