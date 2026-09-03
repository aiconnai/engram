#!/usr/bin/env bash
# docs/harness/bin/test-fixtures.sh
#
# Regression test suite for Agent Harness Hardening Wave 1 schemas and fixtures.
# Verifies:
# - All valid fixtures pass validation with exit code 0
# - All adversarial/invalid fixtures fail closed with exit code 1 and structured errors
# - Standalone execution with Python standard library
# - JSON output envelope matches docs/harness/JSON_OUTPUTS.md
#
# Exits 0 on success, 1 on any test failure.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
VALIDATOR="${SCRIPT_DIR}/validate-evidence.py"
FIXTURES_DIR="${REPO_ROOT}/docs/harness/fixtures"
SCHEMAS_DIR="${REPO_ROOT}/docs/harness/schemas"

cd "${REPO_ROOT}"

PASS_COUNT=0
FAIL_COUNT=0

log_pass() {
  echo "  [PASS] $1"
  PASS_COUNT=$((PASS_COUNT + 1))
}

log_fail() {
  echo "  [FAIL] $1" >&2
  FAIL_COUNT=$((FAIL_COUNT + 1))
}

echo "=== Agent Harness Hardening Wave 1: Fixture Verification Suite ==="
echo "Repo root: ${REPO_ROOT}"
echo "Validator: ${VALIDATOR}"
echo ""

# 1. Verify schema files exist
echo "--- Step 1: Checking schema existence ---"
for schema in "task-v1.schema.json" "evidence-v1.schema.json" "review-v1.schema.json"; do
  if [ -f "${SCHEMAS_DIR}/${schema}" ]; then
    log_pass "Schema exists: ${schema}"
  else
    log_fail "Schema missing: ${schema}"
  fi
done
echo ""

# 2. Verify valid fixtures pass (exit code 0)
echo "--- Step 2: Testing valid fixtures (expect exit code 0) ---"
for fixture in "valid_task.json" "valid_evidence.json" "valid_review.json"; do
  fix_path="${FIXTURES_DIR}/${fixture}"
  if [ ! -f "${fix_path}" ]; then
    log_fail "Fixture file missing: ${fixture}"
    continue
  fi

  if python3 "${VALIDATOR}" "${fix_path}" >/dev/null 2>&1; then
    log_pass "${fixture} passed validation (exit 0)"
  else
    log_fail "${fixture} unexpectedly failed validation"
  fi
done
echo ""

# 3. Verify adversarial fixtures fail closed (exit code 1)
echo "--- Step 3: Testing adversarial fixtures (expect exit code 1) ---"
for fixture in "invalid_wrong_sha.json" "invalid_scope_violation.json" "invalid_missing_required.json"; do
  fix_path="${FIXTURES_DIR}/${fixture}"
  if [ ! -f "${fix_path}" ]; then
    log_fail "Fixture file missing: ${fixture}"
    continue
  fi

  output=""
  exit_code=0
  output=$(python3 "${VALIDATOR}" "${fix_path}" 2>&1) || exit_code=$?

  if [ "${exit_code}" -eq 1 ]; then
    log_pass "${fixture} failed closed with exit code 1"
  else
    log_fail "${fixture} returned exit code ${exit_code}, expected 1"
  fi
done
echo ""

# 4. Verify specific diagnostic error patterns
echo "--- Step 4: Verifying diagnostic error messages ---"

# 4a. invalid_wrong_sha.json must complain about invalid SHA
sha_out=$(python3 "${VALIDATOR}" "${FIXTURES_DIR}/invalid_wrong_sha.json" 2>&1 || true)
if echo "${sha_out}" | grep -q "commit_sha" && echo "${sha_out}" | grep -qE "(invalid_pattern|invalid_sha)"; then
  log_pass "invalid_wrong_sha.json correctly diagnosed invalid commit_sha"
else
  log_fail "invalid_wrong_sha.json missing expected SHA diagnostic"
fi

# 4b. invalid_scope_violation.json must diagnose scope/capability violation
scope_out=$(python3 "${VALIDATOR}" "${FIXTURES_DIR}/invalid_scope_violation.json" 2>&1 || true)
if echo "${scope_out}" | grep -q "arbitrary_host_exec" && echo "${scope_out}" | grep -qE "(scope_violation|invalid_enum_value)"; then
  log_pass "invalid_scope_violation.json correctly diagnosed capability/path scope violation"
else
  log_fail "invalid_scope_violation.json missing expected scope violation diagnostic"
fi

# 4c. invalid_missing_required.json must diagnose missing required fields
req_out=$(python3 "${VALIDATOR}" "${FIXTURES_DIR}/invalid_missing_required.json" 2>&1 || true)
if echo "${req_out}" | grep -q "missing_required"; then
  log_pass "invalid_missing_required.json correctly diagnosed missing required fields"
else
  log_fail "invalid_missing_required.json missing expected required fields diagnostic"
fi
echo ""

# 5. Verify JSON output envelope
echo "--- Step 5: Testing JSON output mode (--json) ---"
json_out=$(python3 "${VALIDATOR}" --json "${FIXTURES_DIR}/valid_task.json" 2>/dev/null)
if echo "${json_out}" | grep -q '"schema_version": "harness-json-v1"' && echo "${json_out}" | grep -q '"status": "pass"'; then
  log_pass "--json on valid fixture emitted conforming JSON envelope with status pass"
else
  log_fail "--json on valid fixture did not emit expected envelope"
fi

json_err=$(python3 "${VALIDATOR}" --json "${FIXTURES_DIR}/invalid_wrong_sha.json" 2>/dev/null || true)
if echo "${json_err}" | grep -q '"schema_version": "harness-json-v1"' && echo "${json_err}" | grep -q '"status": "fail"'; then
  log_pass "--json on invalid fixture emitted conforming JSON envelope with status fail"
else
  log_fail "--json on invalid fixture did not emit expected failure envelope"
fi
echo ""

# 6. Verify built-in --self-test flag
echo "--- Step 6: Testing --self-test flag ---"
if python3 "${VALIDATOR}" --self-test >/dev/null 2>&1; then
  log_pass "Validator --self-test succeeded"
else
  log_fail "Validator --self-test failed"
fi
echo ""

echo "=== Summary ==="
echo "Passed: ${PASS_COUNT}"
echo "Failed: ${FAIL_COUNT}"

if [ "${FAIL_COUNT}" -eq 0 ]; then
  echo "ALL TESTS PASSED"
  exit 0
else
  echo "TEST SUITE FAILED" >&2
  exit 1
fi
