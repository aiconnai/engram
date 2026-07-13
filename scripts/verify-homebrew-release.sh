#!/usr/bin/env bash
set -euo pipefail

BREW_COMMIT=999134536e623073ea9b2a8954eeea5898137239
die() { echo "verify-homebrew-release: $*" >&2; exit 1; }

validate_version() {
  [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "invalid stable version"
}

run_isolated_command() {
  local root status
  root="$(mktemp -d)"
  set +e
  ENGRAM_DISPOSABLE_PREFIX="$root" HOME="$root/home" HOMEBREW_CACHE="$root/cache" "$@"
  status=$?
  set -e
  rm -rf "$root"
  [[ ! -e "$root" ]] || die "disposable prefix cleanup failed"
  return "$status"
}

prefix_probe() {
  [[ -n "${ENGRAM_DISPOSABLE_PREFIX:-}" ]] || return 1
  [[ "$HOME" == "$ENGRAM_DISPOSABLE_PREFIX/home" ]] || return 1
  [[ "$HOMEBREW_CACHE" == "$ENGRAM_DISPOSABLE_PREFIX/cache" ]] || return 1
  mkdir -p "$HOME" "$HOMEBREW_CACHE"
  printf '%s\n' "$ENGRAM_DISPOSABLE_PREFIX" > "$ENGRAM_DISPOSABLE_PREFIX/probe"
}

self_test() {
  validate_version 0.22.0
  if (validate_version '0.22.0;rm') >/dev/null 2>&1; then
    die "self-test accepted unsafe version"
  fi
  echo "verify-homebrew-release self-test: PASS"
}

self_test_prefix() {
  run_isolated_command "$0" --prefix-probe
  echo "verify-homebrew-release prefix isolation self-test: PASS"
}

verify_isolated() {
  local version=$1 prefix="$ENGRAM_DISPOSABLE_PREFIX/brew"
  mkdir -p "$HOME" "$HOMEBREW_CACHE"
  git clone --filter=blob:none https://github.com/Homebrew/brew.git "$prefix"
  git -C "$prefix" checkout --detach "$BREW_COMMIT"
  export HOMEBREW_NO_AUTO_UPDATE=1 HOMEBREW_NO_ANALYTICS=1 HOMEBREW_NO_ENV_HINTS=1
  "$prefix/bin/brew" tap aiconnai/engram
  "$prefix/bin/brew" install "aiconnai/engram/engram@${version}" 2>/dev/null \
    || "$prefix/bin/brew" install aiconnai/engram/engram
  output="$("$prefix/bin/engram-server" --version 2>&1)"
  [[ "$output" == *"$version"* ]] || die "installed binary version mismatch"
  "$prefix/bin/brew" uninstall --force aiconnai/engram/engram >/dev/null
}

case "${1:-}" in
  --self-test) self_test; exit 0 ;;
  --self-test-prefix-isolation) self_test_prefix; exit 0 ;;
  --prefix-probe) prefix_probe; exit 0 ;;
  --verify-isolated) verify_isolated "${2:-}"; exit 0 ;;
esac
version=''
manifest=''
disposable=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --version) version=${2:-}; shift 2 ;;
    --manifest) manifest=${2:-}; shift 2 ;;
    --disposable-prefix) disposable=true; shift ;;
    *) die "unknown argument: $1" ;;
  esac
done
validate_version "$version"
[[ "$disposable" == true ]] || die "--disposable-prefix is mandatory"
if [[ -n "$manifest" ]]; then
  [[ -f "$manifest" ]] || die "manifest not found"
  manifest_version="$(python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print(d.get("channels",{}).get("homebrew",{}).get("version", d.get("version","")))' "$manifest")"
  manifest_version=${manifest_version#v}
  [[ "$manifest_version" == "$version" ]] || die "manifest Homebrew version mismatch"
fi
run_isolated_command "$0" --verify-isolated "$version"
echo "verify-homebrew-release: PASS ($version)"
