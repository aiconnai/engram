### H4 — Runner e evidência externa pós-commit [P1; plataforma; depende H3/H1/H2]

**Files NEW:** `docs/harness/bin/run-task.py`, `check-scope.py`, `record-evidence.py`, `docs/harness/tests/test_runner.py`, `test_scope.py`, `test_evidence_integrity.py`.
**Interfaces proposed:** scope compara base/candidate paths; recorder recebe outcome do supervisor, hashes calculados por ele e trusted identity, não JSON de sucesso do writer. Expor somente fake writer até task posterior autorizada.

- [ ] Testar dirty base, wrong repo/ref, protected rename/delete, symlink escape, submodule, mode change, filenames com newline/dash, lockfile e existing-test weakening. Wall/turn/attempt/retry cap e check timeout → nonzero, logs sobreviverão.
- [ ] Criar candidate commit só depois de scope check; executar gate em checkout limpo desse commit, separado do writer; recusar tracked/untracked output que altere input relevante. Review/evidence incluem base SHA, candidate SHA, tree, política/registry, argv, ferramentas/imagem e hashes completos; writer harness/CLI version, modelo/effort solicitado e identidade reportada (ou unavailable), limites/aprovação/changed paths. Capturar log bruto trusted sem RTK filtrado como única evidência.
- [ ] Model/tool budgets só são marcados enforced quando o adapter realmente pode limitar. Defaults propostos para piloto: wall45min, attempts2, repair1, writer concurrency1; turn cap20 só com adapter observável. Se provider cost cap não é enforceable, registrar limitação e negar task que o exige.
- [ ] Rodar unittest runner/scope/integrity; mudar um byte após gate, apagar log, adulterar hash e tentativa writer PASS → tudo recusado. Cleanup nunca apaga a única ref do candidato nem worktree de outro task.

**Aceite:** evidence aceita apenas candidato imutável verificado; candidato/ref e logs preservados no failure path. **Rollback:** desabilitar adapter/runner sem apagar receipts.
