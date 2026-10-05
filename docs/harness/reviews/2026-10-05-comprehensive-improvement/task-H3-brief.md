### H3 — Registry e sandbox com fake writer [P1; plataforma/segurança; depende H2 e ADRs aceitos]

**Files NEW:** `docs/harness/checks/registry.json`, `docs/harness/bin/sandbox-adapter.py`, `docs/harness/tests/fake_writer.py`, `docs/harness/tests/test_sandbox_adapter.py`. Atualizar segurança/ADR em PR próprio antes do código autoritativo.
**Interfaces proposed:** `run_isolated(manifest_path: Path, worktree_path: Path, run_dir: Path) -> RunOutcome`; outcome contém `status`, `exit_code`, `argv`, `started_at`, `finished_at`, `log_paths` e `limits_enforced`, registrados pelo supervisor. Registry seleciona argv fixo de checks humanos, nunca shell gerado. TCB e run_dir fora do writer-writable mount.

- [ ] Testar falta de runtime/imagem, egress, escrita fora do worktree, TCB/manifest readonly, HOME/SSH/askpass ausentes, timeout/process cap e subprocess sobrevivente → recusa/termination completa. Fake writer tenta adulterar evidência.
- [ ] Implementar uma imagem por digest, capabilities dropped, no-new-privileges, user não privilegiado, network disabled e limites. Credential brokerage de agente real não pertence ao MVP.
- [ ] Rodar unittest isolada e smoke sandbox com fake writer; nenhuma fallback host. Instalação/check cache e execução de código não confiável nunca compartilham secrets.

**Aceite:** boundary comprovada negativamente; fake writer apenas. **Rollback:** disable execution; static review permanece disponível.
