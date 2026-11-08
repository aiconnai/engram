# Release validation — 2026-11-05

Source: main 7ca7a9b8fad666e099d083622a3d8115c9abdde9, tree
bea3e892907a862182f531a2095d8317f566e72e, merged by explicit owner decision in PR264.
This is preparation evidence, not publication authority or a human review receipt.

## Observed results

- Main CI 41935478578 and Python/TypeScript live contracts passed on this SHA.
- Full manual Nightly 41966511833 runs on this SHA, including mutation testing.
  Fuzz, Miri, benchmark tracking and dependency checks passed. Extended property
  tests also passed; only mutations remain pending at this snapshot.
- Cargo 1.96 verified packaging of both engram-types 0.24.0 and engram-core 0.24.0
  passed offline from a clean target directory. Verification was enabled; no
  `--no-verify` or registry upload was used. Packages contain 11/892 files.
- Release-channel read-only checks passed at 2026-11-05T04:16:20Z. Observed latest:
  Git tag/GitHub Release/Homebrew 0.21.2, crates.io engram-core 0.21.1,
  PyPI engram-client 0.4.0, npm engram-client 0.3.0. Target tag/release 0.24.0 absent.
  Prepared local versions remain Core 0.24.0 and SDKs 0.7.0.
- SDK release dry-run 41966515107 passed preflight but failed both build lanes:
  Python Twine rejected License-File; npm verified hashes then installed a relative
  tarball from the wrong consumer directory. Publish jobs were skipped.

## Confirmed fixes

Runner logs show packaging 24.0 and setuptools 84.2.1. The generated wheel declares
Metadata-Version 2.4 and License-File: LICENSE. A clean local environment with
Twine 6.2.0 reproduces the rejection with packaging 24.0; upgrading only packaging
to 24.2 accepts the identical wheel and sdist. This matches the field's defined
[Core Metadata semantics](https://packaging.python.org/en/latest/specifications/core-metadata/#license-file-multiple-use).

An offline fixture using the real npm consumer reproduces ENOENT for a relative
artifact directory with spaces. Canonicalizing the directory before changing cwd
passes that regression and all 15 SDK workflow/signer/metadata tests.

The follow-up tree also rebuilt the real Python wheel/sdist (Twine PASS) and
TypeScript tarball (build and 104 tests PASS). Both installed in clean consumers
and passed the artifact-backed live verifier: the Python happy-path scenario
passed (two other scenario variants were not selected), the npm CRUD/search
contract passed, and the separate wrong-bearer and cross-workspace denial checks
passed. This preparation run uses 7ca7a9b8 as the SDK source identity; SDK product
files are byte-identical to that base. Frozen-candidate binding is recorded after
commit in the ignored completion bundle, without rewriting this snapshot.

Raw execution logs and artifacts are kept in ignored
`target/release-0.24.0-validation/` in the primary checkout. The fix is isolated on
`fix/sdk-release-dry-run-2026-11-05`; final review, human H1, corrected remote
dry-run and owner merge decision remain pending. GATES.md requires the post-gate
before upstream, so the branch remains local until its own receipt is issued. Existing H1 receipts retain their original scopes.

The first full local sensor attempt failed only in the existing context-budget
regression: the refreshed Limites paragraph omitted its required direct link to
WHAT_WE_DONT_DO.md. The link was restored without changing the test or exclusions;
full retry passed at 2026-11-05T04:42:56Z in 409 seconds, without exclusions.
Its log is fix-sensors-retry.log; every deterministic stage and doctor passed. The failed attempt remains
in fix-sensors.log and the tracked sensor history.
