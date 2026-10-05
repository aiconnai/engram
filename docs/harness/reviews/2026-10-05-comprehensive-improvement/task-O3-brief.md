### O3 — Avaliadores probabilísticos como aconselhamento, não gates [P2; avaliação; depende E0/H2]

**Files NEW:** `docs/quality/probabilistic-evaluation-policy.md` e fixtures/eval isoladas aprovadas; não adicionar Laya/Kev/JEV ao gate default. Fontes locais só após reference intake/licença; nenhum peso/secret versionado.

- [ ] Preservar experimento anterior como diagnóstico: Laya6/8/Brier0.21588291625, Kev7/8/0.06903502125; oito casos curados, não holdout. JEV não executado por falta de credencial; alias local jev-latest não conta como independente. Proveniência local em `/tmp/engram-probability-review-2026-10-02/` precisa ser sanitizada/arquivada antes de virar referência durável.
- [ ] Criar benchmark holdout human-labeled separado do autor da policy, com fatos/refutações/unknown e tarefa/risco. Separar p(true), distribution over actions, ordinal score e confidence; não somar/mediar métricas incompatíveis nem tratar modelos correlacionados como votos independentes.
- [ ] Testar explicit abstention, permutações e duas redações predefinidas, contexto/truncamento e identificação do modelo. Laya mostrou P(defer)62.13–82.70% por ordem; policy fail-closed prevalece. Brier por classe, baseline/base rate, erros e estabilidade são reportados com N e limites; não fit/eval na mesma amostra.
- [ ] Hosted JEV só com credencial existente obtida de forma segura, autorização para conteúdo/budget e runtime oficial; ausência → unavailable. Não provisionar chave, instalar proxy ou enviar conteúdo privado para completar tabela.

**Aceite:** resultados não habilitam merge nem ficam obrigatórios/offline-flaky no CI; qualquer mudança policy tem avaliação independente. **Rollback:** retirar advisor sem afetar verifier.
