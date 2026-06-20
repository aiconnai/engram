# Harness Invariants — Regras de Processo Invioláveis (Engram)

> **Canônico para o harness.** Sobrescreve qualquer doc de processo que diverja (incluindo AGENTS.md, Claude.md, CLAUDE.md, specs antigas). Mudança aqui exige ADR em `docs/decisions/` + PR dedicado revisado pela versão **anterior** dos gates (não pelo diff que a altera).

Cada invariant tem um **gate** indicando como é enforçado.

## Session & Context

1. **Bootstrap obrigatório no início de toda sessão.** Todo agente (Claude Code, Grok Build, Codex, Cursor, etc.) deve rodar `bash docs/harness/bin/bootstrap.sh` antes de planejar ou editar. Output do bootstrap é o ponto de partida canônico.
   _Gate: revisão manual + codex/review-gate prompt checa menção ao bootstrap; doctor.sh valida que bootstrap roda rápido e sem erro._

2. **Ordem de leitura obrigatória.** Antes de qualquer mudança significativa: SPEC.md → INVARIANTS.md (harness) → WHAT_WE_DONT_DO.md → GATES.md → CODE_REVIEW_POLICY.md → progress.md → active plan.
   _Gate: review-gate prompt injeta a ordem; findings por violação._

3. **Sem drift entre SPEC e progress.** `Active sprint`, `Active task` e `Active plan` devem ser idênticos entre `SPEC.md` e `progress.md` (após normalização).
   _Gate: `doctor.sh` (hard fail)._

4. **Active plan deve existir.** O arquivo apontado por `Active plan` deve existir em `docs/harness/progress/`.
   _Gate: `doctor.sh`._

## Commits & Mudança de Estado

5. **Conventional Commits com scope explícito.** `type(scope): subject`. Scopes válidos incluem `harness`, identificadores de RFC/ENGRA, ou áreas do sistema (`mcp`, `storage`, `intelligence`, `hooks`, `sdk-python`, etc.). Sem `Co-Authored-By` ou trailers de IA.
   _Gate: `check-commit-msg.sh` (bloqueia commit local via hook ou manual)._

6. **PR titles sem marcador `[codex]`.** PR title nunca deve conter o marcador `[codex]`. O título descreve a mudança, não quem a criou.
   _Gate: `check-pr-title.sh` antes de `gh pr create` / `gh pr edit`; `doctor.sh` valida o guard._

7. **Todo commit de feature atualiza memória canônica.** Commits que alteram comportamento de domínio, MCP surface, hooks, ou processo do harness **devem** atualizar `progress.md` + log da sprint ativa. Commit sem atualização de progresso é FAIL no review-gate.
   _Gate: review-gate (prompt checa diff por atualização de progress docs)._

8. **Arquivos específicos no commit.** Nunca `git add .` ou `git add -A` para features. Commits devem listar explicitamente os arquivos alterados.
   _Gate: revisão manual + review-gate._

9. **Issue boundaries exigem version control explícito.** Antes de iniciar uma nova issue, o worktree deve estar limpo ou a sujeira deve estar explicitamente atribuída à issue atual. Antes de marcar issue como done, deve existir commit Git explícito ou change `jj` descrito com o ID da issue. `jj` pode ser usado para evolução local; Git continua canônico para release, tags e `cargo publish`.
   _Gate: `vc-gate.sh start|done|release` + revisão manual._

## Review & Verifier

10. **Single-Process Judgment.** O agente/CLI que implementou a mudança **não** é o juiz final da mesma. Review cross-CLI (ou cross-model) é exigido para tarefas não-triviais.
   _Gate: `review-gate.sh post` (hard gate)._

11. **2 FAILs consecutivos no mesmo task → escalar humano.** Não iterar infinitamente no review-gate post.
   _Gate: `review-gate.sh` + processo humano._

12. **Artefatos de review são imutáveis e versionados.** Nunca sobrescrever `reviews/*-post.md`. Usar sufixos `-v2`, `-v3` etc. para reruns no mesmo dia/UTC.
    _Gate: review-gate script + doctor checks._

13. **Exclusões de sensor só com known-issue + razão + registro prévio.** Somente para dependências externas temporárias. Nunca para "fazer o sensor passar" em código de produção sem ADR.
    _Gate: `sensors.sh` + doctor + review-gate._

14. **`review-gate.sh post` só aceita parsing por marcador explícito.** A decisão hard do post-gate exige `REVIEW_VERDICT: PASS|FAIL ...` no artefato do reviewer; `PASS/FAIL` textual sem este marcador é inválido para automação.
    _Gate: `review-gate.sh post` (parser rígido) + política local aplicada ao revisor._

## Harness Self-Consistency

15. **Doctor.sh é a fonte de verdade para integridade do harness.** Mudanças em scripts, docs de harness, ou wiring (ex.: referências à CODE_REVIEW_POLICY.md) só são válidas se `doctor.sh` passa limpo antes e depois.
    _Gate: `sensors.sh` inclui doctor; review-gate checa doctor output._

16. **Bootstrap é read-only, rápido e estável.** Output limitado a ~50 linhas, <500ms, sem side effects. Mudanças que violam isso quebram o contrato de onboarding de agentes.
    _Gate: `doctor.sh` (bootstrap size + exit code)._

17. **Policy local de review é referenciada consistentemente.** bootstrap.sh, review-gate.sh, GATES.md, README.md e CODE_REVIEW_POLICY.md devem apontar uns para os outros de forma que doctor.sh valide.
    _Gate: `doctor.sh` greps por referências._

## Security Boundary & Anthropic Reference Harness

22. **Static/read-only first é o default para trabalho de harness.** Threat modeling, scan, triage e patch candidates começam como leitura/escrita de artefatos; execução autônoma de Engram nunca é implícita por docs, prompts ou tuning files.
    _Gate: `doctor.sh` valida `docs/harness/security/anthropic-reference-harness.md`; review-gate injeta checagem de scope creep._

23. **Execução autônoma contra Engram exige ADR e sandbox explícitos.** Qualquer pipeline futura que compile, rode, ataque ou mutacione Engram automaticamente exige ADR em `docs/decisions/`, sandbox forte, restrição de egress, ausência de mounts com credenciais e target contract explícito.
    _Gate: review-gate + checklist humano; `doctor.sh` valida o contrato canônico._

24. **O Anthropic reference harness é fonte de padrões, não pipeline drop-in.** Não importar a trilha C/C++/ASAN/Docker/gVisor como default do Engram; transferir somente boundaries, prompt shape, triage discipline, sandbox posture e false-positive tuning.
    _Gate: review-gate flaggea imports indevidos como scope creep._

25. **Docs e scripts do harness referenciam a security note de forma consistente.** `README.md`, `GATES.md`, `CODE_REVIEW_POLICY.md`, `bootstrap.sh`, `doctor.sh`, `sensors.sh` e `review-gate.sh` devem apontar para `docs/harness/security/anthropic-reference-harness.md`.
    _Gate: `doctor.sh` hard fail._

26. **Tuning org-specific vive fora do core policy text.** Categorias extras de scan ficam em `.claude/scan-extras.txt`; regras de falso positivo ficam em `.claude/fp-rules.txt`. Esses arquivos sao versionados e revisados como codigo, mas nao substituem INVARIANTS/GATES/POLICY.
    _Gate: `doctor.sh` hard fail + review-gate._

27. **Sem fallback silencioso para postura de seguranca mais fraca.** Se a security note, anchors do contrato, ou tuning files obrigatorios estiverem ausentes ou divergentes, scripts devem falhar fechado ou reportar explicitamente a inconsistencia.
    _Gate: `doctor.sh` hard fail; review-gate para mudancas em `docs/harness/bin/*`._

## Rust & Engram Específicos

18. **Paridade CI local é sagrada.** `just ci` (ou `make ci`) deve reproduzir o que o GitHub Actions exige (fmt, clippy -D warnings, testes com CI_FEATURES, docs + MCP reference). Não "funciona na minha máquina" com features locais apenas.
    _Gate: `sensors.sh` invoca o ci target + review-gate checa por hacks de CI._

19. **Schema version + testes de migração.** Alterar `storage/migrations.rs` exige atualizar `SCHEMA_VERSION` **e** todos os testes que têm versão hardcoded. Nunca silenciar mismatches.
    _Gate: cargo test + review-gate (prompt procura version drift)._

20. **MCP surface changes são breaking por default.** Adicionar/remover/renomear tools expostas exige atualização de `scripts/generate-mcp-reference.sh`, docs/MCP_TOOLS.md, testes de protocolo, e SDKs (quando breaking). Coordenação visível.
    _Gate: review-gate + sensors (MCP ref check)._

21. **Sem unwrap em paths quentes.** Todos os paths de produção (MCP handlers, storage, hooks, intelligence) usam `?` + contexto de erro. `unwrap()` / `expect()` só em testes ou inicialização com panic intencional documentado.
    _Gate: clippy (allowlist controlada) + review-gate._

## Exclusões & Escapes

- Mudanças puramente em `docs/harness/reviews/*` ou `docs/harness/progress/*` (sem tocar código ou outros docs de harness) podem ter relaxamento no review-gate (ver GATES.md skip allowlist).
- Exclusões de sensor exigem known-issue explícito + razão + registro em progress (contrato em GATES.md).

---

**Observação de governance**: Invariants do harness são estáveis dentro de uma sprint. Mudança exige nota explícita em `progress.md` + justificativa no active plan. Para lições históricas que originaram estes invariants, consulte `ERRORS_AND_LESSONS.md` e o RFC 0001.

Estes invariants protegem a **resumibilidade** e **confiabilidade** do trabalho com agentes em escala. Eles não são burocracia — são o que permite que Grok Build, Claude Code e futuros agentes operem no mesmo repo com contexto compartilhado, memória canônica e sem drift silencioso.
