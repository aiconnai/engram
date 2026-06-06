#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  vc-gate.sh status [ISSUE]
  vc-gate.sh start ISSUE [--allow-dirty-current-issue]
  vc-gate.sh done ISSUE [--allow-dirty]
  vc-gate.sh release VERSION|vVERSION [--allow-untagged]

Purpose:
  Lightweight version-control boundary checks for issue work and releases.

Policy:
  - jj may be used for local issue evolution.
  - Git remains canonical for release commits, tags, and cargo publish.
  - This script observes and blocks unsafe states; it does not create commits,
    run jj new, move tags, or publish.
USAGE
}

die() {
  echo "vc-gate: FAIL: $*" >&2
  exit 1
}

note() {
  echo "vc-gate: $*"
}

mode="${1:-status}"
if [[ "$mode" == "-h" || "$mode" == "--help" ]]; then
  usage
  exit 0
fi
shift || true

case "$mode" in
  status|start|done|release) ;;
  *) usage >&2; die "unknown mode: $mode" ;;
esac

allow_dirty=0
allow_untagged=0
subject="${1:-}"

if [[ -n "${subject:-}" && "$subject" != --* ]]; then
  shift || true
else
  subject=""
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --allow-dirty|--allow-dirty-current-issue)
      allow_dirty=1
      ;;
    --allow-untagged)
      allow_untagged=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
  shift
done

git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
[[ -n "$git_root" ]] || die "not inside a git repository"
cd "$git_root"

git_branch="$(git branch --show-current 2>/dev/null || true)"
git_head="$(git rev-parse --short HEAD 2>/dev/null || true)"
git_status="$(git status --porcelain=v1 2>/dev/null || true)"
dirty_count="$(printf '%s\n' "$git_status" | sed '/^$/d' | wc -l | tr -d ' ')"
untracked_count="$(printf '%s\n' "$git_status" | awk '/^\?\?/ { count++ } END { print count + 0 }')"

jj_available=0
jj_repo=0
jj_description=""
if command -v jj >/dev/null 2>&1; then
  jj_available=1
  if jj root >/dev/null 2>&1; then
    jj_repo=1
    jj_description="$(jj log -r @ --no-graph -T 'description' 2>/dev/null || true)"
  fi
fi

print_status() {
  note "mode=$mode"
  [[ -n "$subject" ]] && note "subject=$subject"
  note "git_root=$git_root"
  note "git_branch=${git_branch:-detached}"
  note "git_head=${git_head:-unknown}"
  note "dirty_files=$dirty_count"
  note "untracked_files=$untracked_count"
  note "jj_available=$jj_available"
  note "jj_repo=$jj_repo"
  if [[ "$jj_repo" -eq 1 && -n "$jj_description" ]]; then
    note "jj_current_description=$(printf '%s' "$jj_description" | head -n 1)"
  fi
}

require_issue() {
  [[ -n "$subject" ]] || die "$mode requires an ISSUE id or short task id"
}

require_clean_unless_allowed() {
  if [[ "$dirty_count" -gt 0 && "$allow_dirty" -ne 1 ]]; then
    printf '%s\n' "$git_status" >&2
    die "worktree is dirty; commit/stash/split current work or pass an explicit allow flag"
  fi
}

latest_git_mentions_issue() {
  local issue="$1"
  git log --oneline -30 2>/dev/null | grep -Eiq "(^|[^A-Za-z0-9_-])${issue}([^A-Za-z0-9_-]|$)"
}

jj_current_mentions_issue() {
  local issue="$1"
  [[ "$jj_repo" -eq 1 ]] || return 1
  printf '%s\n' "$jj_description" | grep -Eiq "(^|[^A-Za-z0-9_-])${issue}([^A-Za-z0-9_-]|$)"
}

check_release_version() {
  local requested="$1"
  [[ -n "$requested" ]] || die "release requires VERSION or vVERSION"

  local version="${requested#v}"
  local manifest_version=""
  local tag="v$version"

  if [[ -f Cargo.toml ]]; then
    manifest_version="$(
      awk '
        $0 == "[package]" { in_package=1; next }
        /^\[/ && in_package { exit }
        in_package && $1 == "version" {
          gsub(/"/, "", $3)
          print $3
          exit
        }
      ' Cargo.toml
    )"
    if [[ -n "$manifest_version" && "$manifest_version" != "$version" ]]; then
      die "Cargo.toml version $manifest_version does not match requested release $version"
    fi
  fi

  if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
    local tag_target
    local head_full
    tag_target="$(git rev-list -n 1 "$tag")"
    head_full="$(git rev-parse HEAD)"
    if [[ "$tag_target" != "$head_full" ]]; then
      die "tag $tag exists but does not point to HEAD"
    fi
    note "release_tag=$tag points_to=HEAD"
  elif [[ "$allow_untagged" -eq 1 ]]; then
    note "release_tag=$tag missing; allowed by --allow-untagged"
  else
    die "release tag $tag is missing; create it after dry-run or pass --allow-untagged for pre-tag checks"
  fi
}

print_status

case "$mode" in
  status)
    if [[ "$jj_available" -eq 1 && "$jj_repo" -ne 1 ]]; then
      note "jj_hint=jj is installed; use jj git init --colocate only after team agreement"
    fi
    ;;
  start)
    require_issue
    require_clean_unless_allowed
    if [[ "$jj_repo" -eq 1 ]]; then
      note "jj_hint=use: jj new -m \"<type>(<scope>): $subject <summary>\""
    else
      note "git_hint=use a task branch or explicit commit series for $subject"
    fi
    ;;
  done)
    require_issue
    require_clean_unless_allowed
    if latest_git_mentions_issue "$subject" || jj_current_mentions_issue "$subject"; then
      note "issue_evidence=found"
    else
      die "no recent Git commit or current jj change description mentions $subject"
    fi
    ;;
  release)
    require_clean_unless_allowed
    check_release_version "$subject"
    note "release_gate=pass"
    ;;
esac
