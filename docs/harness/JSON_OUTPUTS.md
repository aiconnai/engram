# Harness JSON Outputs

This document defines the stable JSON contract for harness scripts that expose a
machine-readable mode. Human output remains the default for every script.

## Global Rules

- JSON output is opt-in. Scripts must keep their existing human output unless a
  JSON flag is explicitly provided.
- JSON mode writes exactly one JSON object to stdout.
- Stderr is reserved for fatal setup or usage errors that prevent JSON
  generation.
- Exit codes must match the equivalent human mode.
- Field names use stable `snake_case`.
- Timestamps use UTC RFC 3339 format.
- Paths are repo-relative unless an absolute path is required to diagnose setup.
- Schema-breaking changes require a new `schema_version`.
- JSON output must never include tokens, cookies, auth headers, private keys,
  raw `.env` contents, complete environment dumps, or unredacted secrets.

## Status Vocabulary

| Status | Exit code | Meaning |
|--------|-----------|---------|
| `pass` | `0` | All required checks passed. |
| `warn` | `0` | Required checks passed, but non-blocking warnings exist. |
| `fail` | non-zero | One or more blocking checks failed. |
| `usage_error` | `2` | Invalid arguments or setup prevented normal validation. |

Existing scripts may document a more specific non-zero exit code, but the JSON
`status` value must still use this vocabulary.

## Common Envelope

```json
{
  "schema_version": "harness-json-v1",
  "tool": "doctor",
  "mode": "json",
  "status": "pass",
  "exit_code": 0,
  "timestamp": "2026-06-09T14:00:00Z",
  "summary": "harness doctor passed",
  "warnings": [],
  "failures": [],
  "checks": [],
  "artifacts": []
}
```

Required common fields:

- `schema_version`: stable schema identifier.
- `tool`: harness tool name without path, for example `doctor`.
- `mode`: `json`.
- `status`: one of the status vocabulary values.
- `exit_code`: integer exit code the command will return.
- `timestamp`: UTC RFC 3339 timestamp generated at command runtime.
- `summary`: short human-readable summary.
- `warnings`: array of warning objects or strings.
- `failures`: array of failure objects or strings.
- `checks`: array of check result objects.
- `artifacts`: array of repo-relative artifact paths or structured artifact
  objects.

Warning and failure entries may be strings in `harness-json-v1`, but structured
objects are preferred:

```json
{
  "id": "active_plan:missing",
  "message": "progress.md does not point at an active plan",
  "path": "docs/harness/progress.md"
}
```

## Check Objects

Each item in `checks` should use this shape:

```json
{
  "id": "required_file:docs/harness/SPEC.md",
  "status": "pass",
  "message": "required file exists",
  "path": "docs/harness/SPEC.md"
}
```

Check statuses:

- `pass`: the check succeeded.
- `warn`: the check found a non-blocking issue.
- `fail`: the check found a blocking issue.
- `skipped`: the check did not run and includes a reason in `message`.

Stable check id families:

- `required_file`
- `required_exec`
- `cross_reference`
- `active_plan`
- `review_verdict`
- `sensors_last`
- `bootstrap_contract`
- `exclusion_record`

## `doctor.sh --json`

`doctor.sh --json` must validate the same read-only harness invariants as the
default human mode and return one JSON object using this contract.

Required fields:

- `schema_version`
- `tool`
- `mode`
- `status`
- `exit_code`
- `repo_root`
- `timestamp`
- `active_plan`
- `active_task`
- `summary`
- `failures`
- `warnings`
- `checks`

Compatibility requirements:

- `bash docs/harness/bin/doctor.sh` remains human-readable by default.
- `bash docs/harness/bin/doctor.sh --json` exits `0` on `pass` or `warn`.
- `bash docs/harness/bin/doctor.sh --json` exits `1` when validation fails.
- `bash docs/harness/bin/doctor.sh --json` exits `2` for usage errors or setup
  errors that prevent validation.
- Successful JSON output must be parseable with `jq .`.
- JSON mode must remain read-only and must not create or update harness files.

## Non-Goals

- JSON output does not replace human diagnostics.
- JSON output does not embed raw logs, review bodies, environment dumps, or
  secrets.
- JSON output does not require network access.
- JSON output does not mutate repository state.
