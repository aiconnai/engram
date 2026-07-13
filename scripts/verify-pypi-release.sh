#!/usr/bin/env bash
set -euo pipefail

die() { echo "verify-pypi-release: $*" >&2; exit 1; }

verify_json() {
  python3 -c 'import json,pathlib,sys
data=json.loads(pathlib.Path(sys.argv[1]).read_text()); package,version=sys.argv[2:]
assert data.get("info",{}).get("name","").lower().replace("_","-")==package.lower().replace("_","-"), "PyPI package mismatch"
assert data.get("info",{}).get("version")==version, "PyPI version mismatch"
files=data.get("releases",{}).get(version,[]); types={item.get("packagetype") for item in files}
assert {"bdist_wheel","sdist"}.issubset(types), "PyPI release lacks wheel or sdist"
assert all(item.get("digests",{}).get("sha256") for item in files), "PyPI release file lacks SHA-256"' "$1" "$2" "$3"
}

if [[ "${1:-}" == "--self-test" ]]; then
  tmp="$(mktemp)"; trap 'rm -f "$tmp"' EXIT
  cat >"$tmp" <<'JSON'
{"info":{"name":"engram-client","version":"0.5.0"},"releases":{"0.5.0":[{"packagetype":"bdist_wheel","digests":{"sha256":"aa"}},{"packagetype":"sdist","digests":{"sha256":"bb"}}]}}
JSON
  verify_json "$tmp" engram-client 0.5.0
  set +e
  verify_json "$tmp" engram-client 9.9.9 >/dev/null 2>&1
  status=$?
  set -e
  [[ $status -ne 0 ]] || die "mismatched version was accepted"
  echo "verify-pypi-release self-test: PASS"
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
  [[ "$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["channels"]["pypi"]["version"])' "$manifest")" == "$version" ]] || die "manifest version mismatch"
fi
tmp="$(mktemp)"; trap 'rm -f "$tmp"' EXIT
curl --fail --silent --show-error --proto '=https' "https://pypi.org/pypi/${package}/${version}/json" >"$tmp"
verify_json "$tmp" "$package" "$version"
echo "verify-pypi-release: PASS"
