### E0 — Reconciliar estado, autorização e baseline [P1; plataforma/owner]

**Files:** ler SPEC, progress, ADR, schemas, validate-evidence.py e `scripts/ci.sh`; criar `docs/harness/audits/2026-10-02-improvement-baseline.md` no PR futuro. Alterações a SPEC/progress/ADR só em governança dedicada.
**Interfaces:** consome snapshot e docs; produz inventário `capability → implementation → policy status → evidence SHA → owner`, sem promover autorização.

- [ ] Separar capacidades implementadas de ativadas e aprovadas; localizar por histórico a introdução de schemas/validator já existentes. A presença desses arquivos não permite recriá-los nem usá-los como TCB automaticamente.
- [ ] Fixar repo/base de execução, SHA remoto, merge-base, divergência e status de revisão/publicação dos 14 commits locais. Owner decide a baseline adequada antes de execução; não publicar, descartar ou incorporar esses commits implicitamente. Congelar manifest, diff e árvore do escopo autorizado.
- [ ] Reconciliar a seção antiga de audit/deny advisory em GATES com o Security Gate agregado obrigatório transitivo. Conferir branch protection live somente por consulta read-only autorizada; não modificar a proteção.
- [ ] Executar bootstrap, doctor e, na futura baseline aprovada, full sensors. Registrar falhas reais separadas de limitações; não atualizar datas/SHAs para fabricar verde.

**Aceite:** discrepâncias têm resolução/owner; ADR permanece Proposed até aceite humano verificável. **Rollback:** revert do PR documental preservando receipts.
