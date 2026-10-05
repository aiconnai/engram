# Rust risk inventory (Q2)

Status: inventory only. Nothing in this document authorizes removing or
refactoring code. Every fix is a separate patch that follows the contract in
[Fix contract](#fix-contract).

| | |
|---|---|
| Date | 2026-10-05 |
| Commit | `46fc3e2969c32967efff590b706dd05894938b12` (branch `claude/engram-improvement-plan-edb43d`) |
| Tree | tracked files identical to the commit (only the two new `scripts/*risk_inventory*.py` files were untracked) |
| Platform | macOS arm64, Python 3.14 (standard library only) |
| Scope | 572 tracked `*.rs` files, 964 007 tokens, 66 Cargo target roots (3 libs, 6 bins, 45 integration tests, 7 benches, 4 examples, 1 build script) |

## 1. Summary

- A raw `grep` overstates production risk by two orders of magnitude.
  `.unwrap()` appears 1 601 times textually; **32** are in production
  code, and of those 32 none is a reachable-by-input candidate (all are static
  regex literals, loop-guarded invariants, infallible constants, mutex poison
  or the in-library bench harness; see section 4).
- `unsafe` appears 28 times textually (5 as a whole word; the rest are
  identifiers such as `unsafe_raw_payload`, a string and a doc comment). There
  are **3** real `unsafe` blocks: two in
  production (`flock`, `statvfs`, both with a correct `SAFETY:` comment) and
  one in a benchmark (rationale comment but no `SAFETY:` label).
- The actionable risk is not `unwrap`/`expect`. It is **byte-index slicing of
  user text/hex** (`&s[..N]`) and **errors silently turned into success**.
  Nine findings were reproduced with the real `engram-server` binary, the real
  `engram-cli` binary or the public library API, and one more at function
  level; each is marked REPRODUCED below and the rest are explicitly
  hypotheses.
- Release profile is `panic = "abort"` (`Cargo.toml` `[profile.release]`), and
  the stdio loop (`src/mcp/protocol.rs`) has no `catch_unwind`. A panic in any
  tool handler therefore terminates the whole server process, not just the
  request. Because the trigger is data stored in the database, restarting does
  not clear it ("poison pill" memory).
- CLI/protocol stdout is legitimate and must not be replaced by `tracing`. Only
  6 library `eprintln!` calls exist outside binaries (section 6).

## 2. Method (reproducible)

Tool: [`scripts/rust_risk_inventory.py`](../../scripts/rust_risk_inventory.py),
tests: [`scripts/test_rust_risk_inventory.py`](../../scripts/test_rust_risk_inventory.py)
(30 fixture tests).

```bash
python3 -m unittest scripts/test_rust_risk_inventory.py        # tool self-tests
python3 scripts/rust_risk_inventory.py                          # markdown summary (section 3 tables)
python3 scripts/rust_risk_inventory.py --format json > inv.json # every occurrence, with context
python3 scripts/rust_risk_inventory.py --format list --kind unwrap --scope prod-lib
python3 scripts/rust_risk_inventory.py --files walk             # fallback when not in a git checkout
```

What the tool does:

1. Lists tracked `*.rs` files (`git ls-files`; `--files walk` falls back to a
   directory walk that skips `target`, `.git`, `.claude`, `.worktrees`).
2. Lexes each file: line/doc/nested block comments, strings, byte strings,
   raw strings with any number of `#`, char literals vs lifetimes, raw
   identifiers. Text inside comments and strings is never counted.
3. Discovers Cargo target roots from every `Cargo.toml` (lib, `[[bin]]`,
   `src/bin/*`, `tests/`, `benches/`, `examples/`, `build.rs`), resolves the
   `mod x;` / `#[path]` / `include!("lit")` tree from each root, and
   propagates `cfg` gates down to child files.
4. Evaluates `#[cfg(...)]` three-valued with `test` known and every feature,
   target and `debug_assertions` unknown. `cfg(test)`, `cfg(all(test, ...))`
   and `#[test]`/`#[tokio::test]` items are test scope; `cfg(not(test))`,
   `cfg(feature = "x")` and `cfg(any(test, feature = "x"))` stay production
   (the latter two flagged `conditional` with the feature list).
5. Assigns each occurrence a scope: `prod-lib`, `prod-bin`, `test-unit`
   (cfg(test) code under `src/`, including out-of-line `#[cfg(test)] mod x;`
   files), `test-integration` (`tests/`), `bench`, `example`, `build-script`,
   `macro-def`, `disabled`, or `unknown`.

Kinds counted: `unwrap`, `expect`, `unwrap_err`, `expect_err`,
`unwrap_unchecked`, `panic_macro` (`panic!`, `unreachable!`, `todo!`,
`unimplemented!`), `assert_macro` (non-debug `assert*!`), `unsafe` (block, fn,
impl, trait, extern block, `#[unsafe(..)]`, with a SAFETY check), `print`
(`print!`, `println!`, `eprint!`, `eprintln!`, `dbg!`), `io_handle`
(`io::stdout()`/`stderr()`), `process_exit`, and silence signals
`let_underscore` (`let _ = ...`, with a heuristic `rhs_class`), `ok_discard`
(statement-level `.ok();` whose value is dropped), `ok_bound` (`let x = ....ok();`,
i.e. any error turned into `None`), `filter_map_ok` (`.filter_map(|r| r.ok())`,
`Result::ok`), `err_default_closure` (`.unwrap_or_else(|_| ...)`),
`err_arm_ignored` (`Err(_) => {}`).

### Limits (what stays unknown)

- **Reachability from external input is never inferred.** Every record carries
  `"reachability": "unknown"`. Section 7 states reachability only for the
  backlog items that were traced by hand and, where marked REPRODUCED, run.
- Counts are syntactic: a `.expect(` on a user type that defines its own
  `expect` would be counted. The tool reports such definitions
  (`custom_unwrap_expect_defs`); at this commit there are 0.
- `cfg` attributes on non-item units (match arms, struct fields, expressions)
  use approximate unit boundaries (`,`/`;`/closing bracket). 286 such units
  exist, 6 of them test-gated (`#[cfg(test)]` enum variant and match arms of
  `CloudBackend::Fixture` in `src/sync/cloud_backend.rs`); the effect on
  counts is negligible but it is a known approximation.
- `macro_rules!` bodies are `macro-def` (expansion context unknown). Unparsed
  `cfg` predicates give `unknown` (0 at this commit). Files unreachable from
  any target root give `unknown` (0 at this commit).
- `#[path]` is resolved relative to the declaring file's directory; `include!`
  only with a string literal (`include!(concat!(env!("OUT_DIR"), ...))` is
  reported as unresolved; 0 at this commit).
- Doc-comment examples (doctests) are comments and are not counted.
- Procedural-macro output and `build.rs` generated code are not visible.
- Not examined: dependencies' `unsafe`, SDKs (`sdks/`), non-Rust code.

### Validation of the lexer against the repository

Textual `.unwrap()` count (`git grep -hoF '.unwrap()' -- '*.rs'`) is 1 601; the
tool counts 1 590. The 11 missing are exactly: 10 inside `//!` doc examples
(`attestation/mod.rs`, `embedding/{clip,cohere,ollama,voyage}.rs`,
`multimodal/vision.rs`, `snapshot/mod.rs`) and 1 inside a string fixture
(`context/reducers/cargo_clippy.rs:278`). The same cross-check for
`.expect(` is 1 908 textual vs 1 907 counted. For production/test split, the
tool agrees with a naive "before the first `#[cfg(test)]` line" rule for every
in-file test module and disagrees only for out-of-line test files
(`src/**/tests.rs`, `*_tests.rs`), where the naive rule is wrong and the
module tree is right.

Fixture tests (30) cover: raw strings (`r#".."#`, `br##".."##`, multi-line),
line/doc/nested block comments, strings with `//` and escaped quotes, char
literals vs lifetimes vs labels, nested `#[cfg(test)]` modules, `cfg(test)`
items inside production files (fn, const, use, impl with generics, fn with
where clause), `#[test]`/`#[tokio::test]`, three-valued predicates,
`cfg_attr` (ignored), unparsed cfg stays `unknown`, inner `#![cfg(test)]` on
file and inline mod, cfg on match arms, `macro_rules!` bodies vs invocation
arguments, macro names inside strings/idents, out-of-line test modules and
`mod.rs` nesting, cfg propagation into child files, `#[path]`, unresolved
`mod`, `include!`, orphan files, `src/main.rs` / `src/bin/*.rs` /
`src/bin/x/main.rs` / `tests/` / `benches/` / `examples/` / `build.rs`,
unsafe kinds and SAFETY placement (including a closure-body brace that must not
end the statement), bound vs discarded `.ok();`, `let _ =`
classes, `filter_map` forms, deterministic JSON and CLI filters. As a sanity
check that the tests bite, seven single-line mutations of the tool (disable
raw strings; flatten nested block comments; ignore `cfg` gates; treat
`macro_rules!` as code; drop gate propagation into child files; drop bracket
jumping in the statement-start walk; never end a statement at a block) each
made 1 to 8 of the 30 tests fail; the unmutated tool passes 30/30.

## 3. Counts (commit `46fc3e2`, 2026-10-05)

Command: `python3 scripts/rust_risk_inventory.py`. Stats: 572 files reached by
a root, 0 orphans, 0 unresolved `mod`, 0 lexer errors, 0 bracket errors, 0
unparsed `cfg`, 1 file shared by two roots (`tests/support/mod.rs`, both
integration tests), 0 files reached from conflicting target kinds.

| kind | prod-lib | prod-bin | test-unit | test-integration | bench | example | build-script | total |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| unwrap | 32 | 0 | 1174 | 323 | 60 | 0 | 1 | 1590 |
| expect | 37 | 8 | 1015 | 838 | 8 | 0 | 1 | 1907 |
| unwrap_err | 0 | 0 | 58 | 2 | 0 | 0 | 0 | 60 |
| expect_err | 0 | 0 | 28 | 30 | 0 | 0 | 0 | 58 |
| panic_macro | 4 | 2 | 36 | 19 | 1 | 0 | 0 | 62 |
| assert_macro | 0 | 0 | 4290 | 1659 | 10 | 0 | 0 | 5959 |
| unsafe | 2 | 0 | 0 | 0 | 1 | 0 | 0 | 3 |
| print | 6 | 236 | 0 | 8 | 2 | 38 | 2 | 292 |
| io_handle | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 4 |
| process_exit | 0 | 12 | 0 | 0 | 0 | 1 | 0 | 13 |
| let_underscore | 86 | 7 | 45 | 37 | 2 | 0 | 0 | 177 |
| ok_discard | 16 | 0 | 1 | 3 | 0 | 0 | 0 | 20 |
| ok_bound | 27 | 3 | 1 | 1 | 0 | 0 | 0 | 32 |
| filter_map_ok | 92 | 2 | 5 | 1 | 0 | 0 | 0 | 100 |
| err_default_closure | 106 | 6 | 0 | 1 | 0 | 0 | 0 | 113 |

`prod-lib` includes the `engram-wasm` crate (4 `expect`, 8 `err_default_closure`).
`unwrap_unchecked`, `todo!`, `unimplemented!`, `dbg!`: 0 occurrences.

Production counts by target:

| target | unwrap | expect | panic | unsafe | print | io_handle | exit | let _ | .ok(); discard | .ok() bound | filter_map_ok | unwrap_or_else(\|_\|) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| engram-core lib | 32 | 33 | 4 | 2 | 6 | 2 | 0 | 86 | 16 | 27 | 92 | 98 |
| engram-wasm lib | 0 | 4 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 8 |
| bin engram-cli | 0 | 2 | 2 | 0 | 182 | 2 | 10 | 2 | 0 | 2 | 2 | 3 |
| bin engram-agent | 0 | 0 | 0 | 0 | 39 | 0 | 0 | 0 | 0 | 0 | 0 | 1 |
| bin engram-bench | 0 | 0 | 0 | 0 | 14 | 0 | 1 | 0 | 0 | 0 | 0 | 0 |
| bin engram-server | 0 | 4 | 0 | 0 | 0 | 0 | 0 | 2 | 0 | 1 | 0 | 1 |
| bin engram-watcher | 0 | 2 | 0 | 0 | 0 | 0 | 0 | 3 | 0 | 0 | 0 | 1 |
| bin engram-pdf-worker | 0 | 0 | 0 | 0 | 1 | 0 | 1 | 0 | 0 | 0 | 0 | 0 |

Textual baselines for comparison (`git grep -hoF`): `.unwrap()` 1 601,
`.expect(` 1 908, `unsafe` 28 (`-w`: 5, of which 3 real), `println!` 302 (this substring also matches `eprintln!` 33), `let _ =` 177.

The counts were identical before and after the two C2 fix commits that landed
while this inventory was being produced (`a9c3bd8`, `7a7809f`, `46fc3e2`
changed `src/sync`, `src/storage`, docs). Re-run the commands above after
later C2 commits; `src/sync/*` and `src/storage/{connection,migrations}` are
under C2 review and are only listed here, not changed.

## 4. Production `unwrap` / `expect` / panic-macro triage

All 32 + 45 + 6 production sites were read. Result: none is a hypothesis for
panic-by-input. This is the evidence that "zero textual occurrences" is the
wrong target.

| Class | Count | Sites | Verdict |
|---|---:|---|---|
| Static regex literal in `Lazy`/init | 13 `unwrap` + 22 `expect` | `intelligence/{aaak.rs:559-576, context_compression.rs:57, entities.rs:263-315, compression_semantic.rs:90-105, entity_extraction.rs:42-53, fact_extraction.rs:73-133}`, `engram-wasm/src/entity.rs:35-46` | Patterns are literals or `regex::escape`d; fails only on a programming error at first use. Keep. |
| Static tokenizer init | 2 `expect` | `intelligence/token_counter.rs:17,22` (`tiktoken_rs::cl100k_base()/o200k_base()`) | Embedded vocabulary; keep. |
| Guarded by a preceding check | 9 `unwrap` + 3 `expect` | `attestation/merkle.rs:36` (loop condition), `graph/conflicts/resolver.rs:182,227` (`is_empty` return), `storage/filter.rs:157` (`starts_with`), `mcp/handlers/multimodal.rs:576` (match pattern), `search/rerank.rs:293-296` (ranks are a permutation of `0..n`), `embedding/queue/status.rs:102` (`chunks_exact(4)`), `bin/cli/mcp.rs:274,290` and `mcp/handlers/context/assembly.rs:246` (`is_object()` ensured above) | Sound today; fragile under edits (candidates for `let ... else`, not urgent). |
| Closure-ran-once invariants | 9 `expect` | `storage/connection.rs:182,187,206,211,534,539`, `storage/db.rs:34,38,48` | `?` returns before the `expect` if the closure errs; sound. (`connection.rs`: C2 file.) |
| Infallible constant | 3 `unwrap` | `intelligence/natural_language.rs:403,406,411` (`and_hms_opt(0,0,0)`) | Cannot fail. |
| `std::sync::Mutex` poison | 5 `unwrap` | `embedding/cache.rs:178,203,262,283,293` | Panics only after another thread panicked while holding it. Low. |
| In-library benchmark harness | 2 `unwrap` | `bench/longmemeval.rs:117,163` | Bench-only code compiled into the lib; low. |
| Startup runtime/client construction | 8 `expect` | `bin/server.rs:297,685,888,939`, `bin/watcher.rs:186,341`, `embedding/mod.rs:96`, `portability/markdown.rs:128` | Fails only on OS resource exhaustion at startup; see Q2-B20. |
| Env-driven startup validation | 1 `expect` | `integrations/langfuse.rs:108` | Documented panic on bad operator env; see Q2-B19. |
| `unreachable!` after exhaustive guard | 5 | `graph_queries.rs:497`, `queries/core/memory_create.rs:69`, `scoping.rs:133`, `bin/cli/main.rs:117,124` | Each is guarded by an earlier `continue`/`if`/early dispatch; sound. |
| Fault-injection branch | 1 `panic!` | `intelligence/pdf_worker.rs:92` | Controlled by an `IoFault` test knob; keep, note only. |

`assert!`/`assert_eq!` in production: 0.

## 5. `unsafe` / SAFETY audit

Real `unsafe` constructs: 3 blocks. No `unsafe fn`, `unsafe impl`,
`unsafe trait`, `extern` block, `static mut`, `#[unsafe(..)]` attribute, and no
crate-level `#![forbid(unsafe_code)]`/`deny` (tool stat `unsafe_code_lints: 0`).

| # | Location | Operation | SAFETY comment | Assessment |
|---|---|---|---|---|
| 1 | `src/storage/lock.rs:79` (prod) | `libc::flock(file.as_raw_fd(), LOCK_EX \| LOCK_NB)` | Yes (line 78): "`file` owns a valid fd; flock with these flags is safe." | Adequate: the `File` outlives the call. Behavior issue (not safety): any non-zero return is reported as "already locked"; see Q2-B15. |
| 2 | `src/storage/connection.rs:313` (prod) | `mem::zeroed::<libc::statvfs>()` + `libc::statvfs(cpath, &mut stat)` | Yes (lines 311-312): "`cpath` is a valid NUL-terminated path; `stat` is zeroed before being filled by statvfs(3)." | Adequate: `statvfs` is a plain C struct of integers (zero is a valid bit pattern); `CString::new(..).ok()?` rules out interior NUL. Could use `MaybeUninit`, not required. |
| 3 | `benches/search.rs:359` (bench only) | `sqlite3_auto_extension(Some(transmute(sqlite3_vec_init as *const ())))` inside `Once::call_once` | Rationale comment inside the block ("mirrored verbatim from sqlite-vec-0.1.6 ... transmute target shape is an internal detail") but **no `SAFETY:` label** and no statement of the invariants (extension entry point has the SQLite extension ABI; registration is process-global and idempotent via `Once`). | Needs a `// SAFETY:` comment; see Q2-B16. Not in the production binaries. |

Cross-check: of the 28 textual `unsafe` hits, 3 are the real blocks above, 2
more are whole-word non-code (a string in `src/dream/eval.rs:358` and a doc
comment in `src/integrations/langfuse.rs:100`), and 23 are identifiers or
strings such as `unsafe_raw_payload`, `unsafe_marker`, `unsafe_fixtures` in
`src/dream/*` and `src/mcp/tools/catalog/misc.rs`.

## 6. Prints: CLI/protocol output vs library output

Rule: user-facing CLI output and protocol output are legitimate and are **not**
converted to `tracing` by bulk substitution.

| Category | Count | Verdict |
|---|---:|---|
| `engram-cli` (`src/bin/cli/**`) `println!`/`eprintln!` | 182 | CLI product output. Legitimate. Concentrated in `mcp.rs` (26), `interactive.rs` (23), `routing.rs` (22), `compact.rs`/`snapshot.rs` (17 each). |
| `engram-agent`, `engram-bench`, `engram-pdf-worker` | 39 + 14 + 1 | CLI/stdout reporting of those binaries. Legitimate. |
| `process::exit(1)` in binaries | 12 | CLI error exits after an error message (`cli/attest.rs`, `cli/snapshot.rs`, `cli/maintenance/queue.rs`, `bench.rs`, `pdf_worker.rs`). Legitimate; not a library concern. |
| `engram-server` (MCP stdio) binary | 0 | Server uses `tracing`; nothing can corrupt the protocol stream. |
| Protocol writers in the library | 2 `io_handle` | `src/mcp/protocol.rs:109` (JSON-RPC over a locked stdout, response writes propagate with `?`; progress notifications use best-effort `let _ = writeln!`, acceptable) and `src/intelligence/pdf_worker/runtime.rs:37` (worker stdout pipe). Legitimate protocol I/O. |
| `build.rs` | 2 | `cargo:rerun-if-changed=...` directives. Protocol with Cargo; must stay `println!`. |
| Tests, benches, examples | 8 + 2 + 38 | Not production. |
| **Library `eprintln!`** | **6** | stderr only (cannot corrupt stdout protocol). Individually triaged below. |

Library `eprintln!` sites (all `prod-lib`):

| Site | What it prints | Note |
|---|---|---|
| `src/app_state.rs:85` (`hooks`) | hook enablement diagnostics | Diagnostic only. |
| `src/hooks/mod.rs:113` (`hooks`) | `Hook handler error for ...` | Error is also dropped by `trigger_and_forget` (`let _ = self.trigger`). |
| `src/hooks/post_tool_use.rs:30`, `src/hooks/stop.rs:12` (`hooks`) | `[Hook] PostToolUse: tool=...` on every tool call | Observed on stderr in every reproduction run below; noisy, harmless to the protocol. |
| `src/mcp/handlers/markdown_export/export/query.rs:109` | `build_related_map DB error` then returns an empty map | An operational error reduced to a stderr line; export silently loses relations. See Q2-B11 (same class). |
| `src/mcp/handlers/markdown_export/import/apply.rs:44` | warns it is wiping all tags (`engram_tags_list` empty) | The warning is the only signal of a destructive import step. Low. |

## 7. Prioritized backlog (concrete failure candidates)

Priority: P1 = process abort or silent data loss reachable from tool/CLI input;
P2 = silent wrong result or invariant violation; P3 = latent, documentation or
robustness. Status: **REPRODUCED** (a focused run produced the failure at this
commit) or **HYPOTHESIS** (static reading only, no reproduction).

Reproduction inputs below were run against debug builds of `engram-server`
and `engram-cli` built from this worktree (`target/debug`, modified
2026-10-05 01:29; the later C2 commits `c74f1c0` and `a9c3bd8` touch only
`src/sync`, `src/storage/migrations` and their tests, and `src/mcp`,
`src/intelligence` and `src/bin` are unchanged since 00:16), always with
`env -i`, a throwaway `HOME` and a temporary database; no network, no real
credentials.

### P1

| ID | Location | Reachable input | Why it fails | Status |
|---|---|---|---|---|
| Q2-B01 | `src/mcp/handlers/search.rs:732` (`memory_search_compact`) | Any memory whose first line is > 80 bytes and has a multibyte char spanning byte 80, then `memory_search_compact {query}` | `&first_line[..80]` is a byte slice; not a char boundary. Panic aborts the process; the stored memory makes every matching search abort again. | **REPRODUCED** (stdio, exit code 101: `end byte index 80 is not a char boundary; it is inside 'é' (bytes 79..81)`). ASCII control does not fail. **FIXED** in `a6f55ac` (C3). |
| Q2-B02 | `src/intelligence/context_grouper.rs:118` (via `memory_prepare_context`) | Memory whose joined group text exceeds 500 bytes with a multibyte char at byte 500 | `&combined[..500]` | **REPRODUCED** (exit 101, `panicked at context_grouper.rs:118:55`). **FIXED** in `a6f55ac` (C3). |
| Q2-B03 | `src/intelligence/gardening.rs:316` (via `memory_garden`, even `dry_run: true`) | Memory > 4096 bytes with a multibyte char crossing byte 4096 | `&m.content[..MAX_CONTENT.min(len)]` | **REPRODUCED** (exit 101, `gardening.rs:316:31`). **FIXED** in `a6f55ac` (C3). |
| Q2-B04 | `src/intelligence/compression.rs:269` (via `context_budget_check`) | Memory > 50 bytes with a multibyte char crossing byte 50 | `&content[..50]` for the preview | **REPRODUCED** (exit 101, `compression.rs:269:38`). **FIXED** in `a6f55ac` (C3). |
| Q2-B05 | `src/mcp/handlers/attestation.rs:232-236` `parse_hex_key` (tools `attestation_log` `sign_key`, `attestation_chain_verify` `verifying_key`) | `verifying_key: "abc"` (odd length) or a non-ASCII string | `&hex_str[i..i + 2]` with no length/ASCII precheck (`snapshot.rs` prechecks length only) | **REPRODUCED** through the real handler (exit 101, `attestation.rs:235:45: end byte index 4 is out of bounds for string of length 3`); non-ASCII variant reproduced on the verbatim function body. **FIXED** in `a6f55ac` (C3; `hex::decode`, typed error). |
| Q2-B06 | `src/mcp/handlers/snapshot.rs:8-18` `parse_hex_key` (tools `snapshot_create`/`snapshot_load` keys) | A 64-byte key containing one 2-byte char, e.g. `"0é" + "0"*61` | length check is in bytes, then `&hex_str[i..i + 2]` splits the char | **REPRODUCED at function level** (verbatim copy of the function compiled standalone: `end byte index 2 is not a char boundary`). Handler path not exercised. **FIXED** in `a6f55ac` (C3; `hex::decode`, typed error). |
| Q2-B07 | `src/bin/cli/mcp.rs:249-290` `install_to_config_file` | `engram-cli mcp install --client claude` while `claude_desktop_config.json` has a trailing comma / comment (not strict JSON) | `serde_json::from_str(..).unwrap_or_else(\|_\| json!({}))` replaces the whole config by `{}` plus the `engram` entry; the backup is written with `let _ = fs::write(&bak_path, ..)` (result ignored) and is **overwritten on every run**, so a second run destroys the only copy. Output says `(created)`. | **REPRODUCED** (HOME sandbox: `other-server` entry lost, success message printed; after a second run `.bak` contained the already-clobbered file). **FIXED** in `ab22911` (Q2F): unparseable config is refused unless `--force`; backups are never overwritten (`.bak`, then `.bak.<ts>[.<n>]`); a per-path failure exits non-zero. |

Shared pattern for Q2-B01..B06: byte-index slicing of user text. A correct
helper already exists privately at `src/mcp/handlers/context/mod.rs:13`
(`safe_truncate`, with comment "Avoids panics on multibyte (emoji, CJK,
accented) input") and similar `boundary` loops in `context/bundle.rs:499`,
`mcp/handlers/digest.rs:614`, `context/tool_output.rs:174`; the unsafe sites
are the ones that were not migrated. Whether to share one helper is a design
decision for the fix patches, not part of this inventory.

### P2

| ID | Location | Reachable input | Why | Status |
|---|---|---|---|---|
| Q2-B08 | `src/intelligence/session_indexing.rs:195` (`session_index`, `max_chars`); `src/mcp/handlers/session.rs:57-60` (`ttl_days * 24 * 60 * 60`) | `max_chars: 5` with content longer than 5 bytes; huge `ttl_days` | `max_chars - marker.len()` usize underflow; `i64` multiply overflow. Debug/test builds panic; release (no `overflow-checks`) wraps, silently ignoring the size limit / giving a wrong TTL. | **REPRODUCED** in the debug binary for `max_chars` (`attempt to subtract with overflow`, exit 101); release behavior and the TTL overflow are HYPOTHESIS (static). **FIXED** in `a6f55ac` (C3; `max_chars` underflow and `ttl_days` bound). |
| Q2-B09 | `src/storage/sqlite_backend/mod.rs:267-270` (`StorageBackend::delete_crossref`) | Any DB error while deleting (locked, I/O, missing table) | The loop does `let _ = queries::delete_crossref(...)` for every edge type to ignore `NotFound`, but also discards every other error; the transaction commits and the method returns `Ok(())`. | **REPRODUCED** (public API: with `crossrefs` dropped, `queries::delete_crossref` -> `Err("no such table: crossrefs")`, `StorageBackend::delete_crossref` -> `Ok(())`). **FIXED** in `62c266b` (Q2F): only `NotFound` is ignored, any other error rolls back and is returned. |
| Q2-B10 | `src/storage/queries/core/list.rs:112` (`list_memories`), also `snapshot/builder.rs:232,337`, `search/hybrid.rs:308,471`, `queries/core/compact.rs:180` | A row whose column fails to decode (corruption, schema drift, a value of the wrong type) | `.filter_map(\|r\| r.ok())` drops the row without error or count; `list_memories` then also does `load_tags(..).unwrap_or_default()`. Snapshot export would silently omit edges/entities while reporting success. 92 production `filter_map_ok` sites in total. | **REPRODUCED** for `list_memories` (public API: `importance = 'not-a-number'` -> `Ok(0 rows)` while `SELECT COUNT(*) FROM memories` is 1). Snapshot/hybrid sites are HYPOTHESIS (same pattern, not run). **FIXED for `list_memories`** in `9e0b306` (Q2F): an undecodable row or a failed tag load is a typed error (MCP `internal_error`), never a shorter list. Snapshot/hybrid/compact sites remain HYPOTHESIS. |
| Q2-B11 | `src/mcp/handlers/memory_crud/create.rs:119-137` (`memory_create`); `markdown_export/export/query.rs:105-110` | Embedding persistence failure after the memory commit | `let _ = ctx.storage.with_connection(..)` drops the error and the in-memory HNSW insert runs anyway: response is success, the DB has no embedding, the index diverges until restart. `build_related_map` turns a DB error into an empty map plus a stderr line. | HYPOTHESIS. **PARTIALLY FIXED**: `memory_create` embedding persistence/HNSW ordering fixed in `f560a6c` (C7); `markdown_export` `build_related_map` still turns a DB error into an empty map (open). |
| Q2-B12 | `src/mcp/handlers/memory_crud/create.rs:340-358` (`context_seed`) and `:702-722` (`memory_create_batch`) | Batch of N memories with a networked embedder (OpenAI/Ollama/...) | `ctx.embedder.embed()` (provider call) runs inside the `with_connection` closure, i.e. while the global connection mutex is held (head-of-line blocking of every DB user, N network round trips under the lock); per-row `let _ = conn.execute(..)` drops write errors. Violates "no external/provider call inside a DB scope". | HYPOTHESIS (static). |
| Q2-B13 | `src/storage/connection.rs:331-345` (`compact`, C2 file) | A failing `PRAGMA page_size/page_count` or count query during `compact --apply` | `unwrap_or(0)` makes `db_size_bytes = 0`, so `vacuum_safe = free >= 0` is true and `VACUUM` runs without the disk-space guard; report counts read 0 instead of failing. | HYPOTHESIS (needs an injected PRAGMA failure). |
| Q2-B14 | `src/storage/memory_blocks.rs:240-241` (`archive_overflow`) | A block whose `max_tokens * 4` byte offset is mid-char | Code slices bytes (`&content[..max_chars]`) under a comment saying "Truncate at a char boundary"; `max_tokens * 4` also unchecked. No non-test caller found (`git grep`), so latent public API. | HYPOTHESIS. **FIXED** in `a6f55ac` (C3). |
| Q2-B21 | `src/intelligence/auto_consolidate.rs:225,401` | `persist_report` fails | `let _ = persist_report(..)`: consolidation report is lost silently. | HYPOTHESIS. |

### P3

| ID | Location | Candidate | Status |
|---|---|---|---|
| Q2-B15 | `src/storage/lock.rs:79-87` (C2-adjacent, listing only) | `flock` return `!= 0` is always reported as "already locked by another process". `EWOULDBLOCK` is contention, but `EINTR`, `ENOLCK` or `EOPNOTSUPP` (filesystems without flock) are mislabeled and `EINTR` is not retried. Holder-metadata writes (`set_len`, `write!`, `flush`, lines 93-98) are ignored: acceptable best-effort diagnostics, but a failed write leaves "another process" with no pid. | HYPOTHESIS (needs a filesystem without flock). |
| Q2-B16 | `benches/search.rs:359` | `unsafe` block without a `SAFETY:` comment (section 5). Documentation fix only. | Confirmed (static). |
| Q2-B17 | `src/auth/tokens.rs:119` (`validate_key`) | `&raw_key[..12]` after `len() < 12`: a non-ASCII char spanning byte 12 panics. No non-test caller in `src/` (`git grep`), so latent public API; becomes P1 if wired into HTTP auth. `create_api_key:73` slices a generated ASCII key (safe). | HYPOTHESIS. **FIXED** in `a6f55ac` (C3). |
| Q2-B18 | `src/intelligence/auto_capture.rs:405,414` | `extracted[..end]`, `text[..max_len]` on arbitrary text; `AutoCaptureEngine` is public but no MCP handler uses it (`git grep`). Latent. | HYPOTHESIS. **FIXED** in `a6f55ac` (C3). |
| Q2-B19 | `src/integrations/langfuse.rs:108` (`LangfuseConfig::from_env`) | `.expect("... failed security validation")`: a bad `LANGFUSE_BASE_URL` aborts startup with a panic (documented), instead of a typed error. | Confirmed (static). |
| Q2-B20 | `src/embedding/mod.rs:96`, `src/portability/markdown.rs:128`, `src/bin/server.rs:297,685,888,939`, `src/bin/watcher.rs:186,341` | Runtime/HTTP-client construction `expect`s. Also `Runtime::new()` created inside a library constructor can panic when the owning value is dropped inside an async context. Prefer a typed startup error. | HYPOTHESIS. |
| Q2-B22 | `src/storage/queries/core/row.rs:22,52,55,86,89`, `queries/sync.rs:161,408,537` | Decode fallbacks `unwrap_or_else(\|_\| ...)` turn bad data into valid values: unknown tier becomes `"permanent"`, unparsable timestamps become `Utc::now()` (a corrupt `created_at` makes a memory look fresh; sync ordering uses these). 106 production `err_default_closure` sites; most are benign defaults and need triage per site. | HYPOTHESIS. |
| Q2-B23 | `src/storage/turso_backend/impls_crud.rs:97,104,323,331`, `core.rs:352-388` (feature `turso`, non-default) | Tag insert/link and index creation use `.await.ok()`; tag-link failures are silent. | HYPOTHESIS. |
| Q2-B24 | `src/hooks/mod.rs:113,124` | Hook errors go only to stderr and `trigger_and_forget` discards the result. Intentional fire-and-forget; documenting it is enough. | Confirmed (static). |
| Q2-B25 | 9 sites using `query_row(..).ok()` as "not found": `storage/queries/core/dedup.rs:42` (`find_by_content_hash`), `entity_queries.rs:60`, `identity_links.rs:297,418`, `clustering.rs:223`, `queries/sync.rs:233`, `queries/core/memory_delete.rs:14`, `intelligence/session_indexing.rs:437`, `mcp/handlers/context/facts.rs:30`; and 7 `ctx.embedder.embed(..).ok()` in `mcp/handlers/{search.rs:127,702, context/assembly.rs:89,291,436}`, `bin/cli/{core.rs:87, interactive.rs:121}` | `query_row(..).ok()` turns any error (lock, I/O, schema) into "row absent": `dedup.rs:42` would then insert a duplicate, `session_indexing.rs:437` would treat an existing session as new. The idiom is `.optional()?`. A failing embedder silently degrades search to lexical-only with no signal in the response (may be intended graceful degradation; needs a documented contract). | HYPOTHESIS |

### Fixed outside the numbered backlog (Q2F, 2026-10-05)

| Item | Commit | Note |
|---|---|---|
| G-3 `memory_search` result cache ignored `limit` (and `min_score`, `workspaces`, `strategy`, `scope`, `scope_path`, `filter`) | `98103b0` | Cache key now carries every result-shaping option; `contract_matrix::memory_search_limit_survives_the_result_cache` un-ignored. |
| C3 follow-up: `ttl_seconds` overflow (`chrono::Duration::seconds`, `DateTime + Duration`) in `memory_create`/`memory_update`/`set_memory_expiration`/`acquire_dream_lock` | `175c977` | `expiry_after` + `MAX_TTL_SECONDS` (100 years); typed `InvalidInput`. |
| C3 follow-up: `content_utils::soft_trim` skipped one byte past a multibyte whitespace (NBSP, U+3000) and panicked | `1e4af34` | `context/bundle.rs::truncate` was already safe and now reuses `text_util::truncate_bytes`. |
| `NaturalLanguageParser` "last N days/weeks/months" overflow | `5100634` | Out-of-range lookback yields no date filter. |

### Verified safe or acceptable (do not "fix" by count)

- `let _ = ...` production (93 sites; heuristic classes: 31 db/state writes,
  26 teardown I/O, 15 other, 14 unused-binding silencers, 7 channel sends).
  Read in full: the following are legitimate best-effort or already reported:
  bench cleanup (`remove_file`), child `kill`/`wait`/`join`
  (`intelligence/pdf_worker.rs`), websocket close timeout, shutdown signals,
  `ROLLBACK` after an error (`mcp/handlers/quality.rs:345`), channel sends to a
  closed receiver, lock-file metadata, final HNSW checkpoint at shutdown
  (`bin/server.rs:984`) and warm-up at startup (`:274`, could log), emitters
  already named `emit_best_effort`, `log_audit(..).map_err(|e| warn!(..))`
  in `storage/operational_context.rs:641` (logged), `let _ = f(..)?` forms where
  only the value is discarded. Bucket "teardown best-effort" is distinct from
  "business error" and is not backlog.
- `.ok();` as a discarded statement: 16 production sites. 5 wrap an
  `emit_best_effort(..)` closure that always returns `Ok`
  (`mcp/handlers/{autonomous.rs:238,274, lifecycle.rs:576, summarize.rs:120,
  misc/auto_tag.rs:199}`), 11 are Turso `.await.ok()` (Q2-B23). `.ok()` bound to
  a variable (`ok_bound`, 30 production sites) converts every error to `None`;
  the consequential ones are listed in Q2-B25.
- `Err(_) => {}` / `Err(_) => ()`: 0 occurrences.
- `src/mcp/protocol.rs`: response writes use `?`; only progress notifications
  use best-effort writes.

## 8. Fix contract

For each backlog item that is later fixed, in its own patch (rollback = revert
that patch):

1. **Input**: a test that reproduces the failure with the input listed above
   (for the Q2-B01..B06 class, a string whose multibyte char straddles the
   cut index, plus the ASCII control).
2. **Assertion**: typed error or no panic, e.g. `assert!(result.is_ok())` /
   `matches!(err, EngramError::InvalidInput(_))`, and for swallowed errors an
   assertion that the error is surfaced (or counted/logged by contract), never
   only "does not crash".
3. **Focused test + required Clippy**: run the focused test with
   `scripts/ci-required-features.env` features and
   `cargo clippy --all-targets --all-features -- -D warnings`.
4. Do not change wire formats of MCP responses/errors outside the minimal fix;
   record the exact error shape in the test.

Out of scope here: no source file under `src/`, `benches/` or `tests/` was
modified by this task. Items in `src/sync/*`, `src/storage/connection.rs` and
`src/storage/migrations/*` (Q2-B13, Q2-B15) are listed for later and must not
be changed outside the C2 review.

## Appendix A. Reproduction recipes

All commands assume the repository root and a debug build
(`cargo build --no-default-features --features "$CI_REQUIRED_FEATURES"` after
`source scripts/ci-required-features.env`; `attestation` tools need a binary
with that feature enabled; the pre-built binary used here had it).

**MCP stdio (Q2-B01..B05, B08).** Generate the requests and pipe them:

```python
# python3 gen.py <scenario> > in.jsonl
import json, sys
init = {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"q2","version":"0"}}}
S = {  # scenario -> list of (tool, arguments)
 "B01": [("memory_create", {"content": "hello world "*6 + "x" + "é"*20}), ("memory_search_compact", {"query": "hello"})],
 "B02": [("memory_create", {"content": "hello x" + "é"*300}), ("memory_prepare_context", {"query": "hello"})],
 "B03": [("memory_create", {"content": "hello x" + "é"*3000}), ("memory_garden", {"dry_run": True})],
 "B04": [("memory_create", {"content": "x" + "é"*60}), ("context_budget_check", {"memory_ids": [1], "model": "gpt-4", "budget": 1000})],
 "B05": [("attestation_chain_verify", {"verifying_key": "abc"})],
 "B08": [("session_index", {"session_id": "s1", "max_chars": 5, "messages": [{"role": "user", "content": "hello "*40, "id": "m1"}]})],
}[sys.argv[1]]
print(json.dumps(init))
for i, (name, args) in enumerate(S, start=2):
    print(json.dumps({"jsonrpc":"2.0","id":i,"method":"tools/call","params":{"name":name,"arguments":args}}))
```

```bash
D=$(mktemp -d)
python3 gen.py B01 > $D/in.jsonl
env -i HOME=$D PATH=/usr/bin:/bin ENGRAM_DB_PATH=$D/m.db RUST_BACKTRACE=0 \
  target/debug/engram-server --transport stdio --embedding-model tfidf < $D/in.jsonl >/dev/null
echo "exit=$?"     # 101 plus a 'panicked at ...' line on stderr; with ASCII-only content the same call succeeds
```

Observed at this commit: B01 `search.rs:732:53`, B02 `context_grouper.rs:118:55`,
B03 `gardening.rs:316:31`, B04 `compression.rs:269:38`, B05
`attestation.rs:235:45`, B08 `session_indexing.rs:195:21`; all exit 101.

**CLI config clobber (Q2-B07).** With `HOME=$D/home`, create
`$D/home/Library/Application Support/Claude/claude_desktop_config.json`
containing `{"mcpServers":{"other-server":{"command":"node"},}}` (trailing
comma), run `engram-cli --db-path $D/x.db mcp install --client claude` twice
from `$D`; the file ends with only the `engram` server, the first `.bak` still
holds the original, the second run overwrites `.bak`.

**Library API (Q2-B09, B10).** A scratch crate outside the repository
(`engram-core = { path = "<worktree>", default-features = false }`, the repo
`Cargo.lock` copied for offline resolution) with:

```rust
let b = SqliteBackend::in_memory().unwrap();
b.storage().with_connection(|c| { c.execute_batch("DROP TABLE crossrefs;")?; Ok(()) }).unwrap();
// queries::delete_crossref(c,1,2,EdgeType::RelatedTo) -> Err("no such table: crossrefs")
// b.delete_crossref(1, 2)                              -> Ok(())            (Q2-B09)
let id = b.storage().with_connection(|c| Ok(create_memory(c, &CreateMemoryInput {
    content: "x".into(), ..Default::default() })?.id)).unwrap();
b.storage().with_connection(|c| { c.execute("UPDATE memories SET importance='not-a-number' WHERE id=?1", [id])?; Ok(()) }).unwrap();
// list_memories(c, &ListOptions::default()) -> Ok(0 rows); SELECT COUNT(*) FROM memories -> 1   (Q2-B10)
```

**Q2-B06** was reproduced by compiling a verbatim copy of
`snapshot.rs::parse_hex_key` with `rustc` and calling it with
`"0é" + "0"*61` (64 bytes): `end byte index 2 is not a char boundary`.

## Not run / pending

- No release (`panic = "abort"`) binary was run; panics were observed as exit
  code 101 in debug builds. The abort consequence in release follows from
  `Cargo.toml`, not from a run.
- The HTTP transport was not exercised (stdio only). The same handlers are
  dispatched by it.
- Linux was not run locally; the inventory is platform-independent for the
  syntactic part, but `#[cfg(target_os = ...)]` branches that are inactive on
  macOS are classified by their predicates, not by compilation.
- `snapshot_*` handler path for Q2-B06, `compact()` failure injection (Q2-B13),
  filesystems without `flock` (Q2-B15), and the Turso backend (Q2-B23) were
  not run.
