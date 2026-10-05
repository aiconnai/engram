### Q2F — Fix reproduced Q2 backlog items outside the Unicode class [P1/P2; core; added by controller]

Source: docs/quality/rust-risk-inventory.md §7-8 (Q2 output) plus review findings from C3/Q4. Each item: reproduce with a RED test on the real entry point, minimal fix, typed error / no silent success, focused test.

- [ ] Q2-B07: `engram-cli mcp install` silently replaces a non-strict-JSON config, and a second run overwrites the `.bak`. Required: refuse (clear error) on unparseable existing config unless an explicit force flag is passed; never overwrite an existing backup (timestamped/unique backup names); test both runs.
- [ ] Q2-B09: `StorageBackend::delete_crossref` returns `Ok` while the underlying statement errors (`let _ = queries::delete_crossref(..)` in sqlite_backend/mod.rs ~267-270). Propagate the error; test with a forced failure.
- [ ] Q2-B10: `list_memories` drops an undecodable row and returns fewer rows with no error. Decide per contract: surface an error (or a structured partial-result signal) instead of silently dropping; test with a corrupted row.
- [ ] G-3 (Q4): `memory_search` result cache key ignores `limit` (limit 3,1,2 → 3 rows each). Include limit (and any other result-shaping params) in the cache key; un-ignore `contract_matrix::memory_search_limit_survives_the_result_cache` and make it pass.
- [ ] C3 follow-up: `Duration::seconds(ttl_seconds)` / chrono overflow panic hazards in src/storage/queries/core/{memory_create,memory_update,expiration,dream}.rs on huge ttl values — bound/validate with typed errors; tests with i64::MAX-ish inputs via MCP where reachable.
- [ ] C3 follow-up: audit byte slices in src/intelligence/content_utils.rs (~143,190,231,239) and src/context/bundle.rs (~499); fix any that slice user text at non-char-boundaries using src/text_util.rs helpers; RED test per real bug, otherwise document why safe.
- [ ] Update docs/quality/rust-risk-inventory.md statuses for every item touched (and for B01–B06/B08/B11/B14/B17/B18 fixed by C3/C7) — status + commit, no count rewriting.

**Aceite:** each fixed item has a RED→GREEN test on the real path; no silent success remains for these items. **Rollback:** per-item revert.
