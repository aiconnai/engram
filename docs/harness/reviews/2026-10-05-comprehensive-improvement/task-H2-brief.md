### H2 — Endurecer schemas e semântica de evidência existentes [P1; harness; depende E0/H1]

**Files:** modificar `docs/harness/bin/validate-evidence.py`, `docs/harness/bin/test-fixtures.sh`, schemas e fixtures existentes; criar `docs/harness/tests/test_validate_evidence.py`; wiring autorizado em `.github/workflows/ci.yml`, `docs/harness/bin/sensors.sh` e `docs/harness/bin/doctor.sh`. Não editar workflow compartilhado em paralelo com Q1/Q5.
**Interfaces:** `validate_file(...)` valida estrutura, não autenticidade. Versões novas não reinterpretam artefatos v1 históricos como confiança nova. Wrapper/avaliador recebe candidate SHA e policy version esperados fora do payload do writer.

- [ ] Testes: duplicate JSON keys, NaN/Infinity, bool usado como inteiro, SHA malformado/errado, unknown fields, timeout/zero check faltante, caps ausentes, traversal, glob amplo não autorizado, data calendário impossível, PASS com finding blocking. Rodar com e sem jsonschema instalado; resultados equivalentes, keyword não suportada falha fechado.
- [ ] Corrigir lacunas comprovadas sem fallback parcial permissivo. Adicionar verificação externa de expected SHA/policy, catálogo de check IDs e hashes de logs; não confundir comparação de dois campos fornecidos pelo autor com origem confiável.
- [ ] Rodar `rtk proxy python3 -m unittest discover -s docs/harness/tests -p 'test_validate_evidence.py'`, `rtk proxy python3 docs/harness/bin/validate-evidence.py --self-test`, `rtk proxy bash docs/harness/bin/test-fixtures.sh`.
- [ ] Tornar as regressões uma lane offline obrigatória do CI/sensors, com checagem de wiring no doctor; fixtures/manual runs não bastam. Missing lane/zero testes/failure → nonzero. H3 não inicia enquanto a lane não estiver persistente e aprovada.

**Aceite:** parser estrito e semanticamente fail-closed; nenhuma fixture agent-authored vira trusted evidence. **Rollback:** versão anterior continua histórica; desabilitar novo consumidor, não downgrade automático.
