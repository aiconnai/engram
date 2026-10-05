# Engram Harness Spec

| Field | Value |
|-------|-------|
| Project | `engram` |
| Active sprint | `Engram comprehensive improvement program — waves 0-4` |
| Active task | `engram-comprehensive-improvement — waves 0-4 execution` |
| Started | `2026-10-05` |
| Owner | `Ronaldo + agents (Claude Code + Claude Code Sonnet reviewer)` |
| Active spec | `docs/harness/SPEC.md` |
| Active plan | `docs/harness/progress/2026-10-05-improvement-lane-p.md` |
| Tracker | Owner-authorized program (chat, 2026-10-05); local commits only, no push/PR/merge |

## Escopo da sprint ativa

> **Este SPEC é o escopo da sprint/tarefa ativa do harness. Não é o blueprint completo do produto.**
> - Blueprint de Harness Memory (produto): [`docs/rfcs/0001-harness-memory-product-boundary.md`](../rfcs/0001-harness-memory-product-boundary.md)
> - Regras de dados (sistema): [`../INVARIANTS.md`](../../INVARIANTS.md)
> - Standards gerais: [`../../STANDARDS.md`](../../STANDARDS.md)
> - Erros & lições: [`../../ERRORS_AND_LESSONS.md`](../../ERRORS_AND_LESSONS.md)

## Tarefa ativa: Engram comprehensive improvement — waves 0-4 execution

- **Plano completo**: [`plans/2026-10-02-engram-comprehensive-improvement-plan.md`](./plans/2026-10-02-engram-comprehensive-improvement-plan.md)
- **Progress log (Lane P — harness/governança/CI docs)**: [`progress/2026-10-05-improvement-lane-p.md`](./progress/2026-10-05-improvement-lane-p.md)
- **Progress log (Lane R — Rust/produto)**: [`progress/2026-10-05-improvement-lane-r.md`](./progress/2026-10-05-improvement-lane-r.md) (pertence à lane R; criado na worktree dessa lane)
- **Baseline de execução**: `1952f3b461fe7a8f8a6b5c106ac8c4f5929bae91` (14 commits locais à frente de `origin/main` `949c9634be28badea43ba2a2b5bcdf5d4c0dd358`; permanecem locais, sem push). Inventário: [`audits/2026-10-02-improvement-baseline.md`](./audits/2026-10-02-improvement-baseline.md)
- **Autorização do owner (2026-10-05, chat)**: ondas 0-4 autorizadas; ADR `agent-harness-hardening-v1` **aceito** (ver ADR); runners da Onda 4 somente com *fake writer*, offline; budget WAL 64 GiB configurável; `defer_embedding=true` enfileira em background; entrega por commits locais na branch da worktree.
- **Status**: active — as 25 tarefas do plano e as tarefas novas G1, P1 e Q2F concluídas com revisão SDD independente (todas "review clean"); lanes integradas (INT); revisão final do branch (Opus): *Ready with fixes*, 0 Critical, corrigida na onda FINALFIX (sem re-revisão ao commitar). Nenhum `review-gate.sh post` com receipt foi rodado. Pendente: decisões do owner e merge humano. Registro: [`reviews/2026-10-05-comprehensive-improvement/README.md`](./reviews/2026-10-05-comprehensive-improvement/README.md).

### Em escopo

- Reconciliar estado/autorização/baseline (E0) e as tarefas H*, Q*, C*, O* do plano, cada uma com seu brief, TDD/evidência e review independente.
- Docs/harness da Lane P: `docs/harness/`, `docs/decisions/`, `docs/harness/audits/`; Lane R mantém seu próprio log.
- Manter `SPEC.md` ↔ `progress.md` sincronizados (`doctor.sh`).

### Fora de escopo

- Push, PR, merge, publicação de pacotes, produção, nuvem, issues remotas, alteração de branch protection.
- Execução autônoma real (writer de agente real), fallback para host, credenciais reais, provedores externos pagos.
- Promover schemas/validator já existentes (commit `2313d8a`) a TCB sem a tarefa de onda correspondente e aprovação do owner.
- Remoção de código/dependências baseada só em auditoria estática; enfraquecer cobertura, assertions ou o gate completo.

### Gates esperados

- `bash docs/harness/bin/bootstrap.sh`
- `bash docs/harness/bin/doctor.sh`
- `bash docs/harness/bin/sensors.sh` (completo, sem argumentos) para completion claims; `sensors.sh quick` apenas como lane rápida
- `git diff --check`
- Review independente por task (dois FAILs consecutivos escalam ao humano)

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
- **Status**: active — close stale live metadata now that the bootstrap sprint
  work is on `main`, `HEAD`/`origin/main` are at `1aa14e5`, and PR #108 commit
  `e156810` is already contained in that history.

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
- Implementar `sensors.sh` — wrapper determinístico sobre `just ci` (fmt + clippy -D + testes paridade Linux + docs + MCP ref) + harness doctor + checks específicos de engram.
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
- `just ci` (paridade mantida)

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
