# Engram Harness Spec

| Field | Value |
|-------|-------|
| Project | `engram` |
| Active sprint | `Remediação pós-levantamento 2026-10-26` |
| Active task | `remediation-2026-10-26` |
| Started | `2026-10-26` |
| Owner | `Ronaldo + agents (Claude Code + Claude Code Sonnet reviewer)` |
| Active spec | `docs/harness/SPEC.md` |
| Active plan | `docs/harness/progress/2026-10-26-remediation.md` |
| Tracker | Owner (chat, 2026-10-26): "resolva tudo o mais paralelo possível" → 6 branches locais paralelas a partir de `08cb17a4`; push, PR, merge, tag, release, publicação e remoção de branches/worktrees **não** autorizados (aguardam decisão explícita do owner) |

## Escopo da sprint ativa

> **Este SPEC é o escopo da sprint/tarefa ativa do harness. Não é o blueprint completo do produto.**
> - Blueprint de Harness Memory (produto): [`docs/rfcs/0001-harness-memory-product-boundary.md`](../rfcs/0001-harness-memory-product-boundary.md)
> - Regras de dados (sistema): [`../INVARIANTS.md`](../../INVARIANTS.md)
> - Standards gerais: [`../../STANDARDS.md`](../../STANDARDS.md)
> - Erros & lições: [`../../ERRORS_AND_LESSONS.md`](../../ERRORS_AND_LESSONS.md)

## Tarefa ativa: Remediação pós-levantamento 2026-10-26

- **Mandato**: o levantamento read-only de 2026-10-26 achou o estado vivo defasado e regressões; o owner
  pediu "resolva tudo o mais paralelo possível" (chat, 2026-10-26).
- **Plano**: [`plans/2026-10-26-remediation-plan.md`](./plans/2026-10-26-remediation-plan.md). **Log ativo e evidência**
  (alvo do campo `Active plan`): [`progress/2026-10-26-remediation.md`](./progress/2026-10-26-remediation.md).
- **Base**: `08cb17a4` (= `main` local `b29cd16c` + `refactor(storage): complete turso-sunset`).
- **Execução**: 6 branches **locais** paralelas (WT1 `fix/workspace-batch-auth`, WT2 `fix/supply-chain-2026-10`,
  WT3 `fix/release-pipeline`, WT4 `fix/mcp-surface-honesty`, WT5 `chore/harness-live-state`,
  WT6 `docs/product-status`). O integrador mescla numa branch de integração, roda o CI completo e só então
  finaliza os SHAs em `progress.md`.
- **Em escopo (WT5, estado vivo e governança)**: transição de sprint; campos vivos e log retroativo de
  `d922a60e`, `e04d3307`, `c9de7136`, `b29cd16c`; proveniência do veredito de 2026-10-14; `CLAUDE.md`,
  `AGENTS.md`, `CONTRIBUTING.md` e docs do harness (caminhos e comandos obsoletos); fechamento do plano
  antigo e de D1/D7/D9; triagem dos minors adiados (`known-issues/`, `risk-register.yaml`).
- **Exceção explícita para scripts do harness** (`docs/harness/bin/*` é process-critical): somente as
  correções do canvas [`2026-10-26-harness-script-fixes`](./canvas/2026-10-26-harness-script-fixes.md) em
  `check-live-state.sh` (autoridade de review derivada de `progress.md`), `test-check-live-state.sh`,
  `bootstrap.sh` (contador MCP, `CLAUDE.md`), `quarterly-audit.sh` (`CLAUDE.md`) e os pisos do tripwire
  (`live_state` 40 para 44, `context_budget` 49 para 50) em `run-offline-lane.sh`/`test_offline_lane.py`, mais um
  teste do contador MCP em `test_context_budget.py`. Essas mudanças **não são
  autoritativas** até haver review independente ou sign-off do owner (WHAT_WE_DONT_DO).
- **Fora de escopo**: push, PR, merge, tag, release, publicação, remoção de branches/worktrees, settings do
  GitHub, produção, dados reais, chamadas pagas, código de produto e CI/dependências (outras frentes),
  weakening de gate, receipt ou PASS fabricado.
- **Gates**: `bootstrap.sh`, `doctor.sh`, `check-live-state.sh` estrito, lane offline e `git diff --check`.
  O gate completo (`sensors.sh`) e o post-gate com receipt humano ficam com o integrador/owner.

## Follow-up autorizado — 2026-10-27

A pilha #244–#262 está mesclada no main `2e84a594`. O GO antigo encerrou às 09:35Z.
Novo GO do owner (chat): diagnosticar Nightly, corrigir defeitos confirmados e lacunas
relevantes de testes, resolver a proposta de verify-tag e preparar release/aviso de migração;
entregar evidências e recomendação antes de publicar ou executar mudanças operacionais.
O [plano do follow-up](plans/2026-10-27-nightly-release-readiness.md) detalha paths e aceite.
Permite alterações e verificações locais; não autoriza push/PR/merge, tags de release,
publicação, envio de avisos, deleções ou settings. Os limites históricos acima referem-se
à frente WT5 original; este follow-up inclui os testes, o fix de scoping e os defeitos confirmados do gerador
de metadados de SDK ali delimitados.

## Follow-up autorizado — 2026-11-01, P0 export Markdown

Owner autorizou seguir no P0 independente após o plano v3. Correção local sobre main revalidada,
testes RED/GREEN, paths/permissões/expiração/integridade/atomicidade e catálogo/reference, com
verificação no candidato. [Log](progress/2026-11-01-markdown-export-safety.md),
[canvas](canvas/2026-11-01-markdown-export-safety.md). Tag 0.24.0 aguarda resultado; sem autorização
de push/PR/merge/tag/publicação, configs reais, RFC0004a ou programa inteiro de evolução.

Follow-up da revisão: corrigir compatibilidade das tags e custo de sincronização na branch export,
revalidar em Linux/macOS e registrar RISK-0011 para import em candidato separado. Sem ampliar para
projeção Git. [Log retomável](progress/2026-11-01-markdown-portability-followup.md).

RISK-0011: branch local separada `fix/markdown-import-safety`, baseada no export `9bca4efa`.
Leitura ancorada limitada, modo conforme raiz e autorização/classificação/aplicação transacionais;
fixtures temporárias com principals reais antes da correção. [Canvas](canvas/2026-11-01-markdown-import-safety.md)
e [log](progress/2026-11-01-markdown-import-safety.md). Mesmos limites operacionais do P0.

## Follow-up autorizado — release closeout, 2026-11-07

Owner pediu executar a sequência revisada após a avaliação de07/11: candidato isolado baseado
no fix SDK2c7544db sobre main7ca7a9b8, reconciliação documental/risco e evidência local para
decisão final. [Plano](plans/2026-11-07-release-closeout.md),
[canvas](canvas/2026-11-07-release-closeout.md). Inclui o defeito CDPATH reproduzido no verificador,
seus testes e a documentação útil do passo6. Sem Rust/API/schema/versões ou novos gates.
H1 humano continua necessário antes de upstream; merge/tag/publicação/notices/settings/cleanup
mantêm decisão própria. A [evolução v4](plans/2026-11-07-engram-evolution-plan-v4.md) atualiza
base/dependências; não aceita RFC0004a nem inicia experimentos ou execução operacional.

## Tarefa anterior: Turso/libSQL sunset v0.24.0

- **Status**: implementação concluída e commitada em `08cb17a4` (2026-10-23); sensores completos PASS
  registrados em `.sensors-last` (310 s). **Post-gate `PENDING`**: não há receipt humano; o parecer Sonnet
  `v2` é opinião, não aprovação. Release, tag e publicação **não** foram feitos.
- **Decisão existente**: [ADR-STORAGE-20260816-1](../adr/2026-08-16-turso-backend-deprecation-and-sunset.md), aceito.
- **Plano e evidência**: [turso-sunset](./progress/2026-10-23-turso-sunset.md).

## Tarefa anterior: Engram comprehensive improvement — ondas 0-6 (concluída em 2026-10-14)

- **Plano histórico**: [`plans/2026-10-02-engram-comprehensive-improvement-plan.md`](./plans/2026-10-02-engram-comprehensive-improvement-plan.md) (encerrado; sem as ondas 5-6).
- **Progress log (Lane P — harness/governança/CI docs)**: [`progress/2026-10-05-improvement-lane-p.md`](./progress/2026-10-05-improvement-lane-p.md)
- **Progress log (Lane R — Rust/produto)**: [`progress/2026-10-05-improvement-lane-r.md`](./progress/2026-10-05-improvement-lane-r.md)
- **Baseline de execução**: `1952f3b461fe7a8f8a6b5c106ac8c4f5929bae91`. Inventário: [`audits/2026-10-02-improvement-baseline.md`](./audits/2026-10-02-improvement-baseline.md)
- **Decisões do owner (2026-10-05, chat, históricas)**: ADR `agent-harness-hardening-v1` **aceito**; runners da Onda 4 só com *fake writer*, offline; budget WAL 64 GiB configurável; `defer_embedding=true` enfileira em background. A autorização de execução daquele programa (commits locais, ondas 0-4) **expirou com o programa** e não vale para outras sprints.
- **Status**: completed (2026-10-14) — as 25 tarefas do plano e as tarefas G1, P1 e Q2F concluídas com revisão SDD independente; lanes integradas (INT); revisão final do branch (Opus): *Ready with fixes*, 0 Critical, corrigida na onda FINALFIX. Ondas 5 e 6 pós-merge (`661b140`..`b1fd869`): `quinn-proto` e deadlock Bash 5.3 em `doctor.sh`, `LazyLock`, `CloudStorage::upload` Invariante #27, `v49` `media_assets` (`#235`), leituras de mídia limitadas (`#236`), wiring `SessionStart`/`PostToolUse`/`SyncWorker` (`#237`), retry `-shm` macOS (`#238`), `aws-sdk-s3` + `rustls 0.23.46` (`#239`), SDKs `161 -> 0` (`G-2`, `G-4`, `G-5`), normalizador de erros MCP (`G-6`, `G-7`, `G-10`), `P1`/`G1`, RFCs 0009–0011, `O2` (73 `.raw`), preparação `v0.24.0` Core / `v0.7.0` SDKs. Registro: [`reviews/2026-10-05-comprehensive-improvement/README.md`](./reviews/2026-10-05-comprehensive-improvement/README.md).
- **Proveniência**: o veredito [`2026-10-14`](./reviews/2026-10-14-engram-comprehensive-improvement-post.md) não registra reviewer, intervalo nem receipt; ver a [nota de proveniência](./reviews/2026-10-14-engram-comprehensive-improvement-post-provenance.md). Review Canvas do programa: **ausente** (desvio registrado em [`audits/2026-10-02-improvement-baseline.md` §9](./audits/2026-10-02-improvement-baseline.md)).

## Tarefa anterior: Agent harness hardening v1 — Wave 0 governance

- **Branch**: `feat/harness-hardening-v1`
- **Progress log**: [`progress/2026-07-21-agent-harness-hardening-v1.md`](./progress/2026-07-21-agent-harness-hardening-v1.md)
- **Status**: superseded 2026-10-05 — governance proposal landed; the ADR was
  **accepted by the owner on 2026-10-05** (see the ADR acceptance record). The
  executable waves now run as part of the comprehensive improvement program above.

### Em escopo

- Proposed trust-boundary ADR, active plan, Review Canvas, and external-reference intake.
- Synchronized `SPEC.md` and `progress.md` live-state routing.
- Independent review under the existing pre-change gates.

### Fora de escopo

- Executable runner, schemas, validators, sandbox adapter, review-parser
  migration, merge gate, CI/workflow changes, model routing, deployment, and
  production access.
- Rust, MCP, storage, SDK, dependency, or product behavior changes.

### Gates esperados

- `bash docs/harness/bin/bootstrap.sh`
- `bash docs/harness/bin/doctor.sh`
- `bash docs/harness/bin/sensors.sh`
- `git diff --check`
- `bash docs/harness/bin/review-gate.sh post agent-harness-hardening-v1`
- Human acceptance of the ADR

## Tarefa anterior: Harness maintenance — live-state closeout

- **Branch**: `chore/harness-live-state`
- **Progress log**: [`progress/2026-06-27-harness-live-state-closeout.md`](./progress/2026-06-27-harness-live-state-closeout.md)
- **Status**: completed (2026-07-10) — fechamento do estado vivo; review PASS em
  [`reviews/2026-07-10-engram-10-of-10-live-state-v4-post.md`](./reviews/2026-07-10-engram-10-of-10-live-state-v4-post.md).

### Em escopo

- Atualizar `docs/harness/progress.md` para deixar de apresentar
  `harness-bootstrap` como tarefa ativa.
- Atualizar este `SPEC.md` para manter o contrato `SPEC.md` ↔ `progress.md`
  exigido pelo `doctor.sh`.
- Registrar um novo log de progresso curto para esta tarefa de housekeeping.
- Atualizar o metadata de último commit/review/sensores com evidência atual.
- Adicionar `docs/harness/bin/check-live-state.sh` e `docs/harness/bin/test-check-live-state.sh` como exceção escopada para tornar o live state self-checking.
- Registrar `docs/harness/canvas/2026-07-09-engram-10-of-10-live-state.md` para a mudança de script do harness.

### Fora de escopo

- Mudanças em scripts do harness além da exceção explícita `check-live-state.sh`/`test-check-live-state.sh`; gates, invariants, policy, workflows, MCP,
  storage, SDKs ou código Rust.
- Escolher ou iniciar um follow-up de produto; este closeout só deixa o estado
  canônico pronto para a próxima branch real.

### Gates esperados

- `bash docs/harness/bin/bootstrap.sh`
- `bash docs/harness/bin/doctor.sh`
- `bash docs/harness/bin/sensors.sh quick`
- `bash docs/harness/bin/review-gate.sh post engram-10-of-10-live-state`

## Sprint encerrada: Harness Engineering v0 — bootstrap & core gates

- **Branch**: (branch atual do trabalho)
- **Progress log**: [`progress/2026-05-30-harness-bootstrap.md`](./progress/2026-05-30-harness-bootstrap.md)
- **Status**: completed — implementação inicial do harness operacional
  inspirado no modelo mbras-backend, adaptado para Rust + MCP + multi-SDK +
  dual CLI (Claude Code + Claude Code Sonnet), já integrada em `main`.

### Em escopo (v0)

- Criar estrutura `docs/harness/` completa (README, SPEC, INVARIANTS, GATES, CODE_REVIEW_POLICY, progress.md, bin/ scripts, reviews/, known-issues/, progress/).
- Implementar `bootstrap.sh` — orientação rápida, read-only, <50 linhas, imprime estado + ordem de leitura.
- Implementar `doctor.sh` — validação de consistência do harness (drift SPEC/progress, executáveis, referências à policy, bootstrap size, etc.).
- Implementar `sensors.sh` — wrapper determinístico com as etapas de `scripts/ci.sh` (alvo de `make ci`; fmt + clippy -D + testes paridade Linux + docs + MCP ref) + harness doctor + checks específicos de engram.
- Implementar `review-gate.sh` — generalizado para múltiplos reviewers (claude-sonnet, codex, local). Suporte a pre/post, range, continuity em FAILs, versionamento de artefatos, exclusão de paths do harness, timeout, prompt rico com fake-success patterns de Rust/MCP.
- Implementar `check-commit-msg.sh` — validador de Conventional Commits com scopes engram/harness.
- Criar `CODE_REVIEW_POLICY.md` adaptada para Rust, engram (MCP tools, hooks, embeddings, storage invariants, cross-SDK), e o cenário dual-CLI atual.
- Criar `GATES.md` com thresholds, fake-success patterns específicos (ex.: tests passando só com features locais mas falhando em CI Linux, schema version drift, MCP protocol breakage, embedding cache bounds violados, etc.).
- Criar `WHAT_WE_DONT_DO.md` como política explícita de escopo negativo para evitar expansão silenciosa de mudanças de harness.
- Criar `docs/harness/canvas/` com template de Review Canvas para mudanças complexas.
- Adicionar `baseline.sh`, `quarterly-audit.sh`, lanes opcionais em `sensors.sh` e guard de review para `docs/harness/bin/*`.
- Atualizar `AGENTS.md` e `Claude.md` para exigir `bootstrap.sh` no início de toda sessão.
- Atualizar pre-commit hook e/ou justfile para reforçar (sem quebrar fluxo atual).
- Seed de progresso para esta sprint + registro de decisões.
- Rodar o loop completo (bootstrap → pre → sensors → post) nesta própria implementação.
- Documentar como o harness se relaciona com o RFC 0001 (Harness Memory product boundary) e dogfooding futuro.

### Gates esperados para v0

- `bash docs/harness/bin/bootstrap.sh`
- `bash docs/harness/bin/doctor.sh`
- `bash docs/harness/bin/sensors.sh`
- `bash docs/harness/bin/review-gate.sh pre harness-bootstrap`
- `bash docs/harness/bin/review-gate.sh post harness-bootstrap`
- `make ci` (`scripts/ci.sh`; paridade mantida; `just ci` é alias opcional quando `just` está instalado)

### Fora de escopo (v0)

- Implementação completa de ingestão automática de eventos de harness no próprio engram (deixa para ENGRA-22+ seguindo RFC 0001).
- Mudanças em storage schema, MCP tools novos, ou intelligence/consolidation.
- Suporte nativo non-interactive exec para todos os CLIs (foco em prompt files + paste workflow para o cenário atual Claude Code + Claude Code Sonnet).
- Substituição total de `.githooks` ou CI GitHub workflows (o harness complementa, não substitui).
- Reabrir ou alterar RFC 0001 neste escopo.
- Mudanças em `INVARIANTS.md` (raiz) ou `STANDARDS.md` — apenas docs de processo do harness.

## Próximas iterações (v1+)

- Dogfooding: usar engram MCP + hooks para registrar sessões de harness, reviews, gate results como memórias com provenance forte.
- `memory_harness_*` tools ou seção dedicada.
- Agentes especializados em harness (planner, verifier, context-engine) como MCP tools ou personas em `docs/harness/agents/`.
- Integração mais profunda com Claude Code Sonnet como reviewer externo non-interactive.
- Suporte a Linear/GitHub sync de tasks no harness (se aplicável ao fluxo de engram).

## Emenda 2026-06-05 — cross-harness improvements

O plano `docs/harness/plans/2026-06-05-engram-harness-improvement-execution-plan.md` adiciona melhorias inspiradas no harness mbras sem importar comportamento de domínio externo:

- Escopo negativo explícito em `WHAT_WE_DONT_DO.md`.
- Review Canvas para evidência em mudanças complexas.
- Guard para mudanças em `docs/harness/bin/*`.
- Baseline snapshot barato.
- Lanes opcionais em `sensors.sh` que não substituem o gate completo.
- Auditoria periódica evidence-only.

## Critérios de Saída da Sprint v0

- Estrutura completa + scripts executáveis + doctor.sh verde.
- Esta própria task passou pelo loop completo (pre + sensors all-green + post PASS).
- AGENTS.md e Claude.md atualizados e bootstrap rodado com sucesso.
- README do harness explica o posicionamento de engram como Memory Manager ideal para harnesses de outros projetos.
- Nenhum drift entre SPEC.md e progress.md; doctor.sh passa limpo.

---

**Nota**: Este SPEC é mutável durante a sprint. Atualizações de escopo vão para o log de progresso e (quando relevante) para um novo version do SPEC com nota de data. Invariants do harness não mudam sem ADR + gates anteriores.

## Follow-up local — candidato integrado Markdown, 2026-11-01

Combinar remediação646636f6 com import1faa5c6f em worktree/branch nova; reter79889460 e hardening,
validar CLI/SDKs/MCP review default, omitir entradas ocultas do vault, rodar gates no candidato.
[Log](progress/2026-11-01-markdown-release-integration.md), [canvas](canvas/2026-11-01-markdown-release-integration.md).
Sem publicar, alterar branch principal ou renovar mandato remoto da branch anterior.

## Follow-up autorizado — Markdown architecture (2026-11-02)

Owner approved local deepening of Markdown export/import together, preserving
public behavior and extracting only shared auth policy. Separate worktree based
on 60ed4eec; [plan](plans/2026-11-02-markdown-portability-core.md) and
[evidence](progress/2026-11-02-markdown-portability-core.md). No remote authority added.

## Release execution follow-up — missing artifact signer, 2026-11-07

Owner requested publication after PR265 merge. Preflight on85502ad8 found the Core
workflow calls a helper removed in3128d444. Bounded local restoration, real bundle
regressions in the existing packaging suite, cleanup/strict envelope checks, docs
and verification are authorized necessary remediation. [Canvas](canvas/2026-11-07-release-attestation-helper.md).
No Rust/API/schema/versions or governance implementation changes. Tag/upload waits
for a fixed reviewed candidate and protected-publication configuration.

## Follow-up local — RUSTSEC-2026-0459 (2026-11-10)

Delegated by the read-only main review of 2026-11-09 (`85502ad8`): update `rusqlite` so both
lockfiles carry `libsqlite3-sys >= 0.39.0`. Scope: `Cargo.toml`, `Cargo.lock`, `fuzz/Cargo.lock`
and mechanical compile/test fallout; no schema, MCP, SDK, other dependency or advisory exception.
[Log](progress/2026-10-26-remediation.md#rustsec-2026-0459-rusqlite-update--2026-11-10).
Local commits only: push, PR, merge, tag, publication and GitHub settings await the owner.

## Follow-up autorizado — remediação pré-v0.24.0 (Opção A, 2026-11-12 / 2026-11-13)

Após os merges autorizados pelo owner dos PRs `#266`–`#276` em `main` até `37ef480a` (`#266` helper de
assinatura, `#267` `rusqlite` `0.41.1`, `#268` A-1..A-4, `#270` B-1/B-2, `#274` `vitest` `5.1.0` /
`@types/node` `22`, `#275` `SECURITY.md`/`CLAUDE.md`, `#276` `CHANGELOG.md`) e habilitação de
controles GitHub (private vulnerability reporting, secret scanning + push protection, Dependabot e
rascunhos privados `GHSA-8xxr-p5q8-c7cg` / `GHSA-6q7w-w273-gq6p`), a revisão de prontidão
`85502ad8..37ef480a` identificou 13 achados e guardas pendentes pré-tag. O owner escolheu a **Opção A**
(`Node >= 22.12`, worktree isolado sobre `origin/main` `37ef480a`, push da branch e abertura de PR
contra `main`):
- **Escopo autorizado**: corrigir os 13 achados de `85502ad8..37ef480a`, o bloqueio de overwrite
  cross-workspace no import Markdown (`GHSA-6q7w-w273-gq6p`), `A-5`/`A-6`/`A-7`/`Q-A`, compressão em
  `ID_SCOPED_TOOLS`, `decay` single-ID, modo `read_only` nos hooks, `refs/remotes/origin/main`
  canônico em `verify-release-tag.sh`/`sdk-release.yml`, paridade SDKs Python/TypeScript, reconciliação
  documental/live-state ([Canvas](canvas/2026-11-12-pre-v0.24.0-remediation.md);
  [Log](progress/2026-10-26-remediation.md#pre-v0240-review-remediation-option-a--2026-11-13)),
  `git push` da branch `fix/pre-v0.24.0-remediation` e abertura do PR contra `main`.
- **Fora de escopo / fronteira remota**: `B-3`/`B-4` (`fu-archival`), refatoração `SAVEPOINT`
  `is_autocommit()` e mutante sobrevivente ficam adiados para `v0.24.1`. Merge em `main` (receipts
  humanos `H1`), assinatura/push da tag `v0.24.0`, publicação dos advisories `GHSA-*` e publicação em
  crates.io, PyPI, npm ou Homebrew permanecem **excluídos** até decisão humana separada.
