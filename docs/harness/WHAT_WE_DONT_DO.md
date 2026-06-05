# What We Do Not Do

This file defines negative scope for Engram harness work.

## Hard No

- Do not change storage schema, MCP tool contracts, hooks, embeddings, sync, or SDK public APIs as part of harness-only work.
- Do not remove code, dependencies, feature flags, docs, or scripts based only on static audit evidence.
- Do not make `docs/harness/bin/*` changes authoritative without independent post-review or human sign-off.
- Do not weaken the no-argument `sensors.sh` full gate.
- Do not treat generated review, progress, audit, or baseline artifacts as proof that implementation is correct.
- Do not add networked, paid, credentialed, or flaky checks to default harness gates.
- Do not bypass `doctor.sh` after changing harness docs, scripts, read order, or review policy.
- Do not use sensor exclusions to make production code look green.

## Allowed With Explicit Scope

- Add documentation-only plans under `docs/harness/plans/`.
- Add evidence-only audit reports under `docs/harness/audits/`.
- Add optional sensor modes if the default gate stays unchanged.
- Add review-canvas artifacts for complex changes.
- Propose product or cleanup follow-ups as separate tasks, issues, or ADRs.

## Review Rule

Reviewers must flag hidden scope creep against this file as `[HIGH]` or `[BLOCKER]` depending on impact.
