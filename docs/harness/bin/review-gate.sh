#!/usr/bin/env bash
# docs/harness/bin/review-gate.sh
#
# Cross-CLI / cross-model review gate for the engram harness (fail-closed).
#
# Supports the user's current workflow (Claude Code + Claude Code Sonnet reviewer session).
#
# Usage:
#   review-gate.sh pre   <task-id> [--base REV --head REV | --range A..B] [--prev FILE]
#       Advisory. Writes the reviewer prompt. Never approves anything (GATE_STATUS: ADVISORY).
#       Without a range the scope is the PREPARATION scope: working tree vs HEAD, which
#       includes staged-only changes (index) and non-ignored untracked files.
#   review-gate.sh scope <task-id> (--base REV --head REV | --range A..B | --prepare)
#       Read-only. Prints the scope the gate binds a review to (shas, diff sha256, paths).
#   review-gate.sh receipt-template <task-id> (--base REV --head REV | --range A..B)
#                                   [--review-file FILE]
#       Read-only producer for the operator: prints a RECEIPT_VERSION=2 receipt with every value
#       recomputed from git (task/base/head/tree/diff/gate sha256, review path + sha256 when
#       --review-file is given) and <placeholders> for OPERATOR, REVIEWER, POLICY_VERSION
#       and REVIEW_CONTEXT (a deliberate operator attestation). An unfilled
#       placeholder is malformed for every consumer; filling it is the operator's attestation.
#   review-gate.sh post  <task-id> (--base REV --head REV | --range A..B)
#                        [--review-file FILE] [--receipt FILE | $ENGRAM_REVIEW_RECEIPT]
#                        --expect-tree SHA --expect-diff-sha256 HASH --expect-gate-sha256 HASH --operator ID
#       Hard gate over the FINAL scope: every commit in base..candidate. There is no default
#       range ("last commit" is never guessed). Options: --repo DIR runs the gate (typically a
#       trusted copy taken from the base revision) against another repository.
#
# Exit codes (GATE_STATUS: <STATUS> reason=<code> is always the last stdout line of a verdict):
#   0  PASS               valid trusted receipt + REVIEW_VERDICT: PASS bound to this exact scope
#   0  SKIPPED_ALLOWLIST  whole diff is docs-only per GATES.md (docs/**/*.md outside docs/harness)
#   0  ADVISORY           pre / scope only; this is NOT an approval
#   1  FAIL               reviewer verdict FAIL
#   1  INVALID_REVIEW     review artifact has no / more than one valid REVIEW_VERDICT marker
#   2  usage error
#   3  PENDING            no acceptable evidence yet: review file missing, or receipt missing,
#                         untrusted, malformed or not matching the recomputed scope. A PASS
#                         marker alone is history only and NEVER reaches exit 0.
#   4  scope error        invalid/empty range, missing commit, git diff/show failure; reported
#                         before any review verdict is read.
#
# Trusted provenance (before H4/H5 automate it): an operator-written receipt stored OUTSIDE any
# worktree / .git directory binds task id, base, head, tree, sha256 of the reviewed diff, review
# artifact path + sha256 and operator identity (format in GATES.md; RECEIPT_VERSION=2 is a superset
# that adds REVIEWER, POLICY_VERSION and REVIEW_CONTEXT for the H5 merge gate; v1 stays
# accepted). The gate recomputes all of
# them from git and compares them with the receipt AND with values the operator passes on the
# command line. The gate itself must be a copy kept OUTSIDE every worktree of the repository it
# judges (taken from a trusted revision, run with --repo); its sha256 is bound by the receipt and
# by --expect-gate-sha256, and a PASS from a gate living inside a worktree is refused.
# See GATES.md "Review gate fail-closed" for the operator runbook.
#
# Environment:
#   ENGRAM_REVIEW_RECEIPT=/abs/path      receipt location (alternative to --receipt)
#   REVIEWER_CLI=claude-sonnet|codex|ollama|manual (affects prompt tone; default "manual")
#   REVIEWER_TIMEOUT_SECS=...            (future non-interactive exec)
#
# The script builds a rich prompt including SPEC, INVARIANTS, WHAT_WE_DONT_DO, GATES, CODE_REVIEW_POLICY,
# docs/harness/security/anthropic-reference-harness.md, .claude/scan-extras.txt,
# .claude/fp-rules.txt, fake-success patterns, and the relevant diff (with harness artifacts excluded).
# It writes artifacts to docs/harness/reviews/ with iteration versioning.
# Verdict is parsed from an explicit marker line:
#   REVIEW_VERDICT: PASS ...
# (or FAIL ...), not from any other PASS/FAIL text.

set -uo pipefail

RC_PASS=0
RC_FAIL=1
RC_USAGE=2
RC_PENDING=3
RC_SCOPE=4

# A caller must not be able to redirect git away from the repository this gate guards.
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES \
  GIT_COMMON_DIR GIT_NAMESPACE GIT_EXTERNAL_DIFF GIT_DIFF_OPTS GIT_PREFIX GIT_CEILING_DIRECTORIES

# Writer-writable .git state must not change what the gate sees: no replace refs, no grafts,
# no system attributes (see also gitx and --text below).
export GIT_NO_REPLACE_OBJECTS=1 GIT_GRAFT_FILE=/dev/null GIT_ATTR_NOSYSTEM=1

# Absolute before any `cd`: the gate hashes and location-checks itself after moving to the repo root.
GATE_DIR="$(cd "$(dirname -- "${BASH_SOURCE[0]}")" 2>/dev/null && pwd)" || {
  echo "ERROR: cannot resolve the gate script path" >&2
  exit "$RC_USAGE"
}
GATE_SCRIPT="$GATE_DIR/$(basename -- "${BASH_SOURCE[0]}")"

# ---------------------------------------------------------------------------
# Small helpers
# ---------------------------------------------------------------------------

usage() {
  cat >&2 <<'EOF'
Usage:
  review-gate.sh pre   <task-id> [--base REV --head REV | --range A..B] [--prev FILE]
  review-gate.sh scope <task-id> (--base REV --head REV | --range A..B | --prepare)
  review-gate.sh receipt-template <task-id> (--base REV --head REV | --range A..B)
                                  [--review-file FILE]
  review-gate.sh post  <task-id> (--base REV --head REV | --range A..B) [--review-file FILE]
                       [--receipt FILE] --expect-tree SHA --expect-diff-sha256 HASH
                       --expect-gate-sha256 HASH --operator ID
  common: [--repo DIR]
EOF
}

usage_err() {
  echo "ERROR: $1" >&2
  usage
  echo "GATE_STATUS: ERROR reason=usage"
  exit "$RC_USAGE"
}

scope_error() { # reason detail
  echo "ERROR: $2" >&2
  echo "GATE_STATUS: ERROR reason=$1"
  exit "$RC_SCOPE"
}

io_error() { # detail
  echo "ERROR: $1" >&2
  echo "GATE_STATUS: ERROR reason=io-failure"
  exit "$RC_SCOPE"
}

pending() { # reason detail
  echo "PENDING: $2"
  echo "No review verdict is accepted without trusted provenance bound to this exact scope."
  echo "GATE_STATUS: PENDING reason=$1"
  exit "$RC_PENDING"
}

sha256_stdin() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum | cut -d' ' -f1
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 | cut -d' ' -f1
  else
    echo "ERROR: neither sha256sum nor shasum is available" >&2
    echo "GATE_STATUS: ERROR reason=no-sha256-tool"
    exit "$RC_USAGE"
  fi
}

sha256_file() { sha256_stdin <"$1"; }

# Physical (symlink-free) directory of a path's parent + basename.
phys_path() {
  local d b
  d="$(cd "$(dirname -- "$1")" 2>/dev/null && pwd -P)" || return 1
  b="$(basename -- "$1")"
  printf '%s/%s\n' "${d%/}" "$b"
}

path_within() { # path dir  -> 0 when path is dir or inside dir
  case "$1/" in "${2%/}"/*) return 0 ;; esac
  return 1
}

valid_task_id() { [[ "$1" =~ ^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$ ]]; }

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------

MODE="${1:-}"
TASK_ID="${2:-}"
BASE_REV=""
HEAD_REV=""
RANGE=""
PREPARE=0
REVIEW_FILE=""
PREV_REVIEW=""
RECEIPT_ARG="${ENGRAM_REVIEW_RECEIPT:-}"
EXPECT_TREE=""
EXPECT_DIFF=""
EXPECT_GATE=""
OPERATOR_ARG=""
REPO_ARG=""

case "$MODE" in
  pre | post | scope | receipt-template) ;;
  *) usage_err "mode must be 'pre', 'scope', 'receipt-template' or 'post' (got '${MODE}')" ;;
esac
[ -n "$TASK_ID" ] || usage_err "missing task-id"
valid_task_id "$TASK_ID" || usage_err "task-id must match [A-Za-z0-9][A-Za-z0-9._-]{0,63} (got '${TASK_ID}')"

shift 2
while [ "$#" -gt 0 ]; do
  arg="$1"
  shift
  val=""
  case "$arg" in
    --prepare) PREPARE=1; continue ;;
    --*=*)
      val="${arg#*=}"
      arg="${arg%%=*}"
      ;;
    --*)
      # value is the next argument (flags below all take one)
      case "$arg" in
        --base | --head | --candidate | --range | --review-file | --receipt | --expect-tree | \
          --expect-diff-sha256 | --expect-gate-sha256 | --operator | --prev | --repo)
          [ "$#" -ge 1 ] || usage_err "$arg requires a value"
          val="$1"
          shift
          ;;
      esac
      ;;
  esac
  case "$arg" in
    --base) BASE_REV="$val" ;;
    --head | --candidate) HEAD_REV="$val" ;;
    --range) RANGE="$val" ;;
    --review-file) REVIEW_FILE="$val" ;;
    --receipt) RECEIPT_ARG="$val" ;;
    --expect-tree) EXPECT_TREE="$val" ;;
    --expect-diff-sha256) EXPECT_DIFF="$val" ;;
    --expect-gate-sha256) EXPECT_GATE="$val" ;;
    --operator) OPERATOR_ARG="$val" ;;
    --prev) PREV_REVIEW="$val" ;;
    --repo) REPO_ARG="$val" ;;
    *) usage_err "unknown arg $arg" ;;
  esac
done

# ---------------------------------------------------------------------------
# Repository resolution (script location by default; --repo for a trusted gate copy)
# ---------------------------------------------------------------------------

if [ -n "$REPO_ARG" ]; then
  REPO_ROOT="$(git -C "$REPO_ARG" rev-parse --show-toplevel 2>/dev/null)" || usage_err "--repo is not a git work tree: $REPO_ARG"
else
  REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." 2>/dev/null && pwd)"
fi
if [ -z "${REPO_ROOT:-}" ]; then
  echo "ERROR: cannot resolve repo root" >&2
  exit "$RC_USAGE"
fi
REPO_ROOT="$(cd "$REPO_ROOT" && pwd -P)"
cd "$REPO_ROOT" || exit "$RC_USAGE"

# Scope selection: explicit only.
if [ -n "$RANGE" ]; then
  [ -z "$BASE_REV" ] && [ -z "$HEAD_REV" ] || usage_err "--range cannot be combined with --base/--head"
  case "$RANGE" in
    *...*) scope_error invalid-range "range '$RANGE': three-dot ranges are not a commit range" ;;
  esac
  r_base="${RANGE%%..*}"
  r_head="${RANGE#*..}"
  if [ "$r_base..$r_head" != "$RANGE" ] || [ -z "$r_base" ] || [ -z "$r_head" ]; then
    scope_error invalid-range "range '$RANGE' is not of the form BASE..CANDIDATE"
  fi
  case "$r_head" in *..*) scope_error invalid-range "range '$RANGE' has more than one '..'" ;; esac
  BASE_REV="$r_base"
  HEAD_REV="$r_head"
fi
if [ "$PREPARE" -eq 1 ]; then
  [ -z "$BASE_REV" ] && [ -z "$HEAD_REV" ] || usage_err "--prepare cannot be combined with a range"
  [ "$MODE" = "scope" ] || usage_err "--prepare is only valid for 'scope' (pre defaults to it)"
fi
if [ -n "$BASE_REV" ] || [ -n "$HEAD_REV" ]; then
  [ -n "$BASE_REV" ] && [ -n "$HEAD_REV" ] || usage_err "--base and --head must be given together"
  case "$BASE_REV" in -*) usage_err "revisions must not start with '-'" ;; esac
  case "$HEAD_REV" in -*) usage_err "revisions must not start with '-'" ;; esac
fi
FINAL_SCOPE=0
if [ -n "$BASE_REV" ]; then FINAL_SCOPE=1; fi
if [ "$MODE" = "post" ] && [ "$FINAL_SCOPE" -eq 0 ]; then
  usage_err "post requires an explicit final scope (--base REV --head REV or --range A..B); the gate never guesses the last commit"
fi
if [ "$MODE" = "receipt-template" ] && [ "$FINAL_SCOPE" -eq 0 ]; then
  usage_err "receipt-template requires an explicit final scope (--base/--head or --range A..B)"
fi
if [ "$MODE" = "scope" ] && [ "$FINAL_SCOPE" -eq 0 ] && [ "$PREPARE" -eq 0 ]; then
  usage_err "scope requires --base/--head, --range or --prepare"
fi

WORK="$(mktemp -d "${TMPDIR:-/tmp}/engram-review-gate.XXXXXX")" || {
  echo "ERROR: cannot create temp dir" >&2
  exit "$RC_USAGE"
}
trap 'rm -rf "$WORK"' EXIT

# ---------------------------------------------------------------------------
# Diff scope (fail closed: any git failure is a scope error, never "diff text")
# ---------------------------------------------------------------------------

# Reviewer-visible diff excludes only harness bookkeeping and build output (self-referential
# loops). Dependency directories such as node_modules/ can hold executable code: NOT excluded.
EXCLUDE_GLOBS=(
  "docs/harness/reviews/*"
  "docs/harness/progress/*"
  "target/*"
  "engram-wasm/target/*"
  "coverage/*"
)
# Plain (fnmatch) pathspecs: '*' also matches '/', so each glob acts as a prefix.
GIT_EXCLUDES=()
GIT_ONLY_EXCLUDED=()
for g in "${EXCLUDE_GLOBS[@]}"; do
  GIT_EXCLUDES+=(":(exclude)$g")
  GIT_ONLY_EXCLUDED+=("$g")
done

# Everything that influences diff bytes is pinned so the sha256 binding is reproducible.
DIFF_FLAGS=(--no-color --no-ext-diff --no-textconv --text --no-renames --full-index --diff-algorithm=myers
  --ignore-submodules=none --src-prefix=a/ --dst-prefix=b/ -O/dev/null)

# --text defeats -diff/binary attributes (incl. .git/info/attributes) that would hide content;
# replace refs, grafts and attribute files are neutralised so only real objects are judged.
gitx() {
  git --no-replace-objects -c advice.graftFileDeprecated=false -c core.attributesFile=/dev/null \
    -c core.quotepath=true -c diff.mnemonicPrefix=false -c diff.noprefix=false "$@"
}

resolve_commit() { # rev -> commit sha on stdout
  gitx rev-parse --verify --quiet "$1^{commit}" 2>/dev/null
}

# git_to FILE ARGS...   run git, stdout to FILE; any failure is a scope error.
git_to() {
  local out="$1"
  shift
  if ! gitx "$@" >"$out" 2>"$WORK/git.err"; then
    scope_error git-failure "git $* failed: $(head -c 400 "$WORK/git.err" | tr '\n' ' ')"
  fi
}

# Emits CHANGED_PATH lines / counts for a NUL-separated list file.
count_nul() { # file -> number of entries
  local n=0 p
  while IFS= read -r -d '' p; do n=$((n + 1)); done <"$1"
  echo "$n"
}

print_paths() { # label file
  local p
  while IFS= read -r -d '' p; do printf '%s=%q\n' "$1" "$p"; done <"$2"
}

compute_final_scope() {
  BASE_SHA="$(resolve_commit "$BASE_REV")" || scope_error missing-commit "base '$BASE_REV' does not resolve to a commit"
  HEAD_SHA="$(resolve_commit "$HEAD_REV")" || scope_error missing-commit "candidate '$HEAD_REV' does not resolve to a commit"
  [ "$BASE_SHA" != "$HEAD_SHA" ] || scope_error invalid-range "base and candidate are the same commit: empty range"
  gitx merge-base --is-ancestor "$BASE_SHA" "$HEAD_SHA" 2>/dev/null
  case $? in
    0) ;;
    1) scope_error invalid-range "base $BASE_SHA is not an ancestor of candidate $HEAD_SHA" ;;
    *) scope_error git-failure "git merge-base --is-ancestor failed" ;;
  esac
  TREE_SHA="$(gitx rev-parse --verify --quiet "$HEAD_SHA^{tree}" 2>/dev/null)" || scope_error missing-commit "cannot resolve tree of $HEAD_SHA"
  git_to "$WORK/all.z" diff -z --name-only "${DIFF_FLAGS[@]}" "$BASE_SHA" "$HEAD_SHA" --
  git_to "$WORK/rpaths.z" diff -z --name-only "${DIFF_FLAGS[@]}" "$BASE_SHA" "$HEAD_SHA" -- . "${GIT_EXCLUDES[@]}"
  git_to "$WORK/epaths.z" diff -z --name-only "${DIFF_FLAGS[@]}" "$BASE_SHA" "$HEAD_SHA" -- "${GIT_ONLY_EXCLUDED[@]}"
  git_to "$WORK/raw.z" diff -z --raw "${DIFF_FLAGS[@]}" "$BASE_SHA" "$HEAD_SHA" --
  git_to "$WORK/diff.txt" diff --unified=0 --inter-hunk-context=0 "${DIFF_FLAGS[@]}" "$BASE_SHA" "$HEAD_SHA" -- . "${GIT_EXCLUDES[@]}"
  [ -s "$WORK/all.z" ] || scope_error invalid-range "range $BASE_SHA..$HEAD_SHA changes no paths"
  [ -s "$WORK/rpaths.z" ] || scope_error empty-scope "every changed path is excluded from review (nothing reviewable in $BASE_SHA..$HEAD_SHA)"
  SCOPE_MODE="final"
}

compute_prepare_scope() {
  HEAD_SHA="$(resolve_commit HEAD)" || scope_error missing-commit "HEAD does not resolve to a commit (preparation scope needs a base commit)"
  BASE_SHA="$HEAD_SHA"
  TREE_SHA=""
  git_to "$WORK/wpaths.z" diff -z --name-only "${DIFF_FLAGS[@]}" HEAD -- . "${GIT_EXCLUDES[@]}"
  git_to "$WORK/cpaths.z" diff --cached -z --name-only "${DIFF_FLAGS[@]}" HEAD -- . "${GIT_EXCLUDES[@]}"
  git_to "$WORK/upaths.z" ls-files -z --others --exclude-standard -- . "${GIT_EXCLUDES[@]}"
  git_to "$WORK/epaths.z" diff -z --name-only "${DIFF_FLAGS[@]}" HEAD -- "${GIT_ONLY_EXCLUDED[@]}"
  git_to "$WORK/diff.txt" diff --unified=0 --inter-hunk-context=0 "${DIFF_FLAGS[@]}" HEAD -- . "${GIT_EXCLUDES[@]}"
  git_to "$WORK/unstaged.z" diff -z --name-only "${DIFF_FLAGS[@]}" -- . "${GIT_EXCLUDES[@]}"
  : >"$WORK/rpaths.z"
  local p q seen qpath in_unstaged w=() u=()
  while IFS= read -r -d '' p; do
    w+=("$p")
    printf '%s\0' "$p" >>"$WORK/rpaths.z"
  done <"$WORK/wpaths.z"
  while IFS= read -r -d '' p; do u+=("$p"); done <"$WORK/unstaged.z"
  # Staged content the working-tree diff cannot show: paths missing from the working-tree diff
  # (e.g. staged then deleted from disk) and paths whose index content differs from the working
  # tree (index != working tree != HEAD). The staged diff is appended so the reviewer sees both.
  while IFS= read -r -d '' p; do
    seen=0
    for q in ${w[@]+"${w[@]}"}; do [ "$q" = "$p" ] && seen=1 && break; done
    in_unstaged=0
    for q in ${u[@]+"${u[@]}"}; do [ "$q" = "$p" ] && in_unstaged=1 && break; done
    if [ "$seen" -eq 0 ]; then
      printf '%s\0' "$p" >>"$WORK/rpaths.z"
    fi
    if [ "$seen" -eq 0 ] || [ "$in_unstaged" -eq 1 ]; then
      qpath="$(printf '%q' "$p")"
      printf '# staged (index) content for %s; the working-tree diff above may differ\n' "$qpath" >>"$WORK/diff.txt"
      git_to "$WORK/staged-only.diff" diff --cached --unified=0 --inter-hunk-context=0 "${DIFF_FLAGS[@]}" HEAD -- ":(literal)$p"
      cat "$WORK/staged-only.diff" >>"$WORK/diff.txt"
    fi
  done <"$WORK/cpaths.z"
  # Untracked (non-ignored) files are part of what will be committed.
  while IFS= read -r -d '' p; do
    printf '%s\0' "$p" >>"$WORK/rpaths.z"
    if [ -d "$p" ]; then
      printf '# untracked directory (nested repository?) not expanded: %s\n' "$p" >>"$WORK/diff.txt"
      continue
    fi
    gitx diff --no-index --unified=0 "${DIFF_FLAGS[@]}" -- /dev/null "$p" >"$WORK/untracked.diff" 2>"$WORK/git.err"
    case $? in
      0 | 1) cat "$WORK/untracked.diff" >>"$WORK/diff.txt" ;;
      *) scope_error git-failure "git diff --no-index failed for untracked path: $(head -c 400 "$WORK/git.err" | tr '\n' ' ')" ;;
    esac
  done <"$WORK/upaths.z"
  SCOPE_MODE="prepare"
}

# Docs-only skip allowlist (GATES.md "Skip Allowlist"): ONLY item 1 is mechanically verifiable.
# docs/**/*.md outside docs/harness/, regular files only (no symlinks / exec bit / submodules),
# every path on both sides of a rename. Comment-only, formatting-only and test-only changes
# need human judgement and are never auto-skipped by this gate.
classify_skip_allowlist() { # uses $WORK/raw.z ; sets SKIP_ALLOWLIST yes|no
  local meta path m_old m_new count=0
  SKIP_ALLOWLIST=yes
  while IFS= read -r -d '' meta && IFS= read -r -d '' path; do
    count=$((count + 1))
    read -r m_old m_new _ <<<"${meta#:}"
    case "$m_old" in 000000 | 100644) ;; *) SKIP_ALLOWLIST=no ;; esac
    case "$m_new" in 000000 | 100644) ;; *) SKIP_ALLOWLIST=no ;; esac
    case "$path" in
      docs/harness/*) SKIP_ALLOWLIST=no ;;
      docs/*.md) ;;
      *) SKIP_ALLOWLIST=no ;;
    esac
  done <"$WORK/raw.z"
  [ "$count" -gt 0 ] || SKIP_ALLOWLIST=no
}

detect_harness_script_changes() { # uses $WORK/all.z (final) or rpaths.z (prepare)
  local f p
  f="$WORK/all.z"
  [ -f "$f" ] || f="$WORK/rpaths.z"
  HARNESS_SCRIPT_CHANGES=""
  while IFS= read -r -d '' p; do
    case "$p" in
      docs/harness/bin/*) HARNESS_SCRIPT_CHANGES="${HARNESS_SCRIPT_CHANGES}$(printf '%q' "$p")"$'\n' ;;
    esac
  done <"$f"
}

if [ "$FINAL_SCOPE" -eq 1 ]; then compute_final_scope; else compute_prepare_scope; fi
DIFF_SHA256="$(sha256_file "$WORK/diff.txt")"
CHANGED_COUNT="$(count_nul "$WORK/rpaths.z")"
EXCLUDED_COUNT="$(count_nul "$WORK/epaths.z")"
SKIP_ALLOWLIST=no
[ "$SCOPE_MODE" = "final" ] && classify_skip_allowlist
detect_harness_script_changes
GATE_SHA256="$(sha256_file "$GATE_SCRIPT")"

print_scope() {
  echo "SCOPE_MODE=$SCOPE_MODE"
  echo "TASK_ID=$TASK_ID"
  echo "BASE_SHA=$BASE_SHA"
  echo "HEAD_SHA=$HEAD_SHA"
  [ -n "$TREE_SHA" ] && echo "TREE_SHA=$TREE_SHA"
  echo "DIFF_SHA256=$DIFF_SHA256"
  echo "CHANGED_PATH_COUNT=$CHANGED_COUNT"
  print_paths CHANGED_PATH "$WORK/rpaths.z"
  echo "EXCLUDED_PATH_COUNT=$EXCLUDED_COUNT"
  print_paths EXCLUDED_PATH "$WORK/epaths.z"
  echo "SKIP_ALLOWLIST=$SKIP_ALLOWLIST"
  echo "GATE_SCRIPT_SHA256=$GATE_SHA256"
}

if [ "$MODE" = "scope" ]; then
  print_scope
  echo "GATE_STATUS: ADVISORY reason=scope-only"
  exit "$RC_PASS"
fi

# Operator receipt producer: every bound value is recomputed here; only identities and the policy
# version are left as <placeholders> (rejected by every consumer until the operator fills them).
if [ "$MODE" = "receipt-template" ]; then
  T_REVIEW="<absolute path of the review artifact>"
  T_RSHA="<sha256 of the review artifact>"
  if [ -n "$REVIEW_FILE" ]; then
    case "$REVIEW_FILE" in
      /*) T_REVIEW="$REVIEW_FILE" ;;
      *) T_REVIEW="$REPO_ROOT/$REVIEW_FILE" ;;
    esac
    { [ -f "$T_REVIEW" ] && [ ! -L "$T_REVIEW" ]; } ||
      usage_err "--review-file must be a regular file: $T_REVIEW"
    T_RSHA="$(sha256_file "$T_REVIEW")"
  fi
  echo "# Operator receipt (RECEIPT_VERSION=2), recomputed from git by review-gate.sh."
  echo "# Fill every <placeholder>; store it outside any worktree (dir 0700, file 0600)."
  echo "# This is not an approval."
  echo "RECEIPT_VERSION=2"
  echo "TASK_ID=$TASK_ID"
  echo "BASE_SHA=$BASE_SHA"
  echo "HEAD_SHA=$HEAD_SHA"
  echo "TREE_SHA=$TREE_SHA"
  echo "DIFF_SHA256=$DIFF_SHA256"
  echo "REVIEW_ARTIFACT=$T_REVIEW"
  echo "REVIEW_SHA256=$T_RSHA"
  echo "OPERATOR=<your operator identity>"
  echo "GATE_SCRIPT_SHA256=$GATE_SHA256"
  echo "REVIEWER=<allowlisted reviewer id>"
  echo "POLICY_VERSION=<policy version, e.g. the registry policy_version>"
  echo "REVIEW_CONTEXT=<fresh-readonly once you checked the reviewer got no writer transcript>"
  echo "# GATE_STATUS: ADVISORY reason=receipt-template"
  exit "$RC_PASS"
fi

# ---------------------------------------------------------------------------
# Prompt + artifact naming (pre, and post when no review exists yet)
# ---------------------------------------------------------------------------

mkdir -p docs/harness/reviews || io_error "cannot create docs/harness/reviews"

DATE="$(date -u +%Y-%m-%d)"
BASE_NAME="${DATE}-${TASK_ID}"

# Find next iteration number for this (date + task)
NEXT_ITER=1
for f in docs/harness/reviews/"${BASE_NAME}"-v*-{pre,post}.md docs/harness/reviews/"${BASE_NAME}"-{pre,post}.md; do
  if [ -f "$f" ]; then
    CANDIDATE=$(echo "$f" | sed -E 's/.*-v([0-9]+)-.*/\1/' | grep -E '^[0-9]+$' || echo 1)
    if [ "$CANDIDATE" -ge "$NEXT_ITER" ]; then
      NEXT_ITER=$((CANDIDATE + 1))
    fi
  fi
done

SUFFIX=""
if [ "$NEXT_ITER" -gt 1 ]; then
  SUFFIX="-v${NEXT_ITER}"
fi

ARTIFACT_TYPE="post"
if [ "$MODE" = "pre" ]; then
  ARTIFACT_TYPE="pre"
fi

ARTIFACT_PATH="docs/harness/reviews/${BASE_NAME}${SUFFIX}-${ARTIFACT_TYPE}.md"
RAW_PATH="${ARTIFACT_PATH}.raw"

write_prompt() { # -> $RAW_PATH
  {
    echo "# Engram Harness — External Reviewer Prompt"
    echo
    echo "**Task**: $TASK_ID"
    echo "**Mode**: $MODE"
    echo "**Date (UTC)**: $DATE"
    echo "**Scope**: $SCOPE_MODE  base=$BASE_SHA  head=$HEAD_SHA  tree=${TREE_SHA:-n/a}"
    echo "**Diff sha256**: $DIFF_SHA256"
    echo
    echo "## Instructions for the Reviewer"
    echo
    echo "You are acting as an independent senior engineer reviewing a diff for the engram project."
    echo "You were NOT the implementer. Your job is to find real problems introduced by the change."
    echo
    echo "Read the following documents (they are the source of truth for this review):"
    echo
    echo "- docs/harness/SPEC.md"
    echo "- docs/harness/INVARIANTS.md (process invariants — canonical)"
    echo "- docs/harness/WHAT_WE_DONT_DO.md (negative scope — no hidden expansion)"
    echo "- docs/harness/GATES.md (especially the fake-success patterns section)"
    echo "- docs/harness/CODE_REVIEW_POLICY.md (this policy)"
    echo "- docs/harness/security/anthropic-reference-harness.md (security boundary)"
    echo "- .claude/scan-extras.txt and .claude/fp-rules.txt (org-specific scan/triage tuning)"
    echo "- docs/harness/README.md (workflow)"
    echo "- Root INVARIANTS.md (data layer invariants for the memory system)"
    echo
    echo "Then review the diff below."
    echo
    echo "Additional harness-specific requirements:"
    echo "- Compare scope against docs/harness/WHAT_WE_DONT_DO.md. Flag hidden scope creep, gate weakening, or product changes bundled into harness work."
    echo "- Security boundary: flag autonomous Engram execution, implied sandboxing, credential mounts, network/egress expansion, or C/C++/ASAN pipeline import unless an ADR and explicit target contract are present."
    echo "- Tuning files: ensure .claude/scan-extras.txt and .claude/fp-rules.txt augment scan/triage behavior without weakening core INVARIANTS/GATES/POLICY or adding blanket suppressions."
    echo "- Review Canvas: if the diff is complex, verify that a matching docs/harness/canvas/YYYY-MM-DD-<task-id>.md exists and includes approaches considered, hot-path complexity, at least two edge cases, and a breakage-risk table."
    echo "- Harness script changes under docs/harness/bin/* are process-critical. Inspect shell safety, path handling, parseability, read-only guarantees, and whether the script weakens any existing gate."
    if [ -n "$HARNESS_SCRIPT_CHANGES" ]; then
      echo
      echo "Harness script changes detected:"
      printf '%s' "$HARNESS_SCRIPT_CHANGES" | sed 's/^/- /'
    fi
    echo
    echo "## Key Fake-Success Patterns (hunt these actively)"
    echo
    echo "1. Tests green only because local-embeddings feature was used; CI Linux parity fails."
    echo "2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes."
    echo "3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated."
    echo "4. Clippy clean but unwrap/expect in hot MCP handler, storage, or hook paths."
    echo "5. Snapshot/attestation tests pass but Merkle or crypto behavior changed."
    echo "6. Hooks (session_end, post_tool_use, etc.) or intelligence modules changed without integration coverage."
    echo "7. Harness doctor or sensors would have caught this but were not run."
    echo "8. Progress docs (harness or active plan) not updated for a domain change."
    echo "9. Cross-SDK (python/typescript) contract drift not reflected."
    echo "10. Reviewer is being shown a self-referential or incomplete prompt (call it out)."
    echo "11. Security boundary drift: static/read-only default weakened, autonomous execution implied, missing ADR/sandbox/egress/target contract, credential mounts allowed, or Anthropic C/C++/ASAN pipeline imported as default."
    echo
    echo "## Files In Scope"
    echo
    print_paths CHANGED_PATH "$WORK/rpaths.z"
    if [ "$EXCLUDED_COUNT" -gt 0 ]; then
      echo
      echo "Paths changed but excluded from this diff (harness bookkeeping, not reviewed here):"
      print_paths EXCLUDED_PATH "$WORK/epaths.z"
    fi
    echo
    echo "## Diff Under Review"
    echo
    echo '```diff'
    cat "$WORK/diff.txt"
    echo '```'
    echo
    echo "## Previous Review Context (if any)"
    echo
    if [ -n "$PREV_REVIEW" ] && [ -f "$PREV_REVIEW" ]; then
      echo "Previous review file: $PREV_REVIEW"
      echo "(Only [BLOCKER] and [HIGH] findings from a prior FAIL are carried; PASS/LOW are not.)"
      echo '```'
      grep -E '^\[BLOCKER\]|\[HIGH\]' "$PREV_REVIEW" | head -20 || echo "(no high-severity carried findings parsed)"
      echo '```'
    else
      echo "(no previous review supplied for continuity)"
    fi
    echo
    echo "## Output Contract (strict)"
    echo
    echo "Your entire response must start with exactly one of:"
    echo
    echo "PASS <one-line summary of what was reviewed and why it is safe>"
    echo
    echo "or"
    echo
    echo "FAIL <one-line summary of the most important problem(s)>"
    echo
    echo "Then a short bullet list using [BLOCKER], [HIGH], [MED], [LOW]."
    echo "At most 3 substantive findings. Evidence and location required for each."
    echo "If nothing substantive: exactly one bullet with [LOW] No issues found..."
    echo
    echo "Remember: you are the external reviewer. Be evidence-driven and skeptical."
    echo
    echo "Machine-parseable verdict (required):"
    echo "Add exactly one line, anywhere in the response, beginning with:"
    echo "REVIEW_VERDICT: PASS <one-line summary>"
    echo "or"
    echo "REVIEW_VERDICT: FAIL <one-line summary>"
    echo "This line is required for hard post-gate enforcement. A response with zero or"
    echo "more than one such line is rejected."
  } >"$RAW_PATH" || io_error "cannot write review prompt $RAW_PATH"
  echo "Review prompt written to: $RAW_PATH"
  echo "Artifact target: $ARTIFACT_PATH"
  echo
}

print_scope_summary() {
  echo "Scope ($SCOPE_MODE): base=$BASE_SHA head=$HEAD_SHA tree=${TREE_SHA:-n/a}"
  echo "Reviewed diff sha256: $DIFF_SHA256 ($CHANGED_COUNT paths; $EXCLUDED_COUNT excluded)"
  if [ -n "$HARNESS_SCRIPT_CHANGES" ]; then
    echo "Harness script changes under docs/harness/bin/* (always require an independent reviewer):"
    printf '%s' "$HARNESS_SCRIPT_CHANGES" | sed 's/^/- /'
  fi
}

# ---------------------------------------------------------------------------
# PRE — advisory
# ---------------------------------------------------------------------------

if [ "$MODE" = "pre" ]; then
  write_prompt
  print_scope_summary
  if [ "$CHANGED_COUNT" -eq 0 ]; then
    echo "WARNING: the preparation scope is empty. Use --range BASE..CANDIDATE to review committed work."
  fi
  echo "=== PRE-GATE (advisory) ==="
  echo "The prompt above is advisory input for the implementer. It approves nothing."
  echo "Copy the content of $RAW_PATH into your separate Claude Code Sonnet reviewer session if doing cross-CLI review."
  echo "Save the reviewer's full response as $ARTIFACT_PATH"
  cp "$RAW_PATH" "$ARTIFACT_PATH" || io_error "cannot save pre-gate artifact $ARTIFACT_PATH"
  echo "Pre-gate artifact (prompt copy) saved to $ARTIFACT_PATH"
  echo "GATE_STATUS: ADVISORY reason=pre-never-approves"
  exit "$RC_PASS"
fi

# ---------------------------------------------------------------------------
# POST — hard gate. Scope is already computed (failures above exited 4, before any verdict).
# ---------------------------------------------------------------------------

echo "=== POST-GATE (hard, fail-closed) ==="
print_scope_summary
echo "Gate script sha256: $GATE_SHA256"
echo

if [ "$SKIP_ALLOWLIST" = "yes" ]; then
  echo "Every changed path is docs-only per the GATES.md skip allowlist (docs/**/*.md outside docs/harness/)."
  echo "No independent review is required; this is NOT a review PASS."
  echo "GATE_STATUS: SKIPPED_ALLOWLIST reason=docs-only"
  exit "$RC_PASS"
fi

if [ -z "$REVIEW_FILE" ]; then
  write_prompt
  echo "No --review-file was supplied. Dual-CLI workflow:"
  echo "  1. Open $RAW_PATH"
  echo "  2. Start a separate Claude Code reviewer session with --model sonnet, then provide the full prompt"
  echo "  3. Save the complete reviewer response to $ARTIFACT_PATH"
  echo "  4. Operator: write the receipt (GATES.md runbook), then re-run with --review-file/--receipt/--expect-*"
  pending review-missing "no review artifact supplied for $BASE_SHA..$HEAD_SHA"
fi

case "$REVIEW_FILE" in /*) ;; *) REVIEW_FILE="$REPO_ROOT/$REVIEW_FILE" ;; esac
if [ ! -f "$REVIEW_FILE" ]; then
  write_prompt
  pending review-missing "review artifact not found: $REVIEW_FILE"
fi
if [ -L "$REVIEW_FILE" ]; then
  pending review-not-regular "review artifact is a symlink: $REVIEW_FILE"
fi

# --- Verdict marker (legacy parser kept: explicit REVIEW_VERDICT line only) -------------------
MARKERS="$(tr -d '\r' <"$REVIEW_FILE" |
  grep -Ei '^REVIEW_VERDICT:[[:space:]]*(PASS|FAIL)[[:space:]].+$' |
  grep -Eiv '^REVIEW_VERDICT:[[:space:]]*(PASS|FAIL)[[:space:]]+<one-line summary>[[:space:]]*$' || true)"
if [ -z "$MARKERS" ]; then
  echo "POST-GATE: no explicit review marker found in $REVIEW_FILE"
  echo "Expected exactly one line matching:"
  echo "REVIEW_VERDICT: PASS <one-line summary>   or   REVIEW_VERDICT: FAIL <one-line summary>"
  echo "Prompt-only artifacts, prose PASS/FAIL and unfilled placeholders are not valid post-gate inputs."
  echo "GATE_STATUS: INVALID_REVIEW reason=no-valid-marker"
  exit "$RC_FAIL"
fi
if [ "$(printf '%s\n' "$MARKERS" | grep -c .)" -ne 1 ]; then
  echo "POST-GATE: $REVIEW_FILE has more than one REVIEW_VERDICT marker; the verdict is ambiguous."
  echo "GATE_STATUS: INVALID_REVIEW reason=ambiguous-marker"
  exit "$RC_FAIL"
fi
VERDICT_LINE="$MARKERS"
VERDICT="$(printf '%s' "$VERDICT_LINE" | sed -E 's/^REVIEW_VERDICT:[[:space:]]*(PASS|FAIL).*/\1/I' | tr '[:lower:]' '[:upper:]')"
echo "Verdict marker: $VERDICT_LINE"

if [ "$VERDICT" = "FAIL" ]; then
  echo "POST-GATE FAIL (from $REVIEW_FILE)"
  echo
  echo "Findings (last 30 lines of artifact for context):"
  tail -30 "$REVIEW_FILE"
  echo "GATE_STATUS: FAIL reason=reviewer-fail"
  exit "$RC_FAIL"
fi
if [ "$VERDICT" != "PASS" ]; then
  echo "GATE_STATUS: INVALID_REVIEW reason=malformed-marker"
  exit "$RC_FAIL"
fi

# --- PASS marker: legacy marker alone is history. Require trusted, bound provenance. ----------
# 1. Operator expectations (command line = operator authority, not writer-controlled files).
[ -n "$RECEIPT_ARG" ] || pending receipt-missing "no receipt (--receipt / ENGRAM_REVIEW_RECEIPT); a PASS marker alone is history only"

# 2. Receipt location / permissions.
case "$RECEIPT_ARG" in /*) ;; *) pending receipt-untrusted-location "receipt path must be absolute (got '$RECEIPT_ARG')" ;; esac
[ -e "$RECEIPT_ARG" ] || pending receipt-missing "receipt file not found: $RECEIPT_ARG"
[ ! -L "$RECEIPT_ARG" ] || pending receipt-untrusted-location "receipt is a symlink: $RECEIPT_ARG"
[ -f "$RECEIPT_ARG" ] || pending receipt-untrusted-location "receipt is not a regular file: $RECEIPT_ARG"
RECEIPT_PHYS="$(phys_path "$RECEIPT_ARG")" || pending receipt-untrusted-location "cannot resolve receipt path"
RECEIPT_DIR="$(dirname "$RECEIPT_PHYS")"

UNTRUSTED_ROOTS=("$REPO_ROOT")
GIT_COMMON="$(gitx rev-parse --git-common-dir 2>/dev/null)" || pending receipt-untrusted-location "cannot resolve git common dir"
UNTRUSTED_ROOTS+=("$(cd "$GIT_COMMON" && pwd -P)")
UNTRUSTED_ROOTS+=("$(cd "$(gitx rev-parse --git-dir 2>/dev/null)" && pwd -P)")
if ! gitx worktree list --porcelain -z >"$WORK/worktrees.z" 2>/dev/null; then
  pending receipt-untrusted-location "cannot enumerate worktrees"
fi
while IFS= read -r -d '' rec; do
  case "$rec" in "worktree "*) UNTRUSTED_ROOTS+=("$(cd "${rec#worktree }" 2>/dev/null && pwd -P || echo "${rec#worktree }")") ;; esac
done <"$WORK/worktrees.z"
# Sets UNTRUSTED_HIT and returns 0 when DIR, or any ancestor of it, is the same filesystem object
# as an untrusted root (identity via -ef: pwd -P keeps typed case on case-insensitive filesystems).
inside_untrusted_root() {
  local anc="$1" root parent
  while :; do
    for root in "${UNTRUSTED_ROOTS[@]}"; do
      if [ "$anc" -ef "$root" ]; then UNTRUSTED_HIT="$root"; return 0; fi
    done
    parent="$(dirname "$anc")"
    [ "$parent" = "$anc" ] && break
    anc="$parent"
  done
  return 1
}

# Compare by filesystem identity, not by spelling: pwd -P keeps the typed case on
# case-insensitive filesystems, so a string prefix test is bypassable (REPO vs Repo).
for root in "${UNTRUSTED_ROOTS[@]}"; do
  if path_within "$RECEIPT_PHYS" "$root"; then
    pending receipt-untrusted-location "receipt lives inside a repository/worktree ($root); it must be stored outside any writer-writable path"
  fi
done
if inside_untrusted_root "$RECEIPT_DIR"; then
  pending receipt-untrusted-location "receipt lives inside a repository/worktree ($UNTRUSTED_HIT); it must be stored outside any writer-writable path"
fi
# A hard link would let the writer edit the receipt through the repository copy.
if [ -n "$(find "$RECEIPT_PHYS" -maxdepth 0 -links +1 2>/dev/null)" ]; then
  pending receipt-untrusted-location "receipt has more than one hard link"
fi

# shellcheck disable=SC2012  # permission string of one known path; no filename parsing
perm_chars() { ls -ld -- "$1" | cut -c1-11; }
# True when an ACL on the path grants write-like access, or cannot be inspected (fail closed).
# macOS prints "@" instead of "+" when a path has both xattrs and an ACL, so callers inspect both.
acl_grants_write() {
  local out
  out="$(ls -lde -- "$1" 2>/dev/null)" || return 0
  printf '%s\n' "$out" | sed 1d | grep -Eq 'allow .*(write|append|delete|add_file|add_subdirectory|chown)'
}
fperm="$(perm_chars "$RECEIPT_PHYS")"
dperm="$(perm_chars "$RECEIPT_DIR")"
if [ ! -O "$RECEIPT_PHYS" ] || [ "${fperm:5:1}" = "w" ] || [ "${fperm:8:1}" = "w" ]; then
  pending receipt-untrusted-permissions "receipt must be owned by the invoking operator and not group/other-writable ($fperm)"
fi
if { [ "${fperm:10:1}" = "+" ] || [ "${fperm:10:1}" = "@" ]; } && acl_grants_write "$RECEIPT_PHYS"; then
  pending receipt-untrusted-permissions "receipt carries an ACL granting write access"
fi
if { [ "${dperm:10:1}" = "+" ] || [ "${dperm:10:1}" = "@" ]; } && acl_grants_write "$RECEIPT_DIR"; then
  pending receipt-untrusted-permissions "receipt directory carries an ACL granting write access"
fi
if [ ! -O "$RECEIPT_DIR" ] || [ "${dperm:5:1}" = "w" ] || { [ "${dperm:8:1}" = "w" ] && [ "${dperm:9:1}" != "t" ] && [ "${dperm:9:1}" != "T" ]; }; then
  pending receipt-untrusted-permissions "receipt directory must be owned by the operator and not group/other-writable ($dperm)"
fi

# The gate script must not be writer-controlled either (a modified gate cannot judge itself).
GATE_PHYS="$(phys_path "$GATE_SCRIPT")" || pending gate-untrusted-location "cannot resolve the gate script path"
if inside_untrusted_root "$(dirname "$GATE_PHYS")"; then
  pending gate-untrusted-location "gate script runs from inside a worktree ($UNTRUSTED_HIT); run a trusted copy kept outside every worktree with --repo"
fi

# 3. Strict receipt parse.
R_GATE="" R_VERSION="" R_TASK="" R_BASE="" R_HEAD="" R_TREE="" R_DIFF="" R_REVIEW="" R_RSHA="" R_OPERATOR=""
R_REVIEWER="" R_POLICY="" R_CONTEXT=""
seen=" "
while IFS= read -r line || [ -n "$line" ]; do
  case "$line" in '' | '#'*) continue ;; esac
  case "$line" in *=*) ;; *) pending receipt-malformed "receipt line is not KEY=VALUE" ;; esac
  key="${line%%=*}"
  val="${line#*=}"
  case "$seen" in *" $key "*) pending receipt-malformed "duplicate key $key" ;; esac
  seen="$seen$key "
  case "$val" in '<'*'>') pending receipt-malformed "unfilled <placeholder> in $key" ;; esac
  case "$key" in
    RECEIPT_VERSION) R_VERSION="$val" ;;
    TASK_ID) R_TASK="$val" ;;
    BASE_SHA) R_BASE="$val" ;;
    HEAD_SHA) R_HEAD="$val" ;;
    TREE_SHA) R_TREE="$val" ;;
    DIFF_SHA256) R_DIFF="$val" ;;
    REVIEW_ARTIFACT) R_REVIEW="$val" ;;
    REVIEW_SHA256) R_RSHA="$val" ;;
    OPERATOR) R_OPERATOR="$val" ;;
    GATE_SCRIPT_SHA256) R_GATE="$val" ;;
    REVIEWER) R_REVIEWER="$val" ;;
    POLICY_VERSION) R_POLICY="$val" ;;
    REVIEW_CONTEXT) R_CONTEXT="$val" ;;
    *) pending receipt-malformed "unknown key $key" ;;
  esac
done <"$RECEIPT_PHYS"
V1_KEYS="RECEIPT_VERSION TASK_ID BASE_SHA HEAD_SHA TREE_SHA DIFF_SHA256 REVIEW_ARTIFACT"
V1_KEYS="$V1_KEYS REVIEW_SHA256 OPERATOR GATE_SCRIPT_SHA256"
V2_EXTRA="REVIEWER POLICY_VERSION REVIEW_CONTEXT"
case "$R_VERSION" in
  1) REQUIRED_KEYS="$V1_KEYS" FORBIDDEN_KEYS="$V2_EXTRA" ;;
  2) REQUIRED_KEYS="$V1_KEYS $V2_EXTRA" FORBIDDEN_KEYS="" ;;
  *) pending receipt-malformed "unsupported RECEIPT_VERSION" ;;
esac
for key in $REQUIRED_KEYS; do
  case "$seen" in *" $key "*) ;; *) pending receipt-malformed "missing key $key" ;; esac
done
for key in $FORBIDDEN_KEYS; do
  case "$seen" in *" $key "*) pending receipt-malformed "key $key needs RECEIPT_VERSION=2" ;; esac
done
if [ "$R_VERSION" = "2" ]; then
  IDENT_RE='^[A-Za-z0-9][A-Za-z0-9._@-]{0,127}$'
  [[ "$R_REVIEWER" =~ $IDENT_RE ]] || pending receipt-malformed "REVIEWER is malformed"
  [[ "$R_POLICY" =~ ^[a-z0-9][a-z0-9._-]{0,63}$ ]] ||
    pending receipt-malformed "POLICY_VERSION is malformed"
  [[ "$R_CONTEXT" =~ $IDENT_RE ]] || pending receipt-malformed "REVIEW_CONTEXT is malformed"
fi
[[ "$R_BASE" =~ ^[0-9a-f]{40}([0-9a-f]{24})?$ ]] || pending receipt-malformed "BASE_SHA is not a full object id"
[[ "$R_HEAD" =~ ^[0-9a-f]{40}([0-9a-f]{24})?$ ]] || pending receipt-malformed "HEAD_SHA is not a full object id"
[[ "$R_TREE" =~ ^[0-9a-f]{40}([0-9a-f]{24})?$ ]] || pending receipt-malformed "TREE_SHA is not a full object id"
[[ "$R_DIFF" =~ ^[0-9a-f]{64}$ ]] || pending receipt-malformed "DIFF_SHA256 is not a sha256"
[[ "$R_RSHA" =~ ^[0-9a-f]{64}$ ]] || pending receipt-malformed "REVIEW_SHA256 is not a sha256"
[[ "$R_GATE" =~ ^[0-9a-f]{64}$ ]] || pending receipt-malformed "GATE_SCRIPT_SHA256 is not a sha256"
[[ "$R_OPERATOR" =~ ^[[:print:]]{1,128}$ ]] || pending receipt-malformed "OPERATOR must be 1-128 printable characters"
[ -n "$R_REVIEW" ] || pending receipt-malformed "REVIEW_ARTIFACT is empty"

# 4. Operator-supplied expected values must be present...
if [ -z "$EXPECT_TREE" ] || [ -z "$EXPECT_DIFF" ] || [ -z "$EXPECT_GATE" ] || [ -z "$OPERATOR_ARG" ]; then
  pending operator-expectations-missing "operator must pass --expect-tree, --expect-diff-sha256, --expect-gate-sha256 and --operator"
fi

# 5. ...and the receipt, the operator and the recomputed scope must all agree.
[ "$R_TASK" = "$TASK_ID" ] || pending receipt-task-mismatch "receipt task '$R_TASK' != '$TASK_ID'"
[ "$R_BASE" = "$BASE_SHA" ] || pending receipt-base-mismatch "receipt base $R_BASE != recomputed $BASE_SHA"
[ "$R_HEAD" = "$HEAD_SHA" ] || pending receipt-head-mismatch "receipt head $R_HEAD != recomputed $HEAD_SHA (stale review?)"
[ "$R_TREE" = "$TREE_SHA" ] || pending receipt-tree-mismatch "receipt tree $R_TREE != recomputed $TREE_SHA"
[ "$R_DIFF" = "$DIFF_SHA256" ] || pending receipt-diff-mismatch "receipt diff sha256 $R_DIFF != recomputed $DIFF_SHA256"
[ "$EXPECT_TREE" = "$TREE_SHA" ] || pending operator-expectation-mismatch "operator expected tree $EXPECT_TREE != recomputed $TREE_SHA"
[ "$EXPECT_DIFF" = "$DIFF_SHA256" ] || pending operator-expectation-mismatch "operator expected diff sha256 $EXPECT_DIFF != recomputed $DIFF_SHA256"
[ "$R_GATE" = "$GATE_SHA256" ] || pending receipt-gate-mismatch "receipt binds gate script $R_GATE, this gate is $GATE_SHA256"
[ "$EXPECT_GATE" = "$GATE_SHA256" ] || pending operator-expectation-mismatch "operator expected gate sha256 $EXPECT_GATE != this gate $GATE_SHA256"
[ "$R_OPERATOR" = "$OPERATOR_ARG" ] || pending receipt-operator-mismatch "receipt operator '$R_OPERATOR' != operator '$OPERATOR_ARG'"

case "$R_REVIEW" in /*) R_REVIEW_ABS="$R_REVIEW" ;; *) R_REVIEW_ABS="$REPO_ROOT/$R_REVIEW" ;; esac
R_REVIEW_PHYS="$(phys_path "$R_REVIEW_ABS")" || pending receipt-review-path-mismatch "receipt review artifact path does not resolve"
REVIEW_PHYS="$(phys_path "$REVIEW_FILE")" || pending receipt-review-path-mismatch "review artifact path does not resolve"
[ "$R_REVIEW_PHYS" = "$REVIEW_PHYS" ] || pending receipt-review-path-mismatch "receipt binds review artifact $R_REVIEW_PHYS, not $REVIEW_PHYS"
[ "$(sha256_file "$REVIEW_FILE")" = "$R_RSHA" ] || pending receipt-review-hash-mismatch "review artifact bytes changed after the receipt was issued"

echo "POST-GATE PASS (from $REVIEW_FILE)"
echo "Receipt $RECEIPT_PHYS verified for operator $R_OPERATOR; scope $BASE_SHA..$HEAD_SHA tree $TREE_SHA"
echo "GATE_STATUS: PASS reason=receipt-verified"
exit "$RC_PASS"
