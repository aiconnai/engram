# ONNX Local Embeddings — Design Spec

**Date:** 2026-05-11
**Status:** Approved (pending implementation)
**Tracks:** [GitHub #17](https://github.com/aiconnai/engram/issues/17)
**Feature flag:** `onnx-embed` (already exists; currently inert)

---

## Context

Engram suporta apenas dois backends de embedding hoje:

- `tfidf` — built-in, sem dependências externas, mas perde em queries parafraseadas
- `openai` — alta qualidade, mas exige API key e network call

Não existe meio-termo: embeddings semânticos reais rodando 100% local. Esse gap foi identificado na comparação com Memora (issue #17).

A feature `onnx-embed` já existe no `Cargo.toml`, com `ort` e `ndarray` como deps opcionais e o módulo `src/embedding/onnx.rs` declarado — mas o arquivo está vazio (apenas um comentário). Não há branch `"onnx"` em `create_embedder()`.

## Goal

Implementar `OnnxEmbedder` opt-in usando ONNX Runtime + modelo `all-MiniLM-L6-v2` (384 dimensões, ~22MB, 256 max tokens), distribuído via download explícito por subcomando CLI.

## Non-Goals

- GPU/CUDA execution provider
- Quantização INT8
- Múltiplos modelos simultâneos no mesmo processo
- Hot-reload de modelo em runtime
- Fallback automático silencioso quando modelo ausente
- Bundling do modelo no binary

## Decisões-chave

### D1 — Distribuição do modelo: download explícito via CLI

**Escolha:** subcomando `engram-cli model download <name>` que baixa para um cache local. Runtime carrega do disco.

**Alternativas descartadas:**
- *Download on-demand no primeiro uso* — esconde network call, complica supply chain (SHA verify, lockfiles concorrentes), quebra air-gapped
- *Path manual sem helper* — fricção sem ganho real

**Razão:** mantém runtime trivial (só lê disco), torna network call explícito (auditável), funciona offline após download inicial, compatível com Docker (`RUN engram-cli model download ...`).

### D2 — Tokenizer: HuggingFace `tokenizers` crate

**Escolha:** dep `tokenizers = "0.20"` carregando `tokenizer.json` direto do HF.

**Razão:** é o padrão de fato do ecosystem, suporta o tokenizer BERT WordPiece usado pelo MiniLM sem reimplementar.

### D3 — Pooling: mean pooling com attention mask + L2 normalize

**Escolha:** fórmula explícita `pooled[i] = Σ(hidden[t,i] * mask[t]) / Σ(mask[t])`, depois L2 normalize.

**Razão:** é a estratégia padrão recomendada pela sentence-transformers para MiniLM. CLS token pooling funciona pior empiricamente.

### D4 — Naming: `minilm-l6-v2` (não `all-MiniLM-L6-v2`)

**Escolha:** kebab-case minúsculo no registry CLI e no diretório de cache.

**Razão:** consistência com convenção do CLI; usuário não precisa lembrar de capitalização.

### D5 — Env var dedicada: `ENGRAM_ONNX_MODEL_DIR`

**Escolha:** env var específica do backend, não genérica.

**Razão:** futuros backends locais (BGE, GTE, etc.) precisam de path próprio. `ENGRAM_MODEL_PATH` genérica colide.

### D6 — Resolução de path

Prioridade decrescente:
1. `EmbeddingConfig.model_path` (programático)
2. `ENGRAM_ONNX_MODEL_DIR` (env)
3. `dirs::data_dir().join("engram/models/minilm-l6-v2")` (default cross-platform)

### D7 — Concorrência: single shared `Arc<OnnxEmbedder>`

`ort::Session` é `Send + Sync` na v2.0, embora inference serialize internamente. Para o MVP isso é aceitável — embeddings são gerados em batch async pela `EmbeddingQueue` existente, não no hot path de cada request.

## Arquitetura

### Componentes a criar/alterar

```
src/embedding/
├── mod.rs              [EDIT]   adicionar branch "onnx" em create_embedder()
├── onnx.rs             [WRITE]  OnnxEmbedder + impl Embedder
└── onnx_registry.rs    [NEW]    REGISTRY: &[ModelEntry] hardcoded

src/bin/
└── cli.rs              [EDIT]   adicionar Commands::Model { action }
                                 + ModelAction enum (Download, List, Path)
                                 + handler que baixa de HF + valida SHA256

Cargo.toml              [EDIT]   atualizar feature onnx-embed:
                                 + dep:tokenizers, dep:reqwest, dep:indicatif
                                 adicionar [dependencies]:
                                 + tokenizers (optional)
                                 + indicatif (optional)
```

### Estrutura `OnnxEmbedder`

```rust
pub struct OnnxEmbedder {
    session: ort::Session,
    tokenizer: tokenizers::Tokenizer,
    dimensions: usize,
    model_name: String,
    max_seq_len: usize,
}

impl OnnxEmbedder {
    pub fn from_dir(model_dir: &Path) -> Result<Self>;
    fn run_inference(&self, encodings: &[Encoding]) -> Result<Vec<Vec<f32>>>;
}

impl Embedder for OnnxEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn dimensions(&self) -> usize;
    fn model_name(&self) -> &str;
}
```

### Data Flow (single embed)

```
text
  → tokenizer.encode(text, truncation=true, max_len=256)
  → input_ids: [1, seq_len], attention_mask: [1, seq_len]
  → ort::Session.run([ids, mask, token_type_ids])
  → last_hidden_state: [1, seq_len, 384]
  → mean_pool(hidden, mask): [1, 384]
  → l2_normalize(): [384]
  → Vec<f32>
```

### Data Flow (batch)

```
texts (N)
  → tokenizer.encode_batch(texts, padding=longest, max_len=256)
  → input_ids: [N, max_len_in_batch]
  → ort::Session.run([ids, mask, types])
  → hidden: [N, max_len_in_batch, 384]
  → mean_pool: [N, 384]  (mask exclui padding)
  → l2_normalize each row
  → Vec<Vec<f32>>
```

### Registry de modelos

```rust
pub struct ModelEntry {
    pub name: &'static str,
    pub model_url: &'static str,
    pub model_sha256: &'static str,
    pub tokenizer_url: &'static str,
    pub tokenizer_sha256: &'static str,
    pub dimensions: usize,
    pub max_seq_len: usize,
}

pub const REGISTRY: &[ModelEntry] = &[
    ModelEntry {
        name: "minilm-l6-v2",
        model_url: "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/onnx/model.onnx",
        model_sha256: "<pin during implementation>",
        tokenizer_url: "https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2/resolve/main/tokenizer.json",
        tokenizer_sha256: "<pin during implementation>",
        dimensions: 384,
        max_seq_len: 256,
    },
];
```

### CLI: `model` subcommand

```
engram-cli model download <name>   # baixa, verifica SHA256, salva no cache
engram-cli model list              # lista REGISTRY + marca quais já estão baixados
engram-cli model path <name>       # imprime path local (útil para scripts)
```

Cache layout:
```
$DATA_DIR/engram/models/<name>/
├── model.onnx
└── tokenizer.json
```

`$DATA_DIR` = `dirs::data_dir()` (Linux: `~/.local/share`, macOS: `~/Library/Application Support`, Windows: `%APPDATA%`).

## Configuração

### Environment Variables

| Var | Description | Default |
|-----|-------------|---------|
| `ENGRAM_EMBEDDING_MODEL` | Set to `onnx` para usar local ONNX | (use `tfidf`) |
| `ENGRAM_ONNX_MODEL_DIR` | Override do path do modelo | `$DATA_DIR/engram/models/minilm-l6-v2` |

### Build flag

```bash
cargo build --features onnx-embed
```

## Error Handling

| Cenário | Comportamento | Código |
|---------|--------------|--------|
| Feature `onnx-embed` desligada + `model=onnx` | Erro com instrução de build | `EngramError::Config` |
| Modelo não baixado | Erro com comando exato pra baixar | `EngramError::Config` |
| SHA256 mismatch no download | Deletar arquivo parcial, abortar | `EngramError::Config` |
| Tokenizer fail (ex: encode panic) | Propaga com texto da query truncado | `EngramError::Embedding` |
| ORT inference fail | Propaga (geralmente bug interno) | `EngramError::Embedding` |
| Network fail no download | Retry 1x, depois aborta | `EngramError::Config` |
| Path do modelo aponta pra arquivo inválido | Erro claro identificando qual arquivo (model/tokenizer) falhou | `EngramError::Config` |
| Download de modelo já presente | Se SHA256 do arquivo local bate, skip; se não, sobrescreve | (sem erro) |
| Dois processos rodando `model download` simultaneamente | Sem proteção no MVP; documentar como limitação. Path-prefixar com `.tmp.<pid>` durante download evita corrupção parcial | (sem erro se sequencial) |

## Testing

### Unit tests (always run)

- `OnnxEmbedder` se compila quando feature ligada
- Branch `"onnx"` em `create_embedder()` retorna erro instructivo quando feature off
- Resolução de path: env var > config > default

### Integration tests (gated, ignored by default)

```rust
#[test]
#[ignore]
fn semantic_similarity_beats_unrelated() {
    let e = OnnxEmbedder::from_dir(&test_model_dir()).unwrap();
    let q = e.embed("how to configure authentication").unwrap();
    let synonym = e.embed("setup auth").unwrap();
    let unrelated = e.embed("recipe for chocolate cake").unwrap();
    assert!(cosine(&q, &synonym) > cosine(&q, &unrelated));
}

#[test]
#[ignore]
fn dimensions_are_384() { /* ... */ }

#[test]
#[ignore]
fn l2_normalized_output() {
    // norm of output vector ≈ 1.0
}

#[test]
#[ignore]
fn batch_matches_individual_within_epsilon() {
    // embed_batch([a, b]) ≈ [embed(a), embed(b)]
}
```

Run via:
```bash
cargo test --features onnx-embed -- --ignored
```

CI: separate job que faz download uma vez e cacheia.

## Riscos Conhecidos

| Risco | Probabilidade | Mitigação |
|-------|---------------|-----------|
| HuggingFace re-publica modelo, SHA256 diverge | Média | Documentar como atualizar; eventualmente mover para mirror próprio (S3/R2) |
| `ort` 2.0-rc12 é pre-release | Já assumido | Pinar versão exata; monitorar release stable |
| Build quebra em ARM64 macOS | Baixa | Validar localmente; documentar `ORT_DYLIB_PATH` se necessário |
| Binary cresce ~30MB com feature ligada | Esperado | Feature off por default; documentar trade-off |
| Cold start ~100-500ms ao carregar modelo | Esperado | Carregar uma vez no startup, compartilhar via `Arc` |

## Implementation Order

1. Implementar `src/embedding/onnx.rs` com `OnnxEmbedder`
2. Implementar `src/embedding/onnx_registry.rs` (SHA256 placeholders)
3. Adicionar branch em `create_embedder()`
4. Atualizar `Cargo.toml` com novas deps
5. Adicionar `Commands::Model` em `src/bin/cli.rs` + handler
6. Baixar modelo manualmente uma vez, computar SHA256 dos dois arquivos (`model.onnx` e `tokenizer.json`), substituir os placeholders `<pin during implementation>` no `onnx_registry.rs` pelos hashes reais
7. Escrever testes (unit primeiro, integration depois)
8. Documentar no README (env vars, build flag, exemplo de uso)

## Verification

Após implementação, validar:

- [ ] `cargo build` (sem feature) — passa, binary não muda
- [ ] `cargo build --features onnx-embed` — passa
- [ ] `cargo build --features full` — passa (full inclui onnx-embed)
- [ ] `cargo test` (sem feature) — passa
- [ ] `cargo test --features onnx-embed` — passa (sem `--ignored`)
- [ ] `cargo test --features onnx-embed -- --ignored` — passa (após `model download minilm-l6-v2`)
- [ ] Smoke test manual: download → server up → memory_create → memory_search com query parafraseada retorna match relevante
- [ ] Benchmark comparativo TF-IDF vs ONNX em queries reais (bonus)
