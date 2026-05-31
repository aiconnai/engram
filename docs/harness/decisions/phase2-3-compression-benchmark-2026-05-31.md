# Fase 2.3 — Decisão #31: Benchmark de compressão de prompt

**Data:** 2026-05-31  
**Issue:** #31 (`Benchmark prompt compression before neural backend`)  
**Referência:** [RFC 0002](../rfcs/0002-compression-benchmarks-for-context.md)

## Resultado

Decisão aprovada: **manter compressão determinística local como abordagem core** (Option B), com caminho neural/external classificado como **optional** e sem implementação imediata.

## Critérios de decisão aplicados

- Latência: execução de `cargo bench --bench token_reduction -- --nocapture` com:
  - `OutputFilter`,
  - `TruncationEngine`,
  - pipeline completo,
  - consolidação.
- Redução de tokens: validações já existentes em `benches/token_reduction.rs` garantem:
  - truncação por orçamento sem expansão;
  - redução no pipeline completo em cenário de orçamento agressivo.
- Qualidade/citação: cobertura inicial por testes e benchmarks em `src/intelligence/compression_semantic.rs` e `benches/token_reduction.rs`:
  - preservação de termos-chave em textos encurtados,
  - extração de entidades,
  - deduplicação com controle de impacto (limiar atual de similaridade documentado + teste adversarial técnico),
  - ratio/recall medidos em corpus fixo (sem cobertura de todos os domínios críticos ainda).
- Custos operacionais:
  - abordagem local não introduz chamadas externas;
  - risco de drift/falha de serviço reduzido.

## Evidência executada

### 1) Benchmarks de compressão existentes

- Evidência de coleta primária:
  - `docs/harness/reviews/2026-05-31-compression-benchmark-ratio-recall.md`

- Comando executado:
  - `cargo bench --bench token_reduction -- --nocapture`
- Saídas observadas incluem:
  - `token_reduction/output_filter/cargo/lines/100` ... `time ~3.4 µs`
  - `token_reduction/truncation_engine/loose_8k/8000` ... `time ~22 µs`
  - `token_reduction/full_pipeline/cargo_lines/1000` ... `time ~31 µs`
  - `token_reduction/semantic_compression/fixed_corpus_ratio_recall` ... `ratio/recall baseline`
  - consolidação em memória consolidada (`consolidation/memories/200`) em ordem de milissegundos

### 2) Evidência de integração e qualidade base

- `src/intelligence/compression_semantic.rs`
  - testes de remoção de filler,
  - ratio calculada e coerente com `original_tokens`/`compressed_tokens`,
  - preservação de entidades numéricas/nominais, fato-chave e deduplicação conservativa.
- `src/intelligence/context_compression.rs`
  - níveis `None/Light/Medium/Heavy` já mapeados;
  - fallback explícito para entradas sem encaixe: retorno de `skipped_ids` em `compress_for_context_with_diagnostics`.

### 3) Limites e próximos passos técnicos

- Esta revisão ainda não prova recall em domínio técnico amplo; a decisão mantém uma garantia parcial (corpus fixo atual).
- Casos adversariais de deduplicação semântica (ex.: endpoints técnicos similares com semântica distinta) foram explicitamente cobertos no compressor com proteção para tokens técnicos divergentes.
- A garantia de recall permanece parcial até expansão do corpus para domínios técnicos adicionais e inclusão de casos de falha de compressão.

## Consequências práticas

1. **Implementação #31 não aciona integração neural de curto prazo.**
2. Métricas de recall/citation permanecem ancoradas ao corpus fixo atual até expansão por sub-issue de cobertura.
3. Segue para Fase 2 restante e Fase 4 de core features, com a observação explícita de que novos casos técnicos entram como gates adicionais.

## Status

- **Decisão finalizada para Fase 2.3**  
- `docs/rfcs/0002-compression-benchmarks-for-context.md` criado com o protocolo de decisão e plano de follow-up.
