#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

usage() {
  cat <<'EOF'
Usage:
  bash docs/harness/bin/check-pr-title.sh --title "concise PR title"
  bash docs/harness/bin/check-pr-title.sh --pr 123

Options:
  --title TITLE  Validate the provided PR title.
  --pr NUMBER    Read a PR title with `gh pr view NUMBER` and validate it.
  -h, --help     Show this help.
EOF
}

TITLE=""
PR_NUMBER=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --title)
      if [ -n "$TITLE" ] || [ -n "$PR_NUMBER" ]; then
        echo "ERROR: --title cannot be combined with another validation argument" >&2
        exit 2
      fi
      if [ "$#" -lt 2 ]; then
        echo "ERROR: --title requires a value" >&2
        exit 2
      fi
      TITLE="$2"
      shift 2
      ;;
    --pr)
      if [ -n "$TITLE" ] || [ -n "$PR_NUMBER" ]; then
        echo "ERROR: --pr cannot be combined with another validation argument" >&2
        exit 2
      fi
      if [ "$#" -lt 2 ]; then
        echo "ERROR: --pr requires a value" >&2
        exit 2
      fi
      PR_NUMBER="$2"
      shift 2
      ;;
    -h|--help)
      if [ "$#" -ne 1 ] || [ -n "$TITLE" ] || [ -n "$PR_NUMBER" ]; then
        echo "ERROR: --help cannot be combined with validation arguments" >&2
        exit 2
      fi
      usage
      exit 0
      ;;
    *)
      if [ -n "$TITLE" ] || [ -n "$PR_NUMBER" ]; then
        echo "ERROR: unexpected argument: $1" >&2
        usage >&2
        exit 2
      fi
      TITLE="$1"
      shift
      ;;
  esac
done

if [ -n "$TITLE" ] && [ -n "$PR_NUMBER" ]; then
  echo "ERROR: use either --title or --pr, not both" >&2
  exit 2
fi

if [ -n "$PR_NUMBER" ]; then
  case "$PR_NUMBER" in
    ''|*[!0-9]*)
      echo "ERROR: PR number must contain digits only." >&2
      exit 2
      ;;
  esac

  if ! command -v gh >/dev/null 2>&1; then
    echo "ERROR: gh is required for --pr validation" >&2
    exit 3
  fi
  TITLE="$(gh pr view "$PR_NUMBER" --json title --jq '.title')"
fi

TRIMMED_TITLE="${TITLE#"${TITLE%%[![:space:]]*}"}"
TRIMMED_TITLE="${TRIMMED_TITLE%"${TRIMMED_TITLE##*[![:space:]]}"}"

if [ -z "$TRIMMED_TITLE" ]; then
  echo "ERROR: PR title is empty." >&2
  exit 2
fi

bash docs/harness/bin/pr-title-policy.sh --title "$TRIMMED_TITLE"
