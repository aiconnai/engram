#!/usr/bin/env bash
set -euo pipefail

STATE_FILE="${LOOP_STATE_FILE:-docs/loops/agentshield-scan/STATE.md}"
SCAN_PATH="${LOOP_SCAN_PATH:-.}"
MAX_ITERATIONS="${LOOP_MAX_ITERATIONS:-1}"
BASELINE_FILE="${AGENTSHIELD_BASELINE:-.agentshield-baseline.json}"
WRITE_BASELINE="${LOOP_WRITE_BASELINE:-0}"
STATE_WRITE="${LOOP_STATE_WRITE:-1}"
FAIL_ON="${AGENTSHIELD_FAIL_ON:-high}"

die() {
  echo "error: $*" >&2
  exit 2
}

redact() {
  sed -E \
    -e 's/(Bearer|token|api[_-]?key|secret|password)[=: ][^[:space:]]+/\1=[REDACTED]/Ig' \
    -e 's/[A-Za-z0-9_\/+=.-]{32,}/[REDACTED]/g'
}

markdown_escape() {
  tr '\n' ' ' | sed -E 's/[|`]/ /g; s/[[:space:]]+/ /g; s/^ //; s/ $//'
}

append_state_row() {
  local status="$1"
  local summary="$2"
  local timestamp tmp

  [ "${STATE_WRITE}" = "1" ] || return 0
  [ -f "${STATE_FILE}" ] || die "state file missing: ${STATE_FILE}"

  timestamp="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  tmp="$(mktemp)"

  awk -v row="| ${timestamp} | \`agentshield scan ${SCAN_PATH}\` | ${status} | ${summary} |" '
    BEGIN { inserted = 0 }
    {
      print
      if (!inserted && $0 ~ /^\|---\|---\|---\|---\|$/) {
        print row
        inserted = 1
      }
    }
    END {
      if (!inserted) {
        print ""
        print "## Evidence Log"
        print "| Time | Command / Action | Result (Pass/Fail) | Notes / Findings |"
        print "|---|---|---|---|"
        print row
      }
    }
  ' "${STATE_FILE}" > "${tmp}"
  mv "${tmp}" "${STATE_FILE}"
}

case "${MAX_ITERATIONS}" in
  ''|*[!0-9]*) die "LOOP_MAX_ITERATIONS must be a positive integer" ;;
esac
[ "${MAX_ITERATIONS}" -ge 1 ] || die "LOOP_MAX_ITERATIONS must be >= 1"
[ "${MAX_ITERATIONS}" -le 5 ] || die "LOOP_MAX_ITERATIONS must be <= 5"

command -v agentshield >/dev/null 2>&1 || die "agentshield CLI is not installed or not in PATH"
[ -d "${SCAN_PATH}" ] || die "scan path is not a directory: ${SCAN_PATH}"
[ -f "${STATE_FILE}" ] || die "state file missing: ${STATE_FILE}"

echo "AgentShield loop"
echo "version=$(agentshield --version | redact)"
echo "scan_path=${SCAN_PATH}"
echo "max_iterations=${MAX_ITERATIONS}"
echo "fail_on=${FAIL_ON}"

if [ "${WRITE_BASELINE}" = "1" ]; then
  baseline_output_file="$(mktemp)"
  echo "Writing baseline to ${BASELINE_FILE}"
  agentshield scan "${SCAN_PATH}" \
    --ignore-tests \
    --write-baseline "${BASELINE_FILE}" \
    --explain >"${baseline_output_file}" 2>&1 || {
      summary="$(redact <"${baseline_output_file}" | head -n 20 | markdown_escape | cut -c1-240)"
      append_state_row "FAIL" "Baseline write failed: ${summary}"
      echo "Baseline write failed"
      redact <"${baseline_output_file}" | head -n 40
      rm -f "${baseline_output_file}"
      exit 1
    }
  rm -f "${baseline_output_file}"
  append_state_row "PASS" "Baseline written to ${BASELINE_FILE}; review before committing."
fi

exit_code=0
for iteration in $(seq 1 "${MAX_ITERATIONS}"); do
  echo "=== iteration ${iteration}/${MAX_ITERATIONS} ==="

  output_file="$(mktemp)"
  cmd=(agentshield scan "${SCAN_PATH}" --ignore-tests --fail-on "${FAIL_ON}" --explain)
  if [ -f "${BASELINE_FILE}" ]; then
    cmd+=(--baseline "${BASELINE_FILE}")
  fi

  set +e
  "${cmd[@]}" >"${output_file}" 2>&1
  exit_code=$?
  set -e

  if [ "${exit_code}" -eq 0 ]; then
    append_state_row "PASS" "No new ${FAIL_ON}+ AgentShield findings."
    echo "Gate status: PASS"
    rm -f "${output_file}"
    exit 0
  fi

  summary="$(redact <"${output_file}" | head -n 20 | markdown_escape | cut -c1-240)"
  append_state_row "FAIL" "AgentShield exited ${exit_code}: ${summary}"
  echo "Gate status: FAIL"
  echo "Redacted findings summary:"
  redact <"${output_file}" | head -n 40
  rm -f "${output_file}"

  if [ "${iteration}" -lt "${MAX_ITERATIONS}" ]; then
    echo "Loop is bounded; no automatic remediation is attempted."
  fi
done

exit "${exit_code}"
