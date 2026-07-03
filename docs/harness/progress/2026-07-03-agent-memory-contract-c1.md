# Progress Log — Agent Memory Contract C1

**Sprint**: Agent Memory Contract C1
**Task**: c1-agent-writeback-candidates — pending agent writebacks on dream candidates
**Date started**: 2026-07-03
**Owner**: Ronaldo + Codex

---

## 2026-07-03 — C1.1 pending writeback candidates

### Contexto

PR #114 landed C1.0 as the read-only `memory_agent_contract` surface. The next
approved slice is pending agent writebacks using existing dream candidates, not a
new writeback table.

### Ações realizadas

1. Added migration v45 to allow `agent_writeback` in the existing
   `dream_candidates.kind` CHECK.
2. Added `agent_writeback` to storage validation in `dream_snapshots`.
3. Added `memory_agent_writeback` as an Advanced-tier, `dream-phase`-gated MCP
   tool.
4. Kept dry-run as the default and required `confirm=true` when creating a real
   pending candidate.
5. Required evidence through either `source_memory_ids` or structured
   `evidence`.
6. Wrote only pending dream candidates and candidate sources before review;
   canonical memories still require `dream_candidate_review` plus
   `dream_candidate_apply`.
7. Updated `memory_agent_contract` to v1, generated MCP reference docs, AI guide,
   implementation plan, review canvas, and lessons catalog.

### Evidência

- `rtk cargo test --lib storage::migrations::tests::test_dream_candidates_allow_agent_writeback_kind` — PASS.
- `rtk cargo test --features dream-phase --test mcp_protocol_tests memory_agent_writeback_tool_is_advanced_dry_run_mutating_surface` — PASS.
- `rtk cargo test --features dream-phase --test dream_integration test_mcp_memory_agent_writeback_requires_review_before_canonical_apply` — PASS.
- `rtk cargo test --test mcp_protocol_tests memory_agent_contract_dispatches_governance_contract` — PASS.
- `rtk cargo fmt --all -- --check` — PASS.
- `rtk git diff --check` — PASS.
- `rtk ./scripts/generate-mcp-reference.sh --check` — PASS.
- `rtk cargo check --workspace --all-targets --locked` — PASS.
- `rtk cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS.
- `rtk cargo test --workspace --all-targets --locked` — PASS, 1249 tests.
- `rtk bash docs/harness/bin/sensors.sh` — PASS, full lane, timestamp
  `2026-07-03T10:18:59Z`, duration 89s.
- MCP stdio smoke with `--features dream-phase` and isolated `ENGRAM_DB_PATH`
  returned `status=dry_run`, `kind=agent_writeback`, and
  `canonical_memory_mutated=false` for default dry-run.
- MCP stdio smoke with `dry_run=false, confirm=true` created pending candidate
  `smoke-agent-writeback-candidate`; a follow-up `dream_candidate_get` returned
  the candidate and source evidence.

### Próxima verificação antes de merge

- Post-review gate on the full diff after the review-fix commit.

---

## 2026-07-03 — Post-review hardening pass

### Contexto

External review found correctness and governance gaps in the first C1.1 draft:
`agent_writeback` applied as generic `note`, dry-run/live response shapes
diverged, candidate id collisions leaked raw SQLite details, synthetic jobs
stayed pending, caller-provided `job_id` values lacked provenance/status
guards, contract validation rules were incomplete, and v45 rebuild coverage did
not preserve existing rows.

### Ações realizadas

1. Mapped `agent_writeback` candidates to canonical `learning` memories on
   `dream_candidate_apply`.
2. Made dry-run and live `memory_agent_writeback` responses share the same
   `candidate.candidate` plus `candidate.sources` wrapper.
3. Added candidate-id preflight and SQLite constraint mapping so duplicate ids
   return domain conflicts instead of raw SQL.
4. Validated reused jobs by workspace, model profile, input-summary provenance,
   candidate kind, and pending status.
5. Completed synthetic agent writeback jobs after candidate/source insert so
   they do not remain permanently pending.
6. Rejected reserved governance metadata keys case-insensitively.
7. Reused the dream candidate preview helper instead of duplicating preview
   truncation logic.
8. Replaced the ambiguous contract `schema_migration_required` flag with a
   structured v45 migration object and documented validation rules.
9. Added a v44-with-data migration regression test for v45 table rebuild.

### Evidência

- `rtk cargo test --features dream-phase --test dream_integration test_mcp_memory_agent_writeback_requires_review_before_canonical_apply --locked` — PASS.
- `rtk cargo test --features dream-phase --test dream_integration test_mcp_memory_agent_writeback_rejects_reuse_and_spoofing --locked` — PASS.
- `rtk cargo test --test mcp_protocol_tests memory_agent_contract_dispatches_governance_contract --locked` — PASS.
- `rtk cargo test --lib storage::migrations::tests::test_v45_preserves_existing_dream_candidate_data --locked` — PASS.
- `rtk cargo check --workspace --all-targets --locked` — PASS.
- `rtk cargo test --features dream-phase --test dream_integration --locked` — PASS, 8 tests.
- `rtk cargo test --features dream-phase --test mcp_protocol_tests memory_agent --locked` — PASS, 3 tests.
- `rtk cargo test --lib storage::migrations::tests --locked` — PASS, 19 tests.
- `rtk ./scripts/generate-mcp-reference.sh --check` — PASS.
- `rtk cargo fmt --all -- --check` — PASS.
- `rtk git diff --check` — PASS.
- `rtk cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS.
- `rtk cargo test --workspace --all-targets --locked` — PASS, 1250 tests.
- MCP stdio smoke with `--features dream-phase` and isolated `ENGRAM_DB_PATH`
  — PASS for default dry-run, confirmed pending candidate creation, and
  `dream_candidate_get`.
- `rtk bash docs/harness/bin/sensors.sh` — PASS, full lane, timestamp
  `2026-07-03T11:44:01Z`, duration 35s.
- `rtk bash docs/harness/bin/review-gate.sh post c1-agent-writeback-candidates-v2`
  — generated
  `docs/harness/reviews/2026-07-03-c1-agent-writeback-candidates-v2-post.md.raw`;
  prompt grep confirmed the fix diff includes `complete_agent_writeback_job`,
  `test_mcp_memory_agent_writeback_rejects_reuse_and_spoofing`, and the new
  structured `schema_migration` contract fields.
