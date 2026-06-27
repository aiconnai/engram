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

Every harness JSON mode must return the common envelope below unless the tool
documents a narrower read-only status command. Tool-specific fields may be added,
but the common fields must keep their meaning across scripts.

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

Tool-specific fields should be shallow, stable, and non-secret. Prefer:

- scalar identifiers such as `repo_root`, `active_plan`, `active_task`, `mode`,
  `sensor`, `known_issue`, or `review_file`;
- counts such as `failure_count`, `warning_count`, or `check_count`;
- short status summaries that use the common status vocabulary or a
  documented tool-specific vocabulary.

Avoid:

- raw command logs;
- full reviewer output;
- environment variable dumps;
- request or response bodies that could contain credentials;
- provider headers, cookies, bearer tokens, or API keys;
- absolute local paths unless the command cannot diagnose setup without them.

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
- `gate_status`
- `artifact`
- `json_contract`

## JSON-Only Output vs Artifacts

Use JSON-only stdout when the complete machine-readable result is small and safe
to keep in process output. Examples:

- `doctor.sh --json`
- `sensors.sh status --json`
- read-only status or validation commands

Use `artifacts` when output is large, reviewer-authored, log-like, or useful as
durable evidence. In that case, stdout still contains one JSON object and the
large material is written elsewhere. The `artifacts` array points to those
repo-relative paths:

```json
{
  "artifacts": [
    {
      "path": "docs/harness/reviews/2026-06-09-example-post.md",
      "kind": "review",
      "format": "markdown"
    }
  ]
}
```

Artifact rules:

- Artifact paths must be repo-relative.
- Artifact objects should include `path`, `kind`, and `format`.
- Artifact files must follow the same secret rules as JSON stdout.
- JSON mode must not hide failures inside artifacts; `status`, `failures`, and
  `checks` still summarize blocking results.
- A command that is read-only in human mode must remain read-only in JSON mode.

## `.sensors-last` Relationship

`docs/harness/.sensors-last` is the current lightweight parseable state file for
the full harness gate. `docs/harness/.sensors-log` is the append-only historical
measurement log. `.sensors-last` remains supported for compatibility, but it is
not the general JSON contract.

Migration path:

1. Keep writing `.sensors-last` exactly as existing users expect.
2. Append each completed run to `.sensors-log` using JSON Lines with
   `schema_version="sensors-log-v1"`.
3. Add a JSON status surface that translates `.sensors-last` into the common
   envelope without running the full gate, for example `sensors.sh status --json`.
4. For full gate runs, either support `sensors.sh --json` directly or emit a JSON
   envelope that points to `.sensors-last` through `artifacts`.
5. Do not require automation to scrape human `sensors.sh` output once a JSON
   status surface exists.

Suggested `sensors` fields:

- `tool`: `sensors`
- `mode`: selected lane, for example `full`, `quick`, `docs`, `mcp`,
  `baseline`, or `status`
- `ci_status`: current CI/gate status from `.sensors-last`
- `doctor_status`: result of the doctor check when available
- `known_issue`: repo-relative known-issue path when an exclusion is active
- `excluded_sensor`: excluded sensor name when an exclusion is active
- `artifacts`: include `docs/harness/.sensors-last` when it is the source of
  truth for the reported state

Suggested `.sensors-log` fields:

- `schema_version`: `sensors-log-v1`
- `timestamp`: UTC RFC3339 timestamp at the end of the run
- `tool`: `sensors`
- `mode`: selected lane
- `status`: `pass`, `pass_with_exclusion`, or `fail`
- `duration_sec`: non-negative integer duration
- `ci_status` and `doctor_status`: status of the two main gate layers
- `ci_command`: short command label, not raw output
- `ci_steps`: granular CI step status map (ex.: `fmt`, `clippy`, `test_lib`,
  `test_integration`, `test_integration_watch`, `wasm_target`,
  `wasm_all_targets`, `wasm_wasm_target`, `doc`, `ref_check`)
- `exclusion`: `null` or a short `{sensor, known_issue, reason}` object
- `artifacts`: repo-relative artifact pointers

Rotation is part of the log contract. `sensors.sh` rotates `.sensors-log` before
append when it reaches `SENSORS_LOG_MAX_BYTES` (default `1048576`) and keeps
`SENSORS_LOG_ROTATIONS` generations (default `5`).

## `harness-stats.sh --json`

`harness-stats.sh --json` is a read-only analytics command over
`docs/harness/.sensors-log` and must return one JSON object with the common
envelope plus a `metrics` object.

Required additional field:

- `metrics`: object with at least:
  - `window`
  - `total_entries`
  - `window_runs`
  - `status_counts`
  - `pass_like_rate`
  - `ci_status_counts`
  - `doctor_status_counts`
  - `mode_stats`
  - `flaky_modes`
  - `last_entry`

Suggested `checks` for analytics:

- `sensors_log:exists`
- `sensors_log:parse`

## Compatibility Rules

- Human output remains the default.
- JSON flags are opt-in and must not weaken existing gates.
- Exit codes must preserve the existing human-mode meaning.
- `--help` may remain human-readable unless a tool documents a JSON help mode.
- JSON status commands may be read-only snapshots and do not need to run the full
  gate.
- Full gate JSON mode must still run the same checks as the human full gate
  unless the command name clearly says it is a status snapshot.

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
- `artifacts`

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
