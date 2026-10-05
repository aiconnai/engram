# Q6 — Dependencies and build time by the graph (Lane R) — report

Status: DONE_WITH_CONCERNS (concerns: shared-index sweep incident, load-flaky multimodal process tests, exception manifests mismatched, aws-sdk-s3/lru deliberately not upgraded).

Scratch evidence directory (all logs referenced below):
`<session-tmp>/520b0d42-3255-4b70-b2a8-ca81f038526a/scratchpad/` (called `$S`).

## Commits (branch claude/engram-improvement-plan-edb43d)

| SHA | Subject | Files | Clears |
|---|---|---|---|
| 6c4ab81 | fix(infra): update rustls to 0.23.45 for RUSTSEC-2026-0285 | Cargo.lock (+ progress) | RUSTSEC-2026-0285 (the only hard cargo-audit/cargo-deny failure) |
| 6c27b57 | fix(infra): update rust_decimal to 1.43.0 to drop rkyv 0.7 | Cargo.lock (+ progress) | RUSTSEC-2026-0235 (rkyv 0.7.46 no longer in Cargo.lock) |
| 35a1592 | fix(infra): update event-listener to 5.4.2 for RUSTSEC-2026-0221 | Cargo.lock (+ O1 files, see Incident) | RUSTSEC-2026-0221 |
| 572495b | build(infra): update dirs to 6.0 to dedupe dirs-sys | Cargo.toml + Cargo.lock | dedup: dirs 5.0.1, dirs-sys 0.4.1, redox_users 0.4.6 gone |
| 8cef307 | docs(harness): record Q6 dependency progress | progress file only | (progress memory; also carries the staged O1 note, see Incident) |

Scope `infra` is used because `docs/harness/bin/check-commit-msg.sh` rejects `deps` as a scope. No trailers.
Final `Cargo.lock` sha256 `753dbbdf0c5a07bf86cab46ecbda8c034b9b60484a3ed8113b5fa86319b20c8f`; `Cargo.toml` sha256 `8d9abb3a5204d712e2775b6f9f45b114d3ad26b6b71acac3f5c17536e2771af3`.
Lock-only commits: rustls, rust_decimal, event-listener are transitive (no root manifest entry exists), so there is no manifest change to pair. dirs is a root dependency, so that commit has manifest + lock together.

### Version choices and release notes read (crates.io API / GitHub releases / CHANGELOGs, fetched today)
- rustls 0.23.36 -> 0.23.45. `cargo update -p rustls@0.23.36` alone picks 0.23.43 (vulnerable), so `--precise 0.23.45`. Notes: 0.23.45 fixes GHSA-2mjx-qc3c-rqvc (affects 0.23.13-0.23.44); 0.23.44 enables ML-DSA by default in the aws-lc-rs provider and makes KeyLogFile owner-only; 0.23.43 panic fixes. Pulled along (resolver): aws-lc-rs 1.17.0->1.18.1 (FIPS 4.x switch only affects the `fips` feature, unused), aws-lc-sys 0.41.0->0.45.0 (AWS-LC 5.7.0), rustls-webpki 0.103.13->0.103.15 (ML-DSA / docs.rs only). 0.103.x was already >= the 0.103.13 patch line.
- rust_decimal 1.40.0 -> 1.43.0 (transitive via duckdb 1.4.4, `duckdb-graph`). 1.43.0 notes: "Remove rkyv 0.7 from feature bridge" (#819), backported fixes and perf (#808, #809). Dry-run showed only removals (rkyv, rkyv_derive, rend, bytecheck 0.6, ptr_meta, ahash 0.7, bitvec/funty/radium/tap/wyz, seahash, simdutf8, syn 1.0.109). Not used directly by Engram.
- event-listener 5.4.1 -> 5.4.2 (via async-channel). Notes: "Fix unbounded Send/Sync implementations on StackSlot" (#163), slab impl removed, spinlock added.
- dirs 5.0 -> 6.0 (not 7.0.0, which exists since 2026-09-05: shellexpand 3.1.1 already uses dirs 6, so 6 is the version that dedupes). README changelog for 6: only `dirs-sys` -> 0.5 / `windows-sys` 0.59. Used APIs unchanged: home_dir, config_dir, data_dir, data_local_dir.

## Duplicate / reverse-dependency inventory (two runs, as in the brief)

Argv (cwd = repo/worktree root, `--locked`, `--no-default-features`, NOT the default `openai`):
1. required: `cargo tree -d --locked --no-default-features --features "$CI_REQUIRED_FEATURES"`, list from `scripts/ci-required-features.env` (sha256 of file `818bed92...b56de8`; sha256 of the comma list `be2baee1bcf77017fd9a8f935a51e71aafa3d7838d4311344315c8e9d50fee69`):
   `openai,pdf,langfuse,watcher,multimodal,emergent-graph,ollama,cohere,voyage,meilisearch,retrieval-excellence,context-engineering,temporal-graph,compression,agentic-evolution,advanced-graph,autonomous-agent,agent-portability,grpc,dream-phase,hooks`
2. full: `cargo tree -d --locked --no-default-features --features "$CI_FEATURES"`, list from `scripts/ci-features.env` (file sha256 `c0f945c9...25072`; list sha256 `c0d8abb6e009c855ea44e5d39f9eb499665bcbe163c0297edbf78dc72de5f618`):
   `cloud,openai,pdf,langfuse,turso,meilisearch,watcher,multimodal,emergent-graph,ollama,cohere,voyage,retrieval-excellence,context-engineering,temporal-graph,duckdb-graph,compression,agentic-evolution,advanced-graph,autonomous-agent,agent-portability,grpc`

Output sha256 (before = pre-Q6 Cargo.lock sha256 `8126b68faf57838797f268e36fd394adcdf965b279fe596717b10d12b17a86ef`):
- before (`$S/dup-required.txt`, `$S/dup-full.txt`): required `6e59ba82cfa477f162d687dc99e4b5fac58ab1bd22822ff7e89a5476519c2e13` (401 lines, 24 duplicated crates); full `fe4f6159b025ea201581bda342a16f542c329cc17796c7a534c07f79dda138b2` (1378 lines, 72 crates).
- after (Cargo.lock `753dbbdf...`): `$S/dup-required-after.txt` `ac3ed7f02c3c9701c9b26d89c1260182787c0230c0129866640d20e3a7c013f6` (388 lines, 22 crates); `$S/dup-full-after.txt` `bd842c16c43b46366b1f94cea3619bc5e16a540b154e9e596b75409654628dfe` (1368 lines, 70 crates). The only crates that left the duplicate set: `dirs`, `dirs-sys`.
(Note: `cargo tree -d` lists a few "same version twice" rows, e.g. bytes/log/regex; these are tree-print artifacts of feature/dep-kind splits, not real duplicates.)

### Remaining duplicates, explained (parents from the `-d` output)
Required list (22): base64 0.21 (root + tiktoken-rs 0.5.9) vs 0.22 (axum/reqwest/tonic/jsonwebtoken); bit-set/bit-vec (fancy-regex 0.12 via tiktoken-rs vs proptest dev-dep); getrandom 0.2/0.3/0.4 (rand_core 0.6/0.9/0.10, ring, tempfile, uuid, lopdf); hashbrown 0.12/0.14/0.16 (indexmap 1.9 / dashmap 5.5 + hashlink / indexmap 2); indexmap 1.9 vs 2; itertools 0.10 vs 0.14; rand 0.8 (root) / 0.9 / 0.10 + rand_core + rand_chacha; socket2 0.5 (hyper-util, tonic 0.12) vs 0.6 (tokio); thiserror 1.0 (root, tungstenite 0.24, yaup) vs 2.0 (lopdf, meilisearch-sdk, zip 2.4.2); tower 0.4 vs 0.5; untrusted 0.7 (aws-lc-rs) vs 0.9 (ring/webpki).
Additional in the full list (turso/cloud/duckdb stacks): axum 0.6/axum-core 0.3/sync_wrapper 0.1/bitflags 1/tower-http 0.4/tower 0.4/tonic 0.11/prost 0.12/hyper-timeout 0.4/hyper 0.14/h2 0.3/http 0.2/http-body 0.4/hyper-rustls 0.25/tokio-rustls 0.25/rustls 0.22/rustls-webpki 0.102/webpki-roots 0.26/rustls-native-certs 0.7/security-framework 2/core-foundation 0.9/zerocopy 0.7/base64 0.21 = all via `libsql 0.9.30` (latest on crates.io, upstream-blocked, `turso` feature only); crypto-bigint 0.4/signature 1.6/elliptic-curve 0.12 = aws-sigv4/aws-sdk (cloud); strum 0.26 (comfy-table) vs 0.27 (duckdb); zip 2.4.2 (root, snapshot) vs 6.0.0 (duckdb dev/build chain); fallible-iterator 0.2 (libsql-rusqlite) vs 0.3; nom 7 (cexpr/bindgen) vs 8 (lopdf, iso8601); rustix 0.38 vs 1.1.
Root-controlled but NOT removable by touching root only (do not promise elimination): `thiserror 1.0` (tungstenite 0.24, yaup, libsql still on 1.x), `rand 0.8` (phf_generator, rand_chacha 0.3/rand_core 0.6 chain from libsql/ring/aws), `base64 0.21` (tiktoken-rs 0.5.9, libsql, tonic 0.11), `tonic 0.12` root vs libsql's tonic 0.11. Root axum 0.7/tower 0.5/tower-http 0.6 are the current line; the old lines are libsql's.
Possible later wins (not done, each separate PR): tiktoken-rs 0.5.9 -> 0.12.1 (newest on crates.io, large API jump, would remove base64 0.21/fancy-regex 0.12/bit-set 0.5 in the required graph; needs its own token-count regression check); lopdf 0.45 drops ttf-parser (skrifa) but `pdf-extract 0.12.1` still pins `lopdf ^0.42`, so blocked until pdf-extract moves; tokenizers 0.23.2 still depends on paste.

## Verification (per commit; ALL run in a clean detached worktree of HEAD + the commit's lock/manifest, because O1/C4 uncommitted WIP in the shared tree broke compile/fmt several times; worktree removed at the end)
Required argv: `cargo test --locked --no-fail-fast --no-default-features --features "$CI_REQUIRED_FEATURES" --tests`; `cargo clippy --locked --all-targets --all-features -- -D warnings`; `cargo clippy --locked --all-targets --no-default-features --features "$CI_FEATURES" -- -D warnings` (full-feature check). Script: `$S/verify.sh`; logs `$S/c{1,2,3,4}-{test,clippy-all,clippy-ci,status}`.

| Commit | `--tests` required (57 binaries) | clippy required (-D warnings) | clippy --all-features | clippy CI_FEATURES | cargo audit (fresh DB) | cargo deny check advisories bans licenses sources |
|---|---|---|---|---|---|---|
| baseline (before Q6) | n/a (main tree had O1 WIP) | n/a | n/a | n/a | exit 1: RUSTSEC-2026-0285 vulnerability + 3 allowed warnings (ttf-parser 0192, event-listener 0221, lru 0253) | exit 1: advisories FAILED (0285); bans/licenses/sources ok |
| 6c4ab81 rustls | 2171 passed, 1 failed (flake, below), 2 ignored | ok | ok | ok | exit 0; 0285 gone; 3 allowed warnings | ok |
| 6c27b57 rust_decimal | 2171 passed, 1 failed (different flake), 2 ignored | ok | ok | ok | exit 0; rkyv absent from lock (grep 0) | ok |
| 35a1592 event-listener | 2172 passed, 0 failed, 2 ignored | ok | ok | ok | exit 0; 0221 gone; 2 allowed warnings (ttf-parser 0192, lru 0253) | ok |
| 572495b dirs | 2172 passed, 0 failed, 2 ignored | ok | ok | ok | exit 0 | ok |
Flakes (load average 12-25 from other lanes): `multimodal::video::failure_tests::hanging_ffmpeg_times_out_is_reaped_and_leaves_no_directory` (c1) and `multimodal::process::tests::timeout_kills_child_and_descendants` (c2) assert that a child/grandchild process is gone within a timeout. Each failed once under load and passed on isolated re-run (2x each, "1 passed" / "4 passed"). They are timing-sensitive process tests unrelated to the lock; flagged as a concern, not touched. In the main tree before any worktree isolation, 2 failures were O1's uncommitted red-phase redaction tests (since committed).
Docs gate (`RUSTDOCFLAGS=-D warnings cargo doc --locked --no-default-features --features "$CI_REQUIRED_FEATURES" --no-deps --document-private-items`) at c4 in the clean tree: FAILED on `src/embedding/queue/drain.rs:26` (`[super::jobs]` links to a private item). Pre-existing doc-lint issue at the old base commit, unrelated to dependencies; NOT investigated further (O1 later changed that file). Record for the controller.
Parity check `python3 scripts/check-security-exceptions.py ...` on this branch: exit 1 only because all 10 advisory records expire 2026-09-30 (today is 2026-10-05); lane P's renewal to 2026-12-31 fixes it on merge. Not edited here per instructions.
Not run: `just backend-smoke` / `docs/harness/bin/sensors.sh` (ort-sys binary download would need network beyond the authorization; `--all-features` clippy used the existing ort cache, no new download observed: ~/Library/Caches/ort.pyke.io untouched). Required-matrix beyond ubuntu-equivalent local run (macOS arm64 only) not applicable.
Network used: crates.io index/crate downloads, RustSec advisory DB (cargo audit/deny), crates.io API and GitHub raw/API for release notes. Nothing else.

## Incremental compile/link (cheap measurement only)
Clean tree, dev profile, required features, `cargo build --locked --no-default-features --features "$CI_REQUIRED_FEATURES" --bin engram-server`: after the final lock, warm build 23.7 s wall (load ~7, includes compiling engram-core); then `touch src/lib.rs` and rebuild (incremental, no `cargo clean`): 5.0 s wall (2.8 s user). `$S/timing-*.txt`. A before/after comparison across the lock updates was NOT RUN: it needs a full dependency rebuild per lock state and load was 7-25 from other lanes, so it would be noise; the commits are lock-only except dirs, and profiles (`[profile.dev]` opt-level 2 for deps, `ci`, `ci-release`) were deliberately left untouched ("keep profiles until a measurable comparison").

## cargo audit with ignores removed (which exceptions still have a live subject)
Method: `Cargo.lock` copied to `$S/noignore/` with empty `.cargo/audit.toml`; cargo-deny with `ignore = []` (`$S/deny-noignore.toml`, config run exit 2 because that temporary config also drops the advisory db setup - audit result is the reliable one). Final lock, remaining advisories and where they live (`cargo tree -i <crate>@<ver>`, per feature):
| Advisory | Crate | Present in | Default graph |
|---|---|---|---|
| 2026-0049 / 0098 / 0099 / 0104 | rustls-webpki 0.102.8 | `turso` only (libsql 0.9.30, hyper-rustls 0.25, rustls 0.22.4) | no |
| 2026-0258 | h2 0.3.27 | `turso` only (libsql -> tonic 0.11, hyper 0.14) | no |
| 2025-0141 / 2025-0134 | bincode 1.3.3 / rustls-pemfile 2.2.0 | `turso` only | no |
| 2024-0436 | paste 1.0.15 | `onnx-embed` / `local-embeddings` only | no |
| 2026-0192 (warning) | ttf-parser 0.25.1 | `pdf` only (lopdf 0.42) | no |
| 2026-0253 (warning, unsound) | lru 0.16.4 | `cloud` only (aws-sdk-s3 1.120.0) | no |
| 2026-0235 | rkyv 0.7.46 | NOT in lock any more (fixed by this task) | n/a |
| 2026-0221 | event-listener 5.4.1 | NOT in lock any more (fixed) | n/a |
Default feature set is `["openai"]` only; `cloud` is not default. `libsql` 0.9.30 is the latest release on crates.io (2026-06-02), so the turso-only exceptions cannot be removed by upgrading.

## Exception owner / rationale / expiry mapping across manifests (REPORT ONLY — no manifest edited; lane P owns the edits and renewed 9 advisory exceptions to 2026-12-31)
Files: `.cargo/audit.toml` (9 ignores: 0049, 0098, 0099, 0104, 2025-0141, 2024-0436, 2025-0134, 0235, 0258), `deny.toml` (5 ignores: 0049, 0098, 0099, 0104, 0258), `docs/security/advisory-exceptions.toml` (10 records: those 9 + 2026-0192 as `cargo-audit:allowed-warning`), `governance/exceptions.toml` (1 record: EXC-0001).
Mismatches found on this branch (older exception-file versions; check against lane P's versions at merge):
1. Expiry: all 10 advisory records say `expires = 2026-09-30` (expired; parity script exits 1). EXC-0001 says 2026-12-31. After lane P: 9 renewed to 2026-12-31. The 10th (probably 0235 or 0192) must be checked; and 0235 should be DELETED, not renewed (rkyv is gone from the lock — `.cargo/audit.toml` ignore and its record are now dead).
2. RUSTSEC-2026-0098/0099/0104/0258 records claim `feature = "cloud default feature via AWS SDK, plus turso ..."`, `feature_gated = false`, `default_graph = true`, versions `0.101.7`+`0.102.8`, and a dependency_path through `aws-smithy-http-client -> hyper-rustls 0.24.2 -> rustls 0.21.12 -> rustls-webpki 0.101.7` / `hyper 0.14.32 -> h2 0.3.27`. In the current lock there is no rustls-webpki 0.101.7, no rustls 0.21 and no hyper-rustls 0.24; the AWS path no longer carries any of these, `cloud` is not a default feature, and the affected versions exist only under `turso` (libsql). Correct record: `feature_gated = true`, `default_graph = false`, versions `["0.102.8"]` / `["0.3.27"]`, path through libsql only. (The global ignores in `deny.toml`/`audit.toml` remain needed because deny runs `all-features = true`.)
3. All records cite `engram-core 0.22.0` in `dependency_path`; the package is 0.23.0. `metadata.as_of = 2026-07-10` is stale.
4. RUSTSEC-2026-0253 (lru, cloud) and (before this task) 0221 had no governed record even though audit prints them as allowed warnings, unlike ttf-parser 0192 which has one (`allowed-warning`). Either add a governed record for 0253 or upgrade aws-sdk-s3 (see below).
5. Schema/owner coherence: advisory records use `owner = "supply-chain"` (team) with their own schema (feature/exposure/remediation); `governance/exceptions.toml` requires `id`, `rule`, `compensating_control`, `record` and uses `owner = "platform-team"`. STANDARDS.md says all security exceptions must live in `governance/exceptions.toml` with EXC- ids; the advisory exceptions have no EXC ids and no `record`/`compensating_control`, so the two manifests are not mapped to each other.
6. EXC-0001 ("dual reqwest 0.12 and 0.13", rule `P-A.2.1`): the lock has only reqwest 0.12.28 (no 0.13) in any feature graph, so the waiver has no subject today; and the standard (P-A.2.1) requires `Cargo.toml` and any cargo-deny skip to cite the same EXC id — neither does; the standard text also says the entry should reference R-2.1.3, while the record uses P-A.2.1. Unexpired (2026-12-31), no code consequence, but misleading governance state.

## Deliberately NOT done (and why)
- aws-sdk-s3 1.120.0 -> 1.144+ (first release with `lru ^0.18.2`; 1.152.0 latest; `cargo update -p aws-sdk-s3` alone only reaches 1.121.0 because of locked aws-runtime/aws-config; `-p aws-sdk-s3 -p aws-config` pulls RustCrypto 0.11 generation: sha2 0.11/digest 0.11/p256 0.13, lru 0.18.5, but also removes crypto-bigint 0.4/signature 1.6/spki/pkcs8 duplicates). Skipped because: the only gain is clearing an "unsound" *warning* (lru panic-safety in `LruCache::pop`, in an endpoint cache), not a vulnerability; Engram calls `aws_config::BehaviorVersion::latest()` (src/sync/cloud.rs:56, src/storage/image_storage.rs:754) so an SDK jump silently moves default behavior; there is no S3/R2 canary or credentials available here (cloud tests are not in the required matrix); the SDK changelog also announces a default-HTTP-client change for 2.x (Nov 2026). Recommendation: separate task with explicit decision to pin `BehaviorVersion::v2026_01_12` (or the version in use) and a bucket canary, then bump `aws-sdk-s3`/`aws-config` together; add a governed record for 0253 meanwhile.
- No dependency removal; `async-trait` kept (optional, `multimodal`, used with `dyn`). Empty features (`emergent-graph`, `hooks`, `dream-phase`, `ollama`, `cohere`, `voyage`, `retrieval-excellence`, `context-engineering`, `temporal-graph`, `compression`, `agentic-evolution`, `advanced-graph`) left as cfg-gating contracts. No profile changes.
- thiserror/rand/base64 root bumps (would not remove the duplicates, see above).
- Exception files (`deny.toml`, `.cargo/audit.toml`, `docs/security/advisory-exceptions.toml`, `governance/exceptions.toml`): untouched (lane P owns; merge-time reconciliation noted above).

## Incident (shared git index) — needs controller attention
1. While my event-listener commit (`git commit` without pathspec) ran, O1's lane had its whole change set staged (it was in the middle of its own commit/hook). My commit swept all of it: commit 35a1592 contains 52 files of O1's observability work, `docs/OPERATIONS.md`, and O1's progress text under my message "update event-listener to 5.4.2". My own earlier check (`git diff --cached --stat`) had shown only my 2 files; the other lane staged in between. O1 then committed a docs note explaining it (9a6ed82).
2. While trying to split it I ran `git reset --soft HEAD~1`, not realizing 9a6ed82 was already on top; that detached 9a6ed82 from the branch (tree content kept staged; object still in reflog). A further `git reset --soft 9a6ed82` to restore was denied by the permission classifier, so I did not retry by other means. I verified `git diff --cached` was byte-identical (md5) to `git show 9a6ed82`, and then committed it together with my Q6 progress entry in 8cef307 (pathspec commit of the progress file only). Net result: content preserved, but history differs from what O1 expects: there is no 9a6ed82 on the branch (use `git cherry-pick`/ignore; its diff is already in 8cef307), and 35a1592 is a mixed commit. History was not otherwise rewritten. After this I used `git commit -- <paths>` for every commit.
3. Lesson recorded: with concurrent lanes sharing one index, always `git commit -m ... -- <explicit paths>` (never a bare `git commit`).

## Self-review
- Completeness: the three Q5 priorities done and verified (0285, 0235 subject, 0221), plus inventory with argv/list/hash, mapping report, one dedup upgrade (manifest+lock). Acceptance "risk real reduzido ou ganho demonstrado sem feature/API regression; duplicatas remanescentes explicadas": met (1 vulnerability + 1 unsound + 1 unmaintained-subject removed from the lock; 2 duplicated crates removed; remaining duplicates attributed).
- Discipline: no `cargo clean`, no `--no-verify`, no `git add -A`, no push, exception files untouched, one upgrade per commit.
- Open concerns: (a) incident above; (b) two load-flaky multimodal process tests; (c) docs job lint failure at `src/embedding/queue/drain.rs:26` (pre-existing/unrelated to deps, may already be fixed by the later O1 edit); (d) exception manifests mismatched as listed; (e) lru 0253 remains as an ungoverned audit warning.
