# Engram Harness — External Reviewer Prompt

**Task**: cloud-download-cancellation
**Mode**: pre
**Date (UTC)**: 2026-11-04
**Scope**: prepare  base=d9915e992c375751c3b88c5f5577f113b188a4bb  head=d9915e992c375751c3b88c5f5577f113b188a4bb  tree=n/a
**Diff sha256**: e6413ce4a75d09d7118d321e3589f31f6162ad8bce08db4bf828cbd99a58c719

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

CHANGED_PATH=src/sync/cloud_tests_failure.rs
CHANGED_PATH=docs/harness/canvas/2026-11-04-cloud-download-cancellation.md
CHANGED_PATH=docs/harness/plans/2026-11-04-cloud-download-cancellation.md

## Diff Under Review

```diff
diff --git a/src/sync/cloud_tests_failure.rs b/src/sync/cloud_tests_failure.rs
index fa20607f28e89389ceef4133977e2cf1ff9053cd..1d9da5dd510fb2402d3f30351c603b291682525d 100644
--- a/src/sync/cloud_tests_failure.rs
+++ b/src/sync/cloud_tests_failure.rs
@@ -21,0 +22,64 @@ fn write(path: &std::path::Path, bytes: &[u8]) {
+#[test]
+fn cancelled_download_with_queued_file_creation_leaves_no_temp_file() {
+    use std::future::Future;
+    use std::task::{Context, Poll, Waker};
+
+    fn occupy_pool() -> (std::sync::mpsc::Sender<()>, tokio::task::JoinHandle<()>) {
+        let (release, wait) = std::sync::mpsc::channel();
+        let (entered, ready) = std::sync::mpsc::channel();
+        let task = tokio::task::spawn_blocking(move || {
+            entered.send(()).expect("signal occupied blocking pool");
+            wait.recv_timeout(Duration::from_secs(10))
+                .expect("release occupied blocking pool");
+        });
+        ready
+            .recv_timeout(Duration::from_secs(10))
+            .expect("blocking pool worker started");
+        (release, task)
+    }
+
+    let runtime = tokio::runtime::Builder::new_current_thread()
+        .enable_all()
+        .max_blocking_threads(1)
+        .build()
+        .expect("single blocking worker runtime");
+    runtime.block_on(async {
+        let store = InMemoryCloudStore::default();
+        let cloud = CloudStorage::test_fixture_plaintext("bucket", "path.db", store);
+        let dir = tempfile::tempdir().unwrap();
+        let source = dir.path().join("source.db");
+        let target = dir.path().join("local.db");
+        write(&source, b"remote generation two");
+        cloud.upload(&source).await.unwrap();
+        write(&target, b"local generation one");
+
+        let mut download = Box::pin(cloud.download(&target));
+        let mut context = Context::from_waker(Waker::noop());
+
+        // Complete the real download's create_dir_all before queuing its file
+        // preparation. The occupied worker makes both poll boundaries exact.
+        let (release_directory, directory_blocker) = occupy_pool();
+        assert!(matches!(download.as_mut().poll(&mut context), Poll::Pending));
+        release_directory.send(()).unwrap();
+        directory_blocker.await.unwrap();
+        tokio::task::spawn_blocking(|| {}).await.unwrap();
+
+        let (release_file, file_blocker) = occupy_pool();
+        assert!(matches!(download.as_mut().poll(&mut context), Poll::Pending));
+        drop(download);
+        release_file.send(()).unwrap();
+        file_blocker.await.unwrap();
+        // One blocking worker drains the queued file operation before this
+        // fence, including disposal of a result whose caller was cancelled.
+        tokio::task::spawn_blocking(|| {}).await.unwrap();
+
+        assert_eq!(std::fs::read(&target).unwrap(), b"local generation one");
+        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
+            .unwrap()
+            .map(|entry| entry.unwrap().file_name())
+            .filter(|name| name.to_string_lossy().contains(".download-"))
+            .collect();
+        assert!(leftovers.is_empty(), "orphaned download files: {leftovers:?}");
+    });
+}
+
diff --git a/docs/harness/canvas/2026-11-04-cloud-download-cancellation.md b/docs/harness/canvas/2026-11-04-cloud-download-cancellation.md
new file mode 100644
index 0000000000000000000000000000000000000000..f7735df565a1d70ae02c2b6c68cc216766669712
--- /dev/null
+++ b/docs/harness/canvas/2026-11-04-cloud-download-cancellation.md
@@ -0,0 +1,36 @@
+# Cloud download cancellation — review canvas
+
+Task: cloud-download-cancellation; base d9915e99. S1 requires cancellation without
+residual partial files. The old future owns its guard while Tokio owns the pending
+blocking open: cancellation can unlink before the open creates the temporary file.
+
+## Approaches
+
+1. Keep a guard outside each async filesystem operation: rejected because an
+   abandoned blocking operation can outlive the guard, including on Windows.
+2. Put preparation and commit in one blocking task: rejected because cancellation
+   of queued preparation would also replace the target.
+3. Use two blocking ownership transfers: prepare all file bytes and permissions,
+   returning the cleanup guard; then commit by rename with the guard owned by that
+   operation. Selected. Discarding either queued/in-flight result cannot strand a
+   temporary file. A commit already queued may complete after cancellation.
+
+## Complexity and boundaries
+
+Preparation is O(N) time for N downloaded bytes; move the existing Vec into the
+blocking task rather than copying it. Two blocking dispatches, one atomic rename,
+file fsync and best-effort directory fsync. No network or database transaction is
+inside these tasks. The guard starts owning the path only after exclusive creation.
+
+## Edge cases and risk
+
+| Case | Expected behavior / verification |
+|---|---|
+| Cancel with file preparation queued | Original destination unchanged; no temporary remains after the queue drains; deterministic channel-barrier regression |
+| Cancel while preparation runs or its result awaits delivery | The blocking operation/result owns cleanup until the closed file can be removed |
+| Preparation or rename fails | Original error propagates and the closed temporary is removed; existing failure suite |
+| Cancel after commit is queued | Commit may finish; destination has one complete generation and no temporary remains |
+| New/existing destination | New file remains owner-only; existing mode retained; existing tests |
+
+Public behavior risk is limited to cancellation cleanup and internal scheduling.
+No live databases or external services are used for verification.
diff --git a/docs/harness/plans/2026-11-04-cloud-download-cancellation.md b/docs/harness/plans/2026-11-04-cloud-download-cancellation.md
new file mode 100644
index 0000000000000000000000000000000000000000..6c21214d41d72517fe571ab98c1d57c7a2a488e9
--- /dev/null
+++ b/docs/harness/plans/2026-11-04-cloud-download-cancellation.md
@@ -0,0 +1,24 @@
+# Cloud download cancellation — PR 264 follow-up
+
+Task: cloud-download-cancellation. Base: d9915e992c375751c3b88c5f5577f113b188a4bb.
+Owner requested the fix after the xhigh two-axis review found S1 partially met.
+
+## Scope and acceptance
+
+- Reproduce cancellation while the real download's file creation is queued in a
+  one-worker blocking pool. Use channel barriers and real temporary files, not sleeps.
+- Keep creation, writing, permission handling and cleanup ownership together in a
+  blocking preparation operation. A discarded preparation result cleans its file.
+- Transfer the prepared file's cleanup ownership into a blocking rename/directory
+  sync operation. Once commit is queued it may finish despite caller cancellation;
+  the destination is atomically replaced in full or remains intact.
+- Preserve public signatures, encryption validation, owner-only new files,
+  existing destination modes, SQLite protections and directory-fsync warnings.
+- Run RED/GREEN regression, cloud failure/encryption suites, format, cloud Clippy,
+  full local sensors, independent review and the PR's remote checks. Keep the old
+  receipt bound to its original HEAD; a new receipt is a human-only step.
+
+No schema, dependency, SDK, MCP catalog, release or deployment changes.
+
+Evidence: active [log](../progress/2026-10-26-remediation.md) and
+[canvas](../canvas/2026-11-04-cloud-download-cancellation.md).
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
