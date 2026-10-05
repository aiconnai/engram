# Task C3 report — Unicode and adverse parsers (Lane R)

Commit: `a6f55ac fix(search): make text truncation and hex parsing char-boundary safe` (local only, branch claude/engram-improvement-plan-edb43d, on top of 6750b9b).
Note: the RTK hook rewrote my first `git commit -m` subject to "...: rtk make ..."; I amended it with `-F` so the final subject is clean.

## What was implemented
Shared helper `src/text_util.rs` (`floor_char_boundary`, `ceil_char_boundary`, `truncate_bytes`, `suffix_bytes`; `pub mod text_util` in lib.rs). The private `safe_truncate` in `mcp/handlers/context/mod.rs` now delegates to it.

Fixes (all inventory IDs reproduced RED first):
- Q2-B01 `search.rs` title `&first_line[..80]` -> `truncate_bytes`.
- Q2-B02 `context_grouper.rs` `&combined[..500]`; B03 `gardening.rs` (4096, also dry-run path); B04 `compression.rs` (preview 50).
- Q2-B05/B06 attestation + snapshot `parse_hex_key`: `hex::decode` (odd length, non-ASCII and `+f` sign chars -> typed `{"error": ...}`; previously `u8::from_str_radix` also accepted `+f`). Snapshot keeps its 64-byte precheck message.
- Q2-B08 `session_indexing::truncate_with_marker`: no underflow; contract is now `result.len() <= max_chars` BYTES (head/tail cut on char boundaries; below marker size = plain prefix). `session_index` `ttl_days` validated to 0..=36500 with typed error (see deviations).
- `bm25::generate_highlights`: lowercase->original map per char (`lowercase_with_origin`); match start/end taken in original coordinates, window edges floored/ceiled to char boundaries. Brief reproducer pinned at function level.
- Same class from inventory: Q2-B14 `memory_blocks::archive_overflow` (also `saturating_mul`), Q2-B17 `auth/tokens.rs validate_key` (`raw_key.get(..12)`), Q2-B18 `auto_capture::extract_content` (ASCII lowercase keeps offsets + boundary-safe 500 cut).
- Same bug class found by grep, NOT in inventory: `smart_retrieve::strip_intent_markers` (`to_lowercase()` offset used on original; `İİİİ related to engram` returned "am", other inputs panic) and `dream::worker::extract_procedural_lesson` (char-count mapping gave wrong snippet start). Both now `to_ascii_lowercase()` (markers/keywords are ASCII).

## Files changed (22)
src/text_util.rs (new), src/lib.rs, src/search/bm25.rs, src/mcp/handlers/{search,attestation,snapshot,session,smart_retrieve}.rs, src/mcp/handlers/context/mod.rs, src/intelligence/{context_grouper,gardening,compression,session_indexing,auto_capture}.rs, src/dream/worker.rs, src/storage/memory_blocks.rs, src/auth/tokens.rs, tests/{unicode_adverse_parsers_tests.rs (new), property_tests.rs, property_tests.proptest-regressions (new), aaak_compression_tests.rs}, docs/harness/progress/2026-10-05-improvement-lane-r.md (append).
None of the C1 files were touched.

## TDD evidence
RED (before fixes), `source scripts/ci-required-features.env; cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --test unicode_adverse_parsers_tests`: 9 failed / 2 passed (controls), real panics at the reproduced lines:
- search.rs:732:53 `end byte index 80 is not a char boundary; it is inside 'é' (bytes 79..81)`
- context_grouper.rs:118:55 (500), gardening.rs:316:31 (4096), compression.rs:269:38 (50)
- attestation.rs:235:45 `end byte index 4 is out of bounds for string of length 3` (both attestation_chain_verify and attestation_log)
- snapshot.rs:17:45 `end byte index 2 is not a char boundary; it is inside 'é'` (through `snapshot_load decrypt_key`, so B06 is now reproduced at handler level)
- session_indexing.rs:195:21 `attempt to subtract with overflow`; session.rs:57:30 `attempt to multiply with overflow` (ttl_days = i64::MAX)
BM25: lib unit test panicked bm25.rs:429 `end byte index 102 is not a char boundary; it is inside 'ẞ'`; dispatch-level `bm25_search(.., explain=true)` panicked bm25.rs:429:38 likewise (content uses " target" with a space because FTS5 matches whole tokens; the brief's exact no-space string is the unit test).
Latent B14/B17/B18 and smart_retrieve/dream: tests written first, fixes then temporarily reverted to capture RED (B14 memory_blocks.rs:241 not a char boundary; B17 tokens.rs:121; B18 wrong content "Rust"; smart_retrieve left "am" right "engram"; dream left "tion: restart ..." right "solution: ...").
Property tests: mutation-checked. Restoring the old `generate_highlights` made `bm25_highlight_is_a_slice_of_the_original` fail (panic at bm25.rs:430, shrunk input persisted); replacing `truncate_bytes` by a raw byte slice made `truncate_bytes_contract` fail (shrunk input persisted). Both seeds are in `tests/property_tests.proptest-regressions` and replay on every run; they pass on the fixed code.
GREEN: same commands -> `unicode_adverse_parsers_tests` 12/12, `property_tests` 33/33 (11 new in `unicode_adverse_tests`, fixed seed 0xC3C3_2026_1005, cases 24..512), `aaak_compression_tests` 4/4, `--lib bm25` 20/20, unit tests for B14/B17/B18/smart_retrieve/dream pass.

## Full verification
- `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests --no-fail-fast`: exit 0, 2056 passed, 0 failed, 1 ignored, 53 binaries. (A first full run had 2 failures in `tests/mcp_protocol_tests/contract_matrix.rs`, an untracked WIP file from another concurrent task; unrelated to my files and green on rerun.)
- `cargo clippy --no-default-features --features "$CI_REQUIRED_FEATURES" --all-targets -- -D warnings`: clean (fixed one `manual_repeat_n` in my code). The pre-commit hook (fmt + `clippy --all-targets --all-features -D warnings`) passed on commit.
- `cargo fmt -- --check`: clean. Pre-commit was blocked for a few minutes by unformatted untracked files of another concurrent task (`tests/canonical_journey/`, `tests/mcp_protocol_tests/`); I did not touch them and retried once they were formatted.

## Deviations / decisions
- New test file `tests/unicode_adverse_parsers_tests.rs` and new `src/text_util.rs` (brief's file list did not name them; needed for dispatch-level REDs and the shared helper).
- `ttl_days` is now rejected outside 0..=36500 (typed error). A saturating multiply alone was insufficient: chrono `TimeDelta::seconds` panics downstream (`memory_create.rs:100`). Behavior change: negative `ttl_days` used to be accepted. The same chrono hazard exists for `memory_create ttl_seconds` and other `Duration::seconds(ttl)` sites in storage (listed below as not fixed).
- `truncate_with_marker` contract tightened from "chars" to bytes (the surrounding code compares `str::len`); ASCII behavior unchanged, existing tests pass.
- auto_capture / smart_retrieve / dream worker now use `to_ascii_lowercase()`: behavior differs only for non-ASCII chars whose lowercase is ASCII (e.g. Kelvin sign), irrelevant to ASCII markers.
- FTS5-level property: errors from `bm25_search` on exotic input are not treated as failures (only panics and Ok-result contracts are).

## Not done / concerns
- Not fixed (outside this brief): `Duration::seconds(ttl)` panics for huge `ttl_seconds` in `storage/queries/core/{memory_create,memory_update,expiration,dream}.rs` (memory_create with ttl_seconds=i64::MAX likely panics; not tested). Suggest a follow-up.
- Out of scope per instructions and untouched: mcp install config overwrite (B07), delete_crossref (B09), list_memories dropping rows (B10).
- Strings with `to_lowercase()` used only for matching (not offsets) elsewhere were left alone.
- `docs/quality/rust-risk-inventory.md` statuses were not updated (Q2 owns it); B01-B06, B08, B14, B17, B18 can now be marked fixed.
- Not run: an all-features test run (only the hook's all-features clippy ran). The attestation/snapshot handlers are compiled under the required feature set (`agent-portability`) and were exercised by the new tests.
