# ENGRA-100 Handoff: Dream Snapshot Eval Suite And Docs

## Scope

Added contract-first eval scaffolding and user-facing documentation for the
Dream Snapshot Review Pipeline without depending on unmerged storage, generator,
freshness, harness, or MCP APIs.

## Files Changed

- `docs/DREAM_SNAPSHOT_EVALS.md`
- `tests/dream_eval_tests.rs`
- `docs/AI_GUIDE.md`
- `docs/USING_ENGRAM_IN_A_REPO.md`
- `README.md`

## Verification

- `bash docs/harness/bin/bootstrap.sh` — PASS
- `bash docs/harness/bin/doctor.sh` — PASS
- `cargo fmt --all -- --check` — PASS
- `cargo test --test dream_eval_tests -- --nocapture` — PASS
- `git diff --check` — PASS

## Integration Notes

- The current Rust tests assert RFC/runbook text contracts only. They are
  intentionally compile-safe before `ENGRA-95` through `ENGRA-99` merge.
- After storage/generator/MCP land, extend `tests/dream_eval_tests.rs` with
  behavioral fixtures that call the real generator and `dream_eval_run`.
- Do not regenerate `docs/MCP_TOOLS.md` from this lane. The MCP owner or final
  integration branch should regenerate it after live `dream_*` tools exist.
- Docs label Dream Snapshot Review as planned and point readers to the generated
  MCP reference as the source of truth for live tools.
