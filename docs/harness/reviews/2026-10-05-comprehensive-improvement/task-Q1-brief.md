### Q1 — Reparar o contrato CI/local de qualidade [P0; CI]

**Files:** modificar `.github/workflows/ci.yml`, `scripts/ci.sh`, `scripts/ci-parity-check.sh`; criar `scripts/test_check_quality_ci_contract.py`; revisar `scripts/check-quality-budgets.py` e `docs/quality/retrieval-performance-policy.md`.
**Interfaces:** separar validação de baseline histórica de avaliação de resultados do candidato; input de runtime tem SHA/toolchain/features e arquivo Criterion atual, não só uma fixture de main.

**Etapas independentes:** Q1a entrega contrato CI/local e integridade histórica, sem depender de Q7; todas as dependências `Q1` neste plano significam **Q1a**. Q1b é sub-PR posterior que integra resultados atuais aceitos de Q7. Não esperar Q1b para iniciar Q7 nem chamar Q1a de performance verificada.

- [ ] Testar via subprocess que `--criterion` ausente retorna 2; arquivo ausente, unidade inválida e regressão 116% retornam não-zero; a chamada completa e as self-tests usam arquivos válidos. Testar argv do workflow e paridade required sem depender de strings legadas de Makefile.
- [ ] Corrigir a chamada com argumento obrigatório. A validação do arquivo histórico pode verificar integridade, mas não provar desempenho atual. Integrar medição atual em Q7, sem baixar floors ou fazer argumento optional.
- [ ] Integrar o mesmo contrato do checker em `scripts/ci.sh`/`make ci` e testar argv completos das lanes local e CI, não somente procurar strings em `ci-parity-check.sh`. Enquanto Q7 não fornecer resultado atual, nomear a lane como integridade de baseline histórica; nenhuma alegação de performance do candidato. Ambiente/ferramenta ausente é não-pass.
- [ ] Rodar `rtk proxy python3 -m unittest discover -s scripts -p 'test_check_quality_ci_contract.py'`; `rtk proxy python3 scripts/check-quality-budgets.py --budgets docs/quality/budgets.json --retrieval tests/fixtures/retrieval_quality/baseline.json --criterion benches/results/benchmark_baseline.txt --self-test-degraded`; paridade e CI Linux completos.

**Aceite:** erro reproduzido tem regressão; comandos local/CI concordam; uso histórico aparece como histórico. **Rollback:** revert coordenado de wiring; suspensão explícita não equivale a sucesso.
