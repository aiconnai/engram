# Engram Harness — External Reviewer Prompt

**Task**: markdown-export-safety
**Mode**: pre
**Date (UTC)**: 2026-11-01
**Scope**: prepare  base=2e84a59422b3659aaaf25c69dbe01b01ab321332  head=2e84a59422b3659aaaf25c69dbe01b01ab321332  tree=n/a
**Diff sha256**: 56058b8446c738ac88b700fd0edb28bd4cfdd6df9149483581fb75928afc7c7d

## Instructions for the Reviewer

You are acting as an independent senior engineer reviewing a diff for the engram project.
You were NOT the implementer. Your job is to find real problems introduced by the change.

Read the following documents (they are the source of truth for this review):

- docs/harness/SPEC.md
- docs/harness/INVARIANTS.md (process invariants — canonical)
- docs/harness/WHAT_WE_DONT_DO.md (negative scope — no hidden expansion)
- docs/harness/GATES.md (especially the fake-success patterns section)
- docs/harness/CODE_REVIEW_POLICY.md (this policy)
- docs/harness/security/anthropic-reference-harness.md (security boundary)
- .claude/scan-extras.txt and .claude/fp-rules.txt (org-specific scan/triage tuning)
- docs/harness/README.md (workflow)
- Root INVARIANTS.md (data layer invariants for the memory system)

Then review the diff below.

Additional harness-specific requirements:
- Compare scope against docs/harness/WHAT_WE_DONT_DO.md. Flag hidden scope creep, gate weakening, or product changes bundled into harness work.
- Security boundary: flag autonomous Engram execution, implied sandboxing, credential mounts, network/egress expansion, or C/C++/ASAN pipeline import unless an ADR and explicit target contract are present.
- Tuning files: ensure .claude/scan-extras.txt and .claude/fp-rules.txt augment scan/triage behavior without weakening core INVARIANTS/GATES/POLICY or adding blanket suppressions.
- Review Canvas: if the diff is complex, verify that a matching docs/harness/canvas/YYYY-MM-DD-<task-id>.md exists and includes approaches considered, hot-path complexity, at least two edge cases, and a breakage-risk table.
- Harness script changes under docs/harness/bin/* are process-critical. Inspect shell safety, path handling, parseability, read-only guarantees, and whether the script weakens any existing gate.

## Key Fake-Success Patterns (hunt these actively)

1. Tests green only because local-embeddings feature was used; CI Linux parity fails.
2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
4. Clippy clean but unwrap/expect in hot MCP handler, storage, or hook paths.
5. Snapshot/attestation tests pass but Merkle or crypto behavior changed.
6. Hooks (session_end, post_tool_use, etc.) or intelligence modules changed without integration coverage.
7. Harness doctor or sensors would have caught this but were not run.
8. Progress docs (harness or active plan) not updated for a domain change.
9. Cross-SDK (python/typescript) contract drift not reflected.
10. Reviewer is being shown a self-referential or incomplete prompt (call it out).
11. Security boundary drift: static/read-only default weakened, autonomous execution implied, missing ADR/sandbox/egress/target contract, credential mounts allowed, or Anthropic C/C++/ASAN pipeline imported as default.

## Files In Scope

CHANGED_PATH=docs/harness/SPEC.md
CHANGED_PATH=docs/harness/canvas/2026-11-01-markdown-export-safety.md

## Diff Under Review

```diff
diff --git a/docs/harness/SPEC.md b/docs/harness/SPEC.md
index a40d25d32987d0f2dfb391b7bdcf28f2673e678c..b4e44107ae70b04377f15314b7fe118e98f9c74e 100644
--- a/docs/harness/SPEC.md
+++ b/docs/harness/SPEC.md
@@ -49,0 +50,8 @@
+## Follow-up autorizado — 2026-11-01, P0 export Markdown
+
+Owner autorizou seguir no P0 independente após o plano v3. Correção local sobre main revalidada,
+testes RED/GREEN, paths/permissões/expiração/integridade/atomicidade e catálogo/reference, com
+verificação no candidato. [Log](progress/2026-11-01-markdown-export-safety.md),
+[canvas](canvas/2026-11-01-markdown-export-safety.md). Tag 0.24.0 aguarda resultado; sem autorização
+de push/PR/merge/tag/publicação, configs reais, RFC0004a ou programa inteiro de evolução.
+
diff --git a/docs/harness/canvas/2026-11-01-markdown-export-safety.md b/docs/harness/canvas/2026-11-01-markdown-export-safety.md
new file mode 100644
index 0000000000000000000000000000000000000000..b985afa083a088782cd25b3457699417100ab802
--- /dev/null
+++ b/docs/harness/canvas/2026-11-01-markdown-export-safety.md
@@ -0,0 +1,33 @@
+# Review Canvas: markdown-export-safety
+
+Data: 2026-11-01. Escopo autorizado: P0 local sobre origin/main `2e84a594`.
+
+## Approaches Considered
+
+| Abordagem | Decisão | Motivo |
+|---|---|---|
+| Canonicalize da raiz seguido de fs::write | Rejeitada | Tag escapa, symlink redireciona e check/open tem corrida |
+| Incorporar projeção P1/manifest | Rejeitada | Hotfix do produto existente, sem política/RFC nova |
+| Validar lote e escrever por diretórios ancorados sem seguir links | Selecionada | Conter destinos/efeitos; filesystem não é workspace |
+| Ignorar row/relação inválida | Rejeitada | Oculta perda de dados |
+
+## Hot Path Complexity
+
+Consultar/renderizar antes de escrever. Ordenar O(n log n + e log e); buffers proporcionais ao lote.
+Sem rede/LLM/busca vetorial ou transação SQLite durante I/O de arquivos. Atomicidade por arquivo.
+
+## Edge Cases
+
+Traversal/separadores de outra plataforma; symlink de diretório/arquivo; alias SQLite; read-only e
+workspace alheio; expiradas referenciadas; row/relacionamento inválido; empate no índice; erro de
+escrita/rename; corrida em entradas de diretório e limites da raiz administrativa confiável.
+
+## Breakage Risk
+
+| Risco | Impacto | Mitigação/verificação |
+|---|---|---|
+| Permissão sobe de read-only | Cliente antes permissivo é recusado | Nota/dispatcher negativo antes dos efeitos e referência coerente |
+| Plataforma sem primitiva segura | Fallback reabre escape | Falhar fechado sem escrita insegura |
+| Falha parcial de lote | Falso export completo | Preflight e erro explícito; atomicidade só por arquivo |
+| Link falso/fora de escopo/TTL | Histórico incorreto ou vazamento | IDs elegíveis do lote, direção explícita e negativos |
+| Review/gate de outro SHA | Falsa aprovação | Candidato identificado; revisão/receipt próprios, risco não fecha por baseline |
```

## Previous Review Context (if any)

(no previous review supplied for continuity)

## Output Contract (strict)

Your entire response must start with exactly one of:

PASS <one-line summary of what was reviewed and why it is safe>

or

FAIL <one-line summary of the most important problem(s)>

Then a short bullet list using [BLOCKER], [HIGH], [MED], [LOW].
At most 3 substantive findings. Evidence and location required for each.
If nothing substantive: exactly one bullet with [LOW] No issues found...

Remember: you are the external reviewer. Be evidence-driven and skeptical.

Machine-parseable verdict (required):
Add exactly one line, anywhere in the response, beginning with:
REVIEW_VERDICT: PASS <one-line summary>
or
REVIEW_VERDICT: FAIL <one-line summary>
This line is required for hard post-gate enforcement. A response with zero or
more than one such line is rejected.
