# Review Canvas: harness-script-fixes

Date: 2026-10-26
Owner: Ronaldo / agente WT5 (sprint `Remediação pós-levantamento 2026-10-26`)
Scope: Corrigir três defeitos de `docs/harness/bin/*` achados no levantamento de 2026-10-26: autoridade de review hardcoded em `check-live-state.sh`, contador de tools MCP do `bootstrap.sh` e grafia `Claude.md`; subir os pisos exatos do tripwire na lane offline (`live_state` 40 para 44; `context_budget` 49 para 50, por um teste novo do contador MCP).

## Trigger

| Trigger | Evidência |
|---|---|
| Mudança em scripts do harness (`bin/*` é process-critical, GATES "Harness Script Guard") | `check-live-state.sh`, `test-check-live-state.sh`, `bootstrap.sh`, `quarterly-audit.sh`, `run-offline-lane.sh` (pisos), `tests/test_offline_lane.py` (espelho dos pisos) e `tests/test_context_budget.py` (teste do contador MCP) |
| Mudança em bootstrap e checagem de live state | autoridade de review e contador de tools impressos a toda sessão |
| `WHAT_WE_DONT_DO.md`: `bin/*` não vira autoritativo sem post-review independente ou sign-off humano | **Pendente**: este canvas é evidência, não aprovação. Nenhum receipt foi emitido. |

Sem ADR: nenhum invariant, gate ou política muda de significado (INVARIANTS exige ADR só para mudar INVARIANTS). A regra "Last review precisa ser a review PASS autoritativa" continua; muda só **de onde** a autoridade vem. Precedente: o canvas `2026-07-09-engram-10-of-10-live-state.md` cobriu a criação do mesmo checker.

## Approaches Considered

| Abordagem | Decisão | Motivo |
|---|---|---|
| (a1) Derivar a autoridade do campo `Last review`: slug vem do nome do arquivo, exige a versão mais nova do slug (data, depois `vN`) e, se a `Active task` já tem reviews `post`, exige a mais nova delas | **Aceita** | Preserva as duas proteções originais (review de outra tarefa; versão superada) sem nome de tarefa hardcoded; o bootstrap passa a imprimir `Last review` coerente com a tarefa ativa |
| (a2) Manter a constante `engram-10-of-10-live-state` e trocá-la a cada sprint | Rejeitada | Reproduz o defeito: o campo ficou apontando para julho por ser a única review aceita (`bootstrap` imprimia "Last review 2026-07-10" ao lado do veredito de 2026-10-23) |
| (a3) Exigir que `Last review` seja a review `post` mais nova do diretório inteiro | Rejeitada | Cria churn cross-task: qualquer review mergeada sem refrescar `progress.md` derrubaria `doctor.sh` e o job required `Test (ubuntu-latest)`; H6 evitou esse churn de propósito |
| (a4) Remover a checagem de autoridade | Rejeitada | Enfraquece o gate (WHAT_WE_DONT_DO) |
| (b1) Aceitar `=> {` opcional antes de `cfg!(` na regex do contador | **Aceita** | Mudança mínima; resultado conferido contra a contagem manual (301 − 45 gated = 256) |
| (b2) Contar com `cargo`/`cfg` real | Rejeitada | Bootstrap é read-only, < 500 ms, sem compilar |
| (d) Subir `FLOOR_LIVE_STATE` de 40 para 44 e `FLOOR_CONTEXT_BUDGET` de 49 para 50, com o espelho em `GOOD_COUNTS` | **Aceita** | Pisos são contagens exatas (comentário em `run-offline-lane.sh`); subir é normal e deixar 40 permitiria apagar 4 asserções em silêncio |
| (c) Trocar `Claude.md` por `CLAUDE.md` no bootstrap e no `quarterly-audit.sh` | Aceita | O arquivo é `CLAUDE.md`; em filesystem case-sensitive (Linux/CI) o `rg` do audit ignorava o arquivo |

## Hot Path Complexity

| Path | Impacto de tempo | Impacto de espaço | Notas |
|---|---|---|---|
| `check-live-state.sh` (rodado por `doctor.sh` e pela lane offline) | +2 `find` rasos em `docs/harness/reviews` (≈150 arquivos) e um laço de ordenação | O(n) arquivos, sem I/O de rede | Sem novo subprocesso pesado; `sort`/`cut`/`tail` apenas |
| `bootstrap.sh` | regex com grupo opcional adicional | nenhum | Orçamentos de 50 linhas e 500 ms preservados (`test_context_budget.py`) |

## Edge Cases

| Caso | Verificação |
|---|---|
| `v10` ordena depois de `v2` (ordenação lexical erraria) | `test-check-live-state.sh`: repo hermético com `alpha-task-v1/v2/v10`; `v10` passa, `v2` falha apontando `v10` |
| Tarefa ativa tem review própria, `Last review` aponta outra tarefa | caso `beta-task`: falha `stale for the active task` |
| Tarefa ativa sem review (início de sprint) | caso `omega-task`: aceita a última review PASS de tarefa anterior; o bootstrap mostra `(no review artifact for active task: ...)` |
| `Active task` com sufixo (`id — texto`) | caso `beta-task — extra wording`: usa a primeira palavra, igual ao `bootstrap.sh` |
| Review com marcador `FAIL`, artefato `-pre.md` ou `.raw` | casos `gamma-task` (FAIL), `delta-task-pre` (rejeitado), `.raw` ignorado |
| Braces em arms `cfg!` (`=> {` / `}`) | contador devolve `256 active / 301 total` (3 tools `duckdb-graph` deixam de ser contadas como ativas); teste hermético `test_mcp_counter_treats_brace_wrapped_cfg_arms_as_feature_gated` falha contra o bootstrap antigo (`5 active / 8 total`) |
| bash 3.2 do macOS | suíte e checker rodados com `/bin/bash` 3.2.57 e com bash 5.3 |

## Breakage Risk

| Risco | Impacto | Mitigação | Verificação |
|---|---|---|---|
| Regra nova mais frouxa que a antiga | gate enfraquecido | cobre os mesmos modos de falha (revisão de outra tarefa, versão superada, sem PASS) e acrescenta (revisão de tarefa ativa, artefato não-`post`); `Last review` continua exigindo `REVIEW_VERDICT: PASS` | mutação: a suíte nova **falha** contra o checker antigo |
| Review `PASS` sem receipt humano aceita como "Last review" | falso verde de autoridade | o checker valida apenas existência, marcador e versão; **não** autentica o reviewer. Campo e plano rotulam parecer vs. post-gate com receipt | Texto de `progress.md` e nota de proveniência de `2026-10-14` |
| Integrador mescla branches com reviews novas | `Last review` stale após merge | só a regra "tarefa ativa tem review própria" e "versão superada" falham; checklist de refresh no plano `2026-10-26` | `check-live-state.sh --progress` após o merge |
| Piso do tripwire desatualizado | asserções apagadas sem alarme | piso exato 44 e espelho em `test_offline_lane.py` | `test_offline_lane.py` (19 testes) e `test-check-live-state.sh` (44 asserções) |
| Contador conta mal se um feature gated entrar em `default` | número divergente | limitação documentada: `default = ["openai"]` hoje; WT4 (`fix/mcp-surface-honesty`) é dono da superfície de tools | comparação manual 301 − 45 = 256 |

## Decision

Proceed, com os scripts marcados como **pendentes de post-review independente ou sign-off do owner** (WHAT_WE_DONT_DO; GATES "Harness Script Guard"). Nenhum receipt, reviewer ou PASS foi fabricado. Rollback: reverter os commits `fix(harness)` correspondentes devolve o comportamento anterior; nenhum dado é afetado.
