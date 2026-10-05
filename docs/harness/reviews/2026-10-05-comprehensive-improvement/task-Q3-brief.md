### Q3 — Nightly efetivo: fuzz, mutation e Miri [P1; QA/CI; depende C3/Q5 e autorização de execução]

**Files:** `.github/workflows/nightly.yml`; criar `fuzz/Cargo.toml`, `fuzz/fuzz_targets/entity_extraction.rs`, `fuzz/fuzz_targets/workspace_normalization.rs`, `scripts/run-fuzz-smoke.sh`, `scripts/test_optional_lane_contracts.py`.

- [ ] Fixtures do runner: targets vazios, list failure, crash e infraestrutura indisponível → status explícito não-pass. Escolher APIs públicas reais antes de compilar targets; corpus com entradas C3.
- [ ] Instalar ferramentas apenas em imagem/ambiente aprovado e pinado; listar targets e executar 60s por target com limites de memória/walltime. Não usar `|| true`/continue-on-error para chamar execução falha de bem-sucedida.
- [ ] Corrigir schedule semanal mutants, hoje não alcançado pelo cron diário; testar daily/weekly/manual. Miri usa alvo compatível, limitações SQLite/FFI explícitas, não claims de cobertura total. Rodar unittest do contrato e fuzz bounded em sandbox.
- [ ] Inventory declarado de targets fuzz precisa ser não-vazio e corresponder aos targets listados/compilados/executados; relatório registra contagens por alvo e preserva falhas. Miri tem lista nominal compatível e contagem >0; filtro vazio/target não encontrado → não-pass. Testar mapeamento de `github.event.schedule`, não apenas existência de cron; nenhuma falha desaparece em `continue-on-error`.

**Aceite:** relatórios distinguem pass/fail/not-run/unsupported e incluem reproducer; required gates permanecem inalterados. **Rollback:** desativar lane explicitamente com owner; nunca gerar verde sintético.
