PASS Mechanical split of `embedding/queue.rs` and `mcp/handlers/misc.rs` into submodules — no logic changes, all 9 queue tests pass, clippy clean.

- [LOW] `src/embedding/queue/worker.rs` expõe campos internos do `EmbeddingWorker` como `pub(super)` para permitir acesso direto pelos testes em `tests.rs`. Isso é aceitável dado que o módulo de testes vive no mesmo super-módulo (`queue/`), mas é uma fuga de encapsulamento: os testes constroem o worker diretamente com `Arc<Mutex<Connection>>` bruto (sem `Storage`), exercitando apenas a path do `EmbeddingWorker`, não a path canônica de produção via `drain_pending_embeddings`. Não é um bug introduzido por este diff — existia antes — mas a exposição `pub(super)` torna o padrão mais visível. Impacto: nenhum em runtime.

- [LOW] O canvas `docs/harness/canvas/2026-07-08-mcp-misc-split.md` documenta o split do `misc.rs`, mas o diff inclui também o split do `embedding/queue.rs`. Não há canvas correspondente para `embedding-queue-split`. Se o processo exige canvas para diffs complexos (ADR-CLEANUP-20260708-2 row 2), a ausência é uma lacuna documental — não um defeito funcional.

- [LOW] `src/embedding/queue/mod.rs` usa `#[allow(unused_imports)]` em três re-exports (`get_embedding_queue_health_with_config`, `retry_failed_embeddings`, `EmbeddingRequest`) com comentário explicando que são usados apenas por testes. A supressão é cirúrgica e justificada, mas se esses símbolos forem permanentemente "test-only", o padrão mais limpo seria movê-los para dentro de `#[cfg(test)]`. Sem impacto funcional.

REVIEW_VERDICT: PASS Split puramente mecânico, sem mudança de comportamento, testes verdes, sem regressões ou security drift detectados.
