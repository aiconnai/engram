# Engram Versioning System & Release Governance

> Canonical governance specification for semantic versioning, multi-crate compatibility, database schema evolutions, MCP protocol stability, and release distribution channels in Engram.

---

## 1. Overview & Core Philosophy

Engram operates as a modular, local-first AI memory infrastructure across multiple languages, runtimes, and protocols. Rather than forcing a lockstep version across disparate components (which creates false coupling and premature breaking releases), Engram adopts a **5-Plane Decoupled Semantic Versioning Architecture**:

```
 ┌─────────────────────────────────────────────────────────────────────────┐
 │ 1. Core Rust Engine & Binaries (engram-core, engram-server, engram-cli) │
 └────────────────────────────────────┬────────────────────────────────────┘
                                      │
       ┌──────────────────────────────┼──────────────────────────────┐
       ▼                              ▼                              ▼
┌──────────────┐             ┌──────────────────┐           ┌─────────────────┐
│ 2. MCP Tools │             │ 3. SQLite Schema │           │ 4. Client SDKs  │
│ Protocol     │             │ Migrations       │           │ (Python, TS,    │
│ (243+ Tools) │             │ (SCHEMA_VERSION) │           │  WASM)          │
└──────────────┘             └──────────────────┘           └─────────────────┘
                                      │
                                      ▼
             ┌──────────────────────────────────────────────────┐
             │ 5. Distribution Channels (crates.io, PyPI, npm,  │
             │    Homebrew, GitHub Releases, Docker)            │
             └──────────────────────────────────────────────────┘
```

---

## 2. The Five Versioning Planes

### Plane 1: Core Rust Engine & Binaries (`engram-core`)
- **Package**: `engram-core` (crates.io)
- **Binaries**: `engram-server`, `engram-cli`, `engram-watcher`, `engram-pdf-worker`
- **Specification**: Strict [Semantic Versioning 2.0.0](https://semver.org/) (`MAJOR.MINOR.PATCH`):
  - **`PATCH` (0.22.x $\to$ 0.22.y)**: Bug fixes, performance optimizations, documentation updates, non-breaking internal refactors.
  - **`MINOR` (0.22.0 $\to$ 0.23.0)**: New MCP tools, new CLI subcommands, additive schema migrations, new search features (e.g. new embedding providers or rerankers), backward-compatible API additions.
  - **`MAJOR` (0.x.y $\to$ 1.0.0)**: Breaking changes to public Rust APIs (`engram::*`), breaking protocol modifications, removal of deprecated MCP tools or CLI flags.

### Plane 2: Model Context Protocol (MCP) Surface & Tool Tiers
Engram exposes over 243 MCP tools categorized into tiered access levels:
- **`essential`** (15 core tools): Minimal memory lifecycle (`memory_create`, `memory_get`, `memory_update`, `memory_smart_retrieve`, `session_land`).
- **`standard`** (135 tools): Standard development workflows, knowledge graphs, identities, and session indexing.
- **`all`** (236+ tools): Comprehensive cognitive operations, multimodal sync, councils, and temporal evolutions.

#### Protocol Stability Rules:
1. **Additive Evolution**: New tools are introduced in `all` or `standard` without altering existing tool schemas.
2. **Schema Invariants**: Tool input/output schemas follow JSON-RPC 2.0 specifications defined in [`docs/contracts/advertised-surfaces.toml`](contracts/advertised-surfaces.toml).
3. **Deprecation Window**: Any planned tool deprecation must be preceded by a minor release logging a soft warning before eventual removal in a subsequent breaking release.

### Plane 3: SQLite Storage Schema & Migrations
- **Location**: [`src/storage/migrations.rs`](../src/storage/migrations.rs)
- **Mechanism**: Monotonically increasing integer `SCHEMA_VERSION` (e.g. `22`).
- **Invariants**:
  - **Additive Migrations**: Columns and tables are added with default values or nullable types.
  - **Zero Destructive Rewrites**: User databases (`~/.local/share/engram/memories.db`) must automatically migrate forward on startup without data loss.
  - **Decoupled from SemVer**: `SCHEMA_VERSION` advances only when disk layout changes, independent of crate patch/minor releases.

### Plane 4: Multi-Language Client SDKs & WASM
SDKs provide native ergonomics for application developers and AI agent frameworks (LangGraph, CrewAI, OpenAI Agents):

| Package | Language / Runtime | Source Path | Target Registry |
| :--- | :--- | :--- | :--- |
| `engram-client` | Python (>=3.10) | `sdks/python/pyproject.toml` | **PyPI** |
| `engram-client` | TypeScript / Node.js | `sdks/typescript/package.json` | **npm** |
| `engram-wasm` | WebAssembly (Rust) | `engram-wasm/Cargo.toml` | **crates.io** |

#### Compatibility Windows:
SDKs maintain independent version lifecycles and state their tested Core compatibility range in `docs/releases/channel-matrix.toml`:
- Example: `engram-client` (Python) `v0.5.0` is tested against Core `v0.20.0 .. v0.22.x`.
- SDKs gracefully fallback or return structured error results when connecting to older or newer core servers.

### Plane 5: Release Distribution Channels
Release integrity is enforced through matrix-based policy tracking:
- **Git Tags**: Annotated tags matching `vMAJOR.MINOR.PATCH` (e.g. `v0.22.0`).
- **GitHub Releases**: Cryptographically signed binary artifacts for Linux (x86_64, aarch64), macOS (Apple Silicon, Intel), and Windows.
- **Package Registries**: crates.io, PyPI (via Trusted Publishing OIDC), npm (via Provenance OIDC), Homebrew Tap (`aiconnai/engram/engram`).
- **Channel Matrix Policy**: [`docs/releases/channel-matrix.toml`](releases/channel-matrix.toml) tracks channel readiness and staleness gates (<30 days).

---

## 3. Tooling & Automation

Engram provides an automated version management CLI (`scripts/bump-version.py`) and standard build-tool targets.

### CLI Usage (`scripts/bump-version.py`)

```bash
# 1. Run full repository version consistency validation
python3 scripts/bump-version.py --check

# 2. Refresh channel matrix timestamp (prevents staleness gate errors)
python3 scripts/bump-version.py --refresh-matrix

# 3. Bump Core Rust Engine (updates Cargo.toml, channel-matrix.toml, badges, index.html)
python3 scripts/bump-version.py --core 0.23.0

# 4. Bump Python SDK (updates pyproject.toml and channel matrix compatibility)
python3 scripts/bump-version.py --python 0.6.0

# 5. Bump TypeScript SDK (updates package.json and channel matrix compatibility)
python3 scripts/bump-version.py --typescript 0.6.0

# 6. Bump WASM Crate
python3 scripts/bump-version.py --wasm 0.2.0
```

### Justfile / Makefile Targets

```bash
# Verify consistency across all packages and contracts
just version-check
# or: make version-check

# Refresh matrix timestamp
just version-refresh-matrix
# or: make version-refresh-matrix
```

---

## 4. Release Playbook (Step-by-Step)

When preparing an official release, follow this sequence:

1. **Preflight Validation**:
   ```bash
   just ci                  # Runs fmt, clippy, tests, wasm, doc checks
   just version-check       # Validates version consistency
   ```

2. **Execute Version Bump**:
   ```bash
   python3 scripts/bump-version.py --core <NEW_VERSION>
   ```

3. **Update Changelog**:
   Add release notes and highlights for `[NEW_VERSION]` in [`CHANGELOG.md`](../CHANGELOG.md).

4. **Commit & Tag**:
   ```bash
   git add Cargo.toml Cargo.lock docs/releases/channel-matrix.toml README.md docs/index.html CHANGELOG.md
   git commit -m "chore(release): bump engram-core to v<NEW_VERSION>"
   git tag -a v<NEW_VERSION> -m "Release v<NEW_VERSION>"
   git push origin main --tags
   ```

5. **CI/CD Matrix Attestation**:
   GitHub Actions `.github/workflows/release.yml` automatically triggers on tag push, builds multi-platform binaries, generates checksums, and publishes the release assets.
