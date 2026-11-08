#!/usr/bin/env bash
set -euo pipefail

# Local envelope integrity only. Publisher identity is verified independently by
# verify-release-artifacts.sh using GitHub Actions provenance, never this key.
umask 077
die() { echo "sign-candidate-manifest: $*" >&2; exit 1; }
sha256_file() { openssl dgst -sha256 -r "$1" | awk '{print $1}'; }

sign_manifest() (
  local manifest=$1 signature=$2 key=${3:-} tmp key_origin
  [[ -f "$manifest" ]] || die "manifest not found: $manifest"
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  if [[ -z "$key" ]]; then
    key="$tmp/private.pem"
    openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:3072 -out "$key" >/dev/null 2>&1
    key_origin=ephemeral
  else
    [[ -f "$key" ]] || die "signing key not found: $key"
    key_origin=provided
  fi
  openssl pkey -in "$key" -pubout -out "$tmp/public.pem" >/dev/null 2>&1
  openssl rsa -pubin -in "$tmp/public.pem" -noout >/dev/null 2>&1 \
    || die "signing key must be RSA"
  openssl dgst -sha256 -sign "$key" -out "$tmp/signature.bin" "$manifest"
  MANIFEST_SHA="$(sha256_file "$manifest")" PUBLIC_KEY="$tmp/public.pem" \
  RAW_SIGNATURE="$tmp/signature.bin" KEY_ORIGIN="$key_origin" OUTPUT="$signature" python3 - <<'PY'
import base64
import json
import os
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
  echo "signed $manifest -> $signature"
)

verify_manifest() (
  local manifest=$1 signature=$2 tmp expected actual
  [[ -f "$manifest" ]] || die "manifest not found: $manifest"
  [[ -f "$signature" ]] || die "signature not found: $signature"
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  expected="$(SIGNATURE_FILE="$signature" PUBLIC_KEY="$tmp/public.pem" \
    RAW_SIGNATURE="$tmp/signature.bin" python3 - <<'PY'
import base64
import json
import os
import re
from pathlib import Path

data = json.loads(Path(os.environ["SIGNATURE_FILE"]).read_text())
required = {"schema_version", "algorithm", "manifest_sha256", "public_key_pem", "signature_base64"}
if not isinstance(data, dict) or not required.issubset(data):
    raise SystemExit("signature envelope is missing required fields")
if any(not isinstance(data[field], str) for field in required):
    raise SystemExit("signature envelope fields must be strings")
if data["schema_version"] != "engram-candidate-signature-v1":
    raise SystemExit("unsupported signature schema")
if data["algorithm"] != "rsa-pkcs1v15-sha256":
    raise SystemExit("unsupported signature algorithm")
if re.fullmatch(r"[0-9a-f]{64}", data["manifest_sha256"]) is None:
    raise SystemExit("invalid manifest digest")
Path(os.environ["PUBLIC_KEY"]).write_text(data["public_key_pem"])
Path(os.environ["RAW_SIGNATURE"]).write_bytes(base64.b64decode(data["signature_base64"], validate=True))
print(data["manifest_sha256"])
PY
  )"
  actual="$(sha256_file "$manifest")"
  [[ "$actual" == "$expected" ]] || die "manifest digest mismatch"
  openssl rsa -pubin -in "$tmp/public.pem" -noout >/dev/null 2>&1 \
    || die "signature key must be RSA"
  openssl dgst -sha256 -verify "$tmp/public.pem" -signature "$tmp/signature.bin" "$manifest" >/dev/null \
    || die "cryptographic signature verification failed"
  echo "verified $manifest"
)

manifest=''
signature=''
key=''
verify=''
while [[ $# -gt 0 ]]; do
  case "$1" in
    --manifest) manifest=${2:-}; shift 2 ;;
    --verify) verify=${2:-}; shift 2 ;;
    --signature) signature=${2:-}; shift 2 ;;
    --key) key=${2:-}; shift 2 ;;
    -h|--help)
      echo "Usage: sign-candidate-manifest.sh (--manifest FILE [--key RSA_PEM] | --verify FILE) --signature FILE"
      echo "Self-contained integrity envelope only; publisher authentication requires GitHub provenance."
      exit 0 ;;
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
