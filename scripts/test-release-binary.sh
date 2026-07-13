#!/usr/bin/env bash
set -euo pipefail

die() { echo "test-release-binary: $*" >&2; exit 1; }

host_target() {
  local os arch
  os="$(uname -s)"; arch="$(uname -m)"
  case "$os/$arch" in
    Darwin/arm64) echo aarch64-apple-darwin ;;
    Darwin/x86_64) echo x86_64-apple-darwin ;;
    Linux/aarch64|Linux/arm64) echo aarch64-unknown-linux-gnu ;;
    Linux/x86_64) echo x86_64-unknown-linux-gnu ;;
    *) die "unsupported host: $os/$arch" ;;
  esac
}

artifact_dir=''
run_id=''
target=''
allow_cross=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir) artifact_dir=${2:-}; shift 2 ;;
    --run-id) run_id=${2:-}; shift 2 ;;
    --target) target=${2:-}; shift 2 ;;
    --allow-cross) allow_cross=true; shift ;;
    *) die "unknown argument: $1" ;;
  esac
done
cleanup=
if [[ -n "$run_id" ]]; then
  cleanup="$(mktemp -d)"
  trap 'rm -rf "${cleanup:-}"' EXIT
  gh run download "$run_id" --dir "$cleanup"
  artifact_dir=$cleanup
fi
[[ -d "$artifact_dir" ]] || die "--artifact-dir or --run-id is required"
host="$(host_target)"
[[ -n "$target" ]] || target=$host
archives=()
while IFS= read -r path; do archives+=("$path"); done < <(find "$artifact_dir" -type f -name "engram-v*-${target}.tar.gz" | sort)
if [[ ${#archives[@]} -ne 1 ]]; then
  die "expected one archive for $target, found ${#archives[@]}"
fi
if [[ "$target" != "$host" ]]; then
  [[ "$allow_cross" == true ]] || die "cannot execute $target archive on $host"
  echo "test-release-binary: cross target $target inspected but not executed"
  exit 0
fi
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp" "${cleanup:-}"' EXIT
tar -xzf "${archives[0]}" -C "$tmp"
expected=(engram-server engram-cli)
[[ "$target" == *-linux-* ]] && expected+=(engram-pdf-worker)
for binary in "${expected[@]}"; do
  [[ -x "$tmp/$binary" ]] || die "missing executable: $binary"
  case "$binary" in
    engram-cli)
      output="$("$tmp/$binary" --version 2>&1)" || die "$binary --version failed"
      [[ -n "$output" ]] || die "$binary --version returned empty output"
      ;;
    engram-server)
      output="$("$tmp/$binary" --help 2>&1)" || die "$binary --help failed"
      [[ "$output" == *"Usage:"* ]] || die "$binary --help returned unexpected output"
      ;;
    engram-pdf-worker)
      output="$("$tmp/$binary" --max-pages 1 --max-text-bytes 1024 </dev/null 2>&1)" \
        || die "$binary protocol smoke failed"
      printf '%s' "$output" | python3 -c '
import json, sys
response = json.load(sys.stdin)
if not isinstance(response.get("sections"), list) or "error" not in response:
    raise SystemExit("unexpected PDF worker response")
' || die "$binary protocol smoke returned unexpected output"
      ;;
  esac
done
echo "test-release-binary: PASS ($target)"
