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
Semgrep, Gitleaks, and AgentShield. Every constituent is release-blocking on a
pull request: a failed or unexpectedly skipped constituent makes the aggregate
red. Event-policy skips are neutral only when explicitly listed in
`tests/fixtures/security_gate_matrix.json`; the current pull-request policy has
no such skip.

Branch protection itself is not changed by repository automation. Instead, the
already-required `Test (ubuntu-latest)` job depends on `security-gate`, so a
security failure blocks that required context transitively. Verify the live,
read-only chain with:

```bash
gh api repos/aiconnai/engram/branches/main/protection/required_status_checks \
  > .omo/evidence/task-18-required-contexts.json
python3 scripts/check-security-gate.py \
  --matrix tests/fixtures/security_gate_matrix.json \
  --required-contexts .omo/evidence/task-18-required-contexts.json \
  --workflow .github/workflows/ci.yml
```

The checker also exposes `--self-test-failure` and `--self-test-unrequired` to
prove that a constituent failure or removal of the required-context dependency
fails closed.

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

### Required checks no GitHub (merge em `main`)

Os sensores locais confirmam o trabalho cedo; o GitHub re-confirma os mesmos
contratos como **required status checks** antes do merge. Bloqueiam o merge:

- `Format`, `Clippy`, `Test (ubuntu-latest)`, `Documentation` (os quatro jobs de
  CI baratos e determinísticos);
- `Harness Contract` — gate leve (`bootstrap.sh` + política de título de PR). A
  política só rejeita o marcador literal `[codex]`; não é validação ampla de
  título. O `doctor.sh` **não** entra neste gate required (fica local/advisory).

`Security Audit` e `Cargo Deny` rodam nos PRs como **advisory** e não bloqueiam
merge enquanto o baseline tiver advisories abertas (atualmente
`RUSTSEC-2026-0187` em `lopdf` e `RUSTSEC-2026-0185` em `quinn-proto`). Promover
qualquer um a required exige antes resolver/ignorar essas advisories com
rationale datado. Code review automático é sinal extra, nunca o único bloqueador.

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

- Pre: advisory (sempre 0), mas findings são obrigatórios de ler.
- Post: hard gate. `PASS <resumo>` na primeira linha ou FAIL, **e** incluir sempre a linha `REVIEW_VERDICT: PASS|FAIL ...` para parser hard.
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
