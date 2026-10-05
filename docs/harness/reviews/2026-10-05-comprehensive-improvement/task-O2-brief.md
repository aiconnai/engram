### O2 — Higiene Git e retenção recuperável [P2; operação; depende H6/O1]

**Files:** `.gitignore`, `docs/harness/reviews/`, artefatos/logs e histórico benchmark; criar `docs/OPERATIONS_GIT_RETENTION.md`.

- [ ] Inventário read-only separa tracked/raw, logs, artifacts, git object store e refs compartilhadas; métricas têm timestamp. Verificar sensibilidade sem publicar conteúdo privado.
- [ ] Definir policy por classe: owner, duração, storage/hash, consulta e restauração. Migrar uma classe de artefato em clone descartável; testar links e gates sem caches versionados. Backup/restore comprovado antes de expirar qualquer objeto.
- [ ] `gitignore`/untracking não são history cleanup. `reflog expire --expire=now --all`/`gc --prune=now` ficam fora do plano executivo; qualquer limpeza destrutiva exige autorização separada e exclusão de writers concorrentes.

**Aceite:** evidência ainda localizável/restaurável; nenhum ganho de MB prometido sem medição. **Rollback:** restaurar índice/artefatos via manifest/backup sem reescrever history por padrão.
