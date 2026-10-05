# Task H6 report — contexto curto, retomável e retenção explícita (Lane P)

Worktree `engram-improvement-lane-p`, branch `claude/improvement-lane-p`. Commits locais (sem push/PR/merge):
- `c7a8d71` feat(harness): keep live context short and history retained
- `a0f3dea` docs(harness): refresh progress last commit after H6

## O que foi implementado
- `docs/harness/progress.md`: 1.522 -> 109 linhas (87.559 -> 9.718 bytes). Campos compatíveis com doctor (SPEC<->progress, Active plan), seção "Retomada rápida (escopo, limites, última evidência)", tabela de tasks com links para o log, reconciliação required/advisory (exigida por `check-live-state.sh`), trilha de exclusão (known-issue grpc-transport; `sensors.sh`/doctor fazem grep dela em progress.md).
- `docs/harness/progress-history.md`: texto anterior de `progress.md` (de `4d341a0`) **byte a byte** após marcador `BEGIN-VERBATIM` (tamanho 87.559 + SHA-256, verificados por teste), com índice numerado das 68 seções H2 e âncoras estáveis. Nada apagado.
- `docs/harness/context-budget.md`: orçamentos, método de medição, antes/depois, retenção, regra de enforcement do live state, ordem de leitura/autoridade, **decisão #152**, verificação em clone limpo.
- `bin/measure-context.py`: mede o conjunto obrigatório (bytes/linhas/tokens) com tokenizer identificado e `--section ARQ#âncora` (seção inteira, sem cortar parágrafo). `bin/doc_links.py` + `bin/check-doc-links.py`: checker offline de links relativos e âncoras (regras do GitHub, `-1` em duplicatas, ignora fences/código inline, alvo ignorado por git = quebrado).
- `bin/check-live-state.sh --structural` (+33 asserções no `test-check-live-state.sh`, eram 19) e `doctor.sh`: roda o modo estrutural no `progress.md` real, falha se `progress.md` >150 linhas e se bootstrap >50 linhas (antes 60); novas checagens de arquivos/fiação.
- Lane offline: novo componente `context_budget` (`test_context_budget.py`, 44 testes, piso exato 44), 8 componentes; `test_offline_lane.py` (13 testes) e doctor atualizados.
- `bootstrap.sh`: +2 linhas (Last review, ponteiro de retomada); contrato ≤50 linhas / <500 ms.
- Docs: `GATES.md` (seção H6, itens 7-8 da lane, esclarecimento do job `Harness Doctor Advisory`), `README.md` (tabela + nota de atualização), ADR (linha *Source*), auditoria E0 §9, entrada H6 (e ponteiro D7–D12 na entrada E0) no log da lane P.

## Medição (tokenizer identificado)
- Tokenizer: **`cl100k_base` via `tiktoken-rs 0.5.9`** (mesma crate/versão do `Cargo.lock`, a que `TiktokenCounter` usa; vocabulário embutido, offline), rodada por ferramenta descartável fora do repo (`cargo build --offline`). Proxy, não tokenizer de modelo específico. Python `tiktoken` NÃO está instalado; `measure-context.py` só o usa se importável e em cache (sockets bloqueados), senão `approx:utf8-bytes/4`, rotulado, `exact=false`, com motivo (aqui subestima ~6%: 66.436 vs 70.842). `--tokenizer tiktoken` falha em vez de aproximar.
- Conjunto obrigatório (14 arquivos, active plan resolvido): antes **265.725 B / 70.842 tokens**; depois **196.875 B / 53.151 tokens** (-25,9% / -25,0%). `progress.md`: 87.559/23.331 -> 9.718/2.987 (-88,9% bytes; era 33% do conjunto, agora ~5%). O log do active plan cresceu (33.466 -> 39.685 B), GATES +1,8 KB, README +0,9 KB.
- Retomada (leitura até `progress.md`): 51.220 -> 31.667 tokens.
- Bootstrap: já cumpria a meta. 20 execuções sequenciais antes: mediana 154 ms (40 linhas); intercalado (máquina mais carregada) antes 192 ms vs depois 198 ms (42 linhas). Sem ganho de tempo; custo ~6 ms. Meta agora é contrato testado.
- Duplicação AGENTS/CLAUDE medida: 864 B + 771 B (254 + 192 tokens), ~0,6% do conjunto.

## TDD
- RED: testes escritos antes (`python3 -m unittest discover -s docs/harness/tests -p test_context_budget.py`): 44 testes, 16 falhas + 6 erros reais (ex.: `AssertionError: 1522 not less than or equal to 150 : progress.md has 1522 lines (budget 150)`, sem seção de retomada, sem `measure-context.py`, sem `context-budget.md`). `test-check-live-state.sh` RED: `ERROR unknown argument: --structural`.
- GREEN: `Ran 44 tests ... OK`; `test-check-live-state.sh` -> `PASS ... (assertions: 33)`.
- Mutações manuais (restauradas) derrubaram os testes certos: editar o histórico (hash/índice/seção/links), +60 linhas no resumo, trocar ordem de leitura no `AGENTS.md`, link quebrado no `progress.md`.

## Outras verificações (HEAD `a0f3dea`)
- `bash docs/harness/bin/run-offline-lane.sh` -> `OFFLINE_LANE: PASS components=8 checks=313` (validator_unit 76, validator_self 18, fixtures 39, live_state 33, review_gate 36, lane_contract 13, sandbox_unit 54, context_budget 44).
- `bash docs/harness/bin/doctor.sh` -> `OK harness doctor` (1 WARN pré-existente: sem artefato de review da task ativa).
- `bash docs/harness/bin/check-live-state.sh --progress docs/harness/progress.md` (estrito) PASS; `--structural` PASS (`ancestor_check=ancestor`).
- `python3 -m unittest ... test_offline_lane.py` 13 OK; `shellcheck -x` limpo em bootstrap/doctor/run-offline-lane/check-live-state/test-check-live-state.
- **Clone limpo**: `git worktree add --detach <tmp> HEAD` (a0f3dea, 0 arquivos sujos): `test_context_budget.py` 44 OK, `test-check-live-state.sh` PASS (33), `test_offline_lane.py` 13 OK, checker de links PASS (3 arquivos, 1 `ALLOWED` pendente da lane R), doctor OK. Worktree removido e `git worktree prune`.

## Decisões / deviations
- **Enforcement do live state (deferido de H2)**: doctor roda `--structural` (campos, plano, review autoritativo, linhas de reconciliação, Last commit **ancestral** de HEAD); não exige HEAD exato nem timestamp de `.sensors-last` (churn). Clone raso: `ancestor_check=skipped-shallow` explícito (e ainda valida formato de id). A forma estrita continua fechando tasks. Documentado em `context-budget.md` §4 e `GATES.md`.
- **#152**: ruling do owner registrado (chunks por caracteres, distintos de tokens; nenhum tokenizer/modelo adicionado; `TokenChunker` sem chamador de produção; input para Q7). Sem implementação. Fatos verificados em `document_ingest.rs`, catálogo MCP e `token_counter.rs`/`mod.rs`.
- **Duplicação AGENTS/CLAUDE não removida** (decisão documentada): CLAUDE.md é autocarregado e AGENTS.md serve outros agentes; risco (divergência de ordem) coberto por teste. Ordem de leitura e autoridade inalteradas; D10: INVARIANTS raiz/STANDARDS não consolidados.
- **D4** (GATES required checks): reverificado, consistente com a proteção live (seis contextos) e com a tabela do `progress.md`; apenas esclarecido o job `Harness Doctor Advisory`.
- E0 minors: ADR *Source* corrigido; auditoria §9 registra ausência de Review Canvas no aceite do ADR (desvio); log da lane P aponta D7–D12.
- Arquivos fora da lista literal do brief: `check-live-state.sh`/`test-check-live-state.sh`, `run-offline-lane.sh`, `test_offline_lane.py`, `GATES.md`, `README.md`, ADR, auditoria (todos pedidos pelo controller); novos: `doc_links.py`, `check-doc-links.py`, `measure-context.py`.
- H3 intocado: um `chmod +x` acidental em `sandbox_registry.py` foi revertido para 644 antes de qualquer commit (`git status` sem mudança nele).

## NOT RUN / pendências / concerns
- `sensors.sh` quick/full NÃO rodados (reescrevem `.sensors-last` rastreado -> conflito de telemetria entre lanes, como em H2); doctor e lane rodaram à parte. A linha `Last sensors` do `progress.md` segue a de H2 (não fabricada).
- Rotação das entradas concluídas do log do active plan (maior arquivo não-normativo, ~11,9 mil tokens e crescendo por task) NÃO feita: arquivo anexado por outras tasks/edição concorrente; follow-up registrado em `context-budget.md` §3. Teste não impõe teto a ele; sugerido ao controller.
- Teste de tempo do bootstrap (<500 ms, mediana de 5) pode oscilar em runner muito lento (medido ~150-200 ms aqui).
- Python 3.14 local; usos de `removeprefix`/`Path.is_relative_to` exigem >=3.9 (ok no ubuntu-latest).
- Nenhuma revisão independente (controller). Nenhum push/PR/merge/CI real.

## Rollback
Reverter `c7a8d71` (e `a0f3dea`) devolve `progress.md` anterior, bootstrap/doctor e a lane de 7 componentes; o histórico segue preservado até lá. Links testados.

---

# Fix report — H6 review round 1

Commits: `2be03c1` fix(harness): downgrade unprovable live-state ancestry to a warning; `96c82c5` docs(harness): refresh progress last commit after H6 fix.

## 1. [Important] Structural ancestry no longer fails the required job
- `check-live-state.sh --structural`: only a **malformed** Last commit (not 7-40 hex) is a hard failure. A well-formed id HEAD cannot reach (squash/rebase rewrites SHAs; also diverged) now yields `ancestor_check=unreachable` + `WARN Last commit ... not reachable from HEAD (squash or rebase merges rewrite SHAs ...)` and exit 0. New `--require-ancestor` keeps the hard form (`ancestor_check=not-ancestor`), used only by hermetic fixtures. The real `progress.md` run inside the lane asserts only success (no ancestry assertion), so a squash merge cannot fail `Test (ubuntu-latest)`.
- New tests in `test-check-live-state.sh` (33 -> 40 assertions): unknown well-formed id warns (and fails with `--require-ancestor`), diverged commit hard only with flag, **real squash merge** in a fixture repo (`merge --squash`, Last commit = pre-squash tip) warns/exit 0, malformed id fails on full history, malformed still fails in shallow clone.
- RED evidence: after rewriting those tests the suite failed with `Last commit 1aa14e5 is not an ancestor of HEAD ... (unknown object, diverged or rewritten history)` (old behavior).

## 2. [Important] No silent pass in shallow clones
- doctor.sh `live_state:structural` now maps `ancestor_check`: `ancestor` -> pass; `skipped-shallow` -> `WARN: ancestry not verified (shallow clone)...`; `unreachable` -> `WARN ... not reachable from HEAD (squash/rebase merges rewrite SHAs, ...)`; anything else -> fail. The checker also prints a `WARN ancestry not verified (shallow clone)` line.
- New `DoctorLiveStateAncestry` tests in `test_context_budget.py` (4 tests) run the real `doctor.sh` in clones of committed HEAD (`--no-local --depth 1` and `--local`), rewriting Last commit only in the clone: shallow -> warning, unreachable on full history -> warning + exit 0, malformed -> exit 1 `is not a commit id` (even shallow), reachable parent -> no ancestry warning. RED before the commit: the two warning tests failed against the old doctor; GREEN after.

## 3. [Minor] folded in
- Bootstrap time: `HARNESS_BOOTSTRAP_MAX_SECONDS` wins; default 0.5 s locally, 2.0 s when `CI=true`; the ≤50-line budget stays hard everywhere (doctor + test). Documented in `context-budget.md` §1 and `GATES.md`.
- SHA-256 pinned as constants (`HISTORY_SHA256`, `HISTORY_BYTES`, `HISTORY_SOURCE_COMMIT=4d341a0`) in the test and stated in `context-budget.md` (a test asserts the doc contains it); marker, body and constant must all agree; when `4d341a0:docs/harness/progress.md` exists in git it is also re-hashed (silently skipped only when the object is absent, e.g. shallow clone; the constant still binds).
- Table now has an `approx:utf8-bytes/4` column for before/after (66,436 -> 49,224) and a "re-derive" procedure (worktrees at `4d341a0`/`c7a8d71` + `measure-context.py --root --tokenizer approx`; exact counter = throwaway crate `tiktoken-rs =0.5.9`, `cargo build --offline`, `cl100k_base().encode_with_special_tokens`).

## 4. [Attribution] #152
- Reworded to "Ruling do controller (2026-10-05), pendente de confirmação do owner" in `context-budget.md` §6, `progress.md`, and the lane-P H6 entry; a test asserts the section says controller/pending and no longer says "ruling do owner". (Earlier text of this report also said owner; superseded.)

## Other
- Lane floors: `context_budget` now 49 tests (floor exact 49 in `run-offline-lane.sh`, `GOOD_COUNTS` in `test_offline_lane.py`); header comments updated. Re-read both files immediately before editing; H4 had not touched them yet. H4's untracked/modified files were not staged.
- Lane-P log: H6 entry wording fixed (#152, enforcement) and a "H6 — rodada de correção 1" entry appended.

## Verification (HEAD 96c82c5)
- `test_context_budget.py`: 49 OK; `test-check-live-state.sh`: PASS (40 assertions); `test_offline_lane.py`: 13 OK; shellcheck clean (doctor, run-offline-lane, check-live-state, test-check-live-state).
- `run-offline-lane.sh`: `OFFLINE_LANE: PASS components=8 checks=325` (live_state 40, context_budget 49/49, others unchanged).
- `doctor.sh`: OK (1 pre-existing WARN). Strict `check-live-state.sh --progress docs/harness/progress.md`: PASS (Last commit `2be03c1` = HEAD~1).
- NOT RUN: sensors.sh; independent re-review.
