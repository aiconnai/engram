### Q6 — Dependências e tempo de build pelo grafo [P2; core/CI; depende Q2/Q5]

**Files:** `Cargo.toml`, `Cargo.lock`, `deny.toml`, `.cargo/audit.toml`, `docs/security/advisory-exceptions.toml`, `governance/exceptions.toml`, toolchain/config existentes.

- [ ] Inventariar duplicates/reverse deps em duas execuções `rtk cargo tree -d --locked --no-default-features --features <lista-resolvida>`: listas de `scripts/ci-required-features.env` e `scripts/ci-features.env`, cada argv/lista/hash anexados. Não usar default `openai` como grafo full/required. Medir compile/link incremental sem cargo clean. Mapear exceções entre manifests e exigir owner/rationale/expiry coerentes.
- [ ] Um upgrade por PR, manifest+lock juntos; verificar release notes atuais antes de selecionar versão. Não remover async-trait com dyn; não prometer que root rand/base64 elimina transitivos. Feature vazia pode ser contrato e não lixo.
- [ ] Rodar required matrix, backend-smoke/full-feature-check quando aplicável, deny/audit em ambiente permitido. Manter perfis já otimizados até comparação mensurável.

**Aceite:** risco real reduzido ou ganho demonstrado sem feature/API regression; duplicatas remanescentes explicadas. **Rollback:** manifest+lock/config juntos.
