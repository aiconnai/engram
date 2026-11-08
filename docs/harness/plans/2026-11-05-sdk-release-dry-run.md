# SDK release dry-run follow-up

Task: sdk-release-dry-run. Base: 7ca7a9b8fad666e099d083622a3d8115c9abdde9.
Owner authorized post-merge CI/Nightly, release-channel observations and package
dry-runs on 2026-11-05. Main CI and both installed-SDK contracts passed. The full
Nightly is run 41966511833. SDK dry-run 41966515107 exposed two confirmed blockers.

## Bounded fixes and acceptance

1. Resolve a verified artifact directory to an absolute physical path before
   installing from a separate consumer directory. Preserve paths with spaces,
   channel metadata, checksum checks and installation isolation.
2. Require packaging>=24.2 in the Python build/check tool environment. The runner
   retained 24.0, which rejects License-File in the generated Metadata-Version 2.4.
   Keep the SDK's license content, public API and version unchanged.
3. Add an offline real-npm consumer regression. It fails on the old verifier's
   ENOENT, then succeeds after the path fix. Recheck the actual wheel/sdist with
   Twine 6.2.0 plus packaging 24.0 (RED) and 24.2 (GREEN), without rebuilding them.
4. Preserve every publication/SHA/tag/signature/environment guard. Run workflow
   tests, syntax, full local sensors, independent review and candidate dry-run.
5. Record fresh channel observations and exact-SHA package checks. No tag, release,
   registry upload, notice or settings change is authorized. Any fix merge remains
   a separate owner decision.

Evidence: [audit](../audits/2026-11-05-release-validation.md),
[canvas](../canvas/2026-11-05-sdk-release-dry-run.md), and active remediation log.
