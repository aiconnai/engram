# SDK dry-run blockers — review canvas

Task: sdk-release-dry-run; base 7ca7a9b8. The existing dry-run exercises release-only
paths that the installed-SDK live workflows do not fully cover.

## Decisions

- Reject rewriting every caller to use an absolute artifact path: the public
  verifier accepts relative paths and itself changes directory. Resolve once at
  its boundary after existence validation, before consumer installation.
- Reject stripping License-File or changing the SDK license metadata: the same
  archives validate when read by packaging 24.2. Set the build tool's parser floor
  instead of changing package content or suppressing Twine.
- Keep publication guards and flags intact; diagnose through dry-run only.

## Complexity

One directory canonicalization per verification. Existing archive validation and
hashing retain their complexity. The Python change constrains a build/check tool;
it adds no SDK runtime dependency.

## Cases and risks

| Case | Verification |
|---|---|
| Relative directory with spaces | Real npm init/install/import of a local fixture; offline and isolated cache; RED/GREEN |
| Absolute directory / downloaded artifacts | Existing CLI contract and real candidate dry-run |
| Wheel and sdist Metadata-Version 2.4 | Identical archives fail with parser 24.0 and pass with 24.2 |
| Publish/tag/SHA/environment protections | Existing workflow and signer regressions remain enabled |

No runtime Rust, database schema, SDK public API or version changes. The main
release remains blocked until corrected SDK dry-run evidence is available and
the owner chooses the final release SHA.
