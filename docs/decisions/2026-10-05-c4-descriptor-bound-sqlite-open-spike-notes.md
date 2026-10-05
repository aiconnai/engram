# C4 spike notes: descriptor-bound SQLite open viability tests

Companion to [`2026-10-05-c4-descriptor-bound-sqlite-open.md`](2026-10-05-c4-descriptor-bound-sqlite-open.md).

**Spike code:** [`assets/2026-10-05-c4-spike/`](assets/2026-10-05-c4-spike/README.md). It is
a standalone Cargo package with its own `[workspace]`. It is not an Engram workspace member,
not built or tested by CI, and listed in `.semgrepignore`. Retained outputs are in
`assets/2026-10-05-c4-spike/evidence/`. All databases are synthetic and live under the
spike's `$TMPDIR`; no real Engram database was touched.

## Environment

- **Crate:** `c4spike`, with `rusqlite = "=0.31.0"` (`bundled`: SQLite 3.45.0, the same as
  Engram's `Cargo.lock`) and `libc = "=0.2.180"`. Built `--offline`.
- **macOS:** 27.0.1 (26A434), arm64, rustc/cargo 1.96.0.
- **Linux:** `rust:1.98-bookworm` (cached image), run with `docker run --network none`;
  kernel 7.0.14 (OrbStack), aarch64, glibc 2.36 (`evidence/linux-env.txt`). Crates are
  vendored with `cargo vendor --offline vendor`. Tests run as the **unprivileged uid 1000**
  (`setpriv`), with `TMPDIR` on the container's own filesystem.
- **Second process:** `src/bin/peer.rs` always opens by pathname through the stock `unix`
  VFS, as a CLI, hook or second server would.

## Exact commands

Run from `docs/decisions/assets/2026-10-05-c4-spike/`.

```bash
# macOS: full suite (35 tests), then the 3-mode F7 soak
mkdir -p tmp && chmod 700 tmp
TMPDIR=$PWD/tmp cargo test --offline --no-fail-fast        # -> evidence/green-macos.txt
cargo test --offline --no-run
RUNS=50 TMPDIR=$PWD/tmp ./soak.sh                           # -> evidence/soak-macos.txt
R=$(ls -t target/debug/deps/create_unlink_race-* | grep -v '\.d$' | head -1)
TMPDIR=$PWD/tmp $R --nocapture | grep RACE-RESULT           # -> evidence/probe-macos.txt

# Linux: offline, unprivileged; writes linux.log, soak-linux.log, linux-env.txt here
cargo vendor --offline vendor
docker run --rm --network none -e STRESS_RUNS=50 -v "$PWD":/spike \
  rust:1.98-bookworm /spike/run-linux.sh
# Option E premise with two real uids (container root sets up, uid 1001 attacks)
docker run --rm --network none -v "$PWD":/spike rust:1.98-bookworm /spike/multiuid-demo.sh

# RED for Option B: swap in the pathname-only stub, run, restore
cp src/shim.rs /tmp/shim_impl.rs && cp stub/shim_stub.rs src/shim.rs
TMPDIR=$PWD/tmp cargo test --offline --test shim; cp /tmp/shim_impl.rs src/shim.rs
# Linux RED: same swap, then: docker run ... -e LOG=linux-red.log ... run-linux.sh --test shim
```

**Environment switches:**

- `C4_TRACE=1` traces sidecar opens: `shim_open ... rdonly=N -> fd`, and the stock observer
  `stock_open ... rdonly=N -> fd` in `tests/stress_control.rs`.
- `C4_DISABLE_CREATE_RETRY=1` disables the F7 fix.

## Key code pointers

- **R2 window hook** (`tests/stock_race.rs`):
  - `install_open_hook` replaces the unix VFS `open` syscall through `xSetSystemCall`;
  - `hooked_open` runs the armed swap once, right before the real `open` of the main path;
  - `stock_nofollow_does_not_close_the_window_before_open` arms it: the parent is renamed
    away and replaced with a symlink to the attacker directory.
- **Option B** (`src/shim.rs`):
  - `bind` canonicalizes the parent once and opens it
    `O_RDONLY|O_DIRECTORY|O_NOFOLLOW|O_CLOEXEC`. `verify_dir_fd` then checks the *fd*: owner
    == euid, `mode & 022 == 0`. `create_or_identify_main` either creates the file with
    `O_CREAT|O_EXCL|O_NOFOLLOW` `0600` (fd closed before any SQLite open) or reads it with
    `fstatat(AT_SYMLINK_NOFOLLOW)`, and records `(dev, ino)`.
  - `install` saves the real syscalls, overrides `open`, `stat`, `access`, `unlink` and
    `openDirectory`, and registers `c4-pinned`, a copy of the `unix` VFS struct.
    `pinned_full_pathname` returns bound names verbatim and delegates everything else to the
    real `xFullPathname` (F6).
  - `classify` maps exact `dir/name{,-wal,-shm,-journal}` to the binding.
  - `shim_open` → `openat_create_retrying`: `openat(dirfd, name, flags|O_NOFOLLOW|O_CLOEXEC)`,
    retried up to `RETRIES = 64` while `O_CREAT` gets ENOENT (F7). For the main file it then
    checks `fstat` against `(dev, ino)`; on a mismatch the fd is pushed to `PARKED` (never
    closed: invariant #27, a deliberate leak) and the call fails with `EACCES`.
  - `shim_open_dir` returns `F_DUPFD_CLOEXEC` of the directory fd.
- **Option E** (`src/trust.rs`):
  - `verify_trusted_chain` walks from `/` with `lstat` per component. It refuses symlinks and
    non-directories, and requires each owner to be root or the euid. On the last component it
    requires owner == euid and `mode & 022 == 0`. On ancestors, `mode & 022 != 0` is allowed
    only when the directory is sticky (`0o1000`) **and** the next component is owned by the
    euid or root.
  - `verify_db_artifacts` requires `db` (must exist) and any existing `-wal`, `-shm` and
    `-journal` to be regular, non-symlink, owner == euid, `mode & 077 == 0`.
  - `verify_open_artifact` applies the same rule to an `fstat` of an open handle.
- **Stress workload** (`tests/shim.rs::shim_concurrent_pool_threads_and_peer_processes_keep_integrity`,
  mirrored in `tests/stress_control.rs` on the stock VFS):
  - `THREADS = 4` × `PER_THREAD = 150` inserts, using Engram's local-mode pragmas
    (`engram_local_pragmas`), plus `wal_checkpoint(PASSIVE)` every 50;
  - `PEERS = 2` × `PER_PEER = 40` peer processes, each opening and closing per insert;
  - ends with `integrity_check` and an exact row count.

## Tests (35 per platform; `fd_alias` has platform-specific variants)

| File | Tests | What they show |
|---|---|---|
| `stock_race` (3) | rename swap; symlinked parent vs NOFOLLOW; R2 window hook | R1 and R2 are open today; R2' is closed by NOFOLLOW |
| `fd_alias` (3) | macOS: alias binds main/sidecars fail; name kept verbatim; verbatim-VFS opens main. Linux: refused by NOFOLLOW; resolves back without NOFOLLOW; **verbatim alias + NOFOLLOW stripped (the `98b8446`/`0700cb9` shape) → `CANTOPEN`** | Option A; consistent with the July CI failure |
| `lock_hazard` (2) | extra fd close drops `RESERVED`; dir fd close keeps it | G1 / invariant #27; B is safe to hold a directory fd |
| `alt_vfs` (2) | dotfile/none invisible to a stock peer; dotfile refuses WAL | Option C |
| `shim` (12) | rename swap, sidecars pinned, symlinked/swapped main refused, untrusted dir refused, new DB `0600`, in-process redirection (F5), unbound DBs unaffected, pool+peer+checkpoint, stress, DELETE journal, `VACUUM INTO` | Option B |
| `stress_control` (1) | the same stress workload on the stock VFS | Control |
| `create_unlink_race` (1) | `O_CREAT` vs a concurrent `unlink`, 50k iterations each for `open` and `openat` | F7 kernel probe |
| `trust_chain` (11) | chain: private ok; group-writable, non-sticky world-writable, writable leaf and symlinked component refused; sticky ancestor + owned child ok. Artifacts: owner-only ok; planted `0644` sidecar, `0660` main, symlinked/non-regular sidecar refused; `fstat` re-check refuses a handle that became `0644` | Option E, both parts |

A foreign-*owned* artifact is not covered: an unprivileged test uid cannot create one. The
owner check exists in `check_artifact_metadata` but is untested.

## TDD evidence

| Target | RED | GREEN |
|---|---|---|
| B (`tests/shim.rs` vs `stub/shim_stub.rs`) | macOS 6 failed / 3 passed (9 tests then; `evidence/red-macos-shim.txt`); Linux 7 failed / 4 passed (11 tests then; `evidence/linux-red.txt`); typical `left: "attacker" right: "original"` | 12/12 |
| E chain (accept-all stub) | 4 failed / 2 passed (in-session; not retained) | 6/6 |
| E artifacts (accept-all stubs) | 4 failed / 7 passed (in-session; not retained) | 11/11 |
| F6 `VACUUM INTO` | `CannotOpen` (14) (in-session) | passes |
| F7 retry | Retained: `shim-no-retry` mode fails 12/50 on macOS (`evidence/soak-macos.txt`) | 0/50 with the retry (same file) |

The passing tests in the B RED runs are the compatibility tests (pool/peer, stress, unbound
databases) plus `shim_refuses_symlinked_main_file`, which stock NOFOLLOW already satisfies.

**Full suite, retained:**

- `evidence/green-macos.txt`: exit 0, 35 tests;
- `evidence/linux.txt`: exit=0, 35 tests, plus the Linux probe line.

## Retained soak and probe results

| Evidence file | Platform | Mode | Runs | Failed | Sidecar ENOENT/retry events | Read-only sidecar opens |
|---|---|---|---|---|---|---|
| `soak-macos.txt` | macOS | stock | 50 | 0 | 0 | 0 |
| | | shim | 50 | 0 | 9 | 0 |
| | | shim-no-retry | 50 | **12** | 30 | 15 |
| `soak-linux.txt` | Linux | stock / shim / shim-no-retry | 50 each | 0 | 0 | 0 |
| `macos-soak.txt` | macOS (earlier binary, retry on) | shim / stock | 50 each | 0 / 0 | — | — |
| `stock-trace.txt` | macOS | stock, traced | 40 | 0 | 0 | — |

Earlier in-session counts that were **not** retained ("18 of 95" pre-fix shim failures, 55
further stock runs, a traced post-fix "0 of 100") are unverified, and the ADR does not rely
on them.

**Kernel probe** (`probe-macos.txt`, `linux.txt`): ENOENT from `O_CREAT` racing a concurrent
`unlink` was never seen on Linux (0/50k for both `open` and `openat`). On macOS it is rare and
was seen for both `open` and `openat` across runs (in-session: 0–3 per 50k for each).

**Two-uid demo** (`multiuid-demo.txt`, Linux): the attacker uid could not rename the victim's
directory under a root-owned `755` ancestor (`Permission denied`) or a sticky `1777` ancestor
(`Operation not permitted`). Under a `777` ancestor the rename **succeeded**.

## Source facts checked (SQLite amalgamation, D)

- `unixOpen` always adds `O_NOFOLLOW`, which covers only the last component.
- `SQLITE_OPEN_NOFOLLOW` is enforced in `sqlite3PagerOpen` from `unixFullPathname`'s
  `SQLITE_OK_SYMLINK`.
- `unixOpenSharedMemory` falls back from RW to RO and sets `isReadonly`.
- `unixAccess(EXISTS)` uses `osStat`. `findCreateFileMode` and `fileHasMoved` call `stat` on
  the main-database path.
- `verifyDbFile` warns on `nlink != 1` and on renames. `SQLITE_READONLY_DBMOVED` comes from
  `databaseIsUnmoved`.
- The `xSetSystemCall` documentation calls it a testing facility. Its syscall names are
  identical in 3.45.0, 3.46.0, 3.51.3 and 3.53.2.

## Not run

- FreeBSD and OpenBSD: no host was available. Their semantics are documentation-derived.
- Network, FUSE and ACL-bearing filesystems.
- Foreign-owned artifacts.
- Long soak and CI matrix runs.
- Anything against Engram's real code or real databases. This was out of scope by design.
