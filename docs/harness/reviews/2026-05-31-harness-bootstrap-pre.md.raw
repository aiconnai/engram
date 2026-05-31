# Engram Harness — External Reviewer Prompt

**Task**: harness-bootstrap
**Mode**: pre
**Date (UTC)**: 2026-05-31

## Instructions for the Reviewer

You are acting as an independent senior engineer reviewing a diff for the engram project.
You were NOT the implementer. Your job is to find real problems introduced by the change.

Read the following documents (they are the source of truth for this review):

- docs/harness/SPEC.md
- docs/harness/INVARIANTS.md (process invariants — canonical)
- docs/harness/GATES.md (especially the fake-success patterns section)
- docs/harness/CODE_REVIEW_POLICY.md (this policy)
- docs/harness/README.md (workflow)
- Root INVARIANTS.md (data layer invariants for the memory system)

Then review the diff below.

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

## Diff Under Review

```diff
diff --git a/AGENTS.md b/AGENTS.md
index 88e8a8b..939d5e4 100644
--- a/AGENTS.md
+++ b/AGENTS.md
@@ -85,0 +86,17 @@ O Engram expõe 155+ ferramentas via MCP. Principais:
+## Harness de Desenvolvimento (Obrigatório no Início de Toda Sessão)
+
+Antes de qualquer planejamento ou edição, rode:
+
+```bash
+bash docs/harness/bin/bootstrap.sh
+```
+
+Em seguida leia (em ordem):
+- `docs/harness/SPEC.md`
+- `docs/harness/INVARIANTS.md`
+- `docs/harness/GATES.md`
+- `docs/harness/CODE_REVIEW_POLICY.md`
+- `docs/harness/progress.md`
+
+O harness garante que o trabalho seja retomável entre agentes (Claude Code CLI, Grok Build TUI, etc.) e entre sessões. Ele implementa as camadas de Context Engine, Planner, Memory Manager e Verifier diretamente no repositório.
+
@@ -87,0 +105,6 @@ O Engram expõe 155+ ferramentas via MCP. Principais:
+# Harness bootstrap + gates (obrigatório)
+bash docs/harness/bin/bootstrap.sh
+bash docs/harness/bin/doctor.sh
+bash docs/harness/bin/sensors.sh          # runs just ci + doctor (pode demorar)
+bash docs/harness/bin/review-gate.sh pre harness-bootstrap
+
diff --git a/CLAUDE.md b/CLAUDE.md
index 71cabed..4bbe0ca 100644
--- a/CLAUDE.md
+++ b/CLAUDE.md
@@ -67,0 +68,12 @@ engram/
+## Harness Discipline (Required at Session Start)
+
+Every agent session must begin with:
+
+```bash
+bash docs/harness/bin/bootstrap.sh
+```
+
+Then read in order: `docs/harness/SPEC.md`, `INVARIANTS.md`, `GATES.md`, `CODE_REVIEW_POLICY.md`, `progress.md`.
+
+This operational harness (inspired by proven backend patterns) ensures resumable, auditable, high-signal work across Claude Code CLI and Grok Build TUI.
+
@@ -70,0 +83,6 @@ engram/
+```bash
+bash docs/harness/bin/bootstrap.sh
+bash docs/harness/bin/doctor.sh
+bash docs/harness/bin/sensors.sh   # full deterministic gates (wraps just ci)
+```
+
@@ -82,0 +101 @@ cargo run
+just ci
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
