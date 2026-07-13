#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGETS=(
  aarch64-apple-darwin
  x86_64-apple-darwin
  x86_64-unknown-linux-gnu
  aarch64-unknown-linux-gnu
)

die() { echo "verify-release-artifacts: $*" >&2; exit 1; }
sha256_file() { openssl dgst -sha256 -r "$1" | awk '{print $1}'; }

create_bundle() {
  local binary_dir=$1 output_dir=$2 target=$3 version=$4 sha=$5 ref=$6
  [[ "$sha" =~ ^[0-9a-f]{40}$ ]] || die "--sha must be a full lowercase commit SHA"
  [[ "$version" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "invalid version"
  [[ " ${TARGETS[*]} " == *" $target "* ]] || die "unsupported target: $target"
  [[ -f "$ROOT/Cargo.lock" ]] || die "Cargo.lock is required"
  local binaries=(engram-server engram-cli)
  [[ "$target" == *-linux-* ]] && binaries+=(engram-pdf-worker)
  local binary
  for binary in "${binaries[@]}"; do
    [[ -f "$binary_dir/$binary" ]] || die "missing release binary: $binary_dir/$binary"
  done
  mkdir -p "$output_dir"
  local archive="$output_dir/engram-${version}-${target}.tar.gz"
  local epoch
  epoch="$(git -C "$ROOT" show -s --format=%ct "$sha" 2>/dev/null || printf '0')"
  BINARY_DIR="$binary_dir" ARCHIVE="$archive" BINARIES="${binaries[*]}" EPOCH="$epoch" python3 - <<'PY'
import gzip, io, os, tarfile
from pathlib import Path

binary_dir = Path(os.environ["BINARY_DIR"])
archive = Path(os.environ["ARCHIVE"])
epoch = int(os.environ["EPOCH"])
with archive.open("wb") as raw:
    with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as zipped:
        with tarfile.open(fileobj=zipped, mode="w") as tar:
            for name in sorted(os.environ["BINARIES"].split()):
                data = (binary_dir / name).read_bytes()
                info = tarfile.TarInfo(name)
                info.size = len(data)
                info.mode = 0o755
                info.uid = info.gid = 0
                info.uname = info.gname = "root"
                info.mtime = epoch
                tar.addfile(info, io.BytesIO(data))
PY
  local digest
  digest="$(sha256_file "$archive")"
  printf '%s  %s\n' "$digest" "$(basename "$archive")" > "$archive.sha256"

  ARCHIVE="$archive" DIGEST="$digest" TARGET="$target" VERSION="$version" \
  SHA="$sha" REF="$ref" CARGO_LOCK="$ROOT/Cargo.lock" python3 - <<'PY'
import hashlib, json, os, re
from pathlib import Path

archive = Path(os.environ["ARCHIVE"])
digest = os.environ["DIGEST"]
target = os.environ["TARGET"]
sha = os.environ["SHA"]
ref = os.environ["REF"]
version = os.environ["VERSION"]
packages = []
current = None
for raw in Path(os.environ["CARGO_LOCK"]).read_text().splitlines():
    line = raw.strip()
    if line == "[[package]]":
        if current is not None:
            packages.append(current)
        current = {}
        continue
    if current is None:
        continue
    match = re.fullmatch(r'(name|version|source|checksum)\s*=\s*("(?:[^"\\]|\\.)*")', line)
    if match:
        current[match.group(1)] = json.loads(match.group(2))
if current is not None:
    packages.append(current)
if not packages or any("name" not in package or "version" not in package for package in packages):
    raise SystemExit("Cargo.lock package parsing failed")
packages.sort(key=lambda p: (p["name"], p["version"], p.get("source", "")))

components = []
spdx_packages = []
for index, package in enumerate(packages, 1):
    component = {"type": "library", "name": package["name"], "version": package["version"]}
    if package.get("source"):
        component["purl"] = f"pkg:cargo/{package['name']}@{package['version']}"
    if package.get("checksum"):
        component["hashes"] = [{"alg": "SHA-256", "content": package["checksum"]}]
    components.append(component)
    spdx_packages.append({
        "SPDXID": f"SPDXRef-Package-{index}",
        "name": package["name"],
        "versionInfo": package["version"],
        "downloadLocation": package.get("source", "NOASSERTION"),
        "filesAnalyzed": False,
        "licenseConcluded": "NOASSERTION",
        "licenseDeclared": "NOASSERTION",
        "copyrightText": "NOASSERTION",
    })

cyclonedx = {
    "bomFormat": "CycloneDX",
    "specVersion": "1.5",
    "serialNumber": f"urn:uuid:{digest[:8]}-{digest[8:12]}-{digest[12:16]}-{digest[16:20]}-{digest[20:32]}",
    "version": 1,
    "metadata": {
        "component": {"type": "application", "name": archive.name, "version": version.removeprefix("v")},
        "properties": [
            {"name": "engram:artifact-sha256", "value": digest},
            {"name": "engram:git-commit", "value": sha},
            {"name": "engram:target", "value": target},
        ],
    },
    "components": components,
}
spdx = {
    "spdxVersion": "SPDX-2.3",
    "dataLicense": "CC0-1.0",
    "SPDXID": "SPDXRef-DOCUMENT",
    "name": archive.name,
    "documentNamespace": f"https://github.com/aiconnai/engram/sbom/{sha}/{target}/{digest}",
    "creationInfo": {"created": "1970-01-01T00:00:00Z", "creators": ["Tool: engram-release-v1"]},
    "packages": spdx_packages,
    "annotations": [{
        "annotationType": "OTHER",
        "annotator": "Tool: engram-release-v1",
        "annotationDate": "1970-01-01T00:00:00Z",
        "comment": f"artifact-sha256={digest};git-commit={sha};target={target}",
    }],
}
provenance = {
    "_type": "https://in-toto.io/Statement/v1",
    "subject": [{"name": archive.name, "digest": {"sha256": digest}}],
    "predicateType": "https://slsa.dev/provenance/v1",
    "predicate": {
        "buildDefinition": {
            "buildType": "https://github.com/aiconnai/engram/.github/workflows/release.yml@v1",
            "externalParameters": {"target": target, "version": version, "ref": ref, "sha": sha},
            "resolvedDependencies": [{"uri": "git+https://github.com/aiconnai/engram", "digest": {"gitCommit": sha}}],
        },
        "runDetails": {
            "builder": {"id": f"https://github.com/aiconnai/engram/actions/runs/{os.getenv('GITHUB_RUN_ID', 'local')}"},
            "metadata": {"invocationId": os.getenv("GITHUB_RUN_ATTEMPT", "local")},
        },
    },
}
for suffix, value in (("cyclonedx.json", cyclonedx), ("spdx.json", spdx), ("provenance.json", provenance)):
    Path(f"{archive}.{suffix}").write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")

def file_hash(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

attestation = {
    "schema_version": "engram-release-attestation-v1",
    "artifact": {"name": archive.name, "sha256": digest},
    "commit": sha,
    "ref": ref,
    "target": target,
    "sbom": {
        "cyclonedx_sha256": file_hash(f"{archive}.cyclonedx.json"),
        "spdx_sha256": file_hash(f"{archive}.spdx.json"),
    },
    "provenance_sha256": file_hash(f"{archive}.provenance.json"),
}
Path(f"{archive}.attestation.json").write_text(json.dumps(attestation, indent=2, sort_keys=True) + "\n")
PY
  "$ROOT/scripts/sign-candidate-manifest.sh" \
    --manifest "$archive.attestation.json" \
    --signature "$archive.attestation.sig"
  echo "created release bundle: $archive"
}

verify_archive() {
  local archive=$1 expected_sha=${2:-} expected_target=${3:-} allow_untrusted=${4:-false}
  local checksum="$archive.sha256" cdx="$archive.cyclonedx.json" spdx="$archive.spdx.json"
  local provenance="$archive.provenance.json" attestation="$archive.attestation.json"
  local signature="$archive.attestation.sig"
  for path in "$checksum" "$cdx" "$spdx" "$provenance" "$attestation" "$signature"; do
    [[ -f "$path" ]] || die "missing evidence file: $path"
  done
  local expected_digest expected_name actual_digest
  read -r expected_digest expected_name < "$checksum"
  expected_name="${expected_name#\*}"
  [[ "$expected_name" == "$(basename "$archive")" ]] || die "checksum filename mismatch"
  actual_digest="$(sha256_file "$archive")"
  [[ "$actual_digest" == "$expected_digest" ]] || die "archive checksum mismatch: $archive"
  ARCHIVE="$archive" DIGEST="$actual_digest" EXPECTED_SHA="$expected_sha" \
  EXPECTED_TARGET="$expected_target" python3 - <<'PY'
import hashlib, json, os, tarfile
from pathlib import Path

archive = Path(os.environ["ARCHIVE"])
digest = os.environ["DIGEST"]
expected_sha = os.environ["EXPECTED_SHA"]
expected_target = os.environ["EXPECTED_TARGET"]
cdx = json.loads(Path(f"{archive}.cyclonedx.json").read_text())
spdx = json.loads(Path(f"{archive}.spdx.json").read_text())
provenance = json.loads(Path(f"{archive}.provenance.json").read_text())
attestation = json.loads(Path(f"{archive}.attestation.json").read_text())

if cdx.get("bomFormat") != "CycloneDX" or cdx.get("specVersion") != "1.5" or not cdx.get("components"):
    raise SystemExit("invalid or empty CycloneDX SBOM")
if spdx.get("spdxVersion") != "SPDX-2.3" or not spdx.get("packages"):
    raise SystemExit("invalid or empty SPDX SBOM")
subject = provenance.get("subject", [{}])[0]
if subject.get("name") != archive.name or subject.get("digest", {}).get("sha256") != digest:
    raise SystemExit("provenance subject does not bind the archive")
parameters = provenance.get("predicate", {}).get("buildDefinition", {}).get("externalParameters", {})
if expected_sha and parameters.get("sha") != expected_sha:
    raise SystemExit("provenance SHA mismatch")
if expected_target and parameters.get("target") != expected_target:
    raise SystemExit("provenance target mismatch")
if attestation.get("schema_version") != "engram-release-attestation-v1":
    raise SystemExit("invalid attestation schema")
if attestation.get("artifact") != {"name": archive.name, "sha256": digest}:
    raise SystemExit("attestation artifact mismatch")
if expected_sha and attestation.get("commit") != expected_sha:
    raise SystemExit("attestation commit mismatch")
if expected_target and attestation.get("target") != expected_target:
    raise SystemExit("attestation target mismatch")

def digest_file(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()
if attestation.get("sbom", {}).get("cyclonedx_sha256") != digest_file(f"{archive}.cyclonedx.json"):
    raise SystemExit("CycloneDX digest mismatch")
if attestation.get("sbom", {}).get("spdx_sha256") != digest_file(f"{archive}.spdx.json"):
    raise SystemExit("SPDX digest mismatch")
if attestation.get("provenance_sha256") != digest_file(f"{archive}.provenance.json"):
    raise SystemExit("provenance digest mismatch")
with tarfile.open(archive, "r:gz") as tar:
    names = sorted(tar.getnames())
    if any(name.startswith("/") or ".." in Path(name).parts for name in names):
        raise SystemExit("unsafe archive member")
target = expected_target or attestation.get("target", "")
wanted = ["engram-cli", "engram-server"] + (["engram-pdf-worker"] if "-linux-" in target else [])
if names != sorted(wanted):
    raise SystemExit(f"archive member mismatch: {names} != {sorted(wanted)}")
PY
  "$ROOT/scripts/sign-candidate-manifest.sh" --verify "$attestation" --signature "$signature" >/dev/null
  local github_bundle="$archive.github-attestation.sigstore.json"
  if [[ "$allow_untrusted" == true ]]; then
    [[ ! -e "$github_bundle" ]] || die "local-only verification must not ignore a supplied GitHub attestation"
  else
    [[ "$expected_sha" =~ ^[0-9a-f]{40}$ ]] \
      || die "trusted GitHub attestation verification requires --expected-sha"
    [[ -f "$github_bundle" ]] || die "missing identity-bound GitHub attestation: $github_bundle"
    command -v gh >/dev/null || die "gh is required to verify GitHub identity attestations"
    local signed_file
    for signed_file in "$archive" "$checksum" "$cdx" "$spdx" "$provenance" "$attestation" "$signature"; do
      gh attestation verify "$signed_file" \
        --bundle "$github_bundle" \
        --repo "${GITHUB_REPOSITORY:-aiconnai/engram}" \
        --signer-workflow aiconnai/engram/.github/workflows/release.yml \
        --source-digest "$expected_sha" \
        >/dev/null
    done
  fi
  echo "verified $(basename "$archive")"
}

verify_dir() {
  local dir=$1 expected_sha=${2:-} target=${3:-} allow_untrusted=${4:-false}
  [[ -d "$dir" ]] || die "artifact directory not found: $dir"
  local archives=()
  while IFS= read -r -d '' path; do archives+=("$path"); done < <(find "$dir" -type f -name 'engram-v*-*.tar.gz' -print0 | sort -z)
  [[ ${#archives[@]} -gt 0 ]] || die "no release archives found under $dir"
  if [[ -n "$target" ]]; then
    [[ ${#archives[@]} -eq 1 ]] || die "target verification requires exactly one archive"
    verify_archive "${archives[0]}" "$expected_sha" "$target" "$allow_untrusted"
  else
    [[ ${#archives[@]} -eq ${#TARGETS[@]} ]] || die "expected ${#TARGETS[@]} archives, found ${#archives[@]}"
    local seen=" " archive target_name
    for archive in "${archives[@]}"; do
      target_name="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["target"])' "$archive.attestation.json")"
      [[ " ${TARGETS[*]} " == *" $target_name "* ]] || die "unexpected target: $target_name"
      [[ "$seen" != *" $target_name "* ]] || die "duplicate target: $target_name"
      seen+="$target_name "
      verify_archive "$archive" "$expected_sha" "$target_name" "$allow_untrusted"
    done
  fi
}

self_test_tamper() {
  local tmp sha target archive
  tmp="$(mktemp -d)"
  self_test_tmp=$tmp
  trap 'rm -rf "${self_test_tmp:-}"' EXIT
  mkdir "$tmp/bin"
  for binary in engram-server engram-cli; do
    printf '#!/bin/sh\necho %s 0.22.0\n' "$binary" > "$tmp/bin/$binary"
    chmod +x "$tmp/bin/$binary"
  done
  sha="$(git -C "$ROOT" rev-parse HEAD)"
  target=aarch64-apple-darwin
  create_bundle "$tmp/bin" "$tmp/out" "$target" v0.22.0 "$sha" refs/heads/self-test
  create_bundle "$tmp/bin" "$tmp/out-repeat" "$target" v0.22.0 "$sha" refs/heads/self-test
  [[ "$(sha256_file "$tmp/out/engram-v0.22.0-${target}.tar.gz")" == \
     "$(sha256_file "$tmp/out-repeat/engram-v0.22.0-${target}.tar.gz")" ]] \
    || die "identical inputs did not produce the same archive"
  verify_dir "$tmp/out" "$sha" "$target" true >/dev/null
  archive="$(find "$tmp/out" -name '*.tar.gz' -type f)"
  printf x >> "$archive"
  if (verify_dir "$tmp/out" "$sha" "$target" true) >/dev/null 2>&1; then
    die "tamper self-test accepted a modified archive"
  fi
  echo "verify-release-artifacts tamper self-test: PASS"
}

mode=verify
artifact_dir=''
run_id=''
expected_sha=''
target=''
binary_dir=''
output_dir=''
version=''
sha=''
ref=''
allow_untrusted_local=false
while [[ $# -gt 0 ]]; do
  case "$1" in
    --create-bundle) mode=create; shift ;;
    --self-test-tamper) mode=tamper; shift ;;
    --artifact-dir) artifact_dir=${2:-}; shift 2 ;;
    --run-id) run_id=${2:-}; shift 2 ;;
    --expected-sha) expected_sha=${2:-}; shift 2 ;;
    --target) target=${2:-}; shift 2 ;;
    --binary-dir) binary_dir=${2:-}; shift 2 ;;
    --output-dir) output_dir=${2:-}; shift 2 ;;
    --version) version=${2:-}; shift 2 ;;
    --sha) sha=${2:-}; shift 2 ;;
    --ref) ref=${2:-}; shift 2 ;;
    --allow-untrusted-local) allow_untrusted_local=true; shift ;;
    -h|--help) sed -n '1,35p' "$0"; exit 0 ;;
    *) die "unknown argument: $1" ;;
  esac
done
case "$mode" in
  create)
    [[ -n "$binary_dir$output_dir$target$version$sha$ref" ]] || die "create mode requires all bundle arguments"
    create_bundle "$binary_dir" "$output_dir" "$target" "$version" "$sha" "$ref"
    ;;
  tamper) self_test_tamper ;;
  verify)
    cleanup=
    if [[ -n "$run_id" ]]; then
      cleanup="$(mktemp -d)"
      trap 'rm -rf "${cleanup:-}"' EXIT
      gh run download "$run_id" --dir "$cleanup"
      artifact_dir=$cleanup
      [[ -n "$expected_sha" ]] || expected_sha="$(gh run view "$run_id" --json headSha --jq .headSha)"
    fi
    [[ -n "$artifact_dir" ]] || die "--artifact-dir or --run-id is required"
    verify_dir "$artifact_dir" "$expected_sha" "$target" "$allow_untrusted_local"
    ;;
esac
