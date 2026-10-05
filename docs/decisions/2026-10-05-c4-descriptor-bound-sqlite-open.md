# ADR: Descriptor-bound SQLite open (task C4)

- **Status:** Proposed. The owner decides adoption. Nothing in this ADR is implemented in
  the core crate, and atomic descriptor-bound opening is **not** claimed as resolved.
- **Date:** 2026-10-05 (revised the same day after review round 1)
- **Decision owner:** Ronaldo
- **Source:** task C4 of `docs/harness/plans/2026-10-02-engram-comprehensive-improvement-plan.md`
  (Lane R).
- **Evidence:** a standalone spike, run on macOS and on Linux. It is committed under
  [`assets/2026-10-05-c4-spike/`](assets/2026-10-05-c4-spike/README.md). It is outside the
  core crate, not a workspace member, and not built by CI; its retained outputs are in
  `evidence/`. Results, exact commands and code pointers are in
  [`2026-10-05-c4-descriptor-bound-sqlite-open-spike-notes.md`](2026-10-05-c4-descriptor-bound-sqlite-open-spike-notes.md).
- **Related:** deferral commit `59eb645`; reverted candidates `ef02ee7`, `98b8446` and
  `0700cb9` (reverts `2272d6c`, `21a9af3`, `727154f`); G1 (`974b7a6`, `9712ade`, `bbe9203`);
  root `INVARIANTS.md` #27; `ERRORS_AND_LESSONS.md` ("Closing any descriptor of a live SQLite
  file..." and "SQLITE_OPEN_NOFOLLOW rejects a symlink in ANY path component").

Evidence tags used below: **T** = spike test on macOS and Linux, **D** = SQLite source or
platform documentation only, **—** = not tested.

## 1. Context

Wave 2 tried to make SQLite open the exact file that Engram had already resolved and checked.
The goal was to close the gap between Engram's check and SQLite's `open(2)`. Three commits
were reverted on 2026-07-12 after Linux CI failed:

- `ef02ee7` registered a proxy VFS for each connection. It opened and verified the main file
  through a parent-directory fd. It then gave SQLite a path for that fd: on macOS the current
  pathname from `F_GETPATH`, on Linux the target of `readlink(/proc/self/fd/N)`, and
  `/dev/fd/N` elsewhere. The main-file `xOpen` revalidated that path and then delegated the
  open, still by name, to the stock VFS.
- `98b8446` handled `/dev/fd/N` aliases. When delegating to the stock VFS, it stripped
  `SQLITE_OPEN_NOFOLLOW`, and its identity check followed the alias (`stat` instead of
  `lstat`).
- `0700cb9` stopped resolving the Linux descriptor with `readlink`. It passed the literal
  `/proc/self/fd/N` as the open path, and extended alias and sidecar recognition to
  `/proc/self/fd/`.
- The 2026-07-12 record says Linux CI showed that the stock unix VFS could not open the
  database through that alias. It also says pathname aliases kept a regular-file TOCTOU, and
  transient hardlinks broke either pools or the stock locking protocol.
  - The spike reproduces that shape (verbatim alias, NOFOLLOW stripped): on Linux the open
    fails with `CANTOPEN` (T), which is **consistent with** that CI failure. It does not
    prove that this was the CI failure.
  - The likely reason (D) is that stock `unixOpen` always adds `O_NOFOLLOW` to `open(2)`, and
    `/proc/self/fd/N` is a symlink.

The barrier-v4 and review-v2 PASS for the pre-CI candidate are historical evidence only.
This ADR does not reuse them as closure.

**The open path today** (`src/storage/connection.rs` on this branch, after G1):

- A process-wide `DATABASE_OPEN_LOCK` is held around preparing and opening the database.
- A missing parent directory is created `0700`. A permissive *direct* parent only gets a
  warning.
- A new database file is created with `O_CREAT|O_EXCL|O_NOFOLLOW` and mode `0600`, then
  closed before SQLite knows the inode. An existing file is checked by path (`lstat`, regular
  file, not a symlink) and its mode is only ever narrowed (`fchmodat(AT_SYMLINK_NOFOLLOW)`).
- SQLite opens `canonical(parent)/name` with `SQLITE_OPEN_NOFOLLOW`.
- No descriptor of a live database file is ever opened and closed (invariant #27).

Two facts about SQLite 3.45.0, the version bundled by `libsqlite3-sys 0.28.0` and
`rusqlite 0.31.0` and pinned by `Cargo.lock`, shape every option (D):

1. SQLite opens files **by name every time**. `unixFullPathname` walks the path with `lstat`
   and `readlink`, and `SQLITE_OPEN_NOFOLLOW` turns any symlink it finds into
   `SQLITE_CANTOPEN_SYMLINK`. Later, `open(2)` resolves the resulting string again, and
   `O_NOFOLLOW` covers only its last component. The `-wal`, `-shm` and `-journal` files are
   opened by name, as `fullpath + suffix`, by every connection in every process, at the time
   each one needs them. Journals are deleted by name too.
2. SQLite has **no public API to open a database from an existing descriptor**. The unix VFS
   tracks POSIX locks per `(dev, ino)` in `unixInodeInfo`, and it knows only the descriptors
   it opened itself.

## 2. Threat model

**Assets.** The integrity of the memory database is the main asset. Engram feeds that
database back to agents, so a substituted or poisoned database amounts to prompt injection
at scale. The confidentiality of memories is the second asset, because Engram might write
them into a file the attacker can read. Availability is the third.

**In scope.** A local, unprivileged attacker with a **different uid** and write permission on
the database directory or on any ancestor of it. Examples:

- a group-writable or shared project directory;
- a non-sticky world-writable directory;
- a custom `ENGRAM_DB_PATH` on a multi-user host or CI runner;
- a volume shared between containers that run as different uids.

**Out of scope.**

- **The same uid.** Code running as the same uid can ptrace Engram, edit the database, the
  config and the binary directly. Binding to a descriptor adds **no** protection against it.
  This includes other agent tools that run as the user.
- **Root.**
- Bugs in the kernel or filesystem, and network or FUSE filesystems that misreport ownership.

**Races.** "Swap" means rename, replace, or create a name.

| ID | Race | Today | Evidence |
|---|---|---|---|
| R1 | The attacker swaps the database file or an ancestor directory **without symlinks** between Engram's check and SQLite's `open` | Open | T (`stock_open_follows_a_directory_rename_swap...`) |
| R2 | The attacker swaps an ancestor for a **symlink** after SQLite's `lstat` walk and before `open(2)` | Open | T (`stock_nofollow_does_not_close_the_window_before_open`) |
| R2' | The same symlink swap, but before SQLite's walk | Closed by `SQLITE_OPEN_NOFOLLOW` | T (`stock_nofollow_rejects_a_symlinked_parent...`) |
| R3 | The attacker plants or swaps `-wal`, `-shm` or `-journal` (a hot WAL or journal is replayed into the database) | Open if the attacker can write the database directory | — (follows from D; not tested on current code) |
| R4 | A later opener resolves the name again: pool growth, a CLI, a hook, a second MCP server | Same exposure as R1 to R3, every time | Follows from R1 to R3 |
| R5 | A journal or WAL is deleted or recreated by name | Same as R3 | — |

## 3. Why pathname aliases and hardlink rejection do not close it

- **A canonical path or any other pathname alias is still a name.** It is resolved again at
  every open of the main file and of each sidecar, by every connection and process, so R1,
  R2 and R4 stay open. The spike reproduces R1 and R2 deterministically (T). For R2 it uses a
  syscall hook that swaps the parent directory right before the real `open`, after
  NOFOLLOW's walk has passed. `F_GETPATH` and `readlink(/proc/self/fd/N)` (used by `ef02ee7`)
  turn a descriptor back into such a name.
- **Rejecting hardlinks (`st_nlink > 1`) is itself a check-then-use step.** It also does not
  detect a rename swap, because nlink is unchanged (D).
- **A transient private hardlink** given to SQLite makes SQLite derive *different*
  `-wal`/`-shm` names for the same inode. Two processes or connections that use different
  names then write different WALs for one database, which corrupts it. Removing the link
  later makes SQLite log "unlinked/renamed while open" (`verifyDbFile`), and writes in
  rollback mode fail with `SQLITE_READONLY_DBMOVED`. These are facts from the SQLite source
  (D); the spike did not prototype hardlinks. They match the 2026-07-12 finding that
  hardlinks broke pools or stock locking.
- **A descriptor alias** (`/dev/fd/N`, `/proc/self/fd/N`) fails on its own terms. See Option A
  below.

## 4. Options compared

| Option | Closes R1/R2/R4 | Closes R3/R5 | Stock locking + WAL sharing with other processes | Main cost or blocker | Evidence |
|---|---|---|---|---|---|
| **A.** `sqlite3_open_v2("/dev/fd/N")` or `/proc/self/fd/N` on the stock VFS | macOS/BSD: main file only. Linux: no | No | Breaks: sidecar names are `/dev/fd/N-wal` and similar | macOS: the main file is bound, but writes fail (`-journal` cannot be created on devfs) and WAL cannot be enabled. Linux: NOFOLLOW refuses the magic link; without NOFOLLOW SQLite `readlink`s back to a **mutable pathname**; with the alias kept verbatim, `open(2)` fails. The pinned fd must never be closed (G1) | T |
| **A'.** As A, plus a proxy VFS that remaps sidecars to `openat(parentfd)` (the reverted `ef02ee7` family) | Partly | Partly | At risk | Same alias failures; the verbatim-alias shape fails on Linux (T). The main file is still opened by name after a revalidation (TOCTOU). The per-connection `main_file` lock drop (F10) is **by inspection only** | Reverted; T plus inspection |
| **B.** Pinned parent-directory fd plus a unix-VFS **syscall shim** (`xSetSystemCall` redirecting `open/stat/access/unlink/openDirectory` for the bound names to `*at()`), plus a VFS copy whose `xFullPathname` returns bound names verbatim | Yes | Yes for other uids (the directory fd is verified as owned by the euid and not group- or other-writable); a main-file inode swap fails closed | **Yes.** SQLite keeps all of its own locking, inode, shm and checkpoint logic; peers on the stock path share the same sidecars | Relies on an interface SQLite documents as **for testing**, whose set of calls "varies ... from one version ... to the next". The syscall table is **process-global** and unsynchronized. A macOS-only failure needed a retry (F7). Every rejected main-file open deliberately leaks one parked fd for the life of the process. Needs specialist review | T (12 tests plus soak) |
| **C.** `unix-dotfile` or `unix-none` VFS (no fcntl locks, so no G1 lock drop) | No by itself | No | **Breaks.** A writer on these VFSes is invisible to a stock-VFS peer. `unix-dotfile` cannot enter shared-memory WAL mode | Cross-process corruption, and loss of WAL concurrency | T |
| **D.** Pathname aliases or transient hardlinks | No | No | Breaks (different sidecar names) | See section 3 | D, plus the reverted history |
| **E.** **Trusted directory chain plus trusted artifacts** (OpenSSH StrictModes style), as two parts. (1) Every ancestor is owned by root or the euid and is not group- or other-writable, except a sticky directory whose next component is owned by the euid or root; the database directory is owned by the euid with mode `& 022 == 0`. (2) The database and any existing `-wal`, `-shm` and `-journal` are regular files (no symlink) owned by the euid with mode `& 077 == 0`. They are checked with `lstat` before the open and re-checked with `fstat` on the opened handle after SQLite opens them. | For files created or verified under a trusted chain: no other non-root uid can change any name on the path, so check-then-open cannot be raced | For artifacts that pass (2): a planted foreign-owned or permissive sidecar is refused. **Residual:** a descriptor that another uid opened *before* the file became owner-only survives any mode change | **Unchanged** (no VFS change) | A policy change with real compatibility costs (section 8.2). ACLs and network filesystems are residuals | T (11 checker tests plus a two-uid demo). A foreign-*owned* artifact is rejected by the same owner check but was not tested (tests run as an unprivileged uid that cannot create one) |
| **F.** Fully custom or audited VFS (a fork of `os_unix.c`) or a patched amalgamation with fd/`openat` support | Yes | Yes | Must re-implement or patch ~8k lines of locking/shm/WAL code; a custom `-sys` build | Highest cost and audit surface; every SQLite upgrade means re-auditing a fork | — |

### Platform matrix

| Aspect | Linux | macOS | FreeBSD | OpenBSD |
|---|---|---|---|---|
| `/dev/fd/N` semantics | Symlink → `/proc/self/fd/N` magic link (T) | fdesc node, open = `dup` (T) | Only 0–2 unless `fdescfs` is mounted; then `dup`, or a symlink with `linrdlnk` (D) | `dup` per `fd(4)` (D) |
| Option A works | No: NOFOLLOW refuses it; without NOFOLLOW the name resolves back; with the alias kept verbatim, `open(2)` fails (T) | Main file only; sidecars fail (T) | Not run | Not run |
| `openat/fstatat/unlinkat/faccessat`, `O_DIRECTORY/O_NOFOLLOW/O_CLOEXEC` (needed by B) | Yes (T) | Yes (T) | POSIX.1-2008 (D) | POSIX.1-2008 (D) |
| Refuse symlinks in all components when pinning the directory | `openat2(RESOLVE_NO_SYMLINKS)`, kernel ≥ 5.6 (D) | `O_NOFOLLOW_ANY`, macOS ≥ 11 (D) | `O_RESOLVE_BENEATH` is only partial (D) | None (D) |
| Kernel ENOENT from `O_CREAT` racing an `unlink` | 0 of 50k (T) | Rare, for both `open` and `openat` (T) | Not run | Not run |
| Option B suite plus soak (retained) | 12 of 12; 0/50 with and without the retry (T) | 12 of 12; 0/100 with the retry, 12/50 failures without it (T) | **Not run** | **Not run** |
| Option E checker | 11 of 11; two-uid rename demo (T) | 11 of 11 (T) | Not run | Not run |

Windows is unchanged: filesystem databases already fail closed on non-Unix platforms.

## 5. Interaction with POSIX locks (G1)

- The spike reproduces invariant #27 on its own (`lock_hazard`, T). With a writer holding
  `RESERVED`, opening and closing one more descriptor of the database file makes a peer
  process see the lock `FREE` and start its own write transaction.
- **A and A'** require keeping a descriptor of the database file. Closing it at *any* time
  while any connection in the process holds locks on that inode drops those locks. That
  includes connections opened outside `Storage` and the other members of a pool. In
  practice, the descriptor could never be closed.
- **B** keeps only the **directory** fd. Closing a directory fd keeps the database lock
  (`closing_a_directory_descriptor_keeps...`). SQLite's own fds are fresh `openat` calls on
  the same inodes, so `unixInodeInfo` works as usual. B never closes a database-file fd that
  it handed out or rejected: a main-file fd that fails the identity check is parked for the
  life of the process. This is a deliberate, unbounded leak of one fd per rejected open; an
  implementation would need a cap or a fail-fast mode.
- **C** avoids fcntl locks only by giving up interoperability with the stock-VFS peers that
  Engram's CLI, hooks and other servers use.
- **E** adds no descriptors of its own. The post-open re-check uses `fstat` on SQLite's own
  handle (for example via `SQLITE_FCNTL_FILE_POINTER`), or `lstat` of the same name, which
  cannot change under a trusted chain. It never opens and closes a descriptor of a live file
  (invariant #27).

## 6. Sidecars, multiple connections and processes, checkpoint

- SQLite derives sidecar names from the full path string. Any alias name (A, D) splits the
  WAL and shm between processes that use different names.
- B keeps the real canonical name, so peers on the stock path share one `-wal` and one `-shm`
  with it. The spike passes these tests (T):
  - a two-connection pool plus a separate peer process, with mutual visibility,
    `BEGIN IMMEDIATE` blocking across processes, `wal_checkpoint(TRUNCATE)`, and removal of
    the WAL on last close;
  - 4 threads plus 2 peer processes under Engram's local-mode pragmas, followed by
    `integrity_check`;
  - `journal_mode=DELETE` with `synchronous=FULL` (CloudSafe mode), where the journal is
    created and deleted inside the pinned directory;
  - `VACUUM INTO` an unbound path from a pinned connection, which `snapshot_copy`,
    replication recovery and the DuckDB tools use.

## 7. Spike findings

- **F1.** Option A cannot support writes on macOS. On Linux it is either refused or not
  bound. The reverted candidates' shape (verbatim alias, NOFOLLOW stripped) fails to open on
  Linux, which is **consistent with** the July Linux CI failure. It is not proof of that
  failure's cause.
- **F2.** `SQLITE_OPEN_NOFOLLOW` checks every component, but only at `lstat` time. R2 is
  reproducible in the window between that walk and `open(2)`.
- **F3.** rusqlite reports only the primary code of a failed open, so
  `SQLITE_CANTOPEN_SYMLINK` appears as `CANTOPEN` (14). Tests and diagnostics must not rely on
  the extended code.
- **F4.** B binds the main file and all sidecars. A swap of an ancestor directory after
  binding has no effect, and a symlink or another regular file swapped over the main file
  fails closed. New databases are created `0600` inside the pinned directory.
- **F5.** B is **process-global**. Any opener in the same process of the bound pathname is
  redirected to the pinned directory, through any VFS that shares the unix syscall table,
  including plain `rusqlite::Connection::open`. A separate process sees the current
  namespace. DuckDB's private SQLite copy is not affected.
- **F6.** In B, the VFS copy must delegate `xFullPathname` for unbound names. Otherwise
  `VACUUM INTO` and `ATTACH` through a pinned connection fail with `CANTOPEN` (seen as RED,
  then fixed).
- **F7.** The shim without its retry fails under multi-process stress on macOS. Retained
  evidence: `evidence/soak-macos.txt`, mode `shim-no-retry`, **12 of 50** runs failed with
  `SQLITE_READONLY`, with 15 read-only sidecar opens traced.
  - **Mechanism (from the trace):** `openat(-shm, O_RDWR|O_CREAT)` returns ENOENT while a
    peer process, as the last closer, unlinks `-shm`. SQLite falls back to `O_RDONLY` and
    marks the process-wide shm node read-only, so every in-process connection on that
    database fails writes with `SQLITE_READONLY` until the last of them closes.
  - **With the bounded retry** (`RETRIES = 64` in `openat_create_retrying`): 0 of 50 runs
    failed (`soak-macos.txt`, 9 retry events) plus 0 of 50 (`macos-soak.txt`).
  - **Linux:** 0 of 50 failures in all three modes.
  - **Stock path:** 0 failures and 0 sidecar ENOENT events in the 140 retained runs:
    `stock-trace.txt` (40), `macos-soak.txt` (50) and `soak-macos.txt` (50). The 55 other runs
    reported before this revision (15 + 40) and the shim's first-round "18 of 95" were seen
    in-session only and are **unverified**; this ADR no longer relies on them.
  - The kernel probe shows the same ENOENT for path-based `open` on macOS. The stock path is
    therefore **potentially exposed** to the same process-wide read-only failure, a
    production availability bug, but that is not reproduced. See follow-up 3.
- **F8.** The names of the overridable syscalls are identical in SQLite 3.45.0, 3.46.0,
  3.51.3 and 3.53.2 (the sources cached locally). The new path-based call sites in 3.53.2 are
  a `/proc/.../fdinfo` debug helper and the Cygwin Windows VFS, not the unix database paths.
  This is encouraging, but the API carries no stability guarantee.
- **F9.** The premise of E holds with two real uids (Linux). An attacker uid could not
  rename the victim's directory under a root-owned `755` or sticky `1777` ancestor, and could
  under a `777` one.
- **F10.** By inspection only (no spike reproduction), the reverted `ef02ee7` held a
  per-connection main-file descriptor whose drop would be a G1-class lock drop. If that is
  confirmed, it would have needed rework even if Linux CI had passed.

## 8. Decision (Proposed, pending the owner)

1. **Reject A, A', C and D.** Tests show A and C are not viable. D is rejected from the SQLite
   source and the reverted history. A' is reverted, its verbatim-alias shape fails on Linux
   (T), and its G1-class lock drop is a finding **by inspection only** (F10). **Do not pursue
   F** without a separate funding decision: it is a fork of SQLite's VFS.
2. **Proposed boundary: Option E** (both parts: the trusted chain and trusted artifacts), as
   a separately planned implementation task.
   - **What it gives.** For files created or verified under a trusted chain, it removes the
     capability that R1 to R5 need: another uid able to change names on the path, or to own
     or write an artifact. It does not touch SQLite, covers sidecars and every process, and
     is portable.
   - **What it does not give.** Residuals: a descriptor that another uid opened while a file
     was still permissive; ACLs that grant write without mode bits; network and FUSE
     filesystems; root and the same uid.
   - **Known compatibility costs** that the owner must weigh:
     - user-private-group systems with `umask 002` make directories `775`
       (group-writable), which E refuses even though the group contains only the user;
     - on macOS some ancestors are `root:admin 775` (for example `/Applications`, as observed
       on the spike host), so databases below them are refused;
     - on Docker and Fly, volumes are often root-owned while Engram runs as non-root, or
       owned by a different uid, so the database-directory owner check fails;
     - `create_dir_all_restrictive` creates only *missing* components, with mode `0700`, and
       never chmods existing intermediates, so the verdict depends on who created the path
       (for example a user's `mkdir -p` under `umask 002`);
     - existing databases in group-writable locations, where today the code only warns about
       the direct parent.
   - **Owner decisions:** fail closed or warn, with an explicit opt-out; ACL handling (macOS
     and NFSv4 ACLs, POSIX ACLs); network and FUSE filesystems; the migration message for
     refused paths.

   Before E lands it needs its own plan, tests, a data and portability plan, and specialist
   review.
3. **Option B is viable but deferred.** Consider it only if Engram must operate under
   untrusted ancestors, or must bind the main inode even against same-directory swaps. Its
   preconditions are:
   - specialist SQLite and VFS review;
   - a guard test pinned to the SQLite version, covering syscall names and path call sites;
   - BSD runs;
   - a CI soak on all supported platforms;
   - a bound on the parked-fd leak;
   - explicit acceptance of its process-global semantics and its reliance on a testing
     interface.

   Asking upstream for a supported open-by-descriptor API is an alternative path.
4. Until E or B lands, **atomic descriptor-bound opening stays unresolved**, and the residual
   risk below is recorded.

## 9. Residual risk (current code)

- **Exposed:** another uid with write access to the database directory or an ancestor can
  win R1, R2, R3 or R4. The result is a substituted database, an injected hot WAL or
  journal, or memories written into an attacker-readable file.
- **Current mitigations:**
  - Engram-created directories are `0700`;
  - a warning for a permissive *direct* parent;
  - symlink refusal and NOFOLLOW;
  - `0600` files with modes only narrowed;
  - MCP path guards (`refuse_active_sqlite_artifact`).
- **Likelihood:** low on a single-user machine with the default path. Higher on shared hosts
  and CI with a custom `ENGRAM_DB_PATH` under group- or world-writable directories.
- **Owner:** Ronaldo. **Revisit:** when E or B is planned, or when Engram adds multi-user or
  shared-volume deployment.

## 10. Follow-ups (need owner approval; not started)

1. An implementation plan for Option E (or a recorded "warn-only" decision).
2. If B is ever chosen: an implementation plan that meets the preconditions in 8.3.
3. **Potential production availability bug (owner Ronaldo):** can stock Engram on macOS
   hit F7's `-shm` read-only fallback when several processes start at once? If it does,
   every connection of the process gets `SQLITE_READONLY` until all of them close.
   - **Repro recipe:** in the spike, run `RUNS=200 TMPDIR=... ./soak.sh`. Mode `stock` runs
     `tests/stress_control.rs`, the same workload on the stock VFS: 4 threads × 150 inserts
     plus 2 `peer` processes × 40 inserts, each peer opening and closing per insert, so a
     peer is often the last closer that unlinks `-wal` and `-shm`.
   - **What to look for:** with `C4_TRACE=1`, a line `stock_open -shm ... rdonly=0 -> -1 No
     such file or directory` followed by `stock_open -shm ... rdonly=1 -> <fd>`, and then
     test failures with `SQLITE_READONLY`.
   - **Current status:** 0 of 140 retained stock runs show it. The shim reproduces it at
     12/50 without the retry. A fix would be the same bounded retry, which is only possible
     with a shim, or a startup ordering that keeps one connection open.
