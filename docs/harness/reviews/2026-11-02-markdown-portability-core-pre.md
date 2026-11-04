# Engram Harness — External Reviewer Prompt

**Task**: markdown-portability-core
**Mode**: pre
**Date (UTC)**: 2026-11-02
**Scope**: prepare  base=60ed4eec9390c9ef526411c9b64ea5f69bfbbcce  head=60ed4eec9390c9ef526411c9b64ea5f69bfbbcce  tree=n/a
**Diff sha256**: 68827e4890a1ca98a3653d461bf7736596c947842b582756619786ff115d3782

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

## Files In Scope

CHANGED_PATH=docs/harness/SPEC.md
CHANGED_PATH=docs/harness/progress.md
CHANGED_PATH=GLOSSARY.md
CHANGED_PATH=docs/harness/canvas/2026-11-02-markdown-portability-core.md
CHANGED_PATH=docs/harness/plans/2026-11-02-markdown-portability-core.md

Paths changed but excluded from this diff (harness bookkeeping, not reviewed here):
EXCLUDED_PATH=docs/harness/progress/2026-10-26-remediation.md

## Diff Under Review

```diff
diff --git a/docs/harness/SPEC.md b/docs/harness/SPEC.md
index 35ad3938e7efab880af75b9fd7e8707c1ec46e03..de29807e5fd3af33c379560fb822bb8b60754afa 100644
--- a/docs/harness/SPEC.md
+++ b/docs/harness/SPEC.md
@@ -240,0 +241,7 @@ Sem publicar, alterar branch principal ou renovar mandato remoto da branch anter
+
+## Follow-up autorizado — Markdown architecture (2026-11-02)
+
+Owner approved local deepening of Markdown export/import together, preserving
+public behavior and extracting only shared auth policy. Separate worktree based
+on 60ed4eec; [plan](plans/2026-11-02-markdown-portability-core.md) and
+[evidence](progress/2026-11-02-markdown-portability-core.md). No remote authority added.
diff --git a/docs/harness/progress.md b/docs/harness/progress.md
index acac09344e399e63ac1398fe48005a87acf72b86..17c23f33d368c84fa8b3c35016a4683abc7227e0 100644
--- a/docs/harness/progress.md
+++ b/docs/harness/progress.md
@@ -39,0 +40 @@
+- **Markdown architecture (2026-11-02)**: local refactor authorized; [plan](plans/2026-11-02-markdown-portability-core.md), [evidence](progress/2026-11-02-markdown-portability-core.md).
diff --git a/GLOSSARY.md b/GLOSSARY.md
new file mode 100644
index 0000000000000000000000000000000000000000..3bcedb1c86cce63a3df77011d1a069b0722c6a58
--- /dev/null
+++ b/GLOSSARY.md
@@ -0,0 +1,10 @@
+# Engram
+
+Domain language for persistent memories and their human-editable representations.
+
+## Language
+
+**Markdown portability**:
+A human-editable Markdown representation of Engram memories, with deliberate
+import of reviewed changes into the canonical memory store.
+_Avoid_: Obsidian sync.
diff --git a/docs/harness/canvas/2026-11-02-markdown-portability-core.md b/docs/harness/canvas/2026-11-02-markdown-portability-core.md
new file mode 100644
index 0000000000000000000000000000000000000000..9bea6f56ca4f55c21b86463b0b8ac2e1bb40ba06
--- /dev/null
+++ b/docs/harness/canvas/2026-11-02-markdown-portability-core.md
@@ -0,0 +1,33 @@
+# Review Canvas — Markdown portability core
+
+## Intent and alternatives
+
+Approved design: one deep module for Markdown export/import and thin CLI/MCP
+adapters. Compared with only removing HandlerContext (retains MCP ownership), or
+introducing storage/filesystem traits (no second production implementation), move
+workflow ownership and use real SQLite/tempfiles through the typed seam.
+
+## Hot-path complexity
+
+Preserve existing snapshot query, full-batch render and directory-relative IO.
+Import keeps bounded file collection, parse-before-transaction, plan-before-write
+and one immediate transaction when confirmed. No extra network/provider calls.
+
+## Edge cases
+
+- Foreign and missing IDs remain indistinguishable; check on mutation connection.
+- Errors after commit retain unknown outcome, never claim applied=0.
+- Empty export preserves MCP error versus public Rust success-zero behavior.
+- Symlink/hardlink/ACL/active SQLite protections remain in moved implementation.
+
+## Breakage risk
+
+| Area | Risk | Validation |
+|---|---|---|
+| Permission env/required mode | Unintended widening | Existing permission/safety suites + typed seam tests |
+| Typed result translation | JSON/report drift | Existing adapter and vault tests + golden comparison |
+| Filesystem moves | Visibility/FFI behavior drift | Existing Unix safety tests and G1 regressions |
+| Imports/reexports | Source compatibility | Public imports unchanged; CI feature build |
+| Commit error | False success or rollback claim | Existing post-commit unknown-outcome regression |
+
+No schema, defaults, MCP catalog, SDK contract or gate-policy changes intended.
diff --git a/docs/harness/plans/2026-11-02-markdown-portability-core.md b/docs/harness/plans/2026-11-02-markdown-portability-core.md
new file mode 100644
index 0000000000000000000000000000000000000000..4dd295b67a89830726eed4f7d90e889ca779cc34
--- /dev/null
+++ b/docs/harness/plans/2026-11-02-markdown-portability-core.md
@@ -0,0 +1,31 @@
+# Markdown portability core — implementation plan
+
+Owner approved Q1–Q3 and the consolidated design on 2026-11-02. Base:
+60ed4eec9390c9ef526411c9b64ea5f69bfbbcce. Local branch/worktree:
+refactor/markdown-portability-core / .worktrees/markdown-portability-core.
+
+## Scope
+
+Deepen export and import together under src/portability/markdown/, eliminating
+HandlerContext and crate::mcp imports from that module. Keep public signatures,
+reexports, JSON output, defaults, environment modes, stored-workspace authorization,
+transaction ordering, confined filesystem behavior and uncertain commit outcomes.
+Extract only shared authorization primitives into auth; existing MCP paths delegate.
+No projection, storage schema, embeddings, HNSW, remote operation or new dependency.
+
+## Steps and acceptance
+
+1. Baseline existing Markdown tests with CI_REQUIRED_FEATURES and shared Cargo target.
+2. Move implementation and pure tests; use typed internal options/results/errors.
+   Preserve detailed MCP reports and public Rust compatibility conversions.
+3. Extract shared mode/row-access policy to auth and retain MCP reexports/delegation.
+4. Test domain behavior through the typed seam with SQLite and tempfile; retain
+   adapter contract, safety and direct-handler authorization regression suites.
+5. Run focused tests, fmt, clippy, generated MCP reference and full sensors. Record
+   results and deviations in the task log. No cargo clean.
+6. Freeze a local commit for independent review. Formal post-gate remains pending
+   without a genuine external review and human receipt; do not fabricate either.
+
+## Rollback
+
+Revert the isolated refactor commit; no data migration or deployment is needed.
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
This line is required for hard post-gate enforcement. A response with zero or
more than one such line is rejected.
