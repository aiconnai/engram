#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
die() { echo "verify-github-release: $*" >&2; exit 1; }

validate_release_json() {
  RELEASE_JSON="$1" TAG="$2" python3 - <<'PY'
import json, os
data = json.loads(os.environ["RELEASE_JSON"])
if data.get("tagName") != os.environ["TAG"]:
    raise SystemExit("release tag mismatch")
if data.get("isDraft") or data.get("isPrerelease"):
    raise SystemExit("release is not a final published release")
names = {asset["name"] for asset in data.get("assets", [])}
targets = ("aarch64-apple-darwin", "x86_64-apple-darwin", "aarch64-unknown-linux-gnu", "x86_64-unknown-linux-gnu")
required = set()
for target in targets:
    base = f"engram-{os.environ['TAG']}-{target}.tar.gz"
    required.update({base, f"{base}.sha256", f"{base}.cyclonedx.json", f"{base}.spdx.json", f"{base}.provenance.json", f"{base}.attestation.json", f"{base}.attestation.sig", f"{base}.github-attestation.sigstore.json"})
missing = required - names
if missing:
    raise SystemExit("release assets missing: " + ", ".join(sorted(missing)))
PY
}

self_test() {
  local tag=v0.22.0 json
  json="$(TAG="$tag" python3 - <<'PY'
import json, os
assets=[]
for target in ("aarch64-apple-darwin", "x86_64-apple-darwin", "aarch64-unknown-linux-gnu", "x86_64-unknown-linux-gnu"):
    base=f"engram-{os.environ['TAG']}-{target}.tar.gz"
    for suffix in ("", ".sha256", ".cyclonedx.json", ".spdx.json", ".provenance.json", ".attestation.json", ".attestation.sig", ".github-attestation.sigstore.json"):
        assets.append({"name": base+suffix})
print(json.dumps({"tagName":os.environ["TAG"],"isDraft":False,"isPrerelease":False,"assets":assets}))
PY
)"
  validate_release_json "$json" "$tag"
  if validate_release_json "${json/v0.22.0/v0.21.2}" "$tag" >/dev/null 2>&1; then
    die "self-test accepted a different release tag"
  fi
  echo "verify-github-release self-test: PASS"
}

[[ "${1:-}" == --self-test ]] && { self_test; exit 0; }
tag=''
sha=''
artifact_dir=''
repo=${GITHUB_REPOSITORY:-aiconnai/engram}
while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag) tag=${2:-}; shift 2 ;;
    --sha) sha=${2:-}; shift 2 ;;
    --artifact-dir) artifact_dir=${2:-}; shift 2 ;;
    --repo) repo=${2:-}; shift 2 ;;
    *) die "unknown argument: $1" ;;
  esac
done
[[ "$tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "invalid tag"
[[ "$sha" =~ ^[0-9a-f]{40}$ ]] || die "invalid SHA"
command -v gh >/dev/null || die "gh is required"
commit_sha="$(gh api "repos/${repo}/commits/${tag}" --jq .sha)"
[[ "$commit_sha" == "$sha" ]] || die "release tag commit does not match approved SHA"
release_json="$(gh release view "$tag" --repo "$repo" --json tagName,isDraft,isPrerelease,assets)"
validate_release_json "$release_json" "$tag"
cleanup=
if [[ -z "$artifact_dir" ]]; then
  cleanup="$(mktemp -d)"; trap 'rm -rf "${cleanup:-}"' EXIT
  gh release download "$tag" --repo "$repo" --dir "$cleanup"
  artifact_dir=$cleanup
fi
"$ROOT/scripts/verify-release-artifacts.sh" --artifact-dir "$artifact_dir" --expected-sha "$sha"
echo "verify-github-release: PASS ($tag)"
