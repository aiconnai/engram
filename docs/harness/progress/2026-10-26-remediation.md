# Remediação pós-levantamento 2026-10-26 — log de progresso

Plano: [`plans/2026-10-26-remediation-plan.md`](../plans/2026-10-26-remediation-plan.md). Escopo:
[`SPEC.md`](../SPEC.md). Este é o log da sprint e o alvo do campo `Active plan`; ele registra o que a
frente WT5 (`chore/harness-live-state`) verificou. O estado das frentes WT1-WT4 e WT6 é do integrador.

## Autoridade e limites

- Owner, chat, 2026-10-26: "resolva tudo o mais paralelo possível" → 6 branches locais paralelas a
  partir de `08cb17a4`; o integrador mescla, roda o CI completo e finaliza SHAs em `progress.md`.
- Push, PR, merge, tag, release, publicação e remoção de branches/worktrees **não** estão
  autorizados (aguardam decisão explícita do owner). [Negativo do harness](../WHAT_WE_DONT_DO.md).

## Registro retroativo de commits sem entrada de progresso

Esses commits não tocaram `progress.md` nem um log de sprint (INVARIANTS #7). A entrada abaixo é
posterior e só descreve o que o Git e os artefatos do repositório mostram.

| Commit | Data | Assunto | Evidência no repositório |
|---|---|---|---|
| `d922a60e` | 2026-10-21 | `fix(bench): resolve search benchmark review findings and align RFC 0003 report` | `benches/README.md`, `benches/search.rs` (+76/−27); sem código de produto. Sem artefato de review, sem receipt de sensores. |
| `e04d3307` | 2026-10-21 | `fix(mcp,search): harden search handlers, workspace auth, and SDK contracts` | 23 arquivos (+1132/−270): `src/mcp/handlers/search.rs`, `src/mcp/permission.rs`, `src/mcp/workspace_guard.rs`, busca/BM25/cache, SDKs Python/TS, `docs/MCP_TOOLS.md`, `tests/workspace_auth_enforcement_tests.rs` (+269). Mexe em superfície MCP e autorização: exigiria Review Canvas (GATES); não há canvas, review nem receipt. **Regressão de autorização** (ver abaixo). |
| `c9de7136` | 2026-10-23 | `docs(integrations): add shared decision-triage config and usage guide` | `decision-triage.json`, `docs/integrations/decision-triage.md`, `docs/integrations/README.md`, `AGENTS.md` (+1 linha). O corpo do commit afirma que nenhuma inferência paga foi feita e que o registro não autoriza gasto. Só local; publicar é decisão do owner. |
| `b29cd16c` | 2026-10-23 | `docs(integrations): document muse route for shared decision triage` | só `docs/integrations/decision-triage.md` (+15/−1). Só local. |

**Regressão de autorização em `e04d3307`** (achada pelo levantamento read-only de 2026-10-26;
diagnóstico e correção são da frente WT1, branch `fix/workspace-batch-auth`, **em andamento, não
corrigida nesta branch**): um principal restrito consegue escrever em workspace alheio via
`memory_create_batch` e `memory_ingest_fact_batch`. Hipótese de mecanismo a confirmar por WT1: o
commit trocou a coleta recursiva de `workspace` por leitura só do nível superior
(`requested_workspaces` em `src/mcp/permission.rs`), então workspaces por item de payloads em lote
deixam de ser verificados. Até o merge da correção, vale como risco ativo (`CLAUDE.md`,
`risk-register.yaml`).

Confirmações: `bd3cee13` (release `v0.24.0`, post-gate de outubro) tem entrada na lane P
(`progress/2026-10-05-improvement-lane-p.md`, "RELEASE & CLOSEOUT"). `08cb17a4` (sunset Turso) tem
log [`progress/2026-10-23-turso-sunset.md`](./2026-10-23-turso-sunset.md), canvas, pre-gate e dois
pareceres Sonnet (`v1` com marcador duplicado, inválido para o post-gate; `v2` com um marcador);
SPEC e `progress.md` foram atualizados no mesmo commit. O post-gate do sunset segue `PENDING`: não
há receipt humano.

## Evidência de sensores

- `.sensors-last` commitado em `08cb17a4`: `status=pass mode=full timestamp=2026-10-23T20:10:00Z
  duration_sec=310`, sem exclusões. `.sensors-log` termina com `2026-10-23T20:03:16Z full fail 395 s`
  (`offline_lane`: três testes de links de retomada, corrigidos) e `20:10:00Z full pass 310 s`.
  O receipt não carrega SHA de commit: foi gerado no candidato do sunset, antes das edições finais de
  metadados de progresso (o log do sunset registra o doctor e o teste de contexto repetidos depois).
- O parecer Sonnet `v2` não confirmou o término do segundo gate (log truncado em `offline_lane`);
  o integrador do sunset confirmou o exit 0 terminal. O leitor deve tratar o PASS como declaração do
  integrador, sustentada pelas linhas append-only de `.sensors-log`.
- `.sensors-log` não tem nenhuma linha entre `2026-09-03T13:11:41Z` (quick) e `2026-10-23`: os
  gates completos do programa de outubro (por exemplo `2026-10-05T19:07:34Z`, 328 s) existem só como
  prosa em `progress.md` na época ("receipts not committed").
- Nenhum sensor completo foi rodado para `d922a60e`, `e04d3307`, `c9de7136`, `b29cd16c` isolados, nem
  nesta sprint. A frente WT5 **não** rodou `sensors.sh` (ele reescreve receipts rastreados); o
  integrador roda o gate completo depois do merge.

## Entradas da frente WT5

Commits em `chore/harness-live-state` (SHAs finais: `git log chore/harness-live-state`; o
integrador os reescreve ao mesclar):

1. Canvas `2026-10-26-harness-script-fixes` (evidência, não aprovação).
2. `check-live-state.sh`: autoridade de review derivada do campo `Last review`; suíte
   `test-check-live-state.sh` com casos herméticos (v10 depois de v2, tarefa ativa com review própria,
   marcador FAIL, artefato `-pre`). A suíte nova falha contra o checker antigo (mutação conferida).
3. `bootstrap.sh`: o contador MCP passa a reconhecer arms `=> { cfg!(...) }` (256 ativas / 301, antes
   259); `CLAUDE.md` no lugar de `Claude.md` em bootstrap e `quarterly-audit.sh`.
4. Transição de sprint e campos vivos (este log, plano, `SPEC.md`, `progress.md`).
5. Demais itens: ver o plano, seção "Entregas da frente WT5".

Os scripts de `docs/harness/bin/*` alterados (itens 2 e 3) são process-critical e **não são
autoritativos** até haver review independente ou sign-off do owner (WHAT_WE_DONT_DO; GATES "Harness
Script Guard"). Nenhum reviewer, receipt ou PASS foi produzido por esta frente.

## Verificações locais da frente WT5

Preenchidas ao final da frente (ver "Verificação final" abaixo).

## Verificação final

Pendente.
