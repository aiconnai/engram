# Remediação pós-levantamento 2026-10-26 — plano

> Plano da sprint `Remediação pós-levantamento 2026-10-26`. O registro de execução e a evidência
> ficam no log ativo [`progress/2026-10-26-remediation.md`](../progress/2026-10-26-remediation.md)
> (o campo `Active plan` aponta para ele: INVARIANTS #4 e `test_context_budget.py` exigem um log em
> `progress/` com a última evidência). Escopo canônico: [`SPEC.md`](../SPEC.md).

## Autorização vigente e limites

- **Owner, chat, 2026-10-26**: "resolva tudo o mais paralelo possível". O integrador abriu 6 branches
  **locais** paralelas a partir de `08cb17a4` (= `main` local `b29cd16c` + `08cb17a4 refactor(storage):
  complete turso-sunset`). Ele mescla as 6 numa branch de integração, roda o CI completo e só então
  finaliza os SHAs em `progress.md`.
- **Não autorizados (aguardam decisão explícita do owner)**: push, PR, merge, tag, release,
  publicação, remoção de branches e worktrees. Também fora: produção, dados reais, chamadas pagas,
  mudança de settings do GitHub (branch protection, Dependabot, secret scanning).
- Autoridade anterior ("só commits locais ... programa ondas 0-4", 2026-10-05; sunset, 2026-10-23)
  é **histórica** e não vale como autorização desta sprint.

## Baseline e contexto

- Baseline: `08cb17a4`. Levantamento read-only de 2026-10-26: live state defasado (`Last review`
  apontando para julho, sprint do sunset ainda ativa), commits `d922a60e`..`b29cd16c` sem entrada de
  progresso, regressão de autorização em `e04d3307`, `Security Audit`/`Cargo Deny` vermelhos desde
  2026-10-17 (7 novos RUSTSEC), release pipeline fixo em `v0.22.0`, claims públicos divergentes.
- Programa anterior (`plans/2026-10-02-engram-comprehensive-improvement-plan.md`) encerrado:
  ondas 0-6 concluídas em 2026-10-14. Desvios herdados estão em "Desvios registrados".

## Frentes

| Frente | Branch (local) | Escopo | Aceite determinístico | Estado ao abrir a sprint (integrador atualiza) |
|---|---|---|---|---|
| WT1 | `fix/workspace-batch-auth` | Regressão de `e04d3307`: principal restrito escreve em workspace alheio via `memory_create_batch`/`memory_ingest_fact_batch`; minors de authz adiados (cache de resultado sem principal, `resources/subscribe`, tool desconhecida fail-open) | teste de regressão por tool de lote; `workspace_auth_enforcement_tests` | aberta |
| WT2 | `fix/supply-chain-2026-10` | 7 novos RUSTSEC; pins de ferramentas; Node 24 | `Security Audit` e `Cargo Deny` verdes sem renovar exceção de advisory corrigível | aberta |
| WT3 | `fix/release-pipeline` | `sdk-release.yml` fixo em `v0.22.0`; docs de versão | `check-release-channels.py`, `bump-version.py --check` | aberta |
| WT4 | `fix/mcp-surface-honesty` | gating de tools, contagens, status do rerank placeholder, claims públicos | `generate-mcp-reference.sh --check`, `bootstrap.sh` (256/301 hoje) | aberta |
| WT5 | `chore/harness-live-state` | estado vivo e governança do harness (esta frente) | `doctor.sh`, `check-live-state.sh` estrito, lane offline | esta branch; scripts aguardam review independente |
| WT6 | `docs/product-status` | status de RFC/ADR/plano, ROADMAP | `check-doc-links.py`, doctor | aberta |
| INT | (integrador) | merge das 6 branches, CI completo, SHAs finais | `make ci`, sensors completos, `check-live-state.sh` | pendente das frentes |

Fronteiras de arquivo: cada frente só escreve no seu conjunto; conflitos esperados ficam em
`progress.md`, `SPEC.md`, `CHANGELOG.md` e `README.md` (resolvidos pelo integrador). Esta tabela é
plano e declaração de dono, **não** evidência de que as outras frentes terminaram.

## Entregas da frente WT5

1. Transição de sprint (SPEC/progress/plano) e autoridade atual em "Retomada rápida".
2. Campos vivos refrescados e entradas de log para `d922a60e`, `e04d3307`, `c9de7136`, `b29cd16c`.
3. Scripts: autoridade de review derivada de `progress.md`; contador MCP do bootstrap; `CLAUDE.md`; pisos
   do tripwire e teste do contador MCP (canvas [`2026-10-26-harness-script-fixes`](../canvas/2026-10-26-harness-script-fixes.md)).
4. Nota de proveniência do veredito `2026-10-14` e desvio de Review Canvas do programa de outubro.
5. `CLAUDE.md` (decisões e riscos), `just ci` e caminhos obsoletos; fechamento do plano antigo e de D1/D7/D9.
6. Triagem dos minors adiados em [`known-issues`](../known-issues/2026-10-26-deferred-review-minors.md) e no `risk-register.yaml`.

## Checklist do integrador após o merge

1. `progress.md`: `Last commit`, `Last sensors` (do `.sensors-last` real da execução completa),
   `Last review` (se houver review da tarefa ativa, o checker passa a exigir a mais nova) e
   `Last live-state check`. Rodar `check-live-state.sh --progress docs/harness/progress.md` (estrito).
2. Trocar o estado das frentes por SHAs de merge; registrar a evidência de CI no log ativo.
3. Reconferir contagens de tools citadas em docs contra `bootstrap.sh` depois do merge de WT4.
4. Fechar ou atualizar em `CLAUDE.md` (Active Risks), `risk-register.yaml` e `known-issues` os itens de
   WT1/WT2 com o SHA que os resolveu; hoje todos estão "aberto" ou "em andamento".
5. `doctor.sh` limpo, lane offline (`run-offline-lane.sh`) e `git diff --check`.

## Decisões pendentes do owner

| # | Decisão | Bloqueia |
|---|---|---|
| 1 | Caminho de push/PR (um PR por branch, PR de integração ou outro) | tudo que depende de `origin` |
| 2 | Publicar `docs/integrations/decision-triage.md` | `c9de7136`/`b29cd16c` seguem só locais |
| 3 | HNSW como default vs RFC 0003 | fechamento do relatório de benchmark e do status da RFC |
| 4 | Federação: construir ou retratar o claim público | WT4/WT6 mantêm o claim marcado |
| 5 | Tag/release `v0.24.0` (SDKs `v0.7.0`) | depende de WT3 e do passo 1 |
| 6 | Remover worktrees e branches antigos | limpeza; nada foi removido |
| 7 | Revisão dos 7 `.raw` marcados como sensíveis que seguem no histórico Git | decidir entre aceitar o risco e reescrita de histórico (exige aprovação do lead eng) |
| 8 | Settings do GitHub: branch protection (D8), Dependabot, secret scanning | controles técnicos hoje inexistentes |
| 9 | Review independente ou sign-off dos scripts de `docs/harness/bin/*` alterados por WT5 | scripts não são autoritativos antes disso |
| 10 | Aceitar, alterar ou rejeitar o ADR C4 (SQLite descriptor-bound open), ainda Proposed | risco ativo no `CLAUDE.md` |
| 11 | Receipt humano do post-gate do `turso-sunset` | post-gate segue `PENDING` |

## Desvios registrados

- **Review Canvas ausente no programa de outubro** (2026-10-05..14): nenhum canvas em
  `docs/harness/canvas/` cobre H1-H6 (scripts process-critical), C1/C2, `v49` nem o aceite do ADR de
  hardening, embora GATES exija canvas para essas mudanças. Registrado em
  [`audits/2026-10-02-improvement-baseline.md` §9](../audits/2026-10-02-improvement-baseline.md); não
  é preenchido retroativamente.
- **Sem `review-gate.sh post` com receipt** no programa de outubro: ver a
  [nota de proveniência](../reviews/2026-10-14-engram-comprehensive-improvement-post-provenance.md).
- **Invariante #7**: `d922a60e`, `e04d3307`, `c9de7136` e `b29cd16c` não atualizaram `progress.md`.
  Corrigido retroativamente no log ativo; o desvio permanece registrado.
- **Active plan em `progress/`**: o plano em `plans/` é linkado pelo log ativo, que é o alvo do campo.

## Rollback

Plano e logs são documentais. Reverter os commits `docs(harness)`/`fix(harness)` desta branch devolve
o estado anterior; nada de produto é alterado por WT5.
