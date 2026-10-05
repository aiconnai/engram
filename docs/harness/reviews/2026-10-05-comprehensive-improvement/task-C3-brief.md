### C3 — Unicode e parsers adversos [P1; search/intelligence; depende Q2]

**Files:** `tests/property_tests.rs`, `tests/aaak_compression_tests.rs`, `src/search/`, módulos de normalização/compressão identificados pelo inventário.

- [ ] Casos explícitos `ß/ẞ`, İ, combining marks, emoji/ZWJ, bidi/control, empty e payload no limite. Property strategy baseada em chars inclui categoria C; verificar nunca panic e preservação do contrato, não só validade UTF-8.
- [ ] Regressão explícita de `src/search/bm25.rs::generate_highlights` e caller BM25: query `target`, conteúdo `"x".repeat(100) + "ẞ" + "a".repeat(30) + "target"`. Hoje `find` usa texto lowercased e slices usam original; teste exige ausência de panic e snippet original correto, incluindo limites UTF-8. Observação de fonte, não execução nesta revisão.
- [ ] Corrigir apenas slices cuja origem de offset é incompatível; teste compara resultado e limites antes de qualquer split de módulo.
- [ ] Rodar suites property/compression; lane ampliada tem seed e reproducers persistidos, sem rede/modelos.

**Aceite:** regressões determinísticas e property cases efetivamente executados; strings originais não fatiadas por offsets normalizados. **Rollback:** revert patch sem descartar o reproducer.
