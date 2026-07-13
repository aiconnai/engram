# Engram — Harness Progress (Live State)

| Field | Value |
|-------|-------|
| Project | `engram` |
| Active sprint | `Harness maintenance — live-state closeout` |
| Active task | `engram-10-of-10-live-state — make harness live state truthful and self-checking` |
| Active plan | `docs/harness/progress/2026-06-27-harness-live-state-closeout.md` |
| Last review | `2026-07-10 — pass: docs/harness/reviews/2026-07-10-engram-10-of-10-live-state-v4-post.md` |
| Last sensors | `2026-07-12 — status=pass (mode=full; timestamp 2026-07-12T17:04:23Z)` |
| Last commit | `2cfab4563d5da43932c1cc3aa6741eeea6b487ea` |
| Last live-state check | `2026-07-13 — status=pass (rtk bash docs/harness/bin/check-live-state.sh --progress docs/harness/progress.md)` |

> Sumário curto do trabalho ativo. Logs detalhados em `progress/`.

## Engram 10/10 Wave 2 — security integration with SQLite deferral

- **Status**: partially integrated; atomic descriptor-bound SQLite opening is
  deferred to a dedicated follow-up.
- Barrier v4 passed all 21 required and focused gates without exclusions; the
  immutable receipts and SHA-256 manifest are under
  `.omo/evidence/wave-2-integration-barrier-v4.md` and its sibling directory.
- Independent review v2 passed after verifying the Unix storage remediation and
  focused transport/PDF suites. The earlier FAIL remains historical evidence;
  this PASS is recorded separately in
  `docs/harness/reviews/2026-07-12-engram-10-of-10-wave2-v2-post.md`.
- No tag, release, real publish, or other publication occurred. The Cargo
  publication check was dry-run only.

### Superseding SQLite scope correction

- Subsequent Linux CI proved the descriptor-bound VFS candidate could not open
  the database through the stock SQLite Unix VFS. Two replacement designs were
  rejected: pathname aliases retained a regular-file TOCTOU, while transient
  hardlinks either broke pools or the stock VFS locking protocol.
- Commits `ef02ee7`, `98b8446`, and `0700cb9` were reverted. The earlier
  restrictive permissions and pre-open symlink rejection remain in scope.
- The barrier v4 and review v2 remain immutable historical evidence for the
  pre-CI candidate, but no longer constitute proof that atomic descriptor-bound
  opening is complete. That work requires a dedicated native shim or audited
  VFS implementation with Linux/macOS/BSD ABI and locking coverage.

### PR #189 CI remediation

- Cleared the Linux Clippy portability findings in the historical
  descriptor-bound candidate; those VFS changes were subsequently reverted.
- Replaced hard-coded test salts and WebSocket handshake nonces with runtime
  `OsRng` values, removing four CodeQL alerts and two Gitleaks findings without
  adding scanner allowlists.
- The salt helper returns `OsRng::gen()` directly and independently regenerates
  the second salt, avoiding static initialization patterns flagged by CodeQL.
- Exact CI Clippy, clean-tree Gitleaks, focused storage/cloud/WS/listener tests,
  formatting, doctor, diff-check, and independent review passed locally.

## Engram 10/10 Wave 3 — execution started

- PR #189 merged as `81be152c713230c082901899e6880579fcedabb3`; all
  required checks were green before merge.
- Todos 17–21 are the next dependency-ready lanes and are being executed on
  isolated branches/worktrees for later integration and independent review.
- Atomic descriptor-bound SQLite opening remains a separate deferred follow-up;
  Wave 3 does not claim or silently absorb that work.
- No tag, release, deployment, registry publication, or channel approval is in
  scope for this wave.

### Wave 3 integration result

- Todos 17–21 are integrated on `feat/engram-10-of-10-wave3`.
- HTTP trusted-proxy identity is explicit and bounded; the aggregate security
  gate, canonical real-binary journey, frozen retrieval corpus, and example
  smoke suite are executable and fail closed on their negative fixtures.
- `make ci` and the full harness sensors passed on the integrated branch with
  no exclusions.
- Independent review found that the first canonical journey used separate
  transport databases. Commit `4a70aa0` corrected the harness so stdio and
  authenticated HTTP share one caller-owned isolated database; HTTP asserts
  the record previously updated through stdio, and cleanup occurs only after
  both transport lifecycles finish.
- PR #191 CI exposed a removed Semgrep image tag in both the standalone and
  aggregate security jobs. Both workflows now use the verified official
  `semgrep/semgrep:1.169.0` image; scanner policy and failure behavior are
  unchanged.
- With the scanner restored, Semgrep identified 36 mutable action references
  in four existing workflows. Those references are now pinned to resolved
  commit SHAs while retaining version comments; the same `p/ci --error` scan
  reports zero findings without ignores or exclusions.

## Engram 10/10 Wave 4 — integrated implementation

- **Integration HEAD**: `2cfab4563d5da43932c1cc3aa6741eeea6b487ea` on
  `feat/engram-10-of-10-wave4` (Todos 22–24, 26–27, 37–39).
- Todo 24 had two consecutive independent-review FAILs; both findings were
  remediated in `6a42f008c012428396257a66c0cb830ae2d4cd88`. Per invariant 11 the
  human owner responded `autorizado`, scoped only to accepting/integrating that
  remediated SHA — not publication approval for any channel.
- First release dry-run `29271674475` failed closed before publication because
  the binary smoke assumed `engram-server --version`. Remediation `2cfab45`
  uses server `--help`, CLI `--version`, and a bounded empty-input PDF-worker
  protocol smoke.
- Superseding dry-run `29286930522` completed **success** on exact HEAD
  `2cfab4563d5da43932c1cc3aa6741eeea6b487ea`: four-target builds, checksums,
  SBOMs, signatures/provenance, and native artifact smoke passed. GitHub
  Release and Homebrew jobs were **skipped**. No tag, registry, release,
  Homebrew, or deploy write occurred. No claim that v0.22.0 or any SDK is
  published.
- Atomic descriptor-bound SQLite opening remains deferred (stock VFS could not
  satisfy race freedom, pool compatibility, cleanup, and locking across
  Linux/macOS/BSD without a native shim or audited VFS).

- Tasks 11–16 landed as `bbd49fc` (HTTP fail-closed), `fc37fff` (gRPC
  security), `73f4959` (WebSocket authentication), `bc05e81` (durable cloud
  key identity), `815d1af` (SQLite permissions), and `ce292ac` (bounded PDF
  parsing).
- Barrier remediation landed as `72bdf0b` (structured WebSocket peer-task
  cleanup) and `92734bc` (release packaging for the isolated PDF worker).
- The SQLite follow-up rejects a database symlink before SQLite opens it,
  preserving the target database bytes, journal mode, and sentinel data. The
  regression and runtime evidence are recorded with the Wave 2 barrier
  evidence under `.omo/evidence/`. Platforms without the Unix atomic
  no-follow boundary fail closed for filesystem databases and retain
  `:memory:` support.

## Engram 10/10 Wave 2 — HTTP listener security (Task 11)

- Public HTTP/SSE listeners now fail closed at startup unless an API key is
  configured; anonymous development access remains available on loopback.
- Bearer authentication and principal authorization run before rate limiting
  and MCP dispatch, including JSON-RPC notifications and SSE subscriptions.
- Authentication failures return `401`; authenticated principals without the
  required scope or workspace permission return `403`.
- Contract coverage includes real `engram-server` processes for loopback
  compatibility, public no-key refusal, keyed MCP requests, and keyed SSE.
- Evidence is recorded in the orchestrator-owned Task 11 evidence and report
  files under `.omo/` (outside this committed worktree).

## Engram 10/10 Wave 2 — WebSocket peer cleanup

- WebSocket connections now use structured peer-task cancellation: whichever
  send/receive task finishes first aborts and awaits the blocked peer before
  client registration cleanup returns.
- Deterministic regressions cover client disconnect, outbound send failure,
  and coordinator cancellation, preventing detached tasks on every ownership
  path.
- Targeted realtime tests, listener configuration tests, authenticated and
  anonymous real WebSocket handshakes, Clippy, formatting, and harness doctor
  pass on the Wave 2 cleanup worktree.

## Sprint ativa

- **Harness maintenance — live-state closeout**
- **Log**: [`progress/2026-06-27-harness-live-state-closeout.md`](./progress/2026-06-27-harness-live-state-closeout.md)
- **Status**: active — close stale live metadata after the completed bootstrap
  sprint and the merged lifecycle predicate follow-up. Execution HEAD for this live-state pass is
  `843fd52` (`843fd520cbd0eb4c2b1885fe11c997198beb2ca1`); historical PR #108 commit `e156810` remains contained in main history.

## Live-state self-check — 2026-07-10

- **Execution HEAD**: `843fd520cbd0eb4c2b1885fe11c997198beb2ca1` (`843fd52`).
- **Approved execution baseline**: `843fd520cbd0eb4c2b1885fe11c997198beb2ca1` (`843fd52`); this remains valid after later Wave 0 commits only when the checker can bind it to the current live-state review, Canvas, and snapshot commit metadata.
- **Approved live-state snapshot commit**: `3586a40e7952a051181d162028927a40bd6292f6`; this commit is the direct child of the approved execution baseline and introduced the checker/test/Canvas/review artifacts.
- **Sensor snapshot source**: `docs/harness/.sensors-last`, currently `status=pass`,
  `mode=full`, `timestamp=2026-07-10T00:27:38Z`.
- **Checker surface**: `rtk bash docs/harness/bin/check-live-state.sh --progress docs/harness/progress.md`.
- **Checker status**: pass for current progress; stale fixtures containing the old
  `1aa14e5` Last commit fail with remediation to use either current HEAD or the approved baseline bound to current review/Canvas/snapshot metadata.
- **Dirty worktree handling**: the checker reports `worktree_status=dirty|clean`
  explicitly as diagnostic output; dirty state is not treated as success or failure
  because agents run it before and after staging/commit boundaries.

### Required versus advisory workflow reconciliation

| Check | Live GitHub API status | Workflow source | Current contract |
|---|---|---|---|
| `Format` | branch-required | `.github/workflows/ci.yml` | Required CI job running `cargo fmt --all -- --check`. |
| `Clippy` | branch-required | `.github/workflows/ci.yml` | Required CI job running clippy with required feature set. |
| `Test (ubuntu-latest)` | branch-required | `.github/workflows/ci.yml` | Required Ubuntu test job for lib/tests, binary tests, and WASM checks. |
| `Documentation` | branch-required | `.github/workflows/ci.yml` | Required docs job covering MCP reference check and rustdoc. |
| `Security Audit` | branch-required | `.github/workflows/ci.yml` | Live branch-protection API currently lists this context as required; do not treat older advisory prose as current truth. |
| `Cargo Deny` | branch-required | `.github/workflows/ci.yml` | Live branch-protection API currently lists this context as required; do not treat older advisory prose as current truth. |
| `Harness Contract` | not in `required_status_checks.contexts` | `.github/workflows/harness-contract.yml` | Workflow exists, but the live API receipt does not list it as a required context; do not infer required status from workflow text. |
| `Harness Doctor Advisory` | advisory workflow job | `.github/workflows/harness-contract.yml` | Non-blocking `doctor.sh`; stays advisory and must not be inferred as required. |

## Sprint encerrada

- **Harness Engineering v0 — bootstrap & core gates**
- **Log**: [`progress/2026-05-30-harness-bootstrap.md`](./progress/2026-05-30-harness-bootstrap.md)
- **Status**: completed — the operational harness bootstrap, core gates,
  reviewer path hardening, and follow-up harness policy work have landed on
  `main`. Future work should start from a fresh task branch and update this
  live-state block instead of continuing to present v0 as active.

## Contexto e Motivação

O usuário está usando Claude Code CLI e Claude Code Sonnet em sessão/processo separado como reviewer cross-model para comparar workflows agentic em terminal/editor. A visão é que "the terminal is the product" e que harnesses reais (Context Engine, Planner, Memory Manager, Verifier, Tool Registry, Harness Config) devem viver onde o trabalho de engenharia de mais alto sinal já acontece: o repositório + CLI/editor.

Engram é posicionado de forma única porque ele *é* o Memory Manager para agentes e para times que acumulam contexto proprietário mais rápido do que conseguem organizá-lo manualmente. O harness de desenvolvimento do próprio engram pode dogfood o produto (futuro).

RFC 0001 (`docs/rfcs/0001-harness-memory-product-boundary.md`) já define o product boundary para Harness Memory.

Esta sprint implementa a **camada operacional** (o "harness engineering" process) que permite que agentes (e humanos) trabalhem de forma resumível, auditável e confiável sobre a mesma memória canônica.

## Trabalho em andamento (v0)

- [x] Estrutura de diretórios `docs/harness/{bin,progress,reviews,known-issues}`
- [x] `README.md` — guia operacional completo adaptado para engram/Rust/MCP/dual-CLI
- [x] `SPEC.md` — escopo da sprint v0
- [x] `INVARIANTS.md` — regras de processo invioláveis (session, commits, review, harness self-consistency, Rust/engram specifics)
- [x] `GATES.md` — 3 camadas, thresholds, fake-success patterns específicos de engram (embedding features, MCP, schema version, hooks, etc.)
- [x] `CODE_REVIEW_POLICY.md` — política injetada no reviewer externo, com adaptações para dual-CLI e domínios de engram
- [x] `progress/2026-05-30-harness-bootstrap.md` — log detalhado (este arquivo)
- [x] `bin/bootstrap.sh` — script de orientação (read-only, rápido, determinístico)
- [x] `bin/doctor.sh` — consistência do harness
- [x] `bin/sensors.sh` — wrapper sobre `just ci` + doctor + engram-specific
- [x] `bin/review-gate.sh` — generalizado para claude-sonnet/codex/etc, com prompt engineering, continuity, versioning, timeout
- [x] `bin/check-commit-msg.sh` — validador de commits
- [x] `bin/check-pr-title.sh` — validador de títulos de PR sem o marcador `[codex]`
- [x] `docs/harness/known-issues/2026-05-31-grpc-transport-port-bind.md` — limitação formal para sensor `grpc-transport`
- [x] Atualização de `AGENTS.md` + `Claude.md` para exigir bootstrap
- [x] Execução do loop completo nesta sprint + evidência de PASS
- [x] Integração leve com pre-commit / justfile (sem ruptura)

## Trilha de exclusão ativa

- `docs/harness/known-issues/2026-05-31-grpc-transport-port-bind.md` foi registrado para o sensor `grpc-transport`.
  - Exigir `sensors.sh --exclude-sensor grpc-transport --known-issue docs/harness/known-issues/2026-05-31-grpc-transport-port-bind.md --reason \"sandbox socket bind restriction\"`.
  - `pass_with_exclusion` é aceitável apenas com trilha completa e sem fechar produção (limpeza no ambiente sem exclusão necessária).

## Últimas decisões registradas

- Harness é **complementar** aos gates existentes (`just ci`, pre-commit, GitHub Actions). Não substitui — adiciona memória persistida, review cross-CLI, e disciplina de processo.
- Review-gate será flexível para o cenário atual (prompt files + paste no outro CLI) porque Claude Code CLI e Claude Code Sonnet reviewer são usados em processos/sessões separados.
- Dogfooding com o próprio engram (via MCP + hooks) é objetivo explícito de longo prazo, guiado por RFC 0001, mas fora do escopo de v0 bootstrap.
- Invariants do harness são separados dos data invariants (`INVARIANTS.md` na raiz) para manter clareza.
- 2026-06-08: `ENGRA-103` aberto no Huly para `memory_digest`; RFC 0008 define a ferramenta como digest read-only, determinístico, sem schema novo e com provenance explícita.
- 2026-06-20: títulos de PR criados ou editados por automação não podem conter o marcador `[codex]`; `check-pr-title.sh` e `doctor.sh` agora guardam essa regra.
- 2026-06-26: `NeoLabHQ/context-engineering-kit` fica como referência externa
  seletiva para padrões de workflow, não como dependência ou fonte copiável; a
  primeira adoção foi uma taxonomia local de perspectivas em
  `CODE_REVIEW_POLICY.md`.
- 2026-06-26: `docs/ieee-12207.md` fica como referência local de padrões de
  ciclo de vida sem reivindicação de conformidade; adoções futuras devem
  registrar tailoring, risco, medição, evidência e traceability no harness.
- 2026-06-26: a cópia integral da 12207 é local-only e ignorada pelo Git; o
  repositório deve versionar apenas resumos/checklists próprios do Engram.
- 2026-06-27: Claude Code Sonnet (`claude --model sonnet`) é o reviewer
  cross-model padrão permanente. Backends anteriores ficam apenas como histórico
  em logs/canvas/audits; outro backend requer override explícito do owner e
  verificação de assinatura/autenticação no momento do review.

## PR title guard — 2026-06-20

- Adicionado `docs/harness/bin/check-pr-title.sh` para validar títulos fornecidos por `--title` ou buscados com `--pr`.
- `doctor.sh` agora exige o script, valida executabilidade e testa tanto o caminho permitido quanto o caminho bloqueado.
- README, GATES e INVARIANTS documentam que PR titles devem descrever a mudança sem o marcador `[codex]`.
- Review Canvas: `docs/harness/canvas/2026-06-20-pr-title-guard.md`.
- Verificações:
  - `bash docs/harness/bin/check-pr-title.sh --title "align lifecycle hook contracts"` — PASS.
  - `bash -c 'if docs/harness/bin/check-pr-title.sh --title "[codex] align lifecycle hook contracts"; then exit 1; else exit 0; fi'` — PASS, o checker rejeitou o marcador.
  - `bash docs/harness/bin/check-pr-title.sh --pr 91` — PASS.
  - `bash -c 'if docs/harness/bin/check-pr-title.sh --pr --help; then exit 1; else exit 0; fi'` — PASS, o checker rejeitou identificador de PR não numérico antes de chamar `gh`.
  - `bash -c 'if docs/harness/bin/check-pr-title.sh --title "[codex] align lifecycle hook contracts" --help; then exit 1; else exit 0; fi'` — PASS, `--help` não ignora validação pendente.
  - `bash -c 'if docs/harness/bin/check-pr-title.sh --pr 91 --help; then exit 1; else exit 0; fi'` — PASS, `--help` não ignora validação de PR.
  - `bash -c 'if docs/harness/bin/check-pr-title.sh --title "[codex] align lifecycle hook contracts" --title "align lifecycle hook contracts"; then exit 1; else exit 0; fi'` — PASS, argumentos duplicados não sobrescrevem validação.
  - `bash -n docs/harness/bin/check-pr-title.sh docs/harness/bin/doctor.sh` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.
  - `bash docs/harness/bin/sensors.sh quick` — PASS (`cargo fmt --all -- --check`, `cargo check`, doctor).
  - `bash docs/harness/bin/sensors.sh` — PASS (`make ci + doctor`).

## Crate maintenance — 2026-06-12

- Corrigida a revisão dos crates:
  - `engram-wasm` entrou no workspace raiz e no gate local/GitHub CI.
  - `engram-core` passou a excluir artefatos internos do pacote publicado.
  - Advisories corrigíveis foram atualizados no `Cargo.lock` (`rustls-webpki`
    0.103.13, `tar` 0.4.46, `time` 0.3.47, `rand` 0.8.6,
    `aws-lc-rs` 1.17.0 / `aws-lc-sys` 0.41.0).
  - Dependências opcionais afetadas atualizadas: `libsql` 0.9.30,
    `notify` 8.2.0, `tokenizers` 0.23.1.
  - `cargo audit` e `cargo deny check` passam; ignores restantes são
    transitivos upstream-blocked (`rustls-webpki` via AWS/libsql,
    `quinn-proto`, e unmaintained transitivos sem safe upgrade).
- Verificações:
  - `cargo check --all-targets` — PASS.
  - `cargo check --all-targets --no-default-features --features turso,local-embeddings` — PASS.
  - `cargo audit` — PASS.
  - `cargo deny check` — PASS.
  - `bash scripts/ci.sh` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.

## Version control gate — 2026-06-12

- Corrigido `docs/harness/bin/vc-gate.sh` para não usar pipeline
  `git log | grep -q` sob `set -o pipefail`; quando o `grep -q` encontrava
  uma correspondência cedo, `git log` podia receber SIGPIPE e fazer o gate
  reportar falso negativo.
- `latest_git_mentions_issue` e `jj_current_mentions_issue` agora fazem match
  sobre strings capturadas, preservando o contrato de fechar issue apenas com
  evidência recente no Git ou descrição atual do jj.
- Evidência local:
  - `bash -n docs/harness/bin/vc-gate.sh` — PASS.
  - `bash docs/harness/bin/vc-gate.sh done ci` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.

## Code quality maintenance — 2026-06-16

- Aplicada fatia de alta confiança do relatório de qualidade:
  - Python SDK: `close()` agora é idempotente, zera `_client` após fechar e
    bloqueia `_mcp_call` em cliente fechado; `list()`/`search()` usam
    `filter_` no Python e enviam `filter` no payload MCP.
  - TypeScript SDK: request IDs JSON-RPC agora incrementam por cliente; opções
    públicas ganharam parity para `filter`, `mediaUrl`, workspaces e filtros
    de escopo suportados pela superfície MCP; testes foram reescritos para
    validar métodos públicos em vez de membros privados.
  - Cargo: removidos `anyhow`, `deadpool-sqlite`, `jsonrpc-core`,
    `levenshtein`, dev-deps não usadas, `wasm-bindgen-test` do crate WASM e o
    bin dummy `engram-core`/`src/main.rs`.
- `prost` foi mantido e documentado em `package.metadata.cargo-machete`
  porque é requerido por código gerado do recurso `grpc`.
- Fora de escopo deliberado: deleção dos módulos Rust de confiança média e
  deduplicações maiores; eles exigem revisão de compatibilidade pública.
- Review Canvas:
  `docs/harness/canvas/2026-06-16-code-quality-maintenance.md`.
- Post-review fechado com resposta independente em
  `docs/harness/reviews/2026-06-16-code-quality-maintenance-v2-post.md`;
  `REVIEW_VERDICT: PASS`.
- O reviewer registrou dois follow-ups `MED` nao bloqueantes: teste de regressao
  para `_mcp_call` apos `close()` no Python SDK e alinhamento do README do SDK
  Python com as novas opcoes publicas.

- Verificações:
  - `npm test` em `sdks/typescript` — PASS.
  - `npm run type-check` em `sdks/typescript` — PASS.
  - `uv run --with pytest-asyncio pytest` em `sdks/python` — PASS, 162 tests.
  - `cargo check --all-targets` — PASS.
  - `cargo check -p engram-core --features grpc --all-targets` — PASS.
  - `cargo check -p engram-wasm --all-targets` — PASS.
  - `cargo check -p engram-wasm --target wasm32-unknown-unknown` — PASS.
  - `cargo machete` — PASS.
  - `cargo fmt --all -- --check` — PASS.
  - `cargo clippy --all-targets --all-features -- -D warnings` — PASS.
  - `git diff --check` — PASS.
  - `make ci` — PASS.
  - `bash docs/harness/bin/sensors.sh` — PASS (`make ci + doctor`).
  - `bash docs/harness/bin/review-gate.sh post code-quality-maintenance --review-file docs/harness/reviews/2026-06-16-code-quality-maintenance-v2-post.md`
    — PASS.

## Storage extension semantics cleanup — 2026-06-20

- Storage extension placeholders now fail explicitly instead of returning
  success-shaped no-op data:
  - SQLite `CloudSyncBackend::push` / `pull`;
  - SQLite and Turso `TransactionalBackend::with_transaction`;
  - Turso `sync_delta` / `sync_state`.
- Savepoint names are validated as simple SQL identifiers before interpolation
  in SQLite and Turso backends.
- Review Canvas:
  `docs/harness/canvas/2026-06-20-storage-extension-semantics.md`.
- Verificações:
  - `rtk cargo test sqlite_backend` — PASS.
  - `rtk cargo test --test turso_backend_tests --features turso` — PASS.
  - `rtk cargo fmt --all -- --check` — PASS.
  - `rtk git diff --check` — PASS.
  - `rtk cargo clippy -p engram-core --all-targets --features turso -- -D warnings`
    — PASS.
  - `rtk cargo clippy --test turso_backend_tests --features turso -- -D warnings`
    — PASS.
  - `rtk bash docs/harness/bin/doctor.sh` — PASS.

## Hook contract cleanup — 2026-06-20

- `src/bin/server.rs::enable_hooks` now registers the exported `StopHandler`
  for `LifecycleHook::Stop`, preserving `HookResult::Continue`.
- `src/hooks/post_tool_use.rs` now documents and implements only best-effort
  memory policy reinforcement from explicit memory IDs; the misleading
  `auto_memory` field and unfinished fake auto-memory branch were removed.
- `CHANGELOG.md` now records the feature-gated public API cleanup; the hook
  contract remains covered by behavior tests instead of brittle source-text
  assertions.
- Review Canvas:
  `docs/harness/canvas/2026-06-20-hooks-contracts.md`.
- Verificações:
  - `rtk cargo fmt --all -- --check` — PASS.
  - `rtk git diff --check` — PASS.
  - `rtk cargo test --features hooks test_hook_wiring` — PASS.
  - `rtk cargo test --features hooks test_stop_handler` — PASS.
  - `rtk cargo test --features hooks test_post_tool_use_handler` — PASS.
  - `rtk cargo test --features hooks post_tool_use` — PASS.
  - `rtk cargo clippy -p engram-core --features hooks --all-targets -- -D warnings`
    — PASS.
  - `rtk bash docs/harness/bin/doctor.sh` — PASS.

## Enrichment audit subsecond replay fix — 2026-06-21

- `memory_replay_at_time` now compares replay cutoffs with SQLite
  `julianday(...)` instead of `datetime(...)`, preserving RFC3339 subsecond
  boundaries for both memory versions and enrichment events.
- Event replay ordering now uses `julianday(e.created_at) DESC, e.id DESC` so
  subsecond event timestamps sort consistently with the cutoff comparison.
- Added regression coverage for a replay timestamp between `.050Z` and `.900Z`
  to ensure the future version/event is excluded.
- Scope kept separate from the leftover harness/docs stash; this branch only
  restores `src/mcp/handlers/enrichment_audit.rs` plus this progress evidence.

Verification:

- `rtk bash docs/harness/bin/bootstrap.sh` — PASS.
- `rtk bash docs/harness/bin/doctor.sh` — PASS.
- `rtk bash docs/harness/bin/review-gate.sh pre enrichment-audit-subsecond-replay`
  — PASS/advisory prompt artifact generated.
- `rtk cargo fmt --all -- --check` — PASS.
- `rtk cargo test -p engram-core --lib test_memory_replay_at_time_preserves_subsecond_boundary --locked`
  — PASS.
- `rtk cargo test -p engram-core --lib memory_replay_at_time --locked` — PASS.
- `rtk cargo clippy -p engram-core --lib --locked -- -D warnings` — PASS.
- `rtk git diff --check` — PASS.
- `rtk bash docs/harness/bin/review-gate.sh post enrichment-audit-subsecond-replay --range origin/main..HEAD --review-file docs/harness/reviews/2026-06-21-enrichment-audit-subsecond-replay-v2-post.md`
  — PASS (`REVIEW_VERDICT: PASS`).
- GitHub PR checks — PASS for Format, Clippy, Documentation, and Test
  (ubuntu-latest).

## AI operating guide cleanup — 2026-06-21

- Recovered the docs-only `lazycodex-ai` operating guide from the preserved
  `fix/code-quality-pass` leftovers.
- `AGENTS.md` now points agents to `docs/AI_OPERATING_GUIDE.md` after the
  mandatory harness read order, without changing harness precedence.
- Scope intentionally excludes the already-merged `enrichment_audit` fix,
  generated `.sensors-last`, and AgentShield state evidence from the same
  stash.

Verification:

- `rtk bash docs/harness/bin/doctor.sh` — PASS.
- `rtk git diff --check` — PASS.

## Claude Sonnet reviewer path — 2026-06-27

- Claude Code Sonnet (`claude --model sonnet`) is now the permanent default
  cross-model reviewer in a separate process/session.
- `review-gate.sh`, `README.md`, `GATES.md`, `SPEC.md`, `INVARIANTS.md`,
  `CODE_REVIEW_POLICY.md`, `AGENTS.md`, and `CLAUDE.md` point future agents at
  Sonnet as the canonical reviewer path.
- Older reviewer-path sections below are historical only. Another backend
  requires explicit owner override plus authentication/subscription verification
  at review time.

## Reviewer CLI substitution — 2026-06-22

- Active cross-CLI review guidance now uses Gemini Flash 3.5 instead of Grok.
- `docs/harness/bin/review-gate.sh` documents `REVIEWER_CLI=gemini` and points
  pre/post gate handoff text at Gemini Flash 3.5.
- `docs/harness/README.md` described the reviewer pairing as Claude Code +
  Gemini Flash 3.5 in PR #104; the terminal Gemini example is superseded by the
  Zed Gemini reviewer path clarification below.
- Historical dated notes that mention Grok remain as historical records rather
  than being rewritten.
- Review Canvas:
  `docs/harness/canvas/2026-06-22-reviewer-cli-gemini-substitution.md`.

Verification:

- `rtk gemini --help` — PASS; local Gemini CLI is installed and supports
  non-interactive prompts with `-m/--model`.
- `rtk gemini -m gemini-3.5-flash "Return exactly OK"` — BLOCKED by local
  Google account licensing (`SUBSCRIPTION_REQUIRED`), so Gemini could not be
  used for this post-review run.
- `rtk bash -n docs/harness/bin/review-gate.sh` — PASS.
- `rtk git diff --check` — PASS.
- `rtk bash docs/harness/bin/doctor.sh` — PASS.
- `rtk bash docs/harness/bin/sensors.sh` — PASS, full gate
  (`make ci + pr-title-policy + harness doctor`).
- Post-review artifact:
  `docs/harness/reviews/2026-06-22-reviewer-cli-gemini-substitution-v3-post.md`;
  enforced with `review-gate.sh`, `REVIEW_VERDICT: PASS`.

## Zed Gemini reviewer path clarification — 2026-06-22

- Follow-up clarification: the intended Gemini reviewer is the **Gemini CLI**
  agent in Zed's agent picker, not the standalone terminal `gemini` binary.
- `docs/harness/bin/review-gate.sh` now points handoff instructions to Zed's
  Gemini CLI agent.
- `docs/harness/README.md` now documents Zed Gemini CLI as the canonical Gemini
  reviewer path and explicitly says not to treat the terminal `gemini` binary as
  canonical for this harness.
- Review Canvas:
  `docs/harness/canvas/2026-06-22-zed-gemini-reviewer-path.md`.

Verification:

- Computer Use `get_app_state` for `/Applications/Zed.app` — PASS; Zed is open
  on the Engram workspace and the agent UI is visible.
- Computer Use click attempts did not reliably open the Zed agent selector, so
  no review prompt was submitted through the UI from this Codex session.
- `rtk bash -n docs/harness/bin/review-gate.sh` — PASS.
- `rtk grep -n "gemini -m" docs/harness/README.md docs/harness/bin/review-gate.sh docs/harness/canvas/2026-06-22-zed-gemini-reviewer-path.md`
  — PASS; active README/script no longer contain the terminal Gemini example.
- `rtk git diff --check` — PASS.
- `rtk bash docs/harness/bin/doctor.sh` — PASS.
- `rtk bash docs/harness/bin/sensors.sh` — PASS, full gate
  (`make ci + pr-title-policy + harness doctor`).
- Post-review artifact:
  `docs/harness/reviews/2026-06-22-zed-gemini-reviewer-path-v2-post.md`;
  enforced with `review-gate.sh`, `REVIEW_VERDICT: PASS`.

## Próximos passos imediatos

1. Concluir Fase 1: manter mini-artifacts dos blocos 1.1–1.3 e audit report em `docs/harness/plans/`.
2. Entrar em Fase 2 (decisões/P1/P0): 28, 29, 26, 31, 32.
3. Preparar Fase 4 com base no contrato de `harness_record` + `harness_status`.

## AgentShield loop MVL — 2026-06-16

- Added the minimum viable loop components for a bounded AgentShield security
  scan:
  - Automation: `.github/workflows/agentshield-loop.yml` runs weekly and by
    manual dispatch.
  - Skill: `skills/agentshield-scan/SKILL.md`.
  - State: `docs/loops/agentshield-scan/STATE.md`.
  - Gate: `scripts/run-agentshield-loop.sh`, also exposed as `make
    loop-security` and `just loop-security`.
- Scope is static triage only: no automatic remediation, no production
  credentials, no auto-commit, and `LOOP_MAX_ITERATIONS` is capped at 5.
- The loop is optional and not part of required PR branch protection.

## Security reference harness adaptation — 2026-06-04

- Adicionada adaptação local do
  `anthropics/defending-code-reference-harness` em
  `docs/harness/security/anthropic-reference-harness.md`.
- Adicionados tuning files para scan/triage em `.claude/scan-extras.txt` e
  `.claude/fp-rules.txt`.
- Contrato adotado: static/read-only first; pipeline autônoma bloqueada por
  default até existir ADR, sandbox forte e target contract Rust.
- README e GATES do harness agora referenciam o fluxo de segurança.

## ENGRA-103 / RFC 0008 — `memory_digest` planning — 2026-06-08

- Criado contrato proposto para `memory_digest` em
  `docs/rfcs/0008-memory-digest.md`.
- Criado plano de implementação em
  `docs/harness/plans/2026-06-08-memory-digest-implementation-plan.md`.
- Criado Review Canvas em
  `docs/harness/canvas/2026-06-08-memory-digest.md`.
- Decisão de escopo:
  - v1 é ferramenta MCP read-only;
  - sem schema migration;
  - sem LLM obrigatório;
  - sem raw artifact retrieval;
  - sem mutação de memórias canônicas;
  - implementação deve orquestrar `memory_smart_retrieve`,
    `memory_build_context`, graph/crossrefs e `context_build_bundle`.
- Huly: `ENGRA-103` (`MCP memory_digest actionable retrieval digest`) criado
  com prioridade High.

## ENGRA-111 — deterministic MCP mock parity harness — 2026-06-09

- Added fixture-driven parity coverage for three public MCP shapes:
  `memory_create`/`memory_search`, `context_record`/`context_search`, and the
  unknown-tool error envelope.
- The fixture stores stable normalized expectations only; generated memory IDs,
  timestamps, scores, and UUID-like values stay out of the contract.
- SDK expansion note lives with the fixture so Python and TypeScript parity can
  reuse the same scenario names and expected normalized output later.
- Review Canvas: `docs/harness/canvas/2026-06-09-engra-111-mock-parity.md`.

## Nota da sessão — 2026-05-31

- Compressão: `ContextCompressor` agora expõe diagnósticos de orçamento/skips,
  e `memory_compress_for_context` retorna metadados de orçamento e memórias
  ignoradas.
- Compressão semântica: dedupe preserva endpoints técnicos distintos e normaliza
  pontuação final em tokens técnicos sem perder paths, IDs ou versões.
- Evidência de benchmark: `docs/harness/reviews/2026-05-31-compression-benchmark-ratio-recall.md`.
- Verificações desta sessão:
  - `bash docs/harness/bin/doctor.sh` — PASS.
  - `bash docs/harness/bin/sensors.sh` — PASS limpo, sem exclusão
    (`2026-05-31T15:52:49Z`).
  - `cargo test test_technical_tokens_normalize_trailing_punctuation` — PASS.
  - `cargo test test_deduplication_preserves_distinct_technical_endpoints` — PASS.
  - `cargo test --test v070_integration_tests test_memory_compress_for_context_reports_skip_metadata` — PASS.
  - `bash -n docs/harness/bin/review-gate.sh` — PASS.
  - `bash -n docs/harness/bin/check-commit-msg.sh` — PASS.
  - `bash docs/harness/bin/check-commit-msg.sh --message 'fix(harness): harden review gates'` — PASS.

## Review fixes — 2026-05-31

- `docs/harness/bin/review-gate.sh` agora usa `set -euo pipefail` e corrige a
  grafia de `REVIEWER_CLI=grok` no comentário operacional.
- `CLAUDE.md` agora lista caminhos explícitos para todos os docs obrigatórios
  do harness.
- `tests/v070_integration_tests.rs` cobre o contrato MCP de
  `memory_compress_for_context` para metadados de skip/orçamento.
- Review manual subsequente:
  - dedupe semântico preserva fatos técnicos em relação de superset;
  - `memory_compress_for_context` aceita `memory_ids` além de `ids`;
  - benchmark `fixed_corpus_ratio_recall` usa piso fixo explícito de 7/9.

## Phase 0 post-review — 2026-05-31

- Artefato manual criado por instrução explícita do usuário:
  `docs/harness/reviews/2026-05-31-harness-bootstrap-v9-post.md`.
- `bash docs/harness/bin/review-gate.sh post harness-bootstrap --review-file docs/harness/reviews/2026-05-31-harness-bootstrap-v9-post.md`
  retornou `POST-GATE PASS`.
- Observação: o artefato é `Codex/manual`, não cross-CLI independente.

## ENG-1295 / #26 — passo 3

- `maintenance-status` humano agora imprime seção `Derived indexes:` com
  `embeddings`, `memories_fts` e `crossrefs`.
- A renderização foi extraída para `write_maintenance_status<W: Write>` para
  permitir teste sem captura global de stdout.
- Novo teste:
  `maintenance_status_human_output_includes_derived_indexes`.
- Verificações:
  - `cargo fmt --all -- --check` — PASS.
  - `cargo test maintenance_status` — PASS.
  - `cargo test test_health_check_reports_` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.

## ENG-1296 / #32 — harden queue and operational hygiene surface

- Auditoria de superfície de fila de embeddings aplicada em health + status operacional:
  - Incluídos em `derived_indexes[embeddings].details`: `pending`, `processing`,
    `stale_processing`, `failed`, `retryable_failed`, `exhausted_failed`,
    `max_retry_count`, `oldest_pending_age` (+ alias
    `oldest_pending_age_seconds`).
  - Saída humana de `maintenance-status` agora exibe linha `queue-state` com esses contadores (com fallback estável para chave legada `oldest_pending_age_seconds`).
- Testes/regressões validados:
  - `cargo test maintenance_status_ -- --nocapture` — PASS.
  - `cargo test test_health_check_embedding_details_include_queue_state_counters -- --nocapture` — PASS.
  - `cargo test embedding_queue_health_counts_stale_and_retries -- --nocapture` — PASS.
  - `cargo clippy --all-targets --tests -- -D warnings` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.

## ENG-1296 / #25 — Add embedding queue hygiene policy

- Formalização da política de operação da fila: budget de retry e retenção para `complete` (via constants config in `src/embedding/queue.rs`).
- `maintenance queue-hygiene` adicionado ao CLI com fluxo explícito de reparo:
  - `--dry-run` (padrão) para não mutar
  - `--apply` para executar mudanças
  - `--requeue-failed` para incluir requeue de `failed` com orçamento restante
- Expansão de saúde: `zero_retry_failed`, idades `oldest_processing_age` /
  `oldest_failed_age`, e buckets de `retry_count`.
- Ajuste adicional de observabilidade: buckets `retry_count_0/1/2/3+` passaram a
  usar semântica fixa (`3+` para `>=3`) independente do `max_retries` configurado,
  e passaram a aparecer também na saída humana de `maintenance-status`.
- Estado de saúde e saída humana de `maintenance-status` continuam read-only.
- Testes adicionados/estendidos em Rust para cobertura de:
  - `health` (contadores e idades)
  - path de hygiene em modo dry-run vs apply
  - output de status e comportamento não mutável por padrão.
- Verificações na iteração:
  - `cargo fmt --all -- --check` — PASS.
  - `cargo clippy --all-targets --tests -- -D warnings` — PASS.
  - `cargo test maintenance_status_ -- --nocapture` — PASS.
  - `cargo test test_health_check_embedding_details_include_queue_state_counters -- --nocapture` — PASS.
  - `cargo test test_embedding_queue_health_counts_stale_and_retries -- --nocapture` — PASS.
  - `cargo test test_embedding_queue_hygiene_dry_run_does_not_mutate_and_apply_can_repair -- --nocapture` — PASS.
  - `cargo test maintenance_queue_hygiene_dry_run_does_not_mutate_and_apply_updates -- --nocapture` — PASS.
- Decisão de escopo:
  - `CHEATSHEET_CUTOVER.md` permanece fora do commit; é checklist operacional
    de deploy/cutover e não faz parte da política de higiene da fila.

## ENG-1296 / #26 — Define derived index health contract for external backends

- Padronizado `DerivedIndexHealth` para backends externos:
  - adicionado construtor `DerivedIndexHealth::external(...)` em `backend.rs`;
  - `meilisearch` e `turso` passam a preencher `derived_indexes` com entrada `kind=external`
    (com contadores base e status explícito), em vez de retornar vetor vazio.
- Contrato em `docs/SCHEMA.md` ajustado para exigir representação consistente de
  derived indexes também em backends sem analítica interna por índice.
- Validação:
  - `cargo test test_turso_health_check --test turso_backend_tests --features turso -- --nocapture` — PASS.
  - `cargo clippy --all-targets --tests -- -D warnings` — PASS.
  - `cargo check --tests --features meilisearch` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.

## ENG-1296 / #27 — Generate MCP tools reference from code (partial)

- Documentação passou a usar a referência gerada como fonte principal:
  - `README.md` (`Available MCP Tools`) agora aponta para `docs/MCP_TOOLS.md` e para o gerador.
  - `docs/AI_GUIDE.md` remove contagens manuais e passa a referenciar `docs/MCP_TOOLS.md` como origem.
- Validação:
  - `./scripts/generate-mcp-reference.sh --check` — PASS.
- O fechamento de #27 ainda depende de eliminar/normalizar quaisquer contagens ou listagens manuais remanescentes fora desse escopo.

## Attestation CRITICAL security fixes — 2026-06-01

- `src/attestation/chain.rs` agora lê o tip da cadeia e insere o novo registro
  dentro do mesmo `with_transaction`, removendo a janela TOCTOU entre SELECT e
  INSERT.
- `verify_chain` passou a aceitar `Option<&[u8; 32]>` para verificar assinaturas
  Ed25519; `None` preserva o comportamento legado, e `Some(key)` exige assinatura
  válida em todos os registros verificados.
- Testes adicionados cobrem append concorrente, assinatura válida, assinatura
  adulterada e assinatura removida.
- Verificações:
  - `cargo test --features agent-portability test_verify_chain` — PASS.
  - `cargo test --features agent-portability test_chain_stays_linear` — PASS.
  - `cargo test --features agent-portability attestation` — PASS.
  - `cargo test --features agent-portability scenario_5_chain_verify_valid` — PASS.
  - `cargo test` — PASS.
  - `cargo fmt --all -- --check` — BLOCKED por formatting drift existente
    fora do diff (`compression_semantic.rs`, `token_counter.rs`, handlers MCP).
  - `bash docs/harness/bin/doctor.sh` — PASS.
  - `cargo clippy --all-targets --all-features -- -D warnings` — BLOCKED por
    warnings existentes fora do diff (`token_counter.rs`, `harness.rs`,
    `markdown_export.rs`).

## Council workflow skill / `memory_council` — 2026-06-02

- Fluxo reutilizavel de consensus/council adicionado:
  - ferramenta MCP `memory_council`;
  - handler `src/mcp/handlers/council.rs`;
  - wrappers Python/TypeScript `CouncilSkill`;
  - skill instalavel `skills/engram-council/SKILL.md`;

- Integração de fluxo leve com gates locais concluída:
  - `.githooks/pre-commit` agora usa `just pre-commit` quando disponível e cai
    para os comandos diretos quando `just` não estiver instalado.
  - `justfile` ganhou a receita `pre-commit` como fonte única para o hook.
  - `tests/mcp_protocol_tests.rs` recebeu cobertura de round-trip do tool
    `memory_council` via `tools/call`.
  - documentacao em README, AI guide, guia de uso em repos, SDK READMEs,
    changelog e referencia MCP gerada.
- Ajustes de revisao aplicados:
  - `skills/engram-council/SKILL.md` reestruturada como playbook operacional
    para agentes, com regras de uso, checklist, template de prompt, argumentos
    MCP, interpretacao de resultado e handling de falhas;
  - truncamento de erro no handler agora e seguro para UTF-8;
  - textos publicos distinguem `engram-council` (skill instalavel) de
    `llm-council` (backend de orquestracao);
  - README TypeScript nao depende mais de trailing whitespace para quebra de
    linha.
- Verificacoes:
  - `bash docs/harness/bin/bootstrap.sh` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.
  - `cargo fmt --all -- --check` — PASS.
  - `cargo test council -- --nocapture` — PASS.
  - `git diff --check` — PASS.
- Validacao de skill:
  - `python3 .../skill-creator/scripts/quick_validate.py skills/engram-council`
    bloqueado porque `PyYAML` nao esta instalado no Python global.
  - `rg -n '[[:blank:]]+$' skills/engram-council/SKILL.md` — PASS sem
    trailing whitespace.
  - `LC_ALL=C rg -n "[^ -~]" skills/engram-council/SKILL.md` — PASS sem
    caracteres nao ASCII.
- Limitacoes locais:
  - `pytest sdks/python/tests/test_client.py -k council` bloqueado porque o
    ambiente global nao tem `pytest-asyncio` ativo.
  - `npm run type-check` bloqueado porque `tsc` nao esta instalado no SDK
    TypeScript local.

## ENG-1241 — MCP HTTP auth contract and client docs

- Diagnostico atualizado: o HTTP transport ja aplicava Bearer auth em
  `POST /mcp` e `GET /v1/events` quando `ENGRAM_HTTP_API_KEY`/`--http-api-key`
  estava configurado; o gap real era contrato publico inconsistente.
- `src/mcp/http_transport.rs` agora constroi o router em helper testavel e
  aceita `POST /v1/mcp` como alias compativel de `POST /mcp`, com o mesmo
  contrato de auth.
- Cobertura adicionada para:
  - `POST /mcp` rejeitar request sem Bearer quando API key configurada;
  - `POST /mcp` aceitar Bearer correto;
  - `POST /v1/mcp` usar o mesmo contrato de auth.
- Docs alinhadas:
  - novo `docs/MCP_AUTH.md` para clientes externos;
  - README, AI guide, getting started e guia de uso em repos atualizados para
    flags reais (`--transport http --http-port`, `ENGRAM_HTTP_API_KEY`) e MCP
    JSON-RPC em vez de REST local antigo.
- Validacoes:
  - `cargo test http_transport --lib` — PASS.
  - `cargo fmt --all -- --check` — PASS.
  - `cargo clippy --lib -- -D warnings` — PASS.
  - `cargo clippy --lib --tests -- -D warnings` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.
  - `git diff --check` — PASS.
  - `bash docs/harness/bin/review-gate.sh post eng-1241-mcp-http-auth-docs`
    gerou prompt de post-review em
    `docs/harness/reviews/2026-06-03-eng-1241-mcp-http-auth-docs-v2-post.md.raw`;
    sem verdict porque falta reviewer externo.
  - Escopo deliberadamente fora desta iteracao: rate-limit MCP, metricas/tracing
    especificas de transport, e execucao/verificacao de deploy Fly.io.

## ENG-1241 follow-ups — MCP HTTP follow-through

- **ENGRA-58 (implementado):** Rate limit para MCP HTTP (`/mcp` e `/v1/mcp`) via token-bucket, com chaves por IP/header, teto de buckets e stale cleanup.
- **ENGRA-59 (implementado):** observabilidade do transporte (métricas, tracing e `GET /health` com estado de proteção).
- **ENGRA-60 (implementado):** rollout/documentação de validação de deploy (Fly.io) para auth + rate limit.

## ENGRA-84 — MCP HTTP rate-limit hardening

- Auth agora é avaliado antes do rate limit no `POST /mcp` e `POST /v1/mcp`;
  requests sem Bearer válido continuam retornando `401` e não consomem bucket.
- Regressões adicionadas para interação auth/rate-limit, fallback por
  `x-real-ip`, cleanup de buckets stale e eviction sob pressão de
  `max_buckets`.

## Security fixes — 2026-06-04

- **Merkle `hash_pair` length-separation** (OBS. 9133):
  `hash_pair` alterado de `left || right` para
  `len(left) || left || len(right) || right` para eliminar colisao de
  segunda pre-imagem. Backwards compat mantida via `scheme_version: u8`
  no `MerkleProof` (v1 = concat. naive; v2 = length-sep).
  Novas provas usam v2; provas v1 pre-existentes continuam verificaveis.

- **Attestation signature verification** (OBS. 9132):
  MCP handler `attestation_chain_verify` agora aceita `verifying_key`
  opcional (hex Ed25519). Quando fornecido, valida assinatura de todo
  registro. Schema MCP atualizado em `tools.rs` para expor o parametro.

- **Dedup normalization regression** (ACHADO AUDITORIA):
  `create_memory` e `update_memory` agora usam `compute_dedup_hash`
  (normalizado) na coluna `content_hash`. Backends alternativos
  (`turso_backend`, `meilisearch_backend`) sincronizados.
  `compute_content_hash_raw(cfg(test))` existe para byte-exact checks.
  Markdown import detecta edits case-only como `PendingUpdate` (teste
  `test_import_in_sync_when_body_normalized_matches`).

  Verificacoes:
  - `cargo test --lib` — 988 PASS.
  - `cargo test --features agent-portability --lib` — 1,063 PASS.
  - `cargo clippy --all-targets --all-features` — clean.

---

**Nota**: Este arquivo e atualizado manualmente ao final de cada iteracao significativa ou ao final de sessoes. O log detalhado fica no arquivo apontado por `Active plan`.

## Harness cross-improvements — 2026-06-05

- Plano Engram-only executado a partir de `docs/harness/plans/2026-06-05-engram-harness-improvement-execution-plan.md`.
- Criado `docs/harness/WHAT_WE_DONT_DO.md` para escopo negativo do harness.
- Criado `docs/harness/canvas/` com README e template de Review Canvas para mudanças complexas.
- Criados `docs/harness/bin/baseline.sh` e `docs/harness/bin/quarterly-audit.sh` como evidência de drift/auditoria, sem substituir sensores ou CI.
- `sensors.sh` ganhou modos opcionais `full`, `quick`, `docs`, `mcp` e `baseline`; o modo sem argumentos permanece o gate completo canônico.
- `review-gate.sh` passou a incluir `WHAT_WE_DONT_DO.md`, Review Canvas e guard para mudanças em `docs/harness/bin/*`.
- `doctor.sh` agora valida a nova política, canvas, baseline, audit, sensor lanes e referências cruzadas.

### Verificações planejadas nesta execução

- `bash docs/harness/bin/doctor.sh`.
- `bash docs/harness/bin/sensors.sh baseline`.
- `bash -n docs/harness/bin/bootstrap.sh docs/harness/bin/doctor.sh docs/harness/bin/sensors.sh docs/harness/bin/review-gate.sh docs/harness/bin/baseline.sh docs/harness/bin/quarterly-audit.sh`.

## Huly tracking — 2026-06-05

- Issues criados no Huly para execução do plano:
  - ENGRA-78 — negative-scope policy.
  - ENGRA-79 — Review Canvas.
  - ENGRA-80 — harness script review guard.
  - ENGRA-81 — baseline snapshot.
  - ENGRA-82 — optional sensor lanes.
  - ENGRA-83 — evidence-only quarterly audit.

### Verificações executadas — 2026-06-05 harness cross-improvements

- `bash docs/harness/bin/doctor.sh` — PASS.
- `bash docs/harness/bin/sensors.sh baseline` — PASS; gravou `docs/harness/.baseline-last`.
- `bash -n docs/harness/bin/bootstrap.sh docs/harness/bin/doctor.sh docs/harness/bin/sensors.sh docs/harness/bin/review-gate.sh docs/harness/bin/baseline.sh docs/harness/bin/quarterly-audit.sh` — PASS.
- `bash docs/harness/bin/quarterly-audit.sh` — PASS; gravou `docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md` e `docs/harness/.quarterly-audit-last`.
- `bash docs/harness/bin/doctor.sh` final — PASS.

Limite deliberado: o full `bash docs/harness/bin/sensors.sh` nao foi executado nesta iteracao; a validacao executada foi a lane `baseline` especifica do plano de melhoria do harness.

## Version-control gate / jj adoption — 2026-06-05

- Adicionado `docs/harness/bin/vc-gate.sh` como gate opcional de fronteira de issue e gate recomendado antes de release.
- Contrato adotado:
  - `jj` pode ser usado como camada local para evoluir, splitar e descrever work-in-progress;
  - Git continua canônico para release commits, tags e `cargo publish`;
  - o gate nao cria commits, nao roda `jj new`, nao move tags e nao publica.
- Modos documentados: `status`, `start ISSUE`, `done ISSUE`, `release VERSION`.
- Criado Review Canvas em `docs/harness/canvas/2026-06-05-jj-version-control-gate.md` porque a mudanca toca scripts/policy do harness.
- Validação nao executada nesta iteração por instrução operacional atual de nao rodar verificações sem pedido explícito.

## vc-gate release guard review fix — 2026-06-06

- Post-review for `memory-policy-layer` found `vc-gate.sh release` could print `release_gate=pass` without a release version.
- Fixed `docs/harness/bin/vc-gate.sh` so release mode requires `VERSION` or `vVERSION`.
- Targeted validation:
  - `bash -n docs/harness/bin/vc-gate.sh` — PASS.
  - `bash docs/harness/bin/vc-gate.sh release --allow-dirty` — expected FAIL with `release requires VERSION or vVERSION`.
  - `bash docs/harness/bin/doctor.sh` — PASS.

## Memory policy layer Phase 1 — 2026-06-06

- Added deterministic `heuristic-v1` memory policy scoring with durable `memory_policy` records.
- Added MCP tools for score, promote, decay, explain, and conflict reconciliation.
- Integrated optional retrieval-time policy reranking through `policy_rerank` without changing default search behavior.
- Preserved SQLite/FTS/vector/graph/provenance as canonical state; no automatic synthesized memory writes were added.
- Validation passed: focused checks, `make ci`, `doctor.sh`, and post-review gate.
- Post-review artifact: `docs/harness/reviews/2026-06-06-memory-policy-layer-v2-post.md` with `REVIEW_VERDICT: PASS`.

## Root folder organization — 2026-06-06

- Moved cloud/API reference docs from the repository root into `docs/`:
  `ARCHITECTURE.md`, `OPERATIONS.md`, `QUICKSTART.md`, `REFERENCE.md`, and
  `CONTROL_PLANE_SCHEMA.sql`.
- Kept root reserved for repository identity, build/config entrypoints,
  GitHub-visible policy files, and agent context files consumed by tooling.
- Updated known links in `AGENTS.md`, `CLAUDE.md`, `docs/QUICKSTART.md`, and
  `docs/README.md`.
- No `src/`, runtime schema, MCP surface, hooks, SDK, or harness gate behavior
  changes are included in this branch.

## Security reference harness enforcement — 2026-06-06

- Added `docs/harness/security/anthropic-reference-harness.md` as the canonical
  local contract for `ENGRAM-HARNESS-SECURITY-CONTRACT-v1`.
- Added versioned tuning files `.claude/scan-extras.txt` and
  `.claude/fp-rules.txt`; they augment scan/triage behavior and do not replace
  core invariants, gates, or review policy.
- Updated `doctor.sh` to fail closed when the security note, contract anchors,
  tuning files, or required cross-references are missing.
- Updated `bootstrap.sh`, `sensors.sh`, and `review-gate.sh` to surface the
  security boundary without adding autonomous execution to the default harness
  flow.
- Updated harness docs and onboarding docs so reviewers flag autonomous Engram
  execution, sandbox drift, credential mounts, egress expansion, and C/C++/ASAN
  pipeline import unless an ADR and explicit target contract exist.
- No autonomous execution pipeline, target runner, credential mount, or `src/`
  change is included in this branch.

## CI superseded-run cancellation + ENGRA-92 — 2026-06-07

- Added GitHub Actions `concurrency` for CI runs keyed by
  workflow/event/ref, so superseded pushes on `main` cancel older in-progress
  runs instead of leaving obsolete extended jobs consuming runners.
- Preserved the required PR gate shape: `fmt`, `clippy`, `Test
  (ubuntu-latest)`, and `Documentation`.
- Confirmed `Full Feature Tests` remains schedule/manual only for new pushes.
- Fixed `ENGRA-92` by making `.claude/scan-extras.txt` and
  `.claude/fp-rules.txt` identify themselves with the literal filenames that
  `doctor.sh` validates.
- Verification:
  - `git diff --check` — PASS.
  - YAML parse of `.github/workflows/ci.yml` — PASS.
  - `bash docs/harness/bin/doctor.sh` — PASS.

## Huly backlog audit + ENGRA-74 — 2026-06-07

- Used the local Huly skill and Platform API token flow to read project `ENGRA`.
- Huly returned 22 `Backlog` issues; most were stale against the current repo
  state (rate-limit/observability/deploy docs, harness cross-improvements, and
  Operational Context foundations already exist locally).
- Implemented the remaining visible code gap for **ENGRA-74**:
  `context_get_artifact` now exposes explicit retained artifact retrieval over
  MCP with `artifact_id`, `reason`, scope fields, `max_bytes`, staleness, and
  redaction checks.
- Updated `docs/MCP_TOOLS.md` through the generator.
- Focused validation:
  - `cargo test context_get_artifact --test mcp_protocol_tests -- --nocapture`
    — PASS.
  - `./scripts/generate-mcp-reference.sh --check` — PASS.
  - `cargo clippy --all-targets --tests -- -D warnings` — PASS.
  - `make ci` — PASS.

## ENGRA-103 `memory_digest` MCP implementation — 2026-06-08

- Added read-only MCP tool `memory_digest` as the actionable retrieval entry
  point defined by RFC 0008.
- The v1 implementation is a thin orchestrator over existing primitives:
  `memory_smart_retrieve`, `memory_build_context`, `crossrefs`, and
  `context_build_bundle`.
- Preserved the product boundary: no schema migration, no LLM call, no memory
  mutation, no Dream candidate application, and no raw artifact content return.
- Response includes extractive summary/key points, top memory previews with
  IDs, relationships, Operational Context sections, next actions, provenance,
  and warnings.
- Updated MCP registry/dispatch and regenerated `docs/MCP_TOOLS.md`
  (`Total tools: 277`).
- Added protocol coverage for tools/list read-only metadata, successful
  dispatch with source relationships, input validation, and empty-source
  warnings.
- Validation passed: focused `memory_digest` protocol tests, MCP reference
  check, fmt, clippy, doctor, and full `sensors.sh` (`make ci` + doctor).

## AgentShield harness parity B1 — PR title policy gate — 2026-06-21

- Ported AgentShield's `docs/harness/bin/pr-title-policy.sh` into Engram.
- Wired deterministic PR title policy checks into `sensors.sh` so clean titles
  pass and `[codex]` / `[ CoDeX ]` titles fail with exit 4.
- Updated `doctor.sh` to require the policy script, executable bit, GATES
  documentation, and sensors wiring.
- Added Review Canvas:
  `docs/harness/canvas/2026-06-21-b1-pr-title-policy.md`.

Verification:

- `bash -n docs/harness/bin/pr-title-policy.sh` — PASS.
- `bash -n docs/harness/bin/doctor.sh` — PASS.
- `bash -n docs/harness/bin/sensors.sh` — PASS.
- `bash docs/harness/bin/pr-title-policy.sh --title "fix: clean title"` —
  PASS (`OK: PR title policy`).
- `bash docs/harness/bin/pr-title-policy.sh --title "[codex] fix: bad title"`
  — expected exit 4.
- `bash docs/harness/bin/pr-title-policy.sh --title "[ CoDeX ] fix: bad title"`
  — expected exit 4.
- `printf "%s" "feat: clean" | bash docs/harness/bin/pr-title-policy.sh --stdin`
  — PASS.
- `bash docs/harness/bin/doctor.sh` — PASS.
- `bash docs/harness/bin/doctor.sh --json` parsed as `status=pass` — PASS.
- `bash docs/harness/bin/sensors.sh quick` — PASS; includes PR title policy
  positive and negative checks.
- `bash docs/harness/bin/check-commit-msg.sh --message "chore(harness): add PR title policy gate"`
  — PASS.

## Final broad review fixes — PR title policy alignment — 2026-06-21

- Final broad review found that `check-pr-title.sh` and `pr-title-policy.sh`
  enforced overlapping PR title policies with divergent exit contracts.
- Made `pr-title-policy.sh` the canonical implementation and changed
  `check-pr-title.sh` into a compatibility wrapper that delegates to it.
- Updated `doctor.sh` to require and exercise the canonical script directly,
  including exact exit-code `4` checks for `[codex]` and `[ CoDeX ]`.
- Updated `GATES.md` and `README.md` so the canonical script and wrapper roles
  are explicit.
- Added Review Canvas:
  `docs/harness/canvas/2026-06-21-final-engram-broad-review-fixes.md`.

Verification:

- `bash -n docs/harness/bin/check-pr-title.sh` — PASS.
- `bash -n docs/harness/bin/pr-title-policy.sh` — PASS.
- `bash -n docs/harness/bin/doctor.sh` — PASS.
- `bash docs/harness/bin/pr-title-policy.sh --title "align lifecycle hook contracts"`
  — PASS.
- `bash docs/harness/bin/pr-title-policy.sh --title "[codex] align lifecycle hook contracts"`
  — expected exit 4.
- `bash docs/harness/bin/pr-title-policy.sh --title "[ CoDeX ] align lifecycle hook contracts"`
  — expected exit 4.
- `bash docs/harness/bin/check-pr-title.sh --title "align lifecycle hook contracts"`
  — PASS.
- `bash docs/harness/bin/check-pr-title.sh --title "[codex] align lifecycle hook contracts"`
  — expected exit 4.
- `bash docs/harness/bin/check-pr-title.sh --title "[ CoDeX ] align lifecycle hook contracts"`
  — expected exit 4.
- `bash docs/harness/bin/doctor.sh` — PASS.
- `bash docs/harness/bin/sensors.sh quick` — PASS.
- `PR_TITLE="[codex] env title" bash docs/harness/bin/pr-title-policy.sh --title "align lifecycle hook contracts"`
  — PASS; explicit `--title` is not made ambiguous by `PR_TITLE`.
- `bash docs/harness/bin/sensors.sh full` — PASS; full deterministic gate
  green for final broad-review range.
- `bash docs/harness/bin/review-gate.sh post final-harness-parity --range 2e932ad..HEAD`
  — initial FAIL found PR-title policy divergence and missing canonical doctor
  validation; fixed in `eb4c175`.
- Final review rerun `v2` — FAIL because the range needed full deterministic
  gate evidence; fixed by running `sensors.sh full` and recording
  `.sensors-last` in `43506c7`.
- Final review rerun `v3` — PASS; official `review-gate.sh post` accepted the
  `REVIEW_VERDICT: PASS` artifact.

## AgentShield harness parity B2 — loop-engineering skill + SKILLS.md policy — 2026-06-21

- Ported AgentShield's `loop-engineering` base skill to
  `skills/loop-engineering/SKILL.md`, adapting product references to Engram
  MCP/memory loops while preserving the L1/L2/L3 safety model.
- Added `docs/harness/SKILLS.md` with the current Engram skill inventory,
  canvas-gated promotion policy, and follow-up list for the other AgentShield
  loop skills that are intentionally out of B2 scope.
- Wired conservative skill validation into `doctor.sh`: `SKILLS.md` is required,
  `README.md` cross-links it, each tracked `skills/*/SKILL.md` must have matching
  frontmatter `name`, a `description`, and membership in the current-skills
  inventory.
- Linked `SKILLS.md` from `docs/harness/README.md`.
- Added Review Canvas:
  `docs/harness/canvas/2026-06-21-b2-loop-skills-policy.md`.

Verification:

- `bash -n docs/harness/bin/doctor.sh` — PASS.
- `bash docs/harness/bin/doctor.sh` — PASS.
- `bash docs/harness/bin/sensors.sh` — PASS in the original B2 run.
- `bash docs/harness/bin/check-commit-msg.sh --message "docs(harness): add SKILLS.md policy and loop-engineering skill"`
  — PASS.
- Independent cross-model review iterated through the B2 findings and finished
  with `REVIEW_VERDICT: PASS`; the review artifacts are committed under
  `docs/harness/reviews/2026-06-21-b2-loop-skills-policy*-post.md`.

## AgentShield harness parity follow-up — loop triage skill family — 2026-06-21

- Promoted the four B2 follow-up skills from AgentShield into Engram:
  `loop-triage`, `loop-triage-ci`, `dependency-triage`, and
  `pr-review-triage`.
- Adapted content to Engram's loop paths, full harness gate, MCP/connectors,
  storage, attestation, protocol, and release-risk boundaries.
- Updated `docs/harness/SKILLS.md` so the four skills move from follow-up
  candidates into the Current Skills inventory.
- Added Review Canvas:
  `docs/harness/canvas/2026-06-21-loop-followup-skills.md`.

Verification:

- Skill frontmatter validation via `quick_validate.py` in a temporary
  `/tmp/engram-skill-validate` venv — PASS for all four promoted skills.
- `bash docs/harness/bin/doctor.sh` — PASS.
- `bash docs/harness/bin/sensors.sh quick` — PASS.
- `bash docs/harness/bin/sensors.sh` — PASS (full canonical gate, `make ci`
  + PR-title policy + harness doctor).

## Lifecycle predicate unification — 2026-06-27

- Added `src/intelligence/lifecycle.rs` with the canonical
  `LifecycleConfig`, `normalized_importance`, and `decide_lifecycle_state`
  predicate. The predicate uses `last_accessed_at` with `created_at` fallback,
  importance as a bounded multiplier, a hard idle cap, and monotonic stale /
  archived behavior.
- Reworked `lifecycle_run` to use one permissive SQL candidate query plus the
  canonical predicate for dry-run and apply parity. `min_importance` remains
  accepted as deprecated/no-op compatibility only.
- Made salience decay score/history-only and routed `SalienceScore.suggested_state`
  through the canonical lifecycle predicate.
- Made `memory_decay` policy-score-only; lifecycle transitions now remain in
  `lifecycle_run`.
- Constrained `memory_archive_old` and `compress_old_memories` to compress only
  already-Archived memories. The optional server compression scheduler now logs
  compression rather than archival.
- Updated public MCP tool metadata and regenerated `docs/MCP_TOOLS.md`.
- `SCHEMA_VERSION` remains 44; no migration was added.

Verification:

- `rtk cargo test intelligence::lifecycle --lib` — PASS.
- `rtk cargo test mcp::handlers::lifecycle --lib` — PASS.
- `rtk cargo test intelligence::salience --lib` — PASS.
- `rtk cargo test mcp::handlers::quality --lib` — PASS.
- `rtk cargo test mcp::handlers::memory_policy --lib` — PASS.
- `rtk cargo test mcp::handlers::summarize --lib` — PASS, including
  idempotent compression coverage.
- `rtk cargo test retention --lib` — PASS, including idempotent compression
  coverage.
- `rtk cargo test storage::queries --lib` — PASS.
- `rtk cargo test --test mcp_protocol_tests` — PASS.
- `rtk cargo check --workspace --all-targets --locked` — PASS.
- `rtk cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
  — PASS.
- `rtk cargo test --workspace --all-targets --locked` — PASS, 1227 tests.
- `rtk ./scripts/generate-mcp-reference.sh --check` — PASS.
- `rtk bash docs/harness/bin/doctor.sh` — PASS.
- `rtk git diff --check` — PASS.
- `rtk grep -n "SET lifecycle_state|UPDATE memories SET lifecycle_state|update_memory_lifecycle_state\\(" src`
  — PASS; remaining matches are canonical lifecycle/manual/domain/helper/test
  write sites only.
- `rtk bash docs/harness/bin/sensors.sh` — PASS (full canonical gate, `make ci`
  + PR-title policy + harness doctor).
- Post-review v1/v2 FAIL findings were addressed by adding the missing
  plan/spec/review/canvas evidence chain, correcting README/ROADMAP wording,
  and making compression paths idempotent.
- `bash docs/harness/bin/check-commit-msg.sh --message "docs(harness): port loop triage skill family"` — PASS.
- Cross-repo leakage grep for `AgentShield`, `cargo run -- scan`,
  `release.yml`, `Homebrew`, and `crates.io` across the four new skills — zero
  matches.

## Stash recovery — memory export workspace/scope — 2026-06-22

- During cleanup of old aggregate stashes, recovered only the still-relevant
  `memory_export` behavior fix from the split-query leftovers.
- `memory_export` now honors its documented `workspace` parameter and rejects
  `include_embeddings=true` explicitly because embedding-inclusive export is
  still reserved.
- JSON export now includes `scope_type` and `scope_id`; import restores
  non-global scope instead of silently converting everything to global.
- Duplicate imports with `skip_duplicates=true` now report duplicates as
  `skipped` instead of `imported`.
- Added Review Canvas:
  `docs/harness/canvas/2026-06-22-memory-export-scope-workspace.md`.

Verification:

- `cargo fmt --all -- --check` — PASS.
- `cargo test -p engram-core --lib storage::queries::export --locked` — PASS,
  4 tests.
- `./scripts/generate-mcp-reference.sh --check` — PASS.
- `git diff --check` — PASS.
- `cargo clippy -p engram-core --lib --locked -- -D warnings` — PASS.
- `bash docs/harness/bin/doctor.sh` — PASS.
- `bash docs/harness/bin/sensors.sh` — PASS (full canonical gate, `make ci`
  + PR-title policy + harness doctor).
- `bash docs/harness/bin/review-gate.sh post ENGRA-150 --range origin/main..HEAD --review-file docs/harness/reviews/2026-06-22-ENGRA-150-v2-post.md`
  — PASS (`REVIEW_VERDICT: PASS`).

## ENGRA-150 query-layer lifecycle updates — 2026-06-22

- Added `storage::queries::update_memory_lifecycle_state` as the canonical
  lifecycle transition path for memories, including version rows, memory update
  events, and sync-state bookkeeping.
- Replaced raw lifecycle `UPDATE memories` writes in
  `src/mcp/handlers/dream.rs` and `src/mcp/handlers/lifecycle.rs` with the
  query-layer helper.
- Preserved `memory_set_lifecycle`'s existing missing-memory response shape.
- Kept MCP tool schemas unchanged; `docs/MCP_TOOLS.md` remains up to date.
- Added Review Canvas:
  `docs/harness/canvas/2026-06-22-ENGRA-150-query-layer-lifecycle-updates.md`.

Verification:

- `cargo check -p engram-core --all-targets --locked` — PASS.
- `cargo test -p engram-core --lib test_update_memory_lifecycle_state_records_update_side_effects --locked`
  — PASS.
- `cargo test -p engram-core --lib lifecycle_tests --locked` — PASS.
- `cargo test --test dream_integration --features dream-phase test_mcp_expire_candidate_does_not_apply_when_target_is_no_longer_active --locked`
  — PASS.
- `grep "UPDATE memories" src/mcp/handlers/dream.rs src/mcp/handlers/lifecycle.rs`
  — PASS, zero matches.
- `./scripts/generate-mcp-reference.sh --check` — PASS.
- `cargo fmt --all -- --check` — PASS.
- `git diff --check` — PASS.
- `cargo clippy -p engram-core --lib --locked -- -D warnings` — PASS.
- `bash docs/harness/bin/doctor.sh` — PASS.
- `bash docs/harness/bin/sensors.sh` — PASS (full canonical gate, `make ci`
  + PR-title policy + harness doctor).

## Harness 12207 Wave 0 measurement hardening — 2026-06-25

- Applied the first ISO/IEC/IEEE 12207 follow-up slice to the operational
  harness: strict shell failure mode for `bootstrap.sh` and `lib.sh`, plus
  historical sensor measurement.
- Preserved `.sensors-last` as the compatibility state file and added
  `.sensors-log` as JSON Lines history with `schema_version=sensors-log-v1`,
  `duration_sec`, per-layer statuses, artifact pointers, exclusion metadata, and
  bounded rotation via `SENSORS_LOG_MAX_BYTES` / `SENSORS_LOG_ROTATIONS`.
- `doctor.sh` now validates `.sensors-log` when present, runs `bash -n` over
  harness scripts, and runs optional non-blocking `shellcheck -x` when
  installed.
- Documented the new measurement contract in `docs/harness/GATES.md` and
  `docs/harness/JSON_OUTPUTS.md`.
- Added Review Canvas:
  `docs/harness/canvas/2026-06-25-harness-12207-wave0.md`.

Verification:

- `bash docs/harness/bin/bootstrap.sh` — PASS, 41 lines.
- `bash docs/harness/bin/doctor.sh` — PASS.
- `bash docs/harness/bin/doctor.sh --json | python3 -m json.tool` — PASS,
  includes `bash_syntax:*`, optional `shellcheck:*`, and `sensors_log:format`
  checks.
- `bash docs/harness/bin/sensors.sh status --json | python3 -m json.tool` —
  PASS, exposes `.sensors-log`.
- `SENSORS_LOG_MAX_BYTES=1 SENSORS_LOG_ROTATIONS=2 bash docs/harness/bin/sensors.sh baseline`
  run twice — PASS, proved rotation boundaries; generated rotated artifacts were
  removed after verification.
- `bash docs/harness/bin/sensors.sh quick` — PASS.
- `bash docs/harness/bin/sensors.sh` — PASS (full canonical gate, `make ci` +
  PR-title policy + harness doctor), recorded in `.sensors-log` with
  `mode=full`, `status=pass`, `duration_sec=26`.
- Codex review attempt
  `docs/harness/reviews/2026-06-26-harness-12207-wave0-v2-post.md` —
  `REVIEW_VERDICT: FAIL`; fixed by making optional ShellCheck non-blocking and
  preparing a scoped Codex-only final review.
- Codex review attempt
  `docs/harness/reviews/2026-06-26-harness-12207-wave0-v3-post.md` —
  `REVIEW_VERDICT: FAIL`; fixed by removing Python heredocs from
  `doctor.sh`/`sensors.sh` JSON paths for strict read-only sandbox
  compatibility.
- Codex final review
  `docs/harness/reviews/2026-06-26-harness-12207-wave0-v5-post.md` —
  `REVIEW_VERDICT: PASS scoped harness-12207-wave0 review passed with no
  findings`.
- `bash docs/harness/bin/review-gate.sh post harness-12207-wave0 --review-file docs/harness/reviews/2026-06-26-harness-12207-wave0-v5-post.md`
  — PASS.


## discover_tools detail levels — 2026-06-26

- Extended the existing `discover_tools` MCP tool with `detail` levels instead
  of introducing a redundant discovery tool.
- `detail: "names"` returns only tool names for cheapest discovery.
- Omitted `detail` or `detail: "summary"` preserves the existing response shape:
  name, description and tier.
- `detail: "schema"` adds the parsed input schema object so agents can call a
  discovered tool without a second full `tools/list` round-trip.
- Invalid `detail` values now return an explicit boundary error.

Verification:

- `cargo test --test mcp_protocol_tests discover_tools --locked` — PASS, 5
  tests passed.
- `./scripts/generate-mcp-reference.sh --check` — PASS.
- `bash docs/harness/bin/doctor.sh` — PASS.
- `git diff --check` — PASS.
- `bash docs/harness/bin/sensors.sh` — PASS (full canonical gate).
- LSP diagnostics could not be collected because the local LSP transport closed;
  Rust compiler/test gates are the verification fallback for this session.
- Codex post-review `docs/harness/reviews/2026-06-26-discover-tools-detail-post.md` — PASS.

## Reference intake checklist — 2026-06-27

- Added `docs/harness/REFERENCE_INTAKE.md` as the canonical intake checklist for
  external harness references, standards, articles, repos, benchmarks, tool
  catalogs, awesome lists, prompt/workflow kits, and local-only reference
  artifacts.
- The checklist captures source identity, source type, license boundary, local
  harness relevance, placement, adaptation, exclusions, and verification
  evidence.
- `docs/harness/GATES.md` now requires reference-intake evidence when external
  sources shape harness policy, gates, taxonomy, skills, reviewer prompts, or
  exception handling.
- `docs/harness/CODE_REVIEW_POLICY.md` now instructs reviewers to flag missing
  intake evidence and block copied licensed material, gate weakening, autonomous
  execution imports, or external sources overriding local invariants.
- Added Review Canvas:
  `docs/harness/canvas/2026-06-27-reference-intake-checklist.md`.

Verification:

- `rtk bash docs/harness/bin/doctor.sh` — PASS.
- `rtk git diff --check` — PASS.
- Markdown hygiene check for `REFERENCE_INTAKE.md`, `GATES.md`,
  `CODE_REVIEW_POLICY.md`, and the canvas — PASS.
- `rtk bash docs/harness/bin/sensors.sh quick` — PASS.
- `rtk bash docs/harness/bin/sensors.sh` — PASS (full canonical gate,
  timestamp `2026-06-27T09:10:45Z`, `duration_sec=28`).

Scope notes:

- No script, Rust, MCP, SDK, storage, runtime, or autonomous execution changes.
- Automation for duplicate URL checks, markdown entry-shape checks, or exception
  allowlist validation is intentionally deferred to a separate task.
- The worktree contained unrelated pre-existing dirty changes before this slice;
  closure must stage only separable reference-intake hunks/files.

Post-review fix:

- Independent Codex post-review `docs/harness/reviews/2026-06-27-reference-intake-checklist-post.md` returned `REVIEW_VERDICT: FAIL` because the canvas cited `walkinglabs/awesome-harness-engineering` without dogfooding the new intake evidence.
- Fixed by adding an `External Reference Intake` table to `docs/harness/canvas/2026-06-27-reference-intake-checklist.md` with source identity, source type, license boundary, relevance, placement, adaptation, exclusions, and verification.
- The FAIL is superseded by this fix and requires a new post-review artifact before closure.

Final verification after post-review fix:

- `rtk git diff --check` — PASS.
- Markdown hygiene check for reference-intake files and progress ledgers — PASS.
- `rtk bash docs/harness/bin/doctor.sh` — PASS.
- `rtk bash docs/harness/bin/sensors.sh` — PASS (full canonical gate,
  timestamp `2026-06-27T09:10:45Z`, `duration_sec=28`).

Post-review closure:

- Independent Codex rerun `docs/harness/reviews/2026-06-27-reference-intake-checklist-v2-post.md` returned `REVIEW_VERDICT: PASS reference-intake checklist slice meets acceptance criteria`.
- `rtk bash docs/harness/bin/review-gate.sh post reference-intake-checklist --review-file docs/harness/reviews/2026-06-27-reference-intake-checklist-v2-post.md` — PASS; parser accepted the v2 artifact.

## Lifecycle predicate implementation plan — 2026-06-27

- Lifecycle predicate unification spec is committed and cross-model reviewed:
  - `1f9f25b` — initial lifecycle predicate design.
  - `a085c0f` — finalized lifecycle predicate spec after v3 re-review PASS.
- Added the implementation plan at
  `docs/superpowers/plans/2026-06-27-lifecycle-predicate-unification.md`.
- Plan scope: implement `decide_lifecycle_state`, route `lifecycle_run` through
  the canonical predicate, disarm salience/policy/compression lifecycle writers,
  preserve domain writers, keep `SCHEMA_VERSION=44`, update MCP metadata,
  regenerate `docs/MCP_TOOLS.md`, and verify single-writer behavior.
- Plan review fix: removed `expires_at` from the Step 2.3 `lifecycle_run` SQL
  pre-filter. The pre-filter now selects only `valid_to IS NULL` and non-Archived
  rows, with optional workspace filtering.
- Lesson recorded: lifecycle pre-filters must not filter on fields the canonical
  predicate does not model. `expires_at` now appears only in the explicit
  prohibition line.
- Implementation is intentionally deferred to a dedicated TDD session using this
  plan as the contract.

Verification:

- `rtk grep -nE "expires_at" docs/superpowers/plans/2026-06-27-lifecycle-predicate-unification.md` — PASS; single occurrence in the prohibition line.
- Step 2.3 SQL readback — PASS; only `valid_to IS NULL`, non-Archived, and optional workspace clause remain.
- `rtk git diff --check -- docs/superpowers/plans/2026-06-27-lifecycle-predicate-unification.md` — PASS.

## GitHub harness protection — Wave 1 — 2026-06-27

Plan: `.omo/plans/github-harness-protection.md` (reviewed + corrected;
approved-for-execution). Staged GitHub protection: objective checks block,
AI review advisory, stricter enforcement only after observation. Wave 1
(repo-owned, reversible) implemented; GitHub settings (Wave 2) deferred.

Done this wave:

- **Todo 1 — CODEOWNERS**: `.github/CODEOWNERS` with `@aiconnai/core` over
  protected surfaces only (CI/workflows, harness, agent/governance docs,
  Cargo/deny/audit policy, release/CI scripts, CODEOWNERS self-gate). No
  catch-all owner. Owner handle confirmed by user; no placeholder/guess.
- **Todo 2 — Harness Contract workflow**: `.github/workflows/harness-contract.yml`.
  Required job runs only `bootstrap.sh` + `pr-title-policy.sh`; advisory
  `Harness Doctor Advisory` job (`continue-on-error`) runs the self-diagnostic.
  `PR_TITLE` falls back to a non-empty placeholder on `merge_group`/manual to
  avoid the empty-title usage-error footgun.
- **Todo 3 — PR security (advisory)**: added `pull_request` to existing
  `Security Audit` + `Cargo Deny` jobs in `ci.yml` (reuse, not new workflow).
  Kept advisory — local baseline FAILS (`RUSTSEC-2026-0187` lopdf,
  `RUSTSEC-2026-0185` quinn-proto 7.5 high), so promotion to required is blocked
  until resolved/ignored. Tracked for a dedicated issue.
- **Todo 4 — Supply-chain pin**: AgentShield install pinned `--tag v0.8.7` →
  `--rev a1a571197211793422d31811be3d7735dae0a30a` (peeled commit of the
  annotated tag). Confirmed `nightly.yml` stays schedule/manual/advisory.
- **Todo 5 — Docs/PR template**: PR template + `README.md` + `GATES.md` document
  the local loop (`bootstrap` → `sensors.sh quick` → full `sensors.sh`) and the
  required check names; AI review framed as advisory, `pr-title-policy.sh`
  framed as `[codex]`-only.

Deferred: Todos 6–8 (ruleset, security features, observation/promotion) require
GitHub admin settings (Wave 2/3). Evidence in `.omo/evidence/task-{1..5}-*.md`.

Verification:

- Todo 1 AC (CODEOWNERS has `docs/harness/**` + `.github/workflows/**`, all
  surfaces owned, no placeholder) — PASS.
- Todo 2 AC (required block = bootstrap + pr-title, excludes doctor; `[codex]`
  title → exit 4; merge_group fallback non-empty) — PASS.
- Todo 3 AC (both jobs present, both trigger on PR) — PASS; baseline FAIL
  recorded as the reason they stay advisory.
- Todo 4 AC (AgentShield pinned by `--rev`, no `--tag`; nightly not on PRs) — PASS.
- Todo 5 AC (`Harness Contract` in 3 docs, `sensors.sh quick` in 2; no
  "AI review is authoritative") — PASS.
- `rtk git diff --check` — PASS (no whitespace/conflict markers).

## Agent Memory Contract C1.1 — pending writeback candidates — 2026-07-03

Plan: `docs/superpowers/plans/2026-07-03-agent-memory-contract.md`.
Branch: `codex/c1-agent-writeback-candidates`.
Progress log:
`docs/harness/progress/2026-07-03-agent-memory-contract-c1.md`.

Done in this slice:

- Added schema migration v45 so the existing `dream_candidates.kind` CHECK
  accepts `agent_writeback`; no new writeback table was introduced.
- Added `agent_writeback` to storage-level dream candidate validation.
- Added Advanced-tier, `dream-phase`-gated MCP tool `memory_agent_writeback`.
- `memory_agent_writeback` defaults to `dry_run=true`, requires `confirm=true`
  for live pending-candidate creation, and requires at least one evidence source
  (`source_memory_ids` or structured `evidence`).
- Confirmed calls create only `dream_candidates` and `dream_candidate_sources`;
  canonical memory still changes only through the existing
  `dream_candidate_review` + `dream_candidate_apply` path.
- Post-review hardening maps applied `agent_writeback` candidates to
  `learning`, keeps dry-run/live response shapes isomorphic, cleans duplicate
  candidate conflicts, validates reused job provenance/status, completes
  synthetic jobs after candidate creation, and rejects reserved metadata
  spoofing by casing.
- Updated `memory_agent_contract` to version `agent-memory-contract-v1` with
  the new creation tool, schema version, creation/validation rules, and a
  structured v45 migration object instead of a forever-true migration flag.
- Regenerated `docs/MCP_TOOLS.md` and updated `docs/AI_GUIDE.md`.
- Added Review Canvas:
  `docs/harness/canvas/2026-07-03-c1-agent-writeback-candidates.md`.

Verification so far:

- `rtk cargo test --lib storage::migrations::tests::test_dream_candidates_allow_agent_writeback_kind` — PASS.
- `rtk cargo test --features dream-phase --test mcp_protocol_tests memory_agent_writeback_tool_is_advanced_dry_run_mutating_surface` — PASS.
- `rtk cargo test --features dream-phase --test dream_integration test_mcp_memory_agent_writeback_requires_review_before_canonical_apply` — PASS.
- `rtk cargo test --features dream-phase --test dream_integration test_mcp_memory_agent_writeback_rejects_reuse_and_spoofing --locked` — PASS.
- `rtk cargo test --test mcp_protocol_tests memory_agent_contract_dispatches_governance_contract` — PASS.
- `rtk cargo test --lib storage::migrations::tests::test_v45_preserves_existing_dream_candidate_data --locked` — PASS.
- `rtk cargo fmt --all -- --check` — PASS.
- `rtk git diff --check` — PASS.
- `rtk ./scripts/generate-mcp-reference.sh --check` — PASS.
- `rtk cargo check --workspace --all-targets --locked` — PASS.
- `rtk cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` — PASS.
- `rtk cargo test --workspace --all-targets --locked` — PASS, 1250 tests.
- `rtk bash docs/harness/bin/sensors.sh` — PASS, full lane, timestamp
  `2026-07-03T11:44:01Z`, duration 35s.
- MCP stdio smoke with `--features dream-phase` and isolated `ENGRAM_DB_PATH`:
  `memory_agent_writeback` returned `status=dry_run` and
  `canonical_memory_mutated=false` by default.
- MCP stdio smoke with `dry_run=false, confirm=true` created pending candidate
  `smoke-agent-writeback-candidate`; `dream_candidate_get` returned the pending
  `agent_writeback` candidate and its source.

## Harness Contract workflow YAML repair — 2026-07-12

- Fixed `.github/workflows/harness-contract.yml` after the post-merge push for
  PR #187 produced a zero-job failure because an unquoted expression contained
  the `merge-group: no PR title` scalar.
- Quoted the complete `PR_TITLE` expression so the workflow remains valid YAML
  while preserving the existing pull-request and merge-queue fallback behavior.
- Verification: Ruby YAML parse PASS; harness doctor PASS; quick sensors PASS;
  PR-title fallback PASS; independent Sonnet post-review PASS.
