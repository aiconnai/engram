#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
die() { echo "qa-product-workflows: $*" >&2; exit 1; }

validate_workflow() {
  python3 -c '
import pathlib,re,sys
text=pathlib.Path(sys.argv[1]).read_text()
for job,environment in (("publish-python","release-pypi"),("publish-npm","release-npm")):
 start=text.find("  "+job+":")
 if start<0: raise SystemExit("missing "+job)
 match=re.search(r"\n  [a-z][a-z0-9-]+:\n",text[start+3:]); end=-1 if match is None else start+3+match.start(); block=text[start:] if end<0 else text[start:end]
 if "environment: "+environment not in block: raise SystemExit(job+" lacks protected environment")
 if "needs.preflight.outputs.dry_run == '\''false'\''" not in block or "needs.preflight.outputs.publish == '\''true'\''" not in block: raise SystemExit(job+" lacks fail-closed condition")
 if "git verify-tag" not in block: raise SystemExit(job+" lacks signed-tag revalidation")
' "$1"
}

if [[ "${1:-}" == "--self-test" ]]; then
  workflow="$repo_root/.github/workflows/sdk-release.yml"
  [[ -f "$workflow" ]] || die "SDK release workflow is missing"
  validate_workflow "$workflow"
  grep -Fq 'supplied SHA does not match github.sha' "$workflow" || die "SHA guard is missing"
  grep -Fq 'tag, github.sha, and supplied SHA do not agree' "$workflow" || die "tag/SHA guard is missing"
  grep -Fq 'Dry-run complete: no registry request was attempted.' "$workflow" || die "dry-run terminal assertion is missing"
  tmp="$(mktemp)"; trap 'rm -f "$tmp"' EXIT
  grep -v 'environment: release-pypi' "$workflow" >"$tmp"
  set +e
  validate_workflow "$tmp" >/dev/null 2>&1
  status=$?
  set -e
  [[ $status -ne 0 ]] || die "missing-approval fixture was accepted"
  echo "qa-product-workflows self-test: PASS"
  exit 0
fi

if [[ "${1:-}" == "--self-test-negative" ]]; then
  shift
  artifact_dir=""
  while [[ $# -gt 0 ]]; do
    case "$1" in --artifact-dir) artifact_dir=${2:-}; shift 2 ;; *) die "unknown argument: $1" ;; esac
  done
  [[ -d "$artifact_dir" ]] || die "--artifact-dir is required"
  tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
  while IFS= read -r metadata; do
    cp "$metadata" "$tmp/$(basename "$metadata")"
    while IFS= read -r package; do ln -s "$package" "$tmp/$(basename "$package")"; done < <(find "$(dirname "$metadata")" -maxdepth 1 -type f \( -name '*.whl' -o -name '*.tar.gz' -o -name '*.tgz' \))
  done < <(find "$artifact_dir" -type f -name '*-sdk-metadata.json')
  python3 -c 'import json,pathlib,sys; p=next(pathlib.Path(sys.argv[1]).glob("python-sdk-metadata.json")); d=json.loads(p.read_text()); d["version"]="9.9.9"; p.write_text(json.dumps(d))' "$tmp"
  set +e
  "$repo_root/scripts/verify-sdk-artifacts.sh" --artifact-dir "$tmp" --expected-sha "$(git rev-parse HEAD)" >/dev/null 2>&1
  status=$?
  set -e
  [[ $status -ne 0 ]] || die "package-drift fixture was accepted"

  python3 scripts/check-quality-budgets.py \
    --budgets docs/quality/budgets.json \
    --retrieval tests/fixtures/retrieval_quality/baseline.json \
    --criterion benches/results/benchmark_results.txt \
    --self-test-degraded >/dev/null
  cargo test --test retrieval_quality evaluator_reports_cross_workspace_leak -- --nocapture >/dev/null
  "$repo_root/scripts/verify-sdk-artifacts.sh" \
    --artifact-dir "$artifact_dir" --expected-sha "$(git rev-parse HEAD)" --live
  echo "qa-product-workflows negative self-test: PASS"
  exit 0
fi

artifact_args=()
manifest=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --artifact-dir) artifact_args+=(--artifact-dir "${2:-}"); shift 2 ;;
    --run-id) artifact_args+=(--run-id "${2:-}"); shift 2 ;;
    --expected-sha) artifact_args+=(--expected-sha "${2:-}"); shift 2 ;;
    --manifest) manifest=${2:-}; shift 2 ;;
    *) die "unknown argument: $1" ;;
  esac
done
[[ ${#artifact_args[@]} -gt 0 ]] || die "--artifact-dir or --run-id is required"
if [[ -n "$manifest" ]]; then
  [[ -f "$manifest" ]] || die "manifest does not exist: $manifest"
  manifest_sha="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["sha"])' "$manifest")"
  artifact_args+=(--manifest "$manifest" --expected-sha "$manifest_sha")
fi
"$repo_root/scripts/verify-sdk-artifacts.sh" "${artifact_args[@]}" --live
bash "$repo_root/scripts/test-examples.sh"
cargo test --test retrieval_quality -- --nocapture
python3 "$repo_root/scripts/check-quality-budgets.py" \
  --budgets docs/quality/budgets.json \
  --retrieval tests/fixtures/retrieval_quality/baseline.json \
  --criterion benches/results/benchmark_results.txt
cargo test --test dream_eval_tests --features dream-phase -- --nocapture
echo "qa-product-workflows: PASS"
