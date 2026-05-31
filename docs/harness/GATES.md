# Gates — Sensores, Thresholds e Critérios (Engram)

Três camadas de verificação:

1. **Sensores determinísticos** (`sensors.sh`) — locais, rápidos, reproduzíveis.
2. **Review gate cross-CLI/cross-model** (`review-gate.sh`) — julgamento independente.
3. **Checklist humano em PR** — itens que nenhum gate automatizado cobre sozinho.

## Camada 1 — Sensores Determinísticos

Wrapper principal: `bash docs/harness/bin/sensors.sh`

Ele executa (em ordem):

| # | Sensor | Comando / Threshold | Action on FAIL |
|---|--------|---------------------|----------------|
| 1 | fmt | `cargo fmt --all -- --check` (exit 0) | block; rodar `cargo fmt --all` |
| 2 | clippy | `cargo clippy --all-targets --all-features -- -D warnings` | block; fix warnings |
| 3 | test (paridade) | `just ci` (preferencial) ou `make ci` (outra camada equivalente), com lib + integration e CI_FEATURES | block; investigar flakiness ou feature drift |
| 4 | docs + MCP ref | `./scripts/generate-mcp-reference.sh --check && RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --document-private-items` | block; atualizar referência ou docs |
| 5 | harness doctor | `bash docs/harness/bin/doctor.sh` | block; corrigir drift no harness |
| 6 | (opcional/extensível) | snapshot tests, property tests, embedding cache bounds, etc. | block conforme threshold |

O script `sensors.sh` grava o resultado parseável em `docs/harness/.sensors-last` (status, timestamp, exclusões, etc.).

### Exclusões Documentadas (Contrato Rigoroso)

Exclusão só existe para **dependências externas temporárias** (ex.: API de embedding paga indisponível, serviço de terceiros em outage).

Contrato mínimo:

- Apenas sensores específicos (hoje: possivelmente embedding-related ou watcher integration que exige ambiente GUI).
- `--exclude-sensor <name>`
- `--known-issue docs/harness/known-issues/YYYY-MM-DD-<slug>.md` (arquivo deve existir)
- `--reason "motivo curto e específico"`
- Registro **prévio** do known-issue + razão exata em `progress.md` e no active plan log.
- Fechamento de código de produção **exige** run limpo real (sem exclusão) a menos que ADR ou governança equivalente autorize.

`sensors.sh` bloqueia exclusão se a evidência de registro não existir.

Exemplo:

```bash
bash docs/harness/bin/sensors.sh \
  --exclude-sensor embedding-api-smoke \
  --known-issue docs/harness/known-issues/2026-05-30-cohere-outage.md \
  --reason "Cohere API outage; known issue registrado"
```

Isso grava `status=pass_with_exclusion` em `.sensors-last`. Não é evidência suficiente para merge de produção.

### Fake-Success Patterns (Específicos de Engram)

O review-gate é prompted explicitamente para caçar estes (sensores verdes mas sistema quebrado em produção ou para agentes):

1. **Tests passam só com `local-embeddings` mas CI Linux usa features remotas** — ONNX ou embedding provider ausente em CI, mas `cargo test --features local-embeddings` passa localmente.
2. **MCP protocol tests usam fixtures antigas** — `tests/mcp_protocol_tests.rs` ou golden files não cobrem nova tool ou mudança de schema de request/response.
3. **Schema version atualizada mas testes de migração hardcoded falham** — `storage/migrations.rs` bump + alguns testes em `tests/` ou `src/storage/` ainda têm versão antiga.
4. **Clippy limpo + `unwrap()` em path quente de MCP handler** — allowlist de clippy esconde o problema; handler de tool crítica pode panic em input adverso.
5. **Snapshot tests verdes mas attestation/Merkle mudou** — `src/snapshot/` ou `tests/snapshot_attestation.rs` não reflete mudança em crypto ou serialization.
6. **Hooks (session_end etc.) não testados em integração** — comportamento de consolidação/auto-tag muda, mas só unit tests isolados passam.
7. **Cargo doc passa mas MCP reference gerada está stale** — `scripts/generate-mcp-reference.sh --check` falha silenciosamente ou é pulado em "dev mode".
8. **Review gate roda contra diff que exclui harness artifacts, mas o prompt injetado está incompleto** — self-referential ou prompt drift.
9. **Rustdoc warnings tratados como allow em CI local mas -D warnings no gate** — flags diferentes produzem falso verde.
10. **Identity alias normalization ou scope grants mudam sem atualização de testes de propriedade** — property tests ou `tests/` não cobrem o novo comportamento.

O prompt do review-gate inclui esta lista + instrução para buscar evidência concreta no diff.

## Camada 2 — Review Gate

Ver `review-gate.sh` e `CODE_REVIEW_POLICY.md` para detalhes de execução e prompt.

Características chave:

- Pre: advisory (sempre 0), mas findings são obrigatórios de ler.
- Post: hard gate. `PASS <resumo>` na primeira linha ou FAIL, **e** incluir sempre a linha `REVIEW_VERDICT: PASS|FAIL ...` para parser hard.
- Continuity: após FAIL, reruns injetam `[BLOCKER]`/`[HIGH]` anteriores relevantes (com ids estáveis para dedup).
- Exclusões automáticas de diff: `docs/harness/reviews/*`, `docs/harness/progress/*`, `target/`, `coverage/`, artefatos de build, etc. (anti self-referential loop).
- Timeout configurável via `REVIEWER_TIMEOUT_SECS`.
- Suporte a múltiplos backends via `REVIEWER_CLI` (claude, grok, codex, ollama, ou "manual" que só gera o prompt file).

Formato de output esperado do reviewer (primeira linha):

```
PASS no substantive issues for harness-bootstrap
```

ou

```
FAIL 2 blockers: missing doctor integration, prompt drift in review-gate
```

Parsing hard do gate também exige:

```text
REVIEW_VERDICT: PASS ...
```
ou
```text
REVIEW_VERDICT: FAIL ...
```

## Camada 3 — Checklist Humano em PR / Commit

Itens que os gates automatizados não cobrem completamente:

- [ ] Evidência de que testes rodaram contra features/config reais de CI (não só local-embeddings).
- [ ] Se MCP surface mudou: SDKs Python/TS atualizados ou pelo menos breaking change documentado + issue aberta.
- [ ] Se storage migration ou `SCHEMA_VERSION`: evidência de que testes de migração e integração rodam limpos.
- [ ] Se mudança em hooks/intelligence/consolidation: dry-run ou evidência de que side effects foram considerados.
- [ ] Se embeddings ou cache: impacto em tamanho de binário, benchmarks ou qualidade de retrieval foi medido (quando relevante).
- [ ] `progress.md` + log da sprint atualizados com decisões e evidência de gates.
- [ ] Para mudanças de processo do harness: `doctor.sh` passou antes e depois.
- [ ] Preview/Deploy (quando aplicável): fly.io ou docker build verificado.

## Skip Allowlist (Review-Gate)

Pode pular o review-gate (camada 2) **somente** quando o diff inteiro for:

1. **Docs-only** em `docs/**/*.md`, exceto qualquer arquivo dentro de `docs/harness/` (INVARIANTS, GATES, CODE_REVIEW_POLICY, README, SPEC, scripts em bin/).
2. **Comment-only** ou doc comments (///, //!, //! ) sem mudança de comportamento.
3. **Formatting-only** (cargo fmt) sem outra alteração.
4. **Test-only additions** que não alteram produção (cobertura de path existente, sem mudança de contrato).

**Sensores (camada 1) NUNCA são pulados.**

**Nunca pular** (mesmo em diffs "pequenos"):

- Qualquer `.rs` em `src/` (especialmente mcp/handlers, storage/, hooks/, intelligence/).
- `Cargo.toml`, `Cargo.lock`, `build.rs`, `deny.toml`.
- `scripts/ci.sh`, `justfile`, `Makefile`, `.githooks/`.
- `docs/harness/**` (o próprio harness controla os gates).
- Mudanças em `sdks/python/` ou `sdks/typescript/` que afetam contrato.
- Qualquer coisa que toque MCP protocol, snapshot, attestation, ou auth.

Em dúvida: rode o review-gate.

## Integração com `just ci` / `make ci`

O sensor principal delega para `just ci` quando disponível, senão `make ci`. O contrato de paridade com GitHub permanece o mesmo. O harness adiciona:

- Harness doctor como etapa explícita.
- Review cross-CLI.
- Memória persistida (progress + reviews).
- Fake-success hunting específico de engram.

Isso mantém o "um comando para rodar tudo localmente" enquanto adiciona as camadas de harness.

---

**Princípio**: Evidence before claims. O harness existe para tornar "funcionou no meu prompt" verificável e retomável por outros agentes.
