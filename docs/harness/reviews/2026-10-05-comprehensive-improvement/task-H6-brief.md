### H6 — Contexto curto, retomável e retenção explícita [P2; docs/harness; depende E0/H1]

**Files:** `docs/harness/progress.md`, progress logs, AGENTS/CLAUDE, bootstrap/doctor e onboarding; criar `docs/harness/context-budget.md` e testes de routing/leitura no PR autorizado.

- [ ] Medir bytes e tokens com tokenizer identificado, repetição e arquivos realmente carregados. Testar task ativa, seção histórica, link inexistente e clone limpo; novo índice leva ao active plan sem cortar no meio de parágrafo.
- [ ] Reconciliar #152: counter compartilhado já existe; document ingestion continua por chars e `TokenChunker` não tem caller de produção. Owner decide explicitamente se o contrato exige tokenizer/model conhecido nessa ingestão. Se sim, criar sub-PR independente com seleção explícita de tokenizer, fallback identificado, counts/metadados e teste multilíngue de budget; se não, documentar chars como limite distinto de tokens. Não declarar integração completa por existir o tipo, nem adicionar modelo silenciosamente. Essa decisão é input de Q7; implementação não se mistura ao PR de contexto/harness.
- [ ] Separar live summary e histórico por seção/tarefa, com links estáveis. Reduzir duplicação AGENTS/CLAUDE sem contradizer precedência; read order só muda em política aprovada. Não reduzir mandatory authority para economizar contexto.
- [ ] Meta proposta: live summary ≤150 linhas e bootstrap ≤50 linhas/<500ms; comparar contexto útil e tempo de retomada antes/depois. Não exigir % de economia sem baseline medido. Doctor e test-check-live-state precisam passar.

**Aceite:** cenário de retomada encontra escopo/limites/última evidência sem memória do chat; histórico não perdido. **Rollback:** índice/documentação anterior preservados e links testados.
