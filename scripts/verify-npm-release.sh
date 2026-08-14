#!/usr/bin/env bash
set -euo pipefail

die() { echo "verify-npm-release: $*" >&2; exit 1; }
verify_json() {
  python3 -c 'import json,pathlib,sys
data=json.loads(pathlib.Path(sys.argv[1]).read_text()); package,version=sys.argv[2:]
assert data.get("name")==package and data.get("version")==version, "npm package/version mismatch"
dist=data.get("dist",{}); assert dist.get("integrity") and dist.get("shasum") and dist.get("tarball","").startswith("https://"), "npm release lacks integrity metadata"' "$1" "$2" "$3"
}

if [[ "${1:-}" == "--self-test" ]]; then
  tmp="$(mktemp)"; trap 'rm -f "$tmp"' EXIT
  cat >"$tmp" <<'JSON'
{"name":"engram-client","version":"0.5.0","dist":{"integrity":"sha512-example","shasum":"abc","tarball":"https://registry.npmjs.org/engram-client/-/engram-client-0.5.0.tgz"}}
JSON
  verify_json "$tmp" engram-client 0.5.0
  set +e
  verify_json "$tmp" engram-client 9.9.9 >/dev/null 2>&1
  status=$?
  set -e
  [[ $status -ne 0 ]] || die "mismatched version was accepted"
  echo "verify-npm-release self-test: PASS"
  exit 0
fi

package=""; version=""; manifest=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --package) package=${2:-}; shift 2 ;;
    --version) version=${2:-}; shift 2 ;;
    --manifest) manifest=${2:-}; shift 2 ;;
    *) die "unknown argument: $1" ;;
  esac
done
[[ -n "$package" && -n "$version" ]] || die "--package and --version are required"
if [[ -n "$manifest" ]]; then
  [[ "$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["channels"]["npm"]["version"])' "$manifest")" == "$version" ]] || die "manifest version mismatch"
fi
tmp="$(mktemp)"; trap 'rm -f "$tmp"' EXIT
npm view "${package}@${version}" --json >"$tmp"
verify_json "$tmp" "$package" "$version"
echo "verify-npm-release: PASS"
