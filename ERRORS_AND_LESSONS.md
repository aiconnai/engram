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

### [Security] Generated-memory writebacks need provenance guards
**Context:** MCP handlers that reuse `dream_jobs` for pending generated memory.
**Wrong:** Reusing arbitrary or terminal `job_id` values, allowing caller metadata to spoof governance keys by casing, or leaking raw SQLite constraint errors.
**Right:** Validate job workspace, origin, model profile, and pending status; reject reserved metadata keys case-insensitively; map candidate collisions to domain-level conflicts.
**Date:** 2026-07-03

> **Note:** Replace these examples with real entries as errors are discovered.
> Delete the examples once you have real entries.

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
