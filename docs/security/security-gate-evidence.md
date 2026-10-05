# Security gate: identity, digests, SARIF retention and findings policy

Task Q5 of the comprehensive improvement plan. The executable policy lives in
`docs/harness/GATES.md` ("Required aggregate security gate"); this page records the
evidence model behind it. Nothing here changes branch protection.

## Who provides identity

| Fact | Provided by | Never taken from |
|---|---|---|
| Commit under test | `--expected-sha "${{ github.sha }}"` in the CI job | the SARIF payload |
| Scanner and expected tool name | `--scanner`, `--tool` in the CI job | the SARIF payload |
| Whether the scanner ran / exit code | `--scanner-exit` (the job step order guarantees `0`) | an uploaded file |
| Whether a skip is allowed | `--allowed-skip` / `allowed_skips_by_event` in the matrix | a scenario or the payload |
| Approved exceptions | `docs/security/finding-exceptions.toml` (owner, approver, expiry) | SARIF `suppressions` |

A SARIF run must carry `versionControlProvenance[].revisionId` equal to the
expected SHA; a run with no revision is treated as unverifiable and blocks, unless the
supervisor attests identity with `--checkout-dir`: the job's own checkout HEAD must equal
the expected SHA. CodeQL with `upload: never` writes no provenance (confirmed on the first
CI run of PR #234), so the `codeql-security` job passes `--checkout-dir .`.

## Pinning

- Actions: every non-local `uses:` is a 40-hex commit SHA (checked on every PR by the
  `security-gate` job).
- Images: pinned by `@sha256:` digest. `zricethezav/gitleaks:v8.23.3` is pinned to the
  digest recorded by the local Docker store (overlay2, so the registry index digest).
  `semgrep/semgrep:1.169.0` and the Dockerfile bases (`rust:1-bookworm`,
  `debian:bookworm-slim`) are pinned by registry index digest (resolved 2026-10-05 with
  `docker buildx imagetools inspect`; the gitleaks digest was re-verified against the
  registry the same day). `docs/security/supply-chain-pins.toml` is empty; the checker
  fails on any new unpinned image without an unexpired ledger entry.
- Updates keep provenance: bump the SHA/digest together with the version comment and
  run `python3 scripts/check-workflow-supply-chain.py` plus the gate contract tests.

## SARIF retention and publication

| Workflow / job | Executes scanner | Retains SARIF | Publishes to code scanning | Judges findings |
|---|---|---|---|---|
| `ci.yml` `codeql-security` (PR gate) | CodeQL, `upload: never` | artifact `codeql-security-sarif`, 30 d | no | `check-security-findings.py` |
| `ci.yml` `semgrep-security` | Semgrep `--error` | artifact `semgrep-security-sarif`, 30 d | no | exit code |
| `ci.yml` `gitleaks-security` | Gitleaks `--exit-code 1` | artifact `gitleaks-security-sarif`, 30 d | no | exit code |
| `ci.yml` `agentshield-security` | AgentShield `fail-on: high` | no | no | exit code |
| `codeql.yml` | `analyze` job (read-only) | artifact per language, 30 d | `publish` job, non-PR only | n/a (publication) |
| `semgrep.yml`, `gitleaks.yml` | `scan` job (read-only) | artifact, 30 d | `publish` job, non-PR only | n/a |
| `agentshield.yml` | `scan` (PR, no upload) / `scan-publish` (non-PR) | no | `scan-publish` only | n/a |

Duplicate scanners (the standalone workflows and the `ci.yml` constituents) were
**not** removed: the aggregate does not yet retain AgentShield SARIF, so coverage and
retention cannot be proven for a deduplication.

## Advisory exceptions renewed in Q5

Nine expired exceptions were re-verified on 2026-10-05 against `Cargo.lock` with
`cargo audit --no-fetch` (RustSec advisory-db `ef6173c`, 2026-10-03) and renewed to
2026-12-31 (owner Ronaldo) because no semver-compatible dependency update fixes them.
They concentrate in the optional `libsql` stack (turso feature: `rustls-webpki`
0.102.8 x4, `h2` 0.3.27, `bincode`, `rustls-pemfile`) plus `paste` (tokenizers) and
`ttf-parser` (pdf feature: lopdf 0.45 drops it, but pdf-extract 0.12 pins lopdf ^0.42,
so removing it needs a product change).

Not renewed, handed to Q6 (owner of `Cargo.lock`), no exception added:

- RUSTSEC-2026-0285 `rustls` 0.23.36: update to `>=0.23.45`.
- RUSTSEC-2026-0235 `rkyv` 0.7.46: `cargo update -p rust_decimal --precise 1.43.0` removes
  it (verified on a scratch copy: no rkyv left, `cargo audit` clean for it). The first
  Q5 draft renewed it on weak evidence; the review fix round withdrew the renewal and
  removed the id from `.cargo/audit.toml` as well (parity check still passes).
- RUSTSEC-2026-0221 `event-listener` (warning): update to `>=5.4.2`.

Until Q6 lands, `Security Audit` and `Cargo Deny` stay red on this branch.

**Update after integration (task INT, 2026-10-05):** Q6 landed all three (rustls 0.23.45,
rust_decimal 1.43.0 without rkyv, event-listener 5.4.2). On the integrated tree `cargo audit`
with a freshly fetched database (still `ef6173c`) exits 0 with no vulnerability and two allowed
warnings, and `cargo deny check advisories bans licenses sources` passes. The records for
0098/0099/0104/0258 now name the libsql-only path (turso feature); RUSTSEC-2026-0253 (`lru`
0.16.4 via `aws-sdk-s3`, cloud feature, unsound warning) has a governed allowed-warning record
whose follow-up is to pin `BehaviorVersion` and bump `aws-sdk-s3`/`aws-config` with a bucket
canary. `governance/exceptions.toml` EXC-0001 (dual reqwest) was removed: the lock resolves only
reqwest 0.12.28.

## Ledger entries that need online action

None. The three entries that expired on 2026-11-05 were resolved early (issue #241):
digests pinned in `ci.yml`, `semgrep.yml` and the `Dockerfile`, and the ledger emptied.
