# C4 spike: descriptor-bound SQLite open (evidence, not product code)

This directory is the evidence for
[`../../2026-10-05-c4-descriptor-bound-sqlite-open.md`](../../2026-10-05-c4-descriptor-bound-sqlite-open.md).
Run results and their interpretation are in
[`../../2026-10-05-c4-descriptor-bound-sqlite-open-spike-notes.md`](../../2026-10-05-c4-descriptor-bound-sqlite-open-spike-notes.md).

## Isolation from Engram and CI

- `c4spike` is a standalone Cargo package. Its own empty `[workspace]` table makes it its own
  workspace root. It is **not** a member of Engram's workspace (`members = [".",
  "crates/engram-types", "engram-wasm"]`), so `cargo build`, `cargo test`, `cargo fmt --all`
  and `cargo clippy` at the repository root never build or check it.
- **CI does not build or test it.** No workflow references this path. CodeQL ignores
  `docs/**`, and this directory is listed in `.semgrepignore`.
- `scripts/rust_risk_inventory.py` (git mode, not a CI gate) lists every tracked `*.rs` file,
  so a regenerated inventory will show this crate. That is expected and harmless.
- The spike uses throwaway synthetic databases under `$TMPDIR`. It never touches a real
  Engram database.

## Layout

| Path | What it is |
|---|---|
| `src/shim.rs` | Option B: pinned parent-directory fd plus the unix-VFS syscall shim (`bind`, `open`, the VFS `c4-pinned`) |
| `src/trust.rs` | Option E: `verify_trusted_chain`, `verify_db_artifacts`, `verify_open_artifact` |
| `src/util.rs` | Temp dirs, sentinel databases, the peer-process helper |
| `src/bin/peer.rs` | Second process: opens by pathname through the stock `unix` VFS |
| `tests/*.rs` | 8 test files (see the notes for the inventory) |
| `stub/shim_stub.rs` | The pathname-only stub used for the RED run of `tests/shim.rs` |
| `run-linux.sh` | Offline Linux runner for `rust:1.98-bookworm` (unprivileged uid 1000) |
| `soak.sh` | F7 soak: modes `stock`, `shim`, `shim-no-retry` |
| `multiuid-demo.sh` | Two-uid rename demo for Option E's premise (run as root in a container) |
| `evidence/` | Retained outputs from the 2026-10-05 runs (renamed `*.txt`, because the repository ignores `*.log` and `logs/`) |

## How to run

You need Rust 1.96 or newer. The crates are pinned to `rusqlite =0.31.0`, `libc =0.2.180`,
and the bundled SQLite 3.45.0. Use a private `TMPDIR` (mode `0700`).

**macOS (or any host):**

```bash
cd docs/decisions/assets/2026-10-05-c4-spike
mkdir -p tmp && chmod 700 tmp
TMPDIR=$PWD/tmp cargo test --offline --no-fail-fast          # 35 tests
cargo test --offline --no-run && RUNS=50 TMPDIR=$PWD/tmp ./soak.sh   # F7 soak
```

**Linux (offline docker, uid 1000):**

```bash
cd docs/decisions/assets/2026-10-05-c4-spike
cargo vendor --offline vendor          # vendor/ is not committed (about 41 MB)
docker run --rm --network none -e STRESS_RUNS=50 -v "$PWD":/spike \
  rust:1.98-bookworm /spike/run-linux.sh
# outputs: linux.log, soak-linux.log, linux-env.txt in this directory
docker run --rm --network none -v "$PWD":/spike rust:1.98-bookworm /spike/multiuid-demo.sh
```

**RED reproduction:**

```bash
cp src/shim.rs /tmp/shim_impl.rs && cp stub/shim_stub.rs src/shim.rs
TMPDIR=$PWD/tmp cargo test --offline --test shim     # expect "attacker" vs "original" failures
cp /tmp/shim_impl.rs src/shim.rs
```

The RED runs of `trust.rs` used two stubs, `verify_trusted_chain` and the artifact checks,
that each returned `Ok(())` unconditionally.

**Environment switches:**

- `C4_TRACE=1` prints every shim sidecar `open`/`stat`/`unlink`, and the stock observer in
  `tests/stress_control.rs`.
- `C4_DISABLE_CREATE_RETRY=1` turns off the shim's `O_CREAT` ENOENT retry, which reproduces
  F7.

`target/`, `vendor/` and `tmp/` are generated and must not be committed.
