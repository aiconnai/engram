#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

# Run with the required PR feature set (as scripts/ci.sh does) unless the caller
# supplies one, so feature-gated journeys (e.g. the dream job lifecycle) execute.
if [[ -z "${CI_REQUIRED_FEATURES:-}" ]]; then
  source "$repo_root/scripts/ci-required-features.env"
fi
: "${CI_REQUIRED_FEATURES:?CI_REQUIRED_FEATURES must be set or defined in scripts/ci-required-features.env}"

# A green run that silently executed fewer journeys is not a pass: dropping a
# feature (e.g. dream-phase) removes tests instead of failing them. Fail when the
# executed count falls below the floor or a feature-gated journey did not run.
min_tests="${CANONICAL_JOURNEY_MIN_TESTS:-11}"
required_tests=(
  "canonical_real_binary_journey_over_stdio_and_authenticated_http"
  "contract::rejected_requests_never_mutate_shared_state_across_stdio_and_http"
  "contract::unknown_tool_behavior_follows_the_environment_permission_mode"
  "contract::dream_job_lifecycle_persists_across_processes_and_transports"
)

log="$(mktemp "${TMPDIR:-/tmp}/canonical-journey.XXXXXX")"
trap 'rm -f "$log"' EXIT

# `rtk proxy` keeps the per-test lines the count check needs (plain `rtk cargo test`
# condenses them).
rtk proxy cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" \
  --test canonical_journey -- --nocapture 2>&1 | tee "$log"

passed="$(sed -n 's/^test result: ok\. \([0-9][0-9]*\) passed.*/\1/p' "$log" | tail -1)"
if [[ -z "$passed" || "$passed" -lt "$min_tests" ]]; then
  echo "canonical journey: ran ${passed:-0} tests, expected at least $min_tests" >&2
  exit 1
fi
for name in "${required_tests[@]}"; do
  if ! grep -Eq "^test ${name} \.\.\. ok$" "$log"; then
    echo "canonical journey: required test did not run and pass: $name" >&2
    echo "  (features: $CI_REQUIRED_FEATURES)" >&2
    exit 1
  fi
done
echo "canonical journey: $passed tests passed (floor $min_tests), all required journeys ran"
