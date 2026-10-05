# SDD ledger — plan: docs/harness/plans/2026-10-02-engram-comprehensive-improvement-plan.md

Owner decisions (chat, 2026-10-05): scope = all waves 0–4 (ADR hardening v1 accepted by owner); WAL replay byte budget = 64 GiB configurable; defer_embedding=true = enqueue for background; delivery = local commits only, no push/PR.

Spec = docs/harness/SPEC.md + harness INVARIANTS/WHAT_WE_DONT_DO/GATES + ADR + root INVARIANTS/STANDARDS (reachable).

## Lanes
- Lane R (Rust/product) — worktree engram-improvement-plan-edb43d, branch claude/engram-improvement-plan-edb43d.
  Order: C2 → Q2 → C1 → C3 → Q4 → C7 → C5 → C6 → Q3 → Q7 → O1 → Q6 → C4
- Lane P (harness/CI/docs, Python/Bash) — worktree engram-improvement-lane-p, branch claude/improvement-lane-p.
  Order: E0 → Q1a → H1 → H2 → Q5 → H6 → O3 → H3 → H4 → H5 → O4 → O2
- After both lanes: merge lane P into lane R branch (serial), then Q1b (wire Q7 report into CI), full gate, final review.

## Preflight conflict scan
| Pair / task | Shared file or interface | Finding | Ruling |
|---|---|---|---|
| Q1 / H2 / Q5 | .github/workflows/ci.yml | all three edit the same workflow | Serialize in lane P (Q1a→H2→Q5); Q1b after lane merge |
| H2 / H6 | docs/harness/bin/sensors.sh, doctor.sh | both wire lanes/checks | Serialize in lane P (H2 before H6) |
| E0 / H6 / Rust lane | docs/harness/progress.md, SPEC.md | invariant 7 requires progress updates per feature commit; two lanes would conflict | Ruling: only lane P edits SPEC.md/progress.md; lane R appends to its own log docs/harness/progress/2026-10-05-improvement-lane-r.md; E0 creates the sprint log and links both — cost if wrong: one manual merge of progress text |
| H1 → H5 | review-gate.sh verdict semantics | H5 consumes H1's pending/PASS distinction | H5 dispatch carries H1's final interface |
| H2 → H3/H4 | validate-evidence.py / schemas | H4 recorder builds on H2 validator | H4 dispatch carries H2 interface |
| C7 / C5 | src/mcp/handlers/memory_crud/create.rs | both edit create path | Serialize in lane R (C7 then C5) |
| C2 / C6 | src/sync/* | WAL changes vs sync failure tests | Serialize in lane R (C2 first) |
| Q3 | fuzz/ crate + nightly.yml; depends C3 (Rust) and Q5 (lane P) | cross-lane | Ruling: Q3 runs in lane R after C3; nightly.yml is not touched by lane P — cost if wrong: small yaml merge |
| Q4 → Q1 dependency | Q4 depends on Q1a | Q1a is CI wiring only; Q4 tests do not consume it | Ruling: weak dependency; Q4 may proceed once C1 done even if Q1a still in review — cost: none functional |
| Q6 → Q5 | Cargo.lock vs deny/audit config | Q6 late in lane R | OK |
| Q7 / Q1b | ci.yml, ci.sh | Q7 (lane R) produces report; Q1b wires it | Ruling: Q1b executed after lane merge on lane R branch |
| O1 → H2, O2 → O1 | weak/doc deps | O2 done last in lane P; O1's outcome only informs retention classes | Ruling: O2 may document O1 log class from O1 brief if O1 not yet merged |
| O4 → Q3/H5 | registry/check IDs | O4 late in lane P; consumes H3 registry | OK |
| C4 | ADR + spike | docs-first; spike under scratch, not core | OK |

Per-task self-consistency: all 25 briefs internally consistent with their file lists except: C2 cites `src/storage/migrations/mod.rs` with note to locate the current module (implementer verifies); H3 says "ADRs aceitos" — satisfied by owner acceptance 2026-10-05 recorded by E0; Q4 live SDK lane requires build/install/start authorization — Ruling: owner's "execute everything" covers local isolated build/install/start against localhost only, no publish — cost if wrong: local-only side effects; O3 hosted JEV — Ruling: unavailable (no credential provisioning), reported as such.

Ruling: owner's chat answer "Tudo, inclusive Onda 4" = acceptance of ADR 2026-07-21-agent-harness-hardening-v1; E0 records it in the ADR with date/owner/source — cost if wrong: revert ADR status line.
Ruling: no Co-Authored-By trailers (user global rule + harness invariant 5 override the session attribution reminder) — cost if wrong: amend messages.
Ruling: SDD task reviews (independent model, fresh context) serve as the cross-model review required by harness invariant 10; harness `review-gate.sh post` artifacts are produced per wave by the reviewer output with explicit `REVIEW_VERDICT:` marker when H1 lands — cost if wrong: re-run review-gate for earlier tasks.

## Task log
Setup: plan committed 630dd26; lane P worktree created at 630dd26.
E0: dispatched (sonnet, lane P) BASE=630dd26 agent a82efa0e3a6dfd9bf
C2: dispatched (opus, lane R) BASE=630dd26 agent a7536c5b576ee0739
E0: implementer DONE_WITH_CONCERNS commit 0c6ea2b. Concerns carried forward: D1 expired advisory exceptions (→Q5), D2 RUSTSEC-2026-0285 rustls 0.23.36 (→Q6/Q5), D3 --criterion removed by 1fdffc5 (→Q1a), D4 GATES required-checks stale (→H6), D5 schemas/validator not wired (→H2), D7 cloud removed from default features (info).
E0: review Approved (sonnet).
Task E0: minor (deferred): ADR Source row attributes phrase "Tudo, inclusive Onda 4" to plan note — it is the owner's chat answer, not in plan note; fix wording
Task E0: minor (deferred): no Review Canvas recorded for ADR acceptance/harness policy change; note deviation in audit §6/7
Task E0: minor (deferred): lane-P log summarizes only D1-D6; add pointer to D7-D12
Task E0: minor (deferred): progress.md Last commit stale (630dd26); Last sensors cell verbose
Task E0: minor (deferred): loose evidence SHAs (harness-contract.yml, sensors.sh attribution)
Task E0: complete (commits 630dd26..0c6ea2b, review clean)
Q1: dispatched (sonnet, lane P) BASE=0c6ea2b
Q1: implementer DONE_WITH_CONCERNS commits 1addb94, 96dfa45.
Ruling: Q1 brief acceptance command cites --criterion benches/results/benchmark_baseline.txt but that file has no time: lines; origin/main used benchmark_results.txt — accept benchmark_results.txt as the historical baseline input — cost if wrong: one-line argv change.
Ruling: ci-parity-check.sh rewrite (stale Makefile greps already failing at HEAD) accepted as within Q1 file list — cost if wrong: revert to grep variant.
Note: lane P agents must EnterWorktree(path=lane-p) first; Write hook blocks cross-worktree writes; write reports via Bash heredoc.
Q1: review Approved w/ 1 Important (contract suite not wired into ci.yml/ci.sh) → fix round 1 dispatched (resume implementer). FIX_BASE=96dfa45
Task Q1: minor (deferred): justfile ci recipe only string-grepped, not behaviorally tested
Task Q1: minor (deferred): ci-parity-check.sh lacks rg presence check (pre-existing)
Task Q1: note: GitHub Linux Test job (fetch-depth 0 needed by checker git show) not run — confirm when CI allowed
Task Q1: fix round 1/5 (2 addressed, 0 open; commits 96dfa45..cf6d969)
Task Q1: complete (commits 0c6ea2b..cf6d969, review clean) — Q1a only; Q1b pending after Q7
H1: dispatched (sonnet, lane P) BASE=cf6d969
Ruling: H1 trusted-receipt design = operator-written receipt file outside any worktree (path via CLI/env), binding task/base/head/tree/diff-sha256 + operator identity; gate compares against expected values passed by operator; missing/mismatched receipt → distinct PENDING nonzero exit; REVIEW_VERDICT marker necessary but not sufficient — cost if wrong: redesign receipt format in H4/H5.
C2: implementer DONE_WITH_CONCERNS commits 1398258 ba07ff3 d1a9840 5c113f0 f2f6e7a. Concerns: with_transaction now IMMEDIATE for all calls; pre-change delta packs refused (compat); Windows rename non-atomic; checksum endianness naming bug unfixed; INVARIANTS #25 not updated.
Q2: dispatched (sonnet, lane R, concurrent with C2 review; read-only on C2 files) BASE=f2f6e7a
C2: review Needs fixes (opus): Important staging loses 0600 mode; Important v45 FK-ON path untested; spec gap decompressed total checked post-allocation. Implementer concerns adjudicated by reviewer as Minor (IMMEDIATE txn, old packs refused — no persisted product data uses packs, Windows rename, endianness). Fix round 1 dispatched FIX_BASE=f2f6e7a (minors folded: side-file fail-open, ROLLBACK on commit failure, weak assertions, INVARIANTS #25, CHANGELOG).
Task C2: follow-up (deferred): WAL header checksum endianness naming bug (WalHeader::new emits headers verify rejects)
Task C2: follow-up (deferred): streamer flush_delta doesn't reset last_replicated_frame when nCkpt==0 (pre-existing; packs now refused fail-closed)
H1: implementer DONE_WITH_CONCERNS commits dfd8681 48033da (exit codes 0/1/2/3 PENDING/4 scope; --repo trusted copy; test-review-gate not wired → H2)
Task C2: fix round 1/5 (3 addressed + 5 minors, 0 open; commits f2f6e7a..46fc3e2)
Task C2: minor (deferred): PITR WAL-present path lacks explicit 0600 mode assertion (shares StagedTarget)
Task C2: minor (deferred): DecompressionFailed vs ReplayBudgetExceeded label when remaining == 64MiB cap
Task C2: minor (deferred): with_transaction IMMEDIATE for read-only closures → cross-process read contention (reviewer: Minor)
Task C2: complete (commits 630dd26..46fc3e2, review clean)
C1: dispatched (opus, lane R, concurrent with Q2) BASE=46fc3e2
H1: review Needs fixes (opus): case-variant receipt path accepted (macOS); git replace refs/attributes subvert scope binding; runbook runs writer's gate + gate sha not bound. Fix round 1 dispatched FIX_BASE=48033da (minors folded: node_modules exclusion, literal pathspec, write failures, prep staged view, ACL).
Task H1: minor (deferred): reviewer diff still --unified=0 (no context)
Ruling: H1's own acceptance = independent SDD review + owner decision (base gate cf6d969 cannot verify receipts) — cost if wrong: re-gate H1 later with trusted copy.
Q2: implementer DONE_WITH_CONCERNS commit a87a3ce. Reproduced panics: memory_search_compact(search.rs:732), memory_prepare_context(context_grouper.rs:118), memory_garden(gardening.rs:316), context_budget_check(compression.rs:269) on multibyte; attestation_chain_verify short hex (attestation.rs:235); stdio no catch_unwind + panic=abort → server DoS. Also: mcp install overwrites non-strict JSON + .bak; session_index max_chars=5 overflow; delete_crossref Ok on error; list_memories drops undecodable rows. → C3 owns Unicode/hex panics; others go to backlog (C3/C5/O1 or final).
Task H1: fix round 1/5 (3 addressed + minors, 0 open; commits 48033da..60343b3)
Task H1: minor (deferred): GATE_SCRIPT relative path resolved after cd → hashes/location-checks wrong file (fails closed); make absolute before cd
Task H1: minor (deferred): ACL check skipped when xattr present (ls shows @ not +)
Task H1: minor (deferred): symlinked gate file not resolved (content pinned by sha; TOCTOU only)
Task H1: minor (deferred): test_gate_inside_worktree_cannot_pass asserts only rc=3, not reason
Task H1: complete (commits cf6d969..60343b3, review clean)
H2: dispatched (sonnet, lane P) BASE=60343b3 — carries wiring of test-review-gate.sh/test-fixtures.sh/test-check-live-state.sh + D5
Q2: review Approved (sonnet).
Task Q2: minor (deferred): scripts/rust_risk_inventory.py 1,175 lines (>800 user max), analyze() ~190 lines — split lexer/cfg-tree/report
Task Q2: minor (deferred): mutation evidence not committed; undefined "Confirmed (static)" label in §7 legend
Task Q2: minor (deferred): tool + inventory in one commit (brief: separate PR)
Task Q2: complete (commits 46fc3e2..a87a3ce, review clean)
Ruling: C3 waits for C1 to finish (both edit src/mcp/handlers in the same worktree) — cost: some wall-clock.
C1: implementer DONE_WITH_CONCERNS commit 44d42f8 (IDOR on HTTP/gRPC fixed; restricted principal refused for 65 by-ID no-workspace tools; gaps: in-txn recheck, alias permission-mode skip, resources/subscribe, gRPC e2e)
C1: review Needs fixes (opus): Important context_* path-hash `workspace` alias passes claim rule (oracle); Important permission-mode bypass via graph_predict_links/cluster aliases. Fix round 1 dispatched FIX_BASE=44d42f8.
Ruling: blanket conservative denial of 65 by-ID no-workspace tools for restricted principals (anonymous loopback, namespaced tokens) accepted — stdio and keyed HTTP unaffected; anonymous loopback was already read_only/default — cost if wrong: anonymous local HTTP clients lose those tools until scoped.
Ruling: non-data catalog tools (discover_tools etc.) remain allowed for restricted principals — cost if wrong: tool-catalog metadata visible to anonymous loopback (already public in docs).
Task C1: minor (deferred): TOCTOU on pre-dispatch-only ID tools (summary_of_id, parent_id, feedback, writeback…)
Task C1: minor (deferred): resources/subscribe not principal-aware (activity oracle)
Task C1: minor (deferred): no gRPC e2e with real EngramHandler
Task C1: minor (deferred): handle_request_as default ignores principal (custom handlers must override)
C1: fix round 1 implementer done commit 6750b9b (alias canonicalization, context_* path-hash alias denied, unknown tools require admin, own-ID positives, metadata tools allowed). verify-sdk-artifacts NOT RUN (offline deps). Re-review dispatched.
C3: dispatched (sonnet, lane R, concurrent with C1 re-review) BASE=6750b9b — includes Q2-B01..B06,B08 panics
Ruling: Q2 backlog non-Unicode items (B07 mcp install overwrite, B09 delete_crossref Ok-on-error, B10 list_memories drops rows) → new mini-task "Q2F" in lane R after C5 — cost: delay only.
Task C1: fix round 1/5 (2 addressed + minors, 0 open; commits 44d42f8..6750b9b)
Task C1: minor (deferred): no catalog drift test for WORKSPACE_ALIAS_TOOLS (new tool reusing workspace as path-hash would pass)
Task C1: minor (deferred): unexplained anonymous context_record* reason-assertion exemption in workspace_auth_enforcement_tests
Task C1: minor (deferred): dispatch parser in test_every_dispatchable_name_has_a_permission_mode brittle to wrapped or-patterns
Task C1: minor (deferred): public permission_denial_for_mode still fail-open for unknown tools; permission_mode_status_report ignores write-flag escalation
Ruling: unknown tool under principal/env mode < admin now returns permission_denied (required admin) instead of tool_not_found — accepted as fail-closed, no leak; Q4 pins it with a protocol test — cost if wrong: clients see 403 vs not-found for typos under restricted modes.
Task C1: complete (commits a87a3ce..6750b9b, review clean)
Q4: dispatched (sonnet, lane R, concurrent with C3) BASE=6750b9b
H2: implementer DONE_WITH_CONCERNS commits 283fca2..df9e9c8 (9 commits; also fixed H1 ACL@ minor; offline lane in Test job)
O3: dispatched (sonnet, lane P, concurrent with H2 review; docs-only) BASE=df9e9c8
O3: implementer DONE_WITH_CONCERNS commits b135c87 fc5a688 (provenance unavailable; nothing executed). Review dispatched (haiku).
O3: review Approved w/ 1 Important (phrasings not required in schema) → fix round 1 dispatched FIX_BASE=fc5a688
H2: review Approved (sonnet).
Task H2: minor (deferred): offline-lane floor constants unpinned (lowerable silently); review-gate "Not applicable" count unbounded/unprinted; zero-count stub tests only for unit/self-test
Task H2: minor (deferred): sensors.sh full skips offline_lane when CI_STATUS=pass_with_exclusion
Task H2: minor (deferred): validate-evidence.py 1325 lines / test 1035 lines (>800) — split follow-up; uniqueItems skipped when len>4096 (fail closed instead)
Task H2: minor (deferred): check-live-state.sh no longer run against real progress.md anywhere (Last commit/sensors staleness unenforced) → H6 decide
Task H2: note: first real ubuntu-latest run unverified (apt python3-jsonschema 4.10, setfacl ACL scenario)
Task H2: complete (commits 60343b3..df9e9c8, review clean)
Ruling: advisory exceptions expired 2026-09-30 (D1): re-verify each; advisories fixable by dependency update are NOT renewed — fixed in Q6 (lane R owns Cargo.lock); unfixable ones renewed with owner Ronaldo + rationale + expiry ≤ 90 days. Security Gate may stay red on lane P until Q6 lands; integrated tree must be green — cost if wrong: temporary red gate on lane branch.
Ruling: D2 RUSTSEC-2026-0285 rustls 0.23.36 → Q6 (lane R) — cost: as above.
Q5: dispatched (sonnet, lane P) BASE=fc5a688
Task O3: fix round 1/5 (1 addressed + 2 minors, 0 open; commits fc5a688..17c6497)
Task O3: minor (deferred): Brier figures 11-decimal precision unverified (provenance unavailable)
Task O3: complete (commits df9e9c8..17c6497, review clean)
H3: dispatched (sonnet, lane P, concurrent with Q5; disjoint files) BASE=17c6497
C3: implementer DONE_WITH_CONCERNS commit a6f55ac (text_util helper; B01-B06,B08,B14,B17,B18 + bm25 + 2 extra lowercase-offset sites fixed; ttl_days now bounded 0..=36500, negatives rejected). Follow-up: Duration::seconds(ttl_seconds) sites in storage/queries/core/{memory_create,memory_update,expiration,dream}.rs may panic on huge ttl → Q2F.
C3: review Approved (sonnet).
Ruling: ttl_days bounded 0..=36500, negatives now error (were silently 24h) — accepted fail-closed input tightening; schema unchanged — cost if wrong: clients sending negative ttl get an error.
Task C3: minor (deferred): advertise ttl_days 0..=36500 in catalog/MCP_TOOLS.md
Task C3: minor (deferred): weak contains() assertion in bm25 lengthening-fold test
Task C3: minor (deferred): truncate_with_marker contract chars→bytes (fewer chars kept for multibyte)
Task C3: follow-up (→Q2F): unaudited slices in intelligence/content_utils.rs:143,190,231,239, context/bundle.rs:499; Duration::seconds(ttl_seconds) panic hazards in storage/queries/core
Task C3: complete (commits 6750b9b..a6f55ac, review clean)
Q3: dispatched (sonnet, lane R, concurrent with Q4) BASE=a6f55ac
Q4: implementer DONE_WITH_CONCERNS commit 98bd050. Findings: G-1 CRITICAL pre-existing (#189) restrict_sqlite_artifact_permissions open/close drops POSIX locks → WAL loss; G-2 SDK drift (26 nonexistent tools, 43 unknown args, 11 missing required) baselined in ratchet; G-3 memory_search cache key ignores limit; TS SDK no isError raise / timeout wrap. Live scripts NOT RUN (registry), manual offline equivalents passed.
Ruling: new task G1 (P0, lane R, opus) for G-1 before C7 — cost: none (critical data loss).
Ruling: G-2 SDK/server drift is a public SDK contract decision → owner; ratchet baseline prevents growth; reported at finish — cost if wrong: SDK users keep hitting broken methods until owner decides.
Ruling: G-3 cache key ignoring limit → Q2F batch.
G1: dispatched (opus, lane R) BASE=98bd050
Q4: review Needs fixes (sonnet): Important ratchet --update-baseline ungated/untested/not CODEOWNERS; Important per-tool aggregation masks new call-site drift. Fix round 1 dispatched FIX_BASE=98bd050 (minors folded: dangling doc refs, #[ignore] G-3 test, journey test-count floor, env clearing).
Ruling: workflow path-filter + scripts/ci.sh wiring of the SDK drift ratchet deferred to integration step (post lane merge) to avoid cross-lane conflicts on ci files — cost: ratchet unenforced in CI until integration.
Task Q4: minor (deferred): persisted_state no-mutation check covers memories/memory_tags only; positive control stdio-only
Task Q4: minor (deferred): characterization tests (TS isError/AbortError, camelCase ignored, legacy string errors) must be flipped when bugs fixed
Q5: implementer DONE_WITH_CONCERNS commits f4f547c 945a6cf (verdict pass/neutral/block; findings policy checker; supply-chain checker; bench split; 10 exceptions renewed to 2026-12-31 (unfixable libsql/turso etc.); →Q6: RUSTSEC-2026-0285 rustls>=0.23.45, RUSTSEC-2026-0221 event-listener→5.4.2). Security Audit/Cargo Deny red until Q6.
Q4: fix round 1 implementer done commit 0d36d3d (shrink-only baseline + per call-site entries + CODEOWNERS + #[ignore] G-3 + journey count floor). Integration wiring TODO: add baseline/checker to python-sdk-live.yml + typescript-sdk-live.yml paths; add test_check_sdk_contract_alignment.py + checker (--only typescript) to scripts/ci.sh step 5.
G1: implementer DONE_WITH_CONCERNS commit 974b7a6 (path-based lstat+fchmodat NOFOLLOW; also fixed prepare_database_file lock drop; 2-process regression test RED on macOS+Linux; C1 HTTP positive control added). Documented-not-fixed hazards: replication_recover copies live DB; CloudStorage upload/download on live path; duckdb ATTACH second SQLite copy. INVARIANTS renumbered (new #27).
H3: implementer DONE_WITH_CONCERNS commits 60cf3db e4b0812 (adapter+registry+fake writer; real docker smoke 14 PASS; sandbox_unit lane component floor 30).
Q3: implementer DONE_WITH_CONCERNS commit 4e8ba9a (fuzz workspace 3 targets/115 seeds, fuzz smoke 3/3 pass, Miri 13 tests pass, mutants NOT RUN locally, nightly schedule mapping fixed).
Reviews dispatched: G1 (opus), H3 (opus), Q5 (sonnet), Q4 re-review (sonnet), Q3 (sonnet). C7: dispatched (sonnet, lane R) BASE=4e8ba9a
Task Q4: fix round 1/5 (2 addressed + minors, 0 open; commits 98bd050..0d36d3d)
Task Q4: minor (deferred): drift sites per method not per call expression; TS _enclosing_method only 2-space class members
Task Q4: minor (deferred): coverage map G-1 rows still "open; owned by G1" — reconcile after G1 completes
Task Q4: complete (commits a6f55ac..0d36d3d, review clean)
Q3: review Needs fixes (sonnet): Important mutants lane whole-lib cannot complete in 350min. Fix round 1 dispatched FIX_BASE=4e8ba9a (minors folded: wall-cap grace, dated nightly pin, fuzz lock audit, contract test in plan job).
Q5: review Needs fixes (sonnet): Important rkyv RUSTSEC-2026-0235 renewal likely fixable via rust_decimal 1.43 → Q6; Important CodeQL findings step not pinned by contract test. Fix round 1 dispatched FIX_BASE=945a6cf (minors folded: provenance hint, stale ci.yml comment, supply-chain parser gaps, ledger expiry note).
Ruling: GATES.md required-checks refresh (D4) done in Q5 counts as covered; H6 verifies consistency only.
Ruling: Q6 may fetch from crates.io (index + crate downloads) to resolve dependency upgrades — owner authorized dependency upgrades; registry fetch is not a provider/production action — cost if wrong: network use limited to crates.io.
Task Q5: minor (deferred): bench PR comments removed (regression still fails PR); bench-publish swallows publish failure; exceptions files PR-editable (human merge mitigates)
Task Q5: note: supply-chain ledger entries (semgrep image, Dockerfile bases) expire 2026-11-05 → resolve digests online
G1: review Needs fixes (opus): Important replication_recover default source = live DB opened/closed in server process (same bug); Important duckdb ATTACH of live DB. Fix round 1 dispatched FIX_BASE=974b7a6 (minors: shared refuse_active_sqlite_artifact guard on caller-controlled paths, child helper inflating counts, O_EXCL mutex, macOS swap warn).
H3: review Needs fixes (opus): Important remote docker via context/config bypasses unix-only guard; Important budget test vacuous. Fix round 1 dispatched FIX_BASE=e4b0812 (security minors folded: read-back coverage, argv validator, target_sha naming, run_dir/TCB/worktree guards, log pump errors, smoke assertions, split if >800).
Task H3: note for H4: registry entry pr_title_policy runs writer-writable script with constant input — H4 scope check must protect docs/harness/ or run checks from /tcb.
Task Q5: fix round 1/5 (2 addressed + minors, 0 open; commits e4b0812..62101f8 [Q5 commits cae024c,62101f8])
Task Q5: minor (deferred): CI contract pin misses `- if:` first-key form; commented-out gate commands satisfy assertIn; flow-style permissions single-line only; security-gate contract-test step itself unpinned (consider offline lane/ci.sh)
Task Q5: complete (commits 17c6497..62101f8 excluding H3 60cf3db/e4b0812, review clean). Security Audit/Cargo Deny red until Q6 (RUSTSEC-2026-0285 rustls, 0235 rkyv via rust_decimal 1.43.0, 0221 event-listener 5.4.2).
H3: fix round 1 implementer done commits e2a2930 5d9c04c (endpoint pinning, read-back tightened, sandbox_registry.py split). Re-review dispatched (sonnet).
Ruling: #152 — document ingestion keeps chars-based limits, documented as distinct from tokens; no tokenizer integration — cost if wrong: future sub-PR to add tokenizer selection.
H6: dispatched (sonnet, lane P, concurrent with H3 re-review) BASE=5d9c04c — folds E0 minors (ADR source wording, canvas deviation note, D7-D12 pointer) and H2 minor (real progress.md live-state enforcement).
Task H3: fix round 1/5 (2 Important + minors addressed; 1 new Important: duplicate CliTests class shadows 5 tests; commits 62101f8..5d9c04c)
H3: fix round 2 dispatched FIX_BASE=5d9c04c (rename class + duplicate-class guard/floor; attached inline flags)
Task H3: minor (deferred): ceil budget allows ≤1s overrun; tmpfs parser order-sensitive; INLINE_INTERPRETERS lacks Rscript/deno/bun/pwsh; unix socket forwarding remote undetectable
G1: fix round 1 implementer done commit 9712ade (VACUUM INTO seeding for active DB; duckdb snapshot attach; shared guard). Re-review dispatched (opus). C7 committed f560a6c (awaiting its report).
C7: implementer DONE_WITH_CONCERNS commit f560a6c (MCP defer enqueue atomic; persist helper; stale lease 15min; health diagnostics; CLI create still no job; storage-level create_memory unchanged)
H3: fix round 2 done commits 93ff80b 4d341a0 (54 tests, ast duplicate guard, exact floor). Re-review dispatched (haiku). Q7: dispatched (sonnet, lane R) BASE=f560a6c
Task Q3: fix round 1/5 (1 Important + 4 minors addressed, 0 open; commit 730ede4)
Task Q3: minor (deferred): outdated job continue-on-error remains; cargo-mutants runtime unmeasured (est ~130 mutants); GH-hosted nightly not run
Task Q3: complete (commits 974b7a6..730ede4 [Q3 commits 4e8ba9a,730ede4], review clean)
C7: review Needs fixes (opus): Important 4 production callers (CLI create, CLI interactive, snapshot load, dream promotion) never enqueue → permanent Degraded. Fix round 1 dispatched FIX_BASE=f560a6c (minors folded: runbook command, lifecycle doc, diagnostics overlap, Ok(false) log, retry doc, health tests split).
Task C7: minor (deferred): multi-process lease edge (slow original claimant fail marks re-claimer job failed); checkpoint window between commit and HNSW insert (pre-existing); drain report lost on partial failure; hygiene config hard-coded Default; no test for context_seed embed_after_commit / server observer wiring
Task G1: fix round 1/5 (2 Important + minors addressed; 1 new Important: macOS /dev/fd/N bypasses dev+ino guard; commit 9712ade)
Ruling: replication_recover active-source → latest-state-only and frame/time PITR from live DB refused — accepted fail-closed (PITR from live DB was unsound); must be documented in catalog/MCP_TOOLS.md — cost if wrong: users needing PITR must copy DB first.
G1: fix round 2 dispatched FIX_BASE=9712ade (/dev/fd + /proc/fd refusal, duckdb drop order, catalog docs, markdown import + replication_status guards)
Task H3: fix round 2/5 (1 Important + 1 minor addressed, 0 open; commits 5d9c04c..4d341a0)
Task H3: minor (deferred): prefix inline-flag match over-rejects (e.g. perl -Mstrict)
Task H3: complete (commits 945a6cf..4d341a0 [H3 commits 60cf3db,e4b0812,e2a2930,5d9c04c,93ff80b,4d341a0], review clean)
H4: dispatched (opus, lane P, concurrent with H6) BASE=4d341a0
C7: fix round 1 implementer done commit 081c73f (4 producers enqueue; health counter; docs). Re-review dispatched (sonnet).
G1: fix round 2 implementer done commit bbe9203 (/dev and /proc refused, inode-only on devfs/fdesc/procfs; drop order; catalog docs; markdown import + replication_status guards). Re-review dispatched.
H6: implementer DONE_WITH_CONCERNS commits c7a8d71 a0f3dea (progress.md 1522→109 lines, history preserved w/ sha; tokens 70.8k→53.2k cl100k; doctor structural live-state; E0 minors folded). Review dispatched.
Task C7: fix round 1/5 (Important addressed; new Important: runbook claims CLI rebuild repairs flagged-without-row — false; commit 081c73f)
C7: fix round 2 dispatched FIX_BASE=081c73f (extend rebuild_derived_indexes to repair flag-without-row + runbook match)
Task C7: minor (deferred): dream/eval.rs:532 fixture creates defer=true memories without enqueue (non-test helper)
Task G1: fix round 2/5 (1 Important + 3 minors addressed, 0 open; commit bbe9203)
Task G1: minor (deferred): /dev,/proc refused even under :memory: storage; /dev/shm legitimate paths refused (fail-closed); CloudStorage upload/download hazard documented-not-fixed (not wired); TOCTOU window between guard and open
Task G1: complete (commits 0d36d3d..bbe9203 [G1 commits 974b7a6,9712ade,bbe9203], review clean)
C6: dispatched (sonnet, lane R) BASE=bbe9203+
H6: review Needs fixes (sonnet): Important structural ancestry check fails required lane after squash merges; Important shallow-clone skip silently passes doctor. Fix round 1 dispatched FIX_BASE=a0f3dea (minors: CI-tolerant bootstrap timing, pinned history sha, re-derivable token numbers; #152 attribution corrected to controller ruling).
C7: fix round 2 implementer done commit ec3093c (rebuild repairs flag-without-row). Re-review dispatched. C6 sub-commit (a) hooks landed c601d69.
Task C7: fix round 2/5 (1 Important addressed, 0 open; commit ec3093c)
Task C7: minor (deferred): ec3093c swept in C6's src/hooks/failure_tests.rs (uncompiled at that commit) — C6 notified; queued_at not refreshed on rebuild reset; rebuild count queries unwrap_or(0) (pre-existing)
Task C7: complete (commits bbe9203.. [C7 commits f560a6c,081c73f,ec3093c], review clean)
C5: dispatched (sonnet, lane R) BASE=12d2c3b
Task H6: fix round 1/5 (2 Important + minors addressed, 0 open; commits a0f3dea..96c82c5)
Task H6: minor (deferred): lane-P active log ~12k tokens growing (rotate per task later); AGENTS/CLAUDE read-list duplication kept
Task H6: complete (commits 4d341a0..96c82c5 [H6 commits c7a8d71,a0f3dea,2be03c1,96c82c5], review clean)
C5: implementer DONE commit c22e11e (vocab update after commit; helpers extracted; 6 c5_* characterization tests)
C6 sub-commits landed: c601d69 hooks, 437d0e0 multimodal, 909cdea sync (awaiting report). Q7 landed 12d2c3b (awaiting report).
C5: review dispatched (sonnet).
Q2F: brief written; dispatched (sonnet, lane R)
C5: review Approved (sonnet).
Task C5: minor (deferred): no positive vocabulary assertion after successful create; batch create lacks vocabulary update (pre-existing, follow-up); bench inconclusive (noisy)
Task C5: complete (commit c22e11e, review clean)
C6: implementer DONE_WITH_CONCERNS commits c601d69 437d0e0 909cdea (hooks drain race RETURNING, ttl clamp, payload caps; multimodal run_bounded + FramesDir; sync streamer generation fix, cloud download atomic, worker retry). Review dispatched (opus).
Q7: implementer DONE_WITH_CONCERNS commits 12d2c3b 8e7f76f (runner+capture marker binding, v2 corpus LLM-labeled floors proposed-pending-review, anchored:false). Real candidate FAILS: extract_mixed 12.4× ceiling (bf45c0b #90). Q1b wiring snippet in report. Review dispatched.
Ruling: new task P1 (perf regression fix: find_case_insensitive_match + per-op chmod) in lane R — cost: none.
Ruling: Q1b wires the candidate runner as NON-blocking (report-only) for the Criterion half and floors until floors are anchored by independent review and P1 lands — cost if wrong: perf regressions not blocking in CI meanwhile.
P1: brief written; dispatched (opus, lane R). Q7 review dispatched (sonnet).
C6: review Needs fixes (opus): sync approved; Important hooks commit not self-contained (file in ec3093c); Important panic isolation no-op under release panic=abort (doc claims otherwise); Important ingest dedup raw vs normalized workspace. Fix round 1 dispatched.
Ruling: no history rewrite — hooks sub-PR recorded as including ec3093c's src/hooks/failure_tests.rs; owner can squash/reorder at PR time — cost: non-atomic commits locally.
Ruling: memory_ingest_media retry dedup (returns existing memory, ignores retry's content/tags/importance, deduplicated:true) accepted as defined dedup semantics; documented in catalog — cost if wrong: clients expecting update-on-retry.
Task C6: minor (deferred): hooks fixes not reachable from live server (HookContext::new(None,None), SessionStart drain not wired); SyncWorker not started by any binary; media_assets.file_hash globally UNIQUE → cross-workspace ping-pong (schema limitation); pub frames_dir semver-visible; vision errors wrapped Internal; screencapture unbounded; ingest/describe/search_by_image read whole files uncapped
Q7: review Approved-with-fixes (sonnet): 3 Important trust-semantics (forgeable anchor, vacuous supervisor default, pass with unaccepted floors). Fix round 1 dispatched FIX_BASE=8e7f76f.
H4: implementer DONE_WITH_CONCERNS commits 2bdf942 1501600 (runner/scope/recorder, 62 unit tests, lane comp runner_unit floor 62, docker smoke 4/4). Review dispatched (opus).
C4: dispatched (opus, lane R, docs+scratch spike only)
H4: review Needs fixes (opus): Important lexical symlink check bypassed by 2-hop chain; Important Rust inline test weakening undetected; Important writer_concurrency lock per runs root (claimed enforced). Fix round 1 dispatched FIX_BASE=1501600.
Task H4: minor (deferred): check_repo git status honours operator's local filter config; H4 tests not written test-first (mutation RED substitute)
Q2F: implementer DONE_WITH_CONCERNS. On lane: ab22911 62c266b 98103b0. On branch tmp-q2f (scratch worktree q2f-wt, ff-able from 98103b0): 9e0b306 175c977 1e4af34 5100634 ed2be3e. Agent's `git update-ref` of lane branch was DENIED by permission classifier → controller will NOT perform the ref move on its behalf; will ask owner at integration (ff-merge tmp-q2f). Controller unstaged leftover staged files in shared index (list.rs, memory_read.rs, tests/list.rs, contract_matrix.rs, lane-r log) to avoid them leaking into other agents' commits (index-only).
Note: shared-worktree pre-commit hook blocked by other agents' WIP clippy errors (text_util.rs unusual_byte_groupings — P1?, hooks/post_tool_use.rs explicit_counter_loop — C6 fix).
Q2F review dispatched (sonnet).
Cleanup TODO at end: scratch worktrees p1-before, p1-after, q2f-wt (registered git worktrees under scratchpad).
Q2F: review Needs fixes (sonnet): Important context_record_artifact/meilisearch ttl Duration::seconds panic reachable; overclaimed test name. Fix round 1 dispatched on tmp-q2f in q2f-wt (minors folded: normalized envelope for set_expiration, row id in list error + docs, exact assertions, mcp install unchanged-content backup skip/atomic write/--force docs). FIX_BASE=ed2be3e (tmp-q2f).
Task Q2F: minor (deferred): result cache invalidate_for_workspace ignores workspaces/global; cache key lacks caller principal (check vs C1 auth — principal-scoped results could be served cross-principal?)
Q7: fix round 1 implementer done commit 4d067f1 (trusted --floors-anchor ancestry, --require-supervisor, --candidate-dir trusted root, floors.accepted). Re-review dispatched. C6 fix commits landed: df2f97c 7abedef b3fb284 (awaiting C6 report).
Task Q7: fix round 1/5 (3 Important + minors addressed, 0 open; commit 4d067f1)
Task Q7: minor (deferred): hot-path baselines in candidate budgets.json not anchored (candidate can raise baselines) → anchor budgets like floors (Q1b/follow-up); main-branch push anchor = merge-base == SHA → accepted always false (doc: use first parent); --output dir unlink uncaught error
Task Q7: complete (commits 12d2c3b, 8e7f76f, 4d067f1; review clean). floors.accepted=false until independent human label review; extract_mixed regression → P1.
C6: fix round 1 done commits df2f97c 7abedef b3fb284. Re-review dispatched (sonnet).
O1: dispatched (sonnet, lane R)
H4: fix round 1 done commits e0021c9 240d0a4. Re-review dispatched (sonnet).
Task C6: fix round 1/5 (3 Important + minors addressed, 0 open; commits df2f97c 7abedef b3fb284)
Task C6: minor (deferred, fix before merge): catalog/MCP_TOOLS.md "other workspaces are unaffected" overclaims (file_hash UNIQUE re-points asset across workspaces)
Task C6: minor (deferred): unqualified panic-isolation claim in lane-R log line ~343; reset_interrupted_sync last_error unbounded growth in crash loop; worker Pull guard canonicalize-only (download_checked unused in prod); pid_alive zombie flake risk; waitid EINTR retry
Task C6: complete (commits c601d69 437d0e0 909cdea df2f97c 7abedef b3fb284 + ec3093c's failure_tests.rs; review clean)
Task H4: fix round 1/5 (3 Important + minors addressed; 1 new Important residual: additive #[cfg(any())] / #![cfg] test disabling undetected; commits 1501600..240d0a4)
H4: fix round 2 dispatched FIX_BASE=240d0a4
Q2F: fix round 1 done on tmp-q2f: df554f5 c6d034a c8d4215 669b41c a1a19c4 (more chrono sites bounded, normalized set_expiration, NL ignored_date_filter signal, atomic mcp config). Re-review dispatched. Agent stopped (lingering bg).
P1: implementer DONE_WITH_CONCERNS commits 772e84a 0a46e9b (extract_mixed 229µs→5µs; Q7 runner pass ratio 0.496; storage chmod not the bottleneck — get_memory access-count UPDATE serializes readers). Review dispatched.
Q6: dispatched (sonnet, lane R; crates.io + advisory-db network authorized)
Task Q2F: fix round 1/5 (1 Important class + minors addressed, 0 open; commits on tmp-q2f ed2be3e..a1a19c4)
Task Q2F: minor (deferred, fix before merge): mcp.rs write_atomic temp file briefly default-mode before chmod (0600 config world-readable window) and rename replaces symlinked config (canonicalize first)
Task Q2F: minor (deferred): legacy {"error":string} on memory_create/memory_boost/context_record_artifact out-of-range paths (wire-format follow-up); ignored_date_filter has no consumer; write_unique_backup prefix match
Task Q2F: complete (lane commits ab22911 62c266b 98103b0 + tmp-q2f 9e0b306..a1a19c4 pending owner-approved ff-merge; review clean)
P1: review Approved (opus).
Ruling: stale entity_extractor_new/default baseline (4.96ms vs ~2µs) NOT rebaselined now — proposed for a reviewed per-runner-class rebaseline task (owner); path effectively ungated meanwhile — cost: regressions on that path undetected until rebaseline.
Task P1: minor (deferred): LowercaseIndex offset map reserves 16B/byte for non-ASCII (size by char count / u32); one Σ forces exact scan for whole text; index built even when org/concept extraction disabled; alloc test regex cache-pool sensitivity; no long-text adversarial case
Task P1: follow-up: Q7 readers_N bench measures write-lock contention (get_memory access_count UPDATE) — add read-only variant; G1 per-op chmod re-assert ≈2/3 of trivial point read → owner decision (restrict to write/open paths)
Task P1: complete (commits 772e84a 0a46e9b, review clean)
H4: fix round 2 done commits 26d43ea f96b200 (cfg_gate_added, lock refusal). Re-review dispatched (haiku).
H5: dispatched (opus, lane P) BASE=f96b200
Task H4: fix round 2/5 (1 Important + 1 minor addressed, 0 open; commits 240d0a4..f96b200)
Task H4: minor (deferred): multi-line cfg attribute continuation lines not parsed (first line only); assertion-weakening on same line count undetected; evidence integrity not a signature (same-user host process can rewrite bundle); offline lane +75s
Task H4: complete (commits 96c82c5..f96b200 [H4 commits 2bdf942,1501600,e0021c9,240d0a4,26d43ea,f96b200], review clean)
O2: dispatched (sonnet, lane P, concurrent with H5; no history rewrite)
C4: implementer DONE_WITH_CONCERNS commit 1c246c8 (Proposed ADR: /dev/fd not viable; dirfd shim viable but deferred; trusted-directory chain proposed; spike in scratchpad engram-c4-spike). Review dispatched.
C4: review Needs fixes (sonnet): E overstated (no file/sidecar ownership check); repro steps missing + spike ephemeral; F7 counts unsupported; lock-drop claim unhedged. Fix round 1 dispatched.
Ruling: commit C4 spike under docs/decisions/assets/2026-10-05-c4-spike/ (not a workspace member, not built by CI) so ADR evidence survives — cost: ~1.5k lines of non-product code in repo.
O1: implementer DONE_WITH_CONCERNS — O1's 51 files were swept into Q6's commit 35a1592 (index race; message says event-listener update) + 9a6ed82 note. No history rewrite (owner may split at PR time).
Ruling: keep 35a1592 as-is locally; record that it mixes Q6 (Cargo.lock event-listener) + O1 (51 files); split at PR time if owner wants — cost: misleading commit message.
O1: review Needs fixes (opus): Important 5/9 alerts ok with no samples (contradicts docs/acceptance); Important vacuous recovery redaction test. Fix round 1 dispatched.
Task O1: minor (deferred): RED evidence only for provider-body/panic tests (others guards written after code); panic test output noise
H5: implementer DONE_WITH_CONCERNS commits 8950f47 7e4a2c9 (merge-gate.py eligible/refused; receipt v2 REVIEWER/POLICY_VERSION/REVIEW_CONTEXT — H1 accepts v1 only; 7 hardcoded CI contexts incl. non-required Security Gate). Review dispatched (opus).
Q6: implementer DONE_WITH_CONCERNS commits 6c4ab81 (rustls 0.23.45) 6c27b57 (rust_decimal 1.43 drops rkyv) 35a1592 (event-listener 5.4.2 + O1 files) 572495b (dirs 6.0) 8cef307 (docs + O1's 9a6ed82 content). cargo audit/deny now pass (2 allowed warnings). Q6 ran `git reset --soft HEAD~1` on the shared branch (detached O1 note commit 9a6ed82; content preserved in 8cef307). Docs gate: cargo doc -D warnings fails at src/embedding/queue/drain.rs:26 private link (verify on HEAD at integration). aws-sdk-s3 not upgraded (lru 0253 unsound warning; behaviour-version risk). Exception-file mismatches listed → reconcile at integration with lane P's renewed files.
Q6: review Approved (sonnet). Important items are integration-only (mixed 35a1592; exception list): handled at integration.
INTEGRATION TODO (exceptions): delete RUSTSEC-2026-0235 from .cargo/audit.toml + advisory-exceptions.toml; correct 0098/0099/0104/0258 records (feature_gated, turso-only, versions 0.102.8/0.3.27, engram-core 0.23.0, as_of); add governed record for lru 0253 (cloud) and 0221 removal; EXC-0001 no subject; deny ignore list keeps turso ids.
INTEGRATION TODO: re-check docs gate (drain.rs:26 private intra-doc link); flaky multimodal process tests under load (consider retry/longer timeout).
Task Q6: complete (commits 6c4ab81 6c27b57 35a1592[lock hunk] 572495b 8cef307, review clean)
H5: review Needs fixes (opus): CRITICAL CI provenance spoofable by PR-defined same-named jobs (probe confirmed); Important receipt v2 has no producer and H1 rejects v2; Important test enshrines cross-suite rerun laundering. Fix round 1 dispatched.
Ruling: reruns allowed only within the same check suite; any completed non-success in another suite on the head refuses — cost if wrong: flaky required job needs a human re-run in-suite.
Ruling: H1 accepts receipt v2 as superset of v1 + producer subcommand receipt-template; merge-gate consumes v2 — cost: H1 TCB change re-reviewed in H5 round.
O2: implementer DONE_WITH_CONCERNS commits a40430d e6807e3 980737f 695b553 19a6218 (policy + retention-manifest.py + 73-entry review-raw manifest; disposable-clone proof; real untrack NOT applied — owner decision; 7 .raw flagged sensitive by name).
O4: dispatched (sonnet, lane P, concurrent with H5 fix)
C4: fix round 1 done commit c099ca2 (spike committed to docs/decisions/assets, E file-ownership, F7 reconciled 0/140, hedges; .semgrepignore touched). Re-review dispatched (haiku).
O2: review Approved (sonnet).
Task O2: minor (deferred): manifest field format validation (sha256/git_blob/size types, unique paths); regex fullmatch; symlink/non-regular restore tests; backup root must be outside repo; restore TOCTOU undocumented; "no gc" test greps help text only; early lane floors (unit 60, self-test 15, fixtures 30, live-state 15, review-gate 30, lane-contract 8) not exact despite comment
Ruling: real-branch untrack of review-raw .raw files NOT applied — owner decision (7 files flagged sensitive by name: e-mail/home-path/credential heuristic; human should inspect before any public release).
Task O2: complete (commits a40430d e6807e3 980737f 695b553 19a6218, review clean)
Task C4: fix round 1/5 (4 Important + minors addressed, 0 open; commit c099ca2)
Task C4: minor (deferred): .semgrepignore touched (scoped to spike dir); rust_risk_inventory will list spike .rs files on regeneration; foreign-owned file E check untested; BSD not run
Task C4: complete (commits 1c246c8 c099ca2, review clean). ADR Proposed → owner decision; follow-up: potential macOS SQLITE_READONLY shm availability bug (repro in ADR).
O1: fix round 1 done commit 96d41bb (alerts gated on denominators; file-backed recovery redaction; recovery outcome from engine Result — fixed self-found bug). Re-review dispatched.
Task O1: fix round 1/5 (2 Important + 6 minors addressed, 0 open; commit 96d41bb)
Task O1: minor (deferred): storage-mode warning fully redacted loses operator guidance (use static message); gRPC denials without authorization_checks increment; handler_panic/mcp_timeouts denominator includes non-handler rejections; unaudited log sites in sync/hooks/multimodal/watcher (RISK-0002)
Task O1: complete (commits 35a1592[O1 files] 9a6ed82(orphaned; content in 8cef307) 96d41bb, review clean)
LANE R: all tasks complete (C2 Q2 C1 C3 Q4 G1 C7 C5 C6 Q3 Q7 P1 Q2F O1 Q6 C4). tmp-q2f pending ff-merge (owner approval).
Integration step 1: owner approved ff of tmp-q2f; ff no longer possible (lane advanced) → dropped 14 dirty files byte-identical to ed2be3e (lossless) and merged tmp-q2f: merge commit 77741c3 (progress-log append conflict resolved keeping both).
H5: fix round 1 done commits bcbb568 0d8c7c1 5b2fa0f 280698c 403bf3e. Re-review dispatched.
O4: implementer DONE_WITH_CONCERNS commits a9809d1 1f5879d c8581cc. Review dispatched.
O4: review Approved (sonnet).
Task O4: minor (deferred): no test/doctor assertion of standing-checks.yml concurrency block; /work mounted rw (python checks would trip gate_mutated_checkout); failures outside runner leave no alert; only goal is a constant canary (Q3 lanes need H3 registry/image change); owner handle placeholder `harness-maintainers` (owner to set)
Task O4: complete (commits a9809d1 1f5879d c8581cc, review clean)
Task H5: fix round 1/5 (1 Critical + 2 Important + minors addressed; 1 new Important: protected CI path set incomplete; commits 7e4a2c9..403bf3e)
H5: fix round 2 dispatched FIX_BASE=403bf3e
INT brief written. Dry-run merge: only conflict docs/quality/retrieval-performance-policy.md.
Task H5: fix round 2/5 (1 Important + minors addressed, 0 open; commits 403bf3e..3a373e7)
Task H5: minor (deferred): base_references() fails open (empty) on git error; required_jobs indentation-dependent; >100-char markdown lines; broad protected set (any scripts/** or .github/** PR → human)
Task H5: complete (commits 8950f47 7e4a2c9 bcbb568 0d8c7c1 5b2fa0f 280698c 403bf3e cd5e7f3 29d4f7e 3a373e7, review clean)
LANE P: all tasks complete (E0 Q1a H1 H2 O3 Q5 H3 H6 H4 O2 H5 O4).
INT: dispatched (opus) BASE=77741c3 (lane R) + lane P 3a373e7
INT: DONE commits ee8c4ea..4868d49 (make ci 0, sensors full PASS, audit/deny ok). Controller check: git diff --check over branch fails only on byte-exact artifacts (C4 spike evidence traces, 2 fuzz trailing-ws seeds) → fix via .gitattributes -whitespace in final wave. Final review dispatched (opus).
FINAL REVIEW (opus): Ready with fixes; 0 Critical; Important: mcp.rs write_atomic/backup modes + symlink; MCP_TOOLS "other workspaces unaffected" overclaim; CHANGELOG missing client-visible changes; harness review record/progress table/SPEC out of sync + SDD evidence git-ignored; offline-lane floors not exact. Findings in final-review.md.
Ruling (accepting reviewer disagreement): SDD review evidence archived into docs/harness/reviews/2026-10-05-comprehensive-improvement/ (no review-gate receipts claimed) — cost: ~430KB markdown in repo.
FINALFIX: dispatched (opus) BASE=4868d49 (one fix wave)
