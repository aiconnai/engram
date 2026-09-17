# Rust Engineering Guardrails

## A practical guide to using Rust as your primary language without turning safety, types, and performance into unnecessary complexity

**Status:** engineering reference (rationale and examples)  
**Scope:** backends, APIs, workers, pipelines, gateways, concurrent systems, infrastructure, data services, and internal libraries  
**Objective:** preserve Rust's main advantages (safety, predictability, performance, and safe refactoring) without incurring unnecessary costs in complexity, compilation time, CI, maintenance, or ergonomics.

**Relationship to the Rust Repository Engineering Standard:** the Standard defines the rules (MUST/SHOULD/MAY). This document explains *why* those rules exist, provides examples, and offers judgment criteria for cases that a rule does not cover. In the event of a conflict, the Standard takes precedence; this document should be updated to reflect the rule.

**Translation note:** this is an English translation of `RUST_ENGINEERING_GUARDRAILS_2.md`. Technical recommendations and example versions are preserved from the source. The examples have not been compiled or independently verified as part of this translation.

---

# Part I. Principles

## 1. The central principle

Rust allows many errors to be moved from runtime to compile time. This is one of the language's greatest strengths.

The risk appears when a team starts using the compiler not only to prevent invalid states, but also to represent every possible nuance of the system through types, generics, traits, lifetimes, macros, or typed states.

The main rule of this document is:

> Use the type system to eliminate real classes of errors. Do not use the type system merely because something can be represented through it.

A healthy Rust architecture seeks this balance:

```text
strong correctness
+ important invariants encoded in types
+ explicit code
+ controlled dependencies
+ predictable builds
+ justified abstractions
- ornamental abstractions
- unnecessary generics
- macros that hide control flow
- lifetimes propagated without benefit
- architectural complexity without a corresponding risk
```

---

## 2. What justifies complexity

Every new abstraction should fall into one of these categories. Use this rule during code review.

| Level | Meaning | Action |
|---|---|---|
| 1 | Eliminates a real class of errors | Prefer |
| 2 | Reduces conceptual duplication, not just lines of code | Evaluate |
| 3 | Merely looks elegant | Avoid |

Before introducing an abstraction, ask: does it eliminate a real class of errors? Will the compiler guarantee an important property? Will a new developer understand the reasoning without studying half the project? If most answers are "no," the abstraction probably is not justified.

### Case

You have monetary values in Brazilian reais and US dollars. These deserve distinct types:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BrlCents(i64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UsdCents(i64);
```

This prevents accidental mixing. Note, however, what the type **does not** guarantee: in a release build, `+` on `i64` silently wraps on overflow. If money represents a real invariant, the arithmetic must protect it as well:

```rust
impl BrlCents {
    pub fn checked_add(self, other: Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }
}
```

Alternatively, at minimum, set `overflow-checks = true` in the release profile so that overflow causes a panic rather than an incorrect value.

Now imagine building an entire system:

```rust
struct Money<Currency, Precision, SettlementState, AccountingState, Jurisdiction, TaxState> {
    value: i64,
    _marker: PhantomData<(Currency, Precision, SettlementState, AccountingState, Jurisdiction, TaxState)>,
}
```

This design may be appropriate for a highly regulated financial system. For a CRUD API that only stores prices in cents, it is excessive.

---

## 3. When to accept more complexity and when to simplify

Additional complexity is justified when it protects something valuable: money, identity, authorization, multi-tenancy, critical data, intensive concurrency, protocols, parsers exposed to hostile input, distributed systems, consistency, or infrastructure. In these cases, additional types, traits, and abstractions are an investment.

Prefer a direct solution when the problem is a simple CRUD application, webhook, internal administration tool, small job, operational script, trivial integration, or endpoint without a complex domain. Simple Rust is still Rust: you still get memory safety, freedom from data races, performance, tooling, safe refactoring, and strong types, without turning the system into a type-theory laboratory.

---

# Part II. Modeling and architecture

## 4. Watch out for overengineering

Rust rewards sophisticated architectures because the compiler can verify much of their structure. That incentive needs limits.

### Antipattern

```rust
trait Command {}
trait Validated {}
trait Authorized {}
trait Audited {}
trait Persistable {}

struct CreateUser<C, V, A, U, P> {
    command: C,
    validation: V,
    authorization: A,
    audit: U,
    persistence: P,
}
```

### Preferable when the domain is simple

```rust
pub async fn create_user(
    state: &AppState,
    actor: &Actor,
    input: CreateUserInput,
) -> Result<User, CreateUserError> {
    validate_input(&input)?;
    authorize(actor)?;

    let user = state.users.insert(input).await?;
    state.audit.record_user_created(actor, &user).await?;

    Ok(user)
}
```

The second example remains structured, testable, and explicit. The absence of a more sophisticated abstraction does not mean the absence of architecture.

---

## 5. Make invalid states unrepresentable only when the benefit is real

The principle "parse, don't validate" is extremely powerful: validate once, at the boundary, and carry the proof in the type.

### Good use

```rust
struct RawEmail(String);
struct ValidEmail(String);

impl TryFrom<RawEmail> for ValidEmail {
    type Error = EmailError;

    fn try_from(value: RawEmail) -> Result<Self, Self::Error> {
        if value.0.contains('@') {
            Ok(Self(value.0))
        } else {
            Err(EmailError::Invalid)
        }
    }
}

fn send_email(to: &ValidEmail) {
    // The compiler prevents sending to an email address that has not been validated.
}
```

### A case where it can become excessive

```rust
struct DraftLead;
struct ParsedLead;
struct NormalizedLead;
struct EnrichedLead;
struct ScoredLead;
struct RoutedLead;
struct PersistedLead;
```

This model may be excellent for a critical pipeline, and unnecessary if all these states exist only within a 40-line function.

The question is not "is this elegant?" The question is "which concrete bug does this typing prevent?"

---

## 6. Numeric conversions: prefer `TryFrom` over `as`

`as` silently truncates, wraps, and changes signedness. At boundaries such as parsing, FFI, databases, and protocols, use `TryFrom`/`try_into` and handle the error:

```rust
let port: u16 = raw_port.try_into().map_err(|_| ConfigError::InvalidPort(raw_port))?;
```

Reserve `as` for cases where loss is impossible or intentional, and explain that in a comment.

---

## 7. Do not make everything generic

Generics are a tool for reuse and specialization. They should not become the default form of dependency injection.

### Excessive

```rust
struct Service<R, C, E, A, M, T>
where
    R: Repository,
    C: Cache,
    E: EventBus,
    A: Authorizer,
    M: Metrics,
    T: Tracer,
{
    repository: R,
    cache: C,
    events: E,
    auth: A,
    metrics: M,
    tracer: T,
}
```

This design produces enormous types, long error messages, and broad recompilation.

### A simple alternative

```rust
struct Service {
    repository: Arc<dyn Repository>,
    cache: Arc<dyn Cache>,
    events: Arc<dyn EventBus>,
}
```

Or, when the implementations are stable:

```rust
struct Service {
    repository: PostgresRepository,
    cache: RedisCache,
    events: NatsEventBus,
}
```

There is no obligation to make everything interchangeable. The best design reflects actual substitution requirements.

---

## 8. Use traits for shared behavior or real boundaries

Traits are excellent for contracts. Artificial traits increase the system's cognitive burden.

### Good use

```rust
#[async_trait::async_trait]
trait PaymentProvider {
    async fn charge(&self, request: ChargeRequest) -> Result<ChargeResponse, PaymentError>;
}
```

There are plausible implementations here: Stripe, Adyen, and a fake provider for tests.

### A note on `async fn` in traits and `dyn`

Native `async fn` in traits, stable since Rust 1.75, **is not dyn-compatible**: `Arc<dyn PaymentProvider>` does not compile with it. If you need dynamic dispatch, as in the "simple" alternative in section 7, use `async_trait` or explicitly return `Pin<Box<dyn Future + Send>>`. If you only need static generics, native `async fn` is sufficient and avoids a per-call allocation. Choose deliberately; do not mix both approaches in the same contract without a reason.

### A questionable case

```rust
trait UserIdProvider {
    fn user_id(&self) -> UserId;
}
```

If there is only one struct with a `user_id` field, a regular method is sufficient.

---

## 9. Prefer concrete types until there is a reason to generalize

```text
first case: use a concrete type
second case: observe the difference
third case: consider an abstraction
```

Start with `struct PostgresUserRepository { pool: PgPool }`, not `Repository<D, Q, E, M, T>`. Generalize when there is a real second or third backend, not an imaginary one.

---

# Part III. Ownership, borrowing, and lifetimes

## 10. Ownership should simplify the system, not spread through every API

### Questionable

```rust
struct Context<'a, 'b, 'c> {
    config: &'a Config,
    db: &'b Database,
    metrics: &'c Metrics,
}
```

This is correct, but the complexity grows quickly when structures are stored, moved, or shared between tasks.

### A common alternative in applications

```rust
#[derive(Clone)]
struct AppState {
    config: Arc<Config>,
    db: PgPool,
    metrics: Arc<Metrics>,
}
```

`Arc` has a cost. In servers, architectural clarity is often more valuable than avoiding a few atomic operations on noncritical paths.

---

## 11. Do not fight the borrow checker on principle

When the borrow checker pushes back, either the code violates a real invariant or the chosen ownership model does not represent the problem well. Do not automatically assume the compiler is "getting in the way"; do not rewrite half the system just to satisfy a sophisticated lifetime either.

If you need to share mutable state between tasks, `Arc<Mutex<T>>` may be exactly the right solution. It is not a failure of ownership; it explicitly represents shared ownership with synchronized mutation.

---

## 12. Avoid automatic cloning as a universal solution

Before cloning, understand what you are cloning: is it an inexpensive `Arc`? A large `String`? A `Vec` containing millions of elements? Does the task actually need to own the value?

### Good use

```rust
let state = Arc::clone(&state);
tokio::spawn(async move {
    process(state).await;
});
```

The semantics are explicit here.

---

# Part IV. Async and concurrency

## 13. Async Rust requires its own discipline

Async combines ownership, `Send`/`Sync`, `Pin`, cancellation, timeouts, backpressure, task lifecycles, and resource cleanup. The risk is not simply "making it compile"; it is producing code that compiles but still has incorrect semantics.

---

## 14. Every task should have a conceptual owner

### Questionable

```rust
tokio::spawn(async move {
    consume_messages().await;
});
```

Who cancels it? Who observes the error? Who ensures shutdown? Who restarts it? There is also a surprising detail: **a panic inside a spawned task is silent** unless someone `.await`s its `JoinHandle`.

### Preferable

```rust
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

let token = CancellationToken::new();
let mut tasks = JoinSet::new();

tasks.spawn(consume_messages(token.child_token()));
tasks.spawn(flush_metrics(token.child_token()));

tokio::signal::ctrl_c().await?;
token.cancel();

while let Some(result) = tasks.join_next().await {
    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => tracing::error!(error = %e, "task finished with an error"),
        Err(e) if e.is_panic() => tracing::error!("task panicked"),
        Err(_) => {} // Cancelled.
    }
}
```

`JoinSet` provides structural ownership (the tasks terminate with the set), `CancellationToken` provides cooperative shutdown, and the final loop observes errors and panics. In larger services, add explicit supervision and restart behavior.

---

## 15. Treat cancellation as normal control flow

A `Future` can be dropped at any `.await` point. In practice, this happens primarily through `tokio::select!`, `timeout`, and shutdown. Do not assume that everything after an `.await` will execute.

### A dangerous case

```rust
reserve_inventory().await?;
charge_customer().await?;
confirm_inventory().await?;
```

Cancellation between charging and confirmation leaves the system inconsistent. The solution involves transactions, idempotency, sagas, an outbox, compensation, or persisted state. Rust protects memory; it does not protect distributed logic.

### Cancellation safety

Tokio's documentation marks each method as *cancel safe* or not. A cancelled `recv()` does not lose a message; a cancelled `read_exact()` can lose bytes already read. Check before placing a future in `select!`. If the operation is not cancel safe, isolate it in its own task and communicate through a channel.

---

## 16. Use timeouts for external calls

```rust
use std::time::Duration;
use tokio::time::timeout;

let response = timeout(Duration::from_secs(3), client.send(request))
    .await
    .map_err(|_| Error::Timeout)??;
```

An external call without a timeout can keep tasks, connections, and memory occupied indefinitely.

---

## 17. Control concurrency and backpressure

Never assume that unlimited `spawn` calls will scale.

### Dangerous

```rust
for job in jobs {
    tokio::spawn(process(job));
}
```

With millions of jobs, this creates millions of tasks.

### Be careful with the obvious alternative

```rust
stream::iter(jobs)
    .for_each_concurrent(32, |job| async move { process(job).await })
    .await;
```

This limits concurrency, but all 32 futures run **inside a single task, on the same thread**. There is no parallelism across cores, and any CPU-bound or blocking portion of `process` stalls the other 31. It is suitable for lightweight, pure I/O; it is not suitable for a "high-concurrency worker."

### Preferable: bounded concurrency with actual parallelism

```rust
use futures::stream::{self, StreamExt};

stream::iter(jobs)
    .map(|job| tokio::spawn(process(job)))
    .buffer_unordered(64)
    .for_each(|joined| async {
        match joined {
            Ok(Ok(())) => {}
            Ok(Err(e)) => tracing::error!(error = %e, "job failed"),
            Err(e) => tracing::error!(error = %e, "task panicked"),
        }
    })
    .await;
```

The stream is lazy, so at most 64 tasks exist at the same time, and each can be scheduled on any worker thread. `JoinSet` + `tokio::sync::Semaphore` is the equivalent alternative when you need more control.

Choose the limit according to the database, external APIs, CPU, memory, and desired throughput, then measure it.

---

## 18. Do not block the executor: use `spawn_blocking`

Synchronous I/O, heavy CPU work, compression, password hashing, large serialization workloads, and calls to blocking C libraries do not belong in a regular async task. A blocked worker thread means one fewer worker for the entire process, and a single large `std::fs::read` can explain otherwise mysterious p99 latency.

```rust
let hash = tokio::task::spawn_blocking(move || argon2_hash(password)).await??;
```

For genuinely CPU-bound work at scale, consider a dedicated pool (`rayon`) and a channel bridge rather than saturating Tokio's blocking pool.

---

## 19. Choose locks deliberately

```text
std::sync::Mutex   very short critical sections, with no await inside
tokio::sync::Mutex when the lock needs to span an .await
RwLock             when reads truly dominate and the access pattern justifies it
atomics            very specific cases
channels           ownership through message passing
```

`std::sync::Mutex` is Tokio's own default recommendation for short sections: it is cheaper, and the compiler helps. The standard library's `MutexGuard` is not `Send`, so holding it across an `.await` in a `Send` task does not even compile. If you find yourself switching to `tokio::sync::Mutex` just to "make it compile," stop: you are probably holding a lock during I/O (section 20).

---

## 20. Do not hold locks during I/O

### Bad

```rust
let mut state = shared.lock().await;
state.status = Status::Processing;
external_api_call().await?;
state.status = Status::Done;
```

### Better

```rust
{
    let mut state = shared.lock().await;
    state.status = Status::Processing;
}

external_api_call().await?;

{
    let mut state = shared.lock().await;
    state.status = Status::Done;
}
```

If consistency requires atomicity between the two points, model the problem using another mechanism, such as persisted state, a state machine, or a channel with a single owner.

---

## 21. There is no asynchronous `Drop`

`Drop` is synchronous. Resources that require async cleanup, such as closing a remote session, sending an ACK, or flushing over the network, cannot rely on `Drop`. Expose an explicit `async fn close(self)`, call it on the normal path and during shutdown, and treat `Drop` without `close()` as best effort (at most, a warning log).

---

# Part V. Errors

## 22. Use typed errors at important boundaries

```rust
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CreateUserError {
    #[error("invalid email")]
    InvalidEmail,

    #[error("user already exists")]
    AlreadyExists,

    #[error("persistence failure")]
    Database(#[from] sqlx::Error),
}
```

This enables explicit mapping to HTTP, metrics, and logs. Applying `#[non_exhaustive]` to public enums allows variants to be added without breaking callers.

---

## 23. Do not build an enormous error hierarchy unnecessarily

A public library may need highly specific errors; an internal application generally does not. If two variants always lead to the same operational action, consider whether they actually need to be distinct.

---

## 24. `anyhow` and concrete errors can coexist

```text
libraries: concrete errors
domain: concrete errors
binaries and orchestration: anyhow (with .context()) is appropriate
```

```rust
async fn main_flow() -> anyhow::Result<()> {
    let config = load_config().context("loading configuration")?;
    let pool = connect(&config).await.context("connecting to the database")?;
    run(pool).await
}
```

There is no merit in creating a 25-variant enum just for `main`. Use `.context()` so that the `source()` chain tells the story of the error.

---

## 25. Avoid `unwrap()` and `expect()` on normal production paths

`expect()` is appropriate when failure is impossible by construction:

```rust
use std::sync::LazyLock;

static SLUG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z]+$").expect("valid static regex"));
```

Questionable:

```rust
let user = repository.find(id).await.unwrap();
```

Databases, networks, configuration, and external input fail. Treat these failures as part of the system. Consider `clippy::unwrap_used` as a workspace lint, with local `#[allow]` annotations in tests.

---

# Part VI. Macros and procedural macros

## 26. Macros should reduce boilerplate, not hide architecture

`#[derive(Debug, Clone, Serialize, Deserialize)]` is a good use. An internal macro that opens a transaction, validates authorization, adds tracing, executes a query, publishes an event, retries, and transforms errors turns operational flow into magic. If the behavior matters to understanding the system, prefer explicit code.

---

## 27. Procedural macros increase compilation and debugging costs

Each derive looks inexpensive in isolation; in a large workspace, dozens of procedural macros add up. Before adding a macro-heavy dependency, ask: what boilerplate does it eliminate? What build cost does it add? How difficult will it be to debug the expansion (`cargo expand`)? Is it well maintained?

---

# Part VII. `unsafe` and FFI

## 28. Treat `unsafe` as a safety boundary

`unsafe` means the compiler has delegated the proof of certain invariants to you. Every `unsafe` block should have a `SAFETY:` comment containing the local proof:

```rust
// SAFETY: `ptr` was obtained from a valid allocation, is aligned
// for `T`, and remains alive throughout this read.
let value = unsafe { ptr.read() };
```

If you cannot explain the reasoning in a few lines, the abstraction is not yet mature.

### Make the rule enforceable

```toml
[workspace.lints.rust]
unsafe_op_in_unsafe_fn = "warn"

[workspace.lints.clippy]
undocumented_unsafe_blocks = "warn"
```

In every crate that does not need `unsafe`, which should be most of them, use `#![forbid(unsafe_code)]`.

---

## 29. Concentrate `unsafe` in small modules with safe APIs on top

```text
safe application
      |
safe wrapper
      |
small unsafe module
      |
OS / C / hardware
```

The rest of the application should not need to reason about pointers, aliasing, or manually maintained invariants. Avoid scattering `unsafe` through handlers, business rules, and domain code.

---

## 30. Use specialized tools for `unsafe` code

Use Miri, sanitizers, fuzzing, property tests, and concurrency tests (`loom`) where applicable. The compiler cannot verify the invariants you have assumed manually.

---

## 31. FFI should be a thin, distrustful boundary

```text
C library -> ffi.rs -> safe_adapter.rs -> domain
```

Never trust data arriving through the boundary: validate null pointers, sizes, encoding, alignment, lifetimes, thread safety, and error codes. Rust cannot prove that an external library has honored its contract.

Explicitly document memory ownership: who creates it, who destroys it, whether it can be shared, whether it can cross threads, and how long the pointer remains valid. Ambiguous ownership is the greatest FFI risk.

---

# Part VIII. Dependencies and toolchain

## 32. Every crate adds maintenance surface

| Criterion | Question |
|---|---|
| Necessity | Does the problem justify a dependency? |
| Maintenance | Has the crate had recent activity? |
| Security | Is there a concerning history? |
| Complexity | How many transitive dependencies does it introduce? |
| Build | Does it use procedural macros or a native build? |
| Portability | Does it require system libraries? |
| API | Does it appear stable? |
| License | Is it compatible with the project? |

An entire dependency for five lines of code is not worthwhile: the real cost includes supply-chain exposure, compilation time, updates, licensing, vulnerabilities, and future breakage.

---

## 33. Control features

Avoid `features = ["full"]` when you use only a small part of a library:

```toml
tokio = { version = "1", features = ["rt-multi-thread", "macros", "signal"] }
```

Run `cargo tree -e features` periodically to discover what enabled each feature.

---

## 34. Commit `Cargo.lock`

Always, for both applications and libraries. Cargo's current guidance is to commit the lockfile in both cases; the `.gitignore` template no longer ignores it. Libraries should also be tested in CI against realistic dependency resolution, using `cargo update` before testing or `-Zminimal-versions` where applicable.

---

## 35. Automate dependency auditing and hygiene

```text
cargo audit      advisories
cargo deny       licenses, advisories, sources, duplicates
cargo machete    declared but unused dependencies
```

`cargo deny` is the most valuable: a license and source policy is cheaper to maintain as a file than through manual review.

---

## 36. Centralize versions, lints, and the toolchain in the workspace

```toml
# Cargo.toml (root)
[workspace.dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde = { version = "1", features = ["derive"] }

[workspace.lints.clippy]
unwrap_used = "warn"
undocumented_unsafe_blocks = "warn"
```

```toml
# In each crate
[dependencies]
tokio = { workspace = true }

[lints]
workspace = true
```

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.89.0"
components = ["rustfmt", "clippy"]
```

This eliminates version drift between crates, ensures everyone, including CI, builds with the same compiler, and makes lint policy a single decision. Define an explicit MSRV policy (`rust-version` in `Cargo.toml`) and an edition policy, and treat toolchain updates as regular PRs.

---

# Part IX. Compilation time and builds

## 37. Compilation time is an architectural property

Do not treat slow builds solely as a hardware problem. The main factors are generics and monomorphization, procedural macros, large crates, excessive features, heavy dependencies, build scripts, linking, and debug information.

Measure before optimizing:

```bash
cargo build --timings          # Chart the dominant crates and actual parallelism.
cargo check                    # Use during development when you do not need the binary.
```

---

## 38. Configure the development profile for fast feedback

```toml
[profile.dev]
debug = "line-tables-only"     # Useful backtraces, far less debug information.

[profile.dev.package."*"]
opt-level = 2                  # Optimize dependencies, not your own code.
```

The second setting is particularly valuable in projects with heavy serialization, cryptography, or parsing: dependencies change infrequently and can be built with optimizations once. Avoid turning the debug build into a disguised release build.

---

## 39. Split crates along real boundaries, not out of an obsession with modularity

### A good split

```text
workspace
  api-server
  worker
  domain
  persistence
  integrations
```

### Excessive

```text
user-id, user-name, user-email, user-status, user-validation, user-mapper, ...
```

Split by architectural responsibility, not by trivial entity. At the other extreme, avoid the "god crate": if `domain`, `database`, `http`, `integrations`, `jobs`, and `cli` are in a single crate, small changes cause broad recompilation.

---

## 40. Watch generics in compilation hot spots

A generic helper used hundreds of times produces substantial monomorphization. In noncritical areas, `Box<dyn Handler>` can reduce costs compared with `Handler<T1, T2, T3, T4>`. Do not trade runtime performance without measuring, but do not treat monomorphization as free either. "Zero-cost abstraction" refers to runtime; there are two separate budgets: the runtime budget and the build-time budget.

---

## 41. Linkers and caching

In large projects, linking can account for a significant share of build time. `lld`, or `mold` on Linux, often provides a direct improvement:

```toml
# .cargo/config.toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

Validate correctness, debugging, profiling, and reproducibility after making the change.

`sccache`, locally and in CI, can provide substantial improvements; the key is to measure the hit rate. A poorly configured cache only adds infrastructure.

---

## 42. The `target` directory needs governance

It grows because of debug builds, release builds, incremental compilation, multiple targets, benchmarks, examples, and test artifacts. Monitor its size. `cargo clean` recovers space but destroys all reusable artifacts; do not use it routinely, and understand what is causing the growth. In large workspaces, consider a `CARGO_TARGET_DIR` shared within the project and a separate `target` for Rust Analyzer, so that the IDE and terminal do not invalidate each other's artifacts.

---

## 43. The IDE and hardware are part of the resource budget

Rust Analyzer consumes significant CPU and RAM in large workspaces. Warning signs include constant indexing, autocomplete latency, and continuously growing memory usage. Reduce the analyzed scope when the monorepo has dozens of crates unrelated to your current work.

64 GB of RAM helps considerably. Even so, a workspace that consumes 25 GB to compile something simple signals a poor architecture. Hardware should remove legitimate bottlenecks, not hide waste.

---

# Part X. CI

## 44. Rust CI should be designed as an internal product

A mature pipeline minimizes recomputation and separates fast checks from expensive ones.

### Pull request (fast)

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace            # Or cargo nextest run.
cargo deny check                  # Where applicable.
```

### Main (after merge)

```text
integration tests, coverage, release build, container build,
security checks, smoke tests
```

### Nightly or scheduled

```text
extended fuzzing, benchmarks, dependency updates, cross-compilation,
sanitizers, Miri, full integration environments
```

Do not build release artifacts on every commit if nobody needs them.

---

## 45. Caching requires correct invalidation

Do not blindly cache `target/`. The key must account for the toolchain, `Cargo.lock`, target triple, features, profile, OS, and architecture. `Swatinem/rust-cache` already handles this correctly for GitHub Actions; use it instead of constructing the key by hand. Incorrect caching produces behavior that is difficult to reproduce.

---

## 46. Do not turn CI into ceremony without a return

If a job costs several minutes and never finds problems, question its value. CI exists to reduce risk, not to demonstrate rigor.

---

## 47. Persistent runners can make sense

Ephemeral runners lose caches and recompile extensively. Persistent or self-hosted runners improve throughput, but introduce isolation, cleanup, security, secrets, and maintenance as new concerns. The decision needs to account for the total cost.

---

# Part XI. Performance and binary size

## 48. Do not use Rust to justify premature optimization

Rust already produces fast software. That does not mean every function needs to be micro-optimized.

```text
profile -> identify -> optimize -> measure again
```

Replacing O(n^2) with O(n log n) is far more valuable than eliminating small clones on an irrelevant path.

---

## 49. Do not assume Rust produces small binaries

A service using Tokio, TLS, HTTP, serialization, tracing, database drivers, and compression produces large executables. For ordinary servers, this is irrelevant; for edge, serverless, or embedded distribution, it may matter. LTO, stripping, and profile adjustments reduce size at the cost of build time. Do not enable LTO or `codegen-units = 1` because they "look professional." Measure against the actual objective: latency, throughput, size, startup, or memory.

---

# Part XII. Testing

## 50. Strong types do not replace tests

Rust does not prevent incorrect business rules, incorrect queries, incorrect calculations, unsuitable timeouts, inconsistent distributed flows, conceptually incorrect authorization, or broken integrations.

---

## 51. Use a pragmatic testing pyramid

| Layer | Objective |
|---|---|
| Unit | Pure logic and rules |
| Integration | Databases, queues, APIs, and adapters |
| Contract | Compatibility between services |
| E2E | Genuinely critical flows |
| Property tests | Mathematical or structural invariants |
| Fuzzing | Parsers, protocols, and surfaces exposed to hostile input |

---

## 52. Property testing and fuzzing work very well with Rust

If a function normalizes monetary values, `normalize(normalize(x)) == normalize(x)` is a property, and it may be more valuable than dozens of manual examples (`proptest`). Prioritize fuzzing (`cargo fuzz`) for parsers, decoders, protocols, files, external inputs, FFI wrappers, and `unsafe` code.

---

# Part XIII. Observability and security

## 53. Types do not replace logs, metrics, and tracing

A system can be perfectly memory-safe and operationally invisible. Observability answers: which request failed, which tenant, which dependency, how long it took, how many attempts, which error, and which correlation.

---

## 54. Do not log sensitive data for convenience

`#[derive(Debug)]` exposes tokens, CPF numbers, email addresses, credentials, and payloads. Avoid `tracing::info!(?request, ...)` when `request` contains PII or secrets. Use `secrecy::SecretString` for credentials (its `Debug` output is redacted by construction), and implement `Debug` manually or create dedicated logging structs for entities containing PII.

---

## 55. Do not log the same error at every layer

A repository, service, handler, and middleware all logging the same error produce four copies of the log. Choose the layer responsible for final observability; lower layers enrich the error with context without logging it.

---

## 56. Memory safety is not application security

Rust reduces use-after-free, double free, data races (not race conditions in general), and buffer misuse in safe Rust. It does not prevent SQL injection in manually constructed SQL, IDOR, incorrect authorization, SSRF, CSRF, exposed secrets, insecure configuration, logic bugs, or supply-chain attacks.

Security exists in layers: types, validation, authorization, parameterized queries, least privilege, secrets, network policies, auditing, monitoring, and dependency security. Rust is an important layer; it is not the entire security architecture.

---

# Part XIV. Databases, APIs, and configuration

## 57. Do not try to model the entire database in Rust types

`sqlx` and strong types are excellent, but not every constraint needs to become typestate. Use the database for what belongs in the database (unique constraints, foreign keys, not-null constraints, check constraints, and transaction isolation), and Rust for domain properties. Do not duplicate complexity without a benefit.

---

## 58. Transactions must reflect a business unit of work

```rust
let mut tx = pool.begin().await?;

create_order(&mut tx, order).await?;
reserve_stock(&mut tx, items).await?;
record_audit(&mut tx, event).await?;

tx.commit().await?;
```

A transaction should not exist simply because "it is safer"; it needs to reflect a real atomicity requirement.

---

## 59. Handlers should be thin; extractors are not a framework

```rust
async fn create_user_handler(
    State(state): State<AppState>,
    Json(input): Json<CreateUserInput>,
) -> Result<Json<UserResponse>, ApiError> {
    let user = state.users.create(input).await?;
    Ok(Json(user.into()))
}
```

Business rules belong in the domain. Custom extractors are excellent for authentication, tenants, correlation IDs, and validated input; avoid hiding the entire application in a stack of extractors with side effects.

---

## 60. Configuration: parse at startup and fail early

Apply the same principle from section 5 to configuration: the `Config` type should exist only if it is valid.

```rust
struct RawConfig {
    database_url: String,
    port: String,
}

struct Config {
    database_url: DatabaseUrl,
    port: u16,
}

impl TryFrom<RawConfig> for Config {
    type Error = ConfigError;

    fn try_from(raw: RawConfig) -> Result<Self, ConfigError> {
        Ok(Self {
            database_url: raw.database_url.parse()?,
            port: raw.port.parse().map_err(|_| ConfigError::InvalidPort(raw.port))?,
        })
    }
}
```

Do not wait for the first request to discover that a required variable is missing or malformed.

---

# Part XV. Workspaces and public APIs

## 61. Structure the workspace around stable boundaries

```text
workspace/
  crates/
    domain/
    auth/
    persistence/
    integrations/
    platform/
  apps/
    api/
    worker/
    cli/
```

Avoid both a complete monolith and excessive microcrates.

---

## 62. Keep dependencies pointing in a clear direction

```text
apps -> application -> domain -> ports
infrastructure implements ports
```

Do not turn Clean Architecture into a ritual. If a small service needs only a handler, service, and repository, that is sufficient.

---

## 63. Minimize the public surface

Use `pub(crate)` when an API does not need to leave the crate. A smaller public surface means fewer contracts, less coupling, and more freedom to refactor. For internal crates published for other teams, `cargo semver-checks` in CI prevents accidental breaking changes.

---

# Part XVI. Documentation

## 64. Document the "why"

Comments that repeat the code age poorly.

```rust
// Bad
// Increment the counter.
counter += 1;

// Good
// We keep this counter separate because retries must not
// increase the completed-operations metric.
counter += 1;
```

Public APIs should have rustdoc with `# Errors` and `# Panics` sections where applicable, and executable examples where relevant. Every `unsafe` block explains its invariant with `// SAFETY:` (section 28); this documentation is part of the safety contract.

---

# Part XVII. Language choices by layer

## 65. Rust is the backend default; the product defines the exceptions

```text
Web frontend          Next.js / TypeScript
Backend               Rust (default)
Apple mobile          Swift / SwiftUI, with an optional Rust core
Shared core           Rust when there is a real benefit
Exploration and ML    Python
Data infrastructure   Rust when performance, safety, or concurrency justifies it
```

Keep Python for data exploration, notebooks, statistics, ML experiments, disposable scripts, or quick hypothesis validation. Stabilized components migrate to Rust when there is a benefit.

Do not use Rust merely to achieve a 100% Rust stack. The language should serve the product; the product should not serve the language.

---

# Part XVIII. Refactoring

## 66. Use the compiler as a refactoring partner

When changing `struct UserId(Uuid)` or the signature of `fn process(user: User)`, the compiler reveals much of the affected surface. Take advantage of that.

## 67. Do not turn every compiler error into a local patch

When a change produces 80 errors, look for the structural cause. They are often consequences of a single modified contract. Fixing symptoms one by one produces worse code.

---

# Part XIX. A personal policy for Rust specialists

## 68. The specialist's risk

The better you become at Rust, the easier it becomes to justify sophisticated solutions. Set a deliberate limit:

```text
"I can do it" does not imply "I should do it."
```

Specialization should increase your ability to simplify, not merely your ability to build complex abstractions.

---

# Part XX. The golden rule

## 69. Rust engineering at its best

The best Rust engineering is not the kind that uses the most language features. It uses exactly the features needed to turn important system properties into verifiable guarantees, while keeping the code simple enough to understand, change, test, and operate.

```text
strong types for real invariants
generics when there is real reuse
traits when there is a real contract
unsafe only within small boundaries
async with clear ownership and cancellation
CI proportional to risk
measured performance
controlled dependencies
justified abstractions
preserved simplicity
```

The goal is not to defeat the compiler. The goal is to make the compiler a tool that makes the system harder to break without making it harder to understand.

---

# Appendix A. Code review checklist and merge gate

Use this table during PRs. Each row points to the section that explains its rationale.

| Area | Question | Section |
|---|---|---|
| Problem | Does the code solve the real problem? Is there a more direct solution? | 1-3 |
| Types | Does the type prevent a real error? | 2, 5 |
| Numeric behavior | Do conversions use `TryFrom`? Is monetary arithmetic checked? | 2, 6 |
| Ownership | Is there an unnecessary clone? Is the lifetime necessary, or would `Arc` simplify it? | 10-12 |
| Generics / Traits | Is there more than one real implementation? Does the contract represent a real boundary? | 7-9 |
| Async | Who owns the tasks? Are panics observed? | 14 |
| Cancellation | Does the flow support cancellation? Are futures in `select!` cancel safe? | 15 |
| Timeout | Do external calls have time limits? | 16 |
| Backpressure | Is concurrency bounded, and does it actually run in parallel? | 17 |
| Blocking | Are CPU-bound work and synchronous I/O in `spawn_blocking`? | 18 |
| Locks | Is a lock held across I/O? | 19-20 |
| Errors | Can the caller act on the error? Is there an `unwrap` in production? | 22-25 |
| Logs | Is there duplication or sensitive-data leakage? | 54-55 |
| Unsafe / FFI | Is there a SAFETY comment? Is ownership explicit? | 28-31 |
| Dependencies | Is the new crate justified? Are only necessary features enabled? | 32-33 |
| Compilation | Does the change significantly increase monomorphization or macros? | 27, 40 |
| CI | Does it remain proportional to risk? | 44-46 |
| Database | Does the transaction represent a real atomicity requirement? | 58 |
| Security | Is authorization explicit? | 56 |
| Tests | Has the critical property been tested? | 50-52 |
| Readability | Will another engineer understand the code? | 64, 68 |

---

# Appendix B. Practical cases

## Case A: a simple webhook

Requirement: receive JSON, validate the signature, normalize eight fields, insert into Postgres, and return 200.

Sufficient architecture: an Axum handler, signature validator, input struct, service function, and sqlx repository.

Excessive architecture: CQRS, event sourcing, a generic command bus, six typestates, a repository abstraction with five generics, and a transaction macro.

## Case B: a critical gateway

Requirement: authentication, rate limiting, multi-tenancy, auditing, retries, timeouts, circuit breaking, metrics, and high concurrency.

Strong abstractions are justified here:

```rust
struct AuthenticatedRequest<T> {
    actor: Actor,
    tenant: TenantId,
    payload: T,
}
```

An internal handler accepts only authenticated requests. This typing eliminates a real class of errors.

## Case C: a data pipeline

Pipeline: ingest -> normalize -> validate -> dedupe -> index -> sync.

Typestates are useful if stages are reused separately and their order must be guaranteed (`RawRecord` -> `ValidRecord`). There is no need to create a type for every intermediate transformation if those types never leave the module.

## Case D: a high-concurrency worker

Use the pattern in section 17 (`spawn` + `buffer_unordered`, or `JoinSet` + `Semaphore`), move CPU-bound portions to `spawn_blocking` (section 18), and then measure CPU, memory, database connections, queue latency, and external API limits.

## Case E: integration with a C-only SDK

Create `ffi/`, `safe_client/`, and `domain/`. The rest of the project knows only:

```rust
trait ExternalClient {
    fn execute(&self, input: Input) -> Result<Output, Error>;
}
```

The unsafe complexity remains contained (sections 29-31).

## Case F: an internal CRUD API

Low load and simple rules: do not optimize the design for imaginary scale. Axum, SQLx, Postgres, Serde, Tracing, and thiserror. Few crates, few concepts, explicit code.

---

# Appendix C. Signals

## Signs that the project is becoming overly sophisticated

| Signal | Possible interpretation |
|---|---|
| Compiler errors fill entire screens | Excessive generics |
| A new field requires changes to many traits | Coupled abstraction |
| Many types exist only to satisfy other types | Excessive typestate |
| Developers avoid touching certain modules | Accidental complexity |
| Macros hide important calls | Loss of readability |
| Build time increases sharply after small changes | Poor boundaries |
| Almost everything is `Arc<Mutex<_>>` | The concurrency model needs review |
| Almost everything is generic | Premature abstraction |
| Almost everything is a trait object | Abstraction may also be excessive |
| A simple PR changes 15 crates | Fragmentation or coupling |
| Many `full` features are enabled | Excessive dependency features |
| Unexplained p99 latency in a "lightweight" service | Blocking work inside the executor |

## Signs of a good Rust architecture

```text
readable domain code
explicit APIs
useful errors
few unsafe blocks, all with SAFETY comments
justified dependencies
supervised tasks
explicit timeouts
backpressure with actual parallelism
fast CI on the common path
strong tests for invariants
adequate observability
reproducible builds (toolchain and lockfile committed)
```
