# Turso/libSQL sunset — plano e progresso

## Mandato, desenho existente e limites

- Owner: Ronaldo, chat de 2026-10-23: executar o sunset recomendado com máximo paralelismo possível.
- Base: `b29cd16c205c79746c904d030b41030c1e69f606`; branch de integração: `codex/turso-sunset`.
- Receita aceita: [ADR-STORAGE-20260816-1](../../adr/2026-08-16-turso-backend-deprecation-and-sunset.md), remoção completa na v0.24.0.
- Método: `parallel-model-orchestration` 1.9.0; execução delimitada da receita, sem redesenho arquitetural.
- Fora de escopo: migração de dados reais/conversor, schema SQLite, Dream, publicação/tag, push/PR/merge, chamadas de triagem pagas, limpeza de worktrees existentes.
- Isolamento: worktrees próprias `turso-sunset`, `turso-sunset-docs`, `turso-sunset-ci`; ownership exclusivo, sem workers executarem Cargo. Cache compartilhado preservado, nenhum `cargo clean`.
- Registro operacional: `/tmp/engram-turso-sunset-20261023/run.json` (local, sanitizado; não é aprovação confiável).

## Entregas e dependências

| Entrega | Dono | Aceite | Estado |
|---|---|---|---|
| Consumidores e cobertura | Sol high, leitura | mapa de consumidores, riscos e cobertura genérica preservada | verificado |
| Cargo/Rust/testes | integrador | feature/backend/libsql ausentes; SQLite e CI passam | integrado; sensores completos PASS |
| Documentação/migração | Sol high, worktree docs | instruções verificadas, breaking change explícito, histórico preservado | integrado e verificado |
| CI/exceções/inventário | Sol high, worktree CI | feature removida; só exceções sem dependência sobrevivente retiradas | integrado e verificado |
| Revisão do core | Sol high, leitura | diff congelado Cargo/Rust sem regressão demonstrada | sem achados; estática |
| Integração e gates | integrador | sensores completos + contratos + dependências + revisão do candidato | implementação verificada; post-gate PENDING por receipt humano |

Os três primeiros delegados foram despachados sem barreira de lote (limite nativo: quatro agentes incluindo integrador). Após reconciliação, a vaga foi reutilizada para revisão independente. Builds pesados são seriais por recurso compartilhado.

## Reconciliação comprovada

- Não há consumidor Turso nos SDKs, CLI/MCP ou configuração do servidor. Consumidores internos: exports de storage, `full`, lista CI compartilhada, inventário de contratos e documentação ativa.
- Busca externa limitada a 17 manifests de topo em projetos irmãos: `engram-cloud` depende de Engram sem pedir Turso. Consumidores remotos, deployments e manifests aninhados não foram verificados.
- O backend antigo grava a versão canônica apesar de manter schema manual incompleto. Abrir uma réplica Turso como SQLite Engram não constitui migração validada; destino novo e transferência lógica controlada são necessários. Import/export genérico não preserva todo o estado nem os IDs.
- A asserção de que `TransactionalBackend::with_transaction` rejeitado não executa a closure foi preservada no teste SQLite existente. CRUD, savepoints e saúde já têm cobertura SQLite; falhas de sync exclusivas de Turso são retiradas com o backend.
- Lockfile: 56 pacotes removidos; nenhum pacote/versão adicionado ou atualizado. A árvore `--all-features` resolve offline sem libSQL.

## Validação e evidência

- Bootstrap + doctor da sessão: exit 0. Pre-gate: `ADVISORY` (não aprovação).
- `vc-gate start`: primeira execução detectou apenas artefato criado pelo próprio pre-gate; repetido com atribuição explícita à task (`--allow-dirty-current-issue`), exit 0.
- Manifesto Cargo offline: feature `turso`, dependência `libsql` e target de teste Turso ausentes.
- CI worker: paridade CI (17 testes), política de exceções (6 testes e negativos expired/missing-owner), doctor e diff check passaram. Verificação integrada da política: PASS (2 registros preservados).
- Inventário global anunciado: 23 falhas preexistentes; base e candidato têm stdout/stderr byte a byte iguais. Paridade da seção Cargo: PASS, 32 features em ordem exata, sem `turso`/`deprecated-turso`. Evidência externa e hashes conferidos pelo integrador.
- Audit sem fetch: exit 0 em base e candidato, DB local de 1.314 advisories e warning PDF preexistente. `cargo deny check --disable-fetch advisories` usa seu DB atual separado: 11 ocorrências na base, 10 no candidato, nenhuma nova. A retirada de `rustls-webpki 0.102.8` elimina também `RUSTSEC-2026-0382`; demais achados atuais de dependências sobreviventes ficam fora deste sunset. Não confundir audit verde em DB local com deny verde.
- Primeira execução de sensores completos: FAIL somente em três assertions dos links de retomada que o integrador omitiu no novo `progress.md`; todos os demais componentes da lane passaram. Links corrigidos, teste de contexto repetido: 49/49 PASS (12 s). Gate completo repetido sem exclusões; logs originais preservados no diretório da execução.
- Segunda execução completa: **PASS**, exit 0, `status=pass ci_status=pass doctor_status=pass mode=full`, timestamp `2026-10-23T20:10:00Z`, 310 s, sem exclusões. Inclui fmt, clippy, biblioteca/integração/binários, WASM, rustdoc, referência MCP e todos os componentes offline. `.sensors-last` e as duas novas linhas append-only de `.sensors-log` preservam o resultado e a falha anterior; log integral em `/tmp/engram-turso-sunset-20261023/sensors-full-v2.log`.
- Gates executados: `CARGO_TARGET_DIR=<cache existente> bash docs/harness/bin/sensors.sh` sem exclusões; inventário de contratos; audit sem fetch; deny; diff check. Resultados e falhas preexistentes registrados acima. Doctor e check-live-state estrito finais: PASS; suíte de contexto após atualização dos metadados: 49/49 PASS.
- Revisão canônica: Claude Code Sonnet em processo separado quando a via autenticada estiver disponível; o receipt do operador humano continua obrigatório para `review-gate.sh post` autoritativo. Não fabricar receipt nem atribuir aprovação do writer.
- `lazycodex-ai doctor` via `npx --no-install` indisponível por pacote ausente; nada instalado. Execução segue por agentes nativos explicitamente autorizados.
- Revisão Sonnet v1: `claude-sonnet-5-5` reportado pelo cliente oficial, sem achados substantivos; marcador duplicado torna o formato inválido para o post-gate. Original preservado sem edição; solicitada correção delimitada com evidência nova. Bytes do produto/config/docs fora de harness continuam idênticos ao packet revisado (`sha256 90e685ad78cbd09dce3a395009607e3f22344a67eb4ce2b9262bc51f789c7414`).
- Revisão Sonnet v2: [artefato bruto](../reviews/2026-10-23-turso-sunset-v2-post.md), exatamente um marcador, opinião PASS sem achados substantivos; baseline do inventário e deny confirmados. O reviewer leu o segundo gate antes de seu término; o **integrador** confirmou o exit 0 terminal depois. O reviewer não executou testes nem revalidou hashes: essa limitação está preservada no parecer. Opinião não equivale ao post-gate autoritativo; não há receipt humano.
- Metadados finais: `check-live-state.sh` continua exigindo como primeira referência o review histórico de `engram-10-of-10-live-state`; essa autoridade foi preservada e o novo parecer registrado em separado, sem alterar o checker. As verificações finais de doctor/contexto conferem os metadados atualizados. `cargo check --offline --locked --features turso` rejeita a feature retirada (exit 101 esperado); `cargo metadata --no-deps` isoladamente não valida pedidos de feature, portanto não foi aceito como esse teste negativo.

## Uso de IA observado

| Via | Tarefa | Modelo solicitado → reportado | Effort solicitado → reportado | Agentes |
|---|---|---|---|---:|
| Codex nativo | consumidores/core review | `gpt-6.1-sol` → não observado | high → não observado | 1 |
| Codex nativo | documentação | `gpt-6.1-sol` → não observado | high → não observado | 1 |
| Codex nativo | CI/segurança | `gpt-6.1-sol` → não observado | high → não observado | 1 |
| Codex atual | integração/gates | não observado | não observado | 1 |
| Claude Code MBRAS | revisão integrada + correção de formato | `sonnet` → `claude-sonnet-5-5` (ambos) | high configurado → não reportado independentemente | 2 processos concluídos, 55 turnos CLI reportados |

Métricas nativas não expostas; não estimadas como zero. Subtotal observado **somente dos dois processos Sonnet**, calculado de JSONs distintos: 44 tokens de entrada, 15.443 de saída (3.603 de raciocínio incluídos), 132.550 de criação de cache e 977.287 de leitura de cache. Estimativa do cliente pela tabela de lista: US$0,7824467; autenticação `claude.ai` de assinatura, não recibo financeiro ou custo combinado de toda a execução. Não somar contadores incluídos novamente. Nenhuma chamada à CLI paga de decision-triage nem API LLM direta.

## Recuperação

Mudança isolada na branch local. Reverter a futura entrega restaura backend e dependências; não há migração de schema/dados para reverter. Worktrees antigas e dados de usuário não são tocados. Não publicar a release enquanto o boundary de revisão/compatibilidade aplicável estiver pendente.
