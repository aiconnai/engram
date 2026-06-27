Tenho todas as informações necessárias para o veredito completo. Aqui está minha revisão independente:

---

```
REVIEW_VERDICT: PASS Spec is complete and safe to turn into an implementation plan — all v2 blockers resolved; findings below are MED/LOW only.
```

---

## Inventário de escritores verificado contra código

Executado: `rg -n "SET lifecycle_state|UPDATE memories SET lifecycle_state|update_memory_lifecycle_state\(|compress_old_memories\(|auto_delete_after_days|valid_to = .*archived" src/`

| Site verificado | Classificação do spec | Concordância |
|---|---|---|
| `lifecycle.rs:178,184` → `lifecycle_run` (stale/archived) | decay engine convergente | **AGREE** |
| `salience.rs:450` → `run_salience_decay` (stale/archived) | decay engine convergente → disarmar | **AGREE** |
| `memory_policy.rs:146` → `memory_decay` (lifecycle_target) | decay engine convergente → disarmar | **AGREE** |
| `summarize.rs:329` → `memory_archive_old` (archived) | decay engine convergente → disarmar | **AGREE** |
| `retention.rs:312` → `compress_old_memories` (archived) | decay engine convergente → disarmar | **AGREE** |
| `server.rs:747` → scheduler chama `compress_old_memories` | compressor scheduler opcional → disarmar | **AGREE** — v2 blocker resolvido |
| `consolidation_offline.rs:568` → archive em consolidação | domain writer, coexiste | **AGREE** |
| `retention.rs:182` → max-count archive | domain writer, coexiste | **AGREE** |
| `retention.rs:204` → auto-delete (valid_to, created_at) | domain writer, coexiste | **AGREE** |
| `context_quality.rs:730,737` → conflito KeepA/KeepB | domain writer, coexiste | **AGREE** |
| `dream.rs:377` → dream action apply | domain writer, coexiste | **AGREE** |
| `lifecycle.rs:239` → `memory_set_lifecycle` manual | domain writer, coexiste | **AGREE** |
| `storage/queries/lifecycle.rs:28,39` → helper `update_memory_lifecycle_state` | helper/camada de query | **AGREE** |
| `storage/queries/tests.rs:1543` | fixture de teste | **AGREE** |

**Enumeração: COMPLETA.** Nenhum escritor adicional encontrado além dos listados pelo spec.

---

## Findings

- **[MED] `lifecycle_run` atual usa `created_at` não `last_accessed_at` para stale — diverge do modelo híbrido especificado.**
  `lifecycle.rs:94-95` usa `created_at < stale_cutoff` como predicado de seleção. O spec especifica `idle_days = (now − last_accessed_at.unwrap_or(created_at))` como métrica primária (decisão 3). A implementação atual é exatamente o predicado divergente que o spec pretende substituir. Isso é correto como *gap a corrigir na implementação*, mas o spec poderia apontar explicitamente que `lifecycle.rs:94-113` (o SQL atual de `lifecycle_run`) é removido/substituído — a seção "What is removed" menciona apenas o predicado `importance < X AND access_count < 5`, sem citar a troca de `created_at` → `last_accessed_at` no pre-filter. **Não é bloqueador** (é o ponto central do spec), mas pode causar confusão ao implementador.

- **[MED] `run_salience_decay` escreve `lifecycle_state` via SQL inline (não via `update_memory_lifecycle_state`) — `salience.rs:449-452`.**
  A seção "What is removed" cita `salience.rs:~439-460`. Verificado: `salience.rs:449-452` faz `UPDATE memories SET lifecycle_state = ?, updated_at = ?` diretamente em SQL, não usa o helper de query. A spec diz "remove `salience.rs:~439-460` state block". Correto e preciso. A anotação `~` (aproximado) é aceitável dado que o bloco cobre 439-460 com a lógica de `new_state` em 439-445 e o execute em 449-452.

- **[MED] `memory_archive_old` não filtra por `lifecycle_state` — seleciona **ativas** por age/importance/access.**
  `summarize.rs:261-269`: a seleção em `list_memories` não filtra `lifecycle_state`; depois `create_memory` + `UPDATE SET lifecycle_state = 'archived'` em `summarize.rs:329`. O spec exige (corretamente, test 7) que após a mudança o candidato selection filtre apenas rows já `Archived` — e alerta: "simplesmente deletar o UPDATE final é insuficiente". Isso está capturado e é preciso.

- **[LOW] `compress_old_memories` usa `COALESCE(lifecycle_state, 'active') = 'active'` como filtro explícito (`retention.rs:241`), selecionando apenas ativas — confirma que é um lifecycle writer ativo.**
  O spec afirma isso corretamente. O scheduler em `server.rs:751` ainda usa a palavra "archived" no log (`"Compression scheduler archived {} memories"`). O spec exige que wording "archived" seja trocado para "compressed" — correto e verificável.

- **[LOW] `suggest_lifecycle_state` (`salience.rs:254-278`) usa predicado divergente duplo (score < 0.2 AND days_inactive ≥ threshold; score < 0.4 OR days_inactive ≥ stale_threshold).**
  Verificado: diverge completamente do predicado híbrido proposto. O spec escolhe compatibilidade: manter o campo `suggested_state` mas delegar a `decide_lifecycle_state`. Correto; os testes em `salience.rs:971-981` que verificam `suggest_lifecycle_state` diretamente precisarão ser reescritos (test 9 cobre isso).

- **[LOW] Invariante de monotonicity: `active → archived` direto sem `lifecycle_changed_at`.**
  O spec aceita isso explicitamente (Invariant 2). O `update_memory_lifecycle_state` helper (`lifecycle.rs:28-68`) não verifica monotonicidade — alguém pode chamar `memory_set_lifecycle` para regredir `archived → active`. O spec delimita: regressão é explícita e out-of-scope. Documentado claramente na Invariant 2. Adequado.

- **[LOW] `auto_delete_after_days` semântica de criação (não de "desde archived").**
  Verificado em `retention.rs:199-207`: `created_at < cutoff` — confirma que é criação, não tempo desde archival. O spec aceita isso explicitamente (Invariant 5 + test 10) com aviso a operadores. A consequência (mais rows archived = mais candidatos a soft-delete imediato) está documentada. Adequado como boundary aceito sem bloqueio.

- **[LOW] Cap dormant sob defaults: `archive_base 90 × max_mult 4.0 = 360 < cap 365`.**
  Matemática verificada: correto. A tabela de testes inclui caso "Cap FORCED" com `cap=300 < eff_arch=360`, idle=320 → `Archived` — prova que o cap domina quando configurado abaixo do effective window. Adequado.

- **[LOW] `lifecycle_run.min_importance` aparece em `lifecycle.rs:89-92` como parâmetro real lido e usado como filtro SQL (`AND importance < ?`).**
  O spec exige que seja depreciado/no-op ou traduzido apenas para reporting. Isso está capturado na seção "Public contract migration". Verificado que está no código como filtro real — o spec identifica corretamente que precisa ser removido como filtro de candidatos.

---

**Resumo executivo:** O bloqueador da re-review v2 (scheduler opcional como lifecycle writer oculto) está explicitamente endereçado e verificado contra `server.rs:726-749` e `retention.rs:237-312`. A enumeração de writers está completa e correta. As decisões de disarm são precisas, o pre-filter invariant está correto, e o zero-migration está bem fundamentado. O spec é seguro para virar plano de implementação.
