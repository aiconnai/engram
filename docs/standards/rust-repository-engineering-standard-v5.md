# Rust Repository Engineering Standard

**Status:** Draft v5

**Document owner:** TBD

**Review date:** 2026-09-03

**Audience:** maintainers, contributors, reviewers, CI systems, and coding agents

**Scope:** Rust applications, services, libraries, command-line tools, and mixed workspaces

**Purpose:** convert engineering preferences into explicit, reviewable, and enforceable rules without confusing portable standards with repository-specific history

**Technical review baseline:** Rust 1.90.0 and the Cargo, Tokio, nextest, proptest, GitHub Actions, DuckDB, and duckdb-rs behavior reviewed on 2026-09-03. Exact repository pins that are not yet recorded remain open in Appendix B.

## 0. Normative language, rule identity, and exceptions

[R-0.1] This document uses three requirement levels.

| Term | Meaning |
| --- | --- |
| MUST | A merge-blocking or release-blocking invariant. An exception requires a named owner, written rationale, defined scope, compensating controls, and an expiry date. |
| SHOULD | The default engineering choice. A different choice is acceptable when the pull request or decision record states the tradeoff and evidence. |
| MAY | An optional technique selected according to the product, threat model, compatibility requirements, and measured benefit. |

[R-0.2] Only capitalized MUST, SHOULD, and MAY are normative. A sentence qualified by wording such as "when practical" or "where useful" is guidance and MUST be written as a SHOULD rather than as a hedged MUST.

[R-0.3] Sections 1 through 11 define the portable standard. Appendix A records the current repository profile, measured baselines, local exceptions, and repository-specific controls. Appendix B records unresolved implementation decisions. Repository history and temporary facts MUST remain in the appendices rather than being presented as universal Rust rules.

[R-0.4] Every normative paragraph MUST begin with a stable identifier. Portable rules use `R-<section>.<ordinal>` or `R-<section>.<subsection>.<ordinal>` when a subsection exists. Repository-profile rules use the equivalent `P-<appendix>.<subsection>.<ordinal>` form. An identifier MUST NOT be renumbered once assigned. A removed rule retains its identifier with the text `Withdrawn in vN` so exception records, evidence, and historical reviews remain addressable.

[R-0.5] Every MUST rule MUST map to at least one verification control in Section 11 or to a repository-profile control that Section 11 references. The documentation gate MUST reject an unidentified normative paragraph, a missing control mapping, a mapping to an unknown rule, or an unknown rule referenced by an exception.

[R-0.6] Repository exceptions MUST live in `governance/exceptions.toml`. The governance gate MUST parse that file, validate every required field, reject duplicate exception IDs, confirm that the referenced rule exists, and fail when `expires` is invalid or earlier than the current date. Missing, malformed, and expired entries MUST fail closed before any dependent waiver is accepted.

```toml
[[exception]]
id = "EXC-0001"
rule = "R-2.1.3"
owner = "dependency-owner"
rationale = "reqwest 0.12 isolates the legacy-TLS host from the rustls fleet"
scope = "crates/legacy-client"
compensating_control = "cargo-deny advisories still apply to both dependency lines"
record = "ADR-0014"
expires = "2026-12-31"
```

[R-0.7] Every policy waiver in code or configuration MUST cite an active `EXC-` identifier. This includes lint allowances, dependency-policy skips, disabled security checks, and equivalent suppressions. An ignored test MUST carry a reason string that cites either an active `EXC-` identifier or a tracked `ISSUE-` identifier whose record names an owner and the required replacement job or removal condition. The governance gate MUST reject waiver syntax without the required reference.

[R-0.8] An exception MUST NOT waive a legal, regulatory, contractual, or externally imposed security obligation. The exception owner MUST review the compensating control before renewal.

[R-0.9] A rule that depends on tool, compiler, crate, engine, service, or platform behavior MUST state the version against which the claim was validated. When the exact repository pin is not yet known, the rule MUST state the review date and Appendix B MUST track the missing pin. A pin or supported-target change MUST trigger revalidation of every rule tagged to that component.

## 1. Build velocity, profiles, and linkers

### 1.1 Dependency features

[R-1.1.1] Direct dependency declarations MUST enable only the optional features required by a supported build or runtime path. Transitive feature activation MUST be inspected, although it cannot always be controlled by the immediate workspace.

[R-1.1.2] Broad feature bundles such as `tokio/full` MUST NOT be used unless the pull request identifies the required components and explains why a narrower declaration is not viable.

[R-1.1.3] HTTP clients SHOULD disable default features and select the intended TLS, proxy, compression, DNS, and certificate behavior explicitly. The exact feature set remains a repository decision because requirements vary by service and target.

[R-1.1.4] Feature growth MUST be reviewed with `cargo tree -e features` under the package set, target, and command class that represent the supported build. The displayed tree is an analytical view, not proof that every invocation resolves an identical graph. Dependency-pruning tools MAY support the review, but their output remains advisory until false positives, build dependencies, development dependencies, generated code, and target-specific edges have been examined.

### 1.2 Crate boundaries

[R-1.2.1] Workspace boundaries SHOULD follow change frequency, ownership, dependency direction, test isolation, and measured compilation behavior rather than directory aesthetics. A crate is a compilation unit, but additional crates also add dependency edges and linking work.

[R-1.2.2] Stable domain types and low-churn abstractions SHOULD remain below frequently changed service code. Large crates with repeated serial compile tails SHOULD be split only after timing data identifies a meaningful boundary. Splitting without a dependency-direction plan can duplicate monomorphization and deepen architectural coupling.

### 1.3 Development, CI, and release profiles

[R-1.3.1] The following configuration is a starting baseline, not a universal optimum. Canonical commands MUST select custom profiles explicitly with `--profile`. Scripts, artifact paths, and cache keys MUST account for Cargo writing custom-profile output to `target/<profile>/`, such as `target/ci/` rather than `target/debug/`.

```toml
[profile.dev]
opt-level = 0
debug = "line-tables-only"
incremental = true

[profile.dev.package."*"]
opt-level = 2
debug = false

[profile.ci]
inherits = "dev"
incremental = false

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1

[profile.ci-release]
inherits = "release"
lto = "off"
codegen-units = 16
```

[R-1.3.2] `cargo test` selects Cargo's built-in `test` profile, which inherits from `dev`. Custom profiles such as `ci` and `ci-release` have no effect unless the command selects them. CI documentation and the canonical task runner MUST expose the selected profile, package set, target, and feature set.

[R-1.3.3] The optimization level for third-party crates SHOULD be selected by benchmark. Level 2 is the default baseline. Level 3 is appropriate only when improvement to representative development or test workloads justifies the additional cold-cache compilation cost. Selected package overrides MAY replace the workspace-wide `"*"` override when a small number of runtime-heavy dependencies dominate local execution.

[R-1.3.4] Package overrides do not guarantee that all generic code from a dependency is optimized at the dependency's level because monomorphization can occur in the consuming crate. Acceptance MUST therefore be based on end-to-end compile and runtime measurements rather than profile inspection alone.

[R-1.3.5] Build scripts, procedural macros, and their dependencies are governed by `build-override`, not by `[profile.dev.package."*"]`. A repository MAY benchmark `[profile.dev.build-override] opt-level = 3` when macro or build-script execution is a measured bottleneck, but it MUST record the cold-cache cost and representative local benefit before adoption.

[R-1.3.6] Debug information for third-party crates SHOULD stay disabled in development profiles unless the team routinely steps into those dependencies. Local workspace code SHOULD retain enough information for actionable backtraces, with fuller debug information enabled locally when interactive debugging requires it.

[R-1.3.7] A fast `ci-release` profile MAY omit LTO as a pre-merge signal. It is not production-artifact validation. Before deployment, the release gate MUST build the deployable binary or image through the exact production path, run artifact-level smoke or integration tests against that output, and promote the same digest. Running tests with release-like compiler flags alone does not prove that the deployed artifact was exercised.

[R-1.3.8] `panic = "abort"`, symbol stripping, split debug information, and build identifiers are product decisions. They MUST NOT be copied into a generic profile without evaluating crash diagnostics, FFI behavior, graceful shutdown, binary-size goals, and incident-response requirements.

### 1.4 Linkers and compilation cache

[R-1.4.1] On `x86_64-unknown-linux-gnu`, Rust 1.90.0 uses the bundled LLD linker by default. This claim was validated against Rust 1.90.0 on 2026-09-03. The repository MUST inspect the pinned toolchain, target, `.cargo/config.toml`, compiler flags, and representative artifact before adding an override. For ELF artifacts, `readelf -p .comment <binary>` is one acceptable evidence method. The documented stable opt-out is `-C linker-features=-lld`.

[R-1.4.2] Linux development and CI MAY switch to `mold`, or override the linker on another target, when measurements against the real baseline show a material gain and compatibility tests cover the supported artifacts. macOS SHOULD default to supported native tooling unless an alternative linker has been benchmarked and is actively maintained for the target architecture.

[R-1.4.3] Linker selection MUST be target-specific and MUST fail with a clear configuration error or a documented fallback. A workstation-only linker MUST NOT silently become a production build dependency.

[R-1.4.4] A shared `sccache` backend SHOULD use least privilege. Untrusted pull-request jobs MUST NOT receive write credentials. Fork-originated jobs MUST NOT receive cache, repository, or cloud secrets. Cache namespaces SHOULD include the compiler, target, profile, and relevant configuration inputs. CI MUST publish cache statistics so regressions, ineffective keys, and unexpected writes remain visible.

[R-1.4.5] Container builds MAY use dependency-layer techniques such as `cargo-chef`, provided the final production artifact still comes from the single approved release path.

## 2. Dependency hygiene and supply-chain controls

### 2.1 Graph integrity

[R-2.1.1] `cargo tree -d` reports packages present in multiple versions. The `(*)` marker means that a dependency subtree was already displayed; it is not evidence of another compiled version. Reviews MUST identify each duplicate by complete package ID, inspect reverse dependencies, and distinguish normal, build, development, and target-specific edges.

[R-2.1.2] Duplicate analysis SHOULD run separately for each supported target and relevant command class. Versions that exist only on mutually exclusive targets MUST NOT be reported as a single-artifact cost without evidence that one build actually compiles both.

[R-2.1.3] A duplicate that the repository intentionally carries for protocol, TLS, platform, compatibility, or migration reasons MUST be recorded as an active exception under R-0.6. A comment in `Cargo.toml` MAY explain the context, but it does not replace the expiring exception record.

[R-2.1.4] Shared infrastructure dependencies SHOULD be declared through `[workspace.dependencies]`. This centralizes versions and feature choices while permitting explicit package-level exceptions.

[R-2.1.5] Deployable applications MUST commit a lockfile. Protected CI and release commands MUST use `--locked` so resolution cannot change silently. Hermetic or offline jobs MAY use `--frozen` after every registry, Git, and tool input has been provisioned. Library lockfile policy MUST be explicit and consistent with how the library is built, tested, and published.

[R-2.1.6] Git dependencies MUST use immutable revisions in the manifest, such as `rev = "<full sha>"`. Branches and movable tags MUST NOT be used in protected builds. A committed lockfile records a precise Git revision, but the manifest-level rule keeps the intended revision visible and prevents routine updates from silently moving the dependency.

### 2.2 Feature resolution and matrix testing

[R-2.2.1] The workspace root MUST declare `resolver = "2"` or `resolver = "3"` explicitly in `[workspace]`. The selected resolver MUST be covered by the canonical dependency and feature-resolution evidence. A virtual workspace MUST NOT rely on Cargo's implicit resolver default.

[R-2.2.2] The minimum feature gate MUST cover the plain default feature set, `--no-default-features`, every supported feature in isolation, `--all-features` when that combination is valid, and selected combinations that represent real deployments.

[R-2.2.3] Every deployable package MUST have an explicit release feature set. The release gate MUST inspect `cargo tree -e features` for the exact package, target, lockfile, and release feature set and MUST fail when any prohibited development, test, insecure, or diagnostic feature appears in the resolved artifact graph.

[R-2.2.4] `cargo-hack` MAY implement the matrix. `--each-feature` covers isolated features and baseline states. `--feature-powerset --depth N` covers bounded combinations. `--mutually-exclusive-features` controls generated powerset combinations but does not define the product contract by itself. These flags were validated against cargo-hack 0.6.17 or later on 2026-09-03.

[R-2.2.5] Mutually exclusive features SHOULD be avoided because Cargo features are additive across a dependency graph. When exclusion is unavoidable, the invalid combination MUST fail explicitly in the crate, and the CI invocation MUST encode the same constraint.

[R-2.2.6] A full feature powerset MAY be used for small libraries. It SHOULD NOT be imposed on a large workspace when it creates a combinatorial queue with little additional coverage. Supported and unsupported combinations MUST be documented. Any unsupported combination that users or CI can select MUST fail explicitly with a clear diagnostic.

### 2.3 Security, licensing, actions, and audit records

[R-2.3.1] The workspace SHOULD use `cargo-deny` or an equivalent gate for advisories, licenses, banned crates, and source restrictions.

[R-2.3.2] `cargo-vet` MAY be adopted when the organization maintains a human-review process for third-party source audits. Audit records MUST NOT be described as cryptographic proof unless the repository adds and verifies an actual signing mechanism.

[R-2.3.3] `cargo-semver-checks` SHOULD gate published libraries that make compatibility commitments. It is unnecessary for private application crates without a supported public API.

[R-2.3.4] Every third-party GitHub Action and reusable workflow reference MUST be pinned to a full commit SHA. The GitHub repository or organization policy that requires full-SHA pinning SHOULD be enabled for Actions. As validated on 2026-09-03, that policy does not enforce reusable-workflow references, so a workflow linter such as `zizmor`, `pinact`, or an equivalent control MUST cover those references separately. A nearby version comment SHOULD preserve readability, and an update bot SHOULD maintain the pins.

[R-2.3.5] Protected build jobs SHOULD pin container images by digest and verify downloaded tools or binary assets through an authenticated package manager, checksum, signature, or equivalent provenance control.

### 2.4 Build-time code and generated inputs

[R-2.4.1] Build scripts, procedural macros, compiler wrappers, code generators, native build tools, and package-manager hooks execute code during the build. New or materially changed build-time code MUST be reviewed as executable supply-chain code, not as passive metadata.

[R-2.4.2] Protected builds MUST NOT download or execute unpinned tools, scripts, schemas, models, or datasets. Every remote build input MUST be obtained through an approved package or artifact channel and pinned by immutable version, digest, signature, or an equivalent authenticated reference.

[R-2.4.3] Build-time network access SHOULD be disabled by default. Filesystem and environment access SHOULD be restricted to declared inputs. Generated outputs that affect compilation or release artifacts MUST be reproducible from versioned inputs, or checked in and verified against a deterministic regeneration gate.

## 3. Error architecture and observability

### 3.1 Typed domain errors

[R-3.1.1] Libraries, domain modules, and reusable interfaces SHOULD expose typed errors. `thiserror` is appropriate when callers need to match recoverable conditions, preserve structured causes, or map failures into transport-specific responses.

[R-3.1.2] Application entry points, one-off orchestration layers, and command-line commands MAY use `anyhow` to attach operational context. `anyhow::Error` SHOULD NOT appear in a stable public library interface or erase a domain distinction that callers must handle.

[R-3.1.3] Error conversions MUST preserve the original source through `#[source]`, `#[from]`, or an equivalent explicit chain unless the source is deliberately dropped to prevent data leakage. A deliberate drop SHOULD be documented at the conversion site.

[R-3.1.4] Error messages MUST NOT contain credentials, authorization headers, cookies, secret-bearing configuration values, unrestricted request bodies, raw prompts, or personal data that is not explicitly approved for the telemetry class.

[R-3.1.5] Transport layers MUST map internal failures deliberately. Protected telemetry MAY retain a sanitized causal chain. Clients MUST receive stable machine-readable error codes and sanitized messages that do not disclose internal implementation details.

### 3.2 Structured telemetry

[R-3.2.1] Services SHOULD use structured tracing with request, operation, and correlation identifiers. Instrumentation MUST exclude function arguments by default. Safe fields MUST be added explicitly by name.

```rust
use tracing::instrument;

#[instrument(
    name = "process_request",
    skip_all,
    fields(request.id = %request_id)
)]
async fn process_request(
    request_id: &str,
    request: Request,
) -> Result<Response, AppError> {
    // ...
}
```

[R-3.2.2] Secrets, tokens, cookies, authorization headers, raw prompts, unrestricted SQL parameters, and unapproved personal identifiers MUST NOT enter spans, events, metrics labels, panic messages, or error chains.

[R-3.2.3] Metric labels MUST have bounded cardinality. Request IDs, user-provided text, paths, URLs, SQL text, and error messages belong in appropriately protected traces or logs, not in metric labels.

[R-3.2.4] Sensitive telemetry paths MUST have tests or review fixtures that demonstrate redaction. Secret wrapper types SHOULD redact `Debug` and `Display`. Zeroization MAY reduce exposure for selected buffers, but it is defense in depth rather than proof that no copies remain in allocator, runtime, operating-system, crash-dump, or library memory.

### 3.3 Panics and `unsafe`

[R-3.3.1] Crates that do not need `unsafe` SHOULD declare `#![forbid(unsafe_code)]`.

[R-3.3.2] Every `unsafe` block and `unsafe impl` MUST carry a `// SAFETY:` explanation that states the invariant, ownership assumptions, aliasing rules, lifetime requirements, and caller obligations relevant to that operation. The workspace SHOULD enable `clippy::undocumented_unsafe_blocks`. Under edition 2024, `unsafe_op_in_unsafe_fn` is warn-by-default; this behavior was validated against Rust 1.90.0 on 2026-09-03, and the canonical lint gate SHOULD deny it explicitly.

[R-3.3.3] Crates that contain Rust-level `unsafe` SHOULD run supported tests under Miri on a schedule. FFI calls and platform behavior outside Miri's model SHOULD receive separate boundary tests and, where available, sanitizer or native-tool validation.

[R-3.3.4] Request-handling and long-running service code SHOULD NOT panic on external input. `clippy::unwrap_used` and `clippy::expect_used` MAY be denied for those crates while remaining allowed in tests and narrowly scoped tooling. Any allowance that weakens the repository lint policy MUST cite an active exception under R-0.7.

[R-3.3.5] Whether a panic aborts or unwinds is a product decision. A service that unwinds MUST bound the blast radius through task and request isolation and MUST NOT use `catch_unwind` as a substitute for typed error handling.

## 4. Testing strategy

### 4.1 Test layers and runners

[R-4.1.1] A repository SHOULD cover the following layers in proportion to its architecture and threat model.

| Layer | Required behavior | Typical mechanism |
| --- | --- | --- |
| Pure domain logic | Fast, deterministic tests without network or database dependencies | Unit tests |
| HTTP clients and protocol adapters | Controlled responses, timeouts, malformed payloads, and retry behavior | `wiremock` or an equivalent local server |
| Relational database behavior | Isolated schema or database state with migrations applied | `#[sqlx::test]` or an equivalent fixture strategy |
| Parsers and validators | Broad generated input, absence of panic, and domain-specific invariants | `proptest` |
| Trust boundaries | Arbitrary bytes and structured mutation of representative corpora | `cargo-fuzz` |
| Test-suite quality | Detection of assertions that allow incorrect behavior to survive | Targeted `cargo-mutants` jobs |

[R-4.1.2] Default local tests SHOULD remain independent of external infrastructure. Database and integration tests MUST run in a required CI job. Every ignored test MUST use the standard reason-string form, such as `#[ignore = "ISSUE-1234: requires live PostgreSQL"]`, and MUST cite a tracked issue or active exception that records an owner, required job, and removal condition. The test-policy gate MUST reject bare `#[ignore]` attributes.

[R-4.1.3] A runner such as `cargo-nextest` MAY provide process isolation, scheduling controls, timeouts, and machine-readable reports. Where nextest is the canonical runner, the canonical test task MUST also run `cargo test --doc` because nextest does not execute doctests. This behavior was validated against nextest documentation reviewed on 2026-09-03; the repository MUST record the exact nextest pin in Appendix B before treating the claim as closed.

[R-4.1.4] Retries MUST NOT convert an unexplained deterministic failure into a clean protected-branch result. When retries are enabled, the protected nextest profile MUST set `flaky-result = "fail"`, retries MUST be limited to explicitly classified infrastructure-dependent tests, and the initial failure MUST remain visible in retained reporting.

### 4.2 Property testing

[R-4.2.1] Validators SHOULD be tested with arbitrary valid UTF-8 strings. In the proptest behavior reviewed on 2026-09-03, `any::<String>()` uses the `\PC*` strategy and excludes Unicode general category C, including control and format characters such as ZERO WIDTH JOINER, bidirectional controls, and U+FEFF. Tests for grapheme handling, emoji sequences, directionality attacks, or control-character rejection MUST use an explicit strategy such as `prop::collection::vec(any::<char>(), 0..n)` collected into a `String`, together with fixed fixtures for known sequences. Byte-level parsers MUST also receive arbitrary byte sequences because `&str` cannot represent invalid UTF-8.

[R-4.2.2] Scoring and ranking engines MUST test determinism under a documented configuration and seed, numeric boundedness, and absence of invalid numeric states. Monotonicity MUST be tested when the scoring contract states that strengthening a signal can never lower the result. Property tests MUST encode the actual domain contract rather than assume every scoring model is monotonic.

[R-4.2.3] Canonicalization functions SHOULD test idempotence, equivalence of intended canonical forms, preservation of required information, and absence of panic. Truncation functions SHOULD test prefix preservation, boundary safety, and behavior at zero, exact, and oversized limits. A test that merely proves a returned `String` is valid UTF-8 adds little because Rust already guarantees that invariant.

### 4.3 Fuzzing and mutation testing

[R-4.3.1] Fuzz targets SHOULD cover network payload parsing, webhook ingestion, address and identifier parsers, deserialization, archive handling, and any boundary that converts untrusted bytes into trusted structures.

[R-4.3.2] CI MAY run short regression fuzz jobs against a fixed corpus. Longer campaigns SHOULD run on a scheduled or dedicated worker with crash artifacts retained, minimized, and screened for sensitive data before publication.

[R-4.3.3] Mutation testing SHOULD target high-value domain code and run on a schedule or before major releases. It SHOULD NOT become a blanket per-pull-request gate when runtime cost overwhelms its signal.

## 5. Operational security and filesystem safety

### 5.1 Secret injection

[R-5.1.1] Secrets MUST NOT be passed in command-line arguments. Approved delivery mechanisms SHOULD be selected according to the runtime threat model and orchestrator capabilities.

[R-5.1.2] Read-only ephemeral mounts, secret-manager file descriptors, and stdin SHOULD be preferred because they avoid process-list exposure and can support narrow lifetimes. Environment variables MAY be used only when the platform and threat model accept their visibility characteristics, child-process inheritance is controlled, and operator tooling, crash handling, and diagnostics have been validated not to expose them.

[R-5.1.3] Persistent plaintext secret files MUST NOT be used. When a platform temporarily materializes a secret, permissions, ownership, lifetime, cleanup, backup behavior, and crash behavior MUST be defined. Removing a pathname does not prove that every filesystem or memory copy has disappeared.

[R-5.1.4] Build-time secrets MUST use mechanisms that prevent them from being copied into image layers, build logs, provenance records, or cached build contexts.

### 5.2 Handle-relative filesystem access

[R-5.2.1] Code that operates inside attacker-influenced, shared, upload, temporary, credential, or operator-controlled directories MUST use handle-relative traversal anchored at a trusted directory descriptor or an equivalent capability object.

[R-5.2.2] The implementation SHOULD combine platform-appropriate protections such as `openat`, `openat2`, `NOFOLLOW`, `RESOLVE_BENEATH`, `CLOEXEC`, exclusive creation, and post-open metadata validation. Metadata validation MUST inspect the opened handle, such as with `fstat`, rather than rechecking the path.

[R-5.2.3] `cap-std` SHOULD be evaluated as the default abstraction for capability-relative filesystem access. Its use does not remove the need to test required symlink, hard-link, replacement, and platform semantics. Linux-specific controls such as `openat2` and `RESOLVE_BENEATH` MUST have explicitly documented behavior on other supported targets. `NOFOLLOW` on the final component alone does not validate every intermediate path component.

[R-5.2.4] Path canonicalization performed before opening a file MUST NOT be treated as sufficient protection against time-of-check to time-of-use races.

### 5.3 Downloads and archive extraction

[R-5.3.1] Untrusted downloads MUST enforce streaming limits, deadlines, redirect policy, and cancellation. `Content-Length` is advisory and MUST NOT be the only size control.

[R-5.3.2] When an HTTP client transparently decompresses `Content-Encoding`, the application-level limit MUST apply to the decoded stream it reads. When the threat model also requires an independent wire-byte quota, automatic decompression SHOULD be disabled and the transport and decoder SHOULD be bounded separately.

[R-5.3.3] Archive extraction MUST bound compressed input, decompressed output, entry count, individual entry size, total expansion, and execution time. A nesting-depth limit MUST be enforced when recursive archive processing is supported. Limits that apply only before decompression do not stop a compression bomb.

[R-5.3.4] Extraction MUST reject absolute paths, parent traversal, unsupported special files, unsafe hard links, and symlinks unless a reviewed format-specific policy proves them safe. The extractor MUST track destinations created earlier in the same extraction and reject duplicate or overwriting entries. If symbolic links are permitted, regular files MUST be written before links, and no subsequent entry may resolve through an extracted link.

[R-5.3.5] The extractor MUST reject logical destination collisions under every supported deployment filesystem's case and Unicode-normalization behavior. Tests MUST cover case-only and NFC-versus-NFD collisions on targets where those names alias. When Windows is supported, drive-letter paths, UNC prefixes, and reserved device names MUST be rejected or handled by an explicit safe policy.

[R-5.3.6] Extraction SHOULD occur in a fresh staging directory and be promoted only after validation succeeds. Expected artifacts SHOULD be verified by digest or signature when a trusted manifest or expected value exists.

[R-5.3.7] Secret scanning and static analysis SHOULD run in CI. Sanitized documentation, fixtures, corpora, and crash artifacts MUST contain no real credentials or unapproved personal data.

## 6. Async runtimes and embedded analytical engines

[R-6.1] Blocking database calls, filesystem operations, compression, CPU-heavy transforms, and FFI work MUST NOT execute directly on asynchronous runtime worker threads when they can block for a material duration.

[R-6.2] Finite blocking work SHOULD run in `spawn_blocking` or a dedicated bounded executor. Long-lived or persistent blocking work SHOULD use dedicated threads or a purpose-built pool rather than occupying the runtime's general blocking pool indefinitely.

[R-6.3] Admission control MUST be acquired before blocking work is dispatched so the executor queue cannot grow without bound. Capacity planning MUST include the admission limit multiplied by each separate engine instance's native thread cap and memory cap, plus measured process headroom. Connections that share one engine instance MUST be modeled according to the engine's actual shared-pool semantics rather than multiplied mechanically.

[R-6.4] A timeout around `spawn_blocking` can stop awaiting the result without stopping work that has already started. Any operation that can materially outlive its request deadline MUST therefore have an engine-level deadline or interruption mechanism, or run behind a process boundary that can be terminated. Memory, CPU, spill, and result caps MUST complement cancellation; they do not turn an asynchronous timeout into cancellation.

[R-6.5] Service shutdown MUST stop new admission, invoke the engine's interruption mechanism for every tracked in-flight blocking operation, and bound the remaining wait with a shutdown timeout. Interrupt or cancellation handles MUST be registered before execution and removed after completion. Under the Tokio behavior reviewed on 2026-09-03, dropping a runtime can wait indefinitely for started blocking closures, while `shutdown_timeout` bounds waiting without cancelling those closures; the exact Tokio pin MUST be recorded in Appendix B.

[R-6.6] A multi-threaded service MUST NOT adopt `spawn_local`, `LocalSet`, or a current-thread runtime solely to bypass a `Send` constraint on an engine handle. An intentionally single-threaded architecture MAY use those mechanisms only when ownership, latency, throughput, and shutdown behavior are documented and tested.

[R-6.7] The repository SHOULD benchmark a shared engine or database instance with per-request connections or handles against reopening the database on every request. The selected topology MUST be justified under representative concurrency because safety, cancellation, isolation, memory reuse, and throughput depend on the engine and workload.

[R-6.8] Embedded analytical connections SHOULD default to read-only database access. Resource controls for memory, native worker threads, temporary storage, spill quota, result size, and statement duration MUST be set at the narrowest scope the engine supports. Temporary storage MUST use a controlled, quota-constrained, and monitored location.

[R-6.9] Engine hardening settings that are documented as startup-only MUST be supplied during instance creation. Settings that enforce resource or capability boundaries MUST be locked against runtime modification where the engine supports a configuration lock. A required integration test MUST prove that an untrusted statement cannot relax a locked limit or re-enable a disabled capability.

[R-6.10] Read-only database mode is not a complete sandbox. An embedded engine can still expose filesystem, network, extension, attachment, copy, export, or external-reader capabilities. Services MUST NOT execute untrusted SQL without a separate allowlisted query interface or an operating-system or container sandbox. Engine settings for external access and extension installation or loading MUST be restricted according to the threat model.

[R-6.11] Dynamic SQL MUST use bound parameters for values where supported. Identifiers, sort expressions, operators, and statement classes that cannot be bound MUST come from explicit allowlists or a typed query builder. Query text, returned rows, returned bytes, and exported file size MUST be bounded at the service boundary.

## 7. Unicode and string safety

[R-7.1] Human-readable names, addresses, and other domain text SHOULD use a documented canonicalization policy before equality, deduplication, or hashing. The policy SHOULD record normalization form, case handling, whitespace handling, locale assumptions, and the Unicode data version where reproducibility matters. NFC is a reasonable default when the domain treats canonically equivalent Unicode sequences as the same text.

[R-7.2] NFC does not provide case-insensitive comparison, locale-aware collation, accent folding, or search normalization. Those transformations require a separate domain policy and MUST NOT be inferred from normalization alone.

[R-7.3] Normalization MUST NOT be applied blindly to passwords, tokens, signatures, opaque identifiers, protocol fields, filesystem paths, or cryptographic inputs. Those values follow their defining specification, which may require byte-for-byte preservation. A change to canonicalization that affects persisted keys, hashes, uniqueness, joins, or equality MUST be handled as a versioned data migration with collision analysis, compatibility rules, and a reindex or backfill plan.

[R-7.4] Offsets computed before case conversion, normalization, replacement, or any other transformation MUST NOT be reused to slice the transformed string. Unicode transformations are not guaranteed to preserve byte length, scalar-value count, or grapheme count.

[R-7.5] Every numeric text-length limit exposed by code or configuration MUST encode its unit in the identifier, using a suffix such as `_bytes`, `_scalars`, or `_graphemes`. Generic limit names such as `max_name_len` or `title_limit` MUST be rejected by the repository's domain-lint or policy check. Byte limits SHOULD be used for protocols and storage; grapheme limits SHOULD be used for visible truncation.

```rust
use unicode_segmentation::UnicodeSegmentation;

pub fn truncate_graphemes(text: &str, limit_graphemes: usize) -> &str {
    match text.grapheme_indices(true).nth(limit_graphemes) {
        Some((byte_index, _)) => &text[..byte_index],
        None => text,
    }
}
```

[R-7.6] Property tests SHOULD verify canonicalization idempotence and intended equivalence. Truncation tests SHOULD verify prefix preservation and safe boundaries across combining marks, emoji sequences, directionality controls, and case mappings that expand or contract.

## 8. Agentic tools and model transparency

### 8.0 Definitions

An **evidence class** is a policy-defined trust level assigned to evidence according to its producer, isolation, reproducibility, and review status.

An **evidence receipt** is a machine-readable record that binds a control result to the exact input commit, configuration, tool versions, and artifact digest that produced it.

**Receipt promotion** is the act of accepting evidence from one class as satisfying a higher-trust gate. Promotion is an authorization decision, not a property inferred from the receipt itself.

A **deployment tripwire** is a deterministic startup or release condition that blocks exposure when a deployment assumption changes, such as moving from one trusted operator to multiple remote principals.

### 8.1 Trusted identity and tenant isolation

[R-8.1.1] A model-provided tenant, workspace, user, resource, or authorization identifier is untrusted input. Authentication middleware MUST derive the principal and tenant context from a trusted credential or session and pass that context separately to the tool dispatcher.

[R-8.1.2] Every resource lookup and write MUST authorize the resource against the trusted principal and tenant context. Integer IDs, UUIDs, filenames, URLs, and model-generated selectors MUST receive the same cross-check. Authorization MUST fail closed.

[R-8.1.3] Single-principal tooling MUST declare that deployment mode explicitly. Enabling remote transport or multiple principals without authentication, authorization, and tenant-isolation tests MUST fail a deployment gate or startup check.

### 8.2 Capability boundaries

[R-8.2.1] Agent tools MUST be allowlisted by capability. Credential reads, production writes, policy changes, release promotion, destructive filesystem actions, and cross-tenant access MUST be denied unless an explicit policy grants them.

[R-8.2.2] Tool schemas MUST validate types, ranges, sizes, enum values, and resource selectors before execution. Model text, retrieved content, tool output, or prompt instructions MUST NOT be treated as granting a capability that the trusted policy did not already authorize.

[R-8.2.3] Write operations SHOULD support idempotency keys, dry-run output where practical, bounded retries, and durable audit records. High-impact or irreversible operations MUST require an authorization decision independent of the model-generated request. Model output MUST be treated as untrusted until deterministic validation succeeds.

### 8.3 Model resolution and fallback

[R-8.3.1] Model fallback MUST NOT be silent. Auditable execution metadata MUST record the requested model or capability, resolved provider and model, whether fallback occurred, and the reason. Client-visible disclosure SHOULD follow the product contract and security policy rather than exposing internal routing details indiscriminately.

[R-8.3.2] Missing or invalid credentials MUST produce a typed configuration error. They MUST NOT trigger an unauthenticated mode, a public model, or an unapproved provider.

[R-8.3.3] The repository MUST maintain a checked-in model registry, such as `governance/models.toml`, that records each approved provider, model identifier, capability class, allowed roles, and fallback eligibility. The resolver MUST reject an unlisted model, and a policy test MUST prove that every configured fallback chain is a subset of the registry.

[R-8.3.4] Rate-limit handling SHOULD use bounded retries, jittered backoff, an overall deadline, and a circuit breaker where repeated calls would amplify an outage. Fallback execution MUST remain within the authorization and capability class recorded under R-8.3.3.

### 8.4 Independent approval and evidence

[R-8.4.1] An agent MUST NOT serve as the sole approver or authorization source for changes to its own code, harness, policy, evaluation, security controls, or release permissions.

[R-8.4.2] An agent MAY execute a merge or promotion only after a human reviewer with the relevant authority or an independent deterministic gate has supplied the authorization. The gate MUST load its controlling policy from a protected base revision or trusted control plane, and the candidate change MUST NOT be able to weaken that policy before authorization.

[R-8.4.3] Evidence receipts SHOULD be deterministic, content-addressed, and bound to the reviewed commit and produced artifact. Receipt promotion MUST require an explicit policy decision by an authorized human or independent gate. Cryptographic signatures MUST be added when a verifier, consumer, or governance policy validates them. Without such a verifier, signatures MAY be omitted and MUST NOT be described as establishing a trust decision by themselves.

## 9. Git, CI, release, and provenance

[R-9.1] Local and CI validation MUST execute the same canonical task-runner commands with the same package exclusions, feature flags, targets, profiles, lockfile policy, and warning policy. The canonical Clippy task MUST invoke the selected workspace or package set with `-- -D warnings`. `RUSTFLAGS` MUST NOT carry lint policy because it changes compilation fingerprints and creates local-versus-CI drift.

[R-9.2] Repositories with a shared primary branch MUST protect it against unreviewed direct pushes. Pull requests SHOULD be focused and reviewable. A one-commit policy is optional; squash merging can preserve a clean primary branch without forcing authors to erase useful review history.

[R-9.3] Formatting, Clippy with warnings denied, tests, dependency policy, migration checks, documentation-reference checks, and generated-artifact checks SHOULD be deterministic merge gates when applicable.

[R-9.4] Lint levels SHOULD live in `[workspace.lints]`. Every member intended to inherit the policy MUST declare `[lints] workspace = true`. Application workspaces MUST pin an exact Rust toolchain release in `rust-toolchain.toml`, including required components and targets. Library workspaces MUST pin the primary CI toolchain, SHOULD declare their minimum supported Rust version through `rust-version`, and SHOULD test both that minimum and the current supported stable toolchain.

[R-9.5] Workflow tokens MUST use least privilege. Untrusted pull-request code MUST NOT receive secrets or write-capable repository, package, cache, cloud, or deployment credentials. Self-hosted runners used for untrusted code MUST be ephemeral or strongly isolated from persistent credentials and other workloads.

[R-9.6] Sensitive changes SHOULD be reviewed from Git objects: parent commit, exact file list, complete diff, generated artifacts, and an offline or isolated test at the reviewed commit. Evidence receipts used for approval, merge, or promotion MUST identify the exact reviewed commit SHA and artifact digest. The gate MUST reject a candidate whose commit or digest differs from the authorized receipt.

[R-9.7] The production artifact MUST be built through one approved path. The pipeline SHOULD build once, validate that artifact, and promote the same digest. A faster non-LTO artifact MAY support pre-merge testing, but it cannot replace validation of the deployable artifact.

[R-9.8] Release automation, SBOM generation, DSSE envelopes, Sigstore, or Ed25519 signatures SHOULD be introduced when the repository has versioned releases, external consumers, compliance requirements, or a verifier that consumes the evidence. Provenance machinery without a consumer adds maintenance cost without establishing a trust decision.

[R-9.9] Internal documentation links and repository paths MUST resolve. API documentation SHOULD deny `rustdoc::broken_intra_doc_links`. Markdown link checking SHOULD distinguish internal references from external URLs so a transient third-party outage does not become indistinguishable from repository drift. Permanent or repeated external failures MUST receive an owner, replacement, or documented exception.

## 10. Measurement discipline

[R-10.1] Performance work MUST begin with a reproducible baseline. The record SHOULD include hardware, operating system, Rust toolchain, target triple, linker, profile, feature set, cache state, background load, CPU power mode, thermal conditions, dataset, and command.

[R-10.2] `cargo build --timings` SHOULD identify critical paths and serial tails. `cargo llvm-lines` or equivalent analysis MAY be used when evidence points to excessive monomorphization. Neither tool substitutes for end-to-end elapsed-time and workload measurements.

[R-10.3] Cold-cache and warm-cache results MUST be labeled separately. Comparative measurements SHOULD use repeated runs and report an appropriate summary such as median plus range or percentile. CPU throttling, power-saving governors, container limits, and shared-runner contention are confounders, not performance conclusions.

[R-10.4] Destructive measurements such as `cargo clean`, cache deletion, database reset, or fixture removal SHOULD be announced before execution and SHOULD run only in a disposable or explicitly isolated working environment with a restoration path.

[R-10.5] Optimization changes SHOULD define an acceptance threshold before measurement. A change that adds complexity without crossing that threshold SHOULD be rejected or recorded as experimental.

## 11. Enforcement and evidence controls

[R-11.1] The following control map is normative. Each control MUST run on the protected branch or release path stated in its gate, and its evidence MUST identify the commit, tool versions, configuration, and result. A repository MAY split one control into multiple jobs, but the mapped rule coverage MUST remain machine-readable.

| Control | Mapped MUST rules | Minimum gate | Required evidence | Default owner |
| --- | --- | --- | --- | --- |
| C-GOV-01 | R-0.2; R-0.3; R-0.4; R-0.5; R-0.9; R-11.1; P-A.8.1; P-B.1 | Document-conformance job validates rule IDs, normative-keyword usage, control coverage, version tags, and appendix status | Conformance report containing every discovered rule and mapping | Standard owner |
| C-GOV-02 | R-0.6; R-0.7; R-0.8; R-2.1.3; P-A.2.1 | Exception-manifest parser plus waiver-reference scan; security or legal review for non-waivable obligations | Parsed manifest, expiry result, waiver inventory, approval record | Standard owner plus security owner |
| C-BLD-01 | R-1.1.1; R-1.1.2; R-1.1.4; R-1.3.1; R-1.3.2; R-1.3.4; R-1.3.5; R-1.4.1; R-1.4.3; R-1.4.4; P-A.1.1 | Canonical build-profile and linker jobs for every supported target, with feature inspection and cache telemetry | Commands, `cargo tree -e features`, timing report, linker evidence, cache statistics | Build owner |
| C-DEP-01 | R-2.1.1; R-2.1.2; R-2.1.5; R-2.1.6; R-2.2.1; R-2.2.2; R-2.2.3; R-2.2.5; R-2.2.6; P-A.2.2; P-A.2.3; P-A.5.2 | Locked dependency-resolution and feature-matrix jobs for exact packages and targets, including prohibited-feature rejection | Lockfile diff, resolver value, duplicate report, feature matrix, release feature graph | Dependency owner |
| C-SUP-01 | R-2.3.2; R-2.3.4; R-2.4.1; R-2.4.2; R-2.4.3; R-5.1.4 | Supply-chain policy checks, workflow-reference linter, build-time executable review, remote-input pin verification, build-secret test | Policy reports, workflow lint, input inventory, reproduction or regeneration evidence | Supply-chain owner |
| C-ERR-01 | R-3.1.3; R-3.1.4; R-3.1.5; R-3.2.1; R-3.2.2; R-3.2.3; R-3.2.4 | Error-mapping, telemetry-redaction, and metric-cardinality tests plus protected-path review | Test results and sanitized trace fixtures | Service owner |
| C-UNS-01 | R-3.3.2; R-3.3.4; R-3.3.5 | Clippy policy, unsafe-comment scan, panic-path tests, and scheduled Miri or native boundary validation | Lint output, waiver references, Miri or sanitizer logs, isolation tests | Crate owner |
| C-TST-01 | R-4.1.2; R-4.1.3; R-4.1.4; R-4.2.1; R-4.2.2; P-A.3.1; P-A.3.2 | Canonical unit, integration, doctest, property, fuzz-regression, and flaky-policy jobs | Test reports, ignored-test inventory, nextest config, retained and sanitized crash artifacts | Test owner |
| C-SEC-01 | R-5.1.1; R-5.1.3; R-5.2.1; R-5.2.2; R-5.2.3; R-5.2.4 | Secret-delivery review and handle-relative filesystem tests on supported targets | Threat-model record, process inspection, permissions evidence, race and symlink tests | Security owner |
| C-ARC-01 | R-5.3.1; R-5.3.2; R-5.3.3; R-5.3.4; R-5.3.5; R-5.3.7 | Download and archive adversarial test corpus covering expansion, duplicates, links, collisions, platform prefixes, and sanitation | Test report, corpus manifest, scanner output | Security owner |
| C-ANL-01 | R-6.1; R-6.3; R-6.4; R-6.5; R-6.6; R-6.7; R-6.8; R-6.9; R-6.10; R-6.11; P-A.4.1; P-A.4.3; P-A.4.4; P-A.4.5; P-A.4.6 | Representative load, cancellation, shutdown, memory, spill, lock, sandbox, and query-bound tests | Capacity model, engine configuration, interruption trace, lock-failure test, load report | Data-service owner |
| C-UNI-01 | R-7.2; R-7.3; R-7.4; R-7.5 | Domain string-policy lint, migration review, canonicalization properties, and unit-suffix check | Domain policy, lint output, property-test results, migration record | Domain owner |
| C-AGT-01 | R-8.1.1; R-8.1.2; R-8.1.3; R-8.2.1; R-8.2.2; R-8.2.3; R-8.3.1; R-8.3.2; R-8.3.3; R-8.3.4; R-8.4.1; R-8.4.2; R-8.4.3; P-A.6.1; P-A.6.2; P-A.6.3 | Authentication, tenant-isolation, capability, schema, model-registry, fallback, high-impact write, and independent-approval tests | Policy tests, model registry, audit events, negative tests, approval receipt | Tool owner plus independent reviewer |
| C-CI-01 | R-9.1; R-9.2; R-9.4; R-9.5; R-9.6; R-9.9; P-A.7.1 | Canonical task comparison, branch and workflow policy checks, exact-toolchain validation, SHA-bound evidence check, link checker | Workflow policy report, task manifest, toolchain file, receipt validation, link report | CI owner |
| C-REL-01 | R-1.3.7; R-1.3.8; R-9.7; P-A.5.1 | Build through the approved production path, test the resulting artifact, and promote the identical digest | Build manifest, artifact digest, artifact-level test report, promotion record | Release owner |
| C-MET-01 | R-10.1; R-10.3 | Benchmark record schema and comparison check in performance-affecting changes | Raw measurements, environment record, labeled cache state, comparison summary | Change owner |


## Appendix A. Repository-specific profile

This appendix preserves local facts and controls that should not be presented as universal Rust guidance. It is not yet a complete conformance declaration; unresolved implementation work and unassigned owners remain in Appendix B.

### A.1 Current build baseline

The current `api-server` serial tail is approximately 39 seconds under the governor-disabled measurement described in the measurement notes. The location of those notes remains unresolved in Appendix B. A prior 56-second result was affected by throttling.

[P-A.1.1] Before either value becomes a regression gate, the repository MUST record hardware, operating system, toolchain, target, linker, profile, feature set, cache state, command, measurement date, and repeated-run summary.

[P-A.1.2] The repository's configuration history contains both `opt-level = 3` and `opt-level = 2` for third-party development dependencies. The repository SHOULD benchmark both under cold-cache compilation and representative local tests, then codify one value. Until that comparison exists, the portable baseline uses level 2.

### A.2 Dependency exceptions and feature coverage

[P-A.2.1] The dual `reqwest` 0.12 and 0.13 lines are a proposed temporary exception when they isolate a legacy-TLS host from the rustls-based fleet. The exception MUST NOT be treated as active until `governance/exceptions.toml` contains an unexpired entry that references R-2.1.3 and names the owner, rationale, scope, compensating control, decision record, and expiry. The responsible `Cargo.toml` and any `cargo-deny` skip MUST cite the same `EXC-` identifier.

[P-A.2.2] The workspace root MUST declare resolver 2 or resolver 3 explicitly. The repository has not yet recorded which value is committed; Appendix B tracks that verification.

[P-A.2.3] The supported feature gate currently includes the default feature set, no default features, `tools`, `otlp`, `test-helpers`, `test-insecure-issuers`, all features when valid, and approved deployment combinations. CI MUST test that matrix. The deployable `api-server` artifact MUST resolve without `test-helpers` and `test-insecure-issuers`, and the release feature-graph check MUST fail on either name.

### A.3 Test topology

[P-A.3.1] Default `cargo test` remains database-free. PostgreSQL behavior MUST run in a required isolated job. HTTP clients use controlled local servers. Any database-gated ignored test MUST carry an `ISSUE-` or active `EXC-` reason under R-4.1.2.

[P-A.3.2] The main fuzz workspace covers the external CRM-to-Brain HTTP boundary, address parsing, Brazilian identifier validators, webhook payload parsing, and search-response deserialization. Fuzz corpora and retained crashes MUST be screened so customer, operator, credential, or other restricted data cannot enter repository artifacts.

The repository has not yet recorded whether nextest is the canonical runner, its exact pin, the doctest command, or the protected flaky-result policy. Appendix B tracks that decision.

### A.4 DuckDB and analytical-service limits

DuckDB access currently calls per-request `Connection::open` inside blocking work. The duckdb-rs source behavior reviewed on 2026-09-03 confirms that this path constructs a separate DuckDB instance rather than consulting the internal instance cache. Each request therefore receives a separate buffer manager, catalog load, and native worker pool.

[P-A.4.1] The service MUST retain read-only database mode and bounded admission control. Under the current defaults, a concurrency limit of `N`, `threads = 4`, and `memory_limit = 2 GB` creates planning bounds of approximately `4N` engine threads and `2N GB` of engine-managed memory before process headroom and allocations outside the buffer manager. Container memory and CPU limits MUST be sized against those multiplied bounds or the topology MUST change.

[P-A.4.2] The repository SHOULD benchmark the current separate-instance design against one shared database instance with per-request connections created through `Connection::try_clone()`. The API and shared-instance semantics were verified in duckdb-rs documentation reviewed on 2026-09-03; the exact crate pin remains open in Appendix B.

[P-A.4.3] Temporary spill MUST use a service-owned directory with capacity monitoring and a configured `max_temp_directory_size`. That setting exists in DuckDB 0.10.3 and later and has a documented default of 90 percent of available disk, which is not a service-specific boundary. The exact pinned DuckDB version and binding support MUST be recorded before the control is treated as closed.

[P-A.4.4] Request timeout and service shutdown MUST invoke `Connection::interrupt_handle()` for every tracked in-flight query, stop new admission, and bound the remaining blocking-pool wait. Tests MUST prove that interruption stops the query, maps the error correctly, cleans temporary state, and either leaves the connection usable or discards it safely.

[P-A.4.5] DuckDB's `memory_limit` applies to the buffer manager rather than every allocation in the process. The service MUST retain an operating-system or container memory limit and measured headroom in addition to engine settings. This scope was verified in DuckDB documentation reviewed on 2026-09-03; the exact engine pin remains open in Appendix B.

[P-A.4.6] The DuckDB hardening profile MUST be applied through the configuration supplied at instance creation. It MUST set external-access, extension-installation, extension-loading, community-extension, unsigned-extension, filesystem, temporary-storage, memory, and thread controls before opening the instance, then set `lock_configuration = true` last. `allowed_configs` MAY retain only settings that must remain mutable. The integration suite MUST prove that a later `SET memory_limit` or equivalent relaxation fails. `allowed_directories`, `allowed_paths`, and `disabled_filesystems` MUST be evaluated against the pinned engine and the service's file-access requirements. Startup-only and one-directional settings, including the reviewed behavior of `enable_external_access` and `allow_unsigned_extensions`, MUST NOT be deferred to post-open SQL.

### A.5 Release path and CI parity

[P-A.5.1] The Dockerfile remains the sole approved production-binary path. `ci-release` MAY skip LTO for fast validation, but the deployment gate MUST build the actual thin-LTO image, execute artifact-level tests against that image, and promote the exact tested digest.

[P-A.5.2] The release gate MUST run the feature-resolution control in R-2.2.3 for the deployable `api-server` package and target and MUST reject `test-helpers` or `test-insecure-issuers`. The binary SHOULD also refuse to start when compiled with an insecure-issuer feature unless an explicit non-production marker is present; a compile-time error under a production configuration is an acceptable alternative.

The workspace remains at `0.1.0` and does not require versioned-release machinery until the product adopts versioned releases or external consumers need distributable artifacts.

### A.6 Agentic governance

[P-A.6.1] The current `mcp-server` is single-operator over stdio. The deployment descriptor MUST declare that assumption. Any move to remote transport or multiple principals MUST activate the authentication, dispatcher-level principal-to-workspace authorization, and tenant-isolation tripwire before exposure.

The `api-server` authorization scope remains an open audit item.

[P-A.6.2] The standing negative-capability policy MUST prohibit credential reads, unauthorized production writes, policy weakening, receipt promotion, and inference of higher-trust evidence from lower-trust evidence. Agents MUST NOT authorize changes to their own harness or policy. Independent review evidence MUST be retained and bound to the reviewed commit under R-9.6.

[P-A.6.3] Approved primary and fallback models MUST be recorded in `governance/models.toml` with provider, model identifier, capability class, permitted roles, and fallback eligibility. The current registry contents and owner remain open in Appendix B.

### A.7 Local process rules

[P-A.7.1] The pre-push hook, `make check`, `make test`, and CI MUST agree on exclusions, features, targets, profiles, lockfile behavior, and warning policy. The canonical Clippy command MUST carry `-- -D warnings`; `RUSTFLAGS` MUST NOT carry lint policy. The previously observed `duckdb-analytics` Clippy drift is the regression example for this gate.

[P-A.7.2] Governance documents that are intentionally untracked SHOULD remain untouched during implementation work. Plans SHOULD receive separate changes with fresh baselines. Unrelated changes SHOULD be withheld from a human-gated candidate because any commit change invalidates the existing evidence receipt under R-9.6.

### A.8 Version-sensitive claim register

[P-A.8.1] Every row with a `TBD` repository pin MUST be resolved before this document advances beyond Draft. The document-conformance control MUST fail a non-Draft status while any required pin remains `TBD`.

| Claim | Validated baseline | Repository pin | Revalidation trigger |
| --- | --- | --- | --- |
| Bundled LLD default on `x86_64-unknown-linux-gnu` and `-C linker-features=-lld` opt-out | Rust 1.90.0, reviewed 2026-09-03 | TBD | Rust toolchain or target change |
| `unsafe_op_in_unsafe_fn` warn-by-default under edition 2024 | Rust 1.90.0, reviewed 2026-09-03 | TBD | Edition or Rust toolchain change |
| `cargo-hack` feature flags used by R-2.2.4 | cargo-hack 0.6.17 or later, reviewed 2026-09-03 | TBD | cargo-hack pin change |
| Default proptest `String` strategy uses `\PC*` and excludes category C | proptest documentation reviewed 2026-09-03 | TBD | proptest pin change |
| nextest omits doctests and supports protected flaky-result failure | nextest documentation reviewed 2026-09-03 | TBD | nextest pin change |
| Started `spawn_blocking` work is not cancelled by dropping the waiter or by shutdown timeout | Tokio documentation reviewed 2026-09-03 | TBD | Tokio pin or runtime architecture change |
| `Connection::open`, `interrupt_handle()`, and `try_clone()` semantics | duckdb-rs documentation and source reviewed 2026-09-03 | TBD | duckdb-rs pin change |
| `max_temp_directory_size`, `memory_limit`, startup settings, and `lock_configuration` behavior | DuckDB 0.10.3 or later where stated; current guidance and source reviewed 2026-09-03 | TBD | DuckDB engine pin change |
| GitHub full-SHA policy covers Actions but not reusable-workflow references | GitHub behavior reviewed 2026-09-03 | Hosted service behavior | GitHub policy or platform change |

## Appendix B. Open decisions and implementation blockers

[P-B.1] This document MUST remain Draft until every row has an accountable owner and closure evidence. A decision may close through a benchmark, pull request, decision record, committed configuration, or test report that satisfies the mapped control.

| Decision | Current state | Required resolution | Owner |
| --- | --- | --- | --- |
| Document ownership and review cadence | The standard has no assigned owner or review interval | Name the accountable owner and define periodic and trigger-based review | TBD |
| Rule and control conformance job | The v5 document defines identifiers and mappings but the repository check is not recorded | Implement the parser that validates IDs, mappings, version tags, and Draft blockers | TBD |
| Exception manifest and dual-reqwest record | Schema is defined; repository file and concrete exception fields are unverified | Add `governance/exceptions.toml`, create or reject the dual-reqwest exception, and wire expiry plus waiver scans | TBD |
| Resolver declaration | The standard requires resolver 2 or 3; committed value is not recorded | Confirm the workspace setting and include it in feature evidence | TBD |
| Release feature exclusion | Test-only features are known; artifact gate is not recorded | Add exact-package feature-graph rejection and an optional compile-time or startup tripwire | TBD |
| Third-party development optimization | Source alternates between levels 2 and 3 | Benchmark cold compilation and representative test runtime, then choose one policy | TBD |
| macOS linker policy | Source mentions alternative and native tooling | Select a supported default per architecture and document fallback behavior | TBD |
| DuckDB and duckdb-rs pins | APIs and behavior were reviewed, but exact repository versions are absent | Record the engine and crate pins and rerun the version-sensitive checks | TBD |
| DuckDB instance topology | Per-request instances are confirmed; shared instance is unmeasured | Benchmark separate instances against shared-instance cloned connections under representative concurrency | TBD |
| DuckDB temporary storage | System temporary storage does not provide a service-specific quota | Configure a service-owned directory and `max_temp_directory_size`, then test exhaustion behavior | TBD |
| DuckDB configuration lock | Required startup profile and lock are specified but not recorded in implementation | Apply settings through open-time config, lock them, and prove runtime relaxation fails | TBD |
| DuckDB cancellation and shutdown | Request interruption and shutdown registry are not recorded | Track interrupt handles, drain admission, interrupt in-flight queries, and bound shutdown wait | TBD |
| nextest, doctests, and flaky policy | Canonical runner and exact pin are not recorded | Decide runner, add `cargo test --doc`, and configure protected retry behavior | TBD |
| Measurement-note provenance | The 39-second and 56-second measurements lack a linked location | Link the notes or commit and record the complete environment and command | TBD |
| Toolchain pin and linker evidence | Exact `rust-toolchain.toml` value and produced linker are absent here | Record channel, components, targets, actual linker, and the evidence command | TBD |
| Production parity gate | Fast CI artifact omits LTO | Build, test, and promote the exact thin-LTO image digest | TBD |
| API authorization scope | Audit remains open | Complete principal, tenant, resource, and capability-level authorization review | TBD |
| Build-time input policy | Network, tool, generator, and generated-input controls are not recorded | Inventory build-time executables and remote inputs, then pin and gate them under Section 2.4 | TBD |
| Model registry | Required schema is defined; approved contents and owner are absent | Add `governance/models.toml` and prove every fallback chain is a subset | TBD |
| Version-sensitive claim pass | Several exact crate and engine pins remain `TBD` in A.8 | Resolve every pin and automate the revalidation trigger | TBD |
| Provenance signing | Deterministic receipts exist; an external verifier is absent | Add signatures only when a consumer or policy requires verification | TBD |

## Appendix C. Resolution of the Draft v4 review

| Finding | Resolution in v5 |
| --- | --- |
| F1 | Added stable `R-` and `P-` identifiers and a no-renumbering rule |
| F2 | Defined `governance/exceptions.toml`, mandatory fields, expiry failure, and waiver references |
| F3 | Added explicit controls, made evidence commit-bound, enforced unit suffixes, defined the model registry, and downgraded destructive-work announcements to SHOULD |
| F4 | Converted requirement-like indicative sentences to explicit normative keywords |
| F5 | Required an explicit Cargo resolver and an exact release feature-graph exclusion gate |
| F6 | Added open-time DuckDB hardening, startup-setting treatment, `lock_configuration`, and a mutation-failure test |
| F7 | Recorded that per-request `Connection::open` creates separate instances and multiplied both thread and memory planning bounds |
| F8 | Added tracked interruption during service shutdown and a bounded shutdown wait |
| F9 | Added intra-archive duplicate, link ordering, case, Unicode-normalization, Windows-prefix, and device-name controls |
| F10 | Corrected the proptest default from a control-only exclusion to all Unicode category C and required explicit strategies plus fixtures |
| F11 | Added a separate doctest command and protected nextest flaky-result failure |
| F12 | Recorded custom-profile output under `target/<profile>/` and its cache and script consequences |
| F13 | Added benchmark-gated `build-override` guidance for build scripts and procedural macros |
| F14 | Added the plain default feature set to the minimum matrix |
| F15 | Added the LLD opt-out and a concrete ELF linker-evidence method |
| F16 | Distinguished GitHub's Action pinning policy from reusable-workflow enforcement |
| F17 | Named the canonical `cargo clippy ... -- -D warnings` mechanism and prohibited lint policy in `RUSTFLAGS` |
| F18 | Required `#[ignore = "reason"]` with a tracked issue or active exception |
| F19 | Replaced portable PostgreSQL wording with relational-database wording and moved system temporary-path details to Appendix A |
| F20 | Defined evidence class, evidence receipt, receipt promotion, and deployment tripwire |
| F21 | Added inline validation baselines and the version-sensitive claim register |
| F22 | Recorded `max_temp_directory_size` availability since DuckDB 0.10.3 and narrowed the open work to pin confirmation, path, quota, and exhaustion testing |
