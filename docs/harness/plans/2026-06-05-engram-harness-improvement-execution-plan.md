# Engram Harness Improvement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve the Engram harness with the useful patterns found in the mbras harness, without weakening Engram's existing doctor, review-gate, sensor-exclusion, and verification discipline.

**Architecture:** Keep the current Engram harness as the control plane: `bootstrap.sh` orients sessions, `doctor.sh` validates harness consistency, `sensors.sh` remains the canonical deterministic gate, and `review-gate.sh` enforces independent review. Add improvements as explicit, additive layers: negative scope, review canvas, harness-script guard, baseline snapshot, optional sensor lanes, and evidence-only audit.

**Tech Stack:** Bash, Markdown, Git metadata, ripgrep, Cargo, Make or just, existing Engram harness files under `docs/harness/`.

---

## Scope

This plan covers only improvements inside the Engram repository.

It does not implement reverse adoption in mbras.

It does not change product behavior, MCP tools, storage schema, hooks, embeddings, SDKs, or CI workflows.

## Current Strengths To Preserve

| Area | Keep |
|---|---|
| Bootstrap | Mandatory read-only session orientation |
| Doctor | Drift and harness-consistency checks |
| Review gate | General multi-reviewer pre/post gate with strict `REVIEW_VERDICT` parsing |
| Sensor exclusions | Known-issue, reason, and progress registration contract |
| Verification | `VERIFICATION_MANIFEST.md` convention for explicit evidence and skips |
| Commits | Scoped Conventional Commit validation |

## Files To Create

| Path | Purpose |
|---|---|
| `docs/harness/WHAT_WE_DONT_DO.md` | Explicit negative scope for harness work |
| `docs/harness/canvas/README.md` | Rules for when complex-change evidence is required |
| `docs/harness/canvas/TEMPLATE.md` | Reusable review canvas template |
| `docs/harness/bin/baseline.sh` | Cheap static snapshot of repo and harness state |
| `docs/harness/bin/quarterly-audit.sh` | Evidence-only cleanup and drift audit |
| `docs/harness/audits/.gitkeep` | Keeps audit directory present |

## Files To Modify

| Path | Required change |
|---|---|
| `docs/harness/README.md` | Document new workflow pieces |
| `docs/harness/SPEC.md` | Add these improvements to active harness scope or next-iteration scope |
| `docs/harness/INVARIANTS.md` | Add only non-negotiable rules after deciding they are stable |
| `docs/harness/GATES.md` | Add negative-scope, canvas, baseline, audit, and sensor-lane rules |
| `docs/harness/CODE_REVIEW_POLICY.md` | Require reviewers to enforce negative scope and canvas evidence |
| `docs/harness/bin/bootstrap.sh` | Add `WHAT_WE_DONT_DO.md` to read order |
| `docs/harness/bin/doctor.sh` | Validate new files, references, script executability, and read-order wiring |
| `docs/harness/bin/sensors.sh` | Add optional modes while keeping no-argument full gate unchanged |
| `docs/harness/bin/review-gate.sh` | Include negative scope and canvas checks; guard harness script changes |
| `docs/harness/progress.md` | Record live state and verification evidence |
| `docs/harness/progress/2026-05-30-harness-bootstrap.md` | Record detailed implementation notes |

---

## Task 1: Add Negative Scope

**Files:**

- Create: `docs/harness/WHAT_WE_DONT_DO.md`
- Modify: `docs/harness/README.md`
- Modify: `docs/harness/GATES.md`
- Modify: `docs/harness/CODE_REVIEW_POLICY.md`
- Modify: `docs/harness/bin/bootstrap.sh`
- Modify: `docs/harness/bin/doctor.sh`
- Modify: `docs/harness/bin/review-gate.sh`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/progress/2026-05-30-harness-bootstrap.md`

- [ ] **Step 1: Create `docs/harness/WHAT_WE_DONT_DO.md`**

```markdown
# What We Do Not Do

This file defines negative scope for Engram harness work.

## Hard No

- Do not change storage schema, MCP tool contracts, hooks, embeddings, sync, or SDK public APIs as part of harness-only work.
- Do not remove code, dependencies, feature flags, docs, or scripts based only on static audit evidence.
- Do not make `docs/harness/bin/*` changes authoritative without independent post-review or human sign-off.
- Do not weaken the no-argument `sensors.sh` full gate.
- Do not treat generated review, progress, audit, or baseline artifacts as proof that implementation is correct.
- Do not add networked, paid, credentialed, or flaky checks to default harness gates.
- Do not bypass `doctor.sh` after changing harness docs, scripts, read order, or review policy.
- Do not use sensor exclusions to make production code look green.

## Allowed With Explicit Scope

- Add documentation-only plans under `docs/harness/plans/`.
- Add evidence-only audit reports under `docs/harness/audits/`.
- Add optional sensor modes if the default gate stays unchanged.
- Add review-canvas artifacts for complex changes.
- Propose product or cleanup follow-ups as separate tasks, issues, or ADRs.

## Review Rule

Reviewers must flag hidden scope creep against this file as `[HIGH]` or `[BLOCKER]` depending on impact.
```

- [ ] **Step 2: Update mandatory read order**

Change docs and bootstrap output to this order:

```text
docs/harness/SPEC.md
docs/harness/INVARIANTS.md
docs/harness/WHAT_WE_DONT_DO.md
docs/harness/GATES.md
docs/harness/CODE_REVIEW_POLICY.md
docs/harness/progress.md
active plan
```

- [ ] **Step 3: Update review prompt**

Add this instruction to both pre-review and post-review prompt construction in `docs/harness/bin/review-gate.sh`:

```text
Compare the change against docs/harness/WHAT_WE_DONT_DO.md. Flag hidden scope creep, gate weakening, or product changes bundled into harness work.
```

- [ ] **Step 4: Update doctor checks**

Add hard failures in `doctor.sh` for:

```text
missing docs/harness/WHAT_WE_DONT_DO.md
bootstrap.sh missing WHAT_WE_DONT_DO.md
README.md missing WHAT_WE_DONT_DO.md
GATES.md missing WHAT_WE_DONT_DO.md
CODE_REVIEW_POLICY.md missing WHAT_WE_DONT_DO.md
review-gate.sh missing WHAT_WE_DONT_DO.md
```

- [ ] **Step 5: Run targeted checks**

```bash
bash docs/harness/bin/doctor.sh
bash -n docs/harness/bin/bootstrap.sh
bash -n docs/harness/bin/doctor.sh
bash -n docs/harness/bin/review-gate.sh
```

Expected:

```text
doctor.sh passes
bash -n commands produce no output
```

- [ ] **Step 6: Record progress**

Append:

```markdown
## Harness negative-scope policy - 2026-06-05

- Added `docs/harness/WHAT_WE_DONT_DO.md`.
- Bootstrap, README, GATES, CODE_REVIEW_POLICY, doctor, and review-gate now reference the negative-scope policy.
- Purpose: prevent harness work from silently expanding into product, cleanup, or gate weakening.
```

---

## Task 2: Add Review Canvas

**Files:**

- Create: `docs/harness/canvas/README.md`
- Create: `docs/harness/canvas/TEMPLATE.md`
- Modify: `docs/harness/GATES.md`
- Modify: `docs/harness/CODE_REVIEW_POLICY.md`
- Modify: `docs/harness/bin/doctor.sh`
- Modify: `docs/harness/bin/review-gate.sh`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/progress/2026-05-30-harness-bootstrap.md`

- [ ] **Step 1: Create `docs/harness/canvas/README.md`**

```markdown
# Review Canvas

Review canvas files capture reasoning evidence for complex changes before implementation is judged complete.

Create a canvas when a change matches any trigger:

- More than 200 non-generated lines changed.
- Changes to storage schema, migrations, or data invariants.
- MCP tool surface changes.
- Hook, intelligence, consolidation, embedding, sync, or attestation behavior changes.
- Public SDK contract changes.
- New external dependency, backend, transport, cache, queue, or networked service.
- Harness gate, invariant, bootstrap, sensor, or review policy changes.

Canvas files are evidence, not approval. A post-review can still fail after a complete canvas.
```

- [ ] **Step 2: Create `docs/harness/canvas/TEMPLATE.md`**

```markdown
# Review Canvas: <task-id>

Date: YYYY-MM-DD
Owner: <human or agent>
Scope: <one sentence>

## Trigger

| Trigger | Evidence |
|---|---|
|  |  |

## Approaches Considered

| Approach | Decision | Reason |
|---|---|---|
|  | Accepted / Rejected |  |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
|  |  |  |  |

## Edge Cases

| Edge case | Verification plan |
|---|---|
|  |  |
|  |  |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
|  |  |  |  |

## Decision

Proceed / split / block:

Reason:
```

- [ ] **Step 3: Add GATES language**

Add:

```markdown
### Review Canvas Requirement

Complex changes require a canvas under `docs/harness/canvas/YYYY-MM-DD-<task-id>.md` before post-review.

The post-review gate must flag missing canvas evidence as `[HIGH]` or `[BLOCKER]` when a trigger is present.
```

- [ ] **Step 4: Add reviewer policy**

Add to `CODE_REVIEW_POLICY.md` and the review prompt:

```text
If the diff is complex, verify that a matching review canvas exists and includes approaches considered, hot-path complexity, at least two edge cases, and a breakage-risk table.
```

- [ ] **Step 5: Update doctor checks**

Require:

```text
docs/harness/canvas/README.md
docs/harness/canvas/TEMPLATE.md
GATES.md mentions Review Canvas
CODE_REVIEW_POLICY.md mentions Review Canvas
review-gate.sh mentions Review Canvas
```

- [ ] **Step 6: Run targeted checks**

```bash
bash docs/harness/bin/doctor.sh
bash -n docs/harness/bin/doctor.sh
bash -n docs/harness/bin/review-gate.sh
```

Expected:

```text
doctor.sh passes
bash -n commands produce no output
```

---

## Task 3: Add Harness Script Review Guard

**Files:**

- Modify: `docs/harness/bin/review-gate.sh`
- Modify: `docs/harness/GATES.md`
- Modify: `docs/harness/CODE_REVIEW_POLICY.md`
- Modify: `docs/harness/bin/doctor.sh`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/progress/2026-05-30-harness-bootstrap.md`

- [ ] **Step 1: Add post-review guard**

Implement this rule in `review-gate.sh`:

```text
If the reviewed diff changes docs/harness/bin/*, post-review must require explicit independent evidence that harness script changes were inspected.
```

The guard should fail when:

```text
post mode is evaluating harness script changes
and no external review artifact with REVIEW_VERDICT: PASS was supplied or generated
```

- [ ] **Step 2: Add GATES language**

```markdown
Changes to `docs/harness/bin/*` are process-critical and require independent post-review evidence.

A self-generated summary, missing review artifact, or advisory-only pre-review is not enough to pass script changes.
```

- [ ] **Step 3: Add reviewer policy**

```markdown
Harness script changes must be inspected directly. Reviewers must check shell safety, path handling, parseability, read-only guarantees, and whether the script weakens any existing gate.
```

- [ ] **Step 4: Update doctor checks**

Require:

```text
review-gate.sh mentions docs/harness/bin
GATES.md mentions docs/harness/bin
CODE_REVIEW_POLICY.md mentions harness script changes
```

- [ ] **Step 5: Run targeted checks**

```bash
bash docs/harness/bin/doctor.sh
bash -n docs/harness/bin/review-gate.sh
```

Expected:

```text
doctor.sh passes
bash -n docs/harness/bin/review-gate.sh produces no output
```

---

## Task 4: Add Lightweight Baseline Snapshot

**Files:**

- Create: `docs/harness/bin/baseline.sh`
- Modify: `docs/harness/bin/sensors.sh`
- Modify: `docs/harness/bin/doctor.sh`
- Modify: `docs/harness/README.md`
- Modify: `docs/harness/GATES.md`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/progress/2026-05-30-harness-bootstrap.md`

- [ ] **Step 1: Create `docs/harness/bin/baseline.sh`**

```bash
#!/usr/bin/env bash
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

OUT="docs/harness/.baseline-last"
TMP="$(mktemp "${TMPDIR:-/tmp}/engram-baseline.XXXXXX")" || exit 3

{
  echo "timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "branch=$(git branch --show-current 2>/dev/null || true)"
  echo "commit=$(git log -1 --format=%H 2>/dev/null || true)"
  if [ -n "$(git status --porcelain 2>/dev/null)" ]; then
    echo "dirty=yes"
  else
    echo "dirty=no"
  fi
  echo "cargo=$(cargo --version 2>/dev/null || echo missing)"
  echo "rustc=$(rustc --version 2>/dev/null || echo missing)"
  if command -v just >/dev/null 2>&1; then
    echo "ci_runner=just"
  else
    echo "ci_runner=make"
  fi
  echo "schema_version=$(rg -n 'SCHEMA_VERSION' src/storage 2>/dev/null | head -1 | sed 's/[[:space:]]\\+/ /g')"
  echo "mcp_reference_sections=$(rg -c '^## ' docs/MCP_TOOLS.md 2>/dev/null || echo 0)"
  echo "harness_scripts=$(find docs/harness/bin -maxdepth 1 -type f | wc -l | tr -d ' ')"
  echo "review_artifacts=$(find docs/harness/reviews -maxdepth 1 -type f -name '*.md' 2>/dev/null | wc -l | tr -d ' ')"
} > "$TMP"

mv "$TMP" "$OUT"
echo "Baseline written to $OUT"
cat "$OUT"
```

- [ ] **Step 2: Make it executable**

```bash
chmod +x docs/harness/bin/baseline.sh
```

Expected:

```text
No output
```

- [ ] **Step 3: Add `baseline` sensor mode**

Update `sensors.sh` so:

```bash
bash docs/harness/bin/sensors.sh baseline
```

runs:

```bash
bash docs/harness/bin/baseline.sh
bash docs/harness/bin/doctor.sh
```

Keep this unchanged:

```bash
bash docs/harness/bin/sensors.sh
```

Expected meaning:

```text
full canonical gate
```

- [ ] **Step 4: Document baseline**

Add:

```markdown
`baseline.sh` records cheap static repository facts in `docs/harness/.baseline-last`.

It is evidence for drift review, not a substitute for the full `sensors.sh` gate or CI.
```

- [ ] **Step 5: Update doctor checks**

Require:

```text
docs/harness/bin/baseline.sh exists
docs/harness/bin/baseline.sh is executable
sensors.sh mentions baseline
README.md mentions baseline.sh
GATES.md mentions baseline.sh
```

- [ ] **Step 6: Run targeted checks**

```bash
bash docs/harness/bin/baseline.sh
bash docs/harness/bin/sensors.sh baseline
bash docs/harness/bin/doctor.sh
bash -n docs/harness/bin/baseline.sh
bash -n docs/harness/bin/sensors.sh
```

Expected:

```text
baseline writes docs/harness/.baseline-last
sensors baseline passes
doctor.sh passes
bash -n commands produce no output
```

---

## Task 5: Add Optional Sensor Lanes

**Files:**

- Modify: `docs/harness/bin/sensors.sh`
- Modify: `docs/harness/GATES.md`
- Modify: `docs/harness/README.md`
- Modify: `docs/harness/bin/doctor.sh`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/progress/2026-05-30-harness-bootstrap.md`

- [ ] **Step 1: Add modes**

Add modes:

```text
full      current no-argument canonical gate
quick     doctor + cargo fmt --all -- --check + cargo check
docs      generated MCP reference check + rustdoc check
mcp       generated MCP reference check + MCP protocol tests
baseline  baseline.sh + doctor.sh
```

- [ ] **Step 2: Preserve default behavior**

Ensure:

```bash
bash docs/harness/bin/sensors.sh
```

still runs:

```text
the existing full canonical gate around make ci or just ci plus doctor
```

- [ ] **Step 3: Add GATES warning**

```markdown
Optional sensor lanes are developer aids. They do not replace the no-argument `sensors.sh` full gate for merge, completion, or handoff claims.
```

- [ ] **Step 4: Add doctor checks**

Require:

```text
sensors.sh mentions quick
sensors.sh mentions full
sensors.sh mentions docs
sensors.sh mentions mcp
sensors.sh mentions baseline
GATES.md says optional lanes do not replace the full gate
README.md lists sensor modes
```

- [ ] **Step 5: Run targeted checks**

```bash
bash docs/harness/bin/sensors.sh baseline
bash docs/harness/bin/doctor.sh
bash -n docs/harness/bin/sensors.sh
```

Expected:

```text
baseline mode passes
doctor.sh passes
bash -n docs/harness/bin/sensors.sh produces no output
```

---

## Task 6: Add Evidence-Only Quarterly Audit

**Files:**

- Create: `docs/harness/bin/quarterly-audit.sh`
- Create: `docs/harness/audits/.gitkeep`
- Modify: `docs/harness/README.md`
- Modify: `docs/harness/GATES.md`
- Modify: `docs/harness/bin/doctor.sh`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/progress/2026-05-30-harness-bootstrap.md`

- [ ] **Step 1: Create audit directory**

```bash
mkdir -p docs/harness/audits
touch docs/harness/audits/.gitkeep
```

Expected:

```text
No output
```

- [ ] **Step 2: Create `docs/harness/bin/quarterly-audit.sh`**

```bash
#!/usr/bin/env bash
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$REPO_ROOT"

TIMESTAMP="$(date -u +%Y-%m-%dT%H%M%SZ)"
REPORT_DIR="docs/harness/audits"
REPORT="$REPORT_DIR/${TIMESTAMP}-quarterly-audit.md"
LAST_FILE="docs/harness/.quarterly-audit-last"

mkdir -p "$REPORT_DIR"

need() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "FAIL: required command not found: $1" >&2
    exit 127
  fi
}

append_cmd() {
  local title="$1"
  local cmd="$2"
  local output
  local status

  {
    echo
    echo "### $title"
    echo
    echo '```bash'
    echo "$cmd"
    echo '```'
    echo
    echo '```text'
  } >> "$REPORT"

  output="$(bash -o pipefail -c "$cmd" 2>&1)"
  status=$?
  if [ -n "$output" ]; then
    printf '%s\n' "$output" >> "$REPORT"
  else
    echo "(no output)" >> "$REPORT"
  fi
  {
    echo "exit_status=$status"
    echo '```'
  } >> "$REPORT"
}

append_decision_table() {
  local title="$1"
  {
    echo
    echo "### $title"
    echo
    echo "| Item | Evidence | Decision | Owner | Follow-up |"
    echo "|---|---|---|---|---|"
    echo "|  |  | Keep / Archive / Delete |  |  |"
  } >> "$REPORT"
}

need git
need rg

cat > "$REPORT" <<EOF
# Quarterly Harness Audit

Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)
Repo: \`engram\`
Mode: evidence-only

This report gathers evidence for human cleanup and drift review. It does not declare pass/fail and does not delete, archive, or rewrite anything.

## How To Use

1. Review each evidence section.
2. Fill decision tables with \`Keep\`, \`Archive\`, or \`Delete\`.
3. Convert accepted cleanup into focused tasks or issues.
4. Keep exceptions documented in \`docs/harness/WHAT_WE_DONT_DO.md\`, \`docs/harness/INVARIANTS.md\`, or an ADR.
EOF

append_cmd "Current branch and commit" "git branch --show-current && git log -1 --oneline"
append_cmd "Working tree status" "git status --short"
append_cmd "Harness policy references" "rg -n 'WHAT_WE_DONT_DO|CODE_REVIEW_POLICY|review-gate|doctor.sh|sensors.sh|baseline.sh|quarterly-audit' docs/harness README.md AGENTS.md Claude.md 2>/dev/null | head -160"
append_cmd "Schema and migration references" "rg -n 'SCHEMA_VERSION|migration|migrations' src/storage tests docs 2>/dev/null | head -160"
append_cmd "MCP reference count and manual count risks" "rg -n 'MCP_TOOLS|[0-9]+\\+? tools|tools exposed|Available MCP Tools' README.md docs src sdks 2>/dev/null | head -160"
append_cmd "Temporary, legacy, and cleanup markers" "rg -n -i 'temporary|legacy|compat|deprecated|TODO: remove|remove after|sunset|hack|workaround' src tests docs sdks scripts 2>/dev/null | head -180"
append_cmd "Optional dependencies and feature gates" "rg -n -i 'optional = true|features =|default-features|\\[features\\]' Cargo.toml sdks docs 2>/dev/null | head -180"
append_cmd "Harness generated artifacts volume" "find docs/harness/reviews docs/harness/progress docs/harness/audits -maxdepth 1 -type f 2>/dev/null | sort | wc -l | tr -d ' '"

append_decision_table "Harness Policy Decisions"
append_decision_table "MCP And Docs Drift Decisions"
append_decision_table "Storage And Migration Decisions"
append_decision_table "Cleanup Follow-ups"

{
  echo
  echo "## Human Review Notes"
  echo
  echo "- Decisions:"
  echo "- Follow-up issues:"
  echo "- Exceptions approved:"
  echo "- Next audit date:"
} >> "$REPORT"

printf '%s %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$REPORT" > "$LAST_FILE"

echo "Quarterly audit evidence written to $REPORT"
echo "Last-audit pointer updated at $LAST_FILE"
```

- [ ] **Step 3: Make it executable**

```bash
chmod +x docs/harness/bin/quarterly-audit.sh
```

Expected:

```text
No output
```

- [ ] **Step 4: Document audit semantics**

Add:

```markdown
`quarterly-audit.sh` is evidence-only. It writes reports under `docs/harness/audits/` and updates `docs/harness/.quarterly-audit-last`.

It is not a pass/fail gate and must not delete, archive, or rewrite files.
```

- [ ] **Step 5: Update doctor checks**

Require:

```text
docs/harness/bin/quarterly-audit.sh exists
docs/harness/bin/quarterly-audit.sh is executable
docs/harness/audits exists
README.md mentions quarterly-audit.sh
GATES.md mentions quarterly-audit.sh
```

- [ ] **Step 6: Run targeted checks**

```bash
bash docs/harness/bin/quarterly-audit.sh
bash docs/harness/bin/doctor.sh
bash -n docs/harness/bin/quarterly-audit.sh
```

Expected:

```text
quarterly-audit writes docs/harness/audits/<timestamp>-quarterly-audit.md
doctor.sh passes
bash -n docs/harness/bin/quarterly-audit.sh produces no output
```

---

## Recommended Execution Order

| Order | Task | Reason |
|---|---|---|
| 1 | Task 1 | Negative scope prevents accidental expansion during the rest of the work |
| 2 | Task 2 | Review canvas creates evidence discipline before script changes grow |
| 3 | Task 3 | Harness-script guard protects the next script changes |
| 4 | Task 4 | Baseline adds cheap drift evidence |
| 5 | Task 5 | Sensor lanes should land only after baseline exists |
| 6 | Task 6 | Quarterly audit is useful but not urgent |

## Completion Criteria

- `bootstrap.sh` read order includes `WHAT_WE_DONT_DO.md`.
- `doctor.sh` validates every new policy file, directory, and script.
- No-argument `sensors.sh` remains the canonical full gate.
- `sensors.sh baseline` writes `docs/harness/.baseline-last`.
- `quarterly-audit.sh` writes evidence-only reports under `docs/harness/audits/`.
- `review-gate.sh post <task-id>` protects `docs/harness/bin/*` changes with independent evidence.
- `progress.md` and `docs/harness/progress/2026-05-30-harness-bootstrap.md` record each adopted change.

## Execution Handoff

Use one implementation mode:

1. Subagent-driven development: one fresh worker per task, with review after each task.
2. Inline execution: implement tasks in order, stopping after each task for doctor and review evidence.

Recommended choice: subagent-driven development for Tasks 1 to 3, then inline execution for Tasks 4 to 6 because the script changes are smaller after the guard exists.
