# Engram Invariants

These are truths that must **always** hold. They guide implementation, testing, and code review.

## Core Memory Invariants

1. **Memory content is never empty** - Every memory must have non-whitespace content.

2. **Memory IDs are monotonically increasing** - Once assigned, IDs never change or repeat.

3. **Timestamps are RFC3339 UTC** - All timestamps stored as TEXT in RFC3339 format, parsed as `DateTime<Utc>`.

4. **Content hash is deterministic** - Same content always produces same hash (SHA256 of normalized content).

## Workspace Invariants

5. **Workspace names are normalized** - Always lowercase, `[a-z0-9_-]`, max 64 chars, no leading underscore.

6. **"default" workspace always exists** - Every query without explicit workspace uses "default".

7. **Workspace deletion moves or deletes all memories** - No orphaned memories after workspace delete.

## Tier Invariants (Critical)

8. **Permanent tier memories have no expiration** - `tier = 'permanent'` implies `expires_at IS NULL`. Enforced at write-time.

9. **Daily tier memories always have expiration** - `tier = 'daily'` implies `expires_at IS NOT NULL`. Default: created_at + 24h.

10. **Promotion clears expiration** - `promote_to_permanent()` sets `expires_at = NULL`.

## Identity Invariants

11. **Alias normalization is idempotent** - `normalize_alias(normalize_alias(x)) == normalize_alias(x)`

12. **Aliases are globally unique** - One alias cannot map to two different canonical IDs.

13. **Alias conflict is explicit rejection** - If alias exists for different identity, return error (never silently overwrite).

14. **Identity deletion cascades** - Deleting identity removes all aliases and memory links.

## Session/Transcript Invariants

15. **Chunks have bounded size** - Max 10 messages OR 8000 chars per chunk (whichever first).

16. **Chunk overlap preserves context** - Last N messages of chunk N appear as first N messages of chunk N+1.

17. **TranscriptChunk has default 7-day TTL** - Unless explicitly permanent.

## Search Invariants

18. **Search never panics on bad input** - Empty query, invalid regex, malformed filters return empty results.

19. **TranscriptChunks excluded by default** - Regular search excludes `memory_type = 'transcript_chunk'` unless explicit.

## Cache Invariants

20. **Embedding cache has bounded memory** - Max bytes enforced, LRU eviction when exceeded.

21. **Cache get never blocks writers** - Read lock for get, write lock only for put/evict.

## Concurrency & Storage Invariants

22. **SQLite connections are not shared across threads or held across await points** - `rusqlite::Connection` handles must never be held across Tokio `.await` points (`!Send`/`!Sync`). Use connection pool closures (`with_connection(|conn| ...)`).

23. **Transactions are short-lived** - No network I/O inside transactions (prevents `SQLITE_BUSY` lock starvation).

24. **All external calls have timeouts** - HTTP, embedding APIs, cloud sync.

25. **WAL replication replay is validated before the target is touched**:
    - Every frame passes `preflight_replay` before the target or its parent directory is created or opened:
      - supported page_size and exact frame size;
      - `page_number` in 1..=`MAX_ALLOWED_DB_PAGES` and `db_size_pages` <= `MAX_ALLOWED_DB_PAGES`;
      - checked offset, commit length and cumulative payload within the byte budget (`ReplayLimits`, default and ceiling 64 GiB). The page cap alone is not enough: 100M pages × 64 KiB ≈ 6.55 TB.
    - Decoded pack payloads count against the same budget as they are unpacked.
    - Packs must carry a SQLite checksum-chain context. The chain, salts, ordering and page_size consistency are verified first. This proves integrity, not authenticity.
    - Recovery writes to an owner-only staging file and replaces the target only after fsync and `integrity_check`.

26. **Archive and delta decompression has saturating streaming limits** - Readers use a saturating byte limit (`read_entry_limited`) to halt before memory or disk exhaustion.

27. **Engram never opens and closes a descriptor on a live SQLite file** - POSIX (fcntl) locks belong to the (process, inode) pair: closing ANY descriptor of the database or `-shm` drops every lock this process holds on it, bypassing SQLite's unix VFS. Another process that then opens and closes the database deletes the live WAL, and later commits are lost (G1). Permission hardening works by path (`lstat` + `fchmodat(AT_SYMLINK_NOFOLLOW)`, symlinks refused, mode only narrowed) and the database is created `0600` before SQLite's first open, so SQLite creates `-wal`/`-shm` with the same mode. Raw file reads of a database that this process has open through rusqlite (copies, uploads, recovery seeds) and loading a second SQLite library copy (DuckDB `sqlite` scanner) on it are the same hazard: code that needs the bytes goes through SQLite (`Storage::vacuum_into` / `snapshot_copy`; `replication_recover` and the DuckDB graph tools do), and MCP tools that open caller-supplied paths call `Storage::refuse_active_sqlite_artifact` (device + inode, plus resolved name for `-wal`/`-shm`/`-journal`; paths under `/dev` and `/proc`, which can alias open descriptors such as `/dev/fd/N`, are refused outright). Known open violation, documented at the code: `CloudStorage::upload`/`download` on the active path (`src/sync/cloud.rs`, not started by any binary today). Regression: `tests/storage_posix_lock_regression_tests.rs`.

## String & Multilingual Invariants

28. **String slicing never uses byte offsets from case-folded or normalized text** - Casing changes alter UTF-8 byte lengths (e.g. `ß` vs `ẞ`). Always slice via character iterators, grapheme clusters, or regex word boundaries (`\b`) to prevent char boundary panics.

## Error Invariants

29. **No unwrap() in production paths** - All fallible operations use `?` or explicit error handling.

30. **Errors include context** - Memory ID, workspace name, operation type in error messages.

31. **Validation errors list all problems** - Not just first failure.

## Agentic Governance & Security Invariants

32. **Integer ID lookups must verify workspace authorization** - Every query by integer ID must join/check against `principal.allows_workspace` (anti-IDOR).

33. **Permission modes fail closed** - Calls exceeding active permission mode return structured `{ "error": { "code": "permission_denied", ... } }` denial payloads.

34. **Model routing resolves deterministically and offline** - Zero network calls, explicit degradation reporting (`missing_secret`, `feature_disabled`) rather than silent model fallback.

35. **Harness security boundaries reject autonomous self-approval** - Automated agents are bound by negative scope (`WHAT_WE_DONT_DO.md`) and cannot authorize their own policy or harness mutations without human ADR acceptance.

## Quota Invariants (engram-cloud)

36. **Quota check happens before mutation** - Check quota, then create memory (not reverse).

37. **Storage-counted metrics query tenant SQLite** - Workspaces, Identities, Sessions counted from storage, not control plane.

38. **Quota exceeded returns structured error** - Includes metric name, current value, max value.

---

## Testing Requirements

For each invariant category:

| Category | Unit Tests | Property Tests | Golden Tests |
|----------|-----------|----------------|--------------|
| Workspace normalization | validate edge cases | idempotency, charset | - |
| Tier invariants | promotion, creation | - | - |
| Alias normalization | edge cases | idempotency | - |
| Search | empty/invalid input | never panics | fixture-based |
| Chunking | boundary conditions | size limits hold | sample conversations |

---

## Checklist for New Code

- [ ] Does it respect all applicable invariants?
- [ ] Are there new invariants to document?
- [ ] No `unwrap()` in non-test code?
- [ ] Errors include context?
- [ ] Bounded memory/concurrency if applicable?
