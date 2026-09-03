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
