#!/usr/bin/env bash
set -euo pipefail

die() { echo "verify-crates-release: $*" >&2; exit 1; }
sha256_file() { openssl dgst -sha256 -r "$1" | awk '{print $1}'; }

validate_metadata() {
  METADATA="$1" CRATE="$2" VERSION="$3" python3 - <<'PY'
import json, os
data=json.loads(os.environ["METADATA"])
matches=[item for item in data.get("versions",[]) if item.get("num")==os.environ["VERSION"]]
if len(matches)!=1:
    raise SystemExit("registry version is missing or ambiguous")
item=matches[0]
if item.get("yanked"):
    raise SystemExit("registry version is yanked")
checksum=item.get("checksum","")
if len(checksum)!=64:
    raise SystemExit("registry checksum is invalid")
print(checksum)
PY
}

self_test() {
  local json checksum
  json='{"versions":[{"num":"0.22.0","yanked":false,"checksum":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}'
  checksum="$(validate_metadata "$json" engram-core 0.22.0)"
  [[ "$checksum" == "$(printf 'a%.0s' {1..64})" ]] || die "self-test checksum mismatch"
  if validate_metadata "${json/false/true}" engram-core 0.22.0 >/dev/null 2>&1; then
    die "self-test accepted a yanked crate"
  fi
  echo "verify-crates-release self-test: PASS"
}

[[ "${1:-}" == --self-test ]] && { self_test; exit 0; }
crate=''
version=''
sha=''
while [[ $# -gt 0 ]]; do
  case "$1" in
    --crate) crate=${2:-}; shift 2 ;;
    --version) version=${2:-}; shift 2 ;;
    --sha) sha=${2:-}; shift 2 ;;
    *) die "unknown argument: $1" ;;
  esac
done
[[ "$crate" =~ ^[a-z0-9_-]+$ ]] || die "invalid crate name"
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "invalid version"
[[ "$sha" =~ ^[0-9a-f]{40}$ ]] || die "invalid SHA"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
metadata="$(curl --fail --silent --show-error "https://crates.io/api/v1/crates/${crate}")"
expected="$(validate_metadata "$metadata" "$crate" "$version")"
curl --fail --silent --show-error --location \
  "https://crates.io/api/v1/crates/${crate}/${version}/download" -o "$tmp/package.crate"
[[ "$(sha256_file "$tmp/package.crate")" == "$expected" ]] || die "crate checksum mismatch"
tar -xzf "$tmp/package.crate" -C "$tmp"
vcs_file="$(find "$tmp" -name .cargo_vcs_info.json -type f -print -quit)"
[[ -n "$vcs_file" ]] || die "crate lacks .cargo_vcs_info.json"
actual_sha="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["git"]["sha1"])' "$vcs_file")"
[[ "$actual_sha" == "$sha" ]] || die "crate source SHA mismatch"
mkdir "$tmp/consumer"
cat > "$tmp/consumer/Cargo.toml" <<EOF
[package]
name = "engram-release-consumer"
version = "0.0.0"
edition = "2021"

[dependencies]
${crate} = "=${version}"
EOF
mkdir "$tmp/consumer/src"; printf 'fn main() {}\n' > "$tmp/consumer/src/main.rs"
cargo generate-lockfile --manifest-path "$tmp/consumer/Cargo.toml"
cargo check --locked --manifest-path "$tmp/consumer/Cargo.toml"
echo "verify-crates-release: PASS ($crate $version)"
