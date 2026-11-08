# Engram Harness — External Reviewer Prompt

**Task**: sdk-release-dry-run
**Mode**: pre
**Date (UTC)**: 2026-11-05
**Scope**: prepare  base=7ca7a9b8fad666e099d083622a3d8115c9abdde9  head=7ca7a9b8fad666e099d083622a3d8115c9abdde9  tree=n/a
**Diff sha256**: 2a835422bbf166a3c74b2e55636b1147333edcd0b88d845ed78471172f88c2cf

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

CHANGED_PATH=.github/workflows/sdk-release.yml
CHANGED_PATH=docs/harness/progress.md
CHANGED_PATH=docs/releases/channel-matrix.toml
CHANGED_PATH=scripts/test_sdk_release_workflow.py
CHANGED_PATH=scripts/verify-sdk-artifacts.sh
CHANGED_PATH=docs/harness/audits/2026-11-05-release-validation.md
CHANGED_PATH=docs/harness/canvas/2026-11-05-sdk-release-dry-run.md
CHANGED_PATH=docs/harness/plans/2026-11-05-sdk-release-dry-run.md

Paths changed but excluded from this diff (harness bookkeeping, not reviewed here):
EXCLUDED_PATH=docs/harness/progress/2026-10-26-remediation.md

## Diff Under Review

```diff
diff --git a/.github/workflows/sdk-release.yml b/.github/workflows/sdk-release.yml
index ebc3f92bd3bbec66c0fc077d2e1fbdb8ffbaeb5e..4c637df867b20c497924ef5481c56237107e7c61 100644
--- a/.github/workflows/sdk-release.yml
+++ b/.github/workflows/sdk-release.yml
@@ -155 +155 @@ jobs:
-          python3 -m pip install --disable-pip-version-check 'build>=1,<2' 'twine>=6,<7'
+          python3 -m pip install --disable-pip-version-check 'build>=1,<2' 'twine>=6,<7' 'packaging>=24.2'
diff --git a/docs/harness/progress.md b/docs/harness/progress.md
index 230a1d926e0371decd8fe5ba502bc5b7bb917893..129144e7b2a9caa95599e7a99e0f41ee5a2b18d8 100644
--- a/docs/harness/progress.md
+++ b/docs/harness/progress.md
@@ -11 +11 @@
-| Last commit | `d9915e99` |
+| Last commit | `7ca7a9b8` |
@@ -38 +38 @@
-- **Candidato integrado local (2026-11-01)**: remediação646636f6 + import1faa5c6f (inclui export9bca4efa), mantendo79889460; [log](progress/2026-11-01-markdown-release-integration.md). Trabalho local; nenhuma autorização remota ampliada.
+- **Release validation (2026-11-05)**: PR264 mesclado em main7ca7a9b8 com decisão humana; CI e SDKs live PASS. Nightly completo41966511833 em andamento; crates verificados e canais read-only PASS. SDK dry-run41966515107 encontrou parser Python e caminho npm; [plano do fix](plans/2026-11-05-sdk-release-dry-run.md), [auditoria](audits/2026-11-05-release-validation.md). Sem autorização de tag/publicação.
diff --git a/docs/releases/channel-matrix.toml b/docs/releases/channel-matrix.toml
index c108320505bebc0e932b6551a9f98e77c5cd5855..0668fcd819190b3c393817610393c144873fc686 100644
--- a/docs/releases/channel-matrix.toml
+++ b/docs/releases/channel-matrix.toml
@@ -10 +10 @@ schema_version = 2
-observed_utc = "2026-10-27T18:14:43Z"
+observed_utc = "2026-11-05T04:16:20Z"
diff --git a/scripts/test_sdk_release_workflow.py b/scripts/test_sdk_release_workflow.py
index c20a12b250a3aa6aaa18a6029e6f6c86f529f35a..a7b74004a22dd614352d8a1da1673b72ce9422f4 100644
--- a/scripts/test_sdk_release_workflow.py
+++ b/scripts/test_sdk_release_workflow.py
@@ -10,0 +11 @@ import hashlib
+import io
@@ -15,0 +17 @@ import subprocess
+import tarfile
@@ -202,0 +205,38 @@ class TestArtifactMetadata(unittest.TestCase):
+    @unittest.skipUnless(shutil.which("npm") and shutil.which("node"), "npm and node are required")
+    def test_relative_artifact_directory_installs_from_consumer_workdir(self):
+        matrix = tomllib.loads((REPO_ROOT / "docs/releases/channel-matrix.toml").read_text())
+        version = next(row["local_version"] for row in matrix["channels"] if row["id"] == "npm")
+        with tempfile.TemporaryDirectory() as tmp:
+            working = Path(tmp)
+            artifacts = working / "relative artifacts"
+            artifacts.mkdir()
+            package = {
+                "name": "engram-client", "version": version, "type": "module",
+                "exports": "./dist/index.js",
+            }
+            with tarfile.open(artifacts / f"engram-client-{version}.tgz", "w:gz") as archive:
+                for name, body in (
+                    ("package/package.json", json.dumps(package).encode()),
+                    ("package/dist/index.js", b"export class EngramClient {}\n"),
+                    ("package/dist/index.d.ts", b"export declare class EngramClient {}\n"),
+                ):
+                    entry = tarfile.TarInfo(name)
+                    entry.size = len(body)
+                    archive.addfile(entry, io.BytesIO(body))
+            args = [
+                "bash", str(REPO_ROOT / "scripts/verify-sdk-artifacts.sh"),
+                "--artifact-dir", "relative artifacts", "--expected-sha", "2" * 40,
+            ]
+            metadata = subprocess.run(
+                [*args, "--write-metadata", "npm"], cwd=working,
+                capture_output=True, text=True, timeout=30,
+            )
+            self.assertEqual(metadata.returncode, 0, metadata.stdout + metadata.stderr)
+            verified = subprocess.run(
+                [*args, "--channel", "npm"], cwd=working,
+                env={**os.environ, "npm_config_offline": "true", "npm_config_cache": str(working / "npm-cache")},
+                capture_output=True, text=True, timeout=60,
+            )
+            self.assertEqual(verified.returncode, 0, verified.stdout + verified.stderr)
+            self.assertIn("SDK artifacts verified", verified.stdout)
+
diff --git a/scripts/verify-sdk-artifacts.sh b/scripts/verify-sdk-artifacts.sh
index ed58e607b5cf5e1796b37b01da6409a82cca8db7..88ab6b3610aa385d484fa0cd4cd07153d12ea6a6 100755
--- a/scripts/verify-sdk-artifacts.sh
+++ b/scripts/verify-sdk-artifacts.sh
@@ -101,0 +102,3 @@ fi
+# Installation runs from a separate consumer directory; retain the caller's
+# artifact location across that directory change, including paths with spaces.
+artifact_dir="$(cd -- "$artifact_dir" && pwd -P)"
diff --git a/docs/harness/audits/2026-11-05-release-validation.md b/docs/harness/audits/2026-11-05-release-validation.md
new file mode 100644
index 0000000000000000000000000000000000000000..91e9012df799abed13b510d041430d0d35f6761e
--- /dev/null
+++ b/docs/harness/audits/2026-11-05-release-validation.md
@@ -0,0 +1,38 @@
+# Release validation — 2026-11-05
+
+Source:main7ca7a9b8fad666e099d083622a3d8115c9abdde9, tree
+bea3e892907a862182f531a2095d8317f566e72e, merged by explicit owner decision in PR264.
+This is preparation evidence, not publication authority or a human review receipt.
+
+## Observed results
+
+- Main CI41935478578 and Python/TypeScript live contracts passed on this SHA.
+- Full manual Nightly41966511833 runs on this SHA, including mutation testing.
+  Final outcome is pending at this snapshot.
+- Cargo1.96 verified packaging of both engram-types0.24.0 and engram-core0.24.0
+  passed offline from a clean target directory. Verification was enabled; no
+  `--no-verify` or registry upload was used. Packages contain11/892 files.
+- Release-channel read-only checks passed at2026-11-05T04:16:20Z. Observed latest:
+  Git tag/GitHub Release/Homebrew0.21.2, crates.io engram-core0.21.1,
+  PyPI engram-client0.4.0, npm engram-client0.3.0. Target tag/release0.24.0 absent.
+  Prepared local versions remain Core0.24.0 and SDKs0.7.0.
+- SDK release dry-run41966515107 passed preflight but failed both build lanes:
+  Python Twine rejected License-File; npm verified hashes then installed a relative
+  tarball from the wrong consumer directory. Publish jobs were skipped.
+
+## Confirmed fixes
+
+Runner logs show packaging24.0 and setuptools84.2.1. The generated wheel declares
+Metadata-Version2.4 and License-File:LICENSE. A clean local environment with
+Twine6.2.0 reproduces the rejection with packaging24.0; upgrading only packaging
+to24.2 accepts the identical wheel and sdist. This matches the field's defined
+[Core Metadata semantics](https://packaging.python.org/en/latest/specifications/core-metadata/#license-file-multiple-use).
+
+An offline fixture using the real npm consumer reproduces ENOENT for a relative
+artifact directory with spaces. Canonicalizing the directory before changing cwd
+passes that regression and all15 SDK workflow/signer/metadata tests.
+
+Raw execution logs and artifacts are kept in ignored
+`target/release-0.24.0-validation/` in the primary checkout. The fix is isolated on
+`fix/sdk-release-dry-run-2026-11-05`; final review, candidate dry-run and owner merge
+decision remain pending. Existing H1 receipts retain their original scopes.
diff --git a/docs/harness/canvas/2026-11-05-sdk-release-dry-run.md b/docs/harness/canvas/2026-11-05-sdk-release-dry-run.md
new file mode 100644
index 0000000000000000000000000000000000000000..a1725b65a6dc6d0274f12630e8f1fff94d8bc735
--- /dev/null
+++ b/docs/harness/canvas/2026-11-05-sdk-release-dry-run.md
@@ -0,0 +1,33 @@
+# SDK dry-run blockers — review canvas
+
+Task:sdk-release-dry-run; base7ca7a9b8. The existing dry-run exercises release-only
+paths that the installed-SDK live workflows do not fully cover.
+
+## Decisions
+
+- Reject rewriting every caller to use an absolute artifact path: the public
+  verifier accepts relative paths and itself changes directory. Resolve once at
+  its boundary after existence validation, before consumer installation.
+- Reject stripping License-File or changing the SDK license metadata: the same
+  archives validate when read by packaging24.2. Set the build tool's parser floor
+  instead of changing package content or suppressing Twine.
+- Keep publication guards and flags intact; diagnose through dry-run only.
+
+## Complexity
+
+One directory canonicalization per verification. Existing archive validation and
+hashing retain their complexity. The Python change constrains a build/check tool;
+it adds no SDK runtime dependency.
+
+## Cases and risks
+
+| Case | Verification |
+|---|---|
+| Relative directory with spaces | Real npm init/install/import of a local fixture; offline and isolated cache; RED/GREEN |
+| Absolute directory / downloaded artifacts | Existing CLI contract and real candidate dry-run |
+| Wheel and sdist Metadata-Version2.4 | Identical archives fail with parser24.0 and pass with24.2 |
+| Publish/tag/SHA/environment protections | Existing workflow and signer regressions remain enabled |
+
+No runtime Rust, database schema, SDK public API or version changes. The main
+release remains blocked until corrected SDK dry-run evidence is available and
+the owner chooses the final release SHA.
diff --git a/docs/harness/plans/2026-11-05-sdk-release-dry-run.md b/docs/harness/plans/2026-11-05-sdk-release-dry-run.md
new file mode 100644
index 0000000000000000000000000000000000000000..2ebe0149c732ba7b5a63759b9a9bd49b6b48d801
--- /dev/null
+++ b/docs/harness/plans/2026-11-05-sdk-release-dry-run.md
@@ -0,0 +1,26 @@
+# SDK release dry-run follow-up
+
+Task: sdk-release-dry-run. Base:7ca7a9b8fad666e099d083622a3d8115c9abdde9.
+Owner authorized post-merge CI/Nightly, release-channel observations and package
+dry-runs on2026-11-05. Main CI and both installed-SDK contracts passed. The full
+Nightly is run41966511833. SDK dry-run41966515107 exposed two confirmed blockers.
+
+## Bounded fixes and acceptance
+
+1. Resolve a verified artifact directory to an absolute physical path before
+   installing from a separate consumer directory. Preserve paths with spaces,
+   channel metadata, checksum checks and installation isolation.
+2. Require packaging>=24.2 in the Python build/check tool environment. The runner
+   retained24.0, which rejects License-File in the generated Metadata-Version2.4.
+   Keep the SDK's license content, public API and version unchanged.
+3. Add an offline real-npm consumer regression. It fails on the old verifier's
+   ENOENT, then succeeds after the path fix. Recheck the actual wheel/sdist with
+   Twine6.2.0 plus packaging24.0 (RED) and24.2 (GREEN), without rebuilding them.
+4. Preserve every publication/SHA/tag/signature/environment guard. Run workflow
+   tests, syntax, full local sensors, independent review and candidate dry-run.
+5. Record fresh channel observations and exact-SHA package checks. No tag, release,
+   registry upload, notice or settings change is authorized. Any fix merge remains
+   a separate owner decision.
+
+Evidence: [audit](../audits/2026-11-05-release-validation.md),
+[canvas](../canvas/2026-11-05-sdk-release-dry-run.md), and active remediation log.
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
