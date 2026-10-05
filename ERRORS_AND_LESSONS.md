# Errors & Lessons — Mistake Catalog

Consult this file **before starting any task**. Organized by category, not chronologically.

## Format

```markdown
### [Category] Short description
**Context:** When/where this happens
**Wrong:** What we did that failed
**Right:** What actually works
**Date:** When discovered
```

## Categories

Use one of: Data Processing, Dependencies, API, Deploy, Logic, Config, Testing,
Tech Debt, Security, Performance, Fragile Areas

---

<!-- [placeholder] -->

### [Dependencies] Example: version mismatch after update
**Context:** After updating a dependency, imports or builds break
**Wrong:** Blindly updating all deps at once without testing
**Right:** Update one dependency at a time, run tests between each
**Date:** (template)

### [Config] Example: environment variable not loaded
**Context:** App fails on startup with missing config error
**Wrong:** Hardcoding the value as a workaround
**Right:** Check .env file exists, verify loading mechanism, add to .env.example
**Date:** (template)

### [Logic] Example: off-by-one in pagination
**Context:** API returns duplicate or missing items at page boundaries
**Wrong:** Using 1-based offset with 0-based index
**Right:** Standardize on 0-based indexing internally, convert at boundaries
**Date:** (template)

### [Data Processing] Dream candidate kind changes need schema and storage updates
**Context:** Adding a new `dream_candidates.kind` value such as `agent_writeback`.
**Wrong:** Updating only Rust allowlists or metadata while SQLite still has a CHECK constraint that rejects the new kind.
**Right:** Update storage validation and add a schema migration that rebuilds the constrained table, then cover the new kind with a migration test.
**Date:** 2026-07-03

### [Logic] New dream candidate kinds need explicit apply semantics
**Context:** Adding a generated-memory candidate kind such as `agent_writeback`.
**Wrong:** Letting unknown candidate kinds fall through to the generic `note` memory type, or returning different dry-run/live response shapes.
**Right:** Add an explicit `memory_type_for_candidate` case and keep dry-run/live JSON wrappers isomorphic so clients can preview and confirm safely.
**Date:** 2026-07-03

### [Performance] SQLite WAL lock starvation from holding transactions across async operations
**Context:** Calling embedding APIs, LLM endpoints, or cloud sync while inside a SQLite write transaction.
**Wrong:** Holding an active transaction lock while awaiting network I/O, causing `SQLITE_BUSY` starvation across all concurrent readers and writers.
**Right:** Complete all network calls, tokenization, and embedding computation before opening a transaction; keep transaction locks bounded to microsecond local database writes.
**Date:** 2026-08-15

### [Security] Unbounded WAL frame page seeking produces multi-terabyte sparse files
**Context:** Replaying delta WAL frames or snapshot data from untrusted peers.
**Wrong:** Seeking directly to `(frame.page_number - 1) * page_size` without upper-bound validation, allowing corrupt or malicious page numbers (e.g. `u32::MAX`) to create multi-terabyte sparse files on disk.
**Right:** Validate that `frame.page_number > 0 && frame.page_number <= MAX_ALLOWED_DB_PAGES` before seeking.
**Date:** 2026-08-25

### [Logic] Slicing original UTF-8 string with byte offsets from case-folded copy causes runtime panic
**Context:** Context compression or text filtering matching substrings on lowercased copies and removing/slicing original strings.
**Wrong:** Using byte offsets from `text.to_lowercase().find(...)` to slice or drain `original_text`, which panics when multilingual characters change byte size upon casing (e.g., German `ß` [2 bytes] vs `ẞ` [3 bytes]).
**Right:** Perform substitutions using regex engine word boundaries (`\b`) or Unicode character/grapheme iterators, never raw cross-casing byte indices.
**Date:** 2026-08-30

### [Security] Integer primary key lookups bypass workspace tenant boundary (IDOR)
**Context:** MCP handlers or CLI subcommands fetching records by integer ID (`memory_id: 123`).
**Wrong:** Loading memory by `id` alone and assuming the caller has access because they know the ID.
**Right:** Enforce `principal.allows_workspace(Some(&mem.workspace))` on every primary key lookup before returning records or performing operations.
**Date:** 2026-08-30

### [API] Silent model fallback masks missing credentials and changes cost/quality
**Context:** AI model and provider route resolution when API keys or compile-time features are missing.
**Wrong:** Silently falling back to a dummy or lower-tier provider without returning status metadata, surprising the caller with degraded quality or unexpected behavior.
**Right:** Implement deterministic, zero-network resolution returning explicit status codes (`missing_secret`, `feature_disabled`) and required secret names so callers can handle degradation gracefully.
**Date:** 2026-08-30

### [Performance] DEFERRED write transactions fail instantly under a concurrent writer
**Context:** `Storage::with_transaction` while another connection or process holds the SQLite write lock.
**Wrong:** `conn.transaction()` (DEFERRED). The closure reads first, then the read-to-write upgrade returns `SQLITE_BUSY` ("database is locked") immediately, because SQLite skips the busy handler when the connection already holds a read snapshot.
**Right:** Write transactions use `TransactionBehavior::Immediate`, which takes the write lock up front and waits up to `busy_timeout`. Regression: `tests/storage_concurrency_regression_tests.rs`.
**Date:** 2026-10-05

### [Data Processing] Concurrent opens double-applied migrations and left half-applied steps
**Context:** Two threads or processes opening the same fresh or outdated database file.
**Wrong:** The runner read `MAX(version)` once and ran each step in autocommit mode. Racing openers re-ran the same step, failing with `UNIQUE constraint failed: schema_version.version` or `duplicate column`. A failed step left partial DDL behind.
**Right:** Each step runs in its own `BEGIN IMMEDIATE` and re-reads the version under the lock. `PRAGMA foreign_keys` does nothing inside a transaction, so steps that rebuild tables (v45) switch it off before `BEGIN` and restore it afterwards.
**Date:** 2026-10-05

### [Logic] WAL checksum word order comes from the magic's low bit
**Context:** Verifying SQLite WAL checksum chains (`src/sync/wal_chain.rs`).
**Wrong:** Treating `0x377f0683` as "little-endian". In SQLite, magic `& 1 == 1` means big-endian checksum words, and the stored checksum fields are always big-endian. The legacy `WalHeader::is_little_endian_checksum` naming follows the old reading and is kept for compatibility.
**Right:** Use `checksum_words_le(magic)` and `stored_checksum()`, and test against WAL files that real SQLite writes, not only self-built fixtures.
**Date:** 2026-10-05

### [Data Processing] Closing any descriptor of a live SQLite file drops SQLite's locks and loses WAL commits
**Context:** Hardening file modes of the database, `-wal` and `-shm` after every `Storage` call (`restrict_sqlite_artifact_permissions`, #189), and `prepare_database_file` on an already-open database.
**Wrong:** Opening each artifact with `O_NOFOLLOW`, calling `fchmod` on the descriptor and closing it. POSIX (fcntl) locks belong to the (process, inode) pair, so the close dropped every lock SQLite held, unseen by SQLite's unix VFS. Another process (CLI, hook client, test reader) that opened and closed the database then believed it was the last connection, checkpointed and deleted the live WAL. The server's later commits went to the unlinked WAL and were lost on restart. It also made raw-reader "row unchanged" test assertions vacuous.
**Right:** Never open a SQLite file that this process may have open through rusqlite. Restrict modes by path: `lstat`, refuse symlinks, `fchmodat(AT_SYMLINK_NOFOLLOW)` with a dev+inode re-check fallback where the flag is unsupported. Create the database `0600` with `O_EXCL` before SQLite's first open. Reproduce with a second process: in one process SQLite tracks locks per inode and never deletes the WAL. Regression: `tests/storage_posix_lock_regression_tests.rs`. The same rule applies to raw copies/uploads of the active database and to a second SQLite library copy (DuckDB scanner) in the process.
**Date:** 2026-10-05

### [Config] SQLITE_OPEN_NOFOLLOW rejects a symlink in ANY path component, including VACUUM INTO targets
**Context:** `Storage::vacuum_into` writing a snapshot to `std::env::temp_dir()` (macOS: `/var/folders/...`, where `/var -> /private/var`).
**Wrong:** Passing the path as is. The connection was opened with `SQLITE_OPEN_NOFOLLOW`, which SQLite also applies to `VACUUM INTO` and rejects with "unable to open database" when any directory component is a symlink, not only the final name.
**Right:** Canonicalize the parent directory and join the file name (as `database_open_path` already does for the database itself).
**Date:** 2026-10-05

---

## Rationalization Table

Common excuses that lead to mistakes. If you catch yourself thinking these, stop.

| Excuse | Reality |
|--------|---------|
| "Too simple to test" | Simple code breaks. A test takes 30 seconds. |
| "I'll fix it later" | Later never comes. First fix sets the pattern. |
| "Should work now" | RUN the verification. Assumptions are bugs waiting to happen. |
| "Just a quick fix" | Quick fixes become permanent. Follow the full process. |
| "I'll test after I finish" | Tests written after code are weaker. Write them first. |
| "The agent said it succeeded" | Verify independently. Trust but verify. |
| "One more attempt should fix it" | 3+ failures = architectural problem. Step back. |
| "This doesn't need a plan" | Plans prevent wasted effort. 5 minutes of planning saves hours. |
| "I know this codebase" | Read the code anyway. Memory is unreliable. |

---

## Defense-in-Depth Debugging

After fixing any bug, validate at every layer the data passes through:

1. **Entry point** — is the input correct where it enters the system?
2. **Business logic** — does the transformation produce the right result?
3. **Environment guards** — are configs, permissions, and dependencies correct?
4. **Output verification** — does the final output match expectations?

Don't stop at the first layer that looks correct. Bugs hide behind other bugs.
