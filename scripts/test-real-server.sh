#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/.."

case "${1:-}" in
  "")
    rtk cargo test --test real_server_harness -- --nocapture
    ;;
  --self-test-bad-executable)
    rtk cargo test --test real_server_harness bad_executable_self_test_returns_bounded_error_and_cleans_tempdir -- --nocapture
    ;;
  *)
    printf 'usage: %s [--self-test-bad-executable]\n' "$0" >&2
    exit 64
    ;;
esac
