# Q5 report - aggregated security, findings policy, supply chain (Lane P)

Commits (branch claude/improvement-lane-p, local only): f4f547c feat(ci): fail-closed security gate verdicts and
findings policy; a follow-up docs commit refreshes progress.md "Last commit".

## Implemented
- scripts/check-security-gate.py: tri-state verdict pass/neutral/block (allowed skip = NEUTRAL, never PASS);
  matrix must cover 7 cases; no skip ever allowed on pull_request/push/schedule/workflow_dispatch; scenarios cannot
  grant skips; `required_dependency_job` (test) must still depend on security-gate (removing the needs fails);
  aggregate must keep `if: always()`. Existing --self-test-failure / --self-test-unrequired pass (failure self-test
  also covers unauthorized skip).
- scripts/check-security-findings.py + tests/fixtures/security_findings/ (27-scenario matrix.json, 15 SARIF, 6 TOML):
  high finding blocks with scanner exit 0; approved exception needs owner+approved_by+rationale+unexpired expiry
  (<=90 days ahead), invalid record rejects whole file; payload suppressions ignored; SARIF missing/malformed/wrong
  tool/no provenance/stale SHA => block; scanner not run => block unless supervisor --allowed-skip => neutral.
  Identity only from CLI.
- scripts/check-workflow-supply-chain.py + docs/security/supply-chain-pins.toml ledger: SHA pins, image digests (or
  expiring ledger), no write perms/secrets on pull_request-exposed jobs, no pull_request_target.
- ci.yml / codeql.yml / semgrep.yml / gitleaks.yml / agentshield.yml: measurement jobs read-only, publication jobs
  (security-events/contents write) only on non-PR events; bench split into bench (read-only) + bench-publish (push main);
  codeql-security read-only with SARIF artifact + findings policy; Semgrep/Gitleaks SARIF retained; Gitleaks pinned by
  digest; security-gate job now also runs the contract tests and the supply-chain check.
- Advisory exceptions: 10 re-verified and renewed to 2026-12-31 (owner Ronaldo); checker rejects expiry >90 days.
- Docs: GATES.md (security gate section + required-checks list refreshed to live state, D4), docs/security/
  security-gate-evidence.md, docs/security/finding-exceptions.toml (empty), lane-p log entry (PT-BR), progress.md.

## D1 advisory table (cargo audit --no-fetch, advisory-db ef6173c 2026-10-03; Cargo.lock untouched)
Full `cargo update --offline` on a scratch copy of the workspace (local index, may be stale) still reports all of these.
| Advisory | Crate | Lock path now | Decision |
|---|---|---|---|
| 2026-0049, -0098, -0099, -0104 | rustls-webpki 0.102.8 | libsql 0.9.30 -> hyper-rustls 0.25 -> rustls 0.22.4 (turso, optional) | renewed (pinned ^0.102 by rustls 0.22; libsql 0.9.30 latest stable; only 0.10.0-pre.* exists). 0.101.7 / AWS path gone from lock |
| 2026-0258 | h2 0.3.27 | libsql -> tonic 0.11 -> hyper 0.14 | renewed (patched only in h2 >=0.4.16) |
| 2025-0141 | bincode 1.3.3 | libsql | renewed (unmaintained, no patch) |
| 2025-0134 | rustls-pemfile 2.2.0 | libsql -> hyper-rustls 0.25 | renewed (unmaintained) |
| 2024-0436 | paste 1.0.15 | tokenizers (onnx-embed/local-embeddings) | renewed (unmaintained) |
| 2026-0192 | ttf-parser 0.25.1 | lopdf (pdf feature; in required CI set) | renewed (unmaintained) |
| 2026-0235 | rkyv 0.7.46 | rust_decimal optional-feature lock entry; `cargo tree -e all --all-features --target all -i` finds no edge | renewed (rust_decimal 1.42.1 still locks 0.7.46) |
| 2026-0285 | rustls 0.23.36 | aws-smithy-http-client / reqwest / hyper-rustls 0.27 | -> Q6 (>=0.23.45 compatible; D2, no exception) |
| 2026-0221 (warning, not excepted) | event-listener 5.4.1 | async-channel | -> Q6 (5.4.2) |
| 2026-0253 (warning, not excepted) | lru 0.16.4 | aws-sdk-s3 | no compatible fix found; informational |
Note: the old records' "default_graph = true / cloud default" claims were stale (default = ["openai"]); updated to
feature_gated=true, default_graph=false and verified with cargo tree on default and required feature sets.
Cross-manifest: .cargo/audit.toml and deny.toml ignore lists unchanged (same advisory set, parity check PASS);
governance/exceptions.toml (EXC-0001, reqwest dual version) is unrelated to advisories and unchanged.

## Verification (all run in worktree, exit 0 unless stated)
- python3 scripts/test_check_security_gate.py: 22 OK; test_check_security_findings.py: 11 OK;
  test_check_workflow_supply_chain.py: 26 OK; test_check_security_exceptions.py: 6 OK;
  test_check_quality_ci_contract.py (regression): 27 OK.
- check-security-gate.py --self-test-failure / --self-test-unrequired: PASS; check-workflow-supply-chain.py: PASS;
  check-security-exceptions.py (+ --self-test-expired / --self-test-missing-owner): PASS (10 records).
- actionlint on the 5 edited workflows: clean. PyYAML parses all workflows.
- `cargo audit --no-fetch`: exit 1, ONLY RUSTSEC-2026-0285 (plus 3 allowed warnings). `cargo deny check --disable-fetch
  advisories bans licenses sources`: advisories FAILED (same 0285), bans/licenses/sources ok.
- docs/harness/bin/doctor.sh: OK (expected warn about review artifact). run-offline-lane.sh: OFFLINE_LANE PASS
  components=6 checks=199. check-live-state.sh: PASS.
TDD RED evidence: findings tests before the script existed: 2 failures + 29 errors (script missing, exit 2); gate tests
with the new matrix against the old script: 6 failures + 9 errors ("matrix scenario disagrees"); exceptions horizon tests
before the 90-day rule: 11 failures + 2 errors (expired + no horizon rule). GREEN: counts above.

## Expected state
Security Audit and Cargo Deny stay RED on this branch until Q6 updates rustls (RUSTSEC-2026-0285). The
"Security exception policy" constituent is green again. Because Test (ubuntu-latest) needs security-gate, the required
chain would also be red until Q6.

## NOT RUN / unverified (be honest)
- No workflow ran in real CI. Unverified assumptions: (1) CodeQL analyze `upload: never` + `output: sarif-results`
  writes sarif-results/rust.sarif; (2) the CodeQL SARIF carries versionControlProvenance.revisionId equal to github.sha.
  If (2) is false the findings-policy step fails closed ("no revision provenance") on every PR - first CI run must
  confirm; (3) semgrep `--sarif-output=`; (4) gitleaks digest sha256:8e03b497... taken from the local Docker store
  (overlay2, RepoDigests of a pull by tag => index digest, not verified against the registry).
- semgrep/semgrep:1.169.0 and Dockerfile bases (rust:1-bookworm, debian:bookworm-slim) have no digest (offline); in the
  ledger, expiring 2026-11-05. Dockerfile not modified (outside Q5).
- CodeQL/Semgrep/Gitleaks/AgentShield not executed locally.
- Live branch-protection contexts file was not re-fetched (no remote access); the checker is exercised with the six
  live contexts from baseline section 4.1 as a fixture.

## Deviations / decisions to review
- ci.yml codeql-security and codeql.yml analyze keep `actions: read` (read-only) besides contents: read: private repos
  need it for CodeQL's analysis key; no write permission anywhere on PR-exposed jobs. Ruling said contents: read only.
- PR comments from bench removed (comment-on-alert/comment-always off); regression still fails the PR. Code scanning
  SARIF upload no longer happens on pull_request (AgentShield/Semgrep/Gitleaks/CodeQL standalone); PR gating is the
  ci.yml constituents. Standalone scanners were NOT deduplicated (AgentShield SARIF retention not provable).
- GATES.md edited (executable harness policy): security-gate section and the "Required checks no GitHub" list (D4)
  refreshed per the controller's allowance. Needs the reviewer's attention given the policy rule about GATES changes.
- scripts/ci.sh not touched (mirror of the required Test job; security checks live in the security-gate job).
- Untracked files from other tasks (docs/harness/bin/run-sandbox-smoke.sh, sandbox-adapter.py, docs/harness/checks/,
  docs/harness/tests/*sandbox*, fake_writer.py) were left alone.

---
## Fix report - review round 1 (commit cae024c fix(ci): pin CodeQL findings step and withdraw rkyv exception; plus progress refresh)

1. **rkyv (RUSTSEC-2026-0235) renewal withdrawn.** Evidence: index cache has rust_decimal 1.43.0 with no rkyv ^0.7 dep
   (1.42.1 still has it); in a scratch copy `cargo update -p rust_decimal --precise 1.43.0 --offline` leaves 0 rkyv entries
   in Cargo.lock and `cargo audit --no-fetch` has no rkyv finding. Removed the record from
   docs/security/advisory-exceptions.toml (now 9 governed records) and the id from .cargo/audit.toml (parity check PASS);
   routed to Q6 with no exception (header comment, evidence doc, lane-p log updated). `cargo audit --no-fetch` now reports
   2 vulnerabilities (0285 rustls, 0235 rkyv), both Q6. ttf-parser (0192) remediation now states lopdf 0.45 drops it but
   pdf-extract 0.12 pins lopdf ^0.42, so a product change is needed.
2. **CodeQL policy step pinned.** New scripts/test_check_security_ci_contract.py (16 tests): exactly one findings step in
   codeql-security with full argv (--scanner codeql --tool CodeQL --sarif sarif-results/rust.sarif --expected-sha
   "${{ github.sha }}" --scanner-exit 0 --exceptions docs/security/finding-exceptions.toml, no extra flags), no `if`,
   no continue-on-error / `|| true`, runs after analyze, analyze has `upload: never` + `output: sarif-results`, job is
   read-only, SARIF retained; Semgrep `--error`, Gitleaks `--exit-code 1` + digest, AgentShield `fail-on: high` pinned;
   security-gate job must run all security contract commands and keep the runtime matrix; self-check tests prove deleting
   the step or changing --expected-sha/--scanner-exit is detected. Wired into the security-gate job in ci.yml (together
   with test_check_security_exceptions.py, which was previously not wired).
3. **Minor.** "no revision provenance" message now has a fix hint (retained artifact codeql-security-sarif,
   runs[].versionControlProvenance[].revisionId) + test; ci.yml header comment now lists six live contexts (and the
   closing line no longer says "four jobs"); supply-chain checker parses flow-style `permissions: {…}` (job and workflow),
   flags pull_request workflows lacking top-level permissions, and `excludes_pull_request` now splits on `||` (every
   alternative must exclude PR; any `== 'pull_request'` or unanalysed parentheses stay exposed) - 7 new tests; evidence doc
   records that the 3 ledger entries expire 2026-11-05 and need online digest resolution.

Verification (exit 0): test_check_security_gate 22 OK; test_check_security_findings 12 OK; test_check_workflow_supply_chain
33 OK; test_check_security_exceptions 6 OK; test_check_security_ci_contract 16 OK; test_check_quality_ci_contract 27 OK;
--self-test-failure/--self-test-unrequired PASS; check-workflow-supply-chain PASS; check-security-exceptions PASS (9 records);
actionlint ci.yml clean; doctor.sh OK; run-offline-lane.sh PASS components=7 checks=237; check-live-state PASS.
Not touched: H3 files (sandbox*, checks/, run-offline-lane.sh, doctor.sh). Still NOT RUN: real CI execution (see above).
