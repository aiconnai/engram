# Quarterly Harness Audit

Date: 2026-06-05T15:59:33Z
Repo: `engram`
Mode: evidence-only

This report gathers evidence for human cleanup and drift review. It does not declare pass/fail and does not delete, archive, or rewrite anything.

## How To Use

1. Review each evidence section.
2. Fill decision tables with `Keep`, `Archive`, or `Delete`.
3. Convert accepted cleanup into focused tasks or issues.
4. Keep exceptions documented in `docs/harness/WHAT_WE_DONT_DO.md`, `docs/harness/INVARIANTS.md`, or an ADR.

### Current branch and commit

```bash
git branch --show-current && git log -1 --oneline
```

```text
main
ce65877 fix(mcp): align tool registry and markdown sync (#53)
exit_status=0
```

### Working tree status

```bash
git status --short
```

```text
 M docs/harness/.sensors-last
 M docs/harness/CODE_REVIEW_POLICY.md
 M docs/harness/GATES.md
 M docs/harness/INVARIANTS.md
 M docs/harness/README.md
 M docs/harness/SPEC.md
 M docs/harness/bin/bootstrap.sh
 M docs/harness/bin/doctor.sh
 M docs/harness/bin/review-gate.sh
 M docs/harness/bin/sensors.sh
 M docs/harness/progress.md
 M docs/harness/progress/2026-05-30-harness-bootstrap.md
?? docs/harness/.baseline-last
?? docs/harness/WHAT_WE_DONT_DO.md
?? docs/harness/audits/
?? docs/harness/bin/baseline.sh
?? docs/harness/bin/quarterly-audit.sh
?? docs/harness/canvas/
?? docs/harness/plans/2026-06-05-cross-harness-improvement-plan.md
?? docs/harness/plans/2026-06-05-engram-harness-improvement-execution-plan.md
exit_status=0
```

### Harness policy references

```bash
rg -n 'WHAT_WE_DONT_DO|CODE_REVIEW_POLICY|review-gate|doctor.sh|sensors.sh|baseline.sh|quarterly-audit' docs/harness README.md AGENTS.md Claude.md 2>/dev/null | head -160
```

```text
AGENTS.md:98:- `docs/harness/CODE_REVIEW_POLICY.md`
AGENTS.md:107:bash docs/harness/bin/doctor.sh
AGENTS.md:108:bash docs/harness/bin/sensors.sh          # runs just ci + doctor (pode demorar)
AGENTS.md:109:bash docs/harness/bin/review-gate.sh pre harness-bootstrap
Claude.md:77:`docs/harness/GATES.md`, `docs/harness/CODE_REVIEW_POLICY.md`,
Claude.md:87:bash docs/harness/bin/doctor.sh
Claude.md:88:bash docs/harness/bin/sensors.sh   # full deterministic gates (wraps just ci)
docs/harness/reviews/2026-05-31-harness-bootstrap-v2-pre.md:17:- docs/harness/CODE_REVIEW_POLICY.md (this policy)
docs/harness/reviews/2026-05-31-harness-bootstrap-v2-pre.md:56:+- `docs/harness/CODE_REVIEW_POLICY.md`
docs/harness/reviews/2026-05-31-harness-bootstrap-v2-pre.md:64:+bash docs/harness/bin/doctor.sh
docs/harness/reviews/2026-05-31-harness-bootstrap-v2-pre.md:65:+bash docs/harness/bin/sensors.sh          # runs just ci + doctor (pode demorar)
docs/harness/reviews/2026-05-31-harness-bootstrap-v2-pre.md:66:+bash docs/harness/bin/review-gate.sh pre harness-bootstrap
docs/harness/reviews/2026-05-31-harness-bootstrap-v2-pre.md:81:+Then read in order: `docs/harness/SPEC.md`, `INVARIANTS.md`, `GATES.md`, `CODE_REVIEW_POLICY.md`, `progress.md`.
docs/harness/reviews/2026-05-31-harness-bootstrap-v2-pre.md:88:+bash docs/harness/bin/doctor.sh
docs/harness/reviews/2026-05-31-harness-bootstrap-v2-pre.md:89:+bash docs/harness/bin/sensors.sh   # full deterministic gates (wraps just ci)
docs/harness/reviews/2026-05-31-harness-bootstrap-v10-post.md.raw:17:- docs/harness/CODE_REVIEW_POLICY.md (this policy)
docs/harness/reviews/2026-05-31-harness-bootstrap-v10-post.md.raw:64:+- `docs/harness/CODE_REVIEW_POLICY.md`
docs/harness/reviews/2026-05-31-harness-bootstrap-v10-post.md.raw:72:+bash docs/harness/bin/doctor.sh
docs/harness/reviews/2026-05-31-harness-bootstrap-v10-post.md.raw:73:+bash docs/harness/bin/sensors.sh          # runs just ci + doctor (pode demorar)
docs/harness/reviews/2026-05-31-harness-bootstrap-v10-post.md.raw:74:+bash docs/harness/bin/review-gate.sh pre harness-bootstrap
docs/harness/reviews/2026-05-31-harness-bootstrap-v10-post.md.raw:90:+`docs/harness/GATES.md`, `docs/harness/CODE_REVIEW_POLICY.md`,
docs/harness/reviews/2026-05-31-harness-bootstrap-v10-post.md.raw:98:+bash docs/harness/bin/doctor.sh
docs/harness/reviews/2026-05-31-harness-bootstrap-v10-post.md.raw:99:+bash docs/harness/bin/sensors.sh   # full deterministic gates (wraps just ci)
docs/harness/reviews/2026-05-31-harness-bootstrap-v4-pre.md.raw:17:- docs/harness/CODE_REVIEW_POLICY.md (this policy)
docs/harness/reviews/2026-05-31-harness-bootstrap-v4-pre.md.raw:56:+- `docs/harness/CODE_REVIEW_POLICY.md`
docs/harness/reviews/2026-05-31-harness-bootstrap-v4-pre.md.raw:64:+bash docs/harness/bin/doctor.sh
docs/harness/reviews/2026-05-31-harness-bootstrap-v4-pre.md.raw:65:+bash docs/harness/bin/sensors.sh          # runs just ci + doctor (pode demorar)
docs/harness/reviews/2026-05-31-harness-bootstrap-v4-pre.md.raw:66:+bash docs/harness/bin/review-gate.sh pre harness-bootstrap
docs/harness/reviews/2026-05-31-harness-bootstrap-v4-pre.md.raw:81:+Then read in order: `docs/harness/SPEC.md`, `INVARIANTS.md`, `GATES.md`, `CODE_REVIEW_POLICY.md`, `progress.md`.
docs/harness/reviews/2026-05-31-harness-bootstrap-v4-pre.md.raw:88:+bash docs/harness/bin/doctor.sh
docs/harness/reviews/2026-05-31-harness-bootstrap-v4-pre.md.raw:89:+bash docs/harness/bin/sensors.sh   # full deterministic gates (wraps just ci)
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:14:4. Keep exceptions documented in `docs/harness/WHAT_WE_DONT_DO.md`, `docs/harness/INVARIANTS.md`, or an ADR.
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:36: M docs/harness/CODE_REVIEW_POLICY.md
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:42: M docs/harness/bin/doctor.sh
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:43: M docs/harness/bin/review-gate.sh
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:44: M docs/harness/bin/sensors.sh
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:48:?? docs/harness/WHAT_WE_DONT_DO.md
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:50:?? docs/harness/bin/baseline.sh
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:51:?? docs/harness/bin/quarterly-audit.sh
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:61:rg -n 'WHAT_WE_DONT_DO|CODE_REVIEW_POLICY|review-gate|doctor.sh|sensors.sh|baseline.sh|quarterly-audit' docs/harness README.md AGENTS.md Claude.md 2>/dev/null | head -160
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-pre.md:17:- docs/harness/CODE_REVIEW_POLICY.md (this policy)
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-pre.md:56:+- `docs/harness/CODE_REVIEW_POLICY.md`
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-pre.md:64:+bash docs/harness/bin/doctor.sh
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-pre.md:65:+bash docs/harness/bin/sensors.sh          # runs just ci + doctor (pode demorar)
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-pre.md:66:+bash docs/harness/bin/review-gate.sh pre harness-bootstrap
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-pre.md:81:+Then read in order: `docs/harness/SPEC.md`, `INVARIANTS.md`, `GATES.md`, `CODE_REVIEW_POLICY.md`, `progress.md`.
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-pre.md:88:+bash docs/harness/bin/doctor.sh
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-pre.md:89:+bash docs/harness/bin/sensors.sh   # full deterministic gates (wraps just ci)
docs/harness/GATES.md:5:1. **Sensores determinísticos** (`sensors.sh`) — locais, rápidos, reproduzíveis.
docs/harness/GATES.md:6:2. **Review gate cross-CLI/cross-model** (`review-gate.sh`) — julgamento independente.
docs/harness/GATES.md:11:Wrapper principal: `bash docs/harness/bin/sensors.sh`
docs/harness/GATES.md:21:| 5 | harness doctor | `bash docs/harness/bin/doctor.sh` | block; corrigir drift no harness |
docs/harness/GATES.md:24:O script `sensors.sh` grava o resultado parseável em `docs/harness/.sensors-last` (status, timestamp, exclusões, etc.).
docs/harness/GATES.md:39:`sensors.sh` bloqueia exclusão se a evidência de registro não existir.
docs/harness/GATES.md:44:bash docs/harness/bin/sensors.sh \
docs/harness/GATES.md:54:O review-gate é prompted explicitamente para caçar estes (sensores verdes mas sistema quebrado em produção ou para agentes):
docs/harness/GATES.md:67:O prompt do review-gate inclui esta lista + instrução para buscar evidência concreta no diff.
docs/harness/GATES.md:71:`docs/harness/WHAT_WE_DONT_DO.md` define escopo negativo para mudanças de harness.
docs/harness/GATES.md:73:O review-gate deve marcar como `[HIGH]` ou `[BLOCKER]` qualquer mudança que:
docs/harness/GATES.md:98:`bash docs/harness/bin/sensors.sh` sem argumentos continua sendo o full canonical gate.
docs/harness/GATES.md:106:- `baseline` — `baseline.sh` e doctor.
docs/harness/GATES.md:112:`baseline.sh` grava fatos estáticos baratos em `docs/harness/.baseline-last`.
docs/harness/GATES.md:114:Ele é evidência para drift review, não substitui `sensors.sh`, `make ci`, `just ci` ou review independente.
docs/harness/GATES.md:118:`quarterly-audit.sh` grava relatórios em `docs/harness/audits/` e atualiza `docs/harness/.quarterly-audit-last`.
docs/harness/GATES.md:145:  evidência de regressão apropriada e `review-gate.sh post` antes de upstream.
docs/harness/GATES.md:149:Ver `review-gate.sh` e `CODE_REVIEW_POLICY.md` para detalhes de execução e prompt.
docs/harness/GATES.md:169:FAIL 2 blockers: missing doctor integration, prompt drift in review-gate
docs/harness/GATES.md:192:- [ ] Para mudanças de processo do harness: `doctor.sh` passou antes e depois.
docs/harness/GATES.md:197:Pode pular o review-gate (camada 2) **somente** quando o diff inteiro for:
docs/harness/GATES.md:199:1. **Docs-only** em `docs/**/*.md`, exceto qualquer arquivo dentro de `docs/harness/` (INVARIANTS, GATES, CODE_REVIEW_POLICY, README, SPEC, scripts em bin/).
docs/harness/GATES.md:215:Em dúvida: rode o review-gate.
docs/harness/progress.md:7:| Active task | `harness-bootstrap — implement operational harness (bootstrap, doctor, sensors, review-gate)` |
docs/harness/progress.md:38:- [x] `CODE_REVIEW_POLICY.md` — política injetada no reviewer externo, com adaptações para dual-CLI e domínios de engram
docs/harness/progress.md:41:- [x] `bin/doctor.sh` — consistência do harness
docs/harness/progress.md:42:- [x] `bin/sensors.sh` — wrapper sobre `just ci` + doctor + engram-specific
docs/harness/progress.md:43:- [x] `bin/review-gate.sh` — generalizado para claude/grok/etc, com prompt engineering, continuity, versioning, timeout
docs/harness/progress.md:53:  - Exigir `sensors.sh --exclude-sensor grpc-transport --known-issue docs/harness/known-issues/2026-05-31-grpc-transport-port-bind.md --reason \"sandbox socket bind restriction\"`.
docs/harness/progress.md:90:  - `bash docs/harness/bin/doctor.sh` — PASS.
docs/harness/progress.md:91:  - `bash docs/harness/bin/sensors.sh` — PASS limpo, sem exclusão
docs/harness/progress.md:96:  - `bash -n docs/harness/bin/review-gate.sh` — PASS.
docs/harness/progress.md:102:- `docs/harness/bin/review-gate.sh` agora usa `set -euo pipefail` e corrige a
docs/harness/progress.md:117:- `bash docs/harness/bin/review-gate.sh post harness-bootstrap --review-file docs/harness/reviews/2026-05-31-harness-bootstrap-v9-post.md`
docs/harness/progress.md:133:  - `bash docs/harness/bin/doctor.sh` — PASS.
docs/harness/progress.md:148:  - `bash docs/harness/bin/doctor.sh` — PASS.
docs/harness/progress.md:191:  - `bash docs/harness/bin/doctor.sh` — PASS.
docs/harness/progress.md:220:  - `bash docs/harness/bin/doctor.sh` — PASS.
docs/harness/progress.md:252:  - `bash docs/harness/bin/doctor.sh` — PASS.
docs/harness/progress.md:291:  - `bash docs/harness/bin/doctor.sh` — PASS.
docs/harness/progress.md:293:  - `bash docs/harness/bin/review-gate.sh post eng-1241-mcp-http-auth-docs`
docs/harness/progress.md:334:- Criado `docs/harness/WHAT_WE_DONT_DO.md` para escopo negativo do harness.
docs/harness/progress.md:336:- Criados `docs/harness/bin/baseline.sh` e `docs/harness/bin/quarterly-audit.sh` como evidência de drift/auditoria, sem substituir sensores ou CI.
docs/harness/progress.md:337:- `sensors.sh` ganhou modos opcionais `full`, `quick`, `docs`, `mcp` e `baseline`; o modo sem argumentos permanece o gate completo canônico.
docs/harness/progress.md:338:- `review-gate.sh` passou a incluir `WHAT_WE_DONT_DO.md`, Review Canvas e guard para mudanças em `docs/harness/bin/*`.
docs/harness/progress.md:339:- `doctor.sh` agora valida a nova política, canvas, baseline, audit, sensor lanes e referências cruzadas.
docs/harness/progress.md:343:- `bash docs/harness/bin/doctor.sh`.
docs/harness/progress.md:344:- `bash docs/harness/bin/sensors.sh baseline`.
docs/harness/progress.md:345:- `bash -n docs/harness/bin/bootstrap.sh docs/harness/bin/doctor.sh docs/harness/bin/sensors.sh docs/harness/bin/review-gate.sh docs/harness/bin/baseline.sh docs/harness/bin/quarterly-audit.sh`.
docs/harness/reviews/2026-05-31-harness-bootstrap-v8-post.md.raw:17:- docs/harness/CODE_REVIEW_POLICY.md (this policy)
docs/harness/reviews/2026-05-31-harness-bootstrap-v8-post.md.raw:56:+- `docs/harness/CODE_REVIEW_POLICY.md`
docs/harness/reviews/2026-05-31-harness-bootstrap-v8-post.md.raw:64:+bash docs/harness/bin/doctor.sh
docs/harness/reviews/2026-05-31-harness-bootstrap-v8-post.md.raw:65:+bash docs/harness/bin/sensors.sh          # runs just ci + doctor (pode demorar)
docs/harness/reviews/2026-05-31-harness-bootstrap-v8-post.md.raw:66:+bash docs/harness/bin/review-gate.sh pre harness-bootstrap
docs/harness/reviews/2026-05-31-harness-bootstrap-v8-post.md.raw:81:+Then read in order: `docs/harness/SPEC.md`, `INVARIANTS.md`, `GATES.md`, `CODE_REVIEW_POLICY.md`, `progress.md`.
docs/harness/reviews/2026-05-31-harness-bootstrap-v8-post.md.raw:88:+bash docs/harness/bin/doctor.sh
docs/harness/reviews/2026-05-31-harness-bootstrap-v8-post.md.raw:89:+bash docs/harness/bin/sensors.sh   # full deterministic gates (wraps just ci)
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-post.md.raw:17:- docs/harness/CODE_REVIEW_POLICY.md (this policy)
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-post.md.raw:56:+- `docs/harness/CODE_REVIEW_POLICY.md`
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-post.md.raw:64:+bash docs/harness/bin/doctor.sh
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-post.md.raw:65:+bash docs/harness/bin/sensors.sh          # runs just ci + doctor (pode demorar)
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-post.md.raw:66:+bash docs/harness/bin/review-gate.sh pre harness-bootstrap
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-post.md.raw:81:+Then read in order: `docs/harness/SPEC.md`, `INVARIANTS.md`, `GATES.md`, `CODE_REVIEW_POLICY.md`, `progress.md`.
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-post.md.raw:88:+bash docs/harness/bin/doctor.sh
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-post.md.raw:89:+bash docs/harness/bin/sensors.sh   # full deterministic gates (wraps just ci)
docs/harness/CODE_REVIEW_POLICY.md:3:> Política consumida por `review-gate.sh` quando invoca um reviewer externo (Claude Code, Grok Build, Codex, Ollama, etc.).
docs/harness/CODE_REVIEW_POLICY.md:19:Antes de julgar, leia (o prompt do review-gate injeta ou referencia):
docs/harness/CODE_REVIEW_POLICY.md:25:5. `docs/harness/CODE_REVIEW_POLICY.md` (este arquivo)
docs/harness/CODE_REVIEW_POLICY.md:56:- A linha `REVIEW_VERDICT:` é obrigatória para fechamento de `review-gate.sh post`.
docs/harness/CODE_REVIEW_POLICY.md:76:O review-gate injeta checagens específicas para engram:
docs/harness/CODE_REVIEW_POLICY.md:83:- Se mudança em `docs/harness/**` (especialmente bin/ ou INVARIANTS/GATES/POLICY) → exigir doctor.sh verde + post-gate anterior.
docs/harness/CODE_REVIEW_POLICY.md:142:Read `docs/harness/WHAT_WE_DONT_DO.md`.
docs/harness/README.md:53:bash docs/harness/bin/doctor.sh
docs/harness/README.md:56:`doctor.sh` é read-only e valida a consistência interna do harness (drift entre SPEC/progress, scripts executáveis, referências à policy local, etc.). Ele **bloqueia** mudanças no harness quando há inconsistência.
docs/harness/README.md:65:| `WHAT_WE_DONT_DO.md`           | Escopo negativo e anti-patterns para evitar expansão silenciosa |
docs/harness/README.md:67:| `CODE_REVIEW_POLICY.md`        | Política local consumida pelo review-gate (cross-model / cross-CLI) |
docs/harness/README.md:76:| `bin/doctor.sh`                | Consistência read-only do harness |
docs/harness/README.md:77:| `bin/sensors.sh`               | Gate determinístico principal (wrapping `just ci` ou `make ci` + harness checks) |
docs/harness/README.md:78:| `bin/review-gate.sh`           | Gate de review cross-CLI / cross-model (generalizado) |
docs/harness/README.md:79:| `bin/baseline.sh`              | Snapshot estático barato em `.baseline-last` para drift review |
docs/harness/README.md:80:| `bin/quarterly-audit.sh`       | Auditoria evidence-only; nunca apaga, arquiva ou reescreve |
docs/harness/README.md:89:3. `docs/harness/WHAT_WE_DONT_DO.md` — escopo negativo e anti-patterns
docs/harness/README.md:91:5. `docs/harness/CODE_REVIEW_POLICY.md` — política de julgamento para o reviewer externo
docs/harness/README.md:107:bash docs/harness/bin/review-gate.sh pre <task-id>
docs/harness/README.md:114:bash docs/harness/bin/sensors.sh
docs/harness/README.md:117:bash docs/harness/bin/review-gate.sh post <task-id>
docs/harness/README.md:157:`sensors.sh` é o gate local principal. Ele invoca:
docs/harness/README.md:169:`bash docs/harness/bin/sensors.sh` sem argumentos continua sendo o gate completo canônico.
docs/harness/README.md:173:- `full` — equivalente ao default: CI local completo + `doctor.sh`.
docs/harness/README.md:174:- `quick` — `cargo fmt --all -- --check`, `cargo check` e `doctor.sh`.
docs/harness/README.md:175:- `docs` — referência MCP gerada, rustdoc com warnings como erro e `doctor.sh`.
docs/harness/README.md:176:- `mcp` — referência MCP gerada, testes de protocolo MCP e `doctor.sh`.
docs/harness/README.md:177:- `baseline` — `baseline.sh` + `doctor.sh`.
docs/harness/README.md:181:`baseline.sh` grava fatos estáticos baratos em `docs/harness/.baseline-last`. Ele ajuda revisão de drift, mas não prova correção.
docs/harness/README.md:183:`quarterly-audit.sh` grava relatórios evidence-only em `docs/harness/audits/` e atualiza `docs/harness/.quarterly-audit-last`. Ele nunca é um gate pass/fail e não deve apagar, arquivar ou reescrever arquivos.
docs/harness/README.md:187:`review-gate.sh` implementa o princípio de **Single-Process Judgment**:
docs/harness/README.md:200:- Monta prompt rico com: SPEC, INVARIANTS, WHAT_WE_DONT_DO, GATES, CODE_REVIEW_POLICY, fake-success patterns específicos de Rust/engram/MCP, diff (excluindo artefatos do próprio harness para evitar loops).
docs/harness/README.md:230:  ADR, sandbox forte, target contract e review-gate.
docs/harness/README.md:241:- [ ] `just ci` / `sensors.sh` passou limpo (ou exclusão documentada válida)
docs/harness/README.md:254:- **Harness em si**: Mudança em `docs/harness/bin/`, INVARIANTS, GATES ou CODE_REVIEW_POLICY exige rodar doctor + sensors + post-gate.
docs/harness/README.md:263:- Rode `review-gate.sh post ...` — ele prepara o prompt completo.
docs/harness/INVARIANTS.md:10:   _Gate: revisão manual + codex/review-gate prompt checa menção ao bootstrap; doctor.sh valida que bootstrap roda rápido e sem erro._
docs/harness/INVARIANTS.md:12:2. **Ordem de leitura obrigatória.** Antes de qualquer mudança significativa: SPEC.md → INVARIANTS.md (harness) → WHAT_WE_DONT_DO.md → GATES.md → CODE_REVIEW_POLICY.md → progress.md → active plan.
docs/harness/INVARIANTS.md:13:   _Gate: review-gate prompt injeta a ordem; findings por violação._
docs/harness/INVARIANTS.md:16:   _Gate: `doctor.sh` (hard fail)._
docs/harness/INVARIANTS.md:19:   _Gate: `doctor.sh`._
docs/harness/INVARIANTS.md:26:6. **Todo commit de feature atualiza memória canônica.** Commits que alteram comportamento de domínio, MCP surface, hooks, ou processo do harness **devem** atualizar `progress.md` + log da sprint ativa. Commit sem atualização de progresso é FAIL no review-gate.
docs/harness/INVARIANTS.md:27:   _Gate: review-gate (prompt checa diff por atualização de progress docs)._
docs/harness/INVARIANTS.md:30:   _Gate: revisão manual + review-gate._
docs/harness/INVARIANTS.md:35:   _Gate: `review-gate.sh post` (hard gate)._
docs/harness/INVARIANTS.md:37:9. **2 FAILs consecutivos no mesmo task → escalar humano.** Não iterar infinitamente no review-gate post.
docs/harness/INVARIANTS.md:38:   _Gate: `review-gate.sh` + processo humano._
exit_status=0
```

### Schema and migration references

```bash
rg -n 'SCHEMA_VERSION|migration|migrations' src/storage tests docs 2>/dev/null | head -160
```

```text
src/storage/connection.rs:11:use super::migrations::run_migrations;
src/storage/connection.rs:33:        // Run migrations
src/storage/connection.rs:34:        run_migrations(&conn)?;
src/storage/connection.rs:390:        // Run migrations on first connection
src/storage/connection.rs:393:            run_migrations(&conn)?;
src/storage/migrations.rs:1://! Database migrations for Engram
src/storage/migrations.rs:8:pub const SCHEMA_VERSION: i32 = 40;
src/storage/migrations.rs:10:/// Run all migrations
src/storage/migrations.rs:11:pub fn run_migrations(conn: &Connection) -> Result<()> {
src/storage/migrations.rs:12:    // Create migrations table if not exists
src/storage/migrations.rs:29:    if current_version > SCHEMA_VERSION {
src/storage/migrations.rs:32:            current_version, SCHEMA_VERSION
src/storage/migrations.rs:372:        -- Record migration
src/storage/migrations.rs:380:/// Memory scoping migration (v2) - RML-924
src/storage/migrations.rs:398:        -- Record migration
src/storage/migrations.rs:406:/// Entity extraction migration (v3) - RML-925
src/storage/migrations.rs:447:        -- Record migration
src/storage/migrations.rs:467:        -- Record migration
src/storage/migrations.rs:475:/// Memory expiration (TTL) migration (v5) - RML-930
src/storage/migrations.rs:488:        -- Record migration
src/storage/migrations.rs:496:/// Memory deduplication migration (v6) - RML-931
src/storage/migrations.rs:513:        -- Record migration
src/storage/migrations.rs:547:/// Workspace and tier migration (v7) - RML-950
src/storage/migrations.rs:573:        -- Record migration
src/storage/migrations.rs:583:/// Session transcript indexing migration (v8)
src/storage/migrations.rs:631:        -- Record migration
src/storage/migrations.rs:641:/// Identity links migration (v9)
src/storage/migrations.rs:708:        -- Record migration
src/storage/migrations.rs:718:/// Events and sharing migration (v10)
src/storage/migrations.rs:785:        -- Record migration
src/storage/migrations.rs:797:/// This migration handles the case where v10 was applied before agent_id was added
src/storage/migrations.rs:828:    // Record migration
src/storage/migrations.rs:873:        -- Added now to avoid another migration for Phase 3
src/storage/migrations.rs:886:        -- Record migration
src/storage/migrations.rs:917:        -- Record migration
src/storage/migrations.rs:994:        -- Record migration
src/storage/migrations.rs:1018:        -- Note: content_hash may already exist from earlier migration, so we use IF NOT EXISTS pattern
src/storage/migrations.rs:1109:    // Record migration
src/storage/migrations.rs:2019:        run_migrations(&conn).expect("run migrations");
src/storage/migrations.rs:2038:        assert_eq!(SCHEMA_VERSION, 40);
src/storage/migrations.rs:2168:        // Simulate a v17 database by running only migrations up to v17
src/storage/migrations.rs:2181:        // Run all migrations (they'll stop at the current version)
src/storage/migrations.rs:2182:        // We simulate v17 state by running the full migration once,
src/storage/migrations.rs:2184:        run_migrations(&conn).expect("run migrations from scratch");
src/storage/migrations.rs:2193:        assert_eq!(version, 40, "should reach v40 after full migration");
src/storage/migrations.rs:2227:            "enrichment_events table should exist after migration"
src/storage/auto_linker.rs:504:    use crate::storage::migrations::run_migrations;
src/storage/auto_linker.rs:510:        run_migrations(&conn).expect("migrations");
src/storage/pending_injections.rs:136:    use crate::storage::migrations::run_migrations;
src/storage/pending_injections.rs:140:        run_migrations(&c).unwrap();
src/storage/turso_backend.rs:11://! - **Compatible schema**: Same migrations as SQLite backend
src/storage/turso_backend.rs:38:use crate::storage::migrations::SCHEMA_VERSION;
src/storage/turso_backend.rs:212:        // Apply migrations
src/storage/turso_backend.rs:213:        if version < SCHEMA_VERSION {
src/storage/turso_backend.rs:214:            self.apply_migration_v1(&conn).await?;
src/storage/turso_backend.rs:221:    /// Apply migration v1 - base schema
src/storage/turso_backend.rs:222:    async fn apply_migration_v1(&self, conn: &Connection) -> Result<()> {
src/storage/turso_backend.rs:388:        // Record migration
src/storage/turso_backend.rs:391:            libsql::params![SCHEMA_VERSION],
src/storage/mod.rs:37:mod migrations;
src/storage/enrichment_events.rs:76:    use crate::storage::migrations::run_migrations;
src/storage/enrichment_events.rs:81:        run_migrations(&conn).unwrap();
src/storage/enrichment_events.rs:169:        // No migrations — table doesn't exist
src/storage/memory_blocks.rs:23:/// Embed this in a migration when integrating with the main schema.
src/storage/image_storage.rs:80:/// Result of image migration
src/storage/image_storage.rs:874:        use crate::storage::migrations::run_migrations;
src/storage/image_storage.rs:877:        run_migrations(&conn).expect("migrations");
src/storage/image_storage.rs:893:        use crate::storage::migrations::run_migrations;
src/storage/image_storage.rs:896:        run_migrations(&conn).expect("migrations");
docs/superpowers/specs/2026-06-03-enrichment-event-log-design.md:84:- `event_type` has no CHECK constraint — new types should not require a migration.
docs/superpowers/specs/2026-06-03-enrichment-event-log-design.md:258:- [ ] `src/storage/migrations.rs` (migration v40)
docs/superpowers/specs/2026-06-03-enrichment-event-log-design.md:265:- [ ] `src/storage/migrations.rs` — update hardcoded `SCHEMA_VERSION` test to v40
src/storage/scope_grants.rs:203:    use crate::storage::migrations::run_migrations;
src/storage/scope_grants.rs:207:        run_migrations(&conn).expect("run migrations");
src/storage/agent_registry.rs:266:    use crate::storage::migrations::run_migrations;
src/storage/agent_registry.rs:270:        run_migrations(&conn).expect("run migrations");
src/storage/queries/tests.rs:1887:fn test_schema_migration_v34_idempotent() {
src/storage/queries/tests.rs:1888:    use crate::storage::migrations::run_migrations;
src/storage/queries/tests.rs:1890:    run_migrations(&conn).expect("run migrations");
src/storage/queries/tests.rs:1892:    run_migrations(&conn).expect("idempotent second run");
src/storage/queries/tests.rs:1900:    assert_eq!(version, crate::storage::migrations::SCHEMA_VERSION);
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:21:| Modify | `src/storage/migrations.rs` | Migration v40, update `SCHEMA_VERSION` to 40, update 3 hardcoded test assertions |
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:40:- Modify: `src/storage/migrations.rs`
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:45:Add inside `#[cfg(test)]` at the bottom of `src/storage/migrations.rs`:
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:58:    assert_eq!(exists, 1, "enrichment_events table should exist after migration");
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:81:- [ ] **Step 1.3: Update `SCHEMA_VERSION` constant**
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:83:In `src/storage/migrations.rs` line 8, change:
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:85:pub const SCHEMA_VERSION: i32 = 39;
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:89:pub const SCHEMA_VERSION: i32 = 40;
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:94:Add before the `#[cfg(test)]` block in `src/storage/migrations.rs`:
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:150:- [ ] **Step 1.5: Wire migrate_v40 into run_migrations**
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:152:Find the dispatch block in `run_migrations` (around line 178). The current last entry is:
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:154:    if current_version < SCHEMA_VERSION {
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:164:    if current_version < SCHEMA_VERSION {
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:171:Search and replace all occurrences of `assert_eq!(version, 39` → `assert_eq!(version, 40` and `assert_eq!(SCHEMA_VERSION, 39` → `assert_eq!(SCHEMA_VERSION, 40` in `src/storage/migrations.rs`. There are three: lines 1970, 1975, and 2130.
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:209:git add src/storage/migrations.rs docs/SCHEMA.md
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:210:git commit -m "feat(ENG-1240): migration v40 — add enrichment_events table"
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:269:    use crate::storage::migrations::run_migrations;
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:274:        run_migrations(&conn).unwrap();
src/storage/sqlite_backend.rs:67:/// database and trigger migrations or connection pragmas.
docs/MCP_TOOLS.md:3517:Export all memories to a JSON-serializable format for backup or migration.
docs/README.md:9:- `SCHEMA.md`: Local Engram SQLite schema and migrations.
docs/harness/INVARIANTS.md:65:17. **Schema version + testes de migração.** Alterar `storage/migrations.rs` exige atualizar `SCHEMA_VERSION` **e** todos os testes que têm versão hardcoded. Nunca silenciar mismatches.
docs/rfcs/0001-harness-memory-product-boundary.md:89:- documentation establishes a boundary, policy, invariant, or migration path.
docs/USING_ENGRAM_IN_A_REPO.md:224:engram-cli search "database migration gotchas"
docs/USING_ENGRAM_IN_A_REPO.md:230:> Search Engram for prior decisions about database migrations before changing the schema.
docs/USING_ENGRAM_IN_A_REPO.md:247:        "query": "database migration gotchas",
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:228:### Schema and migration references
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:231:rg -n 'SCHEMA_VERSION|migration|migrations' src/storage tests docs 2>/dev/null | head -160
docs/rfcs/0003-search-index-v2.md:58:- **Local-first fit:** Good, but with larger code and migration surface than FTS5.
docs/rfcs/0003-search-index-v2.md:59:- **Rebuild strategy:** Requires custom index lifecycle code (migration, repair, corruption handling).
docs/rfcs/0003-search-index-v2.md:125:   - keep migration docs explicit that local REST-like behavior is not altered.
docs/rfcs/0003-search-index-v2.md:138:- applies explicit guardrails and migration path from current behavior.
docs/harness/reviews/2026-05-31-harness-bootstrap-v2-pre.md:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-pre.md.raw:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/bin/quarterly-audit.sh:91:append_cmd "Schema and migration references" "rg -n 'SCHEMA_VERSION|migration|migrations' src/storage tests docs 2>/dev/null | head -160"
docs/harness/bin/baseline.sh:36:  echo "schema_version=$(rg -n 'SCHEMA_VERSION' src/storage 2>/dev/null | head -1 | sed 's/[[:space:]]\+/ /g')"
docs/SCHEMA.md:13:3. [Migration History](#migration-history)
docs/SCHEMA.md:15:5. [Migration Strategy](#migration-strategy)
docs/SCHEMA.md:1245:2. **Backward Compatible:** Old code continues to work during migration
docs/SCHEMA.md:1246:3. **Atomic:** Each migration is a single transaction
docs/SCHEMA.md:1252:pub fn run_migrations(conn: &Connection) -> Result<()> {
docs/SCHEMA.md:1254:    for version in (current_version + 1)..=SCHEMA_VERSION {
docs/SCHEMA.md:1271:All migrations are located in `src/storage/migrations.rs`.
docs/harness/reviews/2026-05-31-harness-bootstrap-v2-pre.md.raw:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/progress/2026-05-30-harness-bootstrap.md:35:   - Estrutura de MCP handlers, hooks, intelligence, storage/migrations, etc.
docs/harness/reviews/2026-05-31-harness-bootstrap-v7-post.md.raw:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-pre.md:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/bin/review-gate.sh:149:  echo "3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated."
docs/harness/reviews/2026-05-31-harness-bootstrap-v8-pre.md.raw:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/reviews/2026-05-31-harness-bootstrap-v6-pre.md:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/reviews/2026-05-31-harness-bootstrap-v10-post.md.raw:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/reviews/2026-05-31-harness-bootstrap-pre.md:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/reviews/2026-05-31-harness-bootstrap-pre.md.raw:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/reviews/2026-05-31-harness-bootstrap-v4-pre.md:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/reviews/2026-05-31-harness-bootstrap-v8-post.md.raw:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/plans/2026-06-05-engram-harness-improvement-execution-plan.md:194:- Changes to storage schema, migrations, or data invariants.
docs/harness/plans/2026-06-05-engram-harness-improvement-execution-plan.md:406:  echo "schema_version=$(rg -n 'SCHEMA_VERSION' src/storage 2>/dev/null | head -1 | sed 's/[[:space:]]\\+/ /g')"
docs/harness/plans/2026-06-05-engram-harness-improvement-execution-plan.md:687:append_cmd "Schema and migration references" "rg -n 'SCHEMA_VERSION|migration|migrations' src/storage tests docs 2>/dev/null | head -160"
docs/harness/plans/2026-06-05-cross-harness-improvement-plan.md:188:- Changes to storage schema, migrations, or data invariants.
docs/harness/plans/2026-06-05-cross-harness-improvement-plan.md:327:  echo "schema_version=$(rg -n 'SCHEMA_VERSION' src/storage 2>/dev/null | head -1 | sed 's/[[:space:]]\\+/ /g')"
docs/harness/plans/2026-06-05-cross-harness-improvement-plan.md:531:append_cmd "Schema and migration references" "rg -n 'SCHEMA_VERSION|migration|migrations' src/storage tests docs 2>/dev/null | head -160"
docs/harness/reviews/2026-05-31-harness-bootstrap-v5-pre.md.raw:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/reviews/2026-05-31-harness-bootstrap-v4-pre.md.raw:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/GATES.md:58:3. **Schema version atualizada mas testes de migração hardcoded falham** — `storage/migrations.rs` bump + alguns testes em `tests/` ou `src/storage/` ainda têm versão antiga.
docs/harness/GATES.md:87:- Storage schema, migrations ou invariants de dados.
docs/harness/GATES.md:188:- [ ] Se storage migration ou `SCHEMA_VERSION`: evidência de que testes de migração e integração rodam limpos.
docs/harness/reviews/2026-05-31-harness-bootstrap-v6-pre.md.raw:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/reviews/2026-05-31-harness-bootstrap-v8-pre.md:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/README.md:212:Use `docs/harness/canvas/TEMPLATE.md` quando a mudança envolver diffs grandes, storage/migrations, superfície MCP, hooks/intelligence/embeddings/sync/attestation, contratos SDK, dependências externas ou qualquer alteração em gates/scripts/policies do harness.
docs/harness/README.md:250:- **Schema / Migrations** (`storage/migrations.rs`): Atualizar `SCHEMA_VERSION`. Testes têm versão hardcoded em alguns lugares — atualizar também. Nunca quebrar monotonicidade de IDs.
docs/harness/reviews/2026-05-31-harness-bootstrap-v9-post.md.raw:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/CODE_REVIEW_POLICY.md:35:4. Mapeie linhas alteradas para a menor unidade significativa: função, módulo MCP handler, migration, tool definition, hook, embedding provider, test, doc de contrato.
docs/harness/CODE_REVIEW_POLICY.md:78:- Se `storage/migrations.rs` ou `SCHEMA_VERSION` mudou → exigir evidência de testes de migração e integração rodando limpos.
docs/harness/CODE_REVIEW_POLICY.md:94:- Gaps de documentação ou configuração quando o comportamento, API pública, migration, operação ou processo do harness mudou.
docs/harness/canvas/README.md:8:- Changes to storage schema, migrations, or data invariants.
docs/harness/reviews/2026-05-31-harness-bootstrap-v7-pre.md.raw:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/reviews/2026-05-31-harness-bootstrap-v5-pre.md:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-post.md.raw:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
docs/harness/reviews/2026-05-31-harness-bootstrap-v7-pre.md:27:3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
exit_status=0
```

### MCP reference count and manual count risks

```bash
rg -n 'MCP_TOOLS|[0-9]+\+? tools|tools exposed|Available MCP Tools' README.md docs src sdks 2>/dev/null | head -160
```

```text
README.md:378:### Available MCP Tools
README.md:380:The MCP tool reference is generated from source of truth (`src/mcp/tools/registry.rs`) and tracked in `docs/MCP_TOOLS.md`.
README.md:382:- Full reference: [docs/MCP_TOOLS.md](docs/MCP_TOOLS.md)
docs/superpowers/specs/2026-06-03-enrichment-event-log-design.md:263:- [ ] `docs/MCP_TOOLS.md` via `./scripts/generate-mcp-reference.sh` (canonical harness wrapper)
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:1500:Expected: `wrote docs/MCP_TOOLS.md`
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:1512:git add tests/mcp_protocol_tests.rs docs/MCP_TOOLS.md
docs/superpowers/plans/2026-06-03-enrichment-event-log.md:1513:git commit -m "feat(ENG-1240): protocol tests for enrichment_audit tools + regenerate MCP_TOOLS.md"
docs/harness/reviews/2026-05-31-harness-bootstrap-v2-pre.md:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-v10-post.md.raw:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-v4-pre.md.raw:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-v8-post.md.raw:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-v4-pre.md:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-v6-pre.md.raw:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-v8-pre.md:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-pre.md.raw:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-v7-post.md.raw:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-v6-pre.md:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-pre.md.raw:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-v9-post.md.raw:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-pre.md:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-pre.md:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-v8-pre.md.raw:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-v5-pre.md.raw:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/reviews/2026-05-31-harness-bootstrap-v2-pre.md.raw:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:335:docs/MCP_TOOLS.md:3517:Export all memories to a JSON-serializable format for backup or migration.
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:401:rg -n 'MCP_TOOLS|[0-9]+\+? tools|tools exposed|Available MCP Tools' README.md docs src sdks 2>/dev/null | head -160
docs/GETTING_STARTED.md:103:Once configured, your AI tool will have access to the MCP tools listed in [`docs/MCP_TOOLS.md`](docs/MCP_TOOLS.md). Ask it to run `memory_stats` to verify the connection is working.
docs/harness/decisions/phase1-1-issue-snapshot-2026-05-31.md:29:| #27 | P0 | **Parcialmente entregue** (`docs/MCP_TOOLS.md` gerado por script + check no CI) | Confirmar no começo da 1.3 e encadear com issue doc |
docs/harness/reviews/2026-05-31-harness-bootstrap-v5-pre.md:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/plans/2026-06-05-cross-harness-improvement-plan.md:328:  echo "mcp_reference_tools=$(rg -c '^## ' docs/MCP_TOOLS.md 2>/dev/null || echo 0)"
docs/harness/plans/2026-06-05-cross-harness-improvement-plan.md:532:append_cmd "MCP reference count and manual count risks" "rg -n 'MCP_TOOLS|[0-9]+\\+? tools|tools exposed|Available MCP Tools' README.md docs src sdks 2>/dev/null | head -160"
docs/harness/bin/review-gate.sh:148:  echo "2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes."
docs/harness/reviews/2026-05-31-harness-bootstrap-v6-post.md.raw:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/plans/open-issues-audit-2026-05-31.md:10:- `./scripts/generate-mcp-reference.sh --check` → `docs/MCP_TOOLS.md is up to date`
docs/harness/plans/open-issues-audit-2026-05-31.md:11:- `git log --oneline -n 8 -- docs/MCP_TOOLS.md` → HEAD relacionado a `#38` no contexto de reference generator
docs/harness/reviews/2026-05-31-harness-bootstrap-v3-post.md.raw:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/progress.md:196:  - `README.md` (`Available MCP Tools`) agora aponta para `docs/MCP_TOOLS.md` e para o gerador.
docs/harness/progress.md:197:  - `docs/AI_GUIDE.md` remove contagens manuais e passa a referenciar `docs/MCP_TOOLS.md` como origem.
docs/harness/plans/2026-06-05-engram-harness-improvement-execution-plan.md:407:  echo "mcp_reference_sections=$(rg -c '^## ' docs/MCP_TOOLS.md 2>/dev/null || echo 0)"
docs/harness/plans/2026-06-05-engram-harness-improvement-execution-plan.md:688:append_cmd "MCP reference count and manual count risks" "rg -n 'MCP_TOOLS|[0-9]+\\+? tools|tools exposed|Available MCP Tools' README.md docs src sdks 2>/dev/null | head -160"
docs/harness/reviews/2026-05-31-harness-bootstrap-v7-pre.md.raw:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/INVARIANTS.md:68:18. **MCP surface changes são breaking por default.** Adicionar/remover/renomear tools expostas exige atualização de `scripts/generate-mcp-reference.sh`, docs/MCP_TOOLS.md, testes de protocolo, e SDKs (quando breaking). Coordenação visível.
docs/harness/bin/baseline.sh:37:  echo "mcp_reference_sections=$(grep -c '^### ' docs/MCP_TOOLS.md 2>/dev/null || echo 0)"
docs/harness/README.md:251:- **MCP Tools**: Mudança na superfície de 155+ tools exige atualização de `scripts/generate-mcp-reference.sh` output, docs/MCP_TOOLS.md, e testes de protocolo. Coordenação com SDKs Python/TS.
docs/harness/README.md:286:- `docs/MCP_TOOLS.md` — superfície completa de tools expostas via MCP
docs/harness/reviews/2026-05-31-harness-bootstrap-v7-pre.md:26:2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
docs/harness/bin/bootstrap.sh:113:if [ -f docs/MCP_TOOLS.md ]; then
docs/harness/bin/bootstrap.sh:114:  MCP_COUNT="$(grep -c '^### ' docs/MCP_TOOLS.md 2>/dev/null || echo '?')"
docs/AI_GUIDE.md:3:> **Version:** 0.20.0 | **Protocol:** MCP 2025-11-25 | **Tools:** [generated reference](docs/MCP_TOOLS.md) | **Schema:** v34
docs/AI_GUIDE.md:1918:*MCP tool reference: [docs/MCP_TOOLS.md](docs/MCP_TOOLS.md) (generated) | Hybrid search | Knowledge graphs | Cloud sync | Multimodal | Agent portability | Progressive discovery*
docs/harness/bin/quarterly-audit.sh:92:append_cmd "MCP reference count and manual count risks" "rg -n 'MCP_TOOLS|[0-9]+\\+? tools|tools exposed|Available MCP Tools' README.md docs src sdks 2>/dev/null | head -160"
src/mcp/handlers/agent.rs:3://! Provides 6 tools for managing registered AI agents:
docs/harness/progress/2026-05-30-harness-bootstrap.md:145:   - `./scripts/generate-mcp-reference.sh --check` (PASS, MCP_TOOLS atualizado)
docs/harness/progress/2026-05-30-harness-bootstrap.md:510:  `docs/MCP_TOOLS.md`.
exit_status=0
```

### Temporary, legacy, and cleanup markers

```bash
rg -n -i 'temporary|legacy|compat|deprecated|TODO: remove|remove after|sunset|hack|workaround' src tests docs sdks scripts 2>/dev/null | head -180
```

```text
docs/MCP_AUTH.md:6:- `POST /v1/mcp` (compatibility alias)
docs/superpowers/specs/2026-05-11-onnx-local-embeddings-design.md:44:**Razão:** mantém runtime trivial (só lê disco), torna network call explícito (auditável), funciona offline após download inicial, compatível com Docker (`RUN engram-cli model download ...`).
tests/watcher_integration.rs:5://! temporary directories.
src/sync/cloud.rs:21:    /// Create from S3-compatible URI (s3://bucket/path/to/file.db)
docs/ROADMAP.md:132:- **MCP Protocol Upgrade**: v2024-11-05 → v2025-11-25 with full backward compatibility
tests/mcp_protocol_tests.rs:24:    MCP_PROTOCOL_VERSION, MCP_PROTOCOL_VERSION_LEGACY,
tests/mcp_protocol_tests.rs:74:                let result = if client_version == MCP_PROTOCOL_VERSION_LEGACY {
tests/mcp_protocol_tests.rs:76:                        protocol_version: MCP_PROTOCOL_VERSION_LEGACY.to_string(),
tests/mcp_protocol_tests.rs:352:fn test_protocol_negotiation_2024_backward_compat() {
tests/mcp_protocol_tests.rs:359:            "clientInfo": {"name": "legacy-client", "version": "0.1.0"}
tests/mcp_protocol_tests.rs:375:        "Protocol version should be 2024-11-05 for legacy client"
tests/mcp_protocol_tests.rs:378:    // Legacy mode: resources and prompts capabilities should be absent
tests/mcp_protocol_tests.rs:386:        "Should NOT have resources capability in legacy mode"
tests/mcp_protocol_tests.rs:390:        "Should NOT have prompts capability in legacy mode"
src/attestation/merkle.rs:149:/// Naive `left || right` concatenation. Retained for backwards-compatible
src/attestation/merkle.rs:288:    fn test_v1_proof_backwards_compat() {
src/attestation/types.rs:54:    /// backwards-compatible verification). New proofs always use 2.
sdks/python/engram_client/integrations/langchain.py:4:- EngramChatMessageHistory: BaseChatMessageHistory-compatible class backed by Engram.
sdks/python/engram_client/integrations/langchain.py:5:- EngramVectorStore: VectorStore-compatible class backed by Engram's hybrid search.
sdks/python/engram_client/integrations/langchain.py:146:            **kwargs: Ignored (accepted for interface compatibility).
sdks/python/engram_client/integrations/langchain.py:175:            **kwargs: Ignored (accepted for interface compatibility).
sdks/python/engram_client/integrations/llamaindex.py:5:- EngramDocumentStore: BaseDocumentStore-compatible — stores and retrieves
sdks/python/engram_client/integrations/llamaindex.py:7:- EngramLlamaIndexVectorStore: BasePydanticVectorStore-compatible — stores
sdks/python/engram_client/integrations/llamaindex.py:10:- EngramChatStore: BaseChatStore-compatible — persists chat messages as
sdks/python/engram_client/integrations/llamaindex.py:94:            docs: List of LlamaIndex-compatible node objects.
sdks/python/engram_client/integrations/llamaindex.py:285:            nodes: List of LlamaIndex-compatible node objects.
sdks/python/engram_client/integrations/llamaindex.py:286:            **kwargs: Ignored (accepted for interface compatibility).
sdks/python/engram_client/integrations/llamaindex.py:311:            **kwargs: Ignored (accepted for interface compatibility).
sdks/python/engram_client/integrations/llamaindex.py:376:            **kwargs: Ignored (accepted for interface compatibility).
src/embedding/mod.rs:71:/// Supports OpenAI, OpenRouter, Azure OpenAI, and other OpenAI-compatible APIs.
src/embedding/mod.rs:134:    /// Legacy constructor for backwards compatibility
src/embedding/mod.rs:146:    /// Async embedding call to OpenAI-compatible API
src/embedding/mod.rs:299:/// For OpenAI-compatible APIs (OpenRouter, Azure, etc.), set:
sdks/python/engram_client/integrations/crewai.py:44:    """CrewAI-compatible short-term memory backed by Engram daily memories.
sdks/python/engram_client/integrations/crewai.py:116:    """CrewAI-compatible long-term memory backed by permanent Engram memories.
sdks/python/engram_client/integrations/crewai.py:169:    """CrewAI-compatible entity memory backed by Engram's identity system.
scripts/test_generate_mcp_reference.py:50:        with tempfile.TemporaryDirectory() as directory:
src/hooks/stop.rs:20:        // - Cleanup temporary resources
docs/AI_GUIDE.md:65:a compatibility alias). Include `Authorization: Bearer sk_my_secret` when
docs/AI_GUIDE.md:140:| `session` | One conversation only | Temporary context |
docs/AI_GUIDE.md:738:Engram can sync its SQLite database to S3-compatible cloud storage (AWS S3, Cloudflare R2, MinIO).
docs/AI_GUIDE.md:1314:- Endpoint: `POST /mcp` (`POST /v1/mcp` compatibility alias)
docs/AI_GUIDE.md:1648:# All tools (default, backward compatible)
src/types.rs:386:/// - `Session`: Memories are temporary and bound to a conversation session
src/types.rs:394:    /// Session-scoped memory, temporary for one conversation
src/types.rs:398:    /// Global scope, accessible by all (default for backward compatibility)
src/types.rs:813:    /// OpenAI-compatible API base URL (for OpenRouter, Azure, etc.)
src/types.rs:953:    /// Legacy metadata filter (simple key-value equality)
src/types.rs:954:    /// Deprecated: Use `filter` for advanced queries
docs/MCP_TOOLS.md:285:| `type` | `string` | no | Deprecated alias for memory_type Default: `note`. Allowed: `note`, `todo`, `issue`, `decision`, `preference`, `learning`, `context`, `credential`, `episodic`, `procedural`, `summary`, `checkpoint`, `image`, `audio`, `video`. |
docs/MCP_TOOLS.md:320:Deprecated alias for context_seed. Use context_seed instead.
docs/MCP_TOOLS.md:361:| `type` | `string` | no | Deprecated alias for memory_type Allowed: `note`, `todo`, `issue`, `decision`, `preference`, `learning`, `context`, `credential`, `episodic`, `procedural`, `summary`, `checkpoint`, `image`, `audio`, `video`. |
docs/MCP_TOOLS.md:397:| `type` | `string` | no | Deprecated alias for memory_type |
docs/MCP_TOOLS.md:404:| `metadata_filter` | `object` | no | Legacy simple key-value filter (deprecated, use 'filter' instead) |
docs/MCP_TOOLS.md:1732:| `type` | `string` | no | Deprecated alias for memory_type |
docs/MCP_TOOLS.md:3496:| `type` | `string` | no | Deprecated alias for memory_type |
docs/harness/bin/sensors.sh:107:      error_pattern='connection refused|connection reset|timeout|5[0-9]{2}|service unavailable|operation timed out|network|temporary failure'
docs/harness/bin/sensors.sh:115:      error_pattern='provider|api|timeout|connection|5[0-9]{2}|unavailable|transport|rate limit|temporary failure'
src/multimodal/video.rs:235:    /// Frames are saved as PNG files (`frame_001.png`, …) inside a temporary
docs/SCHEMA.md:47:2. **Nullable by Default:** New columns allow NULL for backward compatibility
docs/SCHEMA.md:1245:2. **Backward Compatible:** Old code continues to work during migration
src/bench/longmemeval.rs:408:        // Clean up temporary file
docs/harness/CODE_REVIEW_POLICY.md:13:Priorize: corretude, regressões, segurança, perda de dados, builds quebrados, quebras de compatibilidade (especialmente MCP protocol e SDK contracts), coverage de comportamentos alterados pelo diff, e violação de invariants.
docs/harness/CODE_REVIEW_POLICY.md:46:- **Sinal de risco**: compile failure, wrong result, runtime failure/panic, security issue, violação de invariant escopado, missing regression test para comportamento alterado, ou gap de compatibilidade/documentação.
docs/harness/CODE_REVIEW_POLICY.md:92:- Bugs, vulnerabilidades, data loss, broken builds, regressões de performance significativas, quebras de compatibilidade (MCP, SDK, storage), e testes ausentes para o comportamento alterado.
docs/harness/progress/2026-05-30-harness-bootstrap.md:244:   - Observado tempo estável em outputs de `OutputFilter`, `TruncationEngine` e pipeline completa (`OutputFilter -> TruncationEngine`), com micro-latências compatíveis ao uso local.
docs/harness/progress/2026-05-30-harness-bootstrap.md:466:  - `None` mantém compatibilidade com os callers atuais.
docs/harness/progress/2026-05-30-harness-bootstrap.md:563:- Adicionado alias `POST /v1/mcp` para `POST /mcp`, mantendo compatibilidade
src/multimodal/vision.rs:190:/// Vision provider backed by OpenAI GPT-4o (or compatible) API
docs/harness/bin/quarterly-audit.sh:93:append_cmd "Temporary, legacy, and cleanup markers" "rg -n -i 'temporary|legacy|compat|deprecated|TODO: remove|remove after|sunset|hack|workaround' src tests docs sdks scripts 2>/dev/null | head -180"
src/intelligence/auto_tagging.rs:280:        map.insert("hack", "action/hack");
src/bench/locomo.rs:227:        // Clean up temporary database file if not in-memory
docs/USING_ENGRAM_IN_A_REPO.md:266:compatibility alias). Create a memory:
docs/harness/known-issues/README.md:3:This directory holds short, dated documents that justify temporary exclusion of a specific sensor in `sensors.sh`.
docs/GETTING_STARTED.md:59:Engram speaks the [Model Context Protocol](https://modelcontextprotocol.io/) (MCP), so it integrates with Claude Code, Cursor, VS Code MCP clients (like Cline/Roo Code), and other MCP-compatible tools.
docs/GETTING_STARTED.md:244:Sync your memories to S3-compatible storage (AWS S3, Cloudflare R2, MinIO):
docs/harness/INVARIANTS.md:63:    _Gate: `sensors.sh` invoca o ci target + review-gate checa por hacks de CI._
docs/harness/progress.md:275:  aceita `POST /v1/mcp` como alias compativel de `POST /mcp`, com o mesmo
docs/harness/progress.md:305:  segunda pre-imagem. Backwards compat mantida via `scheme_version: u8`
src/search/neural_rerank.rs:104:/// Uses an ms-marco-MiniLM-L-6-v2 compatible model to score each
src/bench/membench.rs:296:        // Clean up temporary file
docs/harness/decisions/phase1-2-plan-source-unification-2026-05-31.md:11:- A seção “Existing Similar Artifact” já está incorporada no plano principal como nota de compatibilidade.
docs/harness/plans/2026-06-05-cross-harness-improvement-plan.md:533:append_cmd "Temporary, legacy, and cleanup markers" "rg -n -i 'temporary|legacy|compat|deprecated|TODO: remove|remove after|sunset|hack|workaround' src tests docs sdks scripts 2>/dev/null | head -180"
docs/harness/plans/2026-06-05-cross-harness-improvement-plan.md:843:- [ ] **Step 1: Keep `codex-gate.sh` as compatibility wrapper**
docs/harness/plans/2026-06-05-cross-harness-improvement-plan.md:899:Add flags compatible with Engram:
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:285:src/storage/turso_backend.rs:11://! - **Compatible schema**: Same migrations as SQLite backend
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:354:docs/SCHEMA.md:1245:2. **Backward Compatible:** Old code continues to work during migration
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:462:### Temporary, legacy, and cleanup markers
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:465:rg -n -i 'temporary|legacy|compat|deprecated|TODO: remove|remove after|sunset|hack|workaround' src tests docs sdks scripts 2>/dev/null | head -180
src/search/hybrid.rs:173:    // Advanced filter (RML-932) - takes precedence over legacy tags/memory_type
src/search/hybrid.rs:184:        // Legacy filters (deprecated, use `filter` instead)
docs/harness/plans/2026-06-05-engram-harness-improvement-execution-plan.md:689:append_cmd "Temporary, legacy, and cleanup markers" "rg -n -i 'temporary|legacy|compat|deprecated|TODO: remove|remove after|sunset|hack|workaround' src tests docs sdks scripts 2>/dev/null | head -180"
src/bin/server.rs:18:    ToolsCapability, MCP_PROTOCOL_VERSION, MCP_PROTOCOL_VERSION_LEGACY,
src/bin/server.rs:77:    /// OpenAI-compatible API base URL (for OpenRouter, Azure, etc.)
src/bin/server.rs:349:                // Negotiate protocol version: if the client requests the legacy version, respond
src/bin/server.rs:357:                let result = if client_version == MCP_PROTOCOL_VERSION_LEGACY {
src/bin/server.rs:358:                    // Legacy mode: respond with 2024-11-05, no resources/prompts capabilities
src/bin/server.rs:360:                        protocol_version: MCP_PROTOCOL_VERSION_LEGACY.to_string(),
src/bin/bench.rs:33:        /// Path to the benchmark database (uses a temporary file per benchmark)
src/storage/connection.rs:81:    /// Cloud-safe mode (RML-900): DELETE journal for cloud sync compatibility
src/bin/cli.rs:1396:        let dir = tempfile::tempdir().expect("temporary directory should be created");
src/bin/cli.rs:1520:        let dir = tempfile::tempdir().expect("temporary directory should be created");
src/mcp/mod.rs:20:    ToolsCapability, MCP_PROTOCOL_VERSION, MCP_PROTOCOL_VERSION_LEGACY,
src/watcher/browser.rs:5://! copying to a temporary file before reading.
src/watcher/browser.rs:139:/// temporary location and open the copy instead.
src/watcher/browser.rs:156:            // surfaces compatibility issues early.
src/intelligence/emotional.rs:209:    "deprecated",
src/intelligence/emotional.rs:218:    "hacky",
src/intelligence/emotional.rs:219:    "legacy",
src/graph/temporal.rs:73:    /// Added in schema v33. Defaults to `"global"` for backward compatibility.
src/graph/temporal.rs:198:/// compatible).
src/graph/temporal.rs:255:/// are included (backward compatible).
src/graph/temporal.rs:389:/// compatible).
src/mcp/handlers/context.rs:1279:    fn test_build_context_backward_compat() {
src/mcp/protocol.rs:162:/// Legacy MCP protocol version for backward compatibility
src/mcp/protocol.rs:163:pub const MCP_PROTOCOL_VERSION_LEGACY: &str = "2024-11-05";
src/graph/mod.rs:83:    /// Export as vis.js compatible JSON
src/storage/migrations.rs:556:        -- Default 'default' for backward compatibility
src/mcp/tools/search.rs:15:                "type": {"type": "string", "description": "Deprecated alias for memory_type"},
src/mcp/handlers/search.rs:372:    // Accept both legacy aliases (helpful/not_helpful) and canonical names
src/mcp/tools/memory.rs:12:                "type": {"type": "string", "enum": ["note", "todo", "issue", "decision", "preference", "learning", "context", "credential", "episodic", "procedural", "summary", "checkpoint", "image", "audio", "video"], "default": "note", "description": "Deprecated alias for memory_type"},
src/mcp/tools/memory.rs:89:        description: "Deprecated alias for context_seed. Use context_seed instead.",
src/mcp/tools/memory.rs:164:                "type": {"type": "string", "enum": ["note", "todo", "issue", "decision", "preference", "learning", "context", "credential", "episodic", "procedural", "summary", "checkpoint", "image", "audio", "video"], "description": "Deprecated alias for memory_type"},
src/mcp/tools/memory.rs:206:                "type": {"type": "string", "description": "Deprecated alias for memory_type"},
src/mcp/tools/memory.rs:218:                    "description": "Legacy simple key-value filter (deprecated, use 'filter' instead)"
src/storage/filter.rs:72:    /// Legacy: direct value means equality (backwards compatible)
src/storage/filter.rs:578:    fn test_backwards_compatible_direct_value() {
src/storage/filter.rs:725:        // {"tags": {"neq": "deprecated"}} should use NOT EXISTS
src/storage/filter.rs:726:        let json = json!({"tags": {"neq": "deprecated"}});
src/mcp/handlers/markdown_export.rs:189:    // available, otherwise the normalized DB content_hash for backward compat.
src/mcp/handlers/markdown_export.rs:581:        // normalized hash comparison against the DB hash for backward compat.
src/mcp/tools/mod.rs:107:/// - `Some("all")` or `None` → all tools (backward compatible)
src/mcp/tools/maintenance.rs:32:                "type": {"type": "string", "description": "Deprecated alias for memory_type"},
src/storage/queries/tests.rs:457:                    content: "Temporary memory".to_string(),
src/storage/queries/tests.rs:985:    // Dedup helper remains compatible with compute_content_hash
src/storage/image_storage.rs:4://! S3-compatible storage (like Cloudflare R2). Images are stored separately
src/mcp/handlers/mod.rs:97:                map.insert("deprecated".to_string(), json!(true));
src/mcp/handlers/mod.rs:99:                    "deprecated_message".to_string(),
src/mcp/tools/registry.rs:13:                "type": {"type": "string", "enum": ["note", "todo", "issue", "decision", "preference", "learning", "context", "credential", "episodic", "procedural", "summary", "checkpoint", "image", "audio", "video"], "default": "note", "description": "Deprecated alias for memory_type"},
src/mcp/tools/registry.rs:90:        description: "Deprecated alias for context_seed. Use context_seed instead.",
src/mcp/tools/registry.rs:165:                "type": {"type": "string", "enum": ["note", "todo", "issue", "decision", "preference", "learning", "context", "credential", "episodic", "procedural", "summary", "checkpoint", "image", "audio", "video"], "description": "Deprecated alias for memory_type"},
src/mcp/tools/registry.rs:207:                "type": {"type": "string", "description": "Deprecated alias for memory_type"},
src/mcp/tools/registry.rs:219:                    "description": "Legacy simple key-value filter (deprecated, use 'filter' instead)"
src/mcp/tools/registry.rs:1792:                "type": {"type": "string", "description": "Deprecated alias for memory_type"},
src/mcp/tools/registry.rs:3798:                "type": {"type": "string", "description": "Deprecated alias for memory_type"},
src/storage/turso_backend.rs:11://! - **Compatible schema**: Same migrations as SQLite backend
src/storage/queries/core.rs:29:    // Scope columns (with fallback for backward compatibility)
src/storage/queries/core.rs:35:    // TTL column (with fallback for backward compatibility)
src/storage/queries/core.rs:38:    // Content hash column (with fallback for backward compatibility)
src/storage/queries/core.rs:59:    // Workspace column (with fallback for backward compatibility)
src/storage/queries/core.rs:64:    // Tier column (with fallback for backward compatibility)
src/storage/queries/core.rs:1411:    // Advanced filter (RML-932) - takes precedence over legacy metadata_filter
src/storage/queries/core.rs:1421:        // Legacy metadata filter (JSON) - deprecated in favor of `filter`
exit_status=0
```

### Optional dependencies and feature gates

```bash
rg -n -i 'optional = true|features =|default-features|\[features\]' Cargo.toml sdks docs 2>/dev/null | head -180
```

```text
Cargo.toml:41:required-features = ["watcher"]
Cargo.toml:57:[features]
Cargo.toml:146:tokio = { version = "1.35", features = ["full"] }
Cargo.toml:149:rusqlite = { version = "0.31", features = ["bundled", "vtab", "functions", "trace"] }
Cargo.toml:158:serde = { version = "1.0", features = ["derive"] }
Cargo.toml:162:axum = { version = "0.7", features = ["ws"] }
Cargo.toml:163:tower = { version = "0.5", features = ["util"] }
Cargo.toml:164:tower-http = { version = "0.5", features = ["cors", "trace"] }
Cargo.toml:173:reqwest = { version = "0.12", features = ["json", "rustls-tls", "multipart"], default-features = false, optional = true }
Cargo.toml:176:async-trait = { version = "0.1", optional = true }
Cargo.toml:179:aws-sdk-s3 = { version = "1.12", optional = true }
Cargo.toml:180:aws-config = { version = "1.1", optional = true }
Cargo.toml:183:aes-gcm = { version = "0.10", optional = true }
Cargo.toml:188:zip = { version = "2.0", optional = true, default-features = false, features = ["deflate"] }
Cargo.toml:189:ed25519-dalek = { version = "2.1", optional = true, features = ["rand_core"] }
Cargo.toml:193:libsql = { version = "0.6", optional = true, features = ["core", "replication", "remote"] }
Cargo.toml:197:tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
Cargo.toml:200:clap = { version = "4.4", features = ["derive", "env"] }
Cargo.toml:208:chrono = { version = "0.4", features = ["serde"] }
Cargo.toml:209:uuid = { version = "1.6", features = ["v4", "serde"] }
Cargo.toml:225:pdf-extract = { version = "0.7", optional = true }
Cargo.toml:232:tokio-stream = { version = "0.1", features = ["sync"] }
Cargo.toml:233:meilisearch-sdk = { version = "0.32.0", optional = true }
Cargo.toml:236:tonic = { version = "0.12", optional = true, features = ["transport"] }
Cargo.toml:237:prost = { version = "0.13", optional = true }
Cargo.toml:242:ort = { version = "2.0.0-rc.12", optional = true, default-features = false, features = [
Cargo.toml:251:ndarray = { version = "0.16", optional = true }
Cargo.toml:252:tokenizers = { version = "0.20", optional = true, default-features = false, features = ["onig"] }
Cargo.toml:255:toml = { version = "0.8", optional = true }
Cargo.toml:258:notify = { version = "7", optional = true }
Cargo.toml:261:tempfile = { version = "3.9", optional = true }
Cargo.toml:264:duckdb = { version = "1.4", features = ["bundled", "chrono"], optional = true }
Cargo.toml:268:tonic-build = { version = "0.12", optional = true }
Cargo.toml:271:criterion = { version = "0.5", features = ["html_reports"] }
Cargo.toml:275:fake = { version = "2.9", features = ["derive"] }
Cargo.toml:277:tonic = { version = "0.12", features = ["transport"] }
docs/harness/plans/2026-06-05-cross-harness-improvement-plan.md:534:append_cmd "Optional dependencies and feature gates" "rg -n -i 'optional = true|features =|default-features|\\[features\\]' Cargo.toml sdks docs 2>/dev/null | head -180"
docs/harness/plans/2026-06-05-engram-harness-improvement-execution-plan.md:690:append_cmd "Optional dependencies and feature gates" "rg -n -i 'optional = true|features =|default-features|\\[features\\]' Cargo.toml sdks docs 2>/dev/null | head -180"
docs/harness/audits/2026-06-05T155933Z-quarterly-audit.md:628:rg -n -i 'optional = true|features =|default-features|\[features\]' Cargo.toml sdks docs 2>/dev/null | head -180
docs/harness/bin/quarterly-audit.sh:94:append_cmd "Optional dependencies and feature gates" "rg -n -i 'optional = true|features =|default-features|\\[features\\]' Cargo.toml sdks docs 2>/dev/null | head -180"
exit_status=0
```

### Harness generated artifacts volume

```bash
find docs/harness/reviews docs/harness/progress docs/harness/audits -maxdepth 1 -type f 2>/dev/null | sort | wc -l | tr -d ' '
```

```text
29
exit_status=0
```

### Harness Policy Decisions

| Item | Evidence | Decision | Owner | Follow-up |
|---|---|---|---|---|
|  |  | Keep / Archive / Delete |  |  |

### MCP And Docs Drift Decisions

| Item | Evidence | Decision | Owner | Follow-up |
|---|---|---|---|---|
|  |  | Keep / Archive / Delete |  |  |

### Storage And Migration Decisions

| Item | Evidence | Decision | Owner | Follow-up |
|---|---|---|---|---|
|  |  | Keep / Archive / Delete |  |  |

### Cleanup Follow-ups

| Item | Evidence | Decision | Owner | Follow-up |
|---|---|---|---|---|
|  |  | Keep / Archive / Delete |  |  |

## Human Review Notes

- Decisions:
- Follow-up issues:
- Exceptions approved:
- Next audit date:
