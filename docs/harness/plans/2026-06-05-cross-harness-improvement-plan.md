# Cross-Harness Improvement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve the Engram harness using useful patterns from the mbras harness, and document the reverse adoption path for strengths Engram already has.

**Architecture:** Keep Engram's current model as the authority: `bootstrap.sh` orients, `doctor.sh` validates harness consistency, `sensors.sh` runs deterministic gates, and `review-gate.sh` handles independent review. Add mbras-inspired features as additive layers: negative-scope policy, review canvas, baseline snapshots, evidence-only periodic audits, and narrower sensor lanes.

**Tech Stack:** Bash, Markdown, Git, ripgrep, Cargo, Make or just, Engram harness docs under `docs/harness/`.

---

## Comparison Summary

| Area | Engram harness | mbras harness | Recommendation |
|---|---|---|---|
| Bootstrap | Mandatory, read-only, includes branch, dirty state, active sprint, last review, last sensors, read order | Mandatory, simpler session orientation | Keep Engram as-is |
| Consistency checker | `doctor.sh` validates drift, executable scripts, references, bootstrap contract | No equivalent dedicated doctor found | Preserve Engram advantage |
| Review gate | General `review-gate.sh`, multiple reviewer backends, pre/post, strict `REVIEW_VERDICT`, versioned artifacts, continuity | `codex-gate.sh`, Codex-specific, simpler verdict parsing, hard guard for harness script changes | Keep Engram generality, add mbras harness-script guard |
| Review policy | Dedicated `CODE_REVIEW_POLICY.md` | Policy distributed through gates, `WHAT_WE_DONT_DO.md`, and gate script prompt | Preserve Engram advantage |
| Negative scope | Embedded across SPEC/GATES, no dedicated file | Dedicated `WHAT_WE_DONT_DO.md` used by gates | Add dedicated Engram file |
| Complex-change evidence | Strong review prompt, but no structured canvas directory | `canvas/` artifacts and review-canvas requirement for complex work | Add Engram review canvas |
| Sensors | One primary deterministic gate around `make ci` or `just ci` plus doctor, with exclusion contract | Multiple lanes: quick, full, contract, dashboard, downstream, baseline, docs, manticore | Add optional lanes without weakening default gate |
| Baseline snapshot | `VERIFICATION_MANIFEST.md` documents verification records, but no shell baseline snapshot | `baseline.sh` writes `.baseline-last` | Add lightweight Engram baseline |
| Periodic audit | Decisions, known issues, plans, reviews | `quarterly-audit.sh` writes evidence-only audit reports | Add evidence-only audit for cleanup and drift |
| Sensor exclusions | Strict known-issue contract | No equivalent strict exclusion mechanism found | Preserve Engram advantage |
| Commit hygiene | `check-commit-msg.sh` validates scoped Conventional Commits | No equivalent in harness file list | Preserve Engram advantage |

## Implementation Rules

- Do not copy mbras scripts verbatim. They are domain-specific to mbras workspace routes, dashboards, downstream Svelte contracts, Manticore, and docs generators.
- Preserve Engram's default hard gate behavior. `bash docs/harness/bin/sensors.sh` must remain the full canonical gate unless an explicit mode is passed.
- Any change under `docs/harness/bin/`, `INVARIANTS.md`, `GATES.md`, `CODE_REVIEW_POLICY.md`, or bootstrap read order must update `progress.md` and the active progress log.
- Do not claim verification without running the command. If a check is skipped, record the reason in progress and, when available, through the `harness_verify` convention in `VERIFICATION_MANIFEST.md`.

## Files To Touch For Engram Improvements

| Path | Action | Responsibility |
|---|---|---|
| `docs/harness/WHAT_WE_DONT_DO.md` | Create | Negative scope and anti-pattern policy |
| `docs/harness/canvas/README.md` | Create | Explain when complex-change evidence is required |
| `docs/harness/canvas/TEMPLATE.md` | Create | Reusable review canvas template |
| `docs/harness/bin/baseline.sh` | Create | Produce lightweight static repository baseline |
| `docs/harness/bin/quarterly-audit.sh` | Create | Produce evidence-only cleanup and drift audit |
| `docs/harness/audits/.gitkeep` | Create | Keep audit directory present without committing generated reports unless desired |
| `docs/harness/README.md` | Modify | Document new files and workflow |
| `docs/harness/GATES.md` | Modify | Add negative-scope, review-canvas, baseline, and audit rules |
| `docs/harness/CODE_REVIEW_POLICY.md` | Modify | Require reviewers to enforce negative scope and canvas evidence |
| `docs/harness/INVARIANTS.md` | Modify | Add stable rules only if the new behavior is intended to be non-negotiable |
| `docs/harness/bin/bootstrap.sh` | Modify | Print new read order only after policy files exist |
| `docs/harness/bin/doctor.sh` | Modify | Validate new references, directories, and script executability |
| `docs/harness/bin/sensors.sh` | Modify | Add optional modes while preserving default full behavior |
| `docs/harness/bin/review-gate.sh` | Modify | Include negative-scope and canvas evidence in prompts; add harness-script guard |
| `docs/harness/progress.md` | Modify | Record the adopted harness improvement work |
| `docs/harness/progress/2026-05-30-harness-bootstrap.md` | Modify | Add detailed iteration notes |

## Task 1: Add Explicit Negative-Scope Policy

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

- [ ] **Step 1: Create the negative-scope document**

Use this initial content for `docs/harness/WHAT_WE_DONT_DO.md`:

```markdown
# What We Do Not Do

This file defines negative scope for the Engram harness. It prevents harness work from silently expanding into product, infrastructure, or cleanup work without explicit approval.

## Hard No

- Do not change storage schema, MCP tool contracts, hooks, embeddings, sync, or SDK public APIs as part of harness-only work.
- Do not remove code, dependencies, feature flags, docs, or scripts based only on static audit evidence.
- Do not make `docs/harness/bin/*` changes authoritative without an independent post-review or human sign-off.
- Do not weaken `sensors.sh` default behavior. Optional modes may be narrower, but the no-argument default remains the canonical full gate.
- Do not treat generated review, progress, audit, or baseline artifacts as proof that implementation is correct.
- Do not add networked, paid, or credentialed checks to default harness gates.
- Do not bypass `doctor.sh` after changing harness docs, scripts, read order, or review policy.
- Do not use exclusions to make production code look green. Exclusions require a known issue, a reason, and progress registration.

## Allowed With Explicit Scope

- Add documentation-only plans under `docs/harness/plans/`.
- Add evidence-only audit reports under `docs/harness/audits/`.
- Add optional sensor modes if the default gate stays unchanged.
- Add review-canvas artifacts for complex changes.
- Propose product or cleanup follow-ups as separate tasks, issues, or ADRs.

## Review Rule

Reviewers must flag hidden scope creep against this file as `[HIGH]` or `[BLOCKER]` depending on impact.
```

- [ ] **Step 2: Add the file to mandatory reading**

Update `docs/harness/README.md` and `docs/harness/bin/bootstrap.sh` so the read order becomes:

```text
SPEC.md -> INVARIANTS.md -> WHAT_WE_DONT_DO.md -> GATES.md -> CODE_REVIEW_POLICY.md -> progress.md -> active plan
```

- [ ] **Step 3: Teach reviewers to enforce it**

Update `docs/harness/bin/review-gate.sh` prompts so both `pre` and `post` include:

```text
Compare scope against docs/harness/WHAT_WE_DONT_DO.md.
Flag hidden scope creep, weakening of gates, or product changes bundled into harness work.
```

- [ ] **Step 4: Teach `doctor.sh` to require it**

Add checks that fail if:

```text
docs/harness/WHAT_WE_DONT_DO.md is missing
bootstrap.sh does not mention WHAT_WE_DONT_DO.md
README.md does not mention WHAT_WE_DONT_DO.md
GATES.md does not mention WHAT_WE_DONT_DO.md
review-gate.sh does not mention WHAT_WE_DONT_DO.md
```

- [ ] **Step 5: Run the minimum harness checks**

Run:

```bash
bash docs/harness/bin/doctor.sh
bash -n docs/harness/bin/bootstrap.sh
bash -n docs/harness/bin/doctor.sh
bash -n docs/harness/bin/review-gate.sh
```

Expected:

```text
PASS or OK output from doctor.sh
No output from bash -n commands
```

- [ ] **Step 6: Record progress**

Append a short dated note to `docs/harness/progress.md` and `docs/harness/progress/2026-05-30-harness-bootstrap.md`:

```markdown
## Harness negative-scope policy - 2026-06-05

- Added `docs/harness/WHAT_WE_DONT_DO.md` as explicit negative scope.
- Bootstrap, README, GATES, CODE_REVIEW_POLICY, doctor, and review-gate now reference it.
- Purpose: prevent harness work from silently expanding into product, cleanup, or gate weakening.
```

## Task 2: Add Review Canvas For Complex Changes

**Files:**

- Create: `docs/harness/canvas/README.md`
- Create: `docs/harness/canvas/TEMPLATE.md`
- Modify: `docs/harness/GATES.md`
- Modify: `docs/harness/CODE_REVIEW_POLICY.md`
- Modify: `docs/harness/bin/doctor.sh`
- Modify: `docs/harness/bin/review-gate.sh`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/progress/2026-05-30-harness-bootstrap.md`

- [ ] **Step 1: Create the canvas README**

Use this content for `docs/harness/canvas/README.md`:

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

- [ ] **Step 2: Create the reusable template**

Use this content for `docs/harness/canvas/TEMPLATE.md`:

```markdown
# Review Canvas: <task-id>

Date: YYYY-MM-DD
Owner: <human or agent>
Scope: <one sentence>

## Trigger

- Trigger matched:
- Files expected to change:

## Approaches Considered

| Approach | Why accepted or rejected |
|---|---|
|  |  |

## Hot Path Complexity

| Path | Time impact | Space impact | Notes |
|---|---|---|---|
|  |  |  |  |

## Edge Cases To Test Or Trace

| Edge case | Evidence command or manual trace |
|---|---|
|  |  |
|  |  |

## Breakage Risk

| Risk | Impact | Mitigation | Verification |
|---|---|---|---|
|  |  |  |  |

## Decision

- Proceed / split / block:
- Reason:
```

- [ ] **Step 3: Add gate language**

Add to `docs/harness/GATES.md`:

```markdown
### Review Canvas Requirement

Complex changes require a canvas under `docs/harness/canvas/YYYY-MM-DD-<task-id>.md` before post-review.
The review gate should flag missing canvas evidence as `[HIGH]` or `[BLOCKER]` when a trigger is present.
```

- [ ] **Step 4: Add reviewer instructions**

Add to `docs/harness/CODE_REVIEW_POLICY.md` and the `review-gate.sh` prompt:

```text
If the diff is complex, verify that a matching review canvas exists and contains approaches considered, complexity notes, at least two edge cases, and a breakage-risk table.
```

- [ ] **Step 5: Add doctor checks**

Make `doctor.sh` fail if:

```text
docs/harness/canvas/README.md is missing
docs/harness/canvas/TEMPLATE.md is missing
GATES.md does not mention Review Canvas
CODE_REVIEW_POLICY.md does not mention Review Canvas
review-gate.sh does not mention Review Canvas
```

- [ ] **Step 6: Run minimum checks**

Run:

```bash
bash docs/harness/bin/doctor.sh
bash -n docs/harness/bin/doctor.sh
bash -n docs/harness/bin/review-gate.sh
```

Expected:

```text
PASS or OK output from doctor.sh
No output from bash -n commands
```

## Task 3: Add Lightweight Baseline Snapshot

**Files:**

- Create: `docs/harness/bin/baseline.sh`
- Modify: `docs/harness/bin/sensors.sh`
- Modify: `docs/harness/bin/doctor.sh`
- Modify: `docs/harness/README.md`
- Modify: `docs/harness/GATES.md`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/progress/2026-05-30-harness-bootstrap.md`

- [ ] **Step 1: Create `baseline.sh`**

Use this implementation as the first version:

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
  echo "dirty=$(test -n "$(git status --porcelain 2>/dev/null)" && echo yes || echo no)"
  echo "cargo=$(cargo --version 2>/dev/null || echo missing)"
  echo "rustc=$(rustc --version 2>/dev/null || echo missing)"
  echo "ci_runner=$(command -v just >/dev/null 2>&1 && echo just || echo make)"
  echo "schema_version=$(rg -n 'SCHEMA_VERSION' src/storage 2>/dev/null | head -1 | sed 's/[[:space:]]\\+/ /g')"
  echo "mcp_reference_tools=$(rg -c '^## ' docs/MCP_TOOLS.md 2>/dev/null || echo 0)"
  echo "harness_scripts=$(find docs/harness/bin -maxdepth 1 -type f | wc -l | tr -d ' ')"
  echo "review_artifacts=$(find docs/harness/reviews -maxdepth 1 -type f -name '*.md' 2>/dev/null | wc -l | tr -d ' ')"
} > "$TMP"

mv "$TMP" "$OUT"
echo "Baseline written to $OUT"
cat "$OUT"
```

- [ ] **Step 2: Make it executable**

Run:

```bash
chmod +x docs/harness/bin/baseline.sh
```

Expected:

```text
No output
```

- [ ] **Step 3: Add a `baseline` sensor mode without changing the default**

Update `docs/harness/bin/sensors.sh` so:

```text
bash docs/harness/bin/sensors.sh
```

still runs the current full canonical gate, and:

```text
bash docs/harness/bin/sensors.sh baseline
```

runs:

```bash
bash docs/harness/bin/baseline.sh
bash docs/harness/bin/doctor.sh
```

- [ ] **Step 4: Document baseline behavior**

Add to `docs/harness/README.md` and `docs/harness/GATES.md`:

```markdown
`baseline.sh` records cheap static repository facts in `docs/harness/.baseline-last`.
It is evidence for drift review, not a substitute for `sensors.sh` or CI.
```

- [ ] **Step 5: Add doctor checks**

Make `doctor.sh` fail if:

```text
docs/harness/bin/baseline.sh is missing
docs/harness/bin/baseline.sh is not executable
sensors.sh does not mention baseline
README.md does not mention baseline.sh
GATES.md does not mention baseline.sh
```

- [ ] **Step 6: Run minimum checks**

Run:

```bash
bash docs/harness/bin/baseline.sh
bash docs/harness/bin/sensors.sh baseline
bash docs/harness/bin/doctor.sh
bash -n docs/harness/bin/baseline.sh
bash -n docs/harness/bin/sensors.sh
```

Expected:

```text
Baseline written to docs/harness/.baseline-last
PASS or OK output from doctor.sh
No output from bash -n commands
```

## Task 4: Add Evidence-Only Quarterly Audit

**Files:**

- Create: `docs/harness/bin/quarterly-audit.sh`
- Create: `docs/harness/audits/.gitkeep`
- Modify: `docs/harness/README.md`
- Modify: `docs/harness/GATES.md`
- Modify: `docs/harness/bin/doctor.sh`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/progress/2026-05-30-harness-bootstrap.md`

- [ ] **Step 1: Create audit directory**

Run:

```bash
mkdir -p docs/harness/audits
touch docs/harness/audits/.gitkeep
```

Expected:

```text
No output
```

- [ ] **Step 2: Create `quarterly-audit.sh`**

Use this first-version behavior:

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

Run:

```bash
chmod +x docs/harness/bin/quarterly-audit.sh
```

Expected:

```text
No output
```

- [ ] **Step 4: Document audit semantics**

Add to `docs/harness/README.md` and `docs/harness/GATES.md`:

```markdown
`quarterly-audit.sh` is evidence-only. It writes reports under `docs/harness/audits/` and updates `docs/harness/.quarterly-audit-last`.
It is not a pass/fail gate and must not delete, archive, or rewrite files.
```

- [ ] **Step 5: Add doctor checks**

Make `doctor.sh` fail if:

```text
docs/harness/bin/quarterly-audit.sh is missing
docs/harness/bin/quarterly-audit.sh is not executable
docs/harness/audits is missing
README.md does not mention quarterly-audit.sh
GATES.md does not mention quarterly-audit.sh
```

- [ ] **Step 6: Run minimum checks**

Run:

```bash
bash docs/harness/bin/quarterly-audit.sh
bash docs/harness/bin/doctor.sh
bash -n docs/harness/bin/quarterly-audit.sh
```

Expected:

```text
Quarterly audit evidence written to docs/harness/audits/<timestamp>-quarterly-audit.md
PASS or OK output from doctor.sh
No output from bash -n
```

## Task 5: Add Harness-Script Review Guard

**Files:**

- Modify: `docs/harness/bin/review-gate.sh`
- Modify: `docs/harness/GATES.md`
- Modify: `docs/harness/CODE_REVIEW_POLICY.md`
- Modify: `docs/harness/bin/doctor.sh`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/progress/2026-05-30-harness-bootstrap.md`

- [ ] **Step 1: Add the guard rule**

Add post-gate logic to `review-gate.sh`:

```text
If the reviewed diff changes `docs/harness/bin/*`, the post-gate must not silently pass without explicit reviewer evidence that harness script changes were reviewed.
```

The accepted implementation should do one of these:

```text
Require an external review artifact with `REVIEW_VERDICT: PASS`
or return FAIL with instructions to obtain human/cross-CLI review.
```

- [ ] **Step 2: Document the guard**

Add to `docs/harness/GATES.md`:

```markdown
Changes to `docs/harness/bin/*` require independent post-review evidence. A self-generated or missing review artifact is not authoritative for harness script changes.
```

- [ ] **Step 3: Add policy language**

Add to `docs/harness/CODE_REVIEW_POLICY.md`:

```markdown
Harness script changes are process-critical. Reviewers must inspect them directly and must not rely only on generated prompt summaries.
```

- [ ] **Step 4: Add doctor check**

Make `doctor.sh` fail if:

```text
review-gate.sh does not mention docs/harness/bin
GATES.md does not mention docs/harness/bin
CODE_REVIEW_POLICY.md does not mention harness script changes
```

- [ ] **Step 5: Run minimum checks**

Run:

```bash
bash docs/harness/bin/doctor.sh
bash -n docs/harness/bin/review-gate.sh
```

Expected:

```text
PASS or OK output from doctor.sh
No output from bash -n
```

## Task 6: Add Optional Sensor Lanes Without Weakening Default

**Files:**

- Modify: `docs/harness/bin/sensors.sh`
- Modify: `docs/harness/GATES.md`
- Modify: `docs/harness/README.md`
- Modify: `docs/harness/bin/doctor.sh`
- Modify: `docs/harness/progress.md`
- Modify: `docs/harness/progress/2026-05-30-harness-bootstrap.md`

- [ ] **Step 1: Define modes**

Add these modes to `sensors.sh`:

```text
full      current default canonical gate
quick     doctor + cargo fmt --all -- --check + cargo check --workspace
docs      MCP reference check + rustdoc docs check, if these are already part of make ci
mcp       generated MCP reference check + MCP protocol tests
baseline  baseline.sh + doctor.sh
```

- [ ] **Step 2: Preserve no-argument behavior**

Ensure:

```bash
bash docs/harness/bin/sensors.sh
```

continues to mean:

```text
full canonical gate
```

- [ ] **Step 3: Keep CI parity language clear**

Add to `docs/harness/GATES.md`:

```markdown
Optional sensor lanes are developer aids. They do not replace the no-argument `sensors.sh` full gate for merge or completion claims.
```

- [ ] **Step 4: Add doctor checks**

Make `doctor.sh` fail if:

```text
sensors.sh does not mention quick
sensors.sh does not mention full
sensors.sh does not mention baseline
GATES.md does not say optional lanes do not replace the full gate
README.md does not list sensor modes
```

- [ ] **Step 5: Run minimum checks**

Run:

```bash
bash docs/harness/bin/sensors.sh baseline
bash docs/harness/bin/doctor.sh
bash -n docs/harness/bin/sensors.sh
```

Expected:

```text
Baseline written to docs/harness/.baseline-last
PASS or OK output from doctor.sh
No output from bash -n
```

## Reverse Adoption Plan: Improvements mbras Can Take From Engram

This section exists because Engram has harness features that the mbras harness does not appear to have. Execute these steps from the mbras repository, not from Engram.

## Task M1: Add a Dedicated Doctor Script To mbras

**Files in mbras repo:**

- Create: `docs/harness/bin/doctor.sh`
- Modify: `docs/harness/README.md`
- Modify: `docs/harness/GATES.md`
- Modify: `docs/harness/bin/bootstrap.sh`
- Modify: `docs/harness/bin/sensors.sh`

- [ ] **Step 1: Port the concept, not Engram-specific checks**

The mbras `doctor.sh` should validate:

```text
SPEC.md, INVARIANTS.md, WHAT_WE_DONT_DO.md, GATES.md, README.md, and progress.md exist
bootstrap.sh, sensors.sh, codex-gate.sh, baseline.sh, quarterly-audit.sh are executable
bootstrap.sh mentions mandatory read order
progress.md active sprint and task match SPEC.md
audits/, reviews/, canvas/, and progress/ directories exist
```

- [ ] **Step 2: Wire it into sensors**

Make each mbras sensor mode run `doctor.sh` either before or after its lane-specific checks.

- [ ] **Step 3: Run mbras checks**

Run from mbras:

```bash
bash docs/harness/bin/doctor.sh
bash docs/harness/bin/sensors.sh quick
```

Expected:

```text
Doctor passes
Quick sensors pass
```

## Task M2: Extract Review Policy Into `CODE_REVIEW_POLICY.md`

**Files in mbras repo:**

- Create: `docs/harness/CODE_REVIEW_POLICY.md`
- Modify: `docs/harness/bin/codex-gate.sh`
- Modify: `docs/harness/GATES.md`
- Modify: `docs/harness/README.md`

- [ ] **Step 1: Create policy file**

The policy should capture:

```text
finding severity taxonomy
fake-success patterns
downstream contract break checks
WHAT_WE_DONT_DO scope checks
review canvas requirements
expected verdict format
```

- [ ] **Step 2: Update gate prompt**

Make `codex-gate.sh` include:

```text
Read docs/harness/CODE_REVIEW_POLICY.md and apply it as the authoritative review policy.
```

- [ ] **Step 3: Update docs**

Reference `CODE_REVIEW_POLICY.md` from `README.md` and `GATES.md`.

## Task M3: Generalize `codex-gate.sh` Into `review-gate.sh`

**Files in mbras repo:**

- Create: `docs/harness/bin/review-gate.sh`
- Modify: `docs/harness/bin/codex-gate.sh`
- Modify: `docs/harness/README.md`
- Modify: `docs/harness/GATES.md`

- [ ] **Step 1: Keep `codex-gate.sh` as compatibility wrapper**

Make `codex-gate.sh` call:

```bash
REVIEWER_CLI=codex bash docs/harness/bin/review-gate.sh "$@"
```

- [ ] **Step 2: Add strict verdict marker**

Require review artifacts to contain:

```text
REVIEW_VERDICT: PASS
```

or:

```text
REVIEW_VERDICT: FAIL
```

- [ ] **Step 3: Preserve mbras-specific prompt content**

Keep the current mbras checks:

```text
downstream contract breaks
auth and dashboard rollback risk
workspace build and test breakage
scope creep against WHAT_WE_DONT_DO.md
review canvas evidence for complex work
```

## Task M4: Add Known-Issue Sensor Exclusion Contract

**Files in mbras repo:**

- Create: `docs/harness/known-issues/README.md`
- Modify: `docs/harness/bin/sensors.sh`
- Modify: `docs/harness/GATES.md`
- Modify: `docs/harness/README.md`

- [ ] **Step 1: Create known-issues README**

Document that exclusions require:

```text
specific sensor name
known issue file path
short reason
progress.md registration before running the exclusion
```

- [ ] **Step 2: Add sensors flags**

Add flags compatible with Engram:

```text
--exclude-sensor <name>
--known-issue <path>
--reason <text>
```

- [ ] **Step 3: Block undocumented exclusions**

Make `sensors.sh` fail if an exclusion has no known issue file or no reason.

## Task M5: Add Verification Manifest Convention

**Files in mbras repo:**

- Create: `docs/harness/VERIFICATION_MANIFEST.md`
- Modify: `docs/harness/README.md`
- Modify: `docs/harness/GATES.md`

- [ ] **Step 1: Add manifest doc**

Adapt Engram's convention:

```text
Every verification record needs command, exit_code, output_summary, passed, evidence_path, skipped_reason, issue_numbers, workspace, and importance.
Skipped checks must be explicit negative evidence, not omitted.
```

- [ ] **Step 2: Reference from Done criteria**

Update README and GATES so completion claims require either passing verification evidence or explicit skip evidence.

## Recommended Order

| Order | Task | Reason |
|---|---|---|
| 1 | Task 1 | Negative scope is low-risk and improves every later gate |
| 2 | Task 2 | Canvas gives evidence for complex harness script changes |
| 3 | Task 5 | Harness-script guard protects the next script edits |
| 4 | Task 3 | Baseline adds cheap drift evidence |
| 5 | Task 6 | Sensor lanes should be added only after baseline exists |
| 6 | Task 4 | Quarterly audit is useful but lowest urgency |
| 7 | M1-M5 | Reverse adoption should happen in the mbras repo after Engram direction is accepted |

## Completion Criteria

- `bash docs/harness/bin/bootstrap.sh` prints the updated read order.
- `bash docs/harness/bin/doctor.sh` passes after every harness doc or script change.
- `bash docs/harness/bin/sensors.sh` with no args remains the canonical full gate.
- `bash docs/harness/bin/sensors.sh baseline` writes `docs/harness/.baseline-last`.
- `bash docs/harness/bin/quarterly-audit.sh` writes an evidence-only report under `docs/harness/audits/`.
- `review-gate.sh post <task-id>` requires independent evidence for harness script changes.
- `progress.md` and the active progress log record each adopted change.
