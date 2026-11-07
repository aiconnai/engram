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

## Resolução consolidada local (2026-10-29)

Execução local em `fix/resolve-all-2026-10-29` (sobre `21cf80e9`, `origin/main` `2e84a594`) do
[plano consolidado](../plans/2026-10-27-repository-execution-plan.md) e dos 5 lotes da auditoria
`/code-review-rm`, mantendo todos os limites estritamente locais (sem push, PR, merge, tag,
release, publicação em registries, remoção de branches/worktrees ou inferência paga):

- **R1 (`src/text_util.rs`)**: `floor_char_boundary` / `ceil_char_boundary` delegados à stdlib e
  `LowercaseIndex::find_window` reescrito com `split_first()`. `cargo-mutants --no-config --file src/text_util.rs`
  passou com **42/42 mutantes detectados, 0 missed, 0 timeouts**.
- **R3 (release tag verification fail-closed)**: adicionados [`allowed-signers`](../../releases/allowed-signers),
  `scripts/verify-release-tag.sh`, materialização de `origin/main:scripts/verify-release-tag.sh` em
  `.github/workflows/sdk-release.yml` e 14 testes herméticos em `scripts/test_sdk_release_workflow.py`
  (recusa allowlist vazia/placeholder, substituição do verificador na tag e auto-autorização pelo commit da tag).
- **R4 / R5 / R7**: empacotamento local conjunto `cargo package -p engram-types -p engram-core --registry crates-io --offline`
  PASS; artefatos SDK `0.7.0` + contrato `--live` revalidados no SHA final; claims `"100% offline"`
  qualificados em `README.md` e `docs/index.html`; documentos de levantamento/plano de `2026-10-27`
  incorporados e reconciliados com [`v0.24.0-readiness.md`](../../releases/v0.24.0-readiness.md) e
  [`2026-10-27-nightly-release-readiness.md`](../audits/2026-10-27-nightly-release-readiness.md).
- **C1 (portabilidade Markdown RFC 0004)**: CLI `engram-cli import markdown`, `ImportOptions::default()`,
  resposta MCP (`"dry_run"`), SDK Python (`vault_import(..., confirm=False)`) e SDK TypeScript (`confirm?: boolean`)
  alinhados ao modo seguro de review/dry-run por default.
- **H1 / Q1 / H2**: reconciliados `docs/harness/risk-register.yaml`,
  [`2026-10-26-deferred-review-minors.md`](../known-issues/2026-10-26-deferred-review-minors.md) e
  [`retrieval-performance-policy.md`](../../quality/retrieval-performance-policy.md); `merge_gate_ci.py` e
  `merge-gate.py` endurecidos para falhar fechado (`CiRefusal`) quando `git show` da base falha
  (`FLOOR_MERGE_GATE=50`), documentados no [canvas](../canvas/2026-10-29-resolve-all.md).
- **S1 + Unidades 1–5 (runtime, pool SQLite, sync, lifecycle/retenção, consolidação e ingestão)**:
  - `src/bin/server.rs` (timeouts HTTP sem leak, hook budget/truncation UTF-8, `workspace` scoping),
    `src/storage/connection.rs` (`available_connections`, recovery de pool, cleanup de sidecars WAL/SHM),
    `src/storage/active_artifacts.rs` + `src/sync/worker.rs` + `src/sync/cloud.rs` (dedup/cap de
    `last_error` em crash-loop, guarda contra overwrite de DB ativo e `TempDownloadGuard` RAII) e
    `src/multimodal/process.rs` (retry em `EINTR`).
  - Anti-IDOR em 9 tools MCP by-id (`src/mcp/workspace_guard.rs` + handlers de lifecycle/policy +
    [`workspace-operation-matrix.md`](../../security/workspace-operation-matrix.md)), validação estrita e
    escopo por workspace em `src/storage/queries/retention.rs`, e alinhamento 4-way MCP/SDKs
    (`check-sdk-contract-alignment.py` com 0 drift).
  - `src/intelligence/consolidation_offline.rs` + `src/mcp/handlers/compression.rs` (timestamps
    RFC3339 ms válidos, `preview_with_strategy` puro em `dry_run`, savepoint por grupo, filtro de
    `word_sets` vazios/`transcript_chunk`/expiradas) e `src/intelligence/document_ingest.rs`
    (chunking por índice de `char`, Markdown multi-bloco/tabelas, dedup intra-documento e buffer UTF-8
    por página em `BoundedPdfText`).
- **Revisão independente `R2` + Harness Script Guard**: subagente read-only independente revisou todo o
  intervalo `origin/main` (`2e84a594`) até os commits `6aed4f28..56458055` em 3 rodadas e concluiu com
  veredito **`PASS`** (0 achados abertos; **`PASS`** em `docs/harness/bin/*`). Parecer de agente não
  substitui sign-off/receipt humano antes de merge.
- **Pendências exclusivas do owner (em 2026-10-29)**: provisionar chave SSH pública real em
  `docs/releases/allowed-signers`, autorizar push/PR/merge para `main`, emitir receipts humanos de
  post-gate e criar tag assinada `v0.24.0` para disparo de publicação nos registries.

## Provisionamento de signer SSH e abertura de PR (2026-10-30)

Novo GO explícito do owner no chat (`2026-10-30`):
1. Provisionar a chave pública SSH ativa do owner (`ron@ldinho.com.br`,
   `SHA256:iinTT0HVuDeNCuJab1vXYI8fvRc/IPvR2Xx3LYhJLXA`) em
   [`docs/releases/allowed-signers`](../../releases/allowed-signers) — commitada em `82f79bed` e
   validada por `scripts/test_sdk_release_workflow.py` (**14/14 PASS**, cobrindo recusa fail-closed
   para arquivo só com comentários e `ssh-keygen -Y match-principals` no trust store commitado).
2. Executar `git push -u origin fix/resolve-all-2026-10-29` e abrir PR contra `main` para validação
   do CI remoto e revisão humana.
3. **Limites mantidos**: merge do PR, criação de tag/GitHub Release (`v0.24.0`), publicação em
   crates.io/PyPI/npm, remoção de branches/worktrees e inferência paga seguem **não** autorizados.

## P0 export Markdown autorizado — 2026-11-01

Owner autorizou seguir no hotfix separado, sobre origin/main revalidada `2e84a594`, branch local
`fix/markdown-export-safety`. Tag 0.24.0 aguarda resultado; sem push/PR/merge/tag/publicação ou dados
reais autorizados. [Log](2026-11-01-markdown-export-safety.md),
[canvas](../canvas/2026-11-01-markdown-export-safety.md), RISK-0010 open. Checkout principal e
documentos do programa de evolução não foram incorporados ao hotfix. Regressores RED e candidato
local corrigido em verificação; revisão nova/post-gate pendentes, sem aprovação retroativa.


P0: full final PASS em `2026-11-01T09:02:00Z`, 515 s, sem exclusões, depois de dois achados
de revisão (ACL macOS e timestamps) reproduzidos/corrigidos. 78 testes Markdown e 23 integrações
passam. Parecer estático v2 sem novo defeito concreto não é aprovação formal ou receipt humano.
RISK-0010 open até integração/fechamento; sem tag/push/PR/merge. Commit local em preparação.


Código P0 commitado localmente em `0cd2d2d562bb514ac9870a1e2e8e3f657d9feaa4`; hook passou habilitado.
Quinze blobs do commit correspondem ao snapshot estático v2; live-state estrito PASS e worktree
limpa antes do fechamento documental. Sem push/PR/merge/tag; formal post-gate/receipt e integração
no candidato de release continuam pendentes, RISK-0010 open.


### Follow-up Markdown, 2026-11-01

Export local `9bca4efa`: tags codificadas, sincronização por lote, decoder público, testes RED/GREEN,
review estático com um achado corrigido e 11 hashes reconciliados; Linux nativo 81+22 PASS (1 probe
manual ignorado), sensores full PASS/459 s sem exclusões. Log próprio markdown-portability-followup.
Import RISK-0011 em branch própria; tag aguarda bloqueantes, sem push/PR/merge/publicação.


### Candidato separado import, 2026-11-01

RISK-0011: guard de raiz/mode, reader ancorado/limitado sem links, autorização por workspace persistido
com classificação/aplicação no mesmo TX. RED2PASS/12FAIL antes da correção; review detectou parser/tab
e resultado pós-commit, ambos corrigidos com reprodução e GREEN. 100 testes Markdown + integrações
G1/vault/routing verdes; Linux arm64 100+25 PASS (probe/helper declarados). Full PASS/422s sem exclusões,
CI complementar13PASS; baseline documental de ENGRAM_EXPORT_BASE_DIR atualizado sem supressões.
Log próprio 2026-11-01-markdown-import-safety, outputs crus e hashes. Candidato local sobre9bca4efa;
formal post-gate/receipt humano e integração pendentes. Nenhum merge/tag/publicação nesta retomada.


## Candidato integrado local, 2026-11-01

Integração646636f6 +1faa5c6f mantém79889460, todos os fixes da remediação e o hardening Markdown.
Raiz/IDOR/unknown preservados; reader omite metadados e lixeira ocultos, MCP responde dry_run.
Testes focais103+CLI3/G1/vault e SDKs verdes, gates do combinado ainda pendentes.
[Log próprio](2026-11-01-markdown-release-integration.md); formal post-gate/receipt humano pendentes,
sem publicação/merge de main/tag. Planejamento sujo no checkout principal permanece preservado.


### Verificação do combinado9040153b

Full novo PASS em2026-11-02T00:19:12Z (01/11 local),422s, sem exclusões; executou sobre o merge
commitado9040153b, não sobre uma branch isolada. CLI real3PASS/7invocações (semconfirm não altera
estado lógico; confirm aplica2; flagsconflitantes recusam antes doDB). ComplementoCI14comandosPASS.
Linux arm64/Rust1.97.1:103Markdown+29integraçõesPASS (probe/helper explícitos), SDKPython2 eTS4+types.
Review estático corrigiu perda de história documental; formal post-gate/receipt humano permanecem
pendentes. Handoff altera só metadados, sem fonte executável nova depois dos gates.


## Markdown portability architecture — 2026-11-02

Owner approved local implementation after Q1–Q3/design confirmation. Isolated
branch refactor/markdown-portability-core based on integrated60ed4eec. Preserve
contracts and safety; [task evidence](2026-11-02-markdown-portability-core.md).

Markdown architecture: implementação0da56ffc em branch própria, full PASS410s
(2026-11-02T16:51:50Z),112+20 testes focais PASS, duas opiniões estáticas sem
achados. [Evidência](2026-11-02-markdown-portability-core.md). Post-gate formal
PENDING por revisão/receipt ausentes; publicação não ampliada.


Markdown handoff (owner: "fix all",2026-11-02): persistido o parecer Terra6.1/high
PASS e override do Sonnet. Produto igual a0da56ffc; nenhum defeito novo identificado.
Pendência do gate agora é receipt humano; procedimento guiado externo preparado.
[Estado atual](2026-11-02-markdown-portability-core.md#current-handoff-status).

## Cloud download cancellation — 2026-11-04

Task: cloud-download-cancellation; base d9915e992c375751c3b88c5f5577f113b188a4bb.
PR #264 is published; all six required checks and ten PR workflows passed on that
base. Ronaldo's renewed H1 receipt covers60ed4eec..d9915e99, verified on2026-11-03;
it does not authorize a later HEAD or merge. Historical handoff entries above retain
their original dates and pending states.

The requested xhigh two-axis review of main2e84a594..d9915e99 found Standards0 and
Spec1MED: cancelling a queued Tokio file open can outlive the future-owned cleanup
guard and create an orphan temporary afterward. Owner requested this fix in chat.
[Plan](../plans/2026-11-04-cloud-download-cancellation.md) and
[canvas](../canvas/2026-11-04-cloud-download-cancellation.md) define the bounded change.

The regression exercises the real offline CloudStorage download with one blocking
worker and channel barriers, including completion of create_dir_all before queuing
file preparation. No sleeps, real credentials or remote object store are used.
RED: the new queued-open regression ran exactly one test and failed on the orphan
`.download-*.tmp` (0.01s). GREEN: the same test passed after the ownership fix;
all37 cloud tests then passed (0.65s), including queued preparation, an undelivered
prepared result and a commit already queued at cancellation. The last case may
finish a complete replacement; the first two preserve the original destination.

Preparation and commit now own the guard inside blocking work or its result;
file handles close before cleanup, and ownership starts only after exclusive
creation. The existing downloaded Vec is moved, not copied. Public signatures,
mode preservation, encryption and SQLite guards remain unchanged.

An initial unsupported `--no-default-features --features cloud` compilation failed
on the existing reqwest import; no behavior result is claimed for it. Focused
verification uses defaults plus cloud. Cloud all-targets Clippy and formatting
passed. Full sensors PASS on2026-11-04T19:04:50Z in389s, without exclusions:
required-feature Clippy/tests, server/watcher, WASM, rustdoc, MCP reference,
offline harness lane, PR-title fixtures and doctor. The existing warning about
the broader remediation review remains; negative title fixtures intentionally
print FAIL while the aggregate result is PASS. The tracked-tree secret scan passed.

Independent xhigh static review returned PASS on the working patch. Source hashes
still match that snapshot: cloud.rs fc2588322f14a97f8ce4e10bfeea722f51af4d6e1e2d4f00c7040710f910288d;
cloud_tests_failure.rs 7126d98a2f6d19f13b4196b0b66076a8e961daa079da412a61dd81850b05bdcb.
These are static-review/test observations, not a final-commit approval or receipt.
After freezing, rebind the review to the actual HEAD and preserve the opinion plus
the human-only completion procedure in ignored local artifacts. The new receipt
and remote CI are pending at this committed handoff; merge/release remain separate.

## Release validation and SDK dry-run — 2026-11-05

Task:sdk-release-dry-run. Owner authorized the three post-merge validation steps.
Main7ca7a9b8 contains PR264, merge explicitly approved despite disclosed H5 refusal;
the cloud H1 receipt was separately verified for d9915e99..3bffe826. No H5 eligible
verdict or broader receipt is inferred. Historical pending entries remain snapshots.

Main CI and both live SDK workflows passed. Manual Nightly41966511833 and SDK
dry-run41966515107 bind to7ca7a9b8. Both Rust crates passed verified offline packaging
from a clean target; registry/channel observations match the matrix at04:16:20Z.
The SDK dry-run failed on two confirmed release-only paths. The bounded candidate
fixes parser compatibility and artifact-path resolution; real RED/GREEN reproductions
and15 passing workflow tests are recorded in the
[audit](../audits/2026-11-05-release-validation.md). Full fix sensors passed at2026-11-05T04:42:56Z in409s without exclusions,
after repairing the required negative-scope link and retaining the initial failure.
Real local package installs/live checks and104 TypeScript tests also passed.
Nightly mutation outcome, frozen review/H1 and corrected remote dry-run remain
pending. No release operation.

## Release closeout — 2026-11-07

Owner requested execution of the updated assessment. New isolated branch
fix/release-closeout-2026-11-07 starts at SDK fix2c7544db over main7ca7a9b8;
old worktrees/planning files are preserved. [Plan](../plans/2026-11-07-release-closeout.md),
[canvas](../canvas/2026-11-07-release-closeout.md),
[evidence](../audits/2026-11-07-release-closeout.md),
[evolution v4](../plans/2026-11-07-engram-evolution-plan-v4.md).

The07/11 assessment confirmed full manual Nightly41966511833 terminal PASS with
seven successful jobs including mutations, plus current CI/Standing and live SDKs
on main. Scheduled Nightly skipped mutations; scopes are preserved separately.
The SDK remote dry-run remains failed and its local correction is not upstream.

Independent static Terra/xhigh inspection found CDPATH contamination in the new
artifact-dir canonicalization. Existing test reproduced failure with CDPATH=.;
the extended real-npm regression failed for dot and a competing search directory.
Clear CDPATH for that cd; full15 workflow tests then passed. Packaging floor and
publication guards unchanged. The new bytes need new review/H1, not the old bundle.

Step6 architecture/template cleanup integrated selectively; evidence/plan/risk
states reconciled without replacing historical reviews. Main Markdown fixes are
integrated; publication and formal risk closure remain separate. Release notice
preparation includes cache-clear plus Markdown import/export changes. No notices,
tag, registry upload, GitHub setting or deletion performed. Final local checks
and human-only completion are recorded in the evidence report as they finish.

Closeout checks: Python wheel/sdist+Twine PASS; TypeScript type/build/104tests PASS;
artifact-backed installed verifier PASS (Python happy1PASS/2unselected, npm CRUD,
wrong-auth and cross-workspace negatives). Initial sandbox socket/cache/Git-link
limitations were rerun with the required permissions, not exclusions or code fixes.
The first full sensor failed545s at15:34:14Z only because Last evidence lacked its
required active-log link. Link restored; focused50 context tests PASS. Keep the
first fail in sensor history and execute the full retry. Fresh registry checks PASS.

Full retry finished PASS at2026-11-07T15:46:02Z,417s, without exclusions, after the
evidence pointer fix; all deterministic stages/doctor passed. The existing review
warning and negative-title fixture failures are expected outputs, not skipped gates.
Final metadata/review/commit binding follows; no human receipt or remote success
is inferred from this local PASS.
