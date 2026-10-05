### Q2 — Inventário confiável de qualidade Rust [P1; core; depende E0]

**Files:** criar `docs/quality/rust-risk-inventory.md`; revisar `src/storage/connection.rs`, `src/storage/lock.rs`, `benches/search.rs` e módulos apontados por análise contextual. Tool de classificação, se necessário, entra em PR próprio com fixtures.

- [ ] Classificar unwrap/expect/unsafe/prints por cfg, produção/teste, binário/biblioteca e path alcançável; fixtures do classificador incluem raw strings, módulos aninhados, macros e comentários. Resultado desconhecido permanece unknown.
- [ ] Auditar SAFETY dos blocos reais e priorizar apenas caminhos que podem panic por input ou silenciar erro operacional. Saída CLI/protocolo não vira tracing por substituição em massa; teardown best-effort é distinto de erro de negócio.
- [ ] Para cada fix subsequente: input que reproduz a falha, assertion de erro tipado/ausência de panic, teste focado e Clippy required. Inventário sozinho não autoriza remoção/refactor.

**Aceite:** contagens reproduzíveis com método/limites; backlog de falhas concretas, não meta artificial de zero ocorrências textuais. **Rollback:** cada patch isolado.
