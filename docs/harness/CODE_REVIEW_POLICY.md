# Code Review Policy — Local Harness (Engram)

> Política consumida por `review-gate.sh` quando invoca um reviewer externo (Claude Code, Grok Build, Codex, Ollama, etc.).
> Fonte de verdade local para severidade, evidência, e condições de parada.
> Melhora a barra de qualidade do gate; não cria um segundo gate hard por si só.

## Propósito

O review gate existe para capturar **fake successes** e contract drift que sensores locais determinísticos não detectam de forma confiável (especialmente em cenários de dual-CLI onde o implementador e o reviewer são diferentes personas/modelos).

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

1. Entenda a intenção do autor a partir do task-id, sprint docs, commit message, PR description, issue, ou docs ao redor.
2. Extraia requisitos concretos de tickets, specs, plans ou acceptance criteria.
3. Identifique arquivos alterados e quaisquer project instructions com escopo relevante (AGENTS.md, Claude.md, docs/ por área).
4. Mapeie linhas alteradas para a menor unidade significativa: função, módulo MCP handler, migration, tool definition, hook, embedding provider, test, doc de contrato.
5. Revise o comportamento alterado por problemas concretos introduzidos ou piorados pela mudança.
6. Valide cada finding antes de reportar. Se depende de suposições especulativas, contexto ausente ou estado improvável → omita.
7. Self-review pass: remova findings que não foram introduzidos pelo diff, não têm trigger realista, duplicam outro, ou são melhor deixados para formatter/linter/compiler.

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

## Adaptações para o Cenário Dual-CLI (Claude + Grok Build)

- O reviewer (o outro CLI) recebe um prompt rico e estruturado.
- O implementador (este CLI) não deve "ajudar" o reviewer com raciocínio extra no mesmo contexto.
- Quando o reviewer é Grok Build (otimizado para agentic/tool use, long-running, subagents), o prompt pode ser mais denso e pedir exploração ativa de flows (ex.: rodar `cargo test` específico via tool se o reviewer tiver acesso ao terminal).
- Quando o reviewer é Claude Code, o prompt segue o estilo de "external senior reviewer" do reference mbras harness.
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
