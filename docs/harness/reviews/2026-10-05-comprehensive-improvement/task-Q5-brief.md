### Q5 — Segurança agregada, findings e supply chain [P1; segurança/CI; depende E0/Q1]

**Files:** `.github/workflows/ci.yml`, `.github/workflows/codeql.yml`, `scripts/check-security-gate.py`, `tests/fixtures/security_gate_matrix.json`, configs dos scanners e docs de segurança.

- [ ] Exercitar constituent failure, missing, cancelled e unauthorized skip → recusa; skip explicitamente permitido → neutral, não PASS genérico. Executar self-tests já existentes `--self-test-failure` e `--self-test-unrequired` com a matrix.
- [ ] Definir separadamente execução de scanner, publicação SARIF e política de findings bloqueantes. Deduplicar só após provar cobertura do aggregate e retenção SARIF. Separar job de medição read-only do job de comentário/publicação; PR não confiável não recebe credenciais write.
- [ ] Adicionar fixtures e decisão mecânica para finding high bloqueante mesmo com scanner exit0, exceção explicitamente aprovada com owner/expiry, SARIF ausente/stale/malformado e scanner não executado. Identidade/SHA do relatório vêm do supervisor confiável; findings policy exige aceite próprio, não autodeclaração do payload. Não confundir upload SARIF bem-sucedido com ausência de findings.
- [ ] Revisar imagens por digest e actions já pinadas por SHA; atualizações mantêm provenance e smoke. Workflow/matrix checker devem rejeitar remoção do required-context dependency.

**Aceite:** nenhum scan ausente conta como limpo; identidade/digest/finding policy documentados. **Rollback:** workflow e matrix revertidos juntos sem bypass.
