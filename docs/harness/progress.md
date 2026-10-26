# Engram — Harness Progress (Live State)

| Field | Value |
|-------|-------|
| Project | `engram` |
| Active sprint | `Remediação pós-levantamento 2026-10-26` |
| Active task | `remediation-2026-10-26` |
| Active plan | `docs/harness/progress/2026-10-26-remediation.md` |
| Last review | `2026-10-23 — parecer Sonnet PASS, opinião independente e NÃO post-gate com receipt humano: docs/harness/reviews/2026-10-23-turso-sunset-v2-post.md. Sem review da sprint atual nem de d922a60e/e04d3307/c9de7136/b29cd16c. O veredito de 2026-10-14 não registra reviewer, intervalo nem receipt (nota de proveniência em reviews/).` |
| Last sensors | `2026-10-23 status=pass mode=full timestamp=2026-10-23T20:10:00Z duration_sec=310, sem exclusões (receipts commitados em 08cb17a4; rodado no candidato do sunset, run anterior falhou em offline_lane). Sem receipt commitado de 2026-09-03 a 2026-10-23; não reexecutado depois. Detalhe em progress/2026-10-26-remediation.md.` |
| Last commit | `7ff6cafb` |
| Last live-state check | `2026-10-26 — PASS estrito na árvore da frente WT5 (Last commit = HEAD 7ff6cafb antes do commit de refresh; depois dele é o pai): rtk bash docs/harness/bin/check-live-state.sh --progress docs/harness/progress.md. Vale só para esta branch; o integrador refaz após o merge. Não substitui post-gate.` |

> Resumo vivo, **≤150 linhas** (task H6). Orçamentos, medição e regras de retenção:
> [`context-budget.md`](./context-budget.md). O texto anterior deste arquivo foi movido
> **sem alteração** para [`progress-history.md`](./progress-history.md) (índice de 68 seções
> com âncoras estáveis; nada foi apagado). Logs por task em `progress/`.

## Retomada rápida (escopo, limites, última evidência)

- **Escopo**: sprint `Remediação pós-levantamento 2026-10-26`: corrigir o que o levantamento read-only de
  2026-10-26 achou (estado vivo defasado, regressão de autorização de `e04d3307`, advisories, release
  pipeline, claims públicos). [SPEC](./SPEC.md); [plano](./plans/2026-10-26-remediation-plan.md);
  log ativo [`progress/2026-10-26-remediation.md`](./progress/2026-10-26-remediation.md).
- **Autorização atual** (owner, chat, 2026-10-26: "resolva tudo o mais paralelo possível"): 6 branches
  **locais** paralelas a partir de `08cb17a4`; o integrador mescla numa branch de integração, roda o CI
  completo e só então finaliza os SHAs neste arquivo.
- **Limites**: push, PR, merge, tag, release, publicação e remoção de branches/worktrees **não** estão
  autorizados (aguardam decisão explícita do owner); sem dados reais, chamadas pagas, produção ou settings
  do GitHub. [Negativo do harness](./WHAT_WE_DONT_DO.md); [escopo canônico](./SPEC.md).
- **Última evidência**: [sensores](./progress/2026-10-26-remediation.md#evidência-de-sensores): full PASS de
  2026-10-23 (310 s), o único recibo commitado desde 2026-09-03; nenhum gate completo roda nesta sprint
  antes do integrador. Post-gate do sunset `PENDING` (sem receipt humano).
- **Como retomar**: `bash docs/harness/bin/bootstrap.sh` → leitura obrigatória na ordem de
  `AGENTS.md`/`CLAUDE.md` (inalterada) → este arquivo → active plan →
  `bash docs/harness/bin/doctor.sh`. Seção histórica inteira, sem cortar parágrafo:
  `python3 docs/harness/bin/measure-context.py --section docs/harness/progress-history.md#<âncora>`.

## Sprint atual — frentes (estado ao abrir; o integrador atualiza)

WT1 `fix/workspace-batch-auth`, WT2 `fix/supply-chain-2026-10`, WT3 `fix/release-pipeline`, WT4
`fix/mcp-surface-honesty`, WT6 `docs/product-status`: abertas, não verificadas por esta branch. WT5
`chore/harness-live-state`: esta branch. INT: integração e SHAs finais, pendente. Escopo, aceite e
checklist pós-merge no [plano](./plans/2026-10-26-remediation-plan.md#frentes).

## Programa anterior — comprehensive improvement, ondas 0-6 (concluído em 2026-10-14)

- **Status**: completed (ondas 0-6 encerradas em 2026-10-14). Plano:
  [`plans/2026-10-02-engram-comprehensive-improvement-plan.md`](./plans/2026-10-02-engram-comprehensive-improvement-plan.md).
  Log da lane R (Rust/produto):
  [`progress/2026-10-05-improvement-lane-r.md`](./progress/2026-10-05-improvement-lane-r.md)
  (integrado ao tree desta branch pelo merge das lanes, task INT).
- **Decisões do owner (2026-10-05, chat; históricas)**: ADR hardening v1 **aceito**; WAL 64 GiB
  configurável; `defer_embedding=true` enfileira em background. A autorização de execução daquele
  programa (commits locais, ondas 0-4) expirou com ele e **não** vale nesta sprint. Review Canvas do
  programa: **ausente**, desvio registrado na [auditoria §9](./audits/2026-10-02-improvement-baseline.md).
- **Baseline de execução (decisão do owner)**: `1952f3b`, 14 commits locais à frente de
  `origin/main` `949c963` (0 atrás); permanecem locais e sem revisão independente no repositório.
  O commit do plano `630dd26` é documental e fica sobre essa baseline.

Registro (revisões SDD independentes), ledger, briefs, reports e revisão final:
[`reviews/2026-10-05-comprehensive-improvement/`](./reviews/2026-10-05-comprehensive-improvement/README.md).
O veredito [`2026-10-14`](./reviews/2026-10-14-engram-comprehensive-improvement-post.md) é PASS **sem** reviewer,
intervalo nem receipt: ver a [nota de proveniência](./reviews/2026-10-14-engram-comprehensive-improvement-post-provenance.md).

| Task | Lane | Revisão → rodadas de correção | Commits |
|---|---|---|---|
| E0 estado/ADR/baseline | P | sonnet Approved → 0 | `630dd26..0c6ea2b` |
| Q1a contrato CI/local do quality budget | P | Approved c/ 1 Important → 1 | `0c6ea2b..cf6d969` |
| H1 review gate fail-closed | P | opus Needs fixes → 1 | `cf6d969..60343b3` |
| H2 schemas/evidência + lane offline | P | sonnet Approved → 0 | `60343b3..df9e9c8` |
| O3 avaliadores probabilísticos | P | haiku Approved c/ 1 Important → 1 | `df9e9c8..17c6497` |
| Q5 security gate e supply chain | P | sonnet Needs fixes → 1 | `f4f547c..62101f8` (sem H3) |
| H3 registry + sandbox (fake writer) | P | opus Needs fixes → 2 | `60cf3db..4d341a0` (6 commits) |
| H6 contexto curto e retenção | P | sonnet Needs fixes → 1 | `c7a8d71..96c82c5` |
| H4 runner + evidência externa | P | opus Needs fixes → 2 | `2bdf942..f96b200` |
| O2 higiene Git e retenção | P | sonnet Approved → 0 | `a40430d..19a6218` |
| H5 review SHA-bound + merge policy | P | opus Needs fixes (1 Critical) → 2 | `8950f47..3a373e7` |
| O4 standing checks | P | sonnet Approved → 0 | `a9809d1..c8581cc` |
| C2 WAL replay/recovery + concorrência | R | opus Needs fixes → 1 | `1398258..46fc3e2` |
| Q2 inventário de risco Rust | R | sonnet Approved → 0 | `a87a3ce` |
| C1 autorização por workspace | R | opus Needs fixes → 1 | `44d42f8`, `6750b9b` |
| C3 panics Unicode/hex | R | sonnet Approved → 0 | `a6f55ac` |
| Q4 contratos MCP/SDK | R | sonnet Needs fixes → 1 | `98bd050`, `0d36d3d` |
| G1 perda de WAL por lock (P0) | R | opus Needs fixes → 2 | `974b7a6`, `9712ade`, `bbe9203` |
| Q3 fuzz/Miri/mutants | R | sonnet Needs fixes → 1 | `4e8ba9a`, `730ede4` |
| C7 `defer_embedding` enfileira | R | opus Needs fixes → 2 | `f560a6c`, `081c73f`, `ec3093c` |
| C5 vocabulário no create | R | sonnet Approved → 0 | `c22e11e` |
| C6 falhas hooks/multimodal/sync | R | opus Needs fixes → 1 | `c601d69..b3fb284` (6 commits) |
| Q7 runner de qualidade candidato | R | sonnet Approved c/ 3 Important → 1 | `12d2c3b`, `8e7f76f`, `4d067f1` |
| P1 regressão de performance | R | opus Approved → 0 | `772e84a`, `0a46e9b` |
| Q2F backlog do Q2 | R | sonnet Needs fixes → 1 | `ab22911..98103b0` + merge `77741c3` |
| O1 observabilidade e redação | R | opus Needs fixes → 1 | `35a1592` (arquivos O1), `8cef307`, `96d41bb` |
| Q6 advisories de dependências | R | sonnet Approved → 0 | `6c4ab81..8cef307` (5 commits) |
| C4 ADR fd SQLite + spike | R | sonnet Needs fixes → 1 | `1c246c8`, `c099ca2` |
| INT integração das lanes, Q1b | — | coberta pela revisão final | `ee8c4ea..4868d49` |
| Revisão final do branch | — | opus Ready with fixes (0 Critical) | `1952f3b..4868d49` |
| FINALFIX correções da revisão final | — | sem re-revisão ao commitar | [entrada FINALFIX](./progress/2026-10-05-improvement-lane-p.md#finalfix--correções-da-revisão-final-do-branch-2026-10-05) |

- **Discrepâncias de E0** (detalhe na auditoria): D1, D7 e D9 com nota de fechamento de 2026-10-26 (D9 mantém o desvio de canvas/receipt), D2 (`rustls` 0.23.46), D3 (Q1a), D4 (Q5/H6), D5 (H2/H3), D6, D10 (`context-budget.md`), D11, D12 resolvidas. Em aberto: D8 (branch protection GitHub sem review humano obrigatório).
- **#152 (confirmada pelo owner Ronaldo em 2026-10-14)**: limites de chunk na ingestão de documentos são em **caracteres**, distintos de tokens; decisão selada em [`../decisions/2026-10-14-default-features-loopback-and-chunking-signoff.md`](../decisions/2026-10-14-default-features-loopback-and-chunking-signoff.md) e [`context-budget.md`](./context-budget.md#6-decisão-152--chunks-por-caracteres-não-por-tokens).

## Próximos passos (owner)

- Decisões pendentes: tabela no [plano](./plans/2026-10-26-remediation-plan.md#decisões-pendentes-do-owner). Nada foi publicado, tagueado nem removido.
- `v0.24.0` Core + `v0.7.0` SDKs seguem **preparados** (`CHANGELOG.md`, `channel-matrix.toml`) sem tag nem release; `O2` untrack executado para os 73 `.raw` (histórico Git ainda os contém).
- Resumo pós-merge e decisões de 2026-10-14: [registro de revisão](./reviews/2026-10-05-comprehensive-improvement/README.md#owner-decisions--post-merge-waves-56-resolution-2026-10-14), [sumário Waves 5–6](./reviews/2026-10-05-comprehensive-improvement/wave5-6-summary.md).

## Enforcement do live state

- `doctor.sh` roda `check-live-state.sh --structural` neste arquivo (task H6): campos
  obrigatórios, Active plan existente, review PASS autoritativo (derivado do campo `Last review`: precisa ser a versão mais nova da própria tarefa e, se a tarefa ativa já tem review, a dela; canvas `2026-10-26-harness-script-fixes`, scripts pendentes de review independente), linhas de reconciliação abaixo e
  Last commit **bem formado**. Ancestralidade de HEAD é verificada quando possível; clone raso
  ou SHA reescrito (squash/rebase) viram **warning** do doctor, nunca falha nem pass silencioso.
  Não exige HEAD exato nem timestamp de sensores (churn por commit). A checagem estrita
  (`check-live-state.sh --progress docs/harness/progress.md`, sem flag) continua disponível e é a
  que fecha tasks.
- **Trilha de exclusão registrada** (pré-registro exigido por `sensors.sh --exclude-sensor`):
  [`known-issues/2026-05-31-grpc-transport-port-bind.md`](./known-issues/2026-05-31-grpc-transport-port-bind.md)
  (sensor `grpc-transport`, restrição de bind de socket em sandbox; exige `--known-issue` e `--reason`).

### Required versus advisory workflow reconciliation

| Check | Live GitHub API status | Workflow source | Current contract |
|---|---|---|---|
| `Format` | branch-required | `.github/workflows/ci.yml` | Required CI job running `cargo fmt --all -- --check`. |
| `Clippy` | branch-required | `.github/workflows/ci.yml` | Required CI job running clippy with required feature set. |
| `Test (ubuntu-latest)` | branch-required | `.github/workflows/ci.yml` | Required Ubuntu test job for lib/tests, binary tests, WASM checks and the offline harness lane. |
| `Documentation` | branch-required | `.github/workflows/ci.yml` | Required docs job covering MCP reference check and rustdoc. |
| `Security Audit` | branch-required | `.github/workflows/ci.yml` | Live branch-protection API lists this context as required; do not treat older advisory prose as current truth. |
| `Cargo Deny` | branch-required | `.github/workflows/ci.yml` | Live branch-protection API lists this context as required; do not treat older advisory prose as current truth. |
| `Harness Contract` | not in `required_status_checks.contexts` | `.github/workflows/harness-contract.yml` | Workflow exists, but the live API receipt does not list it as a required context; do not infer required status from workflow text. |
| `Harness Doctor Advisory` | advisory workflow job | `.github/workflows/harness-contract.yml` | Non-blocking `doctor.sh`; stays advisory and must not be inferred as required. |

Observado pelo levantamento de 2026-10-26: `Security Audit` e `Cargo Deny` vermelhos desde 2026-10-17 (novos RUSTSEC; frente WT2). A tabela descreve a configuração, não o resultado.

## Histórico (nada foi apagado)

- Tudo o que estava aqui antes de H6 (sprints v0, ondas 10/10, ENG-1241/1295/1296, ENGRA-*,
  lifecycle, AgentShield, Wave 1 de proteção do GitHub etc.) está em
  [`progress-history.md`](./progress-history.md), com índice numerado e âncoras por seção.
- Mais recentes: [Harness Contract workflow YAML repair](./progress-history.md#harness-contract-workflow-yaml-repair--2026-07-12),
  [Agent Memory Contract C1.1](./progress-history.md#agent-memory-contract-c11--pending-writeback-candidates--2026-07-03),
  [GitHub harness protection — Wave 1](./progress-history.md#github-harness-protection--wave-1--2026-06-27),
  [Engram 10/10 Wave 4](./progress-history.md#engram-1010-wave-4--integrated-implementation),
  [Engram 10/10 Wave 3](./progress-history.md#engram-1010-wave-3--execution-started),
  [Live-state self-check 2026-07-10](./progress-history.md#live-state-self-check--2026-07-10).
- Novas tasks acrescentam entrada ao log da lane (`progress/`) e atualizam só os campos e a
  tabela de tasks acima; este arquivo não volta a acumular histórico.
