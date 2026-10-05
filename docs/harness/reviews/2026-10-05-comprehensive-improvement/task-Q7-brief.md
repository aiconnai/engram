### Q7 — Retrieval e performance do candidato, não fixture stale [P2; search/QA; depende Q1/C3/Q4]

**Files:** `tests/retrieval_quality.rs`, fixtures corpus/baseline, budgets/schema/policy, `benches/search.rs`, `benches/memory_ops.rs`, `benches/mcp_dispatch.rs`, scripts de baseline/comparison.

**Interface NEW proposed:** `scripts/run-quality-candidate.py --candidate-sha <SHA> --corpus <path> --features <lista> --criterion <resultado-atual> --output <path>` seleciona argv fixo aprovado, verifica checkout/tree e produz report com candidate SHA/tree, toolchain/features, corpus SHA256/seed, métricas calculadas e hash do Criterion atual. Não recebe comando shell livre. Histórico é input comparativo separado; fonte atual do Criterion tem vínculo ao mesmo candidato/supervisor, não só um path fornecido. Resultado incompleto/stale é nonzero. Q1 integra este report somente após Q7 aceito.

- [ ] Testar runner com candidato diferente, metric ausente/NaN, corpus hash divergente, regressão e floor editado; avaliação atual tem seed, SHA e corpus/feature/provider. Histórico não substitui esse output.
- [ ] Expandir corpus sintético versionado com PT/EN, typos, negation, entidades ambíguas, duplicatas, daily/transcript filtering e isolamento workspace; revisão de relevância separada do tuning. Fixtures atuais com métricas 1.0 não provam retrieval de clientes.
- [ ] Rodar `rtk cargo test --test retrieval_quality` e benchmarks equivalentes (in-memory/disco/WAL/concorrência separados). Preservar ceiling existente 1.15, reconciliar thresholds entre lanes sem relaxamento automático. Floors novos só com review do corpus/runner; registrar warmup/hardware/percentis quando medidos.

**Aceite:** resultado do candidato reproduzível; nenhuma promessa de SLO hospedado. **Rollback:** dataset/runner/floors versionados; não mascarar regression rebaselining.
