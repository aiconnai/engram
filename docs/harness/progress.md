# Engram — Harness Progress (Live State)

| Field | Value |
|-------|-------|
| Project | `engram` |
| Active sprint | `Engram comprehensive improvement program — waves 0-4` |
| Active task | `engram-comprehensive-improvement — waves 0-4 execution` |
| Active plan | `docs/harness/progress/2026-10-05-improvement-lane-p.md` |
| Last review | `2026-07-10 — pass: docs/harness/reviews/2026-07-10-engram-10-of-10-live-state-v4-post.md (último review-gate post com marcador). Programa 2026-10-05: revisões SDD independentes + revisão final Opus (Ready with fixes), sem receipt do review-gate post: docs/harness/reviews/2026-10-05-comprehensive-improvement/README.md` |
| Last sensors | `2026-10-05 — FINALFIX (HEAD bdf4620): full status=pass (mode=full; timestamp 2026-10-05T19:07:34Z, 328s) after make ci exit 0; receipts not committed. Earlier: INT full status=pass (2026-10-05T18:11:26Z). Committed receipt .sensors-last (not refreshed, to avoid telemetry churn): status=pass mode=quick timestamp 2026-09-03T13:11:41Z` |
| Last commit | `bdf4620` |
| Last live-state check | `2026-10-05 — FINALFIX: status=pass (rtk bash docs/harness/bin/check-live-state.sh --progress docs/harness/progress.md)` |

> Resumo vivo, **≤150 linhas** (task H6). Orçamentos, medição e regras de retenção:
> [`context-budget.md`](./context-budget.md). O texto anterior deste arquivo foi movido
> **sem alteração** para [`progress-history.md`](./progress-history.md) (índice de 68 seções
> com âncoras estáveis; nada foi apagado). Logs por task em `progress/`.

## Retomada rápida (escopo, limites, última evidência)

- **Escopo**: programa de melhoria abrangente, ondas 0-4, autorizado pelo owner em
  2026-10-05. Esta lane (P) cuida de harness, governança e CI docs. Fonte de verdade:
  [`SPEC.md`](./SPEC.md), [plano](./plans/2026-10-02-engram-comprehensive-improvement-plan.md)
  e o [log da lane P (active plan)](./progress/2026-10-05-improvement-lane-p.md).
- **Limites**: só **commits locais** na branch da lane; sem push, PR, merge, publicação,
  produção/nuvem, issues remotas nem alteração de branch protection. Runners da Onda 4 apenas
  com *fake writer*, offline, sem credenciais reais. Negativo e não concedido:
  [`WHAT_WE_DONT_DO.md`](./WHAT_WE_DONT_DO.md) e o [ADR aceito](../decisions/2026-07-21-agent-harness-hardening-v1.md).
  Merge humano continua obrigatório.
- **Última evidência**: [entrada FINALFIX do log](./progress/2026-10-05-improvement-lane-p.md#finalfix--correções-da-revisão-final-do-branch-2026-10-05)
  (correções da revisão final, gates completos), [entrada INT](./progress/2026-10-05-improvement-lane-p.md#int--integração-das-lanes-q1b-e-todos-de-integração-2026-10-05)
  (lanes P e R integradas, NOT RUN) e
  [entrada H6](./progress/2026-10-05-improvement-lane-p.md#h6--contexto-curto-retomável-e-retenção-explícita--2026-10-05); baseline e discrepâncias em
  [`audits/2026-10-02-improvement-baseline.md`](./audits/2026-10-02-improvement-baseline.md).
  Lane offline obrigatória: `bash docs/harness/bin/run-offline-lane.sh` (`OFFLINE_LANE: PASS`).
- **Como retomar**: `bash docs/harness/bin/bootstrap.sh` → leitura obrigatória na ordem de
  `AGENTS.md`/`CLAUDE.md` (inalterada) → este arquivo → active plan →
  `bash docs/harness/bin/doctor.sh`. Seção histórica inteira, sem cortar parágrafo:
  `python3 docs/harness/bin/measure-context.py --section docs/harness/progress-history.md#<âncora>`.

## Programa ativo — comprehensive improvement, ondas 0-4

- **Status**: active desde 2026-10-05. Plano:
  [`plans/2026-10-02-engram-comprehensive-improvement-plan.md`](./plans/2026-10-02-engram-comprehensive-improvement-plan.md).
  Log da lane R (Rust/produto):
  [`progress/2026-10-05-improvement-lane-r.md`](./progress/2026-10-05-improvement-lane-r.md)
  (integrado ao tree desta branch pelo merge das lanes, task INT).
- **Autorização do owner (2026-10-05, chat)**: ondas 0-4; ADR hardening v1 **aceito**
  (sem Review Canvas registrado — desvio anotado na auditoria E0); WAL 64 GiB configurável;
  `defer_embedding=true` enfileira em background; commits locais apenas.
- **Baseline de execução (decisão do owner)**: `1952f3b`, 14 commits locais à frente de
  `origin/main` `949c963` (0 atrás); permanecem locais e sem revisão independente no repositório.
  O commit do plano `630dd26` é documental e fica sobre essa baseline.

Registro (revisões SDD independentes; **nenhum** `review-gate.sh post` com receipt rodado), ledger,
briefs, reports e revisão final: [`reviews/2026-10-05-comprehensive-improvement/`](./reviews/2026-10-05-comprehensive-improvement/README.md).

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

- **Discrepâncias de E0** (detalhe na auditoria): resolvidas D2 (`RUSTSEC-2026-0285`, Q6: rustls 0.23.45), D3 (Q1a), D4 (Q5 refrescou o
  `GATES.md`; H6 verificou a consistência), D5 (classificada/endurecida em H2/H3), D6, D11,
  D12. Em aberto: D1 renovação de exceções (Q5 renovou até 2026-12-31, owner Ronaldo; INT reconciliou
  os registros após Q6), D7 `default` features, D8 proteção live sem review
  obrigatório, D9 commits sem review/Canvas, D10 consolidação de AGENTS/INVARIANTS/STANDARDS
  (H6 mediu e decidiu: ver `context-budget.md`).
- **#152 (ruling do controller, 2026-10-05, pendente de confirmação do owner)**: limites de chunk
  na ingestão de documentos são em **caracteres**, distintos de tokens; nenhum tokenizer/modelo foi
  adicionado. Entrada para Q7;
  registro em [`context-budget.md`](./context-budget.md#6-decisão-152--chunks-por-caracteres-não-por-tokens).

## Próximos passos (owner)

- **Prazo 2026-11-05**: 3 entradas de `docs/security/supply-chain-pins.toml` expiram; depois disso
  o job security-gate (dependência do required `Test (ubuntu-latest)`) reprova até resolver os
  digests online e renovar as entradas.
- Merge humano: squash ou split autorizado de `35a1592`/`ec3093c`; follow-ups e decisões do owner
  listados no [registro de revisão](./reviews/2026-10-05-comprehensive-improvement/README.md#owner-decisions-surfaced-not-implemented).

## Enforcement do live state

- `doctor.sh` roda `check-live-state.sh --structural` neste arquivo (task H6): campos
  obrigatórios, Active plan existente, review PASS autoritativo, linhas de reconciliação abaixo e
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
