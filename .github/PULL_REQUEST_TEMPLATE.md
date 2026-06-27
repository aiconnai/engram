## Description

Brief description of the changes in this PR.

## Type of Change

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to change)
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Code refactoring
- [ ] Test update

## Related Issues

Fixes #(issue number)

## Changes Made

- Change 1
- Change 2
- Change 3

## Local harness loop (run before opening this PR)

- [ ] Session start: `bash docs/harness/bin/bootstrap.sh`
- [ ] During development: `bash docs/harness/bin/sensors.sh quick`
- [ ] Before opening the PR: full `bash docs/harness/bin/sensors.sh`

GitHub then confirms the same contracts via **required status checks**:
`Format`, `Clippy`, `Test (ubuntu-latest)`, `Documentation`, and `Harness Contract`.
`Harness Contract` runs `bootstrap.sh` plus the PR-title policy, which today only
blocks the literal `[codex]` marker — it is not a broad title-quality gate.

`Security Audit` and `Cargo Deny` run on PRs as **advisory** signals (not merge
blockers yet). Automated code review (Copilot / third-party) is **extra signal,
not authoritative** and does not block merge on its own.

## Testing

- [ ] I have added tests that prove my fix/feature works
- [ ] All new and existing tests pass (`cargo test`)
- [ ] I have run `cargo clippy` with no warnings
- [ ] I have run `cargo fmt`

## Documentation

- [ ] I have updated the documentation accordingly
- [ ] I have added doc comments to new public APIs

## Checklist

- [ ] My code follows the project's style guidelines
- [ ] I have performed a self-review of my code
- [ ] I have commented my code where necessary
- [ ] My changes generate no new warnings
- [ ] I have checked my code for potential security issues

## Screenshots (if applicable)

## Additional Notes

Any additional information reviewers should know.
