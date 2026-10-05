# Task Q2 report: reliable Rust quality inventory

Status: DONE_WITH_CONCERNS (see Concerns). Date 2026-10-05. Counts taken at HEAD
`46fc3e2969c32967efff590b706dd05894938b12` (branch `claude/engram-improvement-plan-edb43d`).

## What was implemented

- `scripts/rust_risk_inventory.py`: stdlib-only lexer + module-tree/cfg classifier
  for unwrap/expect/unwrap_err/expect_err/unwrap_unchecked, panic/assert macros,
  unsafe (with SAFETY placement), prints, io handles, process::exit, and
  error-silencing signals (`let _ =`, `.ok();` discard, bound `.ok()`,
  `filter_map(|r| r.ok())`, `unwrap_or_else(|_|..)`, `Err(_) => {}`).
  Scopes: prod-lib, prod-bin, test-unit, test-integration, bench, example,
  build-script, macro-def, disabled, unknown. Reachability is always `unknown`.
- `scripts/test_rust_risk_inventory.py`: 30 fixture tests (raw strings, nested
  block comments, doc comments, escapes/char/lifetimes, nested cfg(test) modules,
  cfg(test) items in non-test files, #[test], three-valued cfg, cfg_attr,
  unparsed cfg -> unknown, inner #![cfg(test)], cfg on match arms, macro_rules
  bodies vs invocations, out-of-line test modules, #[path], include!, orphan
  files, tests/benches/examples/build.rs/src/bin vs lib, unsafe kinds + SAFETY
  placement, swallow signals, deterministic JSON/CLI filters).
- `docs/quality/rust-risk-inventory.md`: method + limits, counts (exact command,
  date, SHA), production unwrap/expect/panic triage, unsafe/SAFETY audit (3
  real blocks), CLI/protocol vs library prints, prioritized backlog (25 items,
  each REPRODUCED or HYPOTHESIS), fix contract, reproduction recipes, not-run list.
- `docs/harness/progress/2026-10-05-improvement-lane-r.md`: appended a short PT-BR
  entry (last step before commit).

No file under `src/`, `benches/`, `tests/` was modified.

## Key results

- Textual `.unwrap()` 1 601 -> 32 in production, none a panic-by-input candidate
  (regex literals, guarded invariants, infallible constants, mutex poison, bench
  harness). Textual `unsafe` 28 -> 3 real blocks (2 prod with correct SAFETY
  comments, 1 bench without a `SAFETY:` label).
- 9 findings REPRODUCED (real `engram-server` stdio, real `engram-cli`, public
  lib API) + 1 function-level:
  - Q2-B01 `memory_search_compact` (`search.rs:732`), Q2-B02 `memory_prepare_context`
    (`context_grouper.rs:118`), Q2-B03 `memory_garden` (`gardening.rs:316`, even
    dry_run), Q2-B04 `context_budget_check` (`compression.rs:269`), Q2-B05
    `attestation_chain_verify` key (`attestation.rs:235`): process panics, exit 101,
    byte-slice of user text/hex. Release profile is `panic = "abort"` and the stdio
    loop has no catch_unwind, so one stored memory can repeatedly abort the server.
  - Q2-B06 snapshot `parse_hex_key`: function-level (verbatim copy compiled with rustc).
  - Q2-B07 `engram-cli mcp install`: non-strict-JSON config is silently replaced,
    `.bak` is overwritten on the next run (data loss), success message printed.
  - Q2-B08 `session_index max_chars=5`: subtract overflow (debug build).
  - Q2-B09 `StorageBackend::delete_crossref` returns Ok(()) with the table dropped.
  - Q2-B10 `list_memories` returns 0 rows without error while COUNT(*) is 1.

## Verification (commands, exit codes, counts)

| Command | Exit | Result |
|---|---:|---|
| `python3 -m unittest scripts/test_rust_risk_inventory.py` | 0 | Ran 30 tests, OK |
| `python3 scripts/rust_risk_inventory.py` | 0 | 572 files, 66 roots, 0 orphans, 0 unresolved mod, 0 lexer errors, 0 unparsed cfg |
| lexer cross-check vs `git grep -hoF '.unwrap()'` | n/a | 1601 textual vs 1590 counted; the 11 difference = 10 `//!` doc examples + 1 string fixture (listed in doc) |

Cargo/Clippy: this task changes no Rust. The repo pre-commit hook (`cargo fmt --check` +
`cargo clippy --all-targets --all-features -D warnings`) first failed (rc=1) on another
session's unformatted WIP in `tests/workspace_auth_enforcement_tests.rs` (not mine, untouched);
after that file became fmt-clean the same commit command passed the hook (exit 0).

## Commit

`a87a3ce docs(harness): add Rust risk inventory and classifier` (parent `46fc3e2`; 4 files, +2134).
Scope `quality` is rejected by `docs/harness/bin/check-commit-msg.sh`, hence `harness`.

## TDD evidence (honest account)

The classifier was built exploratory-first against the real repository (a
reverse of strict RED-first); the fixture suite was written afterwards. To prove
the fixtures actually bite (the substitute for a RED run), seven single-line
mutations of the tool were run against the 30 tests, each restored afterwards
(`diff` against a saved copy, then 30/30 OK):

| Mutation | Failing tests |
|---|---|
| raw strings disabled | 1 (`test_raw_strings_hide_code_like_text`) |
| nested block comments flattened | 1 (`test_comments_nested_and_doc`) |
| cfg gates ignored (e_prod/e_test = unknown) | 8 |
| `macro_rules!` bodies treated as code | 1 |
| gates not propagated to child files | 3 |
| no bracket jumping in statement-start walk | 1 (`test_ok_discard_ignores_bound_multiline_chains_with_closure_bodies`) |
| block end never ends a statement | 2 (SAFETY leak tests) |

Two real bugs found by the repository run and fixed with tests added: SAFETY
window of 4 lines was too loose (now contiguous-comment-above-statement rule),
and bound multi-line `.ok();` chains with closure bodies were mis-counted as
discards (now bracket-jumping statement start, plus a separate `ok_bound` kind).

## Backlog reproduction evidence (summary; full recipes in the doc appendix)

- stdio: `target/debug/engram-server --transport stdio --embedding-model tfidf` under
  `env -i`, throwaway HOME/DB. B01: memory_create `"hello world "*6 + "x" + "é"*20` then
  `memory_search_compact {query:"hello"}` -> stderr `panicked at src/mcp/handlers/search.rs:732:53: end byte
  index 80 is not a char boundary; it is inside 'é' (bytes 79..81 of string)`, exit 101. ASCII control -> ok.
  B02 `context_grouper.rs:118:55` (500), B03 `gardening.rs:316:31` (4096), B04 `compression.rs:269:38` (50),
  B05 `attestation.rs:235:45` ("abc": end byte index 4 is out of bounds for string of length 3),
  B08 `session_indexing.rs:195:21` (attempt to subtract with overflow).
- CLI B07: HOME sandbox + `engram-cli --db-path ... mcp install --client claude` twice.
- Lib API B09/B10: scratch crate outside the repo (`engram-core` path dependency,
  repo `Cargo.lock` copied, `--offline`): `queries::delete_crossref -> Err("Database error: no such table: crossrefs")`
  vs `StorageBackend::delete_crossref -> Ok(())`; `list_memories -> Ok(0) rows; SELECT COUNT(*) -> 1`.
- Binaries used were `target/debug/*` built 2026-10-05 01:29 by the C2 session; the later
  C2 commits touch only `src/sync` and `src/storage/migrations`.

## Deviations from the brief

- The brief lists `src/storage/connection.rs`, `src/storage/lock.rs`, `benches/search.rs` for
  review; all three were read in full (relevant parts) and audited; findings are backlog
  items only (Q2-B13, B15, B16). Nothing in them was modified (C2 is reviewing).
- "Tool in its own PR with fixtures": the tool and fixtures are committed together with the
  inventory on the lane branch (local commit only, no PR per conventions).
- `unwrap`/`expect` "path alcançável" is not inferred by the tool: always `unknown`; reachability
  is stated per backlog item after manual tracing/reproduction (documented in Limits).

## Concerns

1. `scripts/rust_risk_inventory.py` is ~1 160 lines (> the 800-line guideline; `analyze()` ~150 lines).
   Kept as one file because the decision text fixes the path. A later split (lexer / cfg+tree / report)
   is straightforward if wanted.
2. Counts reflect HEAD `46fc3e2`; C2 may land more commits (a modified
   `tests/workspace_auth_enforcement_tests.rs` was in the worktree from C2, not mine). The commands
   in the doc re-derive the numbers.
3. `cfg` on non-item units (match arms/fields/expressions) is approximate (286 units, 6 test-gated,
   all in `src/sync/cloud_backend.rs`); stated in Limits.
4. Reproductions were debug builds on macOS arm64 only; HTTP transport, release (abort) binary, Linux
   and the snapshot handler path (B06) were not run.

## Not run

Listed in the doc under "Not run / pending": release-binary abort, HTTP transport, Linux, snapshot handler
path, compact() failure injection, filesystems without flock, Turso backend.

## Files changed

- `scripts/rust_risk_inventory.py` (new)
- `scripts/test_rust_risk_inventory.py` (new)
- `docs/quality/rust-risk-inventory.md` (new)
- `docs/harness/progress/2026-10-05-improvement-lane-r.md` (appended)
