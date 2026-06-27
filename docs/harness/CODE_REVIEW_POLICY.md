# Code Review Policy — Local Harness (Engram)

> Política consumida por `review-gate.sh` quando invoca um reviewer externo (Claude Code Sonnet, Codex, Ollama, etc.).
> Fonte de verdade local para severidade, evidência, e condições de parada.
> Melhora a barra de qualidade do gate; não cria um segundo gate hard por si só.

## Propósito

O review gate existe para capturar **fake successes** e contract drift que sensores locais determinísticos não detectam de forma confiável (especialmente em cenários de dual-CLI onde o implementador e o reviewer são diferentes personas/modelos).

No Engram, isso protege a qualidade da memória operacional e da superfície MCP que o time usa como fonte de verdade compartilhada.

O reviewer deve julgar o diff **como um engenheiro sênior externo**, não como o implementador re-reasoning sobre seu próprio trabalho.

Priorize: corretude, regressões, segurança, perda de dados, builds quebrados, quebras de compatibilidade (especialmente MCP protocol e SDK contracts), coverage de comportamentos alterados pelo diff, e violação de invariants.

Não faça crítica ampla de estilo. Não "premie" o código. Não redeclare o diff. Revise apenas código alterado, a menos que contexto próximo seja necessário para provar um problema introduzido pela mudança.

## Inputs Obrigatórios para o Reviewer

Antes de julgar, leia (o prompt do review-gate injeta ou referencia):

1. `docs/harness/SPEC.md`
2. `docs/harness/INVARIANTS.md` (processo)
3. `docs/harness/GATES.md`
4. `docs/harness/README.md`
5. `docs/harness/CODE_REVIEW_POLICY.md` (este arquivo)
6. Root `INVARIANTS.md` (data invariants do sistema de memória)
7. `STANDARDS.md` + `ERRORS_AND_LESSONS.md` (quando relevantes para o diff)
8. Diff ou range de commits em análise (excluindo paths de harness artifacts)

## Workflow de Review

1. Entenda a intenção do autor a partir do task-id, sprint docs, commit message, PR description, issue, ou docs ao redor. Em Engram, a intenção normalmente envolve preservar ou melhorar a confiabilidade do contexto proprietário que agentes e humanos consultam.
2. Extraia requisitos concretos de tickets, specs, plans ou acceptance criteria.
3. Identifique arquivos alterados e quaisquer project instructions com escopo relevante (AGENTS.md, Claude.md, docs/ por área).
4. Mapeie linhas alteradas para a menor unidade significativa: função, módulo MCP handler, migration, tool definition, hook, embedding provider, test, doc de contrato.
5. Revise o comportamento alterado por problemas concretos introduzidos ou piorados pela mudança.
6. Valide cada finding antes de reportar. Se depende de suposições especulativas, contexto ausente ou estado improvável → omita.
7. Self-review pass: remova findings que não foram introduzidos pelo diff, não têm trigger realista, duplicam outro, ou são melhor deixados para formatter/linter/compiler.

## Perspectivas de Review

Use estas perspectivas como lentes locais do Engram para evitar review
unidimensional. Elas não aumentam o limite de findings; cada problema reportado
continua exigindo evidência concreta no diff.

1. **Bug e edge cases** — entradas adversas, estados vazios, concorrência,
   ordering, timeouts, retries e caminhos de erro.
2. **Segurança e privacidade** — vazamento de conteúdo proprietário, segredos,
   credenciais, egress, prompts não confiáveis, hooks e multimodal.
3. **Contrato e compatibilidade** — MCP wire format, tool signatures, storage
   invariants, SDKs Python/TypeScript, CLI/API pública e compatibilidade
   retroativa.
4. **Testes e verificabilidade** — cobertura do comportamento alterado,
   sensores corretos, golden/reference updates e ausência de falso verde.
5. **Manutenibilidade e operação** — complexidade de hot path, ownership,
   observabilidade, mensagens de erro, documentação operacional e rollback.
6. **Drift histórico e memória canônica** — alinhamento com `progress.md`,
   active plan, decisões anteriores, `ERRORS_AND_LESSONS.md` e padrões já
   adotados no código.

Para diffs em `docs/harness/**`, priorize especialmente segurança/processo,
escopo negativo, verificabilidade, drift histórico e risco de enfraquecer gates.

## Fontes Externas e Boundary de Licença

Kits externos de workflow de agentes podem ser consultados apenas como fontes de
padrões de alto nível. O `NeoLabHQ/context-engineering-kit` é tratado pelo
Engram como referência externa, não como dependência, vendor, marketplace ou
substituto do harness local.

Como boundary conservador: se uma fonte externa estiver sob GPL-3.0 ou outra
licença copyleft, não copie prompts, comandos, checklists, scripts, docs ou
texto para o Engram sem revisão explícita de licença. Adaptações aceitas devem
usar wording próprio do Engram, preservar os gates locais e registrar a decisão
quando afetar processo do harness.

## Formato de Finding

Todo finding substantivo deve incluir:

- **Severidade**: `critical`, `high`, `medium`, ou `low` (mapeado para `[BLOCKER]`, `[HIGH]`, `[MED]`, `[LOW]` no output).
- **Localização**: arquivo + linha, ou a menor área de código relevante.
- **Sinal de risco**: compile failure, wrong result, runtime failure/panic, security issue, violação de invariant escopado, missing regression test para comportamento alterado, ou gap de compatibilidade/documentação.
- **Problema**: o que está errado e por que importa.
- **Evidência**: o path de código, caller, flow afetado, input, regra, ou cenário que prova o issue.
- **Recomendação**: o menor fix prático ou o teste específico que capturaria.

## Contrato de Output do Harness

- **Primeira linha**: exatamente `PASS <resumo curto>` ou `FAIL <resumo curto>`
- **Linha obrigatória para parser**: exatamente `REVIEW_VERDICT: PASS <resumo curto>` ou `REVIEW_VERDICT: FAIL <resumo curto>`
- Depois: lista concisa de findings prefixados por `[BLOCKER]`, `[HIGH]`, `[MED]`, ou `[LOW]`
- A linha `REVIEW_VERDICT:` é obrigatória para fechamento de `review-gate.sh post`.
- Review deve ser conciso e baseado em evidência
- Se o reviewer não consegue explicar por que a mudança é segura → fail closed
- Se nenhum issue substantivo encontrado, use **exatamente um bullet**:
  ```
  - [LOW] No issues found. Checked for bugs, regressions, security, tests for changed behavior, documentation, MCP protocol stability, SDK contracts, and scoped project instructions.
  ```

## O que Flaggear Primeiro (Ordem de Prioridade)

1. Quebra de contrato (MCP tool signature, wire format, SDK client expectations, storage invariants)
2. Perda de dados, corrupção de memória, ou violação de monotonicidade (IDs, timestamps, hashes)
3. Segurança / privacidade / leakage de conteúdo (especialmente em hooks, watcher, multimodal)
4. Fake-success patterns listados em `GATES.md` (especialmente embedding features, schema version, MCP golden tests)
5. Coverage ausente para o comportamento exato alterado pelo diff
6. Atualização de `progress.md` / active plan ausente (para mudanças de domínio)
7. Issues menores de corretude ou maintainability

## Checks Obrigatórios no Prompt

O review-gate injeta checagens específicas para engram:

- Se `storage/migrations.rs` ou `SCHEMA_VERSION` mudou → exigir evidência de testes de migração e integração rodando limpos.
- Se MCP handlers/tools mudaram → checar protocolo tests, reference gerada, e impacto em SDKs Python/TS.
- Se hooks (`src/hooks/`) ou intelligence modules mudaram → checar side effects, consolidação, e testes de integração.
- Se embeddings, ONNX, ou cache → checar bounds, feature flags vs CI parity, e benchmarks quando relevantes.
- Se snapshot/attestation/crypto → checar testes de atestado e golden files.
- Se mudança em `docs/harness/**` (especialmente bin/ ou INVARIANTS/GATES/POLICY) → exigir doctor.sh verde + post-gate anterior.
- Violação de root `INVARIANTS.md` (data layer) ou `STANDARDS.md`.

## Bar de Findings

Default: no máximo 3 findings substantivos. Menos é melhor quando há alta confiança de que não há issues.

Reportar:

- Bugs, vulnerabilidades, data loss, broken builds, regressões de performance significativas, quebras de compatibilidade (MCP, SDK, storage), e testes ausentes para o comportamento alterado.
- Não-conformidade com ticket/spec/plan/acceptance criteria quando verificável no diff ou código próximo.
- Gaps de documentação ou configuração quando o comportamento, API pública, migration, operação ou processo do harness mudou.
- Violações de project instructions com escopo aplicável ao path modificado.

**Não reportar**:

- Issues pre-existentes que não foram piorados pela mudança.
- Estilo, naming, formatação, legibilidade pura, ou preferências subjetivas de design.
- Implementações alternativas sem um modo de falha concreto.
- Reclamações genéricas de "faltam testes". Só flaggear teste ausente para comportamento específico alterado ou path de regressão crível.
- Issues inferidos apenas de contexto omitido em um diff parcial.

## Disciplina de Partial Diff

Assuma que a entrada pode conter apenas hunks do PR, não o codebase completo.

- Foque em código novo ou modificado introduzido pelo PR.
- Use código removido apenas para entender a mudança de comportamento.
- Não invente contexto ausente para criar findings especulativos.

## Adaptações para o Cenário Dual-CLI (implementador + Claude Code Sonnet)

- O reviewer padrão permanente é Claude Code Sonnet (`claude --model sonnet`) em outro processo/sessão autenticado localmente.
- Não use outro reviewer como padrão; só aceite outro backend com override explícito do owner e verificação de assinatura/autenticação no momento do review.
- O reviewer (o outro CLI/processo) recebe um prompt rico e estruturado.
- O implementador (este CLI) não deve "ajudar" o reviewer com raciocínio extra no mesmo contexto.
- Quando o reviewer é Claude Code Sonnet, o prompt segue o estilo de "external senior reviewer" do reference mbras harness e pode pedir exploração ativa de flows (ex.: rodar `cargo test` específico via tool se o reviewer tiver acesso ao terminal).
- O artefato salvo deve conter o output cru do reviewer para que os humanos possam auditar.
- O parser de gate usa a linha obrigatória `REVIEW_VERDICT: ...` (não a primeira linha) para decisão hard.

## Fake-Success Patterns Específicos (Injetados no Prompt)

Ver `GATES.md` para a lista completa. O reviewer deve ativamente procurar por:

- Paridade de features de embedding entre local e CI.
- Drift de versão de schema vs testes.
- MCP tools sem cobertura de protocolo/golden.
- Hooks/intelligence com side effects não testados.
- `unwrap` ou panic paths em handlers MCP/storage.
- Progress docs não atualizados em mudanças de domínio.

---

**Lembrete**: O objetivo não é perfeição. É **evidência suficiente** para que um agente futuro (ou o outro CLI) possa retomar o trabalho com confiança de que o que foi entregue é sólido dentro do escopo declarado.
## Additional Harness Policy Checks

Reviewers must apply these checks in addition to the normal finding format and output contract.

### Security Boundary

Read `docs/harness/security/anthropic-reference-harness.md`.

Flag as `[BLOCKER]` when a harness change implies autonomous execution against Engram without ADR, strong sandboxing, egress constraints, no credential mounts, and an explicit target contract.

Flag as `[HIGH]` or `[BLOCKER]` when `.claude/scan-extras.txt` or `.claude/fp-rules.txt` weakens the core policy, adds blanket suppressions, or moves org-specific tuning into invariant text.

The Anthropic reference harness is a pattern source only. Imports of its C/C++/ASAN execution pipeline are out of scope unless a separate ADR explicitly authorizes that design.

### Negative Scope

Read `docs/harness/WHAT_WE_DONT_DO.md`.

Flag hidden scope creep as `[HIGH]` or `[BLOCKER]` when a harness task changes product behavior, weakens gates, removes live code based only on static evidence, or uses sensor exclusions to make production code look green.

### Review Canvas

For complex diffs, verify that a matching Review Canvas exists under `docs/harness/canvas/YYYY-MM-DD-<task-id>.md`.

The canvas should include approaches considered, hot-path complexity, at least two edge cases, and a breakage-risk table. Missing canvas evidence is `[HIGH]` by default and `[BLOCKER]` when the change touches storage, MCP surface, harness gates, or process-critical scripts.

### Reference intake

Read `docs/harness/REFERENCE_INTAKE.md` when a change cites or adapts an
external harness resource, standard, article, repo, awesome list, benchmark,
tool catalog, prompt/workflow kit, or local-only reference artifact.

Flag as `[HIGH]` when an external source materially shapes harness process,
gates, taxonomy, skills, reviewer prompts, or exception handling without an
intake record covering source identity, license boundary, local placement,
adaptation, exclusions, and verification evidence.

Flag as `[BLOCKER]` when the change copies licensed text/prompts/scripts,
weakens local gates, imports autonomous execution, or treats an external source
as authoritative over Engram invariants, GATES, security boundary, or negative
scope.

### 12207 lifecycle tailoring

Read the 12207-inspired tailoring checklist in `docs/harness/GATES.md` when a
change cites `docs/ieee-12207.md`, lifecycle-process standards, or changes how
Engram plans, verifies, validates, measures, reviews, releases, or maintains
work.

Flag as `[HIGH]` when the change uses 12207 concepts without local tailoring
evidence: scope/circumstances, lifecycle area, rationale, risk threshold,
measurement need, verification-vs-validation evidence, and traceability.

Flag as `[BLOCKER]` when the same gap appears in changes to gates, invariants,
or process-critical scripts, or when the diff implies ISO/IEC/IEEE 12207
conformance, imports a standards process as a drop-in pipeline, or copies
licensed reference wording instead of using Engram-local language.

### Harness script changes

Harness script changes are process-critical. Reviewers must inspect `docs/harness/bin/*` changes directly for shell safety, path handling, parseability, read-only guarantees, and gate weakening.
