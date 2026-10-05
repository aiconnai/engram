# Engram — Improvement program baseline and reconciliation (task E0)

> **Evidence-only.** Este relatório inventaria, reconcilia e registra; não é gate
> pass/fail, não promove autorização e não prova que a implementação está correta
> (`WHAT_WE_DONT_DO.md`). A única autorização nova que ele registra é a que o owner
> deu em chat em 2026-10-05, reproduzida no ADR.
>
> - Task: E0 do [plano abrangente](../plans/2026-10-02-engram-comprehensive-improvement-plan.md), Lane P.
> - Data de observação: 2026-10-05. Executor: agente implementador (Lane P).
> - Método: leitura do repositório, `git` somente leitura (`git fetch` permitido, nenhum push),
>   `gh api` somente leitura e execução local dos gates. Nenhuma escrita remota.
> - Worktree: `engram-improvement-lane-p`, branch `claude/improvement-lane-p`.

## 1. Baseline de execução congelada

Decisão do owner (Ronaldo, chat, 2026-10-05): **os 14 commits locais são a baseline de
execução e permanecem locais (sem push)**. Nenhum deles foi publicado, descartado ou
incorporado implicitamente por esta task.

| Fato | Valor (re-verificado em 2026-10-05 por git/gh somente leitura) |
|---|---|
| Repo remoto | `https://github.com/aiconnai/engram.git` (`origin`) |
| `origin/main` (após `git fetch`) | `949c9634be28badea43ba2a2b5bcdf5d4c0dd358` |
| `main` no GitHub (`gh api repos/aiconnai/engram/branches/main`) | `949c9634be28badea43ba2a2b5bcdf5d4c0dd358` (igual ao fetch) |
| Baseline de código (HEAD dos 14 commits) | `1952f3b461fe7a8f8a6b5c106ac8c4f5929bae91`, árvore `66cb1885bd4ed47a01d60d5e4733a2099cea2623` |
| Merge-base `1952f3b` / `origin/main` | `949c9634be28badea43ba2a2b5bcdf5d4c0dd358` (o próprio `origin/main`) |
| Divergência `origin/main...1952f3b` | 0 atrás / **14 à frente** (confirma o plano) |
| HEAD desta worktree ao iniciar E0 | `630dd26c72c69dcee81b53a16230022596b89602` (= baseline + 1 commit documental do plano), árvore `e612d8ed2945e1a9d0f8d038066f8a05d2c49054`; 0 atrás / 15 à frente |
| Publicação dos commits locais | `gh api repos/aiconnai/engram/compare/main...1952f3b…` → **404 Not Found**: os objetos não existem no GitHub |
| Diff `origin/main..1952f3b` | 92 arquivos, +8651 / −579; `git diff --binary` SHA-256 `0a72d6e6eca4f5d3f69b400f166787817a64655e0e51ce9b73dad4bebac237b2` |
| Manifest (`git diff --name-status origin/main 1952f3b \| sort`) | 92 linhas, SHA-256 `ca635820642fe65cf56ca632a26f99ff75268c49e7598e2391dc248baf4827e7` |
| Delta documental do plano `1952f3b..630dd26` | 1 arquivo (o plano); `git diff --binary` SHA-256 `8ed57ae5e2ea5872831816baa6ea4733ad79b6fc430f8d56b9c7a395d377f936` |
| Outras worktrees locais | `main` (checkout principal) em `1952f3b`; worktrees `wave4-todo22…39` e `harness-hardening-v1` em branches próprias — **não tocadas** |

Para refazer: `git diff --name-status origin/main 1952f3b | sort | shasum -a 256` (manifest) e
`git diff --binary origin/main 1952f3b | shasum -a 256` (diff). Se `origin/main` avançar, o re-cálculo
muda por construção; a baseline é o par (`1952f3b`, `949c963`) acima.

### 1.1 Os 14 commits locais: status de revisão e publicação

Todos: **não publicados** (404 acima). **Nenhum** deles adiciona artefato em `docs/harness/reviews/` ou
`docs/harness/canvas/` (`git diff --name-only origin/main..HEAD -- docs/harness/reviews docs/harness/canvas`
é vazio): não há review independente registrado no repositório para o intervalo. Mensagens que citam revisão
(`48b1965 "remediate max code review findings"`) ou "full deterministic pass" (`16ddab7`) são afirmações de
commit, **não** evidência verificável; os únicos receipts são `.sensors-last/.sensors-log` (telemetria
operacional, ver §5).

| # | SHA | Data | Assunto | Classe | Review/Canvas no repo |
|--:|---|---|---|---|---|
| 1 | `b58029f` | 2026-08-24 | test(wal): add decompression bomb guard test + refactor unpack_frames | produto/teste (`src/sync/wal_replication.rs`) | nenhum |
| 2 | `4fecbc0` | 2026-08-24 | feat(routing): implement RFC 0011 Model Routing Contract and spatial review fixes | produto (42 arquivos; CLI, MCP, SDKs) | nenhum |
| 3 | `48b1965` | 2026-08-30 | fix(review): remediate max code review findings across aaak, routing, and wal replication | produto (13 arquivos) | nenhum |
| 4 | `16ddab7` | 2026-08-30 | chore(harness): update sensors receipts after full deterministic pass | telemetria | n/a |
| 5 | `2313d8a` | 2026-09-03 | feat(harness,mcp): RFC 0009 contract validator, RFC 0010 permission modes, harness hardening wave 1 | **misto**: harness (schemas/validator/fixtures) + MCP/SDK/produto, 26 arquivos | nenhum (sem Canvas, apesar de tocar harness/MCP) |
| 6 | `0032cb6` | 2026-09-03 | docs(standards): codify embedded storage, unicode safety, agentic governance | docs/política (`INVARIANTS.md` raiz, `STANDARDS.md`, `AGENTS.md`) | nenhum |
| 7 | `6abd6fb` | 2026-09-06 | docs(standards): engineering standard v5, governance manifests, pin toolchain | docs + `rust-toolchain.toml` (1.96.0) + `governance/*.toml` | nenhum |
| 8 | `8e81c8b` | 2026-09-06 | perf(build): consolidate build velocity, dedupe deps, storage desmonomorphization | build/deps/CI cache/storage | nenhum |
| 9 | `1fdffc5` | 2026-09-06 | ci: integrate cargo-nextest | **workflow CI** (`ci.yml`, `Makefile`, `scripts/ci.sh`) | nenhum |
| 10 | `80a4df6` | 2026-09-07 | perf(build): decouple cloud feature from default set | **contrato de build público** (`default = ["openai"]`; `cloud` sai do default) | nenhum |
| 11 | `1a7fa53` | 2026-09-07 | perf(storage): add desmonomorphized query helpers and DbConnectionExt | produto/storage | nenhum |
| 12 | `823878a` | 2026-09-10 | refactor(workspace): extract engram-types subcrate into Cargo workspace | estrutura de crates | nenhum |
| 13 | `e768625` | 2026-09-10 | feat(governance): enhance cargo-deny rules and add local audit targets | `deny.toml`, `Makefile`, `justfile`, `Cargo.lock` | nenhum |
| 14 | `1952f3b` | 2026-09-16 | docs(standards): add rust-engineering-guardrails companion document | docs | nenhum |
| — | `630dd26` | 2026-10-05 | docs(harness): add comprehensive improvement plan with owner authorization | plano (não é um dos 14) | n/a |

Consequência: a baseline escolhida contém mudança de CI/workflow (#9), de contrato de build (#10) e de política de
engenharia (#6, #7) **sem review independente registrado**. Elas valem como ponto de partida de execução, não como
aprovadas. Cada uma tem destino em §6.

## 2. Inventário de capacidades

Legenda: **Impl** = código/arquivos existem; **Ativada** = ligada a doctor/sensors/CI ou ao runtime;
**Política** = status de aprovação no ADR/GATES/INVARIANTS. "Evidence SHA" é onde a capacidade entrou
(`git log --reverse -- <path>`), não prova de que está correta. Owner = quem decide/mantém (task do plano entre
parênteses); a decisão final de aceite é sempre do owner humano Ronaldo.

### 2.1 Harness hardening (ADR `agent-harness-hardening-v1`)

| Capacidade | Implementação | Impl / Ativada / Política | Evidence SHA | Owner (task) |
|---|---|---|---|---|
| ADR de trust boundaries, plano Wave 0, Canvas, intake de referência | `docs/decisions/2026-07-21-agent-harness-hardening-v1.md`, `docs/harness/progress/2026-07-21-agent-harness-hardening-v1.md`, `docs/harness/canvas/2026-07-21-agent-harness-hardening-v1.md` | Impl; n/a; **Accepted em 2026-10-05** (era Proposed desde 2026-07-21) | `6064847` (em `origin/main`); aceite registrado por E0 | Ronaldo (E0 registra) |
| Review independente da Wave 0 (marcador) | `docs/harness/reviews/2026-07-22-agent-harness-hardening-v1-v3-post.md` (`REVIEW_VERDICT: PASS`) | Impl; histórica; não é evidência SHA-bound futura | `6064847` (em `origin/main`) | harness (H1/H5) |
| Schemas JSON `task-v1`, `evidence-v1`, `review-v1` | `docs/harness/schemas/*.schema.json` | **Impl; não ativada** (nenhuma referência em `doctor.sh`, `sensors.sh`, `ci.sh`, workflows); política: Wave 1, agora desbloqueada pelo ADR aceito, mas **não** TCB | `2313d8a` (2026-09-03, local-only) | H2 |
| Validador estático + suíte de fixtures | `docs/harness/bin/validate-evidence.py` (697 linhas), `docs/harness/bin/test-fixtures.sh`, `docs/harness/fixtures/*.json` (6) | **Impl; não ativada**; `test-fixtures.sh` roda localmente **15/15 pass** (E0, 2026-10-05); não vira TCB automaticamente | `2313d8a` (local-only) | H2 |
| Task registry / sandbox adapter / scope checker | — | **Ausente** (nenhum arquivo); Wave 2 | — | H3 |
| Runner com *fake writer* + evidência externa pós-commit | — | **Ausente**; Wave 2; fake writer somente, offline | — | H4 |
| Review SHA-bound estrito + merge-policy read-only | só `review-v1.schema.json`; o gate vigente continua `review-gate.sh` com `REVIEW_VERDICT:` por marcador | Parcial; o marcador permanece o gate aprovado até a Wave 3 | schema `2313d8a`; gate desde o bootstrap do harness | H5 |
| Writer real / execução autônoma / fallback host | — | **Não autorizado** (ADR: só fake writer; sem fallback) | — | fora de escopo |

### 2.2 Contratos de gate e CI

| Capacidade | Implementação | Impl / Ativada / Política | Evidence SHA | Owner (task) |
|---|---|---|---|---|
| Sensores determinísticos (`sensors.sh`: full/quick/docs/mcp/baseline) | `docs/harness/bin/sensors.sh` | Impl; ativada local; política: `GATES.md` (full é o gate canônico) | harness bootstrap (`abeda17` introduz `scripts/ci.sh`, 2026-05-30) | harness |
| `doctor.sh`, `bootstrap.sh`, `check-live-state.sh` | `docs/harness/bin/*.sh` | Impl; ativadas; `check-live-state.sh` **não** é chamado por doctor/sensors | harness bootstrap | harness (H1/H6) |
| Security Gate agregado (7 constituintes) | `.github/workflows/ci.yml` job `security-gate`, `scripts/check-security-gate.py`, `tests/fixtures/security_gate_matrix.json` | Impl; ativado em CI; **obrigatório de forma transitiva** via `Test (ubuntu-latest)` `needs: security-gate` | `962655a` (2026-07-13, em `origin/main`) | Q5 |
| Política de exceções de advisory | `docs/security/advisory-exceptions.toml`, `scripts/check-security-exceptions.py` | Impl; ativada (constituinte `security-exceptions`); **hoje vermelha** (D1) | `81be152` (2026-07-12) | Q5/Q6 |
| Budgets de retrieval/performance em CI | `scripts/check-quality-budgets.py`, passo "Enforce retrieval and performance budgets" em `ci.yml` | Impl; ativada; **quebrada na baseline local** — passo sem `--criterion` (D3) | `5782340` (2026-07-13); regressão em `1fdffc5` (local-only) | Q1 |
| `nextest` em CI e local | `ci.yml`, `Makefile`, `scripts/ci.sh` | Impl; ativada **só localmente** (não publicada); alteração de workflow sem review (§1.1 #9) | `1fdffc5` (local-only) | Q1/Q5 |
| Harness Contract (bootstrap + título de PR) | `.github/workflows/harness-contract.yml` | Impl; ativada em CI; **não** listada como required na proteção live (D4) | histórico de `harness-contract.yml` | H6 |
| Proteção de branch `main` | configuração remota (não versionada) | Ver §4; lida read-only | n/a | Ronaldo |

### 2.3 Capacidades de produto que chegaram nos 14 commits (para a Lane R; não aprovadas)

| Capacidade | Implementação | Impl / Ativada / Política | Evidence SHA | Owner (task) |
|---|---|---|---|---|
| RFC 0011 Model Routing | `src/routing/*`, `src/mcp/handlers/model_routing.rs`, `src/bin/cli/routing.rs`, SDKs, `tests/model_routing_contract_tests.rs`, `governance/models.toml` | Impl; presente no runtime (handler, CLI, SDKs); RFC 0011 ainda `Status: proposed`; sem review no repo | `4fecbc0`, `48b1965`, `6abd6fb` | Lane R |
| RFC 0010 Permission modes | `src/mcp/permission.rs`, `tests/permission_modes_tests.rs`, SDKs | Impl; RFC 0010 `Status: proposed`; entrou **no mesmo commit** dos schemas de harness | `2313d8a` | Lane R (C1) |
| RFC 0009 MCP contract validator (reescrito) | `scripts/validate_mcp_contract.py`, `scripts/test_validate_mcp_contract.py` | Impl; RFC 0009 `Status: proposed` | `2313d8a` | Lane R (Q4) |
| Guarda de decompression bomb no replay WAL (teste) + refator | `src/sync/wal_replication.rs` | Impl (teste do guarda existente); o cap de bytes de `db_size_pages` ainda não existe — budget 64 GiB aprovado, implementação em C2 | `b58029f`, `48b1965` | C2 |
| Workspace `engram-types` | `crates/engram-types/`, `Cargo.toml` | Impl; ativado; sem review | `823878a` | Lane R |
| Default features sem `cloud` | `Cargo.toml` (`default = ["openai"]`) | Impl; **muda o contrato de build público** (D7) | `80a4df6` | Ronaldo / Q6 |
| Padrão de engenharia Rust v5, guardrails, toolchain 1.96.0, `governance/*.toml` | `docs/standards/*`, `rust-toolchain.toml`, `governance/` | Docs/política não revisados; `INVARIANTS.md` e `STANDARDS.md` da raiz alterados (§1.1 #6, #7) | `0032cb6`, `6abd6fb`, `1952f3b` | Ronaldo (H6/Q2) |

## 3. Introdução dos schemas/validator já existentes

`git log --reverse -- docs/harness/schemas/ docs/harness/bin/validate-evidence.py docs/harness/bin/test-fixtures.sh docs/harness/fixtures/`
aponta **um único commit de introdução**: `2313d8ae9cb5948f208ff2391feb1e371a192eb5` (2026-09-03, autor Ronaldo Martins,
`feat(harness,mcp): implement RFC 0009 contract validator, RFC 0010 permission modes, and harness hardening wave 1`).

- É um dos 14 commits **locais** (não está em `origin/main`). Foi escrito quando o ADR ainda estava *Proposed* e,
  portanto, antes da autorização da Wave 1 (o ADR diz que cada onda parte de branch limpa após aceite da anterior).
- Mistura harness com código de produto (RFC 0010, SDKs, MCP handlers, `docs/MCP_TOOLS.md`), contra o princípio
  "harness-only não modifica MCP/SDKs" (`WHAT_WE_DONT_DO.md`). Sem Canvas e sem review independente.
- Não está ligado a `doctor.sh`, `sensors.sh`, `scripts/ci.sh` nem a nenhum workflow. A suíte `test-fixtures.sh` passa
  15/15 localmente, o que mostra que o validador rejeita as 3 fixtures adversariais atuais, **não** que o parser é
  fail-closed completo (o plano H2 lista lacunas a testar: chaves JSON duplicadas, NaN/Infinity, bool como inteiro etc.;
  não testadas em E0).
- Decisão: **não recriar** e **não tratar como TCB**. Seguem como artefatos candidatos; H2 os endurece e liga a uma lane
  offline obrigatória; H3 só começa quando H2 estiver aprovado (plano §5).

## 4. Reconciliação do Security Gate (GATES.md × workflow × proteção live × scanners)

### 4.1 O que cada fonte diz

| Fonte | Afirma |
|---|---|
| `GATES.md`, seção "Required aggregate security gate" | Job `Security Gate` agrega Cargo Audit, Cargo Deny, exceções governadas, CodeQL, Semgrep, Gitleaks, AgentShield; falha/skip inesperado = vermelho; a proteção **não** é alterada por automação; `Test (ubuntu-latest)` depende de `security-gate` → bloqueio **transitivo** |
| `GATES.md`, seção "Required checks no GitHub" (**antiga**) | Required = `Format`, `Clippy`, `Test (ubuntu-latest)`, `Documentation` + `Harness Contract`; `Security Audit` e `Cargo Deny` **advisory** (por `RUSTSEC-2026-0187` em `lopdf` e `RUSTSEC-2026-0185` em `quinn-proto`) |
| `ci.yml` (HEAD) | `test` tem `needs: [fmt, clippy, security-gate]`; `security-gate` (`if: always()`) depende dos 7 jobs; comentários de `audit`/`deny` dizem que o contexto live "may also require this job directly" |
| `docs/harness/progress.md` (tabela de reconciliação, 2026-07-10) | `Security Audit` e `Cargo Deny` = **branch-required**; `Harness Contract` **não** está em `required_status_checks.contexts` |
| **Proteção live de `main`** (`gh api …/branches/main/protection[/required_status_checks]`, leitura, 2026-10-05) | `strict: true`; contexts = `Format`, `Clippy`, `Documentation`, `Test (ubuntu-latest)`, `Security Audit`, `Cargo Deny` (todos `app_id` 15368). **Sem** `Security Gate` nem `Harness Contract`. Sem `required_pull_request_reviews`; `enforce_admins: false`; sem rulesets (`[]`). SHA-256 do JSON de contexts: `545b7c986c1325cae1433f246b538b747dc3ef2bf3f4cd61adc1e61ea63ace8c` |
| `scripts/check-security-gate.py` com os contexts live | `security-gate contract: PASS`; `--self-test-failure` PASS; `--self-test-unrequired` PASS (exit 0 nos três) |

### 4.2 Conclusão da reconciliação

1. A cadeia obrigatória **transitiva** é real e verificada: `Test (ubuntu-latest)` (required) → `security-gate` → 7
   constituintes. Para PR, falha em qualquer constituinte bloqueia o merge, **além** de `Security Audit` e `Cargo Deny`
   serem required diretamente.
2. A seção antiga de `GATES.md` ("Security Audit/Cargo Deny advisory", "Harness Contract required",
   `RUSTSEC-2026-0187/0185` em `lopdf`/`quinn-proto`) está **desatualizada** em três pontos: (a) Audit/Deny são required
   na proteção live; (b) `Harness Contract` não é required live; (c) o lock atual tem `lopdf 0.42.0` e `quinn-proto 0.11.15`
   (versões que satisfazem os floors da issue #177) e os advisories ativos hoje são outros (§4.3). `progress.md` já
   registra (a) e (b) corretamente. **E0 não edita `GATES.md`** (política executável do harness; mudança exige governança
   dedicada e review pela versão anterior dos gates); a correção fica atribuída a H6 (D4).
3. `Security Gate` em si **não** é um context required; a obrigatoriedade depende de `Test (ubuntu-latest)` manter
   `needs: security-gate` (o checker acima protege isso). Renomear/remover esse `needs` exige re-rodar o checker com os
   contexts live.
4. A proteção live **não** exige revisão de PR e `enforce_admins` é `false`: "merge humano obrigatório" é regra de
   processo do programa, **não** controle técnico do GitHub. Registrado como D8 para decisão do owner; E0 não altera
   nada remoto.

### 4.3 Estado dos scanners hoje (execução local, offline)

| Verificação | Resultado (2026-10-05) |
|---|---|
| `python3 scripts/check-security-exceptions.py --config docs/security/advisory-exceptions.toml --audit-config .cargo/audit.toml --deny-config deny.toml` | **exit 1 (FAIL)**: 10 exceções expiraram em 2026-09-30 (RUSTSEC-2026-0049, -0098, -0099, -0104, -0258, 2025-0141, 2024-0436, 2025-0134, 2026-0192, 2026-0235). O constituinte `security-exceptions` do Security Gate falharia em PR hoje. |
| `cargo audit --no-fetch` (DB local em cache, commit `ef6173c` de 2026-10-03; 1290 advisories, 666 crates) | **exit 1**: 1 vulnerabilidade **não ignorada** `RUSTSEC-2026-0285` (`rustls 0.23.36`, 5.3 medium; solução `>=0.23.45`); avisos de `ttf-parser` (0192), `event-listener` (0221), `lru` (0253) |
| `cargo deny check --disable-fetch advisories bans licenses sources` | **exit 1**: `advisories FAILED` (mesmo `RUSTSEC-2026-0285`), `bans ok`, `licenses ok`, `sources ok`; vários `warning[duplicate]` (multiple-versions = warn) |
| CodeQL, Semgrep, Gitleaks, AgentShield | **NOT RUN** (executam só em CI/containers; não são offline) |

Limitação: o DB de advisories é um cache local de 2026-10-03, sem `git fetch`; o resultado pode mudar com DB atualizado.
Não é afirmação de que o `main` remoto está vermelho: ele não contém os 14 commits locais.

## 5. Gates executados

| Gate | Comando | Resultado |
|---|---|---|
| Bootstrap | `bash docs/harness/bin/bootstrap.sh` | exit 0; branch limpa; mostrava ainda o sprint antigo (antes da troca de live state) e última sensors 2026-09-03 quick pass |
| Doctor (pré-edição, HEAD `630dd26`) | `bash docs/harness/bin/doctor.sh` | `OK harness doctor` |
| Doctor (pós-troca SPEC/progress/ADR) | idem | `OK harness doctor` + `WARN: no review artifact found for active task: engram-comprehensive-improvement (expected after first post-gate)` (warn esperado, não é falha) |
| `check-live-state.sh` (pré-edição) | `bash docs/harness/bin/check-live-state.sh --progress docs/harness/progress.md` | **FAIL** (`Last commit b7ecea9` stale; `Last sensors` sem o timestamp `2026-09-03T13:11:41Z`); campos corrigidos por E0; re-execução pós-edição: `PASS live state matches current repository facts` (exit 0) |
| Fixtures do validador | `bash docs/harness/bin/test-fixtures.sh` | exit 0, `Passed: 15 Failed: 0` |
| Security gate checker | ver §4.1 | PASS ×3 |
| Sensores `quick` | `bash docs/harness/bin/sensors.sh quick` | **pass** (exit 0; mode=quick; início 2026-10-05T03:52:34Z, `timestamp=2026-10-05T03:53:13Z`, 39 s; fmt + cargo check + pr-title-policy + doctor). Primeira compilação da worktree incluída. |
| Sensores **full** (sem argumentos) | `bash docs/harness/bin/sensors.sh` | **pass** (exit 0; mode=full; `timestamp=2026-10-05T03:57:55Z`, `duration_sec=282`; `ci_steps`: fmt, clippy, test_lib, test_integration, test_integration_watch, wasm_target, wasm_all_targets, wasm_wasm_target, doc, ref_check = pass; `doctor_status=pass`; sem exclusões). Rodado sobre HEAD `630dd26` + edições documentais não commitadas desta task (SPEC/progress/ADR/auditoria/log); nenhum código foi alterado. **Cuidado (fake-success):** `sensors.sh` full **não** executa `check-quality-budgets.py`, `check-security-exceptions.py`, `cargo audit`/`deny` nem o Security Gate; por isso fica verde enquanto D1, D2 e D3 (§6) existem. Verde dos sensores não implica CI verde. |

**Receipts operacionais.** `.sensors-last` e `.sensors-log` são telemetria rastreada pelo git. Esta task **não** os
commita (evita conflito de merge cross-lane no `.sensors-log` append-only); os resultados acima vêm dos logs reais da
execução. O último receipt **commitado** é `mode=quick`, `status=pass`, `timestamp=2026-09-03T13:11:41Z`; o último
receipt **full** commitado é de `2026-08-30T20:16:09Z` (anterior a 10 dos 14 commits locais) e **não** vale como
evidência da baseline atual.

## 6. Discrepâncias, resolução e owner

| ID | Discrepância | Evidência | Resolução / status | Owner (task) |
|---|---|---|---|---|
| D1 | `Security exception policy` vermelho: 10 exceções expiradas em 2026-09-30; dependency paths citam `engram-core 0.22.0` (workspace atual 0.23.0) | §4.3 | **Aberta.** Renovar com racional datado ou remover exige decisão humana de supply-chain; E0 não edita `advisory-exceptions.toml`/`deny.toml`/`.cargo/audit.toml` | Q5/Q6 (supply-chain) |
| D2 | `RUSTSEC-2026-0285` (`rustls 0.23.36`) falha `cargo audit`/`cargo deny` localmente; não está em nenhuma lista de ignore | §4.3 | **Aberta.** Upgrade (`>=0.23.45`) ou exceção governada é decisão por PR; sem afirmar que o `main` remoto falha | Q5/Q6 |
| D3 | `--criterion benches/results/benchmark_results.txt` foi removido do passo de budgets de `ci.yml` por `1fdffc5`; `check-quality-budgets.py` sai com **exit 2** sem o argumento (reproduzido). Em `origin/main` o argumento existe (`5782340`) | `git log -S`, execução local | **Aberta.** Se os 14 commits fossem publicados, o job required `Test (ubuntu-latest)` falharia antes dos testes. A correção de `ci.yml` pertence a Q1 | Q1 |
| D4 | `GATES.md` seção "Required checks no GitHub" desatualizada (Audit/Deny advisory; `Harness Contract` required; advisories `0187/0185`) vs live e `progress.md` | §4.2 | **Aberta, não alterada em E0** (política do harness exige governança dedicada). Corrigir em H6 ou tarefa de política com review pela versão anterior dos gates | H6 (lane P) |
| D5 | Schemas/validator/fixtures (`2313d8a`) existem e passam 15/15, mas não estão ativados, entraram antes do aceite do ADR e junto de código de produto sem review | §3 | **Resolvida como classificação**: candidatos, fora do TCB; endurecimento e lane obrigatória em H2 | H2 |
| D6 | `check-live-state.sh` stale (Last commit `b7ecea9`, sensors) e SPEC/progress apontavam para a sprint de julho | §5 | **Resolvida** por E0 (SPEC/progress trocados; campos atualizados) | E0 |
| D7 | `80a4df6` altera `default = ["openai"]` (remove `cloud`): muda o conjunto de features padrão de `cargo build` sem review; `scripts/ci-required-features.env` não depende do default | `git show 80a4df6` | **Aberta.** Decisão do owner: manter, reverter ou documentar como breaking; E0 não toca `Cargo.toml` | Ronaldo / Q6 |
| D8 | Proteção live de `main` sem `required_pull_request_reviews`, `enforce_admins=false`; `Security Gate` e `Harness Contract` não são contexts required | §4.1 | **Aberta, somente registrada.** Nenhuma alteração remota; mudar proteção é decisão do owner | Ronaldo |
| D9 | Os 14 commits não têm review nem Canvas no repo (inclusive os de código, CI e política); RFCs 0009/0010/0011 ainda `proposed` | §1.1 | **Aberta.** Baseline aceita pelo owner como ponto de partida; revisão por task conforme DAG do plano | Ronaldo / lanes |
| D10 | `INVARIANTS.md` raiz, `STANDARDS.md`, `AGENTS.md` alterados por `0032cb6`/`6abd6fb` sem ADR/review | §1.1 #6, #7 | **Aberta, registrada.** O escopo de H6/Q2 decide o que consolidar | Ronaldo (H6/Q2) |
| D11 | O brief E0 previa SPEC/progress/ADR "só em governança dedicada"; o owner autorizou em chat a troca de live state e o aceite do ADR dentro de E0 | brief × decisão de 2026-10-05 | **Resolvida por decisão do owner** (registrada no ADR e no log da lane P) | Ronaldo |
| D12 | O aceite do ADR pressupunha PR dedicado da Wave 0 com full sensors e review; o owner aceitou direto em chat e o programa não abre PR | ADR | **Resolvida por decisão do owner**; a evidência de gates fica registrada aqui sem substituir o aceite | Ronaldo |

Nenhuma discrepância foi resolvida mudando datas/SHAs para fabricar verde.

## 7. Limitações e itens NOT RUN

- Nenhum push, PR, merge, escrita remota, issue ou alteração de proteção. Consultas `gh` somente leitura.
- CodeQL/Semgrep/Gitleaks/AgentShield: NOT RUN (CI). `cargo audit`/`cargo deny` com DB local sem fetch (§4.3).
- Risco/cobertura dos 14 commits de produto **não** foi avaliado (fora do escopo de E0); só classificados e listados.
- Issues remotas (#143–#183) não foram consultadas em E0 (o plano §9 é observação local anterior).
- `.sensors-last/.sensors-log` não foram commitados (ver §5).

## 8. Aceite e rollback

- **Aceite de E0:** discrepâncias têm resolução/owner (§6); ADR **Accepted** com registro verificável (owner, data, fonte,
  histórico *Proposed* preservado) no próprio ADR; nenhuma autorização promovida além da dada pelo owner em chat.
- **Rollback:** reverter o commit de E0 (um commit documental). Devolve o ADR a *Proposed* e SPEC/progress ao estado
  anterior; nenhum receipt é apagado.

## 9. Notas posteriores (task H6, 2026-10-05; as seções acima ficam como registradas por E0)

- **Desvio registrado — sem Review Canvas para o aceite do ADR.** O aceite do ADR
  `agent-harness-hardening-v1` foi decidido em chat pelo owner e gravado por E0, mas **nenhum Review
  Canvas** (`docs/harness/canvas/TEMPLATE.md`) foi registrado para essa decisão; só o registro de aceite
  no próprio ADR e esta auditoria existem. O aceite do owner permanece válido; o desvio fica visível
  aqui e em D9/D12 em vez de ser preenchido retroativamente.
- **Atribuição da fonte do aceite corrigida.** A frase "Tudo, inclusive Onda 4" foi a resposta do
  owner no chat, não texto da nota de autorização do plano; a linha *Source* do ADR agora diz isso.
- **D3** resolvida por Q1a (`1addb94`). **D4**: Q5 refrescou a seção "Required checks no GitHub" do
  `GATES.md`; H6 reverificou contra §4.1 (seis contexts required, `Security Gate`/`Harness Contract`
  fora, sem review de PR obrigatório) e esclareceu que `doctor.sh` roda no job separado
  `Harness Doctor Advisory`; sem divergência restante. **D5** endurecida/classificada em H2/H3.
- **D10**: H6 mediu AGENTS.md/CLAUDE.md e **não** consolidou nem alterou
  `INVARIANTS.md` raiz, `STANDARDS.md` ou a lista de leitura; a decisão (duplicação deliberada,
  protegida por teste de ordem) está em `docs/harness/context-budget.md`.
