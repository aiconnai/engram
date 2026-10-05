### H5 — Review SHA-bound e merge-policy read-only [P1; plataforma/segurança; depende H4/Q5]

**Files:** schemas/review existentes; NEW `docs/harness/bin/merge-gate.py`, `docs/harness/tests/test_merge_gate.py`, workflow separado `agent-evidence.yml` somente após desenho TCB aceito.
**Interfaces:** consome aprovação externa, PR head atual, CI artifacts e policy version esperados; produz decisão structured `eligible/refused` com razões, nunca merge/deploy.

- [ ] Fixtures wrong task/SHA/policy, reviewer unavailable, malformed/prose PASS, PASS com blocking, log missing, later FAIL, stale base, head trocado durante avaliação e approval do candidato anterior → refused. Rechecar current head antes de qualquer ação humana posterior.
- [ ] Reviewer readonly/fresh-context recebe task/diff/files/evidence, sem transcript do writer. Migração dos consumidores ocorre conjuntamente; marcador legado continua histórico, não satisfaz a nova decisão.
- [ ] Reviewer identity/provenance vem de supervisor/serviço autorizado fora do writer payload; campo livre `reviewer` não autentica aprovação. Fixtures reviewer forjado/não autorizado, replay de approval e troca de lineage → refused. Política de lineage é validada por identidade confiável ou permanece unavailable, nunca inferida do nome textual.
- [ ] CI supervisor/policy de base protegido não usa script alterado pelo próprio PR para registrar confiança. Candidato é input não confiável, sem secrets; validar provenance do job/artifact fora desse ambiente. Examinar merge queue/synthetic merge: head e tree integrado têm evidências distintas.
- [ ] Rodar unittest do merge-gate e testes de workflow com fixtures trusted/untrusted. Workflow apenas contents-read, sem approve/merge/publish token.

**Aceite:** decisão determinística recusa incompletude; humano ainda decide merge. **Rollback:** desabilitar avaliador novo e preservar gates aceitos; evidência velha não é reetiquetada.
