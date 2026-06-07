# Fase 2.2 — Search vector bottleneck e lacuna `sqlite-vec`

**Data:** 2026-06-07
**Contexto:** Follow-up da RFC 0003 (`docs/rfcs/0003-search-index-v2.md`) e do
pacote de benchmark `docs/rfcs/0003-search-index-v2-benchmark.md`.
**Status da decisão:** Medido; spike bench-only `sqlite-vec` executado;
`sqlite-vec` classificado como otimização de constante, não solução do alvo 1M.

## Evidência medida

O pacote RFC 0003 foi executado decision-grade (`100K × 384d × 100 iters`),
agora incluindo o spike `sqlite-vec` `vec0` e a métrica de completude do
postfilter:

```bash
ENGRAM_SEARCH_BENCH_REPORT=1 \
ENGRAM_SEARCH_BENCH_REPORT_SIZES=100000 \
ENGRAM_SEARCH_BENCH_EMBEDDING_DIMS=384 \
ENGRAM_SEARCH_BENCH_REPORT_ITERS=100 \
cargo bench --bench search search_index_v2_report
```

Resultado em `target/criterion/search-index-v2/report.md`:

| eixo | p50 | p95 | vs alvo 10 ms |
|---|---:|---:|---|
| keyword / FTS5 BM25 | 3.242 ms | 3.803 ms | dentro |
| manual_cosine | 239.420 ms | 249.585 ms | 25x acima |
| hybrid / BM25 + cosine + RRF | 250.357 ms | 261.377 ms | 26x acima |
| vec0_ideal (sem filtros) | 30.007 ms | 30.315 ms | 3.0x acima |
| vec0_postfilter (com filtros) | 35.128 ms | 35.528 ms | 3.6x acima |

Completude do postfilter: `returned@N = 9/9/9 de 10`, `underfilled = 100/100`.
**Todas as 100 iterações devolveram 9 resultados, não 10.** A latência de
35.5 ms p95 do postfilter é portanto um piso otimista — mede um resultado
truncado. O número para entregar 10/10 (com over-fetch maior) só pode ser igual
ou pior.

Outros sinais:

- FTS rebuild: 238 ms.
- FTS drift after rebuild: 0.
- Delete visibility lag: 117 us, 1 check.
- Disk loaded: 717.29 MiB for 100K memories with 384d TF-IDF embeddings + a
  parallel `vec0` table (vs 559 MiB sem a vec0 — a tabela vetorial sozinha
  adiciona ~158 MiB / 100K, sinal concreto de que disk growth é um eixo pesado
  para qualquer índice vetorial).

Escala O(n) confirmada: 1K smoke mostrou cosine ~4.5 ms; 100K mostrou ~245 ms.
Projeção linear para 1M: manual_cosine ~2.5 s, vec0_postfilter ~355 ms — ambos
muito acima do alvo de 10 ms.

Caveat de spike (honestidade): o `underfilled = 100/100` também pode indicar que
o over-fetch default de 10x é pequeno para a seletividade do corpus sintético
(~60% filtrado). Isso é uma limitação do spike, não prova isolada contra o
`sqlite-vec`. Mas como o veredito de latência já reprova mesmo no caso otimista
(9 de 10), a limitação não afeta a conclusão — só impede exagerar a força da
evidência.

## Evidência no código

- `src/search/hybrid.rs` implements `semantic_only_search` by selecting all
  candidate memories with `has_embedding = 1`, loading each embedding via
  `get_embedding`, and computing cosine in Rust.
- `src/embedding/mod.rs` provides the Rust `cosine_similarity` loop used by
  this path.
- `src/storage/migrations.rs` creates a regular `embeddings` table with
  `embedding BLOB`, not a `vec0` virtual table.
- `Cargo.toml` includes `sqlite-vec = "0.1"`.
- `cargo tree -i sqlite-vec` confirms `sqlite-vec v0.1.6` is linked into
  `engram-core`.
- Repository search found no production usage of `sqlite_vec::sqlite3_vec_init`,
  `sqlite3_auto_extension`, `vec0`, or a KNN-style `MATCH ... ORDER BY distance`
  query.
- The local `sqlite-vec` crate exposes `sqlite3_vec_init`; its own test registers
  the extension with `sqlite3_auto_extension` and validates `vec_version()`.

## Documentation drift

Existing docs claim vector search is backed by `sqlite-vec`:

- `docs/ROADMAP.md` says hybrid search is BM25 + vector (`sqlite-vec`) + fuzzy +
  RRF.
- `docs/SCHEMA.md` says the schema supports vector embeddings via `sqlite-vec`.
- `docs/AI_GUIDE.md` says vector similarity is semantic embeddings via
  `sqlite-vec`.

The implementation currently stores embeddings in SQLite but does not use a
`sqlite-vec` vector table or vector-distance query in the hot search path.

## Decision

**1. Tantivy permanece descartado.** O problema de latência medido não é lexical:
FTS5/BM25 está em 3.8 ms p95 @ 100K, dentro do alvo. O gargalo é o caminho
vetorial (cosine manual 250 ms p95).

**2. `sqlite-vec` brute-force NÃO vai para produção — reprovado por dado.** O spike
mediu `vec0_postfilter` em 35.5 ms p95 @ 100K (3.6x acima do alvo), e mesmo esse
número é otimista porque devolveu 9 de 10 resultados (`underfilled = 100/100`).
Projeção linear para 1M: ~355 ms. É otimização de constante (~7x sobre cosine
manual), não solução de classe. Confirma a calibração abaixo: reduz constante,
não muda O(n).

**3. Drift de documentação a corrigir** (separado desta decisão de arquitetura):
`ROADMAP`/`SCHEMA`/`AI_GUIDE` afirmam que o vector search usa `sqlite-vec`; o hot
path usa cosine manual. Corrigir os docs OU conectar o path — não deixar ambos.

**4. Próximo passo: spike de ANN/vector-index real** (HNSW ou equivalente), que
precisa provar os cinco eixos da RFC 0003 explicitamente — latência (100K +
projeção 1M), rebuild time, **delete propagation física** (não só visibilidade
lógica — é o eixo do incidente Chroma/HNSW), disk growth (baseline: a vec0 table
sozinha já adicionou ~158 MiB / 100K), e health/drift/rebuild/drop-recreate.

Calibration: `sqlite-vec` 0.1.x is not an ANN index. Its `vec0` KNN path is
still brute-force O(n); it can reduce the constant factor by doing distance
calculation in C/SIMD inside SQLite and avoiding Rust heap materialization of
every embedding BLOB, but it does not change the scaling class. The acceptance
criterion for the spike is projected 1M-query latency inside the 10 ms target,
not merely speedup over the current 543 ms / 100K baseline. If `sqlite-vec`
brute force does not meet that bar, the decision escalates to a real ANN/vector
index option with delete propagation and disk growth measured explicitly.

Any vector-index change must keep RFC 0003 guardrails:

- SQLite memory rows remain canonical.
- Vector index is rebuildable, health-checkable, drift-detectable, and
  disposable.
- Delete propagation must be measured separately from logical search visibility.
- Disk growth must be measured against the 559.15 MiB / 100K baseline.

## Spike bench-only instrumentado

`benches/search.rs` now includes a focused implementation spike for
`semantic_only_search` vs `sqlite-vec`. The report measures three latency lines
side by side:

- `manual_cosine`: current Rust cosine path.
- `vec0_ideal`: `sqlite-vec` KNN without production filters, useful only as a
  best-case constant-factor measurement.
- `vec0_postfilter`: `sqlite-vec` KNN with over-fetch, default search filters
  (`valid_to`, `expires_at`, `transcript_chunk`, `archived`, workspace), and
  truncation to the requested limit. This is the acceptance line.

Bench-only implementation details:

- register `sqlite-vec` with `sqlite3_auto_extension` once before opening the
  benchmark database,
- run a binding-correctness gate before measurement by inserting known vectors
  into a disposable `vec0` table and checking nearest-neighbor agreement with
  production `cosine_similarity`,
- create `bench_vec` as a disposable `vec0` table with `rowid = memories.id`,
- populate it in parallel with the canonical `embeddings` table,
- and update `target/criterion/search-index-v2/report.md` with the three
  latency lines.

Smoke validation was run with:

```bash
ENGRAM_SEARCH_BENCH_REPORT=1 \
ENGRAM_SEARCH_BENCH_REPORT_SIZES=1000 \
ENGRAM_SEARCH_BENCH_EMBEDDING_DIMS=384 \
ENGRAM_SEARCH_BENCH_REPORT_ITERS=2 \
cargo bench --bench search search_index_v2_report
```

Smoke result:

| memories | manual_cosine p95 | vec0_ideal p95 | vec0_postfilter p95 |
|---:|---:|---:|---:|
| 1K | 1.982 ms | 0.381 ms | 0.332 ms |

These smoke numbers only validate wiring and binding correctness. They are not
decision-grade because `ENGRAM_SEARCH_BENCH_REPORT_ITERS=2`.

Decision-grade spike was run with:

```bash
ENGRAM_SEARCH_BENCH_REPORT=1 \
ENGRAM_SEARCH_BENCH_REPORT_SIZES=100000 \
ENGRAM_SEARCH_BENCH_EMBEDDING_DIMS=384 \
ENGRAM_SEARCH_BENCH_REPORT_ITERS=100 \
cargo bench --bench search search_index_v2_report
```

Decision-grade result:

| memories | keyword p95 | hybrid p95 | manual_cosine p95 | vec0_ideal p95 | vec0_postfilter p95 |
|---:|---:|---:|---:|---:|---:|
| 100K | 5.251 ms | 246.488 ms | 245.378 ms | 31.487 ms | 36.764 ms |

Other measured axes:

- Rebuild time: 186 ms.
- FTS drift after rebuild: 0.
- Delete visibility lag: 76 us, 1 check.
- Disk loaded: 717.27 MiB with canonical embeddings plus disposable `bench_vec`.

Interpretation:

- `sqlite-vec` improves the vector constant factor by about 6.7x against the
  filtered manual-cosine baseline (`245.378 ms / 36.764 ms`).
- It still misses the 10 ms target at 100K by about 3.7x.
- Linear projection to 1M is about 368 ms p95 for `vec0_postfilter`, or about
  36.8x over the 10 ms target.
- Therefore `sqlite-vec` 0.1.x does not satisfy the acceptance criterion for
  production vector search. It remains useful as evidence that moving distance
  calculation into SQLite/C reduces overhead, but it does not solve the O(n)
  scaling class.

## Next action

No production migration, schema version bump, or health contract should be added
for `sqlite-vec` brute-force. Escalate to an ANN/vector-index candidate with
delete propagation and disk growth measured explicitly.
