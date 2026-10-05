# Gates — Sensores, Thresholds e Critérios (Engram)

Estes gates existem para manter a memória operacional do Engram confiável: o time precisa confiar que sensores, review e artefatos refletem o estado real da base de conhecimento e da superfície MCP.

Três camadas de verificação:

1. **Sensores determinísticos** (`sensors.sh`) — locais, rápidos, reproduzíveis.
2. **Review gate cross-CLI/cross-model** (`review-gate.sh`) — julgamento independente.
3. **Checklist humano em PR** — itens que nenhum gate automatizado cobre sozinho.

## Camada 1 — Sensores Determinísticos

Wrapper principal: `bash docs/harness/bin/sensors.sh`

Ele executa (em ordem):

| # | Sensor | Comando / Threshold | Action on FAIL |
|---|--------|---------------------|----------------|
| 1 | fmt | `cargo fmt --all -- --check` (exit 0) | block; rodar `cargo fmt --all` |
| 2 | clippy | `cargo clippy --all-targets --no-default-features $CI_REQUIRED_FEATURE_ARGS -- -D warnings` | block; fix warnings |
| 3 | test_lib | `cargo test --profile ci --no-default-features $CI_REQUIRED_FEATURE_ARGS --lib --tests` | block; investigar flakiness ou feature drift |
| 4 | test_integration | `cargo test --profile ci --no-default-features $CI_REQUIRED_FEATURE_ARGS --bin engram-server` | block; investigar regressão ou flakiness de integração |
| 5 | test_integration_watch | `cargo test --profile ci --no-default-features $CI_REQUIRED_FEATURE_ARGS --bin engram-watcher` | block; investigar regressão ou flakiness de integração |
| 6 | wasm_target | `rustup target list --installed | grep -qx "wasm32-unknown-unknown"` | block; instalar `wasm32-unknown-unknown` |
| 7 | wasm_all_targets | `cargo check -p engram-wasm --all-targets` | block; corrigir falha no WASM crate |
| 8 | wasm_wasm_target | `cargo check -p engram-wasm --target wasm32-unknown-unknown` | block; corrigir build WASM |
| 9 | doc | `RUSTDOCFLAGS="-D warnings" cargo doc --no-default-features $CI_REQUIRED_FEATURE_ARGS --no-deps --document-private-items` | block; atualizar docs |
| 10 | ref_check | `./scripts/generate-mcp-reference.sh --check` | block; atualizar referência MCP |
| 11 | harness doctor | `bash docs/harness/bin/doctor.sh` | block; corrigir drift no harness |
| 12 | PR title policy | `bash docs/harness/bin/pr-title-policy.sh --title "<title>"` rejeita marcador `[codex]` | block; renomear PR/commit de handoff |
| 13 | shell syntax | `doctor.sh` executa `bash -n` nos scripts do harness | block; corrigir sintaxe shell |
| 14 | shellcheck opcional | `doctor.sh` executa `shellcheck -x` nos scripts quando o binário está instalado | warn se instalado e falhar; skip explícito se ausente |
| 15 | (opcional/extensível) | snapshot tests, property tests, embedding cache bounds, etc. | block conforme threshold |

O script `sensors.sh` grava o resultado parseável mais recente em
`docs/harness/.sensors-last` (status, timestamp, `duration_sec`, exclusões,
etc.) e também anexa cada execução em `docs/harness/.sensors-log` para histórico
de medição.

### Sensors measurement log

`docs/harness/.sensors-log` é JSON Lines append-only e usa
`schema_version="sensors-log-v1"`. Cada linha deve conter:

- `timestamp` — UTC RFC3339 do fim da execução.
- `tool` — sempre `sensors`.
- `mode` — `full`, `quick`, `docs`, `mcp` ou `baseline`.
- `status` — `pass`, `pass_with_exclusion` ou `fail`.
- `duration_sec` — duração inteira não negativa.
- `ci_status` e `doctor_status` — status das duas camadas principais.
- `ci_steps` — objeto por etapa (`fmt`, `clippy`, `test_lib`, `test_integration`,
  `test_integration_watch`, `wasm_target`, `wasm_all_targets`,
  `wasm_wasm_target`, `doc`, `ref_check`) com `pass|fail|not_run`.
- `ci_command` — resumo curto do comando executado, sem logs brutos.
- `exclusion` — `null` ou `{sensor, known_issue, reason}`.
- `artifacts` — inclui `docs/harness/.sensors-last` como estado leve atual.

Rotação: antes de anexar uma nova linha, `sensors.sh` rotaciona o arquivo quando
ele atinge `SENSORS_LOG_MAX_BYTES` (padrão: `1048576`) e mantém
`SENSORS_LOG_ROTATIONS` gerações (padrão: `5`), como
`.sensors-log.1`, `.sensors-log.2`, etc. `doctor.sh` valida o JSONL quando o
arquivo existe.

### Métricas de tendência de sensores (`harness-stats.sh`)

`bash docs/harness/bin/harness-stats.sh` analisa `.sensors-log` e calcula métricas de execução:

- janela móvel (`--window N`, padrão `30`, `0=all`),
- contagens por status e taxa de sucesso,
- estatísticas por `mode` (executações e duração média),
- transições recentes que podem indicar flakiness (ex.: `pass`→`fail`),
- último estado conhecido de `ci_status` e `doctor_status`.

Formato de uso:

```bash
bash docs/harness/bin/harness-stats.sh               # saída humana
bash docs/harness/bin/harness-stats.sh --json         # saída JSON (`sensors`/`harness-stats` metrics envelope)
```

O script de métricas não altera estado do harness (somente leitura de `.sensors-log`).

Saídas JSON opt-in para scripts do harness devem seguir
[`JSON_OUTPUTS.md`](./JSON_OUTPUTS.md): um único objeto JSON em stdout,
vocabulário de status estável, exit code preservado e nenhum segredo ou dump de
ambiente. O output humano continua sendo o default.

### Required aggregate security gate

The PR-visible `Security Gate` job in `.github/workflows/ci.yml` aggregates
Cargo Audit, Cargo Deny, governed exception-policy validation, CodeQL,
Semgrep, Gitleaks, and AgentShield. The decision is tri-state
(`scripts/check-security-gate.py`):

| Verdict | When | Gate result |
|---|---|---|
| `pass` | every constituent is `success` | green, reported as `PASS` |
| `neutral` | the only non-success states are `skipped` constituents explicitly listed for the event in `tests/fixtures/security_gate_matrix.json` | green, reported as `NEUTRAL`, never as `PASS` |
| `block` | anything else: `failure`, `cancelled`, `timed_out`, missing result, unknown state, or an unauthorized `skipped` | red |

`pull_request`, `push`, `schedule` and `workflow_dispatch` may never allow a skip
(the matrix validator rejects it), and a matrix scenario cannot grant a skip its
event does not allow. The matrix must keep one scenario for each case: all-pass,
constituent failure, cancelled, timed-out, missing, unauthorized skip and allowed
skip (neutral); removing a case fails the checker.

Branch protection itself is not changed by repository automation. The live
protection of `main` (read-only check, 2026-10-05) requires `Format`, `Clippy`,
`Documentation`, `Test (ubuntu-latest)`, `Security Audit` and `Cargo Deny`.
`Security Gate` is **not** itself a required context: the already-required
`Test (ubuntu-latest)` job `needs: security-gate`, so a security failure blocks
that required context transitively. The checker fails when that `needs:` is
removed (`required_dependency_job` in the matrix), when the aggregate stops
running with `if: always()`, or when it stops depending on any constituent.
Verify the live, read-only chain with:

```bash
gh api repos/aiconnai/engram/branches/main/protection/required_status_checks \
  > .omo/evidence/task-18-required-contexts.json
python3 scripts/check-security-gate.py \
  --matrix tests/fixtures/security_gate_matrix.json \
  --required-contexts .omo/evidence/task-18-required-contexts.json \
  --workflow .github/workflows/ci.yml
```

The checker also exposes `--self-test-failure` (failure, cancelled, timed-out,
missing and unauthorized skip all fail closed) and `--self-test-unrequired`
(removing the required-context dependency fails closed).

#### Scanner execution, SARIF publication and findings policy are separate

1. **Execution** - did the scanner run and exit cleanly? A scanner that did not run
   is a `block` unless the supervisor explicitly authorizes the skip (`neutral`).
2. **Publication** - uploading SARIF to GitHub code scanning is a trusted-event
   action (`push`/`schedule`) in `codeql.yml`, `semgrep.yml`, `gitleaks.yml` and
   `agentshield.yml`. A successful upload proves nothing about the absence of
   findings and is never read as a clean scan.
3. **Findings policy** - `scripts/check-security-findings.py` decides from the
   SARIF itself. `ci.yml` applies it to CodeQL (`codeql-security`), whose exit code
   is 0 even with high findings. Semgrep (`--error`) and Gitleaks (`--exit-code 1`)
   already fail on any finding and AgentShield on `fail-on: high`; their SARIF is
   retained as artifacts (30 days) but is not re-judged.

Findings policy (mechanical, fail-closed; fixtures in
`tests/fixtures/security_findings/`, tests in `scripts/test_check_security_findings.py`):

- A **high** finding (`level: error`, or `security-severity >= 7.0`, also resolved
  through the rule default) blocks even when the scanner exit code is 0.
- An **approved exception** needs `owner`, `approved_by`, `rationale` and an
  unexpired `expires` (at most 90 days ahead), matched on scanner + rule id (+ path)
  in `docs/security/finding-exceptions.toml`. Any invalid record rejects the whole
  file. SARIF `suppressions` embedded in the report are ignored: a payload cannot
  approve itself.
- SARIF **missing**, **malformed** (not JSON, wrong version, no runs, no results
  array), from another tool, with **no revision provenance**, or **stale** (revision
  differs from the expected SHA) blocks.
- Identity comes from the supervisor on the command line (`--scanner`, `--tool`,
  `--expected-sha`, `--scanner-exit`, `--allowed-skip`), never from the payload.
- A run with no revision provenance (CodeQL with `upload: never` omits it) is accepted
  only when the supervisor passes `--checkout-dir` and that checkout's HEAD equals
  `--expected-sha` (job-checkout attestation, recorded in the reasons). Explicit
  provenance, when present, must still match; attestation never excuses findings.
- Exit codes: 0 for `pass`/`neutral`, 1 for `block`, 2 for usage errors.

#### Workflow supply chain and pull-request exposure

`scripts/check-workflow-supply-chain.py` (run inside the `security-gate` job, with
`scripts/test_check_workflow_supply_chain.py`) enforces:

- every third-party `uses:` is pinned by a full commit SHA;
- every container image is pinned by `@sha256:` digest, or is listed in the
  expiring ledger `docs/security/supply-chain-pins.toml` (owner, reason, expiry;
  stale or expired entries fail);
- a job that can run on `pull_request` has no write permission and uses no secret
  other than `GITHUB_TOKEN`; publication/comment jobs are excluded from pull
  requests with `if: github.event_name != 'pull_request'` (or run only on
  `push`/`schedule`);
- `pull_request_target` is never used.

Measurement jobs are read-only (`ci.yml` `bench` and `codeql-security`, and the
`scan` jobs of the standalone scanner workflows); publication lives in separate
jobs (`bench-publish`, the `publish` jobs, `agentshield.yml` `scan-publish`).

#### Advisory exceptions

`docs/security/advisory-exceptions.toml` stays in lockstep with `.cargo/audit.toml`
and `deny.toml` (`scripts/check-security-exceptions.py`). Besides owner/expiry
parity, an expiry more than 90 days ahead is rejected. An advisory that a
dependency update can fix is never renewed: it is handed to the task that owns
`Cargo.toml`/`Cargo.lock`. The gate may stay red until that update lands; renewing
an exception to hide a fixable advisory is not allowed.

### PR Title Policy

Wrapper: `bash docs/harness/bin/pr-title-policy.sh`

Este gate impede handoffs ou PRs com marcador de ferramenta no título. O padrão
bloqueado é case-insensitive e tolera espaços dentro dos colchetes:
`[codex]`, `[ Codex ]`, `[ CoDeX ]`.

Modos:

- `--title "<title>"` — valida um título explícito.
- `--stdin` — lê o título de stdin.
- `--current-pr` — lê o título do PR atual via `gh pr view`.
- Sem argumentos, usa `PR_TITLE` quando a variável estiver definida.

Exit codes: `0` para título aceito, `4` para marcador `[codex]` rejeitado,
`2` para erro de uso e `3` quando `--current-pr` exige `gh` indisponível.
`sensors.sh` executa casos positivos e negativos determinísticos para manter
esse contrato vivo.

### Version-Control Gate

Wrapper: `bash docs/harness/bin/vc-gate.sh`

Este gate e opcional durante desenvolvimento normal, mas recomendado em
fronteiras de issue e obrigatorio antes de releases manuais.

Modos:

- `status [ISSUE]` — mostra branch, `HEAD`, dirty/untracked count e estado `jj`
  quando disponivel.
- `start ISSUE` — bloqueia iniciar uma nova issue com worktree sujo, a menos
  que `--allow-dirty-current-issue` torne a atribuicao explicita.
- `done ISSUE` — exige worktree limpo e evidencia recente de commit Git ou
  descricao `jj` mencionando a issue.
- `release VERSION` — exige worktree limpo, versao do `Cargo.toml` alinhada e
  tag `vVERSION` apontando para `HEAD`; use `--allow-untagged` apenas para
  checagens pre-tag antes do dry-run.

Contrato:

- `jj` e permitido como camada local para evoluir, splitar e descrever trabalho
  de issue.
- Git continua canonico para commits de release, tags e `cargo publish`.
- O gate nao cria commits, nao roda `jj new`, nao move tags e nao publica crate.
- Falhas de `vc-gate.sh release` bloqueiam qualquer tentativa de publish.

### PR Title Compatibility Wrapper

Wrapper legado: `bash docs/harness/bin/check-pr-title.sh`

`pr-title-policy.sh` é a implementação canônica deste gate. O wrapper
`check-pr-title.sh` existe para comandos antigos (`--title` e `--pr`) e delega a
validação final para `pr-title-policy.sh`; portanto ele compartilha o mesmo
contrato de rejeição do marcador `[codex]`, incluindo exit code `4`.

Este gate é obrigatório antes de criar ou editar PRs por automação.

Contrato:

- PR titles não podem conter o marcador `[codex]`.
- PR titles devem descrever a mudança, não a ferramenta ou agente que a criou.
- `doctor.sh` faz self-test do caminho permitido, do caminho bloqueado e do exit
  code canônico.

Exemplos:

```bash
bash docs/harness/bin/pr-title-policy.sh --title "align lifecycle hook contracts"
bash docs/harness/bin/check-pr-title.sh --title "align lifecycle hook contracts"
bash docs/harness/bin/check-pr-title.sh --pr 123
```

### Exclusões Documentadas (Contrato Rigoroso)

Exclusão só existe para **dependências externas temporárias** (ex.: API de embedding paga indisponível, serviço de terceiros em outage).

Contrato mínimo:

- Apenas sensores específicos (hoje: possivelmente embedding-related ou watcher integration que exige ambiente GUI).
- `--exclude-sensor <name>`
- `--known-issue docs/harness/known-issues/YYYY-MM-DD-<slug>.md` (arquivo deve existir)
- `--reason "motivo curto e específico"`
- Registro **prévio** do known-issue + razão exata em `progress.md` e no active plan log.
- Fechamento de código de produção **exige** run limpo real (sem exclusão) a menos que ADR ou governança equivalente autorize.

`sensors.sh` bloqueia exclusão se a evidência de registro não existir.

Exemplo:

```bash
bash docs/harness/bin/sensors.sh \
  --exclude-sensor embedding-api-smoke \
  --known-issue docs/harness/known-issues/2026-05-30-cohere-outage.md \
  --reason "Cohere API outage; known issue registrado"
```

Isso grava `status=pass_with_exclusion` em `.sensors-last`. Não é evidência suficiente para merge de produção.

### Fake-Success Patterns (Específicos de Engram)

O review-gate é prompted explicitamente para caçar estes (sensores verdes mas sistema quebrado em produção ou para agentes):

1. **Tests passam só com `local-embeddings` mas CI Linux usa features remotas** — ONNX ou embedding provider ausente em CI, mas `cargo test --features local-embeddings` passa localmente.
2. **MCP protocol tests usam fixtures antigas** — `tests/mcp_protocol_tests.rs` ou golden files não cobrem nova tool ou mudança de schema de request/response.
3. **Schema version atualizada mas testes de migração hardcoded falham** — `storage/migrations.rs` bump + alguns testes em `tests/` ou `src/storage/` ainda têm versão antiga.
4. **Clippy limpo + `unwrap()` em path quente de MCP handler** — allowlist de clippy esconde o problema; handler de tool crítica pode panic em input adverso.
5. **Snapshot tests verdes mas attestation/Merkle mudou** — `src/snapshot/` ou `tests/snapshot_attestation.rs` não reflete mudança em crypto ou serialization.
6. **Hooks (session_end etc.) não testados em integração** — comportamento de consolidação/auto-tag muda, mas só unit tests isolados passam.
7. **Cargo doc passa mas MCP reference gerada está stale** — `scripts/generate-mcp-reference.sh --check` falha silenciosamente ou é pulado em "dev mode".
8. **Review gate roda contra diff que exclui harness artifacts, mas o prompt injetado está incompleto** — self-referential ou prompt drift.
9. **Rustdoc warnings tratados como allow em CI local mas -D warnings no gate** — flags diferentes produzem falso verde.
10. **Identity alias normalization ou scope grants mudam sem atualização de testes de propriedade** — property tests ou `tests/` não cobrem o novo comportamento.
11. **Security boundary drift** — docs ou scripts passam a sugerir execução autônoma, sandbox implícito, mounts de credenciais, ou import da pipeline C/C++/ASAN sem ADR e target contract.

O prompt do review-gate inclui esta lista + instrução para buscar evidência concreta no diff.

### Negative Scope Gate

`docs/harness/WHAT_WE_DONT_DO.md` define escopo negativo para mudanças de harness.

O review-gate deve marcar como `[HIGH]` ou `[BLOCKER]` qualquer mudança que:

- Faça product work dentro de uma task de harness.
- Enfraqueça o gate completo sem registrar decisão explícita.
- Remova código, dependências, docs ou scripts baseado só em evidência estática.
- Use exclusões de sensor para mascarar falha de produção.

### 12207-Inspired Tailoring Checklist

Uma cópia local não versionada, por exemplo `docs/ieee-12207.md`, pode ser
usada como referência privada de padrões de processo de ciclo de vida. Esse
arquivo é ignorado pelo Git, não deve ser distribuído no repositório, e o Engram
não reivindica conformidade com a norma nem copia texto, prompts ou checklists
dela para o harness. A adoção é sempre tailoring local: transformar conceitos
em critérios verificáveis do Engram.

Quando uma mudança de harness, codificação ou review citar essa referência ou
alterar processo de engenharia, registre em `progress.md`, no plano ativo ou no
Review Canvas aplicável:

- **Escopo e circunstâncias** — qual lacuna local está sendo tratada, quais
  stakeholders/paths são afetados e o que fica explicitamente fora de escopo.
- **Área de ciclo de vida** — planejamento/controle, decisão, risco,
  configuração/informação, medição, QA, verificação, validação, operação ou
  manutenção.
- **Racional de decisão** — alternativas consideradas, opção escolhida,
  premissas e motivo para não adotar a referência como pipeline ou conformidade.
- **Risco e threshold** — risco mitigado, sinal que indicaria regressão e ação
  esperada se o sinal piorar.
- **Medição** — necessidade de informação, medida ou artefato observado,
  frequência/custo e onde a evidência fica armazenada.
- **Evidência separada** — verificação objetiva (`doctor.sh`, sensores, testes,
  diff checks) separada de validação de fitness para o usuário/stakeholder
  quando aplicável.
- **Traceability** — links entre requisito/intenção, arquivo alterado, canvas,
  review e progresso, com plano de rollback se algum gate for enfraquecido.

O review-gate deve marcar como `[HIGH]` uma adoção 12207 sem esse registro e
como `[BLOCKER]` quando a mudança também tocar gates, invariants ou scripts
process-critical sem evidência de segurança e reversibilidade.

### Reference Intake Checklist

`docs/harness/REFERENCE_INTAKE.md` defines the intake contract for external
harness references, standards, articles, repos, benchmark suites, tool catalogs,
and awesome lists. Use it whenever an external source shapes Engram harness
policy, gates, taxonomy, skills, reviewer prompts, or exception handling.

Minimum evidence for process-affecting adoption:

- source identity and date read;
- source type and license boundary;
- local harness primitive affected;
- placement decision and rejected adjacent placements when ambiguous;
- what is explicitly not imported, vendored, executed, or treated as
  authoritative;
- verification evidence or reason executable verification does not apply.

The review-gate should mark missing reference-intake evidence as `[HIGH]` when
a harness/process change relies on an external source, and `[BLOCKER]` when the
change copies licensed material, weakens gates, imports autonomous execution, or
lets an external source override local invariants.

### Review Canvas Requirement

Mudanças complexas exigem Review Canvas em `docs/harness/canvas/YYYY-MM-DD-<task-id>.md` antes de post-review.

Triggers:

- Mais de 200 linhas não geradas.
- Storage schema, migrations ou invariants de dados.
- Mudança na superfície MCP.
- Hooks, intelligence, consolidation, embeddings, sync ou attestation.
- Contratos públicos dos SDKs.
- Nova dependência externa, backend, transport, cache, fila ou serviço de rede.
- Mudança em harness gates, invariants, bootstrap, sensores ou policy.

O canvas deve conter abordagens consideradas, hot-path complexity, ao menos dois edge cases e tabela de breakage risk. Canvas é evidência, não aprovação.

### Sensor Modes

`bash docs/harness/bin/sensors.sh` sem argumentos continua sendo o full canonical gate.

These optional lanes do not replace the full gate; gates preserve full sensor gate.

Modos opcionais:

- `full` — gate completo canônico.
- `quick` — fmt, check e doctor.
- `docs` — referência MCP e rustdoc.
- `mcp` — referência MCP e testes de protocolo MCP.
- `baseline` — `baseline.sh` e doctor.

Essas lanes opcionais não substituem o gate completo para merge, handoff ou completion claims.

### Offline mandatory lane (H2)

`bash docs/harness/bin/run-offline-lane.sh` é a lane offline **obrigatória** do harness
(sem rede, sem credenciais, fixtures sintéticas). Ela roda, fail-closed:

1. `python3 -m unittest discover -s docs/harness/tests -p 'test_validate_evidence.py'`;
2. `python3 docs/harness/bin/validate-evidence.py --self-test`;
3. `bash docs/harness/bin/test-fixtures.sh`;
4. `bash docs/harness/bin/test-check-live-state.sh`;
5. `bash docs/harness/bin/test-review-gate.sh`;
6. `python3 -m unittest discover -s docs/harness/tests -p 'test_offline_lane.py'` (contrato
   fail-closed do próprio runner).
7. `python3 -m unittest discover -s docs/harness/tests -p 'test_sandbox_adapter.py'` (adaptador de
   sandbox H3, offline com `docker` fake; o smoke com Docker real **não** faz parte da lane);
8. `python3 -m unittest discover -s docs/harness/tests -p 'test_context_budget.py'` (H6: orçamento
   do resumo vivo e do bootstrap, retenção byte a byte do histórico, links/âncoras, ordem de leitura).
9. `python3 -m unittest -v docs/harness/tests/test_runner.py docs/harness/tests/test_scope.py
   docs/harness/tests/test_evidence_integrity.py` (H4: runner, scope e evidência, `docker` stub);
10. `python3 -m unittest discover -s docs/harness/tests -p 'test_merge_gate.py'` (H5: avaliador
   de merge-policy read-only, produtor de receipt e contrato do `agent-evidence.yml`).

Um componente só conta como verde com exit 0, resumo próprio encontrado com contagem > 0 e sem
falhas, contagem **no piso** de tripwire (apagar testes em silêncio falha; subir o piso é o
caminho normal, baixá-lo exige review), e sem skip/`NOT RUN` (o review-gate tolera no máximo
`HARNESS_LANE_MAX_NOT_RUN` lacunas de plataforma, default 0). `jsonschema` precisa estar
instalado: a metade "presente" da paridade jsonschema presente = ausente é exercida de verdade,
nunca pulada (a ausência é simulada com import blocker, sem desinstalar nada).

Wiring (o `doctor.sh` falha se qualquer item sumir, e comentário não conta):

- `sensors.sh quick` (`run_offline_lane`) e `sensors.sh` full (passo `offline_lane`);
- `scripts/ci.sh`;
- passo incondicional do job `Test (ubuntu-latest)` em `.github/workflows/ci.yml`. Esse job foi
  escolhido porque é required na proteção live de `main`; `Harness Contract` **não** é required
  live (audits/2026-10-02-improvement-baseline.md §4), então não carrega a lane. O passo instala
  `python3-jsonschema` e `acl` por apt antes de rodar.

Escopo de confiança de `validate-evidence.py`: valida **estrutura e semântica**, nunca
autenticidade. SHA candidato esperado, versão de política, hash do catálogo de checks, diretório
de logs e a task vêm do **chamador** (`--expect-candidate-sha`, `--expect-policy-version`,
`--expect-catalog-sha256`, `--logs-dir`, `--task`; `--require-expectations` os torna
obrigatórios) e nunca do payload. Artefatos `*-v1` são históricos: validam estrutura, aparecem
como `NOTE[HISTORICAL_V1]` e não viram evidência confiável; `*-v2` são os schemas endurecidos.
Nenhuma fixture escrita por agente é evidência confiável.

### Context budget and live-state enforcement (H6)

O estado de retomada é curto e o histórico é retido, não apagado (detalhe, medições e a
decisão #152 em [`context-budget.md`](./context-budget.md)):

- `progress.md` é o **resumo vivo, ≤150 linhas**; o texto anterior está byte a byte em
  `progress-history.md` (marcador `BEGIN-VERBATIM` com tamanho e SHA-256, verificado por teste).
  O `doctor.sh` falha se o resumo passar de 150 linhas.
- `bootstrap.sh` imprime ≤50 linhas (doctor e `test_context_budget.py`; duro em qualquer ambiente) em <500 ms
  (só o teste; `HARNESS_BOOTSTRAP_MAX_SECONDS`, 2,0 s quando `CI=true`). A **ordem de
  leitura obrigatória** é a mesma em bootstrap, `AGENTS.md`, `CLAUDE.md` e `INVARIANTS.md`; o teste
  falha se divergirem. Reduzir contexto nunca remove autoridade obrigatória.
- `check-live-state.sh --structural` (rodado pelo `doctor.sh` no `progress.md` real) exige campos,
  Active plan existente, review PASS autoritativo, linhas de reconciliação e Last commit **bem
  formado**; não exige HEAD exato nem o timestamp de `.sensors-last` (churn por commit e por run
  de sensores). Ancestralidade de HEAD é verificada quando possível: SHA bem formado que HEAD não
  alcança (squash/rebase reescrevem SHAs) e clone raso (`ancestor_check=skipped-shallow`) viram
  **warning** do doctor e linha `WARN` do checker, nunca falha nem pass silencioso;
  `--require-ancestor` é a variante dura, usada só por fixtures herméticos. A forma estrita (sem a
  flag) continua sendo a que fecha tasks.
- `check-doc-links.py` valida links relativos e âncoras (regras do GitHub); alvo ignorado por git
  conta como quebrado porque não existe num clone limpo.

### Required checks no GitHub (merge em `main`)

Os sensores locais confirmam o trabalho cedo; o GitHub re-confirma os mesmos
contratos como **required status checks** antes do merge. A proteção live de `main`
(leitura somente, 2026-10-05; `strict: true`) exige seis contexts:

- `Format`, `Clippy`, `Documentation`, `Test (ubuntu-latest)` (os jobs de CI
  baratos e determinísticos);
- `Security Audit` e `Cargo Deny` (exigidos diretamente);
- `Test (ubuntu-latest)` também bloqueia de forma **transitiva** via
  `needs: security-gate` (seção "Required aggregate security gate").

`Security Gate` e `Harness Contract` **não** são contexts required live.
`Harness Contract` é um gate leve (`bootstrap.sh` + política de título de PR, que só
rejeita o marcador literal `[codex]`); o `doctor.sh` não entra nesse job: roda no job
separado `Harness Doctor Advisory` (non-blocking, não required) e localmente. A proteção também não exige revisão de PR e `enforce_admins` é
`false`: "merge humano obrigatório" é regra de processo, não controle técnico.
Advisories abertas no lock (ex.: `RUSTSEC-2026-0285` em `rustls`) deixam
`Security Audit`/`Cargo Deny` vermelhos até a atualização de dependência; não se
renova exceção para esconder advisory corrigível. Code review automático é sinal
extra, nunca o único bloqueador.

### Baseline Snapshot

`baseline.sh` grava fatos estáticos baratos em `docs/harness/.baseline-last`.

Ele é evidência para drift review, não substitui `sensors.sh` (full), `sensors.sh` com lanes `docs/mcp/baseline`, ou review independente.

### Evidence-Only Audit

`quarterly-audit.sh` grava relatórios em `docs/harness/audits/` e atualiza `docs/harness/.quarterly-audit-last`.

Ele é evidence-only: não é pass/fail gate e não pode deletar, arquivar ou reescrever arquivos.

### Harness Script Guard

Mudanças em `docs/harness/bin/*` são process-critical.

O post-gate deve exigir evidência independente explícita para alterações nesses scripts. Prompt gerado, review advisory ou artefato sem `REVIEW_VERDICT` não é suficiente.

### Security Reference Harness Gate

Adaptações baseadas no `anthropics/defending-code-reference-harness` seguem
`docs/harness/security/anthropic-reference-harness.md`.

Hard rules:

- `doctor.sh` valida o anchor `ENGRAM-HARNESS-SECURITY-CONTRACT-v1` e os campos
  `DEFAULT_MODE=static_read_only`, `AUTONOMOUS_EXECUTION_REQUIRES_ADR=true`,
  `NO_CREDENTIAL_MOUNTS=true` e
  `TUNING_FILES=.claude/scan-extras.txt,.claude/fp-rules.txt`.
- O modo default é static/read-only: threat model, scan, triage e patch
  candidates sem execução de código alvo por agentes.
- `.claude/scan-extras.txt` e `.claude/fp-rules.txt` sao obrigatorios quando
  referenciados e vivem fora do texto central de INVARIANTS/GATES/POLICY.
- A pipeline autônoma da referência não é aceita como drop-in para Engram,
  porque o target padrão é C/C++ com ASAN.
- Qualquer execução autônoma contra Engram exige ADR prévio, sandbox forte
  (gVisor ou equivalente), egress restrito, nenhum mount de credenciais, e
  target contract Rust com build, proof, reproduce, regress e re-attack.
- `--dangerously-no-sandbox` é bloqueado para runs em código Engram ou máquinas
  de desenvolvimento com credenciais.
- Patches gerados por agente são drafts. Eles precisam de revisão independente,
  evidência de regressão apropriada e `review-gate.sh post` antes de upstream.

## Camada 2 — Review Gate

Ver `review-gate.sh` e `CODE_REVIEW_POLICY.md` para detalhes de execução e prompt.

Características chave:

- Pre: advisory (`GATE_STATUS: ADVISORY`, exit 0) e **nunca aprova**; findings são obrigatórios de ler.
- Post: hard gate **fail-closed** (detalhes na subseção abaixo). `PASS <resumo>` na primeira linha ou FAIL, **e** incluir sempre uma única linha `REVIEW_VERDICT: PASS|FAIL ...` para parser hard. O marcador sozinho é histórico: exit 0 exige receipt confiável vinculado ao escopo exato.
- Continuity: após FAIL, reruns injetam `[BLOCKER]`/`[HIGH]` anteriores relevantes (com ids estáveis para dedup).
- Exclusões automáticas de diff: `docs/harness/reviews/*`, `docs/harness/progress/*`, `target/`, `coverage/`, artefatos de build, etc. (anti self-referential loop).
- Timeout configurável via `REVIEWER_TIMEOUT_SECS`.
- Suporte a múltiplos backends via `REVIEWER_CLI` (claude-sonnet, codex, ollama, ou "manual" que só gera o prompt file). Claude Code Sonnet é o reviewer padrão; outros backends não são canônicos sem override explícito do owner.

Formato de output esperado do reviewer (primeira linha):

```
PASS no substantive issues for harness-bootstrap
```

ou

```
FAIL 2 blockers: missing doctor integration, prompt drift in review-gate
```

Parsing hard do gate também exige:

```text
REVIEW_VERDICT: PASS ...
```
ou
```text
REVIEW_VERDICT: FAIL ...
```

### Review gate fail-closed (H1): escopo, exit codes e receipt manual

Semântica de `review-gate.sh post` (substitui o comportamento antigo "sem review → exit 0"):

- **Fonte do diff é explícita.** Modo final = `--base REV --head REV` (ou `--range A..B`, duas
  pontas, sem `...`): todos os commits de `base..candidate`, com `base` ancestral de `candidate`.
  `post` sem escopo explícito é erro de uso — o gate nunca "adivinha o último commit". Modo
  preparação (`pre`/`scope --prepare`, sem range) = working tree vs `HEAD`, incluindo mudanças
  *staged-only* e arquivos untracked não ignorados. Caminhos são tratados com NUL (`-z`) e
  `--no-renames`, então rename/delete listam os dois lados; nomes com espaço, newline ou `-` inicial
  são seguros. O modo preparação mostra também o conteúdo *staged* quando index ≠ working tree ≠
  `HEAD`. O diff é calculado com `--text` (atributos `-diff`/binary, inclusive `.git/info/attributes`,
  não escondem conteúdo; binários aparecem como patch bruto), sem textconv/ext-diff, e com replace
  refs, grafts e arquivos de atributos neutralizados (`--no-replace-objects`, `GIT_GRAFT_FILE`,
  `core.attributesFile`): estado de `.git` gravável pelo writer não altera o que o gate enxerga.
  `node_modules/` **não** é excluído (pode conter código executável); só bookkeeping do harness e
  saída de build (`docs/harness/reviews|progress`, `target/`, `engram-wasm/target/`, `coverage/`).
- **Falha de git antes de qualquer verdict.** Range inválido/vazio, commit ausente, `git diff`/`git
  show` falhando → exit 4 sem ler o artefato de review. Texto de erro nunca entra como diff.
- **Provenance confiável (procedimento manual, até H4/H5 automatizarem).** Um receipt escrito pelo
  operador humano, guardado **fora** de qualquer worktree e do `.git` (path absoluto via `--receipt`
  ou `ENGRAM_REVIEW_RECEIPT`), vincula task, base, head, tree, sha256 do diff revisado, path +
  sha256 do artefato de review, sha256 do script do gate e identidade do operador. O gate recomputa base/head/tree/diff a
  partir do git e exige igualdade com o receipt **e** com os valores que o operador passa na linha de
  comando (`--expect-tree`, `--expect-diff-sha256`, `--expect-gate-sha256`, `--operator`). Receipt
  dentro do repo/worktree/`.git` (comparado por **identidade de filesystem**, não por grafia — variante
  de caixa em APFS não escapa), symlink, hard link, não regular, de outro dono, gravável por
  grupo/outros ou com ACL que concede escrita (receipt e diretório) é recusado. Nenhum JSON/marcador
  do writer se autentica sozinho.
- **PENDING distinto de PASS.** Review ausente, receipt ausente/não confiável/malformado ou
  divergente do escopo recomputado (inclui review "velho" reaproveitado após mudar um byte) →
  `GATE_STATUS: PENDING`, exit 3, mesmo com `REVIEW_VERDICT: PASS` no artefato.
- **Parser de marcador mantido (invariante 14), mais estrito**: exatamente uma linha
  `REVIEW_VERDICT: PASS|FAIL <resumo>`; zero marcadores, mais de um, ou o placeholder
  `<one-line summary>` do prompt → `INVALID_REVIEW`, exit 1. Prosa `PASS` sem marcador não conta.
- **Skip allowlist inalterada.** O gate só dispensa review mecanicamente o item 1 da seção
  *Skip Allowlist* (todo path alterado, nos dois lados de renames, é `docs/**/*.md` fora de
  `docs/harness/`, arquivos regulares) → `GATE_STATUS: SKIPPED_ALLOWLIST` (exit 0, **não** é PASS).
  Itens 2-4 (comment-only, formatting-only, test-only) exigem julgamento humano e nunca são
  dispensados automaticamente. Qualquer path em `docs/harness/**` — em especial `docs/harness/bin/*`
  — exige reviewer sempre; paths de bookkeeping excluídos do diff (`docs/harness/reviews|progress`,
  `target/` etc.) não "diluem" a allowlist e são listados como `EXCLUDED_PATH`.
- **O script alterado não autoriza a própria alteração.** O `post` **sempre** deve rodar a partir de
  uma cópia do gate mantida fora de qualquer worktree, tirada de um revision confiável (`base`/`main`),
  apontada com `--repo <worktree>`: `git show BASE:docs/harness/bin/review-gate.sh >
  /dir/do/operador/review-gate.sh`. O sha256 desse script (impresso como `Gate script sha256`) é
  vinculado pelo receipt (`GATE_SCRIPT_SHA256`) e conferido com `--expect-gate-sha256`; um PASS vindo
  de um gate que mora dentro de um worktree do repo julgado é recusado (`gate-untrusted-location`,
  PENDING).
- **Como o próprio H1 é aceito.** O `base` de H1 (`cf6d969`) ainda não tem receipts nem gate
  fail-closed, então nenhum gate pode aceitar H1. A aceitação de H1 é a revisão independente do SDD
  mais a decisão do owner; a primeira mudança aceita *pelo* gate é a seguinte (a partir de H1 como
  `base` confiável).

| Exit | `GATE_STATUS` | Significado | Consumidor |
|-----:|---------------|-------------|------------|
| 0 | `PASS` | receipt confiável + marcador PASS ligados ao escopo recomputado | pode prosseguir |
| 0 | `SKIPPED_ALLOWLIST` | diff docs-only elegível; review não exigido | pode prosseguir; não citar como review |
| 0 | `ADVISORY` | `pre` / `scope`; não é aprovação | **não** aprova |
| 1 | `FAIL` | reviewer reprovou | bloquear |
| 1 | `INVALID_REVIEW` | zero/múltiplos marcadores, placeholder, marcador malformado | bloquear |
| 2 | `ERROR reason=usage` | uso inválido (sem range, flag desconhecida, task-id inválido) | bloquear |
| 3 | `PENDING` | falta evidência confiável (review/receipt ausente, não confiável ou stale) | bloquear; **nunca** tratar como PASS |
| 4 | `ERROR` (`invalid-range`, `missing-commit`, `git-failure`, `empty-scope`) | escopo não verificável | bloquear |

Consumidores (CI, doctor, agentes) devem aceitar apenas exit 0 com `GATE_STATUS: PASS` como review
aprovado; `SKIPPED_ALLOWLIST` só quando a política permite e deve ser registrado como skip.

**Runbook do operador (receipt manual).** Executado por um humano autenticado, fora do writer:

0. Copie o gate **de um revision confiável, para fora de qualquer worktree** (nunca use o script do
   working tree do writer) e guarde seu hash:
   `git show <BASE>:docs/harness/bin/review-gate.sh > $OPDIR/review-gate.sh; shasum -a 256 $OPDIR/review-gate.sh`.
   Todos os comandos abaixo usam `G="bash $OPDIR/review-gate.sh"` e `--repo <worktree>`.
1. `$G scope <task> --repo <worktree> --base <BASE> --head <HEAD>` — confira
   `BASE_SHA`, `HEAD_SHA`, `TREE_SHA`, `DIFF_SHA256`, lista de paths e `SKIP_ALLOWLIST`.
2. `$G post <task> --repo <worktree> --base <BASE> --head <HEAD>` (sem
   `--review-file`) gera o prompt `.raw`; um reviewer independente produz o artefato com
   `REVIEW_VERDICT:`. Inspecione a origem do review.
3. Escreva o receipt num diretório seu (`chmod 700`), fora do repo e de worktrees, modo `0600`/`0644`:

   ```text
   RECEIPT_VERSION=1
   TASK_ID=<task>
   BASE_SHA=<sha completo>
   HEAD_SHA=<sha completo>
   TREE_SHA=<tree sha>
   DIFF_SHA256=<sha256 do diff, do passo 1>
   REVIEW_ARTIFACT=<path do artefato de review>
   REVIEW_SHA256=<sha256 do artefato>
   OPERATOR=<sua identidade>
   GATE_SCRIPT_SHA256=<sha256 da cópia do gate do passo 0>
   ```

   Atalho recomendado: `$G receipt-template <task> --repo <worktree> --base <BASE> --head <HEAD>
   --review-file <artefato>` imprime o receipt já com os valores recomputados (formato
   `RECEIPT_VERSION=2`, superset aceito pelo `post`; ver seção H5) e `<placeholders>` para o que só
   o operador atesta. Confira e preencha; placeholder esquecido → `receipt-malformed`.

4. `ENGRAM_REVIEW_RECEIPT=/abs/receipt.env $G post <task> --repo <worktree> --base <BASE> --head
   <HEAD> --review-file <artefato> --expect-tree <TREE_SHA> --expect-diff-sha256 <DIFF_SHA256>
   --expect-gate-sha256 <hash do passo 0> --operator <identidade>` com os valores **que você
   conferiu**, não copiados do writer. Qualquer mudança posterior de um byte (commit novo, artefato editado) invalida o receipt:
   emita outro.

Limites honestos: o gate roda `git` contra um repositório que o writer pode editar (por exemplo
`.git/config` com comandos de filtro/fsmonitor); replace refs, grafts e atributos são neutralizados,
mas a execução de config arbitrária do writer só é impedida pelo isolamento de H4/H5. Arquivos
ignorados (`.gitignore`, `info/exclude`) ficam fora do escopo de preparação. O receipt é fronteira
procedimental. Isolamento real do writer (mesmo usuário do SO)
chega com H4/H5; por ora a decisão humana continua sendo a autoridade e não é substituída por este
script nem por pareceres de IA. O `doctor.sh` apenas verifica presença do marcador (legado/histórico).

### Merge-policy read-only (H5): `merge-gate.py` e `agent-evidence.yml`

`docs/harness/bin/merge-gate.py` (+ `merge_gate_ci.py`) responde só se **este head exato** está
`eligible` para a decisão humana de merge; nunca aprova, faz merge, deploy, push nem publica. Saída:
um JSON `{decision: eligible|refused, reasons[], sections{}, bound{task, base, head, tree, policy,
merge_policy, ci_policy_sha256, evaluator_sha256}}`; exit 0 eligible, 1 refused, 2 uso. `eligible`
exige **todas** as seções em `pass` (seção não avaliada = refused): refs, evidence, gate_history,
review_receipt, review_scope, review, reviewer, lineage, ci, integrated, head_recheck. A versão de
política esperada precisa existir em `MERGE_POLICIES` (hoje `harness-hardening-v1` →
`merge-policy-v1`), coincidir com o registry confiável, e o fingerprint das constantes de CI
(`ci_policy_sha256`) precisa ser o fixado ali; mudar as constantes sem refixar → `policy_mismatch`.

- **Base confiável, head não confiável.** Avaliador, registry/catálogo, schema `review-v2` e a
  política do Q5 vêm do checkout onde o script mora (base/cópia do operador); o candidato entra por
  `--repo`. Head e base (e `--integrated-ref`) são re-resolvidos do git, nunca de payload, e
  re-checados logo antes de emitir; ref movido durante a avaliação → refused. Com SHA cru (o CI) o
  re-check é vazio: o humano compara o head vivo do PR com `bound.head` no momento do merge.
- **Evidência H4** só via `record-evidence.py verify()` para o head atual; base da evidência ≠ base
  atual → `stale_base`; outro gate do mesmo task/candidato com veredito ≠ pass → refused.
- **Pacote do reviewer (fresh context).** O reviewer readonly recebe só: a task (id + brief), o diff
  `base..head` e a lista de arquivos de `review-gate.sh scope`, os arquivos do head e a evidência H4
  (receipt + `evidence.json` + logs). **Nunca** o transcript, prompts ou rascunhos do writer. Produz
  um `review-v2` JSON (`head_sha`/`base_sha`/`policy_version` do candidato, findings, verdict,
  `reviewer` = seu id na allowlist).
- **Receipt do operador (produtor + consumidores).** `review-gate.sh receipt-template <task> --repo
  <worktree> --base <BASE> --head <HEAD> --review-file <review.json>` (cópia confiável do gate, fora
  de worktrees) imprime um receipt `RECEIPT_VERSION=2` com task/base/head/tree/diff/gate sha256 e
  path+sha256 do review recomputados, e `<placeholders>` em `OPERATOR`, `REVIEWER`,
  `POLICY_VERSION` e `REVIEW_CONTEXT`. O operador confere, preenche, grava fora de worktrees (dir
  `0700`, arquivo `0600`). Placeholder não preenchido é `receipt-malformed` nos dois consumidores.
  `REVIEW_CONTEXT=fresh-readonly` é a atestação deliberada do operador de que o pacote acima foi
  respeitado (por isso nunca vem preenchido). v2 é superset de v1: `review-gate.sh post` aceita v1
  e v2; `merge-gate.py` exige v2 (v1 → `reviewer_unavailable`, histórico). Um receipt vincula **um**
  artefato: o fluxo H1 (`post`, review com marcador `REVIEW_VERDICT`) e o fluxo H5 (review-v2 JSON)
  precisam de artefatos de review e receipts **separados**; o review com marcador nunca satisfaz
  H5. O diff sha256 é recomputado pela cópia confiável (`--review-gate`, `--expect-gate-sha256`);
  o artefato `review-v2` é validado por H2; prosa, marcador `REVIEW_VERDICT`, `review-v1`, PASS
  com finding bloqueante ou veredito ≠ `pass` → refused.
- **Identidade e lineage** só da allowlist do operador (`--identities`, `merge-gate-identities-v1`:
  `operators`, `reviewers{id: {lineage}}`, `writers{adapter: {lineage}}`). O campo livre `reviewer`
  precisa coincidir com o `REVIEWER` do receipt, mas não autentica; lineage do writer vem do adapter
  registrado pelo runner; lineage ausente → unavailable, igual → refused.
- **CI (proveniência).** O operador salva, com credencial read-only (nunca a do writer):
  `gh api --paginate --slurp "repos/<o>/<r>/commits/<head>/check-runs?per_page=100" > ci.json` e
  `gh api --paginate --slurp "repos/<o>/<r>/actions/runs?head_sha=<head>&per_page=100" > runs.json`
  (`--ci-results`, `--workflow-runs`). Regras: (1) PR que toca o que define, configura ou implementa
  um job required → `ci_policy_paths_changed` (o próprio PR poderia fabricar seu verde; julgamento
  humano sem `eligible`): `.github/**` inteiro (workflows, actions, CodeQL config, CODEOWNERS),
  `.cargo/**`, `.config/**`, `scripts/**`, `docs/harness/{bin,tests,checks,schemas}/**`,
  `docs/security/**` (exceções), `docs/quality/**`, `benches/results/**`,
  `tests/fixtures/retrieval_quality/**`, `tests/fixtures/security_gate_matrix.json` e configs na
  raiz (`deny.toml`, `.gitleaks.toml`, `.gitleaksignore`, `.semgrepignore`, `rust-toolchain*`,
  `rustfmt.toml`, `clippy.toml`, `Makefile`, `justfile`); além disso, todo arquivo citado nos steps
  do fecho de jobs required (nomes required + `needs:`) do `ci.yml` **da base** é protegido
  dinamicamente, e um teste falha se um arquivo citado pelo `ci.yml` atual escapar da lista
  estática. O blob de `.github/workflows/ci.yml` do head tem de ser o da base. (2) Só check runs
  do app `github-actions` para o head contam. (3) Cada context required (`Format`, `Clippy`,
  `Test (ubuntu-latest)`, `Documentation`, `Security Audit`, `Cargo Deny`, `Security Gate`) resolve para **exatamente um** check suite, e esse suite pertence a
  um workflow run com `path` `.github/workflows/ci.yml` (`ci_workflow_unbound` caso contrário);
  outro suite com o mesmo nome → `ci_context_multiple_suites` (sem lavar falha entre suites).
  Reabrir o PR ou `workflow_dispatch` que crie um segundo suite de `ci.yml` para o mesmo head deixa
  esse head **permanentemente** `ci_context_multiple_suites`; a recuperação é um commit novo (novo
  head, nova evidência, review e receipt), nunca apagar runs do arquivo salvo.
  (4) Dentro do suite, re-run vale (o mais recente vence; re-run em andamento bloqueia) e o
  resultado passa pelo `verdict()` do Q5 sem skips permitidos.
- **Arquivos do operador** (receipt, allowlist, check runs, workflow runs, cópia do gate): abertos
  uma vez com `O_NOFOLLOW` e validados no descritor (regular, um link, nossos, sem escrita de
  grupo/outros, sem ACL de escrita), diretório pai nosso e sem escrita de grupo/outros, ancestrais
  nossos ou de root e
  sem escrita alheia salvo sticky (`/tmp`), fora de qualquer worktree/`.git` (identidade de
  filesystem). A cópia do gate é executada a partir dos bytes verificados.
- **Merge queue / merge sintético**: a evidência vale para a árvore do head. `--integrated-ref`
  avalia o commit integrado em separado: árvore igual à do head → ok; diferente → refused
  (`integrated_tree_unverified`: precisa de evidência própria, que H4 ainda não produz para merges).
  Sem `--integrated-ref`, `stale_base` garante que o merge sobre a base atual reproduz a árvore.
- **Workflow `agent-evidence.yml`** (`pull_request`, nunca `pull_request_target`; `contents: read`;
  sem secrets; checkouts sem credencial persistida; expressões `${{ }}` só via `env:`, nunca dentro
  de `run:`): roda testes e avaliador do checkout da **base** (`trusted/`) contra o head
  (`candidate/`, nunca executado). Sob `pull_request` o arquivo do workflow é a cópia do PR, então o
  resultado do job não é confiável nem sinal de merge; sem inputs do operador a decisão é `refused`
  por desenho e o job só verifica que ela é bem formada. Rollback: remover workflow/avaliador; gates
  aceitos e evidência antiga ficam.

## Camada 3 — Checklist Humano em PR / Commit

Itens que os gates automatizados não cobrem completamente:

- [ ] Evidência de que testes rodaram contra features/config reais de CI (não só local-embeddings).
- [ ] Se MCP surface mudou: SDKs Python/TS atualizados ou pelo menos breaking change documentado + issue aberta.
- [ ] Se storage migration ou `SCHEMA_VERSION`: evidência de que testes de migração e integração rodam limpos.
- [ ] Se mudança em hooks/intelligence/consolidation: dry-run ou evidência de que side effects foram considerados.
- [ ] Se embeddings ou cache: impacto em tamanho de binário, benchmarks ou qualidade de retrieval foi medido (quando relevante).
- [ ] `progress.md` + log da sprint atualizados com decisões e evidência de gates.
- [ ] Para mudanças de processo do harness: `doctor.sh` passou antes e depois.
- [ ] Preview/Deploy (quando aplicável): fly.io ou docker build verificado.

## Skip Allowlist (Review-Gate)

Pode pular o review-gate (camada 2) **somente** quando o diff inteiro for:

1. **Docs-only** em `docs/**/*.md`, exceto qualquer arquivo dentro de `docs/harness/` (INVARIANTS, GATES, CODE_REVIEW_POLICY, README, SPEC, scripts em bin/).
2. **Comment-only** ou doc comments (///, //!, //! ) sem mudança de comportamento.
3. **Formatting-only** (cargo fmt) sem outra alteração.
4. **Test-only additions** que não alteram produção (cobertura de path existente, sem mudança de contrato).

O `review-gate.sh post` implementa mecanicamente somente o item 1 (ver "Review gate fail-closed"); os
itens 2-4 dependem de julgamento humano registrado e não são dispensados automaticamente.

**Sensores (camada 1) NUNCA são pulados.**

**Nunca pular** (mesmo em diffs "pequenos"):

- Qualquer `.rs` em `src/` (especialmente mcp/handlers, storage/, hooks/, intelligence/).
- `Cargo.toml`, `Cargo.lock`, `build.rs`, `deny.toml`.
- `scripts/ci.sh`, `justfile`, `Makefile`, `.githooks/`.
- `docs/harness/**` (o próprio harness controla os gates).
- Mudanças em `sdks/python/` ou `sdks/typescript/` que afetam contrato.
- Qualquer coisa que toque MCP protocol, snapshot, attestation, ou auth.

Em dúvida: rode o review-gate.

## Integração com Paridade CI

O modo `full` do sensor principal `sensors.sh` mantém paridade funcional com o contrato do projeto por meio de etapas equivalentes a `CI_FEATURES`/`just ci`:

- `fmt`
- `clippy -D warnings`
- testes de biblioteca (`--lib --tests`)
- testes de integração (`--bin engram-server`, `--bin engram-watcher`)
- `cargo doc` com `RUSTDOCFLAGS="-D warnings"`
- `./scripts/generate-mcp-reference.sh --check`

Essas etapas são registradas granularmente em `.sensors-log` (`ci_steps`) para triagem e análise.

O harness também adiciona:

- Harness doctor como etapa explícita.
- Review cross-CLI.
- Memória persistida (progress + reviews).
- Fake-success hunting específico de engram.

Isso mantém o "um comando para rodar tudo localmente" enquanto adiciona as camadas de harness.

---

**Princípio**: Evidence before claims. O harness existe para tornar "funcionou no meu prompt" verificável e retomável por outros agentes.
