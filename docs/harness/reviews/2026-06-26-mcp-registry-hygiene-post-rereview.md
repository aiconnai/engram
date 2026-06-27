PASS

No actionable findings in the scoped changes.

Verified:
- `bootstrap.sh` reports `MCP tools (source): 238 active / 278 total`.
- Bootstrap count path uses `python3 -c`; no heredoc/temp-file pattern found.
- `src/mcp/tools/discovery.rs` is deleted and has no live code references.
- Static registry probe shows `278` total names, `238` active, `discover_tools=1`, `unique=True`.
- `./scripts/generate-mcp-reference.sh --check` passed.
- `git diff --check` passed for scoped files.

Could not rerun the focused Cargo tests in this read-only sandbox because Cargo cannot open `target/debug/.cargo-build-lock` (`Operation not permitted`). The progress docs correctly record the narrower `mcp` lane, not a full gate claim, for this hygiene follow-up.

REVIEW_VERDICT: PASS scoped MCP registry hygiene changes satisfy the requested checks.
