# Task Q3 report: effective nightly (fuzz, mutation, Miri)

Status: DONE_WITH_CONCERNS. Commit `4e8ba9a ci(infra): make nightly fuzz, mutation and Miri lanes effective`.

## What was implemented
- `fuzz/` own cargo workspace (`[workspace] members=["."]`, own `fuzz/Cargo.lock` seeded from the root lock; root
  `Cargo.toml`/`Cargo.lock` untouched, `cargo metadata` at root still OK, root lock has no libfuzzer-sys).
  `engram-core` consumed with `default-features = false`.
- Targets (real public APIs, verified `pub`):
  - `entity_extraction`: `intelligence::entity_extraction::extract_entities(content, &ExtractionConfig, None)`
    (bounds, sort order, dedup, char-boundary positions, confidence floor).
  - `workspace_normalization`: there is NO public `normalize_workspace` (only a private one in
    `mcp/handlers/agent_writeback_plan.rs`). The target covers the public workspace/scope identifier surface:
    `storage::scoping::MemoryScope::{parse,new,parent,ancestors,contains}`,
    `auth::TransportPrincipal::allows_workspace` (anonymous/namespaced/unscoped),
    `mcp::permission::extract_requested_scopes`, `mcp::workspace_guard::memory_id_arguments`.
  - `text_util_boundaries` (third, cheap): `text_util::{floor,ceil}_char_boundary/truncate_bytes/suffix_bytes`.
- Seeds (`fuzz/seeds/<target>/`, 115 files) from C3 inputs (proptest regression strings, ZWJ/flags/combining/bidi/
  case-fold chars, split-2-byte-char at 64/200, hex `+f`/non-ASCII/odd cases) plus entity/workspace cases.
- `scripts/run-fuzz-smoke.sh` + `scripts/optional_lane_report.py` (shared reporter): declared inventory
  (`fuzz/targets.inventory`) must be non-empty and equal `cargo fuzz list`, compiled set and executed set;
  60 s/target (`-max_total_time`, `-rss_limit_mb=2048`, `-timeout=25`, hard wall cap); statuses
  pass/fail/not-run/unsupported; per-target executed/corpus/seed counts; reproducers copied to
  `<out>/reproducers/<target>/` and listed; exit code mirrors status (0/1/2/3); no `|| true`.
  Also fails: missing seeds, exit 0 with no executed-units evidence, build failure.
- `scripts/run-miri-smoke.sh` + `scripts/miri-inventory.txt`: nominal SQLite-free filters, per-filter test count
  must be > 0; empty inventory / zero-match filter / failing test -> fail; missing Miri -> not-run.
  SQLite/FFI limitation and "not full UB coverage" stated in the inventory and workflow comments.
- `.github/workflows/nightly.yml`: crons `0 3 * * 1-6` (daily) and `0 3 * * 0` (weekly); new `plan` job maps
  `EVENT_NAME:github.event.schedule` -> lane (daily/weekly/manual; unmapped -> exit 1); `mutants` gated by
  `needs.plan.outputs.run_mutants`; fuzz and Miri jobs call the runners; `cargo install cargo-fuzz --version 0.13.2
  --locked`, `cargo-mutants --version 27.1.0 --locked`; no `continue-on-error`/`|| true` in fuzz/mutants/miri;
  report artifacts uploaded with `if: always()`. `ci.yml` / required gates untouched.
- `scripts/test_optional_lane_contracts.py` (30 tests).

## TDD / RED evidence
Tests were written against a fake `cargo` (runner behaviour) and the workflow text. RED check of the workflow
contract against the ORIGINAL `nightly.yml` (`git show HEAD~:.github/workflows/nightly.yml`):
`Ran 9 tests ... FAILED (failures=6, errors=3)`: no `plan` job (KeyError), cron list `['0 3 * * *']` (weekly
unreachable), `continue-on-error` + `|| true` in fuzz job, unpinned `cargo install cargo-mutants`, mutants `if`
compares raw `github.event.schedule`. (Runner scripts did not exist before, so their RED is "file missing".)
GREEN: `python3 -m unittest scripts/test_optional_lane_contracts.py` -> `Ran 30 tests ... OK` (exit 0).
Fixtures covered: empty inventory, list failure, empty list, inventory drift, crash (reproducer preserved, other
targets still run), infra unavailable (not-run, exit 2), unsupported OS (exit 3), build failure, no execution
evidence, missing seeds; Miri empty inventory / zero-match filter / failing test / missing miri; daily/weekly/manual
and unmapped-trigger mapping by executing the workflow's own `run:` block; no masking.

## Real local runs (honest results)
- Fuzz smoke, 20 s/target (`FUZZ_SMOKE_MAX_TOTAL_TIME=20 bash scripts/run-fuzz-smoke.sh`): PASS, exit 0.
- Fuzz smoke, full CI parity 60 s/target: PASS, exit 0 (executed units: entity_extraction 366810,
  workspace_normalization 449316, text_util_boundaries 483430; corpus 2262/2580/89; seeds 24/53/38; no crashes found).
- Miri (`bash scripts/run-miri-smoke.sh`, nightly + miri present): PASS, 5 filters, 13 tests, exit 0.
- cargo-mutants: NOT RUN locally (not installed; CI installs pinned 27.1.0 `--locked`). The pinned version and the
  weekly run are untested end to end.
- The first local attempt (20 s) honestly FAILED two targets with exit 124: the hard wall cap (then 140 s) was
  exceeded because `cargo fuzz run` re-compiled engram-core (a cargo fingerprint invalidation caused by
  concurrent file churn in the shared worktree: engram-core has no `rerun-if-changed`, so every file touched
  in the package dir invalidates it). Fixed by raising the default grace to 600 s; the failure was reported, not hidden.
- Workflow YAML parsed with PyYAML (jobs: plan, fuzz, mutants, property-tests-extended, miri, outdated, bench-full).
  GitHub-hosted execution of the workflow itself: NOT RUN.

## Files changed
.github/workflows/nightly.yml; fuzz/{Cargo.toml,Cargo.lock,.gitignore,targets.inventory,fuzz_targets/*.rs,seeds/**};
scripts/{run-fuzz-smoke.sh,test_optional_lane_contracts.py}; progress entry in
docs/harness/progress/2026-10-05-improvement-lane-r.md.
Unlisted but added (deviation, reason): scripts/run-miri-smoke.sh + scripts/miri-inventory.txt (testable Miri
contract; inline YAML could not be fixture-tested), scripts/optional_lane_report.py (shared report/status logic).

## Concerns
1. `mutants` keeps the original whole-lib scope (`cargo mutants --timeout 300 --jobs 2 -- --lib`); runtime is
   unmeasured and may exceed the 350-min job timeout, and any missed mutant now makes the job red (no masking).
   Owner may want `--file` scoping or an explicit disable-with-owner (rollback path in the brief).
2. `workspace_normalization` is named per the brief but no public normalizer exists; see above. Product decision if
   a public normalizer should be extracted and fuzzed.
3. `outdated` job still has `continue-on-error: true` (out of Q3 scope, informational).
4. Fuzz corpus under `fuzz/corpus` is ephemeral (ignored); only `fuzz/seeds` is versioned. No cache of discovered
   corpus between nights.
5. cargo-fuzz run uses ASAN on nightly; on ubuntu-latest, `cargo install cargo-fuzz` + cold ASAN build time is
   unmeasured (job timeout 60 min).

## Fix report, review round 1 (commit 730ede4 `ci(infra): scope mutants lane and harden fuzz/Miri nightly caps`)

1. [Important] mutants scoped. Added `.cargo/mutants.toml` with `examine_globs` = text_util, storage/scoping,
   intelligence/entity_extraction, auth/transport_principal (~1.5k lines, ~55 fns, est. ~130 mutants, 1.5-2 h at
   `--jobs 2`; UNMEASURED, cargo-mutants not installed). Owner (Ronaldo), rationale and "How to widen" are in the
   file header. Job `timeout-minutes` 350 -> 240; still fail-closed (no `continue-on-error`, nonzero = red).
   New contract test `test_mutants_job_is_scoped_and_fail_closed`: no continue-on-error, timeout <= 300, config has
   1..8 explicit existing files (no wildcards), total < 3000 lines, required modules present, no hiding keys
   (`exclude_globs`, `skip_calls`), owner/widen documented. RED: with `.cargo/mutants.toml` moved away the test
   errors (`FAILED (errors=1)`); GREEN with it present.
2. [Minor]
   - Fuzz wall-cap grace 600 -> 120 s (workflow sets it explicitly). Per-target `elapsed_secs` is now in every row
     (fuzz and Miri), JSON and markdown. A target with exit 0 but elapsed > budget+grace FAILS (tested with a fake
     cargo that sleeps: `test_elapsed_over_budget_fails_even_with_exit_zero`); the hard kill is 30 s later.
   - Dated nightly pin: `NIGHTLY_TOOLCHAIN: nightly-2026-04-21` (local toolchain: rustc 1.97.0-nightly 66da6cae1
     2026-04-20, manifest date 2026-04-21) used by fuzz and Miri via `toolchain:` and new
     `FUZZ_SMOKE_TOOLCHAIN` / `MIRI_SMOKE_TOOLCHAIN` (runners call `cargo +<toolchain>`; recorded in report
     `settings.toolchain`; tested).
   - `cargo audit --file fuzz/Cargo.lock` step in the fuzz job (cargo-audit 0.22.0 pinned `--locked`; uses
     `.cargo/audit.toml`). Local: `cargo audit --file fuzz/Cargo.lock --no-fetch --stale` rc=0, 1 allowed warning
     (RUSTSEC-2026-0221 event-listener, transitive via engram-core; same lock content as root). Note in fuzz/Cargo.toml.
   - `plan` job now checks out and runs `python3 -m unittest scripts/test_optional_lane_contracts.py`.
3. Verification: `python3 -m unittest scripts/test_optional_lane_contracts.py` -> `Ran 37 tests ... OK`.
   Real fuzz smoke 3x20 s: PASS exit 0 (elapsed 22/23/22 s; executed 100148/118945/135193). Miri: PASS, 13 tests
   (elapsed 17-36 s per filter). An intermediate fuzz attempt FAILED honestly (exit 1 + 2x exit 124 at the 170 s
   cap) because concurrent C7 edits left the tree briefly non-compiling (`Storage` missing `Debug`) and then churned
   the engram-core fingerprint; rerun on the settled tree passed. Not run: cargo-mutants, GitHub-hosted workflow,
   audit with a fresh advisory-db fetch, the pinned dated nightly itself (local toolchain is plain `nightly`).
