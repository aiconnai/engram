# RustSec `2026-12-31` Advisory-Exception Remediation Plan (Frente 3)

- **Date:** `2026-11-19`
- **Baseline:** `origin/main` `531a80121d13621dff44a7899b1af6360e60b4eb` (`engram-core` `0.24.1` prepared)
- **Hard Deadline:** `2026-12-31` (`42` days) — all three governed records in [`advisory-exceptions.toml`](../../security/advisory-exceptions.toml) expire on `2026-12-31` and block `Security exception policy` + `Security Gate` on `2027-01-01`.

---

## 1. Empirical Baseline (`2026-11-19` Live Verification)

Every dependency path and upstream crate version below was verified directly via `cargo tree --workspace --all-features -i <crate>`, the RustSec `advisory-db`, and the `crates.io` v1 API on `2026-11-19`:

| Advisory | Locked Crate | Kind | Graph Scope | Call Sites in Workspace | Upstream Reality (`crates.io` `2026-11-19`) | Can Be Eliminated From `Cargo.lock` Today? |
|---|---|---|---|---|---|---|
| `RUSTSEC-2026-0192` | `ttf-parser 0.25.1` | `unmaintained` (`cargo-audit:allowed-warning`) | `pdf` feature only (`default_graph = false`; enabled in `CI_REQUIRED_FEATURES`) | **1 call site** (`3` lines of `pdf_extract` in `src/intelligence/document_ingest.rs:660-663`) | `pdf-extract 0.12.1` still pins `lopdf ^0.42`, **but `lopdf 0.47.0` already dropped `ttf-parser` completely** and exposes `Document::extract_text(&[page_number])` | **Yes — 100% removable in Wave 1 (`v0.24.x`)** |
| `RUSTSEC-2024-0436` | `paste 1.0.15` | `unmaintained` proc-macro (`cargo-audit:ignore`) | `onnx-embed` / `local-embeddings` optional feature only (`default_graph = false`; not in `CI_REQUIRED_FEATURES`) | **1 file** (`src/embedding/onnx.rs` via `tokenizers 0.23.1`) | HuggingFace `tokenizers 1.0.0-rc.4` (`tk-convert`, `tk-encode`, `tk-serialize`, `tk-train`) **removed both `paste` and `macro_rules_attribute`** (`0.23.2` still depends on both) | **Yes — 100% removable in Wave 2 via `tokenizers 1.0` (stable or RC alongside `ort 2.0.0-rc`)** |
| `RUSTSEC-2026-0543` | `chrono 0.4.43` | `unmaintained` (`cargo-deny:ignore`, `patched = []` across all versions including `0.4.45`) | **Default graph** (`default_graph = true`) + optional `duckdb-graph` | **`195` `.rs` files / `340` occurrences**, including public struct fields in `engram-types` (`20` sites) and `engram-core` (`19908`-line public API baseline) | Recommended replacements: `jiff 1.0.1` or `time 0.3.47`. **Critical upstream blocker:** even `duckdb 1.20000.2` → `arrow 60.0.0` → `arrow-array 60.0.0` retains a **mandatory (`optional = false`) normal dependency on `chrono ^0.4.40`** | **Default graph: Yes in `v0.25.0` (semver-minor API wave). Full `Cargo.lock` removal: Blocked upstream by Apache `arrow-array` while `duckdb` remains in workspace** |

---

## 2. Sequenced Execution Waves

### Wave 0 (Completed in This PR) — Horizon-Test Anchor Hygiene
- **Problem:** `scripts/test_check_security_exceptions.py` hardcoded `TODAY = date(2026, 10, 5)`, which caps `expires <= 2027-01-03` inside the `Security Gate` CI job (`.github/workflows/ci.yml:527`, transitively blocking required `Test (ubuntu-latest)`) even when a valid renewal (`<= 90` days from evaluation day) passes `check-security-exceptions.py`.
- **Fix landed here:** `scripts/check-security-exceptions.py` now parses and enforces `[metadata].as_of` (`as_of <= today` and `as_of <= expires <= as_of + 90 days`), `[metadata].as_of` is updated to `2026-11-19`, and `scripts/test_check_security_exceptions.py` derives its deterministic review anchor directly from `parse_as_of(CONFIG)`.

---

### Wave 1 (`W1-PDF`, Target: Pre-`2026-12-10`, Patch-Compatible `v0.24.x`) — Eliminate `RUSTSEC-2026-0192` (`ttf-parser`)

- **Goal:** Remove `pdf-extract` and `ttf-parser` from `Cargo.toml` and `Cargo.lock`, shrinking governed exceptions from `3` to `2` with zero public API, MCP, or schema changes.
- **Concrete Changes:**
  1. `Cargo.toml`: remove `pdf-extract = { version = "0.12", optional = true }` and `"dep:pdf-extract"` from the `pdf` feature; upgrade `lopdf` from `0.42` to `0.47` (`default-features = false`, preserving WASM/minimal feature surface; verify transitive `thiserror 2` / `nom 8` / `indexmap 2.14` lockfile diff).
  2. `src/intelligence/document_ingest.rs:636-675`: replace `pdf_extract::PlainTextOutput` + `pdf_extract::output_doc_page(&document, &mut output, *page_number)` with `document.extract_text(&[*page_number])`, feeding the returned string through the existing `BoundedPdfText` / `limits.max_text_bytes` / `limits.max_pages` / `catch_unwind` bounds and keeping `engram-pdf-worker` (`RLIMIT_AS`, `RLIMIT_CPU`, `RLIMIT_FSIZE = 0`) unchanged.
  3. `docs/security/advisory-exceptions.toml` & `scripts/check-security-exceptions.py`: delete the `RUSTSEC-2026-0192` record and update `check-security-exceptions.py` (`PDF_ADVISORY`) so that instead of requiring `len(pdf_records) == 1`, it verifies `name = "ttf-parser"` is absent from `Cargo.lock` whenever `RUSTSEC-2026-0192` is absent from `advisory-exceptions.toml` (fail-closed if `ttf-parser` ever re-enters `Cargo.lock` without a governed record).
- **Acceptance & Verification Gates:**
  - `cargo tree --workspace --all-features -i ttf-parser` exits non-zero (absent).
  - Existing PDF unit, malformed-PDF, encrypted-PDF, byte-cap, and `engram-pdf-worker` packaging/sandbox tests (`test_check_pdf_worker_packaging.py` + integration suite) pass.
  - `check-advertised-contracts.py` (`19908` public API lines unchanged), `cargo audit`, `cargo deny`, and `check-security-exceptions.py` pass (`2 governed records`).

---

### Wave 2 (`W2-ONNX`, Target: Pre-`2026-12-15`, Patch-Compatible) — Eliminate `RUSTSEC-2024-0436` (`paste`)

- **Goal:** Remove `paste 1.0.15` and `macro_rules_attribute` from `Cargo.lock` and empty `.cargo/audit.toml` (`ignore = []`), shrinking governed exceptions from `2` to `1`.
- **Concrete Changes:**
  1. Check crates.io for stable `tokenizers 1.0.0`; if still in RC (`1.0.0-rc.4+`), evaluate adopting `tokenizers = { version = "=1.0.0-rc.4", optional = true, default-features = false, features = ["bpe", "wordpiece", "unigram", "normalizers", "serialize"] }` (matching our existing pinned RC policy on `ort = "2.0.0-rc.12"` in the same `onnx-embed` feature!).
  2. Update `src/embedding/onnx.rs` (sole consumer in the workspace) for any `Tokenizer::from_file` / `encode` API adjustments and verify against real local `tokenizer.json` fixtures (`--features local-embeddings`).
  3. Remove `"RUSTSEC-2024-0436"` from `.cargo/audit.toml` (`ignore = []`) and `docs/security/advisory-exceptions.toml`.
- **Fallback (if `tokenizers 1.0` exhibits regression on `tokenizer.json` compatibility):** Keep `tokenizers 0.23.1` isolated in optional `onnx-embed` (`default_graph = false`) and renew `RUSTSEC-2024-0436` at the `2026-12-20` checkpoint with dated upstream evidence per `GATES.md`.
- **Acceptance & Verification Gates:**
  - `cargo tree --workspace --all-features -i paste` exits non-zero (absent).
  - `cargo test --locked --features local-embeddings onnx` + `cargo clippy --all-targets --features local-embeddings --locked -- -D warnings` pass.
  - `cargo audit` passes with **zero** `ignore` entries in `.cargo/audit.toml`.

---

### Wave 3 (`W3-CHRONO`, Three Sub-Phases) — `RUSTSEC-2026-0543` (`chrono`)

Because `chrono` sits in **two distinct layers** — (1) Engram's own public/internal Rust types (`default_graph = true`) and (2) Apache `arrow-array`'s non-optional dependency inside optional `duckdb-graph` — conflating them into one big-bang PR would either break semver or fail to remove `chrono` from `Cargo.lock`. Split into three explicit sub-phases:

#### Phase `W3A` — Immediate Feature Hygiene & Internal Time Seam (Pre-`2026-12-18`, Non-Breaking `v0.24.x`)
1. **Trim `chrono` default features:** change `Cargo.toml` and `crates/engram-types/Cargo.toml` from `chrono = { version = "0.4", features = ["serde"] }` (which enables `oldtime` and `wasmbind` -> `wasm-bindgen`/`js-sys`!) to `chrono = { version = "0.4", default-features = false, features = ["std", "clock", "serde"] }`.
2. **Audit `duckdb` feature list (`Cargo.toml:302`):** verify whether `src/graph/` uses `duckdb`'s `"chrono"` feature directly; if not, trim `duckdb` to `features = ["bundled"]` so only `arrow-array` retains the transitive link.
3. **Introduce `src/time_util.rs` (or `engram_types::time`):** add `jiff = { version = "1", features = ["serde"] }` (MIT/Apache-2.0, maintained by BurntSushi, RustSec-recommended replacement) and golden compatibility tests proving byte-for-byte / instant-equivalent RFC3339 parsing and formatting against existing SQLite `TEXT` timestamps (`+00:00` / `Z`, fractional nanoseconds, `CURRENT_TIMESTAMP` `"YYYY-MM-DD HH:MM:SS"` fallback!).
   > **Critical Storage Pitfall to Test:** SQLite `DEFAULT CURRENT_TIMESTAMP` writes `"YYYY-MM-DD HH:MM:SS"` (space separator, no offset), whereas `to_rfc3339()` writes `"YYYY-MM-DDTHH:MM:SS.ffffff+00:00"` or `"...Z"`. Every parser seam in `src/storage/queries/core/row.rs` MUST preserve both formats with property + golden tests before migrating types!

#### Phase `W3B` — Migrate Direct Workspace Usage Off `chrono` (`v0.25.0` Minor Release)
1. Migrate `crates/engram-types` (`Memory`, `CrossRef`, `Stats`, `Config` — `20` sites) from `chrono::DateTime<Utc>` to `jiff::Timestamp` (with custom/standard RFC3339 `serde` ensuring **zero JSON wire change** for MCP tools, HTTP REST, Python SDK, and TypeScript SDK).
2. Migrate `engram-core` internal modules by vertical slice (`storage` -> `search`/`context` -> `intelligence`/`dream` -> `mcp`/`auth`/`sync`/`bin`) and update `docs/contracts/advertised-surfaces.toml` for the intentional `v0.25.0` Rust API bump.
3. Remove direct `chrono` dependencies from `Cargo.toml`, `crates/engram-types/Cargo.toml`, and `fuzz/Cargo.toml`.
4. **Result:** `cargo tree --no-default-features` and `cargo tree` (default + `CI_REQUIRED_FEATURES`) contain **zero `chrono`**! `RUSTSEC-2026-0543` in `advisory-exceptions.toml` flips to `default_graph = false`, `feature_gated = true` (`duckdb-graph` optional analytics only).

#### Phase `W3C` — Pre-`2026-12-31` Governance Checkpoint (Target: `2026-12-20`)
- Re-check upstream `arrow-rs` / `duckdb` for `chrono` removal (or optional `duckdb-graph` workspace separation if desired).
- For any exception still blocked upstream (`arrow-array -> chrono`, and `W3B` direct `chrono` if `v0.25.0` is scheduled for Q1 2027), perform a fresh human-approved review updating `[metadata].as_of` and `expires` (`<= 90` days) in `docs/security/advisory-exceptions.toml` — compliant with `GATES.md` because `RUSTSEC-2026-0543` has `patched = []` (no semver-compatible fix exists).

---

## 3. Recommended Execution Order & Effort Summary

| Step | Scope | Breaks Rust Public API? | Eliminates Advisory Record? | Estimated Size |
|---|---|---|---|---|
| **Wave 0 (Done here)** | `metadata.as_of` anchor in `check-security-exceptions.py` + `test_check_security_exceptions.py` | No | Prevents CI horizon test trap | Small (`3` files) |
| **Wave 1 (`W1-PDF`)** | `lopdf 0.42 -> 0.47`, drop `pdf-extract` (`3` lines in `document_ingest.rs`) | No (`v0.24.x` safe) | **Yes — deletes `RUSTSEC-2026-0192` (`ttf-parser`)** | Small–Medium (`~4` files + lockfile) |
| **Wave 2 (`W2-ONNX`)** | `tokenizers 0.23 -> 1.0` in `src/embedding/onnx.rs` | No (`v0.24.x` safe) | **Yes — deletes `RUSTSEC-2024-0436` (`paste`), empties `.cargo/audit.toml`** | Small (`~4` files + lockfile) |
| **Wave 3A (`W3A`)** | Trim `chrono` `oldtime`/`wasmbind` features + RFC3339/SQLite `CURRENT_TIMESTAMP` oracle tests | No (`v0.24.x` safe) | Reduces attack/build surface & de-risks `W3B` | Small (`~4` files) |
| **Wave 3B (`W3B`)** | Migrate `engram-types` + `engram-core` off `chrono` to `jiff 1` | **Yes (`v0.25.0` Rust API; 0 MCP/SDK/SQLite wire change)** | Moves `RUSTSEC-2026-0543` out of default graph (`duckdb-graph` only) | Large (multi-PR wave across `195` `.rs` files) |
| **Wave 3C (`2026-12-20`)** | Pre-expiry review of upstream `arrow-array -> chrono` | No | Keeps `Security Gate` green across `2027-01-01` | Small (`1` governance PR) |
