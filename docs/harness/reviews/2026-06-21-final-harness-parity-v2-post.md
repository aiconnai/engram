I now have enough information to complete this review. Let me analyze the key findings:

**Key observations:**

1. **`sensors.sh` double-invocation in `full` mode is benign**: The `case "$MODE"` block for `full` falls through (empty `;;`) to the code below, which runs `run_pr_title_policy` once at line 525. No double-invocation in `full` mode. Other modes (`quick`, `docs`, `mcp`, `baseline`) call it inside the if-chain and then `exit`, so they never reach line 525. Clean.

2. **`pr-title-policy.sh` SOURCE_COUNT with `PR_TITLE` env var**: If `PR_TITLE` is set in the environment AND `--title` is passed explicitly, SOURCE_COUNT becomes 2, causing exit 2 (usage error). This is correct by design — it refuses ambiguous sources. Sensors/doctor always call with `--title` explicitly, so `PR_TITLE` env var in CI could cause false failures. This is a minor but real fragility.

3. **`run_expected_exit` in `sensors.sh`**: The function temporarily uses `set +e`, captures exit code, and returns 1 on mismatch. The `|| return 1` in `run_pr_title_policy` propagates failure correctly. Sound.

4. **Security**: The `subtle::ConstantTimeEq` usage is correct. No new autonomous execution paths, no credential mounts, no security boundary weakening.

5. **Scope**: The diff is strictly harness/gate alignment. No hidden scope creep into storage schema, MCP surface, or business logic beyond the already-reviewed hook cleanup. Storage changes (fake-success removal, savepoint validation) and hook cleanup are clearly scoped and documented.

6. **Canvas coverage**: All non-trivial changes have matching canvas files with approaches, edge cases, and breakage-risk tables.

7. **`sensors-last` mode downgrade**: `.sensors-last` switches from `mode=docs` to `mode=quick`. This is an accurate reflection of what was run for this commit (no MCP/rustdoc check) — not a weakening of the gate itself, just an honest record.

8. **`progress.md` records multiple sessions verified with `sensors.sh quick`** but the last "full" run timestamp in `.sensors-last` is now overwritten by `quick`. The invariant #17 (local CI parity) requires `just ci`, and the sensors-last for this commit only records `quick`. The progress.md logs `sensors.sh quick — PASS` repeatedly without a `full` run confirmation for this batch of changes. This is a mild gap — the diff includes Rust changes (hooks, storage, wasm) that warrant a full `just ci` pass per invariant #17.

---

FAIL Missing `full` sensors run for Rust code changes; `sensors-last` records only `quick` mode

- **[HIGH]** `docs/harness/.sensors-last` records `mode=quick` for a commit that includes Rust production code changes (`src/auth/tokens.rs`, `src/bin/server.rs`, `src/hooks/post_tool_use.rs`, `src/storage/sqlite_backend.rs`, `src/storage/turso_backend.rs`, `engram-wasm/src/tfidf.rs`, `engram-wasm/src/graph.rs`). Invariant #17 ("Paridade CI local é sagrada") requires `just ci` — which runs `cargo clippy -D warnings`, full test suite with `CI_FEATURES`, and MCP reference check — before domain-behavior commits. `progress.md` logs only `sensors.sh quick` (fmt + cargo check) for these changes. The storage-extension-semantics canvas even records a clippy FAIL on `engram-wasm` that was scoped away; the full gate was never confirmed to pass.

- **[MED]** `pr-title-policy.sh` SOURCE_COUNT logic silently rejects calls made with `--title` when the `PR_TITLE` environment variable is also set (counter incremented to 2, exits with code 2 — usage error). CI environments that export `PR_TITLE` (e.g., GitHub Actions PR event) would get a spurious usage-error failure rather than the expected title validation. No test in `doctor.sh` or `sensors.sh` covers this interaction; no documentation warns callers to unset `PR_TITLE`.

- **[LOW]** The canvas `2026-06-20-storage-extension-semantics.md` (Verification section) explicitly records `rtk cargo clippy --workspace --all-targets --all-features -- -D warnings` as FAIL on `engram-wasm` warnings, and notes these are "outside this task scope." The diff includes `engram-wasm/src/graph.rs` and `engram-wasm/src/tfidf.rs` changes (clippy fixes). These are bundled into the same PR without a re-run of workspace-wide clippy confirming the wasm warnings were resolved before submission.

REVIEW_VERDICT: FAIL Missing full CI gate pass (sensors quick only) for commits that include Rust production-code changes; PR_TITLE env-var interaction with --title causes spurious exit-2 in CI contexts
