# Engram Harness — External Reviewer Prompt

**Task**: release-attestation-helper
**Mode**: pre
**Date (UTC)**: 2026-11-08
**Scope**: prepare  base=85502ad884b48ce7bf893da551b0499005903887  head=85502ad884b48ce7bf893da551b0499005903887  tree=n/a
**Diff sha256**: d6e5c4f45bbd2c10769a3f2db7496a00a325663f40494cc6fa8a67bc08bd74bb

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
CHANGED_PATH=docs/harness/canvas/2026-11-07-release-attestation-helper.md

Paths changed but excluded from this diff (harness bookkeeping, not reviewed here):
EXCLUDED_PATH=docs/harness/progress/2026-10-26-remediation.md

## Diff Under Review

```diff
diff --git a/docs/harness/SPEC.md b/docs/harness/SPEC.md
index f935c26a67948b8ba977d1a2fcb32ae561396b21..92968f77622e7cf486d1ff2d32c198c891eb131c 100644
--- a/docs/harness/SPEC.md
+++ b/docs/harness/SPEC.md
@@ -258,0 +259,9 @@ on 60ed4eec; [plan](plans/2026-11-02-markdown-portability-core.md) and
+
+## Release execution follow-up — missing artifact signer, 2026-11-07
+
+Owner requested publication after PR265 merge. Preflight on85502ad8 found the Core
+workflow calls a helper removed in3128d444. Bounded local restoration, real bundle
+regressions in the existing packaging suite, cleanup/strict envelope checks, docs
+and verification are authorized necessary remediation. [Canvas](canvas/2026-11-07-release-attestation-helper.md).
+No Rust/API/schema/versions or governance implementation changes. Tag/upload waits
+for a fixed reviewed candidate and protected-publication configuration.
diff --git a/docs/harness/progress.md b/docs/harness/progress.md
index d1ed9b9ba1d13eecc93507965c6bf804645fb5ca..5a05a35950c8b8e13966769b259e9c5d9819fcb9 100644
--- a/docs/harness/progress.md
+++ b/docs/harness/progress.md
@@ -11 +11 @@
-| Last commit | `2c7544db` |
+| Last commit | `85502ad8` |
diff --git a/docs/harness/canvas/2026-11-07-release-attestation-helper.md b/docs/harness/canvas/2026-11-07-release-attestation-helper.md
new file mode 100644
index 0000000000000000000000000000000000000000..8fc894fe92afaf7b81342209778dec2e1766f4b4
--- /dev/null
+++ b/docs/harness/canvas/2026-11-07-release-attestation-helper.md
@@ -0,0 +1,26 @@
+# Release attestation helper — bounded repair
+
+Base85502ad8; publication request received from owner after PR265 merge.
+Actual Core dry-run42430325179 compiled binaries but bundle creation failed because
+verify-release-artifacts.sh invokes removed sign-candidate-manifest.sh. Restore the
+existing signature-envelope interface and test real create/verify/tamper paths.
+No tag, registry upload, protection weakening or settings change in this repair.
+
+Alternatives: removing local signatures weakens the artifact contract and is rejected;
+replacing the verifier/provenance scheme expands scope and is rejected. Restore the
+small helper from repository history, correcting cleanup and strict field validation.
+Identity remains checked by the existing GitHub provenance verifier; a self-contained
+envelope alone does not authenticate a publisher.
+
+Public seam: verify-release-artifacts.sh create-bundle and verify CLI, using harmless
+fixture binaries; real OpenSSL cryptography and actual checksum/SBOM/provenance checks.
+Existing PDF packaging CI test file owns the regression, so no CI invocation is added.
+Cases: Linux3binaries/Mac2, deterministic archive bytes, altered attestation/signature,
+missing fields, temporary-key cleanup and GitHub trust default fail-closed.
+Cost: one ephemeral RSA key per bundle, existing archive/hash complexity unchanged.
+Risk: restoring an old untracked dependency could leave CI broken; test real CLI from
+fixture checkout plus committed-helper manifest. Keys are synthetic and never logged.
+
+Required: RED/GREEN focused tests, shellcheck/syntax, full sensors, independent static
+review of exact new candidate, fresh human H1 before upstream, new remote matrix before
+release. Existing receipts apply only to their old heads; no receipt is fabricated.
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
