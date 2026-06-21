# Engram Harness — Operational Discipline

Disciplina operacional do repositório `aiconnai/engram`.

O harness existe para manter o trabalho retomável, auditável e de alta qualidade entre sessões de agentes (Claude Code CLI, Grok Build TUI, Codex, Cursor, etc.) e entre humanos + agentes. Ele fornece spec feed-forward, invariants duros, gates mensuráveis (locais + cross-model review), reviews persistidos como artefatos, e progresso registrado de forma canônica.

Se um agente futuro (ou você em outra sessão) não consegue responder rapidamente:

- Qual é a tarefa ativa?
- Quais regras não mudam nunca?
- Quais checks bloqueiam o progresso?
- O que mudou desde a última sessão e onde isso foi registrado?

...então o harness está incompleto.

## Filosofia

> **The terminal is the product.**

Este harness foi desenhado para agentes que *vivem* no terminal (Grok Build TUI, Claude Code CLI, etc.), não para UIs bonitas com chat embutido. O CLI é a interface de mais alto leverage: já instalada, profundamente treinada nos modelos, self-documenting, infinitamente composable, baixa latência, zero chrome.

O harness não é "outra interface de chat". É o conjunto de camadas (Context Engine, Planner, Memory Manager, Verifier, Tool Registry, Harness Config) que vive *dentro* do repo, onde o trabalho real acontece.

Engram é especialmente adequado para isso porque ele *é* um Memory Manager para agentes e para times que acumulam contexto proprietário rápido demais para depender de memória humana ou de chat logs soltos. O harness de engram pode (e deve) eventualmente dogfood o próprio engram para armazenar sessões, decisões, reviews e eventos de verificação.

## Contrato de Bootstrap (Obrigatório)

Todo agente / sessão começa por:

```bash
bash docs/harness/bin/bootstrap.sh
```

O bootstrap é:

- Rápido (< 500ms)
- Read-only (sem side effects)
- Determinístico
- Limite: ~50 linhas de output

Ele imprime:

- Branch + dirty/clean state
- Último commit
- Sprint/tarefa ativa (de `progress.md`)
- Último verdict de review cross-model
- Último resultado de sensores
- Ordem de leitura obrigatória

Se o bootstrap indicar arquivos ausentes, drift, review sem verdict ou output excessivo, rode:

```bash
bash docs/harness/bin/doctor.sh
```

`doctor.sh` é read-only e valida a consistência interna do harness (drift entre SPEC/progress, scripts executáveis, referências à policy local, etc.). Ele **bloqueia** mudanças no harness quando há inconsistência.

Quando automação precisar de output parseável, use o contrato em
[`JSON_OUTPUTS.md`](./JSON_OUTPUTS.md). O modo humano continua sendo o default;
flags JSON devem ser opt-in e nunca devem emitir segredos, tokens, headers,
cookies ou dumps de ambiente.

## Estrutura

| Caminho                        | Papel |
|--------------------------------|-------|
| `README.md`                    | Guia operacional e fluxo de trabalho |
| `SPEC.md`                      | Escopo curto da sprint/tarefa ativa (mutável por tarefa) |
| `INVARIANTS.md`                | Regras de processo invioláveis (canônico sobre docs divergentes) |
| `WHAT_WE_DONT_DO.md`           | Escopo negativo e anti-patterns para evitar expansão silenciosa |
| `GATES.md`                     | Sensores, thresholds, retry policy, exclusões documentadas, fake-success patterns |
| `CODE_REVIEW_POLICY.md`        | Política local consumida pelo review-gate (cross-model / cross-CLI) |
| `JSON_OUTPUTS.md`              | Contrato de output JSON para automação do harness |
| `SKILLS.md`                    | Inventário e política de promoção de skills repo-locais (`skills/<name>/SKILL.md`) |
| `security/`                    | Contrato de segurança do harness, incluindo a adaptação Anthropic |
| `.claude/scan-extras.txt`      | Tuning versionado para categorias extras de scan/triage |
| `.claude/fp-rules.txt`         | Tuning versionado para exclusões conservadoras de falso positivo |
| `canvas/`                      | Evidência estruturada para mudanças complexas |
| `audits/`                      | Relatórios evidence-only de auditoria periódica |
| `progress.md`                  | Estado vivo curto: sprint, task, último review, último sensor, commit |
| `progress/*.md`                | Logs permanentes por sprint/tarefa (detalhados) |
| `known-issues/*.md`            | Incidentes externos que justificam exclusão auditável de sensor |
| `reviews/*.md`                 | Artefatos permanentes de pre/post review (versionados por iteração) |
| `bin/bootstrap.sh`             | Orientação rápida da sessão (obrigatório no início) |
| `bin/doctor.sh`                | Consistência read-only do harness |
| `bin/sensors.sh`               | Gate determinístico principal (wrapping `just ci` ou `make ci` + harness checks) |
| `bin/review-gate.sh`           | Gate de review cross-CLI / cross-model (generalizado) |
| `bin/baseline.sh`              | Snapshot estático barato em `.baseline-last` para drift review |
| `bin/quarterly-audit.sh`       | Auditoria evidence-only; nunca apaga, arquiva ou reescreve |
| `bin/vc-gate.sh`               | Gate opcional de version control para issue boundaries, `jj` local e releases Git/Cargo |
| `bin/check-commit-msg.sh`      | Validador de Conventional Commit com scope |
| `bin/pr-title-policy.sh`       | Política canônica de títulos de PR, incluindo bloqueio do marcador `[codex]` |
| `bin/check-pr-title.sh`        | Wrapper compatível para validar títulos de PR via política canônica |

## Leitura Obrigatória (em ordem)

Antes de editar código, docs de processo, ou planejar qualquer mudança significativa, leia nesta ordem:

1. `docs/harness/SPEC.md` — escopo da sprint/tarefa ativa
2. `docs/harness/INVARIANTS.md` — regras duras de processo (vence conflitos)
3. `docs/harness/WHAT_WE_DONT_DO.md` — escopo negativo e anti-patterns
4. `docs/harness/GATES.md` — critérios de sensores e review
5. `docs/harness/CODE_REVIEW_POLICY.md` — política de julgamento para o reviewer externo
6. `docs/harness/security/anthropic-reference-harness.md` — boundary static/read-only vs execucao autonoma
7. `docs/harness/progress.md` — estado vivo
8. O active plan apontado em `Active plan`
9. `AGENTS.md` (raiz) e `Claude.md` / docs de onboarding relevantes
10. `INVARIANTS.md` (raiz) — data invariants do sistema de memória
11. `STANDARDS.md` + `ERRORS_AND_LESSONS.md`

`INVARIANTS.md` (harness) vence qualquer conflito com AGENTS.md, Claude.md, specs antigas ou memória de sessão. Mudanças reais em invariants exigem ADR em `docs/decisions/` + PR revisado sob os gates anteriores.

## Loop por Tarefa (Obrigatório para Mudanças que Alteram Estado)

```bash
# 1. Orientar a sessão (obrigatório)
bash docs/harness/bin/bootstrap.sh

# 2. Review prévio (advisory, mas findings são input obrigatório)
bash docs/harness/bin/review-gate.sh pre <task-id>

# 3. Gate de version control no inicio da issue (recomendado)
bash docs/harness/bin/vc-gate.sh start <task-id>

# 4. Implementar a menor mudança correta
#    Rust: TDD onde aplicável, clippy limpo, cobertura de comportamentos alterados.
#    Use `just ci` localmente para paridade com GitHub (ou `make ci` onde `just` não estiver disponível).

# 5. Rodar sensores determinísticos (hard gate)
bash docs/harness/bin/sensors.sh

# 6. Review pós-mudança (hard gate — PASS exigido)
bash docs/harness/bin/review-gate.sh post <task-id>

# 7. Atualizar memória canônica (obrigatório)
$EDITOR docs/harness/progress.md
$EDITOR docs/harness/progress/<active-plan-filename>.md

# 8. Validar mensagem de commit
bash docs/harness/bin/check-commit-msg.sh --message "<type>(<scope>): <description>"

# 9. Validar título de PR antes de criar ou editar PRs por automação
bash docs/harness/bin/pr-title-policy.sh --title "<concise PR title>"
# compatibilidade com automações antigas:
bash docs/harness/bin/check-pr-title.sh --title "<concise PR title>"

# 10. Commitar arquivos específicos (nunca "git add .")
git add <arquivos específicos>
git commit -m "<type>(<scope>): <description>"

# 11. Gate de version control no fechamento da issue (recomendado)
bash docs/harness/bin/vc-gate.sh done <task-id>
```

Não pule a atualização de progresso. `progress.md` + o log da sprint são a **memória canônica** do repositório para agentes futuros.

## Task IDs, Scopes e Commits

Use task IDs curtos, estáveis e rastreáveis (ex.: `harness-bootstrap`, `mcp-harness-memory-v1`, `rfc-0001-impl`).

Commits devem ser machine-parseable e task-scoped:

```bash
bash docs/harness/bin/check-commit-msg.sh --message "docs(harness): add bootstrap and doctor scripts"
```

Formato aceito (alinhado com Conventional Commits + escopos engram):

```
type(scope): concise subject
```

Tipos comuns: `feat`, `fix`, `docs`, `refactor`, `test`, `perf`, `ci`, `chore`, `revert`.

Scopes recomendados: `harness`, `mcp`, `storage`, `search`, `intelligence`, `hooks`, `sdk-python`, `sdk-ts`, `cli`, `server`, `ci`, ou identificadores de issue/RFC (`engra-22`, `rfc-0001`).

Sem `Co-Authored-By` ou trailers de atribuição de IA.

PR titles também não carregam o marcador `[codex]`. Antes de `gh pr create` ou
`gh pr edit`, rode:

```bash
bash docs/harness/bin/pr-title-policy.sh --title "concise PR title"
# compatibilidade com automações antigas:
bash docs/harness/bin/check-pr-title.sh --title "concise PR title"
```

## Version Control Gate e jj

`vc-gate.sh` existe para impedir que trabalho de varias issues acumule em um
worktree sujo sem fronteira clara. Ele complementa Git; nao substitui GitHub,
tags de release ou `cargo publish`.

Uso recomendado:

```bash
# Antes de iniciar uma issue
bash docs/harness/bin/vc-gate.sh start ENGRA-84

# Durante trabalho local com jj, se adotado pelo time
jj new -m "feat(storage): ENGRA-84 memory policy layer"
jj split
jj describe -m "feat(storage): ENGRA-84 add memory policy layer"

# Antes de marcar done
bash docs/harness/bin/vc-gate.sh done ENGRA-84

# Antes de release/publish
bash docs/harness/bin/vc-gate.sh release 0.21.1
```

Regras:

- `jj` pode ser usado para evoluir, splitar e descrever work-in-progress local.
- Git continua canônico para release commits, tags e crates.io.
- O gate nao cria commits, nao roda `jj new`, nao move tags e nao publica.
- Dirty worktree em `start`, `done` ou `release` precisa ser resolvido ou
  explicitamente atribuido com uma flag de allow.

## Sensores (Camada Determinística)

`sensors.sh` é o gate local principal. Ele invoca:

- `just ci` (preferencial) ou `make ci` (fallback): fmt + clippy -D warnings + testes com paridade Linux + docs + MCP reference
- Verificação de harness doctor
- Outros checks específicos de engram (ex.: snapshot tests, property tests, embedding cache bounds, etc.)

Resultado mais recente fica em `docs/harness/.sensors-last`.

Exclusões documentadas (apenas para dependências externas temporárias, ex.: APIs de embedding pagos indisponíveis) seguem contrato rigoroso via `--exclude-sensor`, `--known-issue` e `--reason`, com registro prévio em progress e known-issues/.

### Sensor modes

`bash docs/harness/bin/sensors.sh` sem argumentos continua sendo o gate completo canônico.

Modos opcionais existem apenas como atalhos de desenvolvimento:

- `full` — equivalente ao default: CI local completo + `doctor.sh`.
- `quick` — `cargo fmt --all -- --check`, `cargo check` e `doctor.sh`.
- `docs` — referência MCP gerada, rustdoc com warnings como erro e `doctor.sh`.
- `mcp` — referência MCP gerada, testes de protocolo MCP e `doctor.sh`.
- `baseline` — `baseline.sh` + `doctor.sh`.
- `status --json` — snapshot read-only de `docs/harness/.sensors-last` no
  envelope de `JSON_OUTPUTS.md`; nao roda o gate completo.

Essas lanes opcionais não substituem o gate completo para merge, handoff ou claim de conclusão.

`baseline.sh` grava fatos estáticos baratos em `docs/harness/.baseline-last`. Ele ajuda revisão de drift, mas não prova correção.

`quarterly-audit.sh` grava relatórios evidence-only em `docs/harness/audits/` e atualiza `docs/harness/.quarterly-audit-last`. Ele nunca é um gate pass/fail e não deve apagar, arquivar ou reescrever arquivos.

## Review Gate (Camada Cross-Model / Cross-CLI)

`review-gate.sh` implementa o princípio de **Single-Process Judgment**:

- O agente que *escreve* o código não deve ser o juiz final da sua própria saída.
- Usa outro CLI/modelo (Claude Code, Grok Build, Codex, local via Ollama, etc.) como reviewer independente.

Modos:

- `pre <task-id>` — advisory (sempre sai 0). Findings viram input obrigatório antes de codar.
- `post <task-id>` — hard gate. `FAIL` bloqueia commit/PR.
- `post <task-id> --range=main..HEAD` — para fechamento de sprint/PR.

O script:

- Monta prompt rico com: SPEC, INVARIANTS, WHAT_WE_DONT_DO, GATES, CODE_REVIEW_POLICY, fake-success patterns específicos de Rust/engram/MCP, diff (excluindo artefatos do próprio harness para evitar loops).
- Suporta múltiplos backends de reviewer via env `REVIEWER_CLI` ou flags.
- Escreve artefatos versionados (v1-post.md, v2-post.md...) em `reviews/`.
- Suporta continuity: em reruns de post após FAIL, injeta achados anteriores relevantes.
- Tem modo self-test.

Mudanças em `docs/harness/bin/*` são process-critical. O post-gate exige evidência independente explícita antes de aceitar alterações nesses scripts.

## Review Canvas

Mudanças complexas devem criar um canvas em `docs/harness/canvas/YYYY-MM-DD-<task-id>.md` antes do post-review.

Use `docs/harness/canvas/TEMPLATE.md` quando a mudança envolver diffs grandes, storage/migrations, superfície MCP, hooks/intelligence/embeddings/sync/attestation, contratos SDK, dependências externas ou qualquer alteração em gates/scripts/policies do harness.

O canvas é evidência de raciocínio, não aprovação. O post-review ainda pode falhar depois de um canvas completo.

Política de retry: 2 FAILs consecutivos no mesmo task → escalar para humano. Não tentar 3ª iteração.

## Security Reference Harness

O fluxo de segurança inspirado no `anthropics/defending-code-reference-harness`
vive em `docs/harness/security/anthropic-reference-harness.md`. Esse arquivo
e a fonte canonica local para o contrato `ENGRAM-HARNESS-SECURITY-CONTRACT-v1`;
`doctor.sh` falha se ele ou os arquivos de tuning versionado estiverem ausentes.

Contrato local:

- Static/read-only first: threat model, scan, triage e patch candidates antes de
  qualquer execução autônoma.
- Tuning versionado para Claude Code/reference harness em
  `.claude/scan-extras.txt` e `.claude/fp-rules.txt`.
- Pipeline autônoma bloqueada por default para Engram; qualquer porta Rust exige
  ADR, sandbox forte, target contract e review-gate.
- Findings importados viram artefatos auditáveis e só avançam com evidência,
  supressão explícita, ou patch revisado.
- Nenhum script do harness pode silently fall back para postura mais fraca quando
  esse contrato ou os arquivos `.claude/scan-extras.txt` e
  `.claude/fp-rules.txt` estiverem ausentes.

## Critério de Done

Uma tarefa só está pronta quando:

- [ ] Bootstrap lido + SPEC/INVARIANTS/GATES/POLICY lidos
- [ ] Pre-gate rodado (ou skip registrado)
- [ ] Menor mudança correta implementada + TDD/verificações apropriadas
- [ ] `just ci` / `sensors.sh` passou limpo (ou exclusão documentada válida)
- [ ] Post-gate retornou `PASS ...`
- [ ] `progress.md` + log da sprint atualizados
- [ ] Review artifacts relevantes preservados
- [ ] Mensagem de commit validada e arquivos específicos commitados
- [ ] `vc-gate.sh done <task-id>` passou ou skip foi registrado
- [ ] Evidência adicional registrada para mudanças em MCP surface, storage schema, hooks, embeddings, sync, ou breaking changes em SDKs

## Situações que Exigem Cuidado Extra

- **Schema / Migrations** (`storage/migrations.rs`): Atualizar `SCHEMA_VERSION`. Testes têm versão hardcoded em alguns lugares — atualizar também. Nunca quebrar monotonicidade de IDs.
- **MCP Tools**: Mudança na superfície de 155+ tools exige atualização de `scripts/generate-mcp-reference.sh` output, docs/MCP_TOOLS.md, e testes de protocolo. Coordenação com SDKs Python/TS.
- **Hooks / Intelligence / Consolidation**: Comportamento não-determinístico ou com side effects em produção exige dry-run + evidência.
- **Embeddings / ONNX / Cache**: Mudanças aqui afetam benchmarks, tamanho de binário, e qualidade de retrieval. Exigem baseline comparison quando relevante.
- **Harness em si**: Mudança em `docs/harness/bin/`, INVARIANTS, GATES ou CODE_REVIEW_POLICY exige rodar doctor + sensors + post-gate.
- **Cross-SDK**: Mudança em Python ou TypeScript SDKs que quebra contrato com o core Rust exige testes de integração e possivelmente versão de crate.

## Integração com Agentes Atuais (Claude Code + Grok Build)

O harness é agnóstico a CLI. O bootstrap funciona em qualquer um.

Para review cross-CLI (o cenário atual do usuário):

- Rode `review-gate.sh post ...` — ele prepara o prompt completo.
- Cole o prompt no outro CLI (o "reviewer").
- Salve a resposta completa em `reviews/YYYY-MM-DD-<task>-vN-post.md`.
- O parser extrai o `PASS`/`FAIL` do artefato.

Futuramente, podemos adicionar suporte direto a exec não-interativo quando as CLIs suportarem (ex.: `grok build --exec "prompt..."` ou equivalente).

## Dogfooding com o Próprio Engram

Porque engram *é* o Memory Manager ideal para harnesses:

- Reviews, decisões, sessões de agente, eventos de verificação podem ser armazenados via MCP tools (`memory_create`, `memory_create_section`, identities para agentes, etc.).
- Hooks (`session_end`, etc.) podem registrar resumos de harness events.
- O RFC `docs/rfcs/0001-harness-memory-product-boundary.md` define o boundary exato do que Harness Memory deve e não deve capturar.

Trabalho futuro (ENGRA-22+): ingestão automática de eventos de harness (commits, reviews, gate results) para o próprio engram, com provenance forte e exclusão de segredos/raw logs.

## Referências

- RFC 0001: Harness Memory Product Boundary (`docs/rfcs/0001-harness-memory-product-boundary.md`)
- Root `INVARIANTS.md` (data invariants do sistema)
- `STANDARDS.md`, `ERRORS_AND_LESSONS.md`, `AGENTS.md`, `Claude.md`
- `just ci` / `scripts/ci.sh` — gates obrigatórios de CI
- `docs/MCP_TOOLS.md` — superfície completa de tools expostas via MCP

---

**Cutoff de obrigatoriedade**: O harness entra em vigor a partir da adoção inicial deste conjunto de docs e scripts. PRs anteriores não são retroativamente afetados, mas todo trabalho subsequente segue este README, GATES e INVARIANTS do harness.
