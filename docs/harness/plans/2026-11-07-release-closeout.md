# Release closeout — 2026-11-07

Task: `release-closeout-2026-11-07`, within the existing remediation sprint.
Owner requested execution after the 07/11 assessment. Local base is SDK fix
`2c7544db4936d5ac7d78a9c2afa26334c4a2fda7`, on merged main
`7ca7a9b8fad666e099d083622a3d8115c9abdde9`. Preserve the old worktrees and receipts.

## Delivery and authority

Prepare one reviewable candidate containing the two SDK dry-run fixes, the bounded
CDPATH follow-up, documentation reconciliation and the useful step-6 documentation
cleanup. Record the final candidate, checks and independent opinion. H1 remains a
human operation before upstream under the existing [policy](../CODE_REVIEW_POLICY.md).
Push/PR/dry-run follow that gate; merge, tag/release, registry upload, consumer notices,
settings and deletion remain separate decisions. No new product architecture or
experimental backend is adopted by this task.

## Acceptance

1. Keep packaging>=24.2 in the Python build/check environment. Preserve license,
   versions, SHA/tag/trust/dry-run/protected-environment guards.
2. Resolve the validated artifact directory before consumer cwd changes, ignoring
   inherited CDPATH. The existing real-npm regression covers paths with spaces,
   empty CDPATH, dot and an alternate matching directory. Record actual RED/GREEN.
3. Validate workflow tests, shell syntax/lint, real packages/clean consumers and
   applicable local live checks. Run full sensors serially; record unavailable
   checks accurately rather than inheriting prior PASS.
4. Reconcile current Nightly/merge/trust/Cargo evidence and risks without editing
   historical reviews. Full manual Nightly41966511833 passed seven jobs; today's
   scheduled run skipped mutations and is a different scope.
5. Update the original plan's entry point; publish a self-contained evolution v4
   proposal with current paths, namespaced IDs and separate essential/optional
   acceptance. RFC0004a and other proposals remain unaccepted.
6. Prepare cache-clear plus Markdown migration communication and a concrete final
   release decision checklist. Reobserve registries; create no monitoring service.

## Paths and verification

Functional SDK paths are `.github/workflows/sdk-release.yml`,
`scripts/verify-sdk-artifacts.sh`, `scripts/test_sdk_release_workflow.py`.
Documentation: CLAUDE/lessons, SPEC/progress/log, release readiness, original plan,
new closeout/evolution plans, risk register/known-issues and evidence/canvas.
The integrator owns shared files; helpers are read-only. No Rust/SDK public API,
schema, versions or governance-gate implementation changes.

Checks: 15 SDK workflow tests (including CDPATH subcases), shellcheck/bash syntax,
doc links, doctor, full sensors, and artifact-backed verification where tooling is
available. New source bytes invalidate affected reviews and receipts; bind the
independent opinion and operator bundle to the final frozen candidate.

Evidence: [closeout audit](../audits/2026-11-07-release-closeout.md),
[readiness](../../releases/v0.24.0-readiness.md),
[active log](../progress/2026-10-26-remediation.md).
