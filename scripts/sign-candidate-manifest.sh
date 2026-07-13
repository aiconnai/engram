#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  sign-candidate-manifest.sh --manifest FILE --signature FILE [--key PEM]
  sign-candidate-manifest.sh --verify FILE --signature FILE
  sign-candidate-manifest.sh --self-test

The signature is a self-contained JSON envelope. When --key is omitted a new
ephemeral RSA key is used; once the envelope is committed, Git history binds
that exact public key and signature to the reviewed candidate.
EOF
}

die() { echo "sign-candidate-manifest: $*" >&2; exit 1; }
sha256_file() { openssl dgst -sha256 -r "$1" | awk '{print $1}'; }

sign_manifest() {
  local manifest=$1 signature=$2 key=${3:-}
  [[ -f "$manifest" ]] || die "manifest not found: $manifest"
  command -v openssl >/dev/null || die "openssl is required"
  command -v python3 >/dev/null || die "python3 is required"

  local tmp key_origin
  tmp="$(mktemp -d)"
  trap 'rm -rf "${tmp:-}"' RETURN
  if [[ -z "$key" ]]; then
    key="$tmp/private.pem"
    openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:3072 -out "$key" >/dev/null 2>&1
    key_origin=ephemeral
  else
    [[ -f "$key" ]] || die "signing key not found: $key"
    key_origin=provided
  fi
  openssl pkey -in "$key" -pubout -out "$tmp/public.pem" >/dev/null 2>&1
  openssl dgst -sha256 -sign "$key" -out "$tmp/signature.bin" "$manifest"

  MANIFEST_SHA="$(sha256_file "$manifest")" \
  PUBLIC_KEY="$tmp/public.pem" RAW_SIGNATURE="$tmp/signature.bin" \
  KEY_ORIGIN="$key_origin" OUTPUT="$signature" python3 - <<'PY'
import base64, json, os
from pathlib import Path

envelope = {
    "schema_version": "engram-candidate-signature-v1",
    "algorithm": "rsa-pkcs1v15-sha256",
    "manifest_sha256": os.environ["MANIFEST_SHA"],
    "key_origin": os.environ["KEY_ORIGIN"],
    "public_key_pem": Path(os.environ["PUBLIC_KEY"]).read_text(),
    "signature_base64": base64.b64encode(Path(os.environ["RAW_SIGNATURE"]).read_bytes()).decode(),
}
Path(os.environ["OUTPUT"]).write_text(json.dumps(envelope, indent=2, sort_keys=True) + "\n")
PY
  trap - RETURN
  rm -rf "$tmp"
  echo "signed $manifest -> $signature"
}

verify_manifest() {
  local manifest=$1 signature=$2
  [[ -f "$manifest" ]] || die "manifest not found: $manifest"
  [[ -f "$signature" ]] || die "signature not found: $signature"
  local tmp expected actual
  tmp="$(mktemp -d)"
  trap 'rm -rf "${tmp:-}"' RETURN
  SIGNATURE_FILE="$signature" PUBLIC_KEY="$tmp/public.pem" RAW_SIGNATURE="$tmp/signature.bin" python3 - <<'PY'
import base64, json, os
from pathlib import Path

data = json.loads(Path(os.environ["SIGNATURE_FILE"]).read_text())
required = {"schema_version", "algorithm", "manifest_sha256", "public_key_pem", "signature_base64"}
if set(data) < required:
    raise SystemExit("signature envelope is missing required fields")
if data["schema_version"] != "engram-candidate-signature-v1":
    raise SystemExit("unsupported signature schema")
if data["algorithm"] != "rsa-pkcs1v15-sha256":
    raise SystemExit("unsupported signature algorithm")
Path(os.environ["PUBLIC_KEY"]).write_text(data["public_key_pem"])
Path(os.environ["RAW_SIGNATURE"]).write_bytes(base64.b64decode(data["signature_base64"], validate=True))
print(data["manifest_sha256"])
PY
  expected="$(SIGNATURE_FILE="$signature" python3 -c 'import json,os; print(json.load(open(os.environ["SIGNATURE_FILE"]))["manifest_sha256"])')"
  actual="$(sha256_file "$manifest")"
  [[ "$actual" == "$expected" ]] || die "manifest digest mismatch"
  openssl dgst -sha256 -verify "$tmp/public.pem" -signature "$tmp/signature.bin" "$manifest" >/dev/null \
    || die "cryptographic signature verification failed"
  trap - RETURN
  rm -rf "$tmp"
  echo "verified $manifest"
}

self_test() {
  local tmp
  tmp="$(mktemp -d)"
  self_test_tmp=$tmp
  trap 'rm -rf "${self_test_tmp:-}"' EXIT
  printf '{"sha":"%040d","version":"0.22.0"}\n' 0 > "$tmp/manifest.json"
  sign_manifest "$tmp/manifest.json" "$tmp/manifest.sig"
  verify_manifest "$tmp/manifest.json" "$tmp/manifest.sig"
  printf 'tamper\n' >> "$tmp/manifest.json"
  if (verify_manifest "$tmp/manifest.json" "$tmp/manifest.sig") >/dev/null 2>&1; then
    die "self-test accepted a tampered manifest"
  fi
  echo "sign-candidate-manifest self-test: PASS"
}

manifest=''
signature=''
key=''
verify=''
case "${1:-}" in
  --self-test) self_test; exit 0 ;;
esac
while [[ $# -gt 0 ]]; do
  case "$1" in
    --manifest) manifest=${2:-}; shift 2 ;;
    --verify) verify=${2:-}; shift 2 ;;
    --signature) signature=${2:-}; shift 2 ;;
    --key) key=${2:-}; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) die "unknown argument: $1" ;;
  esac
done
[[ -n "$signature" ]] || die "--signature is required"
if [[ -n "$verify" ]]; then
  [[ -z "$manifest" && -z "$key" ]] || die "--verify cannot be combined with --manifest/--key"
  verify_manifest "$verify" "$signature"
else
  [[ -n "$manifest" ]] || die "--manifest is required"
  sign_manifest "$manifest" "$signature" "$key"
fi
