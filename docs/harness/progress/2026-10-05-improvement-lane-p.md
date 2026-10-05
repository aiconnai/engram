# Progress Log — Engram comprehensive improvement, Lane P (harness / governança / CI docs)

> Plano: [`../plans/2026-10-02-engram-comprehensive-improvement-plan.md`](../plans/2026-10-02-engram-comprehensive-improvement-plan.md)
> Log da Lane R (Rust/produto): [`2026-10-05-improvement-lane-r.md`](./2026-10-05-improvement-lane-r.md)
> — arquivo pertencente à lane R, criado na worktree dessa lane; este link só resolve após a integração das lanes.
> Branch desta lane: `claude/improvement-lane-p` (worktree `engram-improvement-lane-p`).

## Regras desta lane

- A Lane P é dona de `docs/harness/progress.md`, `docs/harness/SPEC.md` e deste log. A Lane R não os edita; esta lane não edita o log da Lane R.
- Entrega somente por **commits locais** na branch da lane. Sem push, PR, merge, publicação, produção/nuvem, issues remotas ou alteração de branch protection.
- Autorização do owner (Ronaldo, chat, 2026-10-05): ondas 0-4; ADR `agent-harness-hardening-v1` aceito; runners da Onda 4 somente com *fake writer*, offline; budget WAL 64 GiB configurável; `defer_embedding=true` enfileira em background.
- Cada task desta lane acrescenta uma entrada abaixo (ID da task, assunto do commit, verificações executadas, itens NOT RUN).

## Entradas

### E0 — Reconciliar estado, autorização e baseline — 2026-10-05

**Resultado**: inventário e reconciliação registrados em
[`../audits/2026-10-02-improvement-baseline.md`](../audits/2026-10-02-improvement-baseline.md).
Nenhum código Rust, MCP, storage, SDK, workflow, hook ou dependência foi tocado.

**Decisões do owner registradas (2026-10-05)**

1. **ADR aceito**: `docs/decisions/2026-07-21-agent-harness-hardening-v1.md` passou de *Proposed* para *Accepted* (data 2026-10-05, owner Ronaldo, fonte: autorização em chat do plano abrangente). O histórico do status *Proposed* foi preservado no próprio ADR. A aceitação não ativa nem aprova retroativamente código existente (schemas/validator do commit `2313d8a` seguem fora do TCB até a task de onda correspondente).
2. **Baseline de execução**: os 14 commits locais (`HEAD` `1952f3b` vs `origin/main` `949c963`, 14 à frente / 0 atrás em 2026-10-05) são a baseline. Permanecem locais (sem push). Re-verificado em 2026-10-05 por git somente leitura (`git fetch` + `rev-list`/`merge-base`) e consulta `gh api` somente leitura (compare `main...1952f3b` retorna 404: os objetos não existem no GitHub).
3. **Live state**: `SPEC.md` e `progress.md` agora apontam para este programa; *Active plan* = este log. Conteúdo do sprint anterior preservado como histórico.

**Fatos re-verificados**

- `origin/main` = `949c9634be28badea43ba2a2b5bcdf5d4c0dd358`; merge-base `HEAD`/`origin/main` = `949c963`.
- Árvores: `1952f3b` → `66cb1885bd4ed47a01d60d5e4733a2099cea2623`; `630dd26` → `e612d8ed2945e1a9d0f8d038066f8a05d2c49054`.
- Proteção live de `main` (leitura): required = `Format`, `Clippy`, `Documentation`, `Test (ubuntu-latest)`, `Security Audit`, `Cargo Deny`; `Security Gate` e `Harness Contract` não aparecem; `Test (ubuntu-latest)` depende de `security-gate` (cadeia transitiva, validada por `scripts/check-security-gate.py`: PASS).

**Discrepâncias que exigem ação (resumo; ver auditoria)**

| # | Discrepância | Owner / destino |
|---|---|---|
| D1 | `Security exception policy` falha hoje: 10 exceções expiraram em 2026-09-30 → `Security Gate` vermelho → `Test (ubuntu-latest)` bloqueado (transitivo) | Q5/Q6 (supply-chain) — decisão humana de renovar/remover |
| D2 | `cargo audit`/`cargo deny` locais (DB offline de 2026-10-03) falham por `RUSTSEC-2026-0285` (`rustls 0.23.36`), não ignorado | Q5/Q6 |
| D3 | Commit local `1fdffc5` removeu `--criterion` do passo de budgets do CI; `check-quality-budgets.py` sai com 2 sem ele | Q1 |
| D4 | `GATES.md` descreve `Security Audit`/`Cargo Deny` como advisory e cita `RUSTSEC-2026-0187/0185`; live os exige como required e o lock já tem `lopdf 0.42.0`/`quinn-proto 0.11.15` | H6 / lane P (política) — não alterada em E0 |
| D5 | Schemas/validator/fixtures (commit `2313d8a`) existem e passam 15/15 localmente, mas não estão em doctor/sensors/CI e vieram junto de código de produto (RFC 0010) num único commit | H2 (endurecer/validar), depois H3 — não são TCB |
| D6 | `check-live-state.sh` estava stale (Last commit `b7ecea9`, Last sensors); atualizado nesta task | E0 (feito) |

D7–D12 (default features `80a4df6`, proteção live sem review obrigatório, commits sem review/Canvas, AGENTS/INVARIANTS/STANDARDS sem ADR, e as duas decisões do owner sobre o fluxo do ADR) estão na tabela completa da auditoria, [§6](../audits/2026-10-02-improvement-baseline.md#6-discrepâncias-resolução-e-owner); esta tabela resume apenas D1–D6.

**Verificações (detalhe na auditoria §5)**: bootstrap exit 0; `doctor.sh` OK (1 WARN esperado: sem artefato de review para a task ativa); `check-live-state.sh` PASS após atualização dos campos; `sensors.sh quick` pass (39 s); `sensors.sh` completo (sem argumentos) **pass** (282 s; fmt, clippy, test_lib, test_integration, test_integration_watch, wasm x3, doc, ref_check). O sensor full **não** cobre D1/D2/D3 (budgets, exceções, audit/deny, Security Gate): verde dos sensores não implica CI verde. `.sensors-last/.sensors-log` não foram commitados.

**Não executado**: nenhum push/PR/merge; nenhuma escrita remota; `cargo audit` com fetch de DB (usado DB local em cache de 2026-10-03, `--no-fetch`); scanners CodeQL/Semgrep/Gitleaks/AgentShield (só rodam em CI).

**Rollback**: reverter o commit de E0 restaura SPEC/progress/ADR anteriores (ADR volta a *Proposed*); nenhum receipt é apagado.

### Q1a — Reparar o contrato CI/local do quality budget — 2026-10-05

**Commit**: `1addb94` `fix(ci): restore --criterion in historical baseline integrity lane`.
Escopo: somente Q1a (contrato CI/local e integridade histórica). **Q1b** (integrar
resultado atual aceito de Q7) permanece pendente e não é desta entrada.

**Causa (D3)**: o commit local `1fdffc5` removeu `--criterion` do passo de budgets
em `ci.yml`; `check-quality-budgets.py` sai com 2 sem ele, o que quebraria o job
required `Test (ubuntu-latest)`. Reproduzido por teste (argv literal do workflow
executado em subprocess: rc 2, "the following arguments are required: --criterion").

**Mudanças**: `ci.yml` restaura `--criterion benches/results/benchmark_results.txt`
(valor de `origin/main`) e adiciona o passo `--self-test-degraded`; `scripts/ci.sh`
ganha a lane (passo 3/6 e `./scripts/ci.sh quality-budgets`) com o mesmo argv, falhando
fechado sem `python3`; `ci-parity-check.sh` passa a executar os testes de contrato de
argv (não depende mais das strings legadas de Makefile, que já falhavam antes);
`docs/quality/retrieval-performance-policy.md` documenta a lane; novo
`scripts/test_check_quality_ci_contract.py` (20 testes). A lane é nomeada
**historical baseline integrity**: valida pisos/snapshot Criterion commitados, não é
desempenho do candidato. `--criterion` segue obrigatório; pisos e teto 1.15 intactos.

**Observação sobre o brief**: o comando de aceite do brief usa `benchmark_baseline.txt`
em `--criterion`, mas esse arquivo (formato `Baseline:` `nome: valor`) não tem linhas
`time:`; o checker sai 1 ("missing named hot path(s)"). O arquivo correto, como em
`origin/main`, é `benchmark_results.txt`; usado em todos os lugares.

**Verificações**: `python3 -m unittest discover -s scripts -p 'test_check_quality_ci_contract.py'`
20/20 OK; checker com `--self-test-degraded` e `benchmark_results.txt` pass;
`bash scripts/ci-parity-check.sh` PASS; `bash scripts/ci.sh` completo exit 0 (fmt,
clippy, lane, nextest 1927 passed/1 skipped, wasm, doc + MCP reference);
`doctor.sh` OK (1 WARN esperado: sem artefato de review); `sensors.sh quick` PASS.
`.sensors-last`/`.sensors-log` revertidos (não commitados).

**Não executado**: CI Linux no GitHub (sem push); medição atual de performance (Q7/Q1b).

**Rollback**: reverter `1addb94` (wiring); suspender a lane explicitamente não equivale a sucesso.

**Q1a — correção da revisão (round 1)**: commit `06e9005`
`fix(ci): run quality-budget contract suite from required job and ci.sh`. A suíte
`scripts/test_check_quality_ci_contract.py` só era chamada por `ci-parity-check.sh`
(sem invocador); agora roda no job required (logo após os passos de budget) e como
passo de topo em `scripts/ci.sh` (fora da função da lane, para evitar recursão).
Novos testes `GateWiring` detectam remoção/deslocamento/`|| true`; rótulos exigem
também "not candidate performance"; `ci-parity-check.sh` não depende mais de `rg`.
Verificação: suíte 27/27, parity PASS, `ci.sh` completo exit 0, doctor OK.

### H1 — Review gate fail-closed e diff completo — 2026-10-05

**Commit**: `dfd8681` `feat(harness): make review gate fail-closed with explicit scope`
(arquivos: `docs/harness/bin/review-gate.sh`, `docs/harness/bin/test-review-gate.sh` novo,
`GATES.md`, `CODE_REVIEW_POLICY.md`, `README.md`). Nenhum Rust, MCP, storage, SDK, workflow,
hook ou dependência tocado.

**Comportamento novo de `review-gate.sh post`**

- Escopo explícito: `--base/--head` ou `--range A..B` (todos os commits, base ancestral do
  candidate). Sem escopo → erro de uso; nenhum "último commit" adivinhado. `pre`/`scope --prepare`
  usam working tree vs HEAD, incluindo staged-only e untracked não ignorados.
- Caminhos via `-z` + `--no-renames` (rename/delete listam os dois lados). Falha de git, range
  inválido/vazio ou commit ausente → exit 4 antes de ler qualquer verdict.
- PASS exige receipt do operador **fora** de worktrees/.git (`--receipt` ou
  `ENGRAM_REVIEW_RECEIPT`) com task/base/head/tree/sha256 do diff/review path+sha256/operador, mais
  `--expect-tree/--expect-diff-sha256/--operator` na linha de comando; tudo recomputado do git.
  Ausente/não confiável/divergente → `PENDING`, exit 3 (mesmo com marcador PASS).
- Marcador legado mantido e mais estrito (exatamente uma linha; placeholder do prompt inválido).
  Allowlist de skip inalterada (só docs-only `docs/**/*.md` fora de `docs/harness/`, mecanizado);
  `docs/harness/bin/*` sempre exige reviewer; `--repo` permite rodar uma cópia confiável do base.
- Exit codes e runbook do receipt manual documentados em `GATES.md`.

**Verificações**: `bash docs/harness/bin/test-review-gate.sh` 24 testes / 181 asserções, 0 falhas
(também sob `/bin/bash` 3.2.57); `test-fixtures.sh` 15/15; `test-check-live-state.sh` PASS;
`doctor.sh` OK (1 WARN pré-existente: sem artefato de review para a task ativa); `shellcheck -x`
limpo nos dois scripts.

**NOT RUN / pendente**: revisão independente de H1 (controller); wiring do teste em
`sensors.sh`/`doctor.sh`/CI — `test-fixtures.sh` e `test-check-live-state.sh` também não estão
ligados a nenhuma lane hoje, então o wiring de `test-review-gate.sh` fica para H2 (lane wiring).
Automação da procedência (H4/H5) e isolamento real do writer seguem fora de escopo.

**Rollback**: reverter `dfd8681`; manter guard externo recusando pending — nunca restaurar o
comportamento antigo "sem review → exit 0".

**H1 — correção da revisão (round 1)**: commit `16ecc5a` `fix(harness): harden review gate against writer-controlled state`.
Com fixtures que falhavam antes (RED 23 asserções; depois 88 ao introduzir o binding do gate): (1)
receipt/gate comparados por identidade de filesystem (variante de caixa em APFS, hard link, ACL com
escrita) em vez de string; (2) git sem replace refs/grafts/atributos e diff com `--text` (replace-ref,
graft e `-diff` em `.git/info/attributes` ou `.gitattributes` não escondem nem forjam o escopo);
(3) receipt ganha `GATE_SCRIPT_SHA256`, novo `--expect-gate-sha256`, PASS de gate dentro de worktree
é recusado, runbook exige cópia externa do gate com `--repo`, e o teste de auto-autorização agora usa
base com o gate antigo e candidate com gate adulterado; (4) `node_modules` não é mais excluído,
pathspec literal no loop staged-only, falha de escrita do prompt é exit 4, preparação mostra conteúdo
staged quando index ≠ working tree. GATES.md registra que H1 é aceito pela revisão independente do SDD
e pela decisão do owner (base `cf6d969` não tem receipts). Verificações: `test-review-gate.sh` 35
testes / 224 asserções, 0 falhas (também sob bash 3.2), shellcheck limpo, doctor OK, test-fixtures
15/15, check-live-state PASS.

## H2 — Endurecer schemas e semântica de evidência existentes (2026-10-05)

**Commits** (branch `claude/improvement-lane-p`, locais, sem push): `283fca2`
`fix(harness): inspect receipt ACLs when xattrs mask the plus flag`; `dfbc0c3` e `3796e26`
`test(harness): ... live-state suite ...`; `4ab5e83` `feat(harness): harden evidence validator and add v2 schemas`;
`73ae4a6` `feat(harness): add mandatory offline lane to sensors, doctor and CI`; `4b678ee`
`test(harness): report inapplicable review-gate scenarios separately`; `64ed24d`
`test(harness): sync live-state fixture to sensors telemetry`; `a545e92` (contagens dos pisos).

**O que mudou**

- `validate-evidence.py` reescrito como parser estrito e fail-closed. JSON: chaves duplicadas,
  NaN/Infinity/overflow, UTF-8 inválido/BOM, arquivo >1 MiB, aninhamento >32, symlink/não-regular
  são rejeitados. O schema é escolhido por `schema_version` (ou `--schema` coincidente), nunca por
  marcadores de chave ou nome de arquivo; as regras semânticas dependem do schema resolvido, não das
  chaves do payload. Validador embutido com conjunto fixo de keywords: keyword não suportada, tipo
  desconhecido ou schema malformado falha fechado nos dois modos. bool nunca é inteiro/número,
  `1.0` não é inteiro, `enum`/`const`/`uniqueItems` não confundem `True` com `1`, `$` não aceita `\n`
  final. `jsonschema` (se presente) é só contra-prova com a mesma regra de inteiro: pode transformar
  pass em `validator_divergence`, nunca relaxar; resultado idêntico com e sem a biblioteca.
- Semântica: SHA 40-hex completo (nulo recusado), timestamp canônico com data de calendário real
  (fev/30, hora 24, segundo 60, ano <2000, futuro > relógio+5 min), caminhos sem traversal, absolutos,
  glob, `.`/`//`/controle/backslash (`broad_path`, `unauthorized_glob`, `unnormalized_path`),
  allowed dentro de protected, ids de check/finding/log duplicados, veredito coerente com checks
  (pass com fail/timeout/warn/skipped, status×exit_code) e com findings (`pass` ou `comment` com
  blocker/critical/high; `fail` sem finding).
- Schemas **v2** novos (`task|evidence|review-v2`): `schema_version` e `policy_version` obrigatórios,
  todos os arrays/strings/objetos/inteiros com teto (teste de lint falha se faltar), evidence exige
  task/base/tree/exit_code/duration/log_hash/log_path, severidades e vereditos canônicos. Schemas v1
  ficam **sem alteração** e históricos: validam estrutura, aparecem como `NOTE[HISTORICAL_V1]`, não
  satisfazem expectativa de policy e nunca viram evidência confiável.
- Verificação externa: `Expectations` / flags `--expect-candidate-sha|base-sha|tree-sha`,
  `--expect-policy-version`, `--expect-catalog-sha256`, `--catalog`, `--logs-dir`, `--task`,
  `--require-expectations` (incompleto → exit 2). Valores vêm do chamador; o payload nunca é fonte.
  Catálogo de check IDs (`schemas/check-catalog-v1.json`), cobertura de checks requeridos/timeout/task_id
  contra a task do chamador e verificação de `log_hash` recomputando os bytes sob `--logs-dir`
  (sem symlink, sem escapar do diretório). `validate_file` continua só estrutura.
- Fixtures v2 sintéticas + 9 adversariais novas; `test-fixtures.sh` 15 → 39 asserções.
- **Lane offline obrigatória** `docs/harness/bin/run-offline-lane.sh` (6 componentes: unittest do
  validator, `--self-test`, `test-fixtures.sh`, `test-check-live-state.sh`, `test-review-gate.sh` e o
  contrato fail-closed da própria lane). Falha em: exit≠0, resumo ausente, contagem 0, abaixo do piso,
  skip/NOT RUN (>`HARNESS_LANE_MAX_NOT_RUN`, default 0), componente apagado, jsonschema ausente.
  Ligada a `sensors.sh` quick e full, `scripts/ci.sh` e ao passo incondicional do job required
  `Test (ubuntu-latest)` de `ci.yml` (escolhido porque é required live; `Harness Contract` não é —
  audit E0 §4). `doctor.sh` falha se o wiring (ou um componente da lane) sumir; comentário não conta.

**Correções fora da lista do brief, necessárias para a lane ficar verde** (declaradas):
`review-gate.sh` deixava passar receipt com ACL quando o `ls` mostra `@` (xattr) em vez de `+` — fail-open
real reproduzido no sandbox, agora coberto por teste (`283fca2`); `test-check-live-state.sh` estava
quebrado em HEAD (acoplado a baseline aprovado antigo e ao Last commit do progress; passa a usar fixture
hermético de baseline aprovado e cópia do progress pinada no HEAD, sem remover asserções);
`test-review-gate.sh` agora separa cenários inaplicáveis na plataforma (`Not applicable`) de `NOT RUN`.
A primeira execução `sensors.sh` full com a lane falhou em `live_state`: a suíte lia o `Last sensors` do
progress.md real, que só bate com `.sensors-last` logo após refrescar o progress (todo run de sensors
reescreve o receipt). Corrigido (`64ed24d`): a suíte usa uma cópia do progress com a linha `Last sensors`
sincronizada ao `.sensors-last` atual — sem isso a lane obrigatória quebraria a cada segunda execução.

**Verificações**: `python3 -m unittest discover -s docs/harness/tests -p 'test_validate_evidence.py'` 76
testes OK; `validate-evidence.py --self-test` 18/18; `test-fixtures.sh` 39/39;
`test-check-live-state.sh` 19 asserções; `test-review-gate.sh` 36 testes / 226 asserções;
`run-offline-lane.sh` PASS 6 componentes / 199 checks; `sensors.sh quick` pass (05:21:48Z) e `sensors.sh`
full pass (2026-10-05T05:27:17Z, 107 s, `ci_steps` com `offline_lane: pass`; receipts `.sensors-*` restaurados,
não commitados, como em E0); `doctor.sh` OK (1 WARN pré-existente); `ubuntu:24.04` não-root sem rede:
self-test, fixtures, live-state (inclusive com HEAD = merge commit) e review-gate OK;
mutações negativas do doctor (9) todas falham como esperado.

**NOT RUN / pendente**: execução no GitHub Actions (nenhum push); cenário de ACL do review-gate em
Linux (sem `setfacl` aqui; o CI instala `acl`); `python3-jsonschema` 4.10 do apt do runner não foi
exercitado (local: 4.26); revisão independente de H2 (controller). H3 não deve iniciar antes de a lane
estar aprovada e persistente.

**Rollback**: reverter os commits de H2 desativa o novo consumidor; artefatos v1 continuam
históricos (sem downgrade automático). Desabilitar o passo de CI exige também o doctor
(o wiring é verificado).

### O3 — Avaliadores probabilísticos como aconselhamento, não gates (2026-10-05)

**Entregue** (commit local na branch `claude/improvement-lane-p`, sem push):
- `docs/quality/probabilistic-evaluation-policy.md`: avaliadores probabilísticos são **somente
  consultivos** — nunca habilitam merge, nunca entram no CI default/`sensors.sh`/lane offline;
  "defer/unknown/abstenção/erro de parse/indisponível" = sem sinal (fail-closed); verificadores
  determinísticos prevalecem. Separação de métricas (p(true), distribuição sobre ações, escore
  ordinal, confiança) sem média entre incompatíveis; modelos correlacionados/alias não são votos
  independentes; testes de abstenção explícita, permutação, duas redações predefinidas,
  contexto/truncamento e identificação do modelo; Brier por classe com base rate, N e limites;
  sem fit/eval na mesma amostra; spec do benchmark holdout humano separado do autor da policy;
  reference intake/licença antes de adotar Laya/Kev/JEV; mudança de policy exige avaliação independente.
- Experimento anterior registrado como histórico diagnóstico (Laya 6/8, Brier 0.21588291625; Kev 7/8,
  0.06903502125; 8 casos curados, não holdout; P(defer) do Laya 62.13–82.70% conforme a ordem; JEV não
  executado, sem credencial; alias `jev-latest` não é independente).
- **JEV hospedado: UNAVAILABLE** — sem provisionar credencial, sem proxy, sem enviar conteúdo privado.
- Proveniência: `/tmp/engram-probability-review-2026-10-02/` **não existe**; registrado como
  "provenance unavailable" em `docs/quality/evaluations/2026-10-02-probability-review/PROVENANCE.md`
  (nenhum arquivo bruto copiado ou reconstruído; números vêm do texto do plano, não verificados contra fontes).
- Fixture offline opcional e pequena: `docs/quality/evaluations/holdout-case.schema.json` (JSON Schema
  2020-12) + `holdout-cases.example.jsonl` (2 linhas sintéticas). Validado manualmente com `jsonschema`;
  **não** ligado a testes, CI, sensors nem doctor.

**Verificações**: schema e 2 linhas de exemplo validam (`jsonschema` Draft 2020-12); `doctor.sh` rodado
(resultado no relatório da tarefa). **NOT RUN**: nenhum avaliador (Laya/Kev/JEV) foi executado; nenhuma
avaliação independente da policy (cabe ao controller/reviewer).

**Rollback**: remover os arquivos novos em `docs/quality/`; nada depende deles.

**O3 — correção da revisão (rodada 1)**: `phrasings` passou a `required` no schema do holdout (exatamente
2 itens; linha sem `phrasings` agora falha, 1 e 3 itens também; as 2 linhas de exemplo continuam válidas).
Policy §7: exclusão de labelers (autor da policy / configurador dos avaliadores) é checagem de processo que o
schema não impõe; §8: `unavailable` (infra/credencial ausente) distinto de `NOT RUN` (não executado/pulado).

### Q5 — Segurança agregada, política de findings e supply chain (2026-10-05)

**Entregue** (commit local na branch `claude/improvement-lane-p`, sem push; nada remoto tocado):
- `scripts/check-security-gate.py`: decisão do agregado agora é **tri-state** (`pass`/`neutral`/`block`).
  Skip permitido explicitamente = `NEUTRAL` (nunca `PASS`); failure, cancelled, timed_out, ausente, estado
  desconhecido e skip não autorizado = `block`. A matriz exige os 7 casos (all-pass, failure, cancelled,
  timed-out, missing, unauthorized-skip, allowed-skip-neutral); `pull_request`/`push`/`schedule`/
  `workflow_dispatch` nunca podem permitir skip; cenário não inventa autorização. Novo: `Test (ubuntu-latest)`
  (`required_dependency_job`) precisa continuar dependendo de `security-gate` (remover `needs:` → falha),
  e o agregado precisa de `if: always()`. `--self-test-failure`/`--self-test-unrequired` seguem verdes.
- `scripts/check-security-findings.py` (+ fixtures `tests/fixtures/security_findings/`, `matrix.json` com 27
  cenários): finding **high** bloqueia mesmo com exit 0 do scanner (level `error` ou `security-severity >= 7.0`,
  inclusive via `defaultConfiguration` da regra); exceção aprovada exige owner + approved_by + rationale +
  expiry <= 90 dias (expirada/ inválida/ de outra regra → block; arquivo inválido rejeita tudo);
  `suppressions` embutidas no SARIF são ignoradas (payload não se auto-aprova); SARIF ausente, malformado,
  de outra ferramenta, sem proveniência de revisão ou stale (SHA != `--expected-sha`) → block; scanner não
  executado → block, salvo `--allowed-skip` do supervisor → neutral. Identidade (scanner, tool, SHA, exit)
  vem só dos argumentos de CLI. Upload SARIF bem-sucedido nunca é lido como "sem findings".
- `scripts/check-workflow-supply-chain.py` (+ ledger `docs/security/supply-chain-pins.toml`): toda `uses:`
  por SHA; toda imagem por digest ou no ledger com owner/expiry (entrada vencida ou sem uso falha); job
  exposto a `pull_request` sem permissão de escrita e sem secrets além de `GITHUB_TOKEN`; sem
  `pull_request_target`. Roda no job `security-gate` junto com os testes de contrato.
- `ci.yml`: `codeql-security` virou job read-only (`upload: never`, SARIF retido como artifact, política de
  findings aplicada com `--expected-sha ${{ github.sha }}`); SARIF de Semgrep/Gitleaks retido; Gitleaks pinado
  por digest; `bench` (medição, `contents: read`, sem comentário/push) separado de `bench-publish`
  (`contents: write`, só push em `main`). Workflows `codeql/semgrep/gitleaks/agentshield.yml`: `scan` read-only
  em qualquer evento + `publish` (`security-events: write`) só fora de `pull_request`.
- Exceções de advisory: as 10 expiradas em 2026-09-30 foram **reverificadas** contra o `Cargo.lock` atual
  (`cargo audit --no-fetch`, advisory-db `ef6173c` de 2026-10-03) e renovadas até 2026-12-31 (owner Ronaldo)
  porque nenhum update compatível as corrige (um `cargo update --offline` completo em cópia scratch mantém
  todas); dependency paths atualizados para `engram-core 0.23.0`; o caminho AWS saiu (webpki 0.101.7 e h2 do
  AWS não existem mais; sobraram libsql/turso e crates não mantidos). `check-security-exceptions.py` agora
  rejeita expiry > 90 dias. **→ Q6** (não renovados): RUSTSEC-2026-0285 `rustls 0.23.36` → `>=0.23.45`
  (D2, sem exceção) e RUSTSEC-2026-0221 `event-listener` → `5.4.2`.
- Docs: `GATES.md` (seção do security gate + lista de required checks refrescada para o estado live:
  Format, Clippy, Documentation, Test, Security Audit, Cargo Deny; D4), `docs/security/security-gate-evidence.md`,
  `docs/security/finding-exceptions.toml` (vazio, schema documentado).

**Verificações**: ver relatório da tarefa (testes de contrato, `check-security-exceptions.py`, `cargo audit`/
`cargo deny` offline, `actionlint`, `doctor.sh`, `run-offline-lane.sh`). **Estado esperado**: `Security Audit`
e `Cargo Deny` ficam **vermelhos** nesta branch (RUSTSEC-2026-0285) até o Q6 atualizar `rustls`; o job
`Security exception policy` volta a ficar verde.

**NOT RUN / não verificado**: nenhum workflow executou em CI real (CodeQL `upload: never`/`output`, proveniência
de revisão no SARIF do CodeQL, `--sarif-output` do Semgrep, digest do Gitleaks derivado do store Docker local);
digest do `semgrep/semgrep:1.169.0` e das bases do `Dockerfile` não resolvidos offline (ledger, vence 2026-11-05);
revisão independente (controller).

**Rollback**: reverter os commits de Q5 (workflows + matriz + scripts + docs juntos); não há bypass: reverter
só o workflow deixaria o checker/matriz reprovando e vice-versa.

### H3 — Registry de checks e adaptador de sandbox com fake writer (2026-10-05)

**Commit**: `feat(harness): add check registry and sandbox adapter with fake writer`.

- **Escopo**: somente fake writer, offline (ADR hardening v1 aceito; lista "não concedido" intacta: sem writer real, sem
  fallback para o host, sem credenciais, sem rede). Nenhum workflow/CI tocado (Q5 em paralelo).
- **Registry** `docs/harness/checks/registry.json` (`check-registry-v1`): imagem pinada por digest
  (`python@sha256:02108f5d…9155d`, índice multi-arch de `python:3.12-slim`; um `docker pull` autorizado, só para o smoke
  local), limites do sandbox e argv fixo por check ID. IDs reutilizam `schemas/check-catalog-v1.json` (subconjunto
  verificado; ID fora do catálogo = recusa). Só `pr_title_policy` é executável na imagem (caso de aceitação apenas — o
  caso de rejeição exige exit esperado ≠ 0 e segue no `sensors.sh`); checks `cargo` aguardam imagem com toolchain.
  Rejeitados: argv como shell string, wrappers (`env`, `timeout`, `nice`, `setsid`, `stdbuf`, `xargs`, `sudo`, `docker`…),
  `awk`/`sed`, flags de código inline (`-c`/`-e`/`-E`/`-p`/`-r`/`--eval`…, varridas em TODOS os args de shells, python e
  interpretadores) e imagem por tag.
- **Adaptador** `docs/harness/bin/sandbox-adapter.py` — `run_isolated(manifest_path, worktree_path, run_dir) -> RunOutcome`
  (`status`, `exit_code`, `argv`, `started_at`, `finished_at`, `log_paths`, `limits_enforced` + extras: `checks`,
  `reason`, `manifest_sha256`, `manifest_target_sha` com `target_sha_bound=false`, `docker_endpoint`, `executed_on_host=false`). Manifesto = `task-v2` validado pelo validador H2 (política
  esperada vem do registry, fora do payload) e exige `sandbox_container` + `network_none`. Container: `--network none`,
  `--cap-drop ALL`, `no-new-privileges`, uid do invocador (nunca root), rootfs read-only, tmpfs `/tmp` noexec,
  memory/swap/pids/cpu, `--init`, `--pull never`, env por allowlist constante (sem HOME/SSH/askpass/credenciais).
  Mounts: worktree em `/work` (rw) e snapshot do TCB (manifest/registry/catalog/tools) em `/tcb` (ro); `run_dir` e o TCB
  ativo nunca são montados e só o supervisor escreve evidência (`outcome.json` + logs com sha256, criação exclusiva).
  `limits_enforced` vem de `docker inspect` lido após o `create`; limite não aplicado pelo Docker = não inicia
  (`error`). Timeout: `docker kill` + `rm -f` + verificação por label de que nada sobrou; sobrevivente = `error`.
  Docker ausente/daemon inacessível/imagem ausente = `unavailable` (exit 3), `refused` (exit 2) para manifesto/ID/layout
  inválidos; **sem fallback para o host** (única coisa criada no host é o processo `docker`, nunca o argv do check).
  Worktree que contém o próprio adaptador ou o manifesto/registry é recusada (o writer editaria o TCB).
- **Testes**: `tests/test_sandbox_adapter.py` (37, OFFLINE com `docker` fake que sintetiza `inspect`; roda na lane offline
  como componente `sandbox_unit`, floor 30 — lane agora 7 componentes, `OFFLINE_LANE: PASS components=7 checks=237`;
  `test_offline_lane.py` e `doctor.sh` atualizados) e `tests/test_sandbox_smoke.py` (14, Docker real + `fake_writer.py`;
  **fora** da lane/CI via `bin/run-sandbox-smoke.sh`, que termina em `PASS`/`FAIL`/`UNAVAILABLE` — exit 3, nunca pass).
  O fake writer tenta: escrever fora do worktree (inclusive via symlink e `/var/tmp`), setuid/mknod/mount/chroot,
  adulterar `/tcb`, rede (TCP/UDP/DNS), ler env/HOME/SSH/canário, fork bomb (parado por EAGAIN no pids cap), sobreviver
  ao timeout, filho daemonizado (não sobrevive), forjar `outcome.json`/logs no `run_dir`, estourar memória (OOM 137)
  e inundar log (truncado no cap). Mutações manuais (tcb rw, rede bridge, rootfs rw, caps mantidas, sem pids, sem
  no-new-privileges, sem memória) derrubam o smoke com a verificação de read-back desligada.
- **Não verificado / NOT RUN**: smoke só em macOS/OrbStack (arm64, Docker 29.4; contexto `orbstack`); Linux/x86 do CI não
  exercitado; vínculo do `target_sha` do manifesto ao HEAD do worktree fica para o runner (H4+); revisão independente
  (controller).

**Rollback**: reverter o commit de H3 (a lane volta a 6 componentes); `run_isolated` não é chamado por nenhum gate,
então desativar a execução = não invocar o adaptador; a revisão estática segue disponível.

---

**Q5 — correção da revisão (rodada 1)**: (1) a renovação de RUSTSEC-2026-0235 (`rkyv 0.7.46`) foi **retirada**
(`cargo update -p rust_decimal --precise 1.43.0` em cópia scratch remove o rkyv; id removido de
`.cargo/audit.toml` e do TOML governado; agora → Q6 como 0285, sem exceção) e a remediação de `ttf-parser`
(0192) explica que cortar exige mudança de produto (pdf-extract 0.12 fixa lopdf ^0.42); (2) novo
`scripts/test_check_security_ci_contract.py` fixa o passo "Enforce CodeQL findings policy" (argv completo,
`upload: never`, sem `if`/`continue-on-error`) e os exit-code de Semgrep/Gitleaks; roda no job `security-gate`;
(3) dica de correção na mensagem "no revision provenance"; comentário de cabeçalho do `ci.yml` (seis
contexts); checker de supply chain agora lê `permissions: {…}` em flow-style, exige `permissions` top-level em
workflows com `pull_request` e trata condições mistas `!=`/`==` de forma conservadora; ledger de imagens
vence 2026-11-05 e precisa de resolução de digest online (documentado em `docs/security/security-gate-evidence.md`).

### H3 — rodada de correção 1 (revisão) (2026-10-05)

- **Endpoint do Docker**: resolvido uma única vez (`DOCKER_HOST` > `DOCKER_CONTEXT` > contexto atual da config, via
  `docker context inspect`); só socket unix local (ssh://, tcp://, npipe → `unavailable`); todas as chamadas seguintes usam
  `DOCKER_HOST=<socket>` sem `DOCKER_CONTEXT`; endpoint registrado no outcome; socket dentro do worktree é recusado.
  Substitui a limitação anterior de "contexto remoto não detectado".
- **Read-back mais estrito**: `SecurityOpt` exatamente `no-new-privileges` (seccomp/apparmor/label unconfined recusados),
  Pid/Ipc/UTS não-host, `Devices` vazio, `RestartPolicy` none, `/tmp` com `noexec,nosuid,nodev` + `size`, entrypoint/cmd
  iguais ao argv do registry; `limits_enforced` também por check.
- **Registry/argv**: módulo irmão `bin/sandbox_registry.py` (política pura, sem subprocess); adaptador ficou com 759 linhas.
- **Evidência/paths**: `run_dir` precisa ser diretório do euid sem escrita de grupo/outros; staging do TCB dentro do worktree
  e worktree que é/contém `$HOME` são recusados; falha de escrita de log vira `error`; o orçamento total do manifesto agora é
  testado de forma não vacuosa (mutação "sem guarda de orçamento" derruba o teste).
- Testes: `test_sandbox_adapter.py` 48 (lane: `OFFLINE_LANE: PASS components=7 checks=248`); smoke real 14 OK no OrbStack.

---

### H3 — rodada de correção 2 (revisão) (2026-10-05)

- `test_sandbox_adapter.py` definia `class CliTests` duas vezes; a segunda sombreava a primeira e 5 testes de
  `build_create_args` nunca rodavam. A primeira voltou a ser `CreateArgsTests` (suíte: 48 -> 54 testes, todos verdes) e foi
  adicionado `ModuleHygieneTests` (via `ast`: sem classes/métodos duplicados nos módulos de teste do sandbox). O floor de
  `sandbox_unit` na lane subiu para a contagem exata (54), então perda silenciosa de teste derruba a lane.
- `validate_argv`: flags inline agora casam por prefixo (`-cCODE`, `-e'puts 1'`, `-pe…`) e `--flag=valor` nas longas; testes
  para python/bash/perl/ruby/node. `OFFLINE_LANE: PASS components=7 checks=254`.

---

### H6 — Contexto curto, retomável e retenção explícita — 2026-10-05

**Commits** (branch `claude/improvement-lane-p`, locais): `feat(harness): keep live context short and history retained` e o
commit de refresh do `Last commit`. Nenhum Rust, MCP, storage, SDK, workflow, hook ou dependência tocado; arquivos de H3
(sandbox, registry, smoke) intactos.

**Entregue**

- **Resumo vivo ≤150 linhas**: `progress.md` (1.522 → 109 linhas; 87,6 KB → 9,7 KB) com campos compatíveis com o doctor,
  seção "Retomada rápida (escopo, limites, última evidência)", tabela de tasks com links para este log, reconciliação
  required/advisory (exigida pelo checker) e a trilha de exclusão registrada. O texto anterior está **byte a byte** em
  [`progress-history.md`](../progress-history.md) (68 seções H2 indexadas, marcador `BEGIN-VERBATIM` com tamanho e SHA-256).
- **Orçamento e medição** em [`context-budget.md`](../context-budget.md): tokenizer **identificado** `cl100k_base` via
  `tiktoken-rs 0.5.9` (mesma crate/versão do `Cargo.lock` e do `TiktokenCounter`; ferramenta descartável fora do repo,
  offline); `measure-context.py` no repo usa `tiktoken` Python só se importável e em cache (sem rede), senão a aproximação
  rotulada `approx:utf8-bytes/4` (subestima ≈6% aqui). Conjunto obrigatório de 14 arquivos: ≈265,7 KB / ≈70,8 mil tokens
  antes → ≈196,9 KB / ≈53,2 mil tokens depois (**−25,9% bytes, −25,0% tokens**; `progress.md` −88,9% bytes); o log
  do active plan cresce a cada task, rode `measure-context.py` para o valor atual.
- **Retenção/leitura**: `measure-context.py --section ARQ#âncora` lê a seção inteira (sem cortar parágrafo);
  `check-doc-links.py` (+ `doc_links.py`) valida links relativos e âncoras no estilo GitHub e trata alvo ignorado por git
  como quebrado (clone limpo). Único alvo pendente tolerado: o log da lane R (aparece como `ALLOWED`).
- **Enforcement do live state** (deferido de H2): `check-live-state.sh --structural`, rodado pelo `doctor.sh` no
  `progress.md` real (`live_state:structural`): campos, plano, review autoritativo, linhas de reconciliação e Last commit
  bem formado; ancestralidade de HEAD só como aviso (versão original dura corrigida na rodada de correção 1 abaixo); sem
  exigir HEAD exato nem timestamp de sensores (churn). A forma estrita continua fechando tasks. `doctor.sh` também falha se `progress.md`
  passar de 150 linhas e se o bootstrap passar de 50 linhas (antes 60).
- **Bootstrap**: já cumpria a meta (40 linhas, mediana 154 ms em 20 execuções); agora 42 linhas (`Last review` e ponteiro de
  retomada), contrato ≤50 linhas e <500 ms testado. Não se afirma ganho de tempo.
- **Lane offline**: novo componente `context_budget` (44 testes, piso exato 44); lane agora com 8 componentes;
  `test_offline_lane.py` (13 testes) e as checagens de fiação do `doctor.sh` atualizados.
- **Ordem de leitura inalterada**; teste compara bootstrap, `AGENTS.md`, `CLAUDE.md` e `INVARIANTS.md` do harness.
  Duplicação AGENTS/CLAUDE medida (864 B + 771 B ≈ 446 tokens, ≈0,6%): **mantida** por decisão (cada arquivo precisa ser
  autossuficiente) e protegida por teste contra divergência; nada normativo removido (D10: sem consolidar `INVARIANTS.md`
  raiz/`STANDARDS.md`).
- **#152 (ruling do controller, pendente de confirmação do owner)**: chunks da ingestão de documentos continuam em **caracteres**, distintos de
  tokens; nenhum tokenizer/modelo adicionado; `TokenChunker` segue sem chamador de produção; registrado em
  `context-budget.md` §6 como entrada para Q7. Sem implementação.
- **Menores de E0**: ADR (linha *Source*) agora atribui "Tudo, inclusive Onda 4" à resposta do owner no chat, não à nota do
  plano; auditoria E0 §9 registra que **não houve Review Canvas** para o aceite do ADR (desvio) e fecha D3/D4/D10; a entrada E0
  deste log aponta D7–D12 para a tabela da auditoria. **D4** (GATES): reverificado contra a proteção live e o `progress.md`
  (seis contexts required; `Security Gate`/`Harness Contract` fora; sem review de PR obrigatório); `GATES.md` ganhou a
  seção de H6, os itens 7-8 da lane e a precisão de que o `doctor.sh` roda no job separado `Harness Doctor Advisory`.

**TDD**: RED = `python3 -m unittest discover -s docs/harness/tests -p test_context_budget.py` com os testes escritos antes
(44 testes, 16 falhas + 6 erros: `progress.md` com 1522 linhas, sem seção de retomada, sem `measure-context.py`/`context-budget.md`,
etc.); mutações manuais (editar o histórico, +60 linhas no resumo, trocar a ordem no `AGENTS.md`, link quebrado) derrubam os
testes certos. Estrutural: `test-check-live-state.sh` RED com `unknown argument: --structural`; GREEN 33 asserções (eram 19).

**Verificações**: `test_context_budget.py` 44 OK; `test-check-live-state.sh` PASS (33); `test_offline_lane.py` 13 OK;
`run-offline-lane.sh` → `OFFLINE_LANE: PASS components=8 checks=313`; `doctor.sh` OK (1 WARN pré-existente: sem artefato
de review da task ativa); `shellcheck -x` limpo nos 5 scripts alterados; checker de links PASS nos 3 documentos; suíte em clone
limpo (`git worktree add --detach HEAD`, removido depois) registrada no relatório da task.

**NOT RUN**: `sensors.sh` quick/full (reescreve o `.sensors-last` rastreado e causaria conflito de telemetria entre lanes;
doctor e lane rodaram à parte); nenhum CI real; revisão independente (controller). **Pendente de propósito**: rotação das entradas
concluídas deste log para arquivos por task (é o maior arquivo não-normativo do conjunto, ≈10 mil tokens); evitado aqui por
conflito com edições concorrentes da lane. Integração de tokenizer na ingestão fica com Q7 se o owner reabrir #152.

**Rollback**: reverter o commit de H6 devolve o `progress.md` anterior (também preservado em `progress-history.md`), o doctor
e a lane de 7 componentes; nada mais depende da divisão. Links testados.

### H6 — rodada de correção 1 (revisão) — 2026-10-05

- **Ancestralidade estrutural não falha mais o job required**: um Last commit bem formado que HEAD não alcança (squash/rebase
  reescrevem SHAs; reproduzido com squash merge) vira `ancestor_check=unreachable` + linha `WARN` e exit 0; só id malformado
  falha. Nova flag `--require-ancestor` mantém a variante dura apenas para fixtures herméticos; o `progress.md` real na lane
  não assevera ancestralidade. Testes: squash merge, divergência/rebase, id malformado em histórico completo e raso.
- **Sem falso pass em clone raso**: `skipped-shallow` e `unreachable` agora viram **warning do doctor** (`live_state:structural`);
  `test_context_budget.py` roda o `doctor.sh` real em clones raso e completo (Last commit reescrito só no clone) e exige o warning.
- **Menores**: tempo do bootstrap configurável (`HARNESS_BOOTSTRAP_MAX_SECONDS`; 0,5 s local, 2,0 s com `CI=true`; ≤50 linhas segue
  duro); SHA-256 do histórico fixado como constante no teste e em `context-budget.md` (com conferência contra
  `git show 4d341a0:docs/harness/progress.md` quando o objeto existe); tabela antes/depois ganhou a coluna `approx:utf8-bytes/4` e o
  procedimento para re-derivar bytes/approx/tokens; **#152 reatribuído** a ruling do controller, pendente de confirmação do owner
  (`context-budget.md` §6 e `progress.md`).

### H4 — Runner e evidência externa pós-commit (2026-10-05)

**Commit**: `feat(harness): add trusted runner, scope check and evidence recorder`.

- **Escopo**: somente fake writer, offline (ADR aceito; "não concedido" intacto: sem writer real, sem fallback para o host,
  sem credenciais, sem rede, sem auto-merge). `writer.adapter` diferente de `fake` = `writer_not_granted`. Nenhum CI tocado.
- **Runner** `docs/harness/bin/run-task.py` (request `runner-request-v1`): base exportada byte a byte (ls-tree + cat-file,
  sem `.git`) para `a<N>/ws`, fake writer via `run_isolated`; workspace re-hasheado em **tree** (hash-object
  `--no-filters` + update-index, sem `.gitattributes`/`.gitignore`); `check-scope.py` base..tree; **só depois** o
  supervisor cria o commit candidato (`commit-tree`, identidade fixa, pai = base) e a ref create-only
  `refs/engram-runner/candidates/<task>/<run>/a<N>r<M>`. Gate em checkout limpo **separado** (`a<N>/c<M>`) com task-v2
  `target_sha = candidato`; checkout re-hasheado depois: qualquer mudança = `gate_mutated_checkout` (nunca pass).
  Recusas: kill switch `ENGRAM_RUNNER_DISABLED=1`, request inválido, repo errado, ref divergente, base suja (tracked),
  runs root dentro de worktree/git dir, segundo writer (lock). Exit 0/1/2/3/4 = passed/failed/refused/unavailable/scope.
- **Orçamentos** (default do piloto = teto do request): wall 2700 s, attempts 2, repair 1, writer concurrency 1 —
  `enforced` (prazo do supervisor/contadores/lock); timeout de check pelo adaptador. `turn_cap` 20 registrado como **não
  enforced** (fake writer sem turnos observáveis) e `cost_cap_usd` **não enforceable**: request que o define ou exige
  (`require_enforced`) é recusado.
- **Scope** `docs/harness/bin/check-scope.py`: `git diff --raw -z --no-renames` com git neutralizado
  (`harness_git.py`: sem replace refs/grafts/atributos/hooks/config global). Protegidos: `docs/harness/` inteiro (TCB —
  resolve o achado do H3: `pr_title_policy` roda o script do checkout, mas o writer não pode alterá-lo), CI/config de
  agente, CODEOWNERS/.gitattributes/.gitmodules/AGENTS.md/CLAUDE.md em qualquer nível, case-fold; lockfiles salvo
  `allow_lockfiles`; symlink que escapa/aponta para protegido; submódulo; mode/type change; nome com controle/newline ou
  não-UTF-8; enfraquecimento de teste (arquivo de teste apagado, linha removida de teste existente, marcador de skip
  adicionado, remoção líquida de asserts em fonte).
- **Evidência** `docs/harness/bin/record-evidence.py`: `record()` só lê insumos do supervisor (outcome.json, logs brutos
  do docker attach sem RTK, task/registry/catalog staged), recalcula hashes e grava evidence-v2 + run-record + receipt em
  `a<N>/g<M>`; `verify()` exige forma exata do caminho sob runs root marcado e não aninhado, candidato esperado, árvore,
  pai = base, ref do runner, validador H2 com todas as expectativas, manifest sha256 recalculado e política sem drift.
  Limite declarado: não é assinatura (processo do mesmo usuário no host reescreve o bundle).
- **TDD**: scope RED (24 testes com esqueleto `NotImplementedError`) → GREEN; runner/evidência escritos junto da
  implementação, RED demonstrado por mutação (sem verificação de manifest/expectativas/forma do caminho/scope →
  9 testes falham). **Testes**: `test_scope.py` 24, `test_runner.py` 23, `test_evidence_integrity.py` 15 (byte alterado
  no log e no candidato, log apagado, hash adulterado com e sem re-selar o receipt, PASS forjado pelo writer, drift de
  política) → componente `runner_unit` (floor exato 62); lane `OFFLINE_LANE: PASS components=9 checks=388`; `doctor.sh`
  OK. Smoke Docker real separado: `run-runner-smoke.sh` → `RUNNER_SMOKE: PASS tests=4` (UNAVAILABLE distinto, exit 3);
  H3 `SANDBOX_SMOKE: PASS tests=14` após novos comportamentos `write_file`/`delete_file`/`claim_pass` do fake writer.
- **NOT RUN**: `sensors.sh` (telemetria entre lanes), CI real, revisão independente (controller).
- **Rollback**: `ENGRAM_RUNNER_DISABLED=1` desliga o runner sem apagar receipts/refs; reverter o commit remove runner,
  scope e recorder (nada no CI depende deles) e devolve a lane de 8 componentes.

### H4 — rodada de correção 1 (revisão) (2026-10-05)

- **Symlinks resolvidos, não léxicos**: `check-scope.py` resolve o alvo contra a árvore candidata seguindo os symlinks da
  própria árvore (limite de 40 saltos; laço = `symlink_escape`). Vale para links novos/alterados e para links antigos cuja
  resolução mudou em relação à base. Sondas `src/a/d -> ../..` + `x -> d/../../outside` (fuga) e `x -> d/docs/harness/bin`
  (TCB) agora recusadas.
- **Testes inline em Rust**: novo código `rust_test_attr_removed` para qualquer `.rs` modificado/apagado que perca (ou edite)
  linha com `#[test]`, `#[tokio::test]`/`path::test`, `#[rstest]`, `#[test_case]` ou `#[cfg(...test...)]`;
  `#[ignore]` adicionado segue em `test_skip_added`. Limite documentado: troca de assert por outro mais fraco na mesma
  contagem de linhas não é detectada.
- **Concorrência real por repositório**: lock em `<git-common-dir>/engram-runner.lock` (fora de qualquer workspace do
  writer); dois runs roots disputando o mesmo repo → `writer_concurrency_cap`.
- **Menores**: marcador do runs root só em diretório recém-criado ou vazio (diretório alheio não é adotado); `cleanup` não
  cria diretórios; export/re-hash com limite de arquivos/bytes (`workspace_too_large`) e prazo (`deadline_exceeded` →
  `wall_budget_exhausted`); `.gitleaksignore`, `.pre-commit-config.yaml`, configs de cobertura/nextest (`.config/`),
  `docs/quality/` e scripts de quality budget protegidos; `_verify_manifest` recusa chaves absolutas/`..`/vazias.
- **Testes**: RED antes da correção (scope 10 falhas; runner/evidência 5 falhas + 3 erros), GREEN 71/71
  (`runner_unit` floor exato 71); lane `components=9 checks=397`; doctor OK; `RUNNER_SMOKE: PASS tests=4`. Sleeps de
  relógio real não dominam o tempo (custo é de processos git/stub), mantidos.

### H4 — rodada de correção 2 (revisão) (2026-10-05)

- **Desligamento aditivo de testes Rust**: novo código `cfg_gate_added` para linha `#[cfg(...)]`/`#![cfg(...)]` adicionada
  em `.rs` modificado (teste ou não) cujo predicado não é um gate simples (permitidos: `test`, `unix`, `windows`,
  `debug_assertions`, `doc`, `miri`, `feature`/`target_*`/`panic = "..."`, com `all(...)`/`any(...)` não vazios). Recusados:
  `any()`, `not(...)`, identificador solto (`cfg(FALSE_FEATURE)`), todo `cfg_attr` adicionado e atributo que não fecha na
  linha. `RUST_TEST_ATTR_RE` agora casa `#![`; `tests.rs`/`test.rs` são arquivos de teste. Limite documentado: atributo
  multilinha é julgado pela primeira linha.
- **Lock do repositório**: falha ao abrir/travar `<git-common-dir>/engram-runner.lock` vira `runner_lock_unavailable`
  (recusa, nunca exceção); teste com `.git` sem permissão de escrita.
- **Testes**: RED (6 falhas + 1 erro) → GREEN 75/75 (`runner_unit` floor exato 75); lane `components=9 checks=401`; doctor
  OK; `RUNNER_SMOKE: PASS tests=4`.

### O2 — Higiene Git e retenção recuperável (2026-10-05)

**Commit**: `a40430d` `feat(harness): add git retention policy, manifest and restore tool`.
Escopo: somente O2. O log operacional de O1 (observabilidade) está registrado como
**pending O1 implementation** (ruling do controller).

- **Inventário somente leitura** (`retention-manifest.py inventory`, 2026-10-05T15:07:31Z, HEAD `f96b200`):
  1.219 arquivos rastreados / 31.366.068 bytes; `review-raw` = 73 arquivos / 4.249.732 bytes em HEAD
  (57 blobs distintos no histórico, 739.705 bytes em disco empacotados); `git count-objects -vH`:
  5862 soltos / 36,06 MiB, `size-pack` 13,47 MiB, `prune-packable` 4464; refs compartilhadas
  59 heads / 59 remotes / 10 tags / 76 outras, 13 worktrees, 1.640 entradas de reflog. Sensibilidade por
  nome de flag, nunca conteúdo: 7 dos 73 `.raw` marcados (`sensitive: yes, path only`; 4 e-mail, 3 caminho
  absoluto do home, 1 heurística de credencial; nenhum token/chave de provedor).
- **Política** em `docs/OPERATIONS_GIT_RETENTION.md`: por classe, owner, duração (propostas), armazenamento/hash,
  consulta e restauração; operações destrutivas (`gc --prune=now`, `reflog expire`, `filter-repo`, force-push)
  listadas como proibidas sem autorização separada e sem writers concorrentes; nenhuma promessa de MB.
- **Ferramenta** `docs/harness/bin/retention-manifest.py` (stdlib): `inventory`, `generate`, `verify`
  (exit 0/1/4), `backup` (endereçado por sha256), `verify-backup`, `restore` (`--backup`/`--from-git`, hash
  antes de escrever, nunca sobrescreve arquivo diferente), `untrack` (dry run por padrão; `--apply` exige
  backup verificado; só `git rm --cached` + regra de ignore; não apaga nem comita). Manifest versionado em
  `docs/harness/retention/review-raw.manifest.json` (73 entradas).
- **Uma classe migrada, só em clone descartável** (`review-raw`): `untrack` + commit em A; clone novo B sem
  `.raw`: `doctor.sh` OK, links OK, `run-offline-lane.sh` `OFFLINE_LANE: PASS components=9 checks=401`;
  `restore --backup` e `restore --from-git` devolvem os 73 arquivos byte a byte (`diff -r` idêntico, `git status`
  limpo). Medição (clones antes x depois): checkout −4.190.592 bytes (−73 `.raw`, +ferramenta/manifest);
  `size-pack` 6.290 → 6.178 KiB, **variação de empacotamento não atribuível**, não prometida (blobs seguem
  no histórico). **No repositório real nada foi destrackeado, `.gitignore` não mudou, nenhum objeto/ref tocado.**
- **TDD**: RED `python3 -m unittest discover -s docs/harness/tests -p test_retention.py` com a ferramenta ausente:
  `FAILED (failures=18, errors=2)` (21 testes; exit 2 de "can't open file"); GREEN: `Ran 21 tests ... OK`.
  Cobre manifest/hash/determinismo, recusa de arquivo sujo, sensibilidade sem vazar sentinela, verify
  (ok/mismatch/missing/unlisted), backup adulterado, restore sem sobrescrita, path traversal, untrack e a
  prova ponta a ponta em clones.
- **Decisão pendente do owner**: aplicar o untracking na branch real (e a regra `docs/harness/reviews/*.raw` no
  `.gitignore`), pois muda o que `review-gate.sh` deixa para commit; fazer com writers concorrentes parados.
- **NOT RUN**: `sensors.sh`, CI real, revisão independente (controller); backup persistente do repositório real
  (o backup da prova foi em diretório temporário).
- **Ligação na lane** (depois que H5 commitou): commit `980737f` `chore(harness): wire retention tests into the offline lane`;
  `run-offline-lane.sh` (componente `retention`, floor exato 21, 11 componentes), `doctor.sh`
  (executável/manifest/doc/ligação) e `test_offline_lane.py` (+1 teste, 16). Lane `OFFLINE_LANE: PASS components=11
  checks=460`; doctor OK.
- **Rollback**: reverter o commit remove ferramenta, manifest, testes e documento; nada versionado foi alterado.

### H5 — Review SHA-bound e merge-policy read-only (2026-10-05)

- **Avaliador**: `docs/harness/bin/merge-gate.py` decide só `eligible`/`refused` (JSON com `reasons`, `sections`,
  `bound{task, base, head, tree, policy}`; exit 0/1/2) e nunca aprova, faz merge, deploy, push ou publica. Roda da
  base confiável contra o candidato (`--repo`); head/base re-resolvidos do git e re-checados antes de emitir.
  Consome H4 `verify()` (evidência do head atual, `stale_base`, outro gate não-pass do mesmo candidato), receipt do
  operador no formato H1 `RECEIPT_VERSION=2` (+ `REVIEWER`, `POLICY_VERSION`, `REVIEW_CONTEXT=fresh-readonly`; v1 →
  `reviewer_unavailable`), diff sha256 recomputado pela cópia confiável de `review-gate.sh`, review `review-v2`
  validado por H2, allowlist de identidades/lineage do operador (campo livre `reviewer` não autentica; lineage do
  writer vem do adapter registrado pelo runner) e check-runs do app `github-actions` salvos pelo operador, julgados
  pelo `verdict()` do Q5 (último resultado por context; FAIL posterior vence). Merge queue: `--integrated-ref` exige
  árvore integrada == árvore do head, senão `integrated_tree_unverified`.
- **Workflow** `.github/workflows/agent-evidence.yml`: `pull_request` (nunca `_target`), `contents: read`, sem
  secrets, checkouts sem credencial persistida; testes e avaliador do checkout da base (`trusted/`), head como input
  (`candidate/`, nunca executado). Sem inputs do operador a decisão é `refused` por desenho; o job valida só que a
  decisão é bem formada. Coberto pelo `check-workflow-supply-chain.py` (PASS).
- **Testes** (`docs/harness/tests/test_merge_gate.py`, 36): fixtures para task/SHA/policy errados, reviewer
  indisponível/forjado/não autorizado, replay de approval do candidato anterior, prosa/marcador/v1/JSON truncado,
  PASS com finding bloqueante, log ausente, gate posterior FAIL, CI FAIL posterior/pendente/de outro app/de outro
  head, base stale, head e base trocados durante a avaliação, troca de lineage, receipt/allowlist/CI dentro do repo,
  symlink, hard link ou graváveis, árvore integrada, contrato read-only do workflow (+ variantes não confiáveis) e
  ausência de escrita no repo/runs root. RED contra stub permissivo (sempre eligible): `Ran 36`, 47 falhas de
  subteste + 3 erros; GREEN 36/36. Lane `merge_gate` (floor exato 36): `OFFLINE_LANE: PASS components=10
  checks=438`; `test_offline_lane.py` 15/15; `doctor.sh` OK (+ checks de wiring e `merge_gate_workflow:read_only`).
- **NOT RUN**: execução real do workflow no GitHub, CI real, `sensors.sh`, revisão independente (controller).
  Primeiro PR que introduz o workflow falha por desenho ("evaluator unavailable on the base revision").
- **Rollback**: remover `agent-evidence.yml` e o componente `merge_gate`; gates aceitos (H1–H4, Q5) e evidência
  antiga ficam intactos e não são reetiquetados.

### O4 — Standing checks read-only com ownership (2026-10-05)

- **Goals**: `docs/harness/goals/registry.json` + `docs/harness/schemas/goal-v1.schema.json`. Um goal só **seleciona**
  um check aprovado por ID do registry H3 (`id`, `check_id`, `owner`, `schedule` daily|weekly|manual, `timeout_seconds`,
  `on_failure="alert_only"`, `runbook`); `additionalProperties:false` recusa qualquer campo `command`/`shell`/`argv`/`env`.
  Validação estrita pelo validador H2 (`parse_json_strict`, `PureSchemaValidator`, cross-check `jsonschema` quando presente)
  + regras do runner: check inexistente no registry H3, timeout acima de `min(timeout do check, máximo do sandbox)`,
  owner/runbook ausentes, id duplicado ou policy diferente → recusa. Único goal real: `pr-title-policy-daily`
  (`pr_title_policy`, owner `harness-maintainers`, ajustável pelo owner).
- **Runner** `docs/harness/bin/run-standing-checks.py` (`validate` / `run --mode manual|dispatched|scheduled`): lock
  exclusivo por goal em `<git-common-dir>/engram-standing-checks/<goal>.lock` (fora do checkout/runs root; concorrência
  do mesmo goal → `refused goal_locked`), export do SHA em diretório simples, execução **só** via
  `sandbox-adapter.run_isolated` (docker, sem rede, sem credenciais, capacidades `workspace_read/test_exec/
  sandbox_container/network_none`, nenhum `tcb_files`, nenhum writer), checagem de que o argv executado é o do registry,
  snapshot do checkout antes/depois (check que muda o checkout = `fail`), receipt (`run_sha`, tree, policy, toolchain,
  hashes de goals/schema/registry/catalog/TCB, outcome e logs) e, em qualquer status ≠ pass, **alerta LOCAL**
  (`delivery.sent=false`, owner + runbook + hashes). Nada é enviado; sem commit/ref/PR/mensagem (o módulo nem importa
  `subprocess`/rede, verificado por AST). Status: pass|fail|timeout|unavailable|refused|error; Docker ausente =
  `unavailable`; outcome/log ausente ou com hash divergente = `error`; nada selecionado = `not-run` (exit 4), nunca pass.
  Kill switch `ENGRAM_STANDING_CHECKS_DISABLED=1`.
- **Workflow** `.github/workflows/standing-checks.yml`: só `schedule` + `workflow_dispatch`, `permissions: contents: read`,
  sem secrets, ações fixadas por SHA (mesmos pins do repo), input do dispatch só via `env`, upload de recibos/alertas
  (sem os checkouts exportados). `scripts/check-workflow-supply-chain.py`: PASS. Rollback: desabilitar o workflow;
  receipts preservados; baseline do goal inalterado.
- **Testes** (`docs/harness/tests/test_standing_checks.py`, 52, offline, stub docker + repos git sintéticos): unknown
  check, campos shell/command extras, timeout acima da policy, owner ausente, concorrência, timeout/failure/artifact
  ausente/hash divergente/checkout mutado/Docker ausente nunca pass, modos manual/dispatched/scheduled, repo e
  refs inalterados, contrato read-only do workflow. RED contra runner permissivo (classify sempre pass, sem lock, sem
  checagens de policy): `Ran 52`, 19 falhas + 2 erros (a implementação foi escrita antes dos testes; o RED foi
  provado por mutante permissivo); GREEN 52/52. Lane `standing_checks` (floor exato 52) com `OFFLINE_LANE: PASS
  components=12`; `test_offline_lane.py` 17/17; `doctor.sh` OK (+ `standing_checks_workflow:read_only`).
- **NOT RUN**: execução real do workflow agendado no GitHub, `docker pull` da imagem pinada em CI, Docker real
  (smoke H3), goals para os lanes Q3 (fuzz/miri/mutants: precisam de toolchain Rust que a imagem do sandbox H3 não
  tem), `sensors.sh`, revisão independente (controller), canal real de entrega do alerta (exige autorização humana).

### H5 — rodada de correção 1 (revisão) (2026-10-05)

- **CI sem auto-lavagem (crítico)**: novo `docs/harness/bin/merge_gate_ci.py`. PR que toca `.github/workflows/**`,
  `.github/actions/**`, `docs/harness/{bin,tests,checks,schemas}/**`, `docs/security/**`, `scripts/check-*.py`,
  `scripts/test_check_*.py`, `scripts/ci.sh`, `scripts/ci-*.env` ou a matriz do security gate →
  `ci_policy_paths_changed`; blob de `ci.yml` do head ≠ base → `ci_workflow_changed`. Cada context required precisa de
  **exatamente um** check suite `github-actions`, ligado a um workflow run `path=.github/workflows/ci.yml`
  (`--workflow-runs`, `gh api .../actions/runs?head_sha=`); outro suite → `ci_context_multiple_suites`, suite sem
  ci.yml → `ci_workflow_unbound`. Re-run só dentro do mesmo suite (ruling a). Sonda RED: avaliador anterior dava
  `eligible` para Format failure no suite 1 + success no suite 2.
- **Receipt v2 (ruling b)**: `review-gate.sh` aceita `RECEIPT_VERSION=2` (superset; v1 segue aceito, chave v2 em v1 →
  malformed) e ganha o produtor `receipt-template` (valores recomputados + `<placeholders>`; placeholder não preenchido
  → `receipt-malformed` nos dois consumidores). Testado produtor → `post` (H1) e produtor → `merge-gate.py`.
- **Menores**: `bound` com `merge_policy` (`MERGE_POLICIES`), `ci_policy_sha256` e `evaluator_sha256`; arquivos do
  operador abertos uma vez com `O_NOFOLLOW` + `fstat`, ancestrais nossos/root sem escrita alheia salvo sticky; cópia
  do gate executada a partir dos bytes verificados; re-check também do `--integrated-ref`; re-check vazio com SHA cru
  no CI documentado; guarda de `${{` dentro de `run:` (teste + doctor); comentário de confiança do workflow
  suavizado; linhas ≤100 nos arquivos H5; fixtures movidas para `tests/merge_gate_test_support.py` (limite 800).
- **Testes**: `test_merge_gate.py` 43/43; `test-review-gate.sh` 38 testes, 245 asserções, 0 falhas; lane
  `OFFLINE_LANE: PASS components=12 checks=523` (floors exatos `review_gate` 38, `merge_gate` 43); `test_offline_lane`
  18/18; `doctor.sh` OK; supply chain PASS.
- **NOT RUN**: workflow real no GitHub, `gh api` real, `sensors.sh`, revisão independente.

### H5 — rodada de correção 2 (revisão) (2026-10-05)

- **Entradas dos jobs required protegidas**: `.github/**` inteiro, `.cargo/**`, `.config/**`, `scripts/**`,
  `docs/quality/**`, `benches/results/**`, `tests/fixtures/retrieval_quality/**` e configs da raiz (`deny.toml`,
  `.gitleaks.toml`, `.gitleaksignore`, `.semgrepignore`, `rust-toolchain*`, `rustfmt.toml`, `clippy.toml`,
  `Makefile`, `justfile`), além do que já estava. Todo arquivo citado nos steps do fecho de jobs required do `ci.yml`
  **da base** é protegido dinamicamente; teste de deriva falha se um arquivo citado pelo `ci.yml` atual (26 hoje,
  ex.: `scripts/generate-mcp-reference.sh`, `.github/codeql/codeql-config.yml`) escapar da lista estática.
- **Menores**: fingerprint das constantes de CI fixado em `MERGE_POLICIES` (`policy_mismatch` se divergir);
  `REVIEW_CONTEXT` vira placeholder no `receipt-template`; teste renomeado mostra que H1 (marcador) e H5 (review-v2)
  exigem artefatos e receipts separados; reabrir/`workflow_dispatch` → segundo suite → `ci_context_multiple_suites`
  permanente, recuperação por commit novo (documentado); redação "privado" corrigida; linhas novas ≤100.
- **Testes**: RED (CI module anterior: 12 falhas de config + deriva + dinâmico); GREEN `test_merge_gate.py` 47/47,
  `test-review-gate.sh` 38/245/0, lane `components=12 checks=527` (`merge_gate` floor exato 47), doctor OK.
- **NOT RUN**: workflow real, `gh api` real, `sensors.sh`, revisão independente.

## INT — Integração das lanes, Q1b e TODOs de integração (2026-10-05)

- **Merge**: `ee8c4ea` `chore(harness): merge improvement lane P into lane R` (`--no-ff` de
  `claude/improvement-lane-p` `3a373e7` na branch `claude/engram-improvement-plan-edb43d`).
  Único conflito, `docs/quality/retrieval-performance-policy.md`: mantidas as duas seções (lane
  de integridade histórica do Q1a primeiro, depois o runner do Q7). `progress.md` passa a linkar
  o log da lane R. Nenhum teste removido.
- **Q1b** (`a2fd341`, correção `11a48a1`): workflow separado e **não required**
  `.github/workflows/quality-candidate.yml` (job "Candidate Quality Evidence (report-only, not
  required)"), fora do `ci.yml` e de todo `needs:`. Captura, runner e o novo consumidor
  `scripts/consume-quality-candidate.py` (regra de consumo do Q1: 0 aceito, 1 rejeitado, 3 íntegro
  mas floors não aceitos) rodam de um worktree confiável na âncora (`merge-base` com
  `origin/main`; primeiro pai em push na main), com `--require-supervisor`
  (`${GITHUB_RUN_ID}-${GITHUB_RUN_ATTEMPT}`) e `--floors-anchor`. Sem `continue-on-error`:
  falha de captura/runner/integridade reprova o job; exit 3 vira warning (fase report-only).
  Bootstrap (âncora sem o runner, caso de `main` hoje) falha explícito. Testes unitários
  bloqueantes no job required `Test (ubuntu-latest)` e no passo 5 de `scripts/ci.sh`.
  Simulação local do workflow (blocos `run:` extraídos do YAML): bootstrap contra `main` real
  falhou explícito como esperado; caminho completo (candidato `33cf8a1`, âncora `a2fd341`) deu
  runner `status: pass`, `anchored: true`, `accepted: false`, `extract_mixed` ratio 0.289;
  a simulação revelou que o passo do veredito sob `bash -e` reprovava no exit 3 — corrigido
  em `11a48a1` com teste que executa o texto real do passo.
- **SDK ratchet** (`b8e693a`): baseline e checker nos `paths:` dos workflows live de Python e
  TypeScript; unit tests + `--only typescript` no job `Test` (instala `python3-httpx`) e no
  passo 5 do `ci.sh`.
- **Exceções** (`a231cdb`): `cargo audit` com DB recém-baixado (`ef6173c`, 1290 advisories):
  0 vulnerabilidades, 2 warnings permitidos (0192 ttf-parser, 0253 lru). 0235/0221 já ausentes;
  registro do 0258 corrigido para o caminho só-libsql; novo registro governado do 0253 (lru via
  aws-sdk-s3, feature `cloud`, owner Ronaldo, expira 2026-12-31; follow-up: fixar
  `BehaviorVersion` e subir aws-sdk-s3/aws-config com canário de bucket); EXC-0001 removido
  (sem sujeito: só reqwest 0.12.28 no lock). `cargo deny check`: ok.
- **Docs gate** (`e121894`): links intra-doc privados/ambíguos corrigidos
  (`src/embedding/queue/drain.rs`, `src/observability/mod.rs`); `RUSTDOCFLAGS=-D warnings cargo
  doc` com features required passa com e sem `--document-private-items`.
- **Housekeeping** (`33cf8a1`): `.gitignore` do `target/` do spike C4; mapa de cobertura com
  G-1 (G1) e G-3 (Q2F) marcados como corrigidos e testes movidos para as seções 3.4/3.3.
- **Verificação** (HEAD `11a48a1`): `make ci` exit 0 (nextest 2234 passed, 2 skipped; passo 5:
  63 + 23 + 28 testes OK, ratchet TS 81 entradas, nenhuma nova; `OFFLINE_LANE: PASS components=12
  checks=527`); `sensors.sh` full `status=pass` (2026-10-05T18:11:26Z, 314s; recibos não
  commitados); `cargo audit` exit 0; `cargo deny check` ok; docs gate ok.
- **NOT RUN**: CI real no GitHub (sem push), workflow `quality-candidate.yml` no runner real,
  revisão independente humana dos floors/corpus v2, revisão independente do INT.

## FINALFIX — Correções da revisão final do branch (2026-10-05)

- **Origem**: revisão final do branch (Opus, `1952f3b..4868d49`): *Ready with fixes*, 0 Critical,
  5 Important, 11 Minor ([texto](../reviews/2026-10-05-comprehensive-improvement/final-review.md)).
  Uma onda de correção, base `4868d49`, sem re-revisão ao commitar.
- **Important 1** (`08d1a9e`, TDD): `engram-cli mcp install` cria temp e backups já com o modo do
  config (0600 se novo) antes de qualquer byte, estreita backup reutilizado e grava através de
  config em symlink (o link sobrevive).
- **Important 2** (`bbaffc7`): catálogo de `memory_ingest_media` declara o limite do `file_hash`
  global entre workspaces; `ttl_days` com `minimum`/`maximum`; `docs/MCP_TOOLS.md` regenerado.
- **Important 3** (`ae22b11`): `CHANGELOG.md` [Unreleased] lista as mudanças visíveis ao cliente.
- **Important 4**: registro de revisão + arquivo do ledger SDD, briefs, reports e revisão final em
  [`reviews/2026-10-05-comprehensive-improvement/`](../reviews/2026-10-05-comprehensive-improvement/README.md);
  tabela do `progress.md` com todas as tasks; `SPEC.md` Status; artefato do doctor com escopo
  explícito (revisões SDD, **sem** `review-gate.sh post` com receipt).
- **Important 5** (`3e788b8`, TDD): floors exatos na lane offline (76/18/39/40/39/19) e teste de
  contrato "um abaixo do floor reprova" para os 12 componentes.
- **Minors**: gRPC sem payload de panic no status (`a896883`, TDD); `GATE_SCRIPT` absoluto antes
  do `cd` (`de39ca8`, TDD); `.gitattributes -whitespace` (`c623580`); upload-artifact v7 no
  standing-checks (`6544f71`); SHA sintético de PR e baselines não ancorados (`13ceac1`);
  CODEOWNERS (`af5881d`); caminhos relativos no plano e panic de hooks qualificado (`12e0835`).
- **Arquivo do registro** (`bdf4620`): ledger SDD, 29 briefs, 29 reports e revisão final copiados
  (caminhos de home/scratch redigidos, espaços finais removidos; sem pacotes `.diff`); gitleaks
  nos 12 commits da onda: sem vazamentos.
- **Verificação** (HEAD `bdf4620`): `make ci` exit 0 (nextest 2241 passed, 2 skipped; `OFFLINE_LANE:
  PASS components=12 checks=529`); `sensors.sh` full `status=pass` (2026-10-05T19:07:34Z, 328s;
  recibos não commitados); `doctor.sh` OK sem warning de review; `run-offline-lane.sh` PASS com
  todos os floors exatos; `git diff --check 1952f3b HEAD` exit 0; `cargo audit` exit 0 (1290
  advisories, 0 vulnerabilidades, 2 warnings permitidos); `cargo deny check` ok; docs gate
  (`RUSTDOCFLAGS=-D warnings cargo doc ... --document-private-items`) e
  `generate-mcp-reference.sh --check` ok.
- **Desvio**: o assunto de `a896883` usa o escopo `grpc`, fora da lista de
  `check-commit-msg.sh` (sem amend por regra; corrigível no squash).
- **NOT RUN**: re-revisão independente desta onda; CI real no GitHub; `review-gate.sh post`.
