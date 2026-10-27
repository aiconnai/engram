# Remediação pós-levantamento 2026-10-26 — log de progresso

Plano: [`plans/2026-10-26-remediation-plan.md`](../plans/2026-10-26-remediation-plan.md). Escopo:
[`SPEC.md`](../SPEC.md). Este é o log da sprint e o alvo do campo `Active plan`; ele registra o que a
frente WT5 (`chore/harness-live-state`) verificou. O estado das frentes WT1-WT4 e WT6 é do integrador.

## Autoridade e limites

- Owner, chat, 2026-10-26: "resolva tudo o mais paralelo possível" → 6 branches locais paralelas a
  partir de `08cb17a4`; o integrador mescla, roda o CI completo e finaliza SHAs em `progress.md`.
- Push, PR, merge, tag, release, publicação e remoção de branches/worktrees **não** estavam
  autorizados em 2026-10-26. Push, PRs e merges foram autorizados em 2026-10-27 (seção Publicação). [Negativo do harness](../WHAT_WE_DONT_DO.md).

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

## Verificação final da frente WT5

Executado em `chore/harness-live-state` (2026-10-26), sem rede e sem `sensors.sh`:

| Verificação | Resultado |
|---|---|
| `bash docs/harness/bin/doctor.sh` | exit 0, `OK harness doctor`, 1 WARN esperado (`no review artifact found for active task`: a sprint ainda não tem review) |
| `bash docs/harness/bin/run-offline-lane.sh` | `OFFLINE_LANE: PASS components=12 checks=534`, todos os pisos exatos (`live_state` 44, `context_budget` 50) |
| `check-live-state.sh --progress docs/harness/progress.md` (estrito) | PASS com `Last commit` = HEAD antes do commit de refresh; repetido depois dele com `Last commit` = pai |
| `test-check-live-state.sh` | 44 asserções PASS com bash 3.2.57 e 5.3.15; a suíte nova **falha** contra o checker antigo |
| `test_context_budget.py` / `test_offline_lane.py` | 50 e 19 testes OK; o teste novo do contador **falha** contra o bootstrap antigo (`5 active / 8 total`) |
| `shellcheck -x` nos scripts alterados, `check-doc-links.py` nos docs alterados, `git diff --check` | PASS |
| `bootstrap.sh` | 42 linhas, `MCP tools (source): 256 active / 301 total` (conferido à mão: 301 − 45 tools sob `cfg!(feature)`) |
| `check-security-exceptions.py` com a config do CI | PASS (2 governed records) |

Não executado: `sensors.sh` completo (reescreve receipts rastreados; é do integrador), `cargo test`, smokes
com Docker real, qualquer escrita remota. O hook de pre-commit (`cargo fmt` + `clippy --all-features`) rodou em
cada commit; esta frente não altera código Rust.

## Achados fora do escopo (não corrigidos)

- `harness-risk-register.sh add` calcula mal o próximo id a partir de `RISK-0008` (aritmética octal com
  `0008`; de `0010` repetiria `RISK-0009`): `RISK-0009` foi criado com `--id` explícito. Lição em
  `ERRORS_AND_LESSONS.md`.
- `harness-risk-register.sh validate` falha até no arquivo de base com Ruby/Psych 4 (`updated_at` vira `Time`).
  A conferência foi feita com `YAML.safe_load(..., permitted_classes: [Time, Date])`: 9 entradas, scores consistentes.
- As duas correções são mudanças em `docs/harness/bin/*` (process-critical) e precisam de review independente.

## Integração (integrador, 2026-10-26)

Branch `remediation/integration` (worktree `.worktrees/rem-int`), base `08cb17a4`. Somente local: push, PR,
merge, tag, release, publicação e remoção de branches/worktrees seguem sem autorização do owner.

- **Onda 1** (merges sem conflito textual): `fix/supply-chain-2026-10` (WT2), `fix/workspace-batch-auth` (WT1),
  `fix/mcp-surface-honesty` (WT4), `fix/release-pipeline` (WT3), `docs/product-status` (WT6),
  `chore/harness-live-state` (WT5), `docs/claims-and-triage-trim` (WT8), `fix/hnsw-default-propagation` (WT7).
- **Correções pós-merge do integrador** (teste primeiro): fiação de `subscribe_as`/`unsubscribe_as` no servidor
  (`f53b75a0`); contador MCP do bootstrap a partir de `catalog::required_features` (`d76bceee`, 254/301);
  checks novos no job Documentation (`c8e1884d`); link intra-doc qualificado (`4134d28b`).
- **Gate que segurou** (`93fc8566`): o drift guard do merge-gate recusou `docs/contracts/advertised-surfaces.toml`
  num job obrigatório (política de CI fixada em `harness-hardening-v1`). A política não foi alterada; o check foi
  para o job não obrigatório `Advertised Contracts`. Proteger o baseline exige versionar a política (decisão do owner).
- **Revisão independente** (Sonnet, só leitura, não é receipt): WT1 revisado pelo integrador; R3 revisou WT2/WT4/WT7
  e `bin/*` sem achado crítico/alto. Achado médio (autoridade de review sem post da tarefa ativa) corrigido em
  `383b70a8`; achados baixos de HNSW encaminhados a WT9. R2: nada a portar de
  `claude/engram-improvement-plan-edb43d`; `wave4-todo24` (75a1e16e) descartável.
- **Onda 2**: `fix/security-followups` (WT10), `fix/log-redaction-sweep` (WT12),
  `docs/followups-and-harness-tools` (WT11), `fix/live-state-review-authority` (integrador),
  `fix/hnsw-followups` (WT9: `ENGRAM_HNSW_ENABLED`, reconcile com SQLite, compactação viva); docs/CHANGELOG em `7c2fc79c`.
- **Evidência**: `make ci` em `4134d28b` exit 0 (offline lane 12/534, 2319/2319 testes, docs/WASM ok);
  `cargo audit` 0 vulnerabilidades (aviso governado `ttf-parser`) e `cargo deny` ok em `c8e1884d` (lockfile igual
  depois); `make ci` em `932e9c8d` falhou só no `cargo doc` (link intra-doc), corrigido em `4134d28b`.
  `sensors.sh` não foi executado (receipt versionado; `Last sensors` segue 2026-10-23).
- **Padrões aplicados por decisão do owner (revisáveis)**: HNSW opt-in; federação marcada como não implementada;
  guia de decision-triage enxugado; remoção de SigV4a; DuckDB 1.5.6; reranker placeholder retorna erro;
  contagens de concorrentes sem fonte removidas.
- **Regressão pega pelo CI final** (`make ci` em `08ca8288`, 1 falha): o pré-check HTTP de `06666ae8` passou a aplicar o
  `_permission_mode` por chamada e devolvia HTTP 403 em vez de 200 + recusa no resultado (contrato
  `tests/canonical_journey`). Corrigido com teste primeiro: o pré-check de transporte deixa a auto-restrição para o
  dispatcher, que continua recusando. Links quebrados de `docs/GETTING_STARTED.md` (pré-existentes) corrigidos.

## Publicação (2026-10-27)

Owner, chat, 2026-10-27: autorizou o push e "um PR por frente"; escolheu a opção A (um merge por PR) e
autorizou o Claude a executar os merges, como exceção explícita à regra de merge humano só para esta
pilha: merge commit, só com os checks obrigatórios verdes no head exato, parar na primeira falha, sem
bypass de admin. GO de 5 h, até 09:35Z; tag, release, registries, deleções e settings ficaram de fora.

- **Pilha**: 19 PRs contra `main`, na ordem de merge. Simulação local antes do push (merge-tree com
  merge commits) limpa em todos os passos e árvore final idêntica à de `remediation/integration`.
  gitleaks no intervalo publicado: sem achados.
- **Cherry-picks para cada PR passar sozinho**: `cdcb093c` (= `4134d28b`, link intra-doc que quebrava o
  `Documentation`) no #247; `7a4357f3` (= `14a65a54`, recusa por `_permission_mode` volta ao resultado)
  no #255. Os commits originais mesclam como no-op no #262.
- **Ordem**: #246 primeiro (contém #244 e #245 e conserta Security Audit e Cargo Deny); depois cada PR foi
  atualizado com `main`, passou no CI e recebeu merge commit.
- **Atualização dos branches**: até o #253 pelo "Update branch" do GitHub. No #254 o GitHub acusou conflito
  falso (6 bases de merge, histórico cruzado); `git merge-tree` (ort) resolvia limpo e dava a árvore prevista
  na simulação. Do #254 em diante, cada branch foi atualizado localmente com merge-ort (primeiro pai = branch,
  segundo = `main`, a mesma forma do "Update branch") e enviado como fast-forward.

| PR | Branch | Merge no `main` |
|---|---|---|
| #244 | `fix/search-hardening-and-decision-triage` | marcado como mesclado junto com #246, 04:55Z |
| #245 | `codex/turso-sunset` | marcado como mesclado junto com #246, 04:55Z |
| #246 | `fix/supply-chain-2026-10` | merge commit `f383c837`, 04:55Z |
| #247 | `fix/workspace-batch-auth` | merge commit `f304bf51`, 05:17Z |
| #248 | `fix/mcp-surface-honesty` | merge commit `347324e8`, 05:34Z |
| #249 | `fix/release-pipeline` | merge commit `9203d8a1`, 05:53Z |
| #250 | `docs/product-status` | merge commit `489271ce`, 06:09Z |
| #251 | `chore/harness-live-state` | merge commit `8bfb79e4`, 06:24Z |
| #252 | `docs/claims-and-triage-trim` | merge commit `958bf5f5`, 06:42Z |
| #253 | `fix/hnsw-default-propagation` | merge commit `a197fcf8`, 07:00Z |
| #254 | `remediation/integrator-round-1` | merge commit `3e6687b7`, 07:18Z |
| #255 | `fix/security-followups` | merge commit `8b1eb9ab`, 07:33Z |
| #256 | `fix/log-redaction-sweep` | merge commit `2dca9f00`, 07:49Z |
| #257 | `docs/followups-and-harness-tools` | merge commit `d68f37ce`, 08:08Z |
| #258 | `fix/live-state-review-authority` | merge commit `88980be0`, 08:26Z |
| #259 | `fix/hnsw-followups` | merge commit `468e76a2`, 08:42Z |
| #260 | `remediation/integrator-round-2` | merge commit `976fbe8b`, 09:01Z |
| #261 | `docs/fix-getting-started-links` | contido no #262 (head `8ee32e87`); sem merge próprio para caber no prazo |
| #262 | `remediation/integration` | este commit entra por aqui (último da pilha) |

- **CI**: Security Audit e Cargo Deny voltaram a verde no `main` no push CI de `f304bf51` (após #247).
  Checks não obrigatórios que falharam durante a cadeia:
  - Python SDK installed-wheel live contract no #262 (1ª rodada, 04:32Z): conflito do `rustup` ao instalar `clippy` no runner; o re-run passou.
  - `Advertised Contracts` no #257, #258 e #259: o baseline das 9 variáveis documentadas pelo #257 só é regenerado em `127e3e89` (#260), como na integração. Exceção explícita, registrada em comentário no #257; voltou a passar no #260.
  - Runs de CI no `main` (push e agendados, incluindo um Nightly atrasado) foram cancelados durante a cadeia para liberar runners: cada árvore intermediária já tinha sido testada no PR. O CI do merge do #262 roda completo.
- **Live state**: `Last commit` = `976fbe8b` (merge do #260), primeiro pai do merge do #262.

## Follow-up Nightly e release (2026-10-27)

Novo GO do owner, chat: diagnosticar Nightly, corrigir defeitos confirmados e lacunas relevantes
de testes, resolver a proposta de verify-tag e preparar release e aviso de migração. Entregar
evidências e recomendação antes de publicar ou executar mudanças operacionais. Não renova
o GO histórico de push/PR/merge; sem tags de release, publicações, avisos enviados, deleções,
settings ou recibos humanos fabricados.

- Base: `2e84a59422b3659aaaf25c69dbe01b01ab321332`, checkout limpo; bootstrap e doctor executados.
- [Plano](../plans/2026-10-27-nightly-release-readiness.md),
  [canvas](../canvas/2026-10-27-nightly-release-readiness.md),
  [evidência](../audits/2026-10-27-nightly-release-readiness.md),
  [readiness](../../releases/v0.24.0-readiness.md).
- Bug confirmado: árvore de escopos achatada por ordenar reverso e depois consumir com pop;
  dois testes novos falharam antes do fix. Corrigida ordem; preservar órfãos e irmãos.
- Lacunas: Bearer vazio, diagnósticos, flags/caps/contadores de extração, filtros e persistência
  de auto-link. Miri mantém 20.000 casos nativos e interpreta prefixo de 256 do mesmo seed.
- `CHANGELOG.md`: remediação incorporada à seção 0.24.0 para entrar nas notas extraídas pelo
  workflow. Data é de preparação do candidato; publicação ainda pendente.
- Pre-gate manual gerou só prompt advisory (sem inferência paga). Review independente e
  receipt humano do candidato final pendentes; não há post-gate PASS desta sessão.

- Fechamento técnico local: make ci PASS (2351 testes, 2 skips preexistentes, offline 12/536);
  Miri PASS (5 filtros, 16 testes, texto 80 s); reexecução focada 18 caught / 0 missed /
  4 timeouts (um ramo redundante removido). O Nightly ainda exige disposição dos timeouts.
- Matriz reobservada integralmente em 2026-10-27T18:14:43Z (todas as verificações PASS).
  SDKs empacotados localmente; proposta de confiança de assinatura e aviso preparados.
  Recomendação: segurar publicação; integrar/revisar o candidato, resolver timeouts e
  provisionamento de assinatura, obter receipts humanos e dry-runs no SHA final.

- Preparação de artefatos encontrou defeitos adicionais confirmados: `--write-metadata npm`
  procurava SDK `npm` onde a matriz declara `typescript`; retries incluíam o próprio metadata
  no hash. Teste primeiro reproduziu ambos, helper corrigido, 12 testes + shellcheck PASS.
  CI repetido para incluir o fix; geração dos metadados reais passa nos dois canais.
- Instalação limpa dos SDKs PASS após corrigir o helper; artefatos vinculados ao SHA
  de origem dos SDKs `2e84a594` (árvore SDK inalterada). Republicação exige artefatos
  refeitos no SHA integrado final. Crates conjuntos empacotam; verificação de staging
  encontra erro interno do Cargo 1.96; fontes extraídos compilam com patch local de types.
