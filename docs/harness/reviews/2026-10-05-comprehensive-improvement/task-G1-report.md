# Task G1 report — SQLite POSIX-lock-dropping permission restriction (WAL/data loss)

Status: DONE_WITH_CONCERNS. Commit `974b7a6 fix(storage): restrict SQLite file modes without dropping locks`.

## Root cause (confirmed)
POSIX (fcntl) advisory locks belong to the (process, inode) pair. Closing ANY descriptor of a
file drops every lock the process holds on it. SQLite's unix VFS keeps its own per-inode
bookkeeping for its own descriptors, but cannot see foreign ones. In WAL mode every open
connection keeps a SHARED lock on the database file (and the DMS lock in `-shm`); the last
closing connection proves it is last by taking EXCLUSIVE on the db file, then checkpoints and
deletes the WAL. Two Engram paths closed foreign descriptors on live SQLite files:

1. `restrict_sqlite_artifact_permissions` (after every `with_connection`/`with_transaction`,
   `checkpoint`, `vacuum`, `compact`, `Storage::open`, `StoragePool`) opened db/-wal/-shm with
   `O_NOFOLLOW`, `fchmod`ed and closed them.
2. `prepare_database_file` (every `create_connection`) opened an EXISTING database to check and
   chmod it, and closed it. Harmless for the first open, but a second `Storage::open` or every
   `StoragePool` connection after the first dropped the locks of the connections already open
   in the process. This second path was not in Q4's finding; the new test proved it separately.

## What was implemented
- `src/storage/connection.rs`
  - `restrict_sqlite_artifact_permissions` now works by path only: `symlink_metadata` (lstat),
    refuse symlinks ("refusing to chmod symlink SQLite artifact"), skip missing/non-regular files,
    compute `current & 0600`, return early if unchanged, otherwise
    `fchmodat(AT_FDCWD, path, mode, AT_SYMLINK_NOFOLLOW)`.
  - Fallback `chmod_artifact_after_recheck` when fchmodat returns ENOTSUP/EOPNOTSUPP/ENOSYS/EPERM
    (old glibc; seccomp rejecting `fchmodat2`; or the path became a symlink on newer Linux): re-lstat,
    refuse symlink, require same dev+inode and regular file, then `chmod` by path. ENOENT = vanished,
    Ok. A genuine EPERM fails again in the fallback and is reported.
  - `prepare_database_file`: new file still created `O_CREAT|O_EXCL|O_NOFOLLOW`, mode 0600 (SQLite
    then creates -wal/-shm with the same mode); existing file is checked by lstat (symlink and
    non-regular refused with the same messages as before) and narrowed by path; never opened.
  - Removed `open_sqlite_artifact_no_follow` and `restrict_open_regular_file_permissions` (only
    callers were the two paths above).
  - Preserved: never follow symlinks, mode only narrowed relative to what lstat saw, 0600 result,
    `SQLITE_OPEN_NOFOLLOW` on SQLite's own open, Windows/non-unix no-op, C2's BEGIN IMMEDIATE,
    per-step migrations and journal-mode retry (untouched).
  - TOCTOU residual documented on `restrict_sqlite_artifact_mode`: lstat and chmod are separate
    lookups. Where `AT_SYMLINK_NOFOLLOW` works the chmod never follows a link (macOS chmods the
    link itself; Linux refuses). In the fallback, chmod could follow a link swapped in between the
    re-check and the chmod. In all cases someone able to rename entries in the db directory can
    redirect the chmod to another file owned by this user, setting it to the owner-only mode
    computed for the artifact (could add owner bits). Such a process can already replace the DB.
- `tests/storage_posix_lock_regression_tests.rs` (new): the binary re-executes itself
  (`child_storage_helper`, active only with `ENGRAM_G1_CHILD_DB`) as a second process, all
  through the public `Storage` API, files in a caller-owned tempdir:
  - `committed_writes_survive_another_process_open_and_close`
  - `second_in_process_open_does_not_drop_the_first_handles_locks`
  Each: commit row 1, child process opens/reads/closes, parent commits row 2, a fresh child
  process must count 2 rows, and `-wal` must still exist.
- `tests/http_transport_security/workspace_auth.rs` (C1): added a positive control
  `assert_raw_reader_sees_accepted_read` (an accepted `memory_get` must change `row_state` as seen
  by a fresh raw reader) in `test_claimed_workspace_does_not_authorize_foreign_id` (per tool, after
  the "unchanged" checks) and `anonymous_resources_read_is_bound_to_persisted_workspace`.
  No existing assertion changed.
- Unit tests in `connection.rs`: `unix_path_replacement_after_open_cannot_redirect_chmod` tested the
  removed fd helpers; ported to `unix_symlink_swapped_in_before_chmod_is_not_followed` (same
  property — a symlink swapped in cannot redirect the chmod to its target — via the `before_chmod`
  seam). Its old assertion "the opened inode ends 0600" has no equivalent: the new design never
  holds the inode. Added `unix_recheck_fallback_chmods_only_the_checked_inode` (fallback: same inode
  narrowed; replaced regular file refused and untouched; symlink refused, target untouched;
  vanished Ok), `unix_permissive_artifact_is_narrowed_by_path`, `unix_non_regular_artifact_is_left_alone`.
- Docs: root `INVARIANTS.md` new #27 (following items renumbered 28–38; no numeric references to
  27–37 found in the repo), `ERRORS_AND_LESSONS.md` entry.
- Hazard notes (doc comments only) at the remaining violation sites, see audit.

## Audit of descriptor open/close on live SQLite files in src/
| Site | Verdict |
|---|---|
| `storage/connection.rs` restrict after every call | FIXED (path-based) |
| `storage/connection.rs` `prepare_database_file`, existing DB | FIXED (lstat, never opened) |
| `storage/connection.rs` new DB `O_EXCL` create + close | Residual, documented: closed before any connection knows the new inode, except another thread opening the same brand-new empty file in that window |
| `storage/connection.rs` `compact` sidecar sizes, `available_disk_bytes` | Safe: `stat`/`statvfs` by path, no fd |
| `storage/lock.rs` `{db}.lock` | Safe: separate inode SQLite never locks; `flock` (per open file description), not fcntl |
| `sync/wal_replication.rs` `WalDeltaReader` reads `-wal` (status, sync_now, PITR) | Safe: SQLite takes no fcntl locks on `-wal` (WAL locks live in db file + `-shm`) |
| `sync/wal_recovery_staging.rs` seed copy `File::open(src)` + close | OPEN HAZARD, documented: MCP `replication_recover` defaults `source_db_path` to the server's own live DB; the copy's close drops the server's locks. Staging file / integrity check / parent-dir fsync are private files: safe |
| `sync/cloud.rs` `upload` (`tokio::fs::read`) / `download` (overwrite in place) | OPEN HAZARD, documented: `SyncWorker` passes the live db path, but no binary starts `SyncWorker` today |
| `graph/duckdb_graph/lifecycle.rs` DuckDB `ATTACH ... (TYPE SQLITE)` on the active DB | SUSPECTED HAZARD, documented, not reproduced: a second SQLite library copy in the process (sqlite.org "How To Corrupt" 2.2.1); feature `duckdb-graph` (in `full`, not in CI-required) |
| `watcher/browser.rs` `fs::copy` of a browser history DB | Safe: not Engram's DB; this process holds no locks on it |
| `storage/turso_backend` (libsql) | Not applicable: separate backend/engine, not opened on the same file as `Storage` in one process |
| snapshot builder/loader, image storage, multimodal, markdown export | Safe: not SQLite artifacts |

Not fixed by decision: the three open hazards need behavior changes on public/feature surfaces
(e.g. refuse `replication_recover` with the active DB as source/target, read the seed through
SQLite's own file handle, or attach DuckDB to a snapshot copy). Recorded as concerns.

## TDD evidence
Required features: `source scripts/ci-required-features.env`; argv
`cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --test storage_posix_lock_regression_tests`.

- RED (macOS, HEAD 98bd050 code + new test): exit 101, 1 passed / 2 failed:
  `committed_writes_survive_another_process_open_and_close` and
  `second_in_process_open_does_not_drop_the_first_handles_locks` both
  `assertion left == right failed: commit made after a foreign open/close was lost (live WAL deleted)
  left: 1 right: 2`. Expected: the fresh process sees only the checkpointed row; the second commit
  went to the WAL that the child deleted. (A first draft asserting `-wal` existence first failed
  with "live WAL was deleted by the other process"; asserts reordered so the data loss is the
  primary assertion.)
- Intermediate (only `restrict_sqlite_artifact_permissions` fixed): 2 passed, 1 failed —
  `second_in_process_open_does_not_drop_the_first_handles_locks` still lost the commit, proving the
  separate `prepare_database_file` hazard. Fixed next.
- GREEN (macOS): 3 passed, 0 failed. `--lib storage::connection`: 12 passed.
- Linux (local Docker image `rust:1.98-bookworm`, `--network none`, offline registry mount,
  glibc 2.36, kernel 7.0.14-orbstack, aarch64, toolchain 1.98.1 since the image lacks the pinned
  1.96, `--no-default-features`): pre-fix copy (`git archive HEAD` + new test) RED with the same
  1 vs 2 failures; fixed tree GREEN: lib `storage::connection` 12 passed, regression test 3 passed.
- C1 positive control: pre-fix copy on Linux, `--test http_transport_security workspace_auth`:
  2 failed (`test_claimed_workspace_does_not_authorize_foreign_id`,
  `anonymous_resources_read_is_bound_to_persisted_workspace`) with "accepted read is not visible to
  the raw reader; row_state comparisons would be vacuous" (before == after). This confirms Q4's
  suspicion that C1's HTTP "unchanged" assertions were vacuous before the fix. After the fix
  (macOS, required features): 4 passed.
- Python repro (`task-Q4-g1-wal-lock-repro.py`, kept in place in .superpowers, not tracked):
  control True / with open-close False — reproduces the mechanism without Engram.

## Verification (macOS, worktree)
- `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests`: exit 0,
  54 binaries, 2065 passed, 0 failed, 2 ignored (the second ignored is Q4 fix round's G-3 test,
  committed concurrently in 0d36d3d).
- `cargo clippy --all-targets --no-default-features --features "$CI_REQUIRED_FEATURES" -- -D warnings`: exit 0.
- `cargo fmt --all -- --check`: exit 0. Pre-commit hook (fmt + clippy `--all-features -D warnings`): passed.
- After a final comment-only edit: regression test 3 passed, `storage::connection` 12 passed.
- `bash docs/harness/bin/check-commit-msg.sh`: OK.

## NOT RUN
- Linux clippy (`-D warnings`) on the Linux target: not run; the errno list uses a slice
  `contains` (not a match pattern) because ENOTSUP == EOPNOTSUPP on Linux. CI (ubuntu-latest) will
  be the first Linux clippy.
- The ENOTSUP/ENOSYS/EPERM fallback is not reachable on macOS or the tested Linux; it is covered by
  calling `chmod_artifact_after_recheck` directly.
- `duckdb-graph` build and the DuckDB hazard: not built, not reproduced (needs the DuckDB sqlite
  extension, possibly a network INSTALL).
- Full Linux suite: only the G1 tests were run in the container.

## Files changed
`src/storage/connection.rs`, `tests/storage_posix_lock_regression_tests.rs` (new),
`tests/http_transport_security/workspace_auth.rs`, `src/sync/wal_recovery_staging.rs`,
`src/sync/cloud.rs`, `src/graph/duckdb_graph/lifecycle.rs` (doc comments only in these three),
`INVARIANTS.md`, `ERRORS_AND_LESSONS.md`, `docs/harness/progress/2026-10-05-improvement-lane-r.md`.
Unlisted in the brief: the three doc-comment files (audit "document each"), and
`workspace_auth.rs` (brief item 4). Q3's files untouched.

## Concerns
1. Open hazards, documented but not fixed (need an owner decision, behavior change on public
   surfaces): MCP `replication_recover` defaults its source to the server's live DB and copies it
   through a closed `File` (reachable by a scoped-write principal; the tool also accepts arbitrary
   filesystem paths). Options: refuse when source/target is the active DB (same dev+ino), or read
   the seed via SQLite's own file handle. `CloudStorage::upload/download` on the live path (not
   wired today). DuckDB `ATTACH` of the active DB (`duckdb-graph`).
2. `INVARIANTS.md` items 27–37 renumbered to 28–38; no numeric references found, but any Lane P
   edit to that list will conflict at merge.
3. Ported unit test: the old `unix_path_replacement_after_open_cannot_redirect_chmod` asserted that
   an already-opened inode ends 0600; that property no longer exists by design (no descriptor). The
   symlink-target-untouched property is kept.
4. Residuals documented: lstat/chmod TOCTOU (directory-write attacker could redirect the chmod and
   add owner bits to another owned file), and the brand-new DB `O_EXCL` fd close window.

---

## Fix report — review round 1 (commit `9712ade fix(storage): keep MCP file paths off the live SQLite files`)

### Changes
1. **[Important] `replication_recover` (fixed).** `src/mcp/handlers/sync.rs`: if the target is
   the active DB or one of its side files, it is refused. If the source is the active DB (device +
   inode, or resolved name; default args land here):
   - with `target_frame`, `target_time` or an explicit `source_wal_path`: refused ("not supported
     while it is open"). PITR to an earlier point needs the raw file + WAL, which cannot be read
     safely while the DB is open.
   - otherwise: new `WalRecoveryEngine::recover_active_database` (`src/sync/wal_recovery.rs`),
     using a new `StageSeed::ActiveStorage` (`wal_recovery_staging.rs`). The staging file is
     written by `VACUUM INTO` (includes committed WAL content) and then goes through the same
     fsync + integrity_check + rename. Contract change: the report shows `frames_replayed: 0`,
     not the WAL frame count; the recovered state is the same latest committed state.
   - A non-active source keeps the old path; an explicit `source_wal_path` naming an active side
     file is refused.
2. **[Important] DuckDB (fixed, reproduced).** `src/mcp/handlers/duckdb_graph.rs`: the 3 tools
   open `TemporalGraph` on `Storage::snapshot_copy()`: a `VACUUM INTO` copy in a private `0700`
   temp dir, removed on drop after the graph. The `lifecycle.rs` docs say never to pass the
   active DB.
   - The hazard is real: with the pre-fix handler, the new test failed on macOS (a fresh process
     saw 2 rows, expected 3).
   - Release exposure: `duckdb-graph` is in `full` and in `scripts/ci-features.env`, but NOT in
     `CI_REQUIRED_FEATURES`, the default features (`openai`), or `release.yml` (`--features pdf`).
     Release binaries are not exposed; a `--features full` build is.
   - Cost: one full DB copy per graph tool call.
3. **[Minor] Shared guard (done).** New `src/storage/active_artifacts.rs`:
   - `Storage::refuse_active_sqlite_artifact`: device + inode for existing files, which catches
     symlinks and hard links; resolved-parent name for not-yet-existing paths; covers
     `-wal`/`-shm`/`-journal`.
   - `Storage::is_active_sqlite_artifact`, `Storage::vacuum_into`, `Storage::snapshot_copy`.
   - Applied to `memory_ingest_document` (also covers the attestation read of the same path), all
     5 multimodal path tools (`describe_image`, `transcribe_audio`, `process_video`,
     `search_by_image`, `ingest_media`), `snapshot_create` (output), `snapshot_load` and
     `snapshot_inspect` (also covers the loader and attestation reads), `memory_upload_image`,
     `palace_visualize` `output_path`, and `replication_recover` (source/target/wal).
   - Residual (documented): the guard is a check followed by an open by path.
   - Not guarded: `replication_status` and `replication_sync_now` read the active `-wal` by
     design. SQLite takes no fcntl locks on `-wal` (its locks live in the db file and `-shm`), so
     this is lock-safe.
   - Still documented, not fixed (not in this round's list): `CloudStorage::upload`/`download`
     (`SyncWorker` is not started by any binary). INVARIANTS #27 text updated; numbering unchanged.
4. **[Minor] Other items.**
   - `child_storage_helper` is `#[ignore]`d; `run_child` passes `--exact --ignored`.
   - `create_connection` holds a process-wide `DATABASE_OPEN_LOCK` across
     `prepare_database_file` and SQLite's open.
   - `warn_if_replaced_by_symlink` logs `tracing::warn!` when the path is a symlink after a
     successful `fchmodat` (the macOS swap race).
   - New lesson in `ERRORS_AND_LESSONS.md`: `SQLITE_OPEN_NOFOLLOW` also applies to
     `VACUUM INTO` and rejects a symlink in any path component (macOS `/var`). `vacuum_into`
     resolves the parent. This was found when the first GREEN run failed with "unable to open
     database".

### Tests (new, in `tests/storage_posix_lock_regression_tests.rs`, module `mcp_paths`, all through `handlers::dispatch` on a file-backed `HandlerContext`)
Each test does its MCP call, then runs the child-touch + fresh-count check, then checks the result:
- `replication_recover_default_source_keeps_locks_and_recovers_latest_state`: also asserts the
  recovered DB holds the committed row.
- `replication_recover_refuses_active_target_and_point_in_time`
- `document_ingest_refuses_an_alias_of_the_active_database` (`notes.md` symlink to the DB)
- `media_ingest_refuses_the_active_database` (`multimodal`)
- `duckdb_graph_tools_do_not_attach_the_active_database` (`duckdb-graph`): covers
  `memory_graph_path` and `memory_temporal_snapshot`.
- Unit tests in `active_artifacts.rs` (4): aliases (path, `./`, symlink, hard link, side files,
  not-yet-existing `-journal`) refused; unrelated or missing files allowed; in-memory storage;
  snapshot holds committed rows, has a `0700` dir and is removed on drop; `vacuum_into` refuses
  the active DB.

### RED
- Linux container (`rust:1.98-bookworm`, offline, `--features multimodal`, source = `git archive
  HEAD`(4e8ba9a) + new test file): the 4 new MCP tests FAILED with `left: 2 right: 3` at the
  fresh-process count (data lost); 2 old tests passed.
- macOS, DuckDB test with the HEAD version of `duckdb_graph.rs` temporarily swapped in, then
  restored and verified with `cmp`: FAILED `left: 2 right: 3`.

### GREEN / verification
C7 had uncommitted, non-compiling edits in the shared worktree (`src/embedding/queue/tests.rs`:
`no method named embed`), so the full suite in the worktree failed to compile. Verification
therefore ran in an isolated tree: `git archive HEAD` + exactly the 16 committed files, with the
shared `target/`. I confirmed with `cmp` that the committed files are byte-identical to that tree.
Results:
- `cargo fmt --all -- --check`: exit 0.
- `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests`: exit 0,
  54 binaries, 2072 passed, 0 failed, 3 ignored. 2065 + 4 MCP + 4 unit − helper (now ignored).
- `cargo clippy --all-targets --no-default-features --features "$CI_REQUIRED_FEATURES" -- -D warnings`: exit 0.
- Same with `"$CI_REQUIRED_FEATURES,duckdb-graph,snapshot,attestation"`: exit 0.
- `cargo clippy --all-targets --all-features -- -D warnings`: clean.
- `cargo test --no-default-features --features "duckdb-graph,snapshot" --test duckdb_graph_tests --test storage_posix_lock_regression_tests`:
  6 passed, and 6 passed + 1 ignored. `--lib storage::active_artifacts`: 4 passed.
- `--features "snapshot,attestation,multimodal" --lib mcp::handlers`: 206 passed.
- Pre-commit hook in the worktree (fmt + clippy all-features): passed at commit time.
- Worktree-only observation: `--lib storage::` showed 3 failures in
  `storage::sqlite_backend::health::tests::*`. That file is C7's uncommitted work, not this change.

### Concerns
- Behavior change on the public `replication_recover` tool: from the active DB, only "latest"
  recovery is allowed now, and `frames_replayed` is 0 for it. PITR from the active DB is refused.
- The DuckDB tools now copy the full DB on every call.
- `CloudStorage::upload`/`download` remain a documented hazard (not wired).
- The check-then-open residual remains for every guarded path.

---

## Fix report — review round 2 (commit `bbe9203 fix(storage): refuse descriptor aliases of the live SQLite files`)

### Changes
1. **[Important] `/dev/fd/N` alias (fixed, reproduced).** `src/storage/active_artifacts.rs`:
   - `refuse_active_sqlite_artifact` first refuses any path that is under `/dev` or `/proc` as
     given, after `std::path::absolute`, after resolving its parent, or after canonicalizing
     (`refuse_descriptor_alias`, own error text "paths under /dev and /proc can alias open file
     descriptors").
   - `same_inode` also matches on inode alone when the candidate's device equals that of `/dev`,
     `/dev/fd` or `/proc/self/fd` (macOS devfs/fdesc, Linux procfs).
   - This covers every guarded MCP path (ingest_media, describe_image, ingest_document,
     upload_image, snapshot_*, replication_recover). `replication_recover` now also refuses a
     non-active explicit `source_db_path` that the guard rejects.
   - Through the inode-only rule, a `/dev/fd/N` source that aliases the active DB is recognized as
     active, so it gets latest-state recovery via `VACUUM INTO`, never a raw copy.
2. **[Minor] DuckDB drop order (fixed).** `open_graph_on_snapshot` returns `(snapshot, graph)` and
   callers bind `let (_snapshot, graph)`, so `graph` drops first. Verified with a rustc probe
   (prints "drop graph" then "drop snapshot"). The reviewer was right: the old
   `(graph, _snapshot)` dropped the snapshot first.
3. **[Minor] Contract documented.** The `replication_recover` description and the
   `source_db_path` text in `src/mcp/tools/catalog/sync.rs` now say: active source → latest
   state only (`frames_replayed` 0, `last_frame_applied` null); frame/time/WAL PITR needs a
   closed copy; the active DB is refused as a target. `docs/MCP_TOOLS.md` was regenerated with
   `scripts/generate-mcp-reference.sh`; `--check` reports "up to date".
4. **[Minor] More call sites.**
   - `memory_import_markdown` refuses `*.md` entries that are the active DB or a descriptor
     alias; each such entry is reported per file as `status: error`.
   - `replication_status` and `replication_sync_now` call the new
     `Storage::refuse_lock_bearing_sqlite_file` on `{db_path}-wal`. It refuses the DB file, `-shm`
     and descriptor aliases, but allows the active `-wal`: SQLite holds no POSIX locks on it, and
     the default usage reads it.
   - INVARIANTS #27 wording extended (numbering unchanged).

### Tests
- Unit tests (`active_artifacts.rs`):
  - `descriptor_aliases_of_the_active_database_are_refused`: opens the DB and tries `/dev/fd/<n>`,
    plus `/proc/self/fd/<n>` on Linux, through both guards; also a symlink to `/dev/fd/<n>`.
  - `lock_bearing_guard_allows_the_wal_only`.
- Integration test (`mcp_paths::descriptor_alias_paths_are_refused_or_snapshotted`, feature
  `multimodal`): holds a descriptor of the DB open and passes `/dev/fd/<n>` to
  `memory_ingest_media` and as `replication_recover` source_db_path. Then it runs the child-touch
  + fresh-count check. Finally it asserts media was refused and recovery was refused or
  snapshotted.
- RED (macOS): with the HEAD versions of `active_artifacts.rs` and `handlers/sync.rs` temporarily
  swapped in (restored and verified with `cmp`), the test FAILED with `left: 2 right: 3`
  (commit lost).
- On Linux the alias already canonicalized to the real file, so there was no RED there.

### Verification
Ran in an isolated tree (`git archive HEAD` + the 7 changed code/docs files; the committed files
are `cmp`-identical; C7/Q7 have uncommitted edits in the worktree).
- `cargo fmt --all -- --check`: exit 0.
- `cargo test --no-default-features --features "$CI_REQUIRED_FEATURES" --tests`: exit 0,
  54 binaries, 2093 passed, 0 failed, 3 ignored. The count includes tests that C7/Q7 committed to
  HEAD since round 1.
- clippy `-D warnings`:
  - First run: `unused_mut` in the new unit test on macOS, where `aliases.push` is Linux-only.
    Fixed with a `cfg!`-built vec. The fix changes test code only; the full `--tests` run above
    predates it.
  - After the fix, required features and `--all-features` are clean; `--lib storage::active_artifacts`
    passes 6/6.
- `--features "duckdb-graph,snapshot,multimodal" --test duckdb_graph_tests --test storage_posix_lock_regression_tests`:
  6 passed, and 8 passed + 1 ignored.
- Linux container (glibc 2.36, offline, `--features multimodal`, run before the `cfg!` test
  tweak): `storage::active_artifacts` 6 passed (including `/proc/self/fd`); regression tests
  7 passed + 1 ignored.
- Pre-commit hook passed.

### Concerns
- Every path under `/dev` and `/proc` is now refused for these tools, including legitimate ones
  like `/dev/shm/x.md`. This fails closed.
- The check-then-open residual remains.
