# Verification Manifest Convention

## Purpose

Agents frequently claim completion without running the checks that would surface regressions. The `harness_verify` tool solves this by creating a permanent, searchable record of every verification command outcome — including failures and skips. A missing record is ambiguous; an explicit skip record with a reason is not. This makes the verification trail auditable across sessions and agents.

## harness_verify fields

| Field | Required | Description |
|---|---|---|
| command | yes | Exact command run |
| exit_code | yes | 0=success, non-zero=failure |
| output_summary | yes | ≤500 chars — counts, key errors, timing |
| passed | no | Explicit override; derived from exit_code == 0 if absent |
| evidence_path | no | Path to full log for audit |
| evidence_hash | no | SHA256 of full log |
| skipped_reason | no | Why the check was skipped (negative evidence) |
| issue_numbers | no | Related GitHub issues |
| memory_ids | no | Related engram memory IDs |
| workspace | no | Defaults to ctx workspace |
| importance | no | 0.0–1.0, default 0.8 |

## Standard checks to record

- `cargo test --lib` — unit tests
- `cargo test` — all tests
- `cargo check` — type check
- `just ci` / `make ci` — full CI gate
- `cargo bench --bench <name>` — benchmark regressions

## Negative evidence

Skipped checks MUST be recorded with a reason, not omitted. A missing record is ambiguous — a skip record is explicit.

## Relationship to harness_status and harness_handoff

- `harness_status` surfaces the most recent `verification_result` record as `last_verification`
- `harness_handoff` accepts `verification_evidence` (string) — populate it from the `output_summary` of the most recent passing verify
- `completion_claimed` in `harness_handoff` is `false` unless `verification_evidence` is non-empty

## Workflow

1. Run the check command
2. Call `harness_verify` immediately (don't defer)
3. If failed: fix, re-run, call `harness_verify` again (do not overwrite — new record)
4. Before handoff: ensure a passing `harness_verify` exists for the CI gate
