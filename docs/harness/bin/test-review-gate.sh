#!/usr/bin/env bash
# docs/harness/bin/test-review-gate.sh
#
# Regression suite for docs/harness/bin/review-gate.sh (task H1: fail-closed review
# gate with complete diff scope).
#
# Every fixture builds a caller-owned temporary git repository (mktemp, removed on
# exit) and a separate "operator" directory that stands in for the location outside
# any writer-writable worktree where the human operator keeps the review receipt.
# The gate under test is a copy of the script in this directory; override with
# REVIEW_GATE_UNDER_TEST=/path/to/review-gate.sh to test another copy.
#
# Exit 0 only when every assertion passes. Nothing here touches the real repository.

set -u

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GATE_SRC="${REVIEW_GATE_UNDER_TEST:-${SCRIPT_DIR}/review-gate.sh}"

# Isolate from ambient git state (hooks export GIT_INDEX_FILE and friends).
for _v in $(env | sed -n 's/^\(GIT_[A-Za-z0-9_]*\)=.*/\1/p'); do unset "$_v"; done
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1
export GIT_AUTHOR_NAME=fixture GIT_AUTHOR_EMAIL=fixture@example.test
export GIT_COMMITTER_NAME=fixture GIT_COMMITTER_EMAIL=fixture@example.test

if [ ! -f "$GATE_SRC" ]; then
  echo "FATAL: gate under test not found: $GATE_SRC" >&2
  exit 2
fi

TMP="$(mktemp -d "${TMPDIR:-/tmp}/review-gate-test.XXXXXX")"
TMP="$(cd "$TMP" && pwd -P)"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

PASS_COUNT=0
FAIL_COUNT=0
TESTS_RUN=0
NOT_RUN_COUNT=0
NOT_APPLICABLE_COUNT=0
FIX_N=0
CURRENT_TEST=""

# Exit codes documented in review-gate.sh and GATES.md.
RC_PASS=0 RC_FAIL=1 RC_USAGE=2 RC_PENDING=3 RC_SCOPE=4

TASK="H1-fixture"
OPERATOR="operator@example.test"

ok() { PASS_COUNT=$((PASS_COUNT + 1)); }
bad() {
  FAIL_COUNT=$((FAIL_COUNT + 1))
  printf '  [FAIL] %s: %s\n' "$CURRENT_TEST" "$1" >&2
  if [ -n "${OUT:-}" ]; then printf '    --- gate output (rc=%s) ---\n%s\n    ---\n' "${RC:-?}" "$OUT" >&2; fi
}

# A scenario that cannot occur on this platform (not a missing capability): the attack surface it
# probes does not exist, e.g. case-variant paths on a case-sensitive filesystem. Reported
# separately and never counted as a pass; genuinely unavailable capabilities use not_run.
not_applicable() { # label reason
  NOT_APPLICABLE_COUNT=$((NOT_APPLICABLE_COUNT + 1))
  printf '  [N/A] %s: %s\n' "$1" "$2"
}
not_run() { # label reason — never counted as a pass
  NOT_RUN_COUNT=$((NOT_RUN_COUNT + 1))
  printf '  [NOT RUN] %s: %s\n' "$1" "$2"
}
expect_rc() { # expected-rc label
  if [ "$RC" -eq "$1" ]; then ok; else bad "$2: expected rc=$1, got rc=$RC"; fi
}
expect_nonzero() {
  if [ "$RC" -ne 0 ]; then ok; else bad "$1: expected nonzero rc, got 0"; fi
}
expect_out() { # needle label
  case "$OUT" in *"$1"*) ok ;; *) bad "$2: output lacks '$1'" ;; esac
}
expect_not_out() { # needle label
  case "$OUT" in *"$1"*) bad "$2: output unexpectedly contains '$1'" ;; *) ok ;; esac
}
expect_status() { expect_out "GATE_STATUS: $1" "$2"; }

# --- fixture plumbing ------------------------------------------------------

g() { git -C "$REPO" -c commit.gpgsign=false -c core.hooksPath=/dev/null "$@"; }

# The operator's trusted gate copy lives outside the repository and is pointed at it with --repo.
gate() {
  OUT="$(cd "$OPS" && bash "$OPS/trusted/review-gate.sh" "$@" --repo "$REPO" 2>&1)"
  RC=$?
}
# The writer-side copy inside the worktree (never trusted for a PASS).
gate_in_repo() {
  OUT="$(cd "$OPS" && bash "$REPO/docs/harness/bin/review-gate.sh" "$@" 2>&1)"
  RC=$?
}

new_fixture() {
  FIX_N=$((FIX_N + 1))
  local d="$TMP/fx$FIX_N"
  REPO="$d/repo"
  OPS="$d/operator"
  mkdir -p "$REPO" "$OPS"
  chmod 700 "$OPS"
  git -C "$REPO" init -q -b main
  mkdir -p "$REPO/src" "$REPO/docs/harness/bin" "$REPO/docs/harness/reviews" "$REPO/docs/harness/progress"
  cp "$GATE_SRC" "$REPO/docs/harness/bin/review-gate.sh"
  chmod +x "$REPO/docs/harness/bin/review-gate.sh"
  mkdir -p "$OPS/trusted"
  cp "$GATE_SRC" "$OPS/trusted/review-gate.sh"
  GATE_SHA="$(sha256_of "$OPS/trusted/review-gate.sh")"
  printf 'root readme\n' >"$REPO/README.md"
  printf 'fn a() {}\n' >"$REPO/src/lib.rs"
  printf 'guide\n' >"$REPO/docs/guide.md"
  printf 'plain\n' >"$REPO/docs/note.txt"
  printf 'gates\n' >"$REPO/docs/harness/GATES.md"
  printf '#!/bin/sh\necho tool\n' >"$REPO/docs/harness/bin/tool.sh"
  printf 'ignored.log\n' >"$REPO/.gitignore"
  g add -A
  g commit -q -m base
  BASE="$(g rev-parse HEAD)"
  HEAD_SHA="$BASE"
}

commit_all() { # message
  g add -A
  g commit -q -m "$1"
  HEAD_SHA="$(g rev-parse HEAD)"
}

# Parse KEY=VALUE lines the gate prints on stdout.
out_val() { printf '%s\n' "$OUT" | sed -n "s/^$1=//p" | head -1; }

compute_scope() { # sets TREE DIFF from gate `scope` (final mode BASE..HEAD_SHA)
  gate scope "$TASK" --base "$BASE" --head "$HEAD_SHA"
  TREE="$(out_val TREE_SHA)"
  DIFF="$(out_val DIFF_SHA256)"
}

sha256_of() { if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi; }

write_review() { # file verdict-line-or-text...
  local f="$1"
  shift
  printf '%s\n' "$@" >"$f"
}

# write_receipt FILE [KEY=VALUE ...]  — fields default to the correct bindings
write_receipt() {
  local f="$1"
  shift
  local r_task="$TASK" r_base="$BASE" r_head="$HEAD_SHA" r_tree="$TREE" r_diff="$DIFF"
  local r_review="$REVIEW" r_rsha r_op="$OPERATOR" r_gate="$GATE_SHA" kv
  local r_version=1 r_reviewer="rev-fixture" r_policy="harness-hardening-v1"
  local r_context="fresh-readonly"
  r_rsha="$(sha256_of "$REVIEW")"
  for kv in "$@"; do
    case "$kv" in
      TASK_ID=*) r_task="${kv#*=}" ;;
      BASE_SHA=*) r_base="${kv#*=}" ;;
      HEAD_SHA=*) r_head="${kv#*=}" ;;
      TREE_SHA=*) r_tree="${kv#*=}" ;;
      DIFF_SHA256=*) r_diff="${kv#*=}" ;;
      REVIEW_ARTIFACT=*) r_review="${kv#*=}" ;;
      REVIEW_SHA256=*) r_rsha="${kv#*=}" ;;
      OPERATOR=*) r_op="${kv#*=}" ;;
      GATE_SCRIPT_SHA256=*) r_gate="${kv#*=}" ;;
      RECEIPT_VERSION=*) r_version="${kv#*=}" ;;
      REVIEWER=*) r_reviewer="${kv#*=}" ;;
      POLICY_VERSION=*) r_policy="${kv#*=}" ;;
      REVIEW_CONTEXT=*) r_context="${kv#*=}" ;;
    esac
  done
  {
    echo "RECEIPT_VERSION=$r_version"
    echo "TASK_ID=$r_task"
    echo "BASE_SHA=$r_base"
    echo "HEAD_SHA=$r_head"
    echo "TREE_SHA=$r_tree"
    echo "DIFF_SHA256=$r_diff"
    echo "REVIEW_ARTIFACT=$r_review"
    echo "REVIEW_SHA256=$r_rsha"
    echo "OPERATOR=$r_op"
    echo "GATE_SCRIPT_SHA256=$r_gate"
    if [ "$r_version" = "2" ]; then
      echo "REVIEWER=$r_reviewer"
      echo "POLICY_VERSION=$r_policy"
      echo "REVIEW_CONTEXT=$r_context"
    fi
  } >"$f"
}

# Review PASS + trusted receipt for the current BASE..HEAD_SHA range.
make_pass_case() {
  compute_scope
  REVIEW="$OPS/review-pass.md"
  RECEIPT="$OPS/receipt.env"
  write_review "$REVIEW" "PASS no issues in scope" "" "- [LOW] No issues found." "REVIEW_VERDICT: PASS no issues in scope"
  write_receipt "$RECEIPT"
}

run_post() { # extra args appended (later flags override earlier ones)
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA" --review-file "$REVIEW" \
    --receipt "$RECEIPT" --expect-tree "$TREE" --expect-diff-sha256 "$DIFF" \
    --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA" "$@"
}

# A simple implementation change that needs a reviewer.
change_src() {
  printf 'fn a() { 1 }\n' >"$REPO/src/lib.rs"
  commit_all "change src"
}

# --- tests -----------------------------------------------------------------

test_review_missing_is_pending() {
  new_fixture
  change_src
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_rc $RC_PENDING "no review file at all"
  expect_status PENDING "no review file at all"
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA" --review-file "$OPS/does-not-exist.md"
  expect_rc $RC_PENDING "review file path missing"
  expect_not_out "GATE_STATUS: PASS" "missing review must not read as PASS"
}

test_prose_pass_without_marker_rejected() {
  new_fixture
  change_src
  compute_scope
  REVIEW="$OPS/prose.md"
  RECEIPT="$OPS/receipt.env"
  write_review "$REVIEW" "PASS looks great, ship it" "- [LOW] No issues found."
  write_receipt "$RECEIPT"
  run_post
  expect_rc $RC_FAIL "prose PASS without REVIEW_VERDICT marker"
  expect_not_out "GATE_STATUS: PASS" "prose PASS"
}

test_fail_verdict_rejected() {
  new_fixture
  change_src
  compute_scope
  REVIEW="$OPS/fail.md"
  RECEIPT="$OPS/receipt.env"
  write_review "$REVIEW" "FAIL blocker found" "- [BLOCKER] x" "REVIEW_VERDICT: FAIL blocker found"
  write_receipt "$RECEIPT"
  run_post
  expect_rc $RC_FAIL "FAIL verdict with valid receipt"
  expect_status FAIL "FAIL verdict"
  # FAIL stays FAIL even without any receipt.
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA" --review-file "$REVIEW"
  expect_rc $RC_FAIL "FAIL verdict without receipt"
}

test_marker_ambiguity_rejected() {
  new_fixture
  change_src
  compute_scope
  RECEIPT="$OPS/receipt.env"
  REVIEW="$OPS/dup.md"
  write_review "$REVIEW" "REVIEW_VERDICT: PASS first" "REVIEW_VERDICT: PASS second"
  write_receipt "$RECEIPT"
  run_post
  expect_rc $RC_FAIL "two marker lines"
  REVIEW="$OPS/mixed.md"
  write_review "$REVIEW" "REVIEW_VERDICT: PASS fine" "REVIEW_VERDICT: FAIL actually not"
  write_receipt "$RECEIPT"
  run_post
  expect_rc $RC_FAIL "PASS and FAIL markers together"
  REVIEW="$OPS/placeholder.md"
  write_review "$REVIEW" "REVIEW_VERDICT: PASS <one-line summary>"
  write_receipt "$RECEIPT"
  run_post
  expect_rc $RC_FAIL "unfilled prompt placeholder marker"
}

test_pass_with_trusted_receipt_accepted() {
  new_fixture
  change_src
  make_pass_case
  run_post
  expect_rc $RC_PASS "valid receipt + PASS marker"
  expect_status PASS "valid receipt + PASS marker"
}

test_pass_marker_without_receipt_is_pending() {
  new_fixture
  change_src
  make_pass_case
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA" --review-file "$REVIEW" \
    --expect-tree "$TREE" --expect-diff-sha256 "$DIFF" --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA"
  expect_rc $RC_PENDING "PASS marker but no --receipt/env"
  expect_status PENDING "PASS marker but no receipt"
  expect_not_out "GATE_STATUS: PASS" "marker alone is history only"
  # Missing receipt file path.
  run_post --receipt "$OPS/nope.env"
  expect_rc $RC_PENDING "receipt path does not exist"
}

test_untrusted_receipt_rejected() {
  new_fixture
  change_src
  make_pass_case
  # (a) receipt inside the repository
  cp "$RECEIPT" "$REPO/receipt.env"
  run_post --receipt "$REPO/receipt.env"
  expect_rc $RC_PENDING "receipt inside repo"
  expect_out "receipt-untrusted-location" "receipt inside repo reason"
  # (b) receipt inside the repo's .git directory
  cp "$RECEIPT" "$REPO/.git/receipt.env"
  run_post --receipt "$REPO/.git/receipt.env"
  expect_rc $RC_PENDING "receipt inside .git"
  # (c) receipt inside another worktree of the same repository
  g worktree add -q -b wt-branch "$TMP/wt$FIX_N" "$BASE"
  cp "$RECEIPT" "$TMP/wt$FIX_N/receipt.env"
  run_post --receipt "$TMP/wt$FIX_N/receipt.env"
  expect_rc $RC_PENDING "receipt inside sibling worktree"
  expect_out "receipt-untrusted-location" "sibling worktree reason"
  # (d) symlink at a trusted path pointing into the repo
  ln -s "$REPO/receipt.env" "$OPS/link.env"
  run_post --receipt "$OPS/link.env"
  expect_rc $RC_PENDING "receipt is a symlink"
  # (e) group/other-writable receipt
  cp "$RECEIPT" "$OPS/ww.env"
  chmod 666 "$OPS/ww.env"
  run_post --receipt "$OPS/ww.env"
  expect_rc $RC_PENDING "world-writable receipt"
  expect_out "receipt-untrusted-permissions" "world-writable reason"
  # (f) relative receipt path is refused (ambiguous authority)
  (cd "$OPS" && cp "$RECEIPT" rel.env)
  run_post --receipt "rel.env"
  expect_nonzero "relative receipt path"
  # sanity: the same bytes at the trusted path pass.
  run_post
  expect_rc $RC_PASS "control: trusted location passes"
}

test_receipt_env_var_accepted() {
  new_fixture
  change_src
  make_pass_case
  OUT="$(cd "$OPS" && ENGRAM_REVIEW_RECEIPT="$RECEIPT" bash "$OPS/trusted/review-gate.sh" post "$TASK" --repo "$REPO" \
    --base "$BASE" --head "$HEAD_SHA" --review-file "$REVIEW" --expect-tree "$TREE" \
    --expect-diff-sha256 "$DIFF" --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA" 2>&1)"
  RC=$?
  expect_rc $RC_PASS "receipt via ENGRAM_REVIEW_RECEIPT"
}

test_receipt_field_mismatch_rejected() {
  new_fixture
  change_src
  make_pass_case
  local bogus40 bogus64 field val
  bogus40="0123456789012345678901234567890123456789"
  bogus64="0123456789012345678901234567890123456789012345678901234567890123"
  for field in TASK_ID BASE_SHA HEAD_SHA TREE_SHA DIFF_SHA256 REVIEW_ARTIFACT REVIEW_SHA256 OPERATOR GATE_SCRIPT_SHA256; do
    case "$field" in
      TASK_ID) val="OTHER-TASK" ;;
      DIFF_SHA256 | REVIEW_SHA256 | GATE_SCRIPT_SHA256) val="$bogus64" ;;
      REVIEW_ARTIFACT) val="$OPS/some-other-review.md" ;;
      OPERATOR) val="intruder@example.test" ;;
      *) val="$bogus40" ;;
    esac
    write_receipt "$RECEIPT" "$field=$val"
    run_post
    expect_rc $RC_PENDING "receipt $field tampered"
    expect_status PENDING "receipt $field tampered"
  done
  # Malformed receipts: unknown key, duplicate key, missing key, garbage line.
  write_receipt "$RECEIPT"
  printf 'EXTRA=1\n' >>"$RECEIPT"
  run_post
  expect_rc $RC_PENDING "unknown receipt key"
  write_receipt "$RECEIPT"
  printf 'OPERATOR=%s\n' "$OPERATOR" >>"$RECEIPT"
  run_post
  expect_rc $RC_PENDING "duplicate receipt key"
  write_receipt "$RECEIPT"
  grep -v '^TREE_SHA=' "$RECEIPT" >"$RECEIPT.tmp" && mv "$RECEIPT.tmp" "$RECEIPT"
  run_post
  expect_rc $RC_PENDING "missing receipt key"
  : >"$RECEIPT"
  run_post
  expect_rc $RC_PENDING "empty receipt"
  # Control.
  write_receipt "$RECEIPT"
  run_post
  expect_rc $RC_PASS "control: correct receipt passes"
}

test_operator_expectations_enforced() {
  new_fixture
  change_src
  make_pass_case
  run_post --expect-tree "0123456789012345678901234567890123456789"
  expect_rc $RC_PENDING "operator expects different tree"
  run_post --expect-diff-sha256 "0123456789012345678901234567890123456789012345678901234567890123"
  expect_rc $RC_PENDING "operator expects different diff hash"
  run_post --operator "someone-else@example.test"
  expect_rc $RC_PENDING "operator identity differs from receipt"
  run_post --expect-gate-sha256 "0123456789012345678901234567890123456789012345678901234567890123"
  expect_rc $RC_PENDING "operator expects a different gate script"
  expect_out "operator-expectation-mismatch" "gate expectation mismatch reason"
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA" --review-file "$REVIEW" --receipt "$RECEIPT" \
    --expect-tree "$TREE" --expect-diff-sha256 "$DIFF" --operator "$OPERATOR"
  expect_rc $RC_PENDING "operator supplied no gate expectation"
  expect_out "operator-expectations-missing" "missing gate expectation reason"
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA" --review-file "$REVIEW" --receipt "$RECEIPT"
  expect_rc $RC_PENDING "operator supplied no expected values"
  expect_out "operator-expectations-missing" "missing expectations reason"
}

test_stale_review_rejected() {
  new_fixture
  change_src
  make_pass_case
  run_post
  expect_rc $RC_PASS "control before staleness"
  # Reuse the same PASS + receipt after changing a single byte in a new commit.
  printf 'fn a() { 2 }\n' >"$REPO/src/lib.rs"
  commit_all "one byte later"
  gate post "$TASK" --base "$BASE" --head HEAD --review-file "$REVIEW" --receipt "$RECEIPT" \
    --expect-tree "$TREE" --expect-diff-sha256 "$DIFF" --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA"
  expect_rc $RC_PENDING "stale PASS reused after a one-byte change"
  expect_not_out "GATE_STATUS: PASS" "stale review"
  # Even if the operator updates the expected values, the receipt no longer matches.
  compute_scope
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA" --review-file "$REVIEW" --receipt "$RECEIPT" \
    --expect-tree "$TREE" --expect-diff-sha256 "$DIFF" --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA"
  expect_rc $RC_PENDING "receipt not re-issued for the new head"
  # Altering the review artifact by a byte after the receipt was issued.
  git -C "$REPO" reset -q --hard "$BASE"
  change_src
  make_pass_case
  printf ' ' >>"$REVIEW"
  run_post
  expect_rc $RC_PENDING "review artifact changed after receipt"
  expect_out "receipt-review-hash-mismatch" "review artifact hash reason"
}

test_invalid_range_rejected() {
  new_fixture
  change_src
  make_pass_case
  local r
  # Otherwise-valid PASS + receipt: only the range can be what blocks the gate.
  gate post "$TASK" --range "$BASE..$HEAD_SHA" --review-file "$REVIEW" --receipt "$RECEIPT" \
    --expect-tree "$TREE" --expect-diff-sha256 "$DIFF" --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA"
  expect_rc $RC_PASS "control: explicit --range A..B"
  for r in "$BASE..$HEAD_SHA..$BASE" "$BASE...$HEAD_SHA" "$HEAD_SHA..$BASE" "$HEAD_SHA..$HEAD_SHA" "..$HEAD_SHA" "$BASE.." "HEAD"; do
    gate post "$TASK" --range "$r" --review-file "$REVIEW" --receipt "$RECEIPT" \
      --expect-tree "$TREE" --expect-diff-sha256 "$DIFF" --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA"
    expect_rc $RC_SCOPE "invalid range '$r'"
    expect_not_out "GATE_STATUS: PASS" "invalid range '$r'"
    expect_not_out "Verdict marker" "verdict must not be consulted for range '$r'"
  done
  # Diverged (base not an ancestor of head).
  g checkout -q -b side "$BASE"
  printf 'side\n' >"$REPO/src/side.rs"
  commit_all "side"
  local side="$HEAD_SHA"
  gate post "$TASK" --base "$side" --head "$(g rev-parse main)" --review-file "$REVIEW" --receipt "$RECEIPT" \
    --expect-tree "$TREE" --expect-diff-sha256 "$DIFF" --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA"
  expect_rc $RC_SCOPE "diverged base/head"
  # No scope at all: never guess the last commit.
  gate post "$TASK" --review-file "$REVIEW" --receipt "$RECEIPT" --expect-tree "$TREE" \
    --expect-diff-sha256 "$DIFF" --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA"
  expect_rc $RC_USAGE "post without explicit range"
  gate post "$TASK" --base "$BASE"
  expect_rc $RC_USAGE "post with --base only"
}

test_missing_commit_rejected() {
  new_fixture
  change_src
  make_pass_case
  local missing="0123456789abcdef0123456789abcdef01234567" tree_obj
  gate post "$TASK" --base "$BASE" --head "$missing" --review-file "$REVIEW" --receipt "$RECEIPT" \
    --expect-tree "$TREE" --expect-diff-sha256 "$DIFF" --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA"
  expect_rc $RC_SCOPE "candidate commit does not exist"
  gate post "$TASK" --base "$missing" --head "$HEAD_SHA" --review-file "$REVIEW" --receipt "$RECEIPT" \
    --expect-tree "$TREE" --expect-diff-sha256 "$DIFF" --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA"
  expect_rc $RC_SCOPE "base commit does not exist"
  gate post "$TASK" --base "$BASE" --head "no-such-branch" --review-file "$REVIEW" --receipt "$RECEIPT" \
    --expect-tree "$TREE" --expect-diff-sha256 "$DIFF" --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA"
  expect_rc $RC_SCOPE "candidate ref does not exist"
  tree_obj="$(g rev-parse "$HEAD_SHA^{tree}")"
  gate post "$TASK" --base "$BASE" --head "$tree_obj" --review-file "$REVIEW" --receipt "$RECEIPT" \
    --expect-tree "$TREE" --expect-diff-sha256 "$DIFF" --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA"
  expect_rc $RC_SCOPE "candidate is a tree, not a commit"
  expect_not_out "GATE_STATUS: PASS" "missing commit"
}

test_git_diff_failure_rejected_before_verdict() {
  new_fixture
  change_src
  make_pass_case
  run_post
  expect_rc $RC_PASS "control before sabotage"
  # Corrupt the object database: commit and tree resolve, the blob is gone, so
  # `git diff` fails while rev-parse still succeeds.
  local blob obj
  blob="$(g rev-parse "$HEAD_SHA:src/lib.rs")"
  obj="$REPO/.git/objects/${blob%"${blob#??}"}/${blob#??}"
  rm -f "$obj"
  run_post
  expect_rc $RC_SCOPE "git diff failure is a scope error"
  expect_not_out "GATE_STATUS: PASS" "git diff failure"
  expect_not_out "Verdict marker" "verdict must not be consulted after diff failure"
  # The failure text must never be embedded as if it were reviewed diff.
  gate pre "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_nonzero "pre with unreadable diff"
  expect_not_out "GATE_STATUS: ADVISORY" "pre with unreadable diff"
}

test_staged_only_path_included() {
  new_fixture
  printf 'staged\n' >"$REPO/src/staged.rs"
  g add src/staged.rs
  printf 'untracked\n' >"$REPO/src/untracked.rs"
  printf 'ignored\n' >"$REPO/ignored.log"
  printf 'fn a() { 3 }\n' >"$REPO/src/lib.rs" # unstaged modification
  gate scope "$TASK" --prepare
  expect_rc $RC_PASS "scope --prepare"
  expect_out "CHANGED_PATH=src/staged.rs" "staged-only path listed"
  expect_out "CHANGED_PATH=src/untracked.rs" "untracked path listed"
  expect_out "CHANGED_PATH=src/lib.rs" "unstaged path listed"
  expect_not_out "ignored.log" "gitignored file stays out of scope"
  gate pre "$TASK"
  expect_rc 0 "pre in preparation mode"
  expect_out "(3 paths;" "pre scope summary counts staged, unstaged and untracked paths"
  local raw="" f
  for f in "$REPO"/docs/harness/reviews/*-"$TASK"*-pre.md.raw; do [ -f "$f" ] && raw="$f" && break; done
  if [ -n "$raw" ] && grep -q '+++ b/src/staged.rs' "$raw" && grep -q '+++ b/src/untracked.rs' "$raw"; then ok; else bad "prompt diff lacks staged-only/untracked content ($raw)"; fi
  # Staged, then removed from disk: the working-tree diff cannot see it, the index still does.
  new_fixture
  printf 'ghost\n' >"$REPO/src/ghost.rs"
  g add src/ghost.rs
  rm "$REPO/src/ghost.rs"
  gate scope "$TASK" --prepare
  expect_out "CHANGED_PATH=src/ghost.rs" "staged path missing from working tree still listed"
  gate pre "$TASK"
  raw=""
  for f in "$REPO"/docs/harness/reviews/*-"$TASK"*-pre.md.raw; do [ -f "$f" ] && raw="$f" && break; done
  if [ -n "$raw" ] && grep -q '+++ b/src/ghost.rs' "$raw"; then ok; else bad "prompt diff lacks staged-then-deleted content"; fi
  # Staged-only deletion and staged rename both sides.
  new_fixture
  g rm -q src/lib.rs
  gate scope "$TASK" --prepare
  expect_out "CHANGED_PATH=src/lib.rs" "staged deletion listed"
  new_fixture
  g mv src/lib.rs src/moved.rs
  gate scope "$TASK" --prepare
  expect_out "CHANGED_PATH=src/lib.rs" "staged rename old path"
  expect_out "CHANGED_PATH=src/moved.rs" "staged rename new path"
}

test_pre_is_advisory_and_never_approves() {
  new_fixture
  change_src
  gate pre "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_rc 0 "pre exits 0"
  expect_status ADVISORY "pre status line"
  expect_not_out "GATE_STATUS: PASS" "pre must never print PASS"
  # Even with a PASS artifact and receipt available, pre approves nothing.
  make_pass_case
  gate pre "$TASK" --base "$BASE" --head "$HEAD_SHA" --review-file "$REVIEW" --receipt "$RECEIPT"
  expect_not_out "GATE_STATUS: PASS" "pre with PASS artifact"
}

test_rename_and_delete_both_paths() {
  new_fixture
  g mv src/lib.rs src/renamed.rs
  printf 'x\n' >"$REPO/src/gone.rs"
  commit_all "rename + add"
  local mid="$HEAD_SHA"
  g rm -q src/gone.rs
  commit_all "delete"
  gate scope "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_rc $RC_PASS "scope for rename/delete range"
  expect_out "CHANGED_PATH=src/lib.rs" "rename old side"
  expect_out "CHANGED_PATH=src/renamed.rs" "rename new side"
  gate scope "$TASK" --base "$BASE" --head "$mid"
  expect_out "CHANGED_PATH=src/gone.rs" "added file present in intermediate head"
  gate scope "$TASK" --base "$mid" --head "$HEAD_SHA"
  expect_out "CHANGED_PATH=src/gone.rs" "deleted path listed"
}

test_odd_filenames_nul_safe() {
  new_fixture
  local nl="docs/we"$'\n'"ird.md" sp="src/with space.rs" dash="-dash.rs" uni="src/café.rs"
  printf 'a\n' >"$REPO/$nl"
  printf 'b\n' >"$REPO/$sp"
  printf 'c\n' >"$REPO/$dash"
  printf 'd\n' >"$REPO/$uni"
  commit_all "odd names"
  gate scope "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_rc $RC_PASS "scope with odd file names"
  local p
  for p in "$nl" "$sp" "$dash" "$uni"; do
    expect_out "$(printf 'CHANGED_PATH=%q' "$p")" "odd path listed: $(printf '%q' "$p")"
  done
  expect_out "CHANGED_PATH_COUNT=4" "odd names counted individually"
  # A newline inside a docs path must not make src/ paths look allowlisted.
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_rc $RC_PENDING "odd names with src content need review"
  # Rename of an odd path lists both sides.
  g mv "$sp" "src/with  two spaces.rs"
  commit_all "rename odd"
  gate scope "$TASK" --base "$HEAD_SHA~1" --head "$HEAD_SHA"
  expect_out "$(printf 'CHANGED_PATH=%q' "$sp")" "odd rename old side"
  expect_out "$(printf 'CHANGED_PATH=%q' "src/with  two spaces.rs")" "odd rename new side"
}

test_range_covers_all_commits() {
  new_fixture
  printf '1\n' >"$REPO/src/one.rs"
  commit_all "c1"
  printf '2\n' >"$REPO/src/two.rs"
  commit_all "c2"
  printf '3\n' >"$REPO/src/three.rs"
  commit_all "c3"
  gate scope "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_out "CHANGED_PATH=src/one.rs" "first commit covered"
  expect_out "CHANGED_PATH=src/two.rs" "middle commit covered"
  expect_out "CHANGED_PATH=src/three.rs" "last commit covered"
  expect_out "CHANGED_PATH_COUNT=3" "exactly the three commits"
  gate scope "$TASK" --range "$BASE..$HEAD_SHA~1"
  expect_not_out "CHANGED_PATH=src/three.rs" "range excludes later commit"
  # A working-tree edit must not leak into final-mode scope.
  gate scope "$TASK" --base "$BASE" --head "$HEAD_SHA"
  local clean_diff
  clean_diff="$(out_val DIFF_SHA256)"
  printf 'dirty\n' >"$REPO/src/dirty.rs"
  printf 'changed\n' >"$REPO/src/one.rs"
  g add src/one.rs
  gate scope "$TASK" --base "$BASE" --head "$HEAD_SHA"
  if [ "$(out_val DIFF_SHA256)" = "$clean_diff" ]; then ok; else bad "final-mode scope changed because of working-tree edits"; fi
}

test_relative_gate_path_hashes_the_gate_itself() {
  new_fixture
  change_src
  # Invoked by a relative path from the operator directory: the gate cds into the repository
  # before hashing itself, so the path must be made absolute first.
  OUT="$(cd "$OPS" && bash trusted/review-gate.sh scope "$TASK" --base "$BASE" --head "$HEAD_SHA" --repo "$REPO" 2>&1)"
  RC=$?
  expect_rc $RC_PASS "scope via a relative gate path"
  if [ "$(out_val GATE_SCRIPT_SHA256)" = "$GATE_SHA" ]; then ok; else bad "GATE_SCRIPT_SHA256 ($(out_val GATE_SCRIPT_SHA256)) != sha256 of the invoked gate ($GATE_SHA)"; fi
}

test_scope_hash_matches_independent_git() {
  new_fixture
  change_src
  printf 'more\n' >"$REPO/src/more.rs"
  commit_all "more"
  gate scope "$TASK" --base "$BASE" --head "$HEAD_SHA"
  local indep
  indep="$(g diff --no-color --no-ext-diff --no-textconv --text --no-renames --full-index --unified=0 \
    --diff-algorithm=myers --ignore-submodules=none --src-prefix=a/ --dst-prefix=b/ "$BASE" "$HEAD_SHA" | {
    if command -v sha256sum >/dev/null 2>&1; then sha256sum; else shasum -a 256; fi
  } | cut -d' ' -f1)"
  if [ "$(out_val DIFF_SHA256)" = "$indep" ]; then ok; else bad "DIFF_SHA256 ($(out_val DIFF_SHA256)) != independent git diff hash ($indep)"; fi
  if [ "$(out_val TREE_SHA)" = "$(g rev-parse "$HEAD_SHA^{tree}")" ]; then ok; else bad "TREE_SHA does not match candidate tree"; fi
  if [ "$(out_val BASE_SHA)" = "$BASE" ] && [ "$(out_val HEAD_SHA)" = "$HEAD_SHA" ]; then ok; else bad "BASE_SHA/HEAD_SHA not resolved correctly"; fi
}

test_docs_only_skip_allowlist_not_widened() {
  new_fixture
  # Docs-only change under docs/**/*.md outside docs/harness: skippable.
  printf 'guide v2\n' >"$REPO/docs/guide.md"
  mkdir -p "$REPO/docs/sub"
  printf 'new\n' >"$REPO/docs/sub/new.md"
  commit_all "docs only"
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_rc $RC_PASS "docs-only allowlisted change"
  expect_status SKIPPED_ALLOWLIST "docs-only allowlisted change"
  expect_not_out "GATE_STATUS: PASS" "skip is distinguishable from PASS"
  # Deleting/renaming within allowlisted docs stays skippable.
  g mv docs/guide.md docs/guide2.md
  commit_all "docs rename"
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_status SKIPPED_ALLOWLIST "docs rename + delete"

  local label
  # Each of these must NOT be skippable.
  skip_denied() { # label  (fixture already holds the change in BASE..HEAD_SHA)
    gate post "$TASK" --base "$BASE" --head "$HEAD_SHA"
    expect_rc $RC_PENDING "not skippable: $1"
    expect_not_out "SKIPPED_ALLOWLIST" "not skippable: $1"
  }
  label="docs/harness/*.md"
  new_fixture; printf 'g2\n' >"$REPO/docs/harness/GATES.md"; commit_all x; skip_denied "$label"
  label="docs/harness/bin script"
  new_fixture; printf '#!/bin/sh\necho 2\n' >"$REPO/docs/harness/bin/tool.sh"; commit_all x; skip_denied "$label"
  label="docs non-md"
  new_fixture; printf 'p2\n' >"$REPO/docs/note.txt"; commit_all x; skip_denied "$label"
  label="root README.md"
  new_fixture; printf 'r2\n' >"$REPO/README.md"; commit_all x; skip_denied "$label"
  label="src rs"
  new_fixture; change_src; skip_denied "$label"
  label="docs md + src"
  new_fixture; printf 'g2\n' >"$REPO/docs/guide.md"; printf 'fn a() { 9 }\n' >"$REPO/src/lib.rs"; commit_all x; skip_denied "$label"
  label="docs md + harness progress (excluded from diff but not allowlisted)"
  new_fixture; printf 'g2\n' >"$REPO/docs/guide.md"; printf 'p\n' >"$REPO/docs/harness/progress/x.md"; commit_all x; skip_denied "$label"
  label="md turned into symlink"
  new_fixture; rm "$REPO/docs/guide.md"; ln -s ../src/lib.rs "$REPO/docs/guide.md"; commit_all x; skip_denied "$label"
  label="md renamed out of docs into src"
  new_fixture; g mv docs/guide.md src/guide.rs; commit_all x; skip_denied "$label"
  label="src renamed into docs md"
  new_fixture; g mv src/lib.rs docs/lib.md; commit_all x; skip_denied "$label"
  label="harness md renamed to plain docs md"
  new_fixture; g mv docs/harness/GATES.md docs/gates.md; commit_all x; skip_denied "$label"
  label="Cargo.toml"
  new_fixture; printf '[package]\n' >"$REPO/Cargo.toml"; commit_all x; skip_denied "$label"
  label="name that looks like docs md on its first line only"
  new_fixture; printf 'a\n' >"$REPO/docs/x.md"$'\n'"y.rs"; commit_all x; skip_denied "$label"
}

test_script_changes_always_require_reviewer() {
  new_fixture
  printf '#!/bin/sh\necho changed\n' >"$REPO/docs/harness/bin/tool.sh"
  commit_all "script change"
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_rc $RC_PENDING "script change without review"
  expect_out "docs/harness/bin/tool.sh" "script change is named"
  expect_out "Harness script changes" "script change flagged"
  # A FAIL review still fails; PASS without receipt is pending.
  compute_scope
  REVIEW="$OPS/review-script.md"
  RECEIPT="$OPS/receipt.env"
  write_review "$REVIEW" "REVIEW_VERDICT: PASS fine"
  write_receipt "$RECEIPT"
  run_post
  expect_rc $RC_PASS "script change with PASS + trusted receipt"
  # Script deleted/renamed: both sides count as a harness script change.
  new_fixture
  g mv docs/harness/bin/tool.sh docs/tool.md
  commit_all "move script out of bin"
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_rc $RC_PENDING "script moved to docs md"
  expect_out "Harness script changes" "moved script flagged"
}

test_modified_gate_cannot_authorize_itself() {
  new_fixture
  # The base holds the OLD, trusted gate; the candidate swaps in a gate that approves anything.
  printf '# old gate marker\n' >>"$REPO/docs/harness/bin/review-gate.sh"
  commit_all "old gate at base"
  BASE="$HEAD_SHA"
  git -C "$REPO" show "$BASE:docs/harness/bin/review-gate.sh" >"$OPS/trusted/review-gate.sh"
  GATE_SHA="$(sha256_of "$OPS/trusted/review-gate.sh")"
  printf '#!/usr/bin/env bash\necho "GATE_STATUS: PASS"\nexit 0\n' >"$REPO/docs/harness/bin/review-gate.sh"
  commit_all "tamper with gate"
  # The writer's copy approves itself...
  gate_in_repo post "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_rc 0 "precondition: tampered in-repo gate approves anything"
  # ...but the operator's trusted base copy refuses without a review.
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_rc $RC_PENDING "trusted base copy refuses tampered gate change"
  expect_out "docs/harness/bin/review-gate.sh" "gate change named"
  # A receipt that binds the tampered gate's hash does not match the operator's trusted gate.
  compute_scope
  REVIEW="$OPS/review-gate-change.md"
  RECEIPT="$OPS/receipt.env"
  write_review "$REVIEW" "REVIEW_VERDICT: PASS fine"
  write_receipt "$RECEIPT" "GATE_SCRIPT_SHA256=$(sha256_of "$REPO/docs/harness/bin/review-gate.sh")"
  run_post
  expect_rc $RC_PENDING "receipt binds a different gate than the trusted copy"
  expect_out "receipt-gate-mismatch" "gate binding reason"
  write_receipt "$RECEIPT"
  run_post
  expect_rc $RC_PASS "control: receipt binds the trusted gate"
}

test_gate_inside_worktree_cannot_pass() {
  new_fixture
  change_src
  make_pass_case
  run_post
  expect_rc $RC_PASS "control: trusted external gate copy"
  gate_in_repo post "$TASK" --base "$BASE" --head "$HEAD_SHA" --review-file "$REVIEW" \
    --receipt "$RECEIPT" --expect-tree "$TREE" --expect-diff-sha256 "$DIFF" \
    --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA"
  expect_rc $RC_PENDING "gate run from inside the worktree it judges"
  expect_out "gate-untrusted-location" "in-worktree gate reason"
  # Same for a sibling worktree copy and a case-variant spelling of the repo path.
  g worktree add -q -b wt-gate "$TMP/wtg$FIX_N" "$BASE"
  mkdir -p "$TMP/wtg$FIX_N/docs/harness/bin"
  cp "$GATE_SRC" "$TMP/wtg$FIX_N/docs/harness/bin/review-gate.sh"
  OUT="$(cd "$OPS" && bash "$TMP/wtg$FIX_N/docs/harness/bin/review-gate.sh" post "$TASK" --repo "$REPO" --base "$BASE" \
    --head "$HEAD_SHA" --review-file "$REVIEW" --receipt "$RECEIPT" --expect-tree "$TREE" \
    --expect-diff-sha256 "$DIFF" --operator "$OPERATOR" --expect-gate-sha256 "$GATE_SHA" 2>&1)"
  RC=$?
  expect_rc $RC_PENDING "gate run from a sibling worktree"
}

test_task_id_and_usage_validation() {
  new_fixture
  change_src
  gate post "../escape" --base "$BASE" --head "$HEAD_SHA"
  expect_rc $RC_USAGE "task id with path traversal"
  gate post "has space" --base "$BASE" --head "$HEAD_SHA"
  expect_rc $RC_USAGE "task id with space"
  gate bogus "$TASK"
  expect_rc $RC_USAGE "unknown mode"
  gate post "$TASK" --frobnicate
  expect_rc $RC_USAGE "unknown flag"
  gate post
  expect_rc $RC_USAGE "no task id"
}

test_receipt_case_variant_path_rejected() {
  new_fixture
  change_src
  make_pass_case
  # Detect (do not assume) a case-insensitive temp filesystem.
  mkdir "$TMP/CaseProbe$FIX_N"
  if [ ! -d "$TMP/caseprobe$FIX_N" ]; then
    not_applicable "receipt case-variant path" "temp filesystem is case-sensitive; the case-variant bypass cannot occur here"
    return
  fi
  cp "$RECEIPT" "$REPO/receipt.env"
  local variant
  variant="$(dirname "$REPO")/REPO"
  if [ "$variant/receipt.env" -ef "$REPO/receipt.env" ]; then ok; else bad "precondition: case variant should name the same file"; fi
  run_post --receipt "$variant/receipt.env"
  expect_rc $RC_PENDING "receipt reached through a different-case repo path"
  expect_out "receipt-untrusted-location" "case-variant repo path reason"
  cp "$RECEIPT" "$REPO/.git/receipt.env"
  run_post --receipt "$(dirname "$REPO")/Repo/.GIT/receipt.env"
  expect_rc $RC_PENDING "receipt reached through a different-case .git path"
  g worktree add -q -b wt-case "$TMP/wtc$FIX_N" "$BASE"
  cp "$RECEIPT" "$TMP/wtc$FIX_N/receipt.env"
  run_post --receipt "$TMP/WTC$FIX_N/receipt.env"
  expect_rc $RC_PENDING "receipt reached through a different-case worktree path"
}

test_receipt_hardlink_into_repo_rejected() {
  new_fixture
  change_src
  make_pass_case
  cp "$RECEIPT" "$REPO/receipt.env"
  ln "$REPO/receipt.env" "$OPS/hard.env"
  run_post --receipt "$OPS/hard.env"
  expect_rc $RC_PENDING "receipt hard-linked to a file the writer can edit"
  expect_out "receipt-untrusted-location" "hardlink reason"
}

test_receipt_acl_rejected() {
  new_fixture
  change_src
  make_pass_case
  cp "$RECEIPT" "$OPS/acl.env"
  if chmod +a "everyone allow write" "$OPS/acl.env" 2>/dev/null; then
    run_post --receipt "$OPS/acl.env"
    expect_rc $RC_PENDING "receipt carrying an ACL"
    expect_out "receipt-untrusted-permissions" "ACL reason"
  elif command -v setfacl >/dev/null 2>&1 && setfacl -m u:nobody:rw "$OPS/acl.env" 2>/dev/null; then
    run_post --receipt "$OPS/acl.env"
    expect_rc $RC_PENDING "receipt carrying an ACL"
  else
    not_run "receipt ACL" "no way to create an ACL on this platform"
  fi
}

# macOS `ls -l` prints "@" instead of "+" when a file has both extended attributes and an ACL;
# the ACL must still be inspected (the sandbox that runs the lane adds provenance xattrs).
test_receipt_acl_with_xattr_rejected() {
  if [ "$(uname -s)" != "Darwin" ]; then
    not_applicable "receipt ACL with xattr" "only macOS ls prints '@' in place of '+' for xattr+ACL paths"
    return
  fi
  if ! command -v xattr >/dev/null 2>&1; then
    not_run "receipt ACL with xattr" "no xattr tool on this platform"
    return
  fi
  new_fixture
  change_src
  make_pass_case
  cp "$RECEIPT" "$OPS/acl-xattr.env"
  if xattr -w com.example.h2-test 1 "$OPS/acl-xattr.env" 2>/dev/null \
    && chmod +a "everyone allow write" "$OPS/acl-xattr.env" 2>/dev/null; then
    run_post --receipt "$OPS/acl-xattr.env"
    expect_rc $RC_PENDING "receipt carrying an ACL and an xattr"
    expect_out "receipt-untrusted-permissions" "ACL+xattr reason"
  else
    not_run "receipt ACL with xattr" "cannot set both an xattr and an ACL here"
  fi
}

test_replace_ref_cannot_forge_scope() {
  new_fixture
  printf 'fn a() { 1 }\n' >"$REPO/src/lib.rs"
  commit_all "benign"
  local benign="$HEAD_SHA"
  g reset -q --hard "$BASE"
  printf 'fn a() { evil() }\n' >"$REPO/src/lib.rs"
  commit_all "evil"
  local evil="$HEAD_SHA" real_tree
  real_tree="$(g --no-replace-objects rev-parse "$evil^{tree}")"
  # Operator reviewed the benign commit...
  HEAD_SHA="$benign"
  make_pass_case
  local benign_tree="$TREE" benign_diff="$DIFF"
  # ...then the writer plants a local replace ref: evil -> benign.
  g replace "$evil" "$benign"
  HEAD_SHA="$evil"
  gate scope "$TASK" --base "$BASE" --head "$evil"
  if [ "$(out_val HEAD_SHA)" = "$evil" ] && [ "$(out_val TREE_SHA)" = "$real_tree" ]; then ok; else bad "scope followed the replace ref: head=$(out_val HEAD_SHA) tree=$(out_val TREE_SHA) (real tree $real_tree)"; fi
  if [ "$(out_val DIFF_SHA256)" != "$benign_diff" ]; then ok; else bad "diff hash is the benign commit's hash"; fi
  # Receipt built from the benign scope but naming the evil head must not pass.
  TREE="$benign_tree"
  DIFF="$benign_diff"
  write_receipt "$RECEIPT"
  run_post
  expect_rc $RC_PENDING "receipt forged via replace ref"
  expect_not_out "GATE_STATUS: PASS" "replace-ref forgery"
}

test_graft_cannot_alter_ancestry() {
  new_fixture
  printf 'fn a() { 1 }\n' >"$REPO/src/lib.rs"
  commit_all "x"
  printf 'fn a() { 2 }\n' >"$REPO/src/lib.rs"
  commit_all "e"
  printf '%s\n' "$HEAD_SHA" >"$REPO/.git/info/grafts"
  gate scope "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_rc $RC_PASS "a grafted-away parent must not make a valid range look invalid"
}

test_attributes_cannot_hide_diff() {
  new_fixture
  printf '* -diff\n' >"$REPO/.git/info/attributes"
  printf 'fn a() { hidden_marker() }\n' >"$REPO/src/lib.rs"
  commit_all "change under -diff"
  gate pre "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_rc 0 "pre with info/attributes -diff"
  local raw="" f
  for f in "$REPO"/docs/harness/reviews/*-"$TASK"*-pre.md.raw; do [ -f "$f" ] && raw="$f" && break; done
  if [ -n "$raw" ] && grep -q '^+fn a() { hidden_marker() }' "$raw" && ! grep -q 'Binary files' "$raw"; then ok; else bad "info/attributes -diff hid content from the reviewer diff"; fi
  # Same via an in-tree .gitattributes added by the candidate, and a textconv driver.
  new_fixture
  printf '*.rs -diff\n' >"$REPO/.gitattributes"
  printf 'fn a() { hidden_marker() }\n' >"$REPO/src/lib.rs"
  commit_all "gitattributes -diff"
  gate pre "$TASK" --base "$BASE" --head "$HEAD_SHA"
  raw=""
  for f in "$REPO"/docs/harness/reviews/*-"$TASK"*-pre.md.raw; do [ -f "$f" ] && raw="$f" && break; done
  if [ -n "$raw" ] && grep -q '^+fn a() { hidden_marker() }' "$raw"; then ok; else bad "in-tree -diff attribute hid content"; fi
  new_fixture
  printf 'src/lib.rs diff=cloak\n' >"$REPO/.git/info/attributes"
  g config diff.cloak.textconv "echo cloaked #"
  printf 'fn a() { hidden_marker() }\n' >"$REPO/src/lib.rs"
  commit_all "textconv"
  gate pre "$TASK" --base "$BASE" --head "$HEAD_SHA"
  raw=""
  for f in "$REPO"/docs/harness/reviews/*-"$TASK"*-pre.md.raw; do [ -f "$f" ] && raw="$f" && break; done
  if [ -n "$raw" ] && grep -q '^+fn a() { hidden_marker() }' "$raw" && ! grep -q 'cloaked' "$raw"; then ok; else bad "textconv driver changed the reviewed diff"; fi
}

test_node_modules_is_reviewed() {
  new_fixture
  mkdir -p "$REPO/node_modules/pkg" "$REPO/sdks/typescript/node_modules/p"
  printf 'module.exports = 1\n' >"$REPO/node_modules/pkg/index.js"
  printf 'module.exports = 2\n' >"$REPO/sdks/typescript/node_modules/p/i.js"
  commit_all "vendored code"
  gate scope "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_out "CHANGED_PATH=node_modules/pkg/index.js" "node_modules change is reviewable"
  expect_out "CHANGED_PATH=sdks/typescript/node_modules/p/i.js" "nested node_modules change is reviewable"
  expect_out "EXCLUDED_PATH_COUNT=0" "nothing silently excluded"
}

test_prepare_shows_staged_and_worktree_content() {
  new_fixture
  printf 'fn a() { staged_marker() }\n' >"$REPO/src/lib.rs"
  g add src/lib.rs
  printf 'fn a() { worktree_marker() }\n' >"$REPO/src/lib.rs"
  gate pre "$TASK"
  expect_rc 0 "pre with index != working tree"
  local raw="" f
  for f in "$REPO"/docs/harness/reviews/*-"$TASK"*-pre.md.raw; do [ -f "$f" ] && raw="$f" && break; done
  if [ -n "$raw" ] && grep -q 'worktree_marker' "$raw"; then ok; else bad "prompt lacks working-tree content"; fi
  if [ -n "$raw" ] && grep -q 'staged_marker' "$raw"; then ok; else bad "prompt lacks staged (index) content that differs from the working tree"; fi
}

test_staged_only_pathspec_is_literal() {
  new_fixture
  printf 'fn gx() { 1 }\n' >"$REPO/src/gx.rs"
  commit_all "add gx"
  BASE="$HEAD_SHA"
  printf 'fn gx() { 2 }\n' >"$REPO/src/gx.rs"
  g add src/gx.rs
  printf 'ghost\n' >"$REPO/src/g*.rs"
  g add "src/g*.rs"
  rm "$REPO/src/g*.rs"
  gate pre "$TASK"
  expect_rc 0 "pre with a glob-named staged ghost"
  local raw="" f n
  for f in "$REPO"/docs/harness/reviews/*-"$TASK"*-pre.md.raw; do [ -f "$f" ] && raw="$f" && break; done
  n="$(grep -c '^+++ b/src/gx.rs' "$raw" 2>/dev/null)"
  if [ "$n" = "1" ]; then ok; else bad "glob in a staged-only path pulled in other files (gx.rs diff appears $n times)"; fi
  if [ -n "$raw" ] && grep -qF '+++ b/src/g*.rs' "$raw"; then ok; else bad "staged ghost with glob characters missing from diff"; fi
}

test_prompt_write_failure_is_nonzero() {
  new_fixture
  change_src
  chmod 500 "$REPO/docs/harness/reviews"
  if [ -w "$REPO/docs/harness/reviews" ]; then
    chmod 700 "$REPO/docs/harness/reviews"
    not_run "prompt write failure" "directory permissions are not enforced for this user"
    return
  fi
  gate pre "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_nonzero "pre cannot write its prompt"
  expect_not_out "GATE_STATUS: ADVISORY" "pre must not report success when the prompt was not written"
  gate post "$TASK" --base "$BASE" --head "$HEAD_SHA"
  expect_rc $RC_SCOPE "post cannot write its prompt"
  chmod 700 "$REPO/docs/harness/reviews"
}

test_receipt_v2_is_a_superset_of_v1() {
  new_fixture
  change_src
  make_pass_case
  write_receipt "$RECEIPT" RECEIPT_VERSION=2
  run_post
  expect_rc $RC_PASS "v2 receipt (v1 keys + REVIEWER/POLICY_VERSION/REVIEW_CONTEXT)"
  expect_status PASS "v2 receipt"
  write_receipt "$RECEIPT"
  run_post
  expect_rc $RC_PASS "v1 receipt stays accepted for history"
  write_receipt "$RECEIPT" RECEIPT_VERSION=2 REVIEWER=
  run_post
  expect_rc $RC_PENDING "v2 receipt with an empty REVIEWER"
  expect_out "receipt-malformed" "empty REVIEWER is malformed"
  write_receipt "$RECEIPT" RECEIPT_VERSION=2 POLICY_VERSION=Bad_Policy
  run_post
  expect_rc $RC_PENDING "v2 receipt with a malformed POLICY_VERSION"
  write_receipt "$RECEIPT"
  printf 'REVIEWER=rev-fixture\n' >>"$RECEIPT"
  run_post
  expect_rc $RC_PENDING "v1 receipt carrying a v2-only key"
  expect_out "needs RECEIPT_VERSION=2" "v2-only key in a v1 receipt"
}

test_receipt_template_round_trips_through_post() {
  new_fixture
  change_src
  make_pass_case
  gate receipt-template "$TASK" --base "$BASE" --head "$HEAD_SHA" --review-file "$REVIEW"
  expect_rc 0 "receipt-template"
  expect_out "# GATE_STATUS: ADVISORY reason=receipt-template" "template is advisory"
  expect_out "TREE_SHA=$TREE" "template binds the recomputed tree"
  expect_out "DIFF_SHA256=$DIFF" "template binds the recomputed diff"
  expect_out "GATE_SCRIPT_SHA256=$GATE_SHA" "template binds the gate copy"
  expect_out "REVIEW_SHA256=$(sha256_of "$REVIEW")" "template binds the review bytes"
  local template="$OUT"
  printf '%s\n' "$template" >"$RECEIPT"
  run_post
  expect_rc $RC_PENDING "unfilled template"
  expect_out "unfilled <placeholder>" "placeholders are rejected"
  printf '%s\n' "$template" | sed -e "s/^OPERATOR=.*/OPERATOR=$OPERATOR/" \
    -e 's/^REVIEWER=.*/REVIEWER=rev-fixture/' \
    -e 's/^POLICY_VERSION=.*/POLICY_VERSION=harness-hardening-v1/' \
    -e 's/^REVIEW_CONTEXT=.*/REVIEW_CONTEXT=fresh-readonly/' >"$OPS/filled.env"
  RECEIPT="$OPS/filled.env"
  run_post
  expect_rc $RC_PASS "filled template accepted by post"
  expect_status PASS "filled template"
  gate receipt-template "$TASK"
  expect_rc $RC_USAGE "receipt-template without a final scope"
}

# --- runner ----------------------------------------------------------------

echo "=== review-gate.sh regression suite ==="
echo "Gate under test: $GATE_SRC"
echo "Fixtures dir:    $TMP"

for t in $(declare -F | sed -n 's/^declare -f \(test_[a-z0-9_]*\)$/\1/p'); do
  CURRENT_TEST="$t"
  TESTS_RUN=$((TESTS_RUN + 1))
  before=$FAIL_COUNT
  OUT=""
  RC=""
  "$t"
  if [ "$FAIL_COUNT" -eq "$before" ]; then echo "  [PASS] $t"; else echo "  [FAIL] $t" >&2; fi
done

echo
echo "Tests: $TESTS_RUN  Assertions passed: $PASS_COUNT  Assertions failed: $FAIL_COUNT  Not run: $NOT_RUN_COUNT  Not applicable: $NOT_APPLICABLE_COUNT"
if [ "$TESTS_RUN" -eq 0 ]; then
  echo "FATAL: no tests were discovered" >&2
  exit 2
fi
[ "$FAIL_COUNT" -eq 0 ]
