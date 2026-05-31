#!/usr/bin/env bash
# docs/harness/bin/check-commit-msg.sh
#
# Lightweight Conventional Commit checker with engram/harness scopes.
# Used by pre-commit and manually before `git commit`.

set -euo pipefail

MSG=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    --message)
      MSG="$2"
      shift 2
      ;;
    *)
      # If a file path is passed (git hook style), read it
      if [ -f "$1" ]; then
        MSG="$(cat "$1")"
      fi
      shift
      ;;
  esac
done

if [ -z "$MSG" ]; then
  echo "Usage: check-commit-msg.sh --message 'type(scope): subject'  or  path/to/COMMIT_EDITMSG" >&2
  exit 2
fi

# Strip comment lines, take the first remaining line, then trim whitespace.
CLEAN_MSG="$(printf '%s\n' "$MSG" | sed '/^#/d' | sed -n '1p')"
CLEAN_MSG="${CLEAN_MSG#"${CLEAN_MSG%%[![:space:]]*}"}"
CLEAN_MSG="${CLEAN_MSG%"${CLEAN_MSG##*[![:space:]]}"}"

# Allowed types (extend as needed)
TYPES='feat|fix|docs|refactor|test|perf|ci|chore|revert|style|build'

# Allowed scopes (harness + common engram areas + explicit tracker ids)
SCOPES='harness|mcp|storage|search|intelligence|hooks|sdk-python|sdk-ts|cli|server|watcher|embedding|graph|sync|snapshot|attestation|ci|docs|infra|engra-[0-9]+|rfc-[0-9]+'

if echo "$CLEAN_MSG" | grep -qE "^(${TYPES})\((${SCOPES})\): .+"; then
  echo "OK commit message: $CLEAN_MSG"
  exit 0
else
  echo "FAIL commit message does not match required format."
  echo "Expected: type(scope): concise subject"
  echo "Allowed types: $TYPES"
  echo "Recommended scopes: harness, mcp, storage, search, intelligence, hooks, sdk-python, sdk-ts, cli, server, ci, or a task id (engra-xxx, rfc-0001, etc.)"
  echo "Got: $CLEAN_MSG"
  exit 1
fi
