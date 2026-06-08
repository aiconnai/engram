# Dream Snapshot Evaluation Fixtures

This runbook defines the deterministic evaluation scaffold for
`ENGRA-100` and RFC 0007. It is intentionally contract-first: the fixtures
describe the behavior the storage, generator, freshness, MCP, and harness lanes
must satisfy after integration, without depending on their unmerged APIs.

Dream snapshot evals must stay local, deterministic, and non-networked. They
measure whether generated candidates are useful review proposals, not whether a
model can invent plausible memory.

## Contract Under Test

All eval fixtures must preserve these invariants:

- Dream output is candidate memory until reviewed and explicitly applied.
- Candidate generation never mutates canonical memories.
- Applying a candidate requires `confirm=true`.
- Every non-scratch candidate has provenance.
- Raw logs, raw transcripts, terminal dumps, secrets, and environment dumps are
  rejected by default.
- Freshness uses RFC3339 UTC and malformed temporal metadata does not panic.
- Candidate ids, reason codes, and metric output are stable across repeated
  local runs on the same fixture.

## Fixture Catalog

Each fixture is a small JSON or Rust in-memory corpus with explicit expected
candidates. Fixtures should avoid wall-clock dependence by pinning `now` as an
RFC3339 UTC timestamp.

| Fixture | Purpose | Required Candidate Signal | Must Not Happen |
|---|---|---|---|
| `carry_forward_context` | Preserve useful project state from decisions, summaries, and harness handoffs. | `summary` or `project_state` candidate with source ids and durable reason code. | No canonical memory write before apply. |
| `preferences_constraints` | Surface durable preferences and hard constraints from repeated evidence. | `preference` and `constraint` candidates with confidence and policy explanation when source memories exist. | No promotion of one-off or contradicted preferences. |
| `freshness_temporal` | Detect stale, future-due, expired, and confirmed temporal facts. | `stale_fact` or `temporal_update` candidate with `freshness_state`. | No panic on malformed timestamps or missing source records. |
| `provenance_correctness` | Verify every candidate links back to the evidence that caused it. | Candidate sources include memory, context, artifact, issue, commit, or harness references as available. | No high-trust candidate without source evidence. |
| `unsafe_raw_log_rejection` | Prevent raw terminal dumps, secrets, and environment output from becoming durable memory. | `ignore` or rejected low-trust candidate with safety reason code. | No raw payload copied into proposed canonical content. |
| `no_canonical_mutation_before_apply` | Guard the review boundary. | Candidate rows may be created; canonical memory count/hash remains unchanged until confirmed apply. | No create/update/promote/expire side effect during generation or review. |

## Expected Metrics

`dream_eval_run` should report deterministic metrics with enough detail for CI
and review:

- `fixtures_run`
- `fixtures_passed`
- `candidate_precision`
- `required_candidate_recall`
- `provenance_coverage`
- `unsafe_payload_rejection_rate`
- `canonical_mutation_violations`
- `freshness_parse_failures`

Minimum v1 acceptance:

- `fixtures_run` equals the number of enabled fixtures.
- `fixtures_passed` equals `fixtures_run`.
- `required_candidate_recall` is `1.0` for deterministic fixtures.
- `provenance_coverage` is `1.0` for non-scratch candidates.
- `unsafe_payload_rejection_rate` is `1.0`.
- `canonical_mutation_violations` is `0`.
- `freshness_parse_failures` is `0`.

Precision can start with a documented floor once real generator behavior lands,
but every unexpected candidate must include a stable reason code so regressions
are reviewable.

## Acceptance Checklist

Use this checklist when integrating the implementation lanes:

- [ ] Eval fixtures are committed as small deterministic inputs, not generated
      artifacts.
- [ ] Focused eval command passes:
      `cargo test dream_eval --all-features -- --nocapture`.
- [ ] `dream_eval_run` returns parseable metrics and does not require network,
      paid model access, credentials, or wall-clock-dependent output.
- [ ] Evals cover carry-forward context, preferences/constraints, freshness,
      provenance, raw-log rejection, and no mutation before apply.
- [ ] Fixture expectations assert candidate kind, proposed action, review
      state, freshness state, reason codes, and source references.
- [ ] The MCP reference is regenerated only by the MCP/integration owner after
      the tool is implemented.
- [ ] User docs say dream output is a proposal until reviewed/applied.
- [ ] Default CI remains green.

## Integration Notes

During Wave 1, `tests/dream_eval_tests.rs` may assert RFC and runbook text
contracts only. After `ENGRA-95` through `ENGRA-99` merge, replace or extend
those docs-contract tests with behavioral fixtures that call the real generator
and MCP eval tool.

Do not modify production behavior from the eval lane to make an eval pass.
Failing behavioral evals should be handed back to the lane that owns the
storage, generator, freshness, MCP, or harness integration defect.
