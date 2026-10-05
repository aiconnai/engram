### C6 — Hooks, multimodal e sync com falha/cancelamento [P1/P2 conforme alcance; core; depende C2/Q4]

**Files:** `src/hooks/`, `src/storage/pending_injections.rs`, `src/intelligence/pdf_worker.rs`, `src/multimodal/`, `src/sync/`; `tests/pdf_worker.rs`, `tests/multimodal_artifact_indexing_tests.rs`, `tests/watcher_integration.rs`, suites dream/sync existentes. Abrir sub-PRs hooks, multimodal e sync: um pode ser aceito sem aprovar os outros.

- [ ] Inventariar side effects e testar cancelamento, payload excedente, subprocess crash, retry duplicado, provider lento e interromper/reiniciar sync. Hooks incluem replay/FIFO/TTL e notas excedentes; multimodal inclui create/index/read/delete/cleanup e path ownership; sync inclui wrong-key non-destructive e restart/conflict. Distinguir operação de negócio de kill/wait best-effort.
- [ ] Video: fake ffmpeg que trava/falha, spawn error e VisionProvider lento/falho após extração. `extract_keyframes` e `create_video_memory` têm timeout/kill/wait e ownership explícito de `engram_frames_*`; erros/cancelamento não deixam filho ou diretório órfão. Sucesso com paths retornados define transferência de ownership e cleanup após último consumidor; não apagar frames ainda usados.
- [ ] Corrigir gaps comprovados com limites/timeout explícitos e deduplicação definida. Descrever resultado parcial em vez de retornar sucesso silencioso; não afirmar exactly-once universal.
- [ ] Executar suites específicas com stubs offline; verificar ausência de subprocess/task órfão, retry infinito e mutação não autorizada. Testes de serviço externo são separados, opcionais e autorizados.

**Aceite:** falha pode ser reproduzida/diagnosticada e não cria estado corrupto; policy de retry conhecida. **Rollback:** feature nova desabilitada explicitamente, sem provider fallback silencioso.
