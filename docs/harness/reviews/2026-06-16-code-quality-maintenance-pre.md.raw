# Engram Harness — External Reviewer Prompt

**Task**: code-quality-maintenance
**Mode**: pre
**Date (UTC)**: 2026-06-16

## Instructions for the Reviewer

You are acting as an independent senior engineer reviewing a diff for the engram project.
You were NOT the implementer. Your job is to find real problems introduced by the change.

Read the following documents (they are the source of truth for this review):

- docs/harness/SPEC.md
- docs/harness/INVARIANTS.md (process invariants — canonical)
- docs/harness/WHAT_WE_DONT_DO.md (negative scope — no hidden expansion)
- docs/harness/GATES.md (especially the fake-success patterns section)
- docs/harness/CODE_REVIEW_POLICY.md (this policy)
- docs/harness/security/anthropic-reference-harness.md (security boundary)
- .claude/scan-extras.txt and .claude/fp-rules.txt (org-specific scan/triage tuning)
- docs/harness/README.md (workflow)
- Root INVARIANTS.md (data layer invariants for the memory system)

Then review the diff below.

Additional harness-specific requirements:
- Compare scope against docs/harness/WHAT_WE_DONT_DO.md. Flag hidden scope creep, gate weakening, or product changes bundled into harness work.
- Security boundary: flag autonomous Engram execution, implied sandboxing, credential mounts, network/egress expansion, or C/C++/ASAN pipeline import unless an ADR and explicit target contract are present.
- Tuning files: ensure .claude/scan-extras.txt and .claude/fp-rules.txt augment scan/triage behavior without weakening core INVARIANTS/GATES/POLICY or adding blanket suppressions.
- Review Canvas: if the diff is complex, verify that a matching docs/harness/canvas/YYYY-MM-DD-<task-id>.md exists and includes approaches considered, hot-path complexity, at least two edge cases, and a breakage-risk table.
- Harness script changes under docs/harness/bin/* are process-critical. Inspect shell safety, path handling, parseability, read-only guarantees, and whether the script weakens any existing gate.

## Key Fake-Success Patterns (hunt these actively)

1. Tests green only because local-embeddings feature was used; CI Linux parity fails.
2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
4. Clippy clean but unwrap/expect in hot MCP handler, storage, or hook paths.
5. Snapshot/attestation tests pass but Merkle or crypto behavior changed.
6. Hooks (session_end, post_tool_use, etc.) or intelligence modules changed without integration coverage.
7. Harness doctor or sensors would have caught this but were not run.
8. Progress docs (harness or active plan) not updated for a domain change.
9. Cross-SDK (python/typescript) contract drift not reflected.
10. Reviewer is being shown a self-referential or incomplete prompt (call it out).
11. Security boundary drift: static/read-only default weakened, autonomous execution implied, missing ADR/sandbox/egress/target contract, credential mounts allowed, or Anthropic C/C++/ASAN pipeline imported as default.

## Diff Under Review

```diff
commit d1b46f1698adb9460a509ccf538cd4627c9509c4
Author: Ronaldo Martins <ron@ldinho.com.br>
Date:   Tue Jun 16 08:09:57 2026 -0300

    ci(harness): add agentshield loop (#89)

diff --git a/.github/workflows/agentshield-loop.yml b/.github/workflows/agentshield-loop.yml
new file mode 100644
index 0000000..a2fc1c9
--- /dev/null
+++ b/.github/workflows/agentshield-loop.yml
@@ -0,0 +1,54 @@
+name: AgentShield Loop
+
+on:
+  schedule:
+    - cron: "17 7 * * 1"
+  workflow_dispatch:
+    inputs:
+      max_iterations:
+        description: "Bounded loop iteration cap"
+        required: false
+        default: "1"
+
+permissions:
+  contents: read
+
+concurrency:
+  group: agentshield-loop-${{ github.ref }}
+  cancel-in-progress: true
+
+jobs:
+  scan:
+    name: AgentShield Scan
+    runs-on: ubuntu-latest
+    timeout-minutes: 20
+    steps:
+      - uses: actions/checkout@v6
+
+      - uses: dtolnay/rust-toolchain@stable
+
+      - uses: Swatinem/rust-cache@v2
+        with:
+          key: agentshield-loop-v0.8.7
+
+      - name: Install AgentShield
+        run: |
+          cargo install \
+            --git https://github.com/limaronaldo/agentshield \
+            --tag v0.8.7 \
+            --features full \
+            --locked \
+            --force
+
+      - name: Run bounded AgentShield loop
+        env:
+          LOOP_MAX_ITERATIONS: ${{ github.event.inputs.max_iterations || '1' }}
+          LOOP_STATE_WRITE: "1"
+        run: bash scripts/run-agentshield-loop.sh
+
+      - name: Upload loop state
+        if: always()
+        uses: actions/upload-artifact@v7
+        with:
+          name: agentshield-loop-state
+          path: docs/loops/agentshield-scan/STATE.md
diff --git a/CHANGELOG.md b/CHANGELOG.md
index 9c4bc49..bce8322 100644
--- a/CHANGELOG.md
+++ b/CHANGELOG.md
@@ -16,0 +17,3 @@ package registries, and dated external sources.
+- **AgentShield loop scaffold** — Added a bounded weekly/manual security scan
+  loop with a repository skill, state handoff, local Make/just target, and
+  explicit high-severity AgentShield gate.
diff --git a/Makefile b/Makefile
index a2971fe..5eb1877 100644
--- a/Makefile
+++ b/Makefile
@@ -50,0 +51,4 @@ docs:
+
+.PHONY: loop-security
+loop-security:
+	@bash scripts/run-agentshield-loop.sh
diff --git a/docs/harness/progress.md b/docs/harness/progress.md
index f4f138d..5825f74 100644
--- a/docs/harness/progress.md
+++ b/docs/harness/progress.md
@@ -105,0 +106,14 @@ Esta sprint implementa a **camada operacional** (o "harness engineering" process
+## AgentShield loop MVL — 2026-06-16
+
+- Added the minimum viable loop components for a bounded AgentShield security
+  scan:
+  - Automation: `.github/workflows/agentshield-loop.yml` runs weekly and by
+    manual dispatch.
+  - Skill: `skills/agentshield-scan/SKILL.md`.
+  - State: `docs/loops/agentshield-scan/STATE.md`.
+  - Gate: `scripts/run-agentshield-loop.sh`, also exposed as `make
+    loop-security` and `just loop-security`.
+- Scope is static triage only: no automatic remediation, no production
+  credentials, no auto-commit, and `LOOP_MAX_ITERATIONS` is capped at 5.
+- The loop is optional and not part of required PR branch protection.
+
diff --git a/docs/loops/agentshield-scan/STATE.md b/docs/loops/agentshield-scan/STATE.md
new file mode 100644
index 0000000..facfd08
--- /dev/null
+++ b/docs/loops/agentshield-scan/STATE.md
@@ -0,0 +1,33 @@
+# AgentShield Scan State File
+
+* **Objective**: Run automated AgentShield security scans to detect and triage high-severity vulnerabilities, credential leaks, and supply chain risks.
+* **Scope**: Repository-wide (`.`)
+* **Non-goals**: No automatic remediation, commits, pushes, dependency updates, or production credential access.
+* **Stop Condition**: `scripts/run-agentshield-loop.sh` exits with `0` and no new high-severity findings are reported.
+* **Hard Stop**: `LOOP_MAX_ITERATIONS` defaults to `1` and is capped at `5`.
+
+## Feasibility Check
+| Condition | Status | Evidence |
+|---|---|---|
+| Task recurs at least weekly | PASS | `.github/workflows/agentshield-loop.yml` runs weekly and can be dispatched manually. |
+| Objective gate exists | PASS | AgentShield exits non-zero on `--fail-on high`; wrapper propagates the exit code. |
+| Agent can execute verification | PASS | Local CLI check: `agentshield --version` is available before the loop runs. |
+| Hard stopping mechanism exists | PASS | Wrapper validates `LOOP_MAX_ITERATIONS` and refuses values above `5`. |
+
+## Current Iteration
+* **Iteration**: #1
+* **Planned Change**: Initialize the MVL components and validate the first bounded scan.
+* **Expected Evidence**: `bash scripts/run-agentshield-loop.sh` exits successfully and appends a redacted evidence row here.
+
+## Evidence Log
+| Time | Command / Action | Result (Pass/Fail) | Notes / Findings |
+|---|---|---|---|
+| 2026-06-16T03:12:44Z | `agentshield scan .` | PASS | No new high+ AgentShield findings. |
+
+## Risks & Mitigations
+| Risk | Mitigation | Status |
+|---|---|---|
+| Credentials leakage | Strip env vars and filter logs in loop automation | Active |
+| Pre-existing scanner issues | Establish `.agentshield-baseline.json` only after explicit review; gate only on new findings when present | Active |
+| Infinite loop or runaway automation | Wrapper caps iterations and workflow has a 20-minute timeout | Active |
+| CI noise from optional security loop | Workflow is scheduled/manual and not a required PR check | Active |
diff --git a/justfile b/justfile
index 98d537a..a4bd7a4 100644
--- a/justfile
+++ b/justfile
@@ -35,0 +36,3 @@ docs:
+
+loop-security:
+    @bash scripts/run-agentshield-loop.sh
diff --git a/scripts/run-agentshield-loop.sh b/scripts/run-agentshield-loop.sh
new file mode 100755
index 0000000..1579db6
--- /dev/null
+++ b/scripts/run-agentshield-loop.sh
@@ -0,0 +1,128 @@
+#!/usr/bin/env bash
+set -euo pipefail
+
+STATE_FILE="${LOOP_STATE_FILE:-docs/loops/agentshield-scan/STATE.md}"
+SCAN_PATH="${LOOP_SCAN_PATH:-.}"
+MAX_ITERATIONS="${LOOP_MAX_ITERATIONS:-1}"
+BASELINE_FILE="${AGENTSHIELD_BASELINE:-.agentshield-baseline.json}"
+WRITE_BASELINE="${LOOP_WRITE_BASELINE:-0}"
+STATE_WRITE="${LOOP_STATE_WRITE:-1}"
+FAIL_ON="${AGENTSHIELD_FAIL_ON:-high}"
+
+die() {
+  echo "error: $*" >&2
+  exit 2
+}
+
+redact() {
+  sed -E \
+    -e 's/(Bearer|token|api[_-]?key|secret|password)[=: ][^[:space:]]+/\1=[REDACTED]/Ig' \
+    -e 's/[A-Za-z0-9_\/+=.-]{32,}/[REDACTED]/g'
+}
+
+markdown_escape() {
+  tr '\n' ' ' | sed -E 's/[|`]/ /g; s/[[:space:]]+/ /g; s/^ //; s/ $//'
+}
+
+append_state_row() {
+  local status="$1"
+  local summary="$2"
+  local timestamp tmp
+
+  [ "${STATE_WRITE}" = "1" ] || return 0
+  [ -f "${STATE_FILE}" ] || die "state file missing: ${STATE_FILE}"
+
+  timestamp="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
+  tmp="$(mktemp)"
+
+  awk -v row="| ${timestamp} | \`agentshield scan ${SCAN_PATH}\` | ${status} | ${summary} |" '
+    BEGIN { inserted = 0 }
+    {
+      print
+      if (!inserted && $0 ~ /^\|---\|---\|---\|---\|$/) {
+        print row
+        inserted = 1
+      }
+    }
+    END {
+      if (!inserted) {
+        print ""
+        print "## Evidence Log"
+        print "| Time | Command / Action | Result (Pass/Fail) | Notes / Findings |"
+        print "|---|---|---|---|"
+        print row
+      }
+    }
+  ' "${STATE_FILE}" > "${tmp}"
+  mv "${tmp}" "${STATE_FILE}"
+}
+
+case "${MAX_ITERATIONS}" in
+  ''|*[!0-9]*) die "LOOP_MAX_ITERATIONS must be a positive integer" ;;
+esac
+[ "${MAX_ITERATIONS}" -ge 1 ] || die "LOOP_MAX_ITERATIONS must be >= 1"
+[ "${MAX_ITERATIONS}" -le 5 ] || die "LOOP_MAX_ITERATIONS must be <= 5"
+
+command -v agentshield >/dev/null 2>&1 || die "agentshield CLI is not installed or not in PATH"
+[ -d "${SCAN_PATH}" ] || die "scan path is not a directory: ${SCAN_PATH}"
+[ -f "${STATE_FILE}" ] || die "state file missing: ${STATE_FILE}"
+
+echo "AgentShield loop"
+echo "version=$(agentshield --version | redact)"
+echo "scan_path=${SCAN_PATH}"
+echo "max_iterations=${MAX_ITERATIONS}"
+echo "fail_on=${FAIL_ON}"
+
+if [ "${WRITE_BASELINE}" = "1" ]; then
+  baseline_output_file="$(mktemp)"
+  echo "Writing baseline to ${BASELINE_FILE}"
+  agentshield scan "${SCAN_PATH}" \
+    --ignore-tests \
+    --write-baseline "${BASELINE_FILE}" \
+    --explain >"${baseline_output_file}" 2>&1 || {
+      summary="$(redact <"${baseline_output_file}" | head -n 20 | markdown_escape | cut -c1-240)"
+      append_state_row "FAIL" "Baseline write failed: ${summary}"
+      echo "Baseline write failed"
+      redact <"${baseline_output_file}" | head -n 40
+      rm -f "${baseline_output_file}"
+      exit 1
+    }
+  rm -f "${baseline_output_file}"
+  append_state_row "PASS" "Baseline written to ${BASELINE_FILE}; review before committing."
+fi
+
+exit_code=0
+for iteration in $(seq 1 "${MAX_ITERATIONS}"); do
+  echo "=== iteration ${iteration}/${MAX_ITERATIONS} ==="
+
+  output_file="$(mktemp)"
+  cmd=(agentshield scan "${SCAN_PATH}" --ignore-tests --fail-on "${FAIL_ON}" --explain)
+  if [ -f "${BASELINE_FILE}" ]; then
+    cmd+=(--baseline "${BASELINE_FILE}")
+  fi
+
+  set +e
+  "${cmd[@]}" >"${output_file}" 2>&1
+  exit_code=$?
+  set -e
+
+  if [ "${exit_code}" -eq 0 ]; then
+    append_state_row "PASS" "No new ${FAIL_ON}+ AgentShield findings."
+    echo "Gate status: PASS"
+    rm -f "${output_file}"
+    exit 0
+  fi
+
+  summary="$(redact <"${output_file}" | head -n 20 | markdown_escape | cut -c1-240)"
+  append_state_row "FAIL" "AgentShield exited ${exit_code}: ${summary}"
+  echo "Gate status: FAIL"
+  echo "Redacted findings summary:"
+  redact <"${output_file}" | head -n 40
+  rm -f "${output_file}"
+
+  if [ "${iteration}" -lt "${MAX_ITERATIONS}" ]; then
+    echo "Loop is bounded; no automatic remediation is attempted."
+  fi
+done
+
+exit "${exit_code}"
diff --git a/skills/agentshield-scan/SKILL.md b/skills/agentshield-scan/SKILL.md
new file mode 100644
index 0000000..2da4125
--- /dev/null
+++ b/skills/agentshield-scan/SKILL.md
@@ -0,0 +1,34 @@
+---
+name: agentshield-scan
+description: Weekly or manual security triage loop that executes AgentShield with a hard iteration cap and records the result in a repository state file.
+metadata:
+  short-description: Automated AgentShield security scanning loop
+---
+
+# AgentShield Security Scan Skill Instructions
+
+## Objective
+Run a bounded security scan on the repository using AgentShield, enforce a
+high-severity gate, and record outcomes so the loop can resume across sessions.
+This loop is for static security triage only; it does not remediate findings by
+itself.
+
+## Always Do
+- Check that the `agentshield` command is available on the path before running scans.
+- Run `bash scripts/run-agentshield-loop.sh` so the scan uses the repository's bounded wrapper.
+- Keep `LOOP_MAX_ITERATIONS` between 1 and 5; the default is 1.
+- Run `agentshield scan . --ignore-tests --fail-on high --explain` through the wrapper's gate path.
+- Use `.agentshield-baseline.json` only after explicit review; create it with `LOOP_WRITE_BASELINE=1`.
+- Append a structured entry to `docs/loops/agentshield-scan/STATE.md` after every iteration.
+
+## Never Do
+- Never auto-fix, auto-commit, or auto-push scanner findings from this loop.
+- Never disable the scanner or bypass high-severity findings without a documented safety exception.
+- Never commit raw API credentials, private keys, or tokens to the repository.
+- Never expose API secrets or tokens in execution logs.
+- Never bypass the gate validation on failure.
+- Never mount production credentials or run the loop with broader privileges than repository read access.
+
+## State Handoff
+- Read and update the local `STATE.md` file located at `docs/loops/agentshield-scan/STATE.md` after every iteration.
+- Record whether the run used a baseline, the iteration cap, and any non-zero exit code.
```

## Previous Review Context (if any)

(no previous review supplied for continuity)

## Output Contract (strict)

Your entire response must start with exactly one of:

PASS <one-line summary of what was reviewed and why it is safe>

or

FAIL <one-line summary of the most important problem(s)>

Then a short bullet list using [BLOCKER], [HIGH], [MED], [LOW].
At most 3 substantive findings. Evidence and location required for each.
If nothing substantive: exactly one bullet with [LOW] No issues found...

Remember: you are the external reviewer. Be evidence-driven and skeptical.

Machine-parseable verdict (required):
Add exactly one line, anywhere in the response, beginning with:
REVIEW_VERDICT: PASS <one-line summary>
or
REVIEW_VERDICT: FAIL <one-line summary>
This line is required for hard post-gate enforcement.
