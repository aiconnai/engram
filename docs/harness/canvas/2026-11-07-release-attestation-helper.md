# Release attestation helper — bounded repair

Base85502ad8; publication request received from owner after PR265 merge.
Actual Core dry-run42430325179 compiled binaries but bundle creation failed because
verify-release-artifacts.sh invokes removed sign-candidate-manifest.sh. Restore the
existing signature-envelope interface and test real create/verify/tamper paths.
No tag, registry upload, protection weakening or settings change in this repair.

Alternatives: removing local signatures weakens the artifact contract and is rejected;
replacing the verifier/provenance scheme expands scope and is rejected. Restore the
small helper from repository history, correcting cleanup and strict field validation.
Identity remains checked by the existing GitHub provenance verifier; a self-contained
envelope alone does not authenticate a publisher.

Public seam: verify-release-artifacts.sh create-bundle and verify CLI, using harmless
fixture binaries; real OpenSSL cryptography and actual checksum/SBOM/provenance checks.
Existing PDF packaging CI test file owns the regression, so no CI invocation is added.
Cases: Linux3binaries/Mac2, deterministic archive bytes, altered attestation/signature,
missing fields, temporary-key cleanup and GitHub trust default fail-closed.
Cost: one ephemeral RSA key per bundle, existing archive/hash complexity unchanged.
Risk: restoring an old untracked dependency could leave CI broken; test real CLI from
fixture checkout plus committed-helper manifest. Keys are synthetic and never logged.

Required: RED/GREEN focused tests, shellcheck/syntax, full sensors, independent static
review of exact new candidate, fresh human H1 before upstream, new remote matrix before
release. Existing receipts apply only to their old heads; no receipt is fabricated.
