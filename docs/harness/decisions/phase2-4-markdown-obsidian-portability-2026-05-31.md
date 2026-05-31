# Fase 2.4 — Decisão #32: Portabilidade Markdown/Obsidian

**Data:** 2026-05-31  
**Issue:** #32  
**RFC:** [RFC 0004](../rfcs/0004-markdown-obsidian-portability.md)

## Decisão

**Option C aceita:** Engram como fonte canônica; Obsidian como espelho legível por humanos; import sempre em review mode com diff explícito antes de aplicar.

## Frontmatter canônico

Campos obrigatórios prefixados com `engram_`:
- `engram_id`, `engram_workspace`, `engram_scope`, `engram_type`
- `engram_created_at`, `engram_updated_at` (RFC 3339 / UTC)
- `engram_content_hash` (sha256) — gate anti-overwrite silencioso
- `engram_version` — detecta conflito DB-newer vs file-newer
- `engram_importance`, `engram_tags`, `engram_tier`
- `engram_source_session` (opcional)

## O que é lossy (não faz round-trip)

`embedding`, `access_count`, `last_accessed_at`, `expires_at`, `procedure_*_count`, `summary_of_id`, `lifecycle_state`, `metadata` (exceto source_session), `owner_id`, `visibility`.

## Contrato de import

| Condição | Ação |
|---|---|
| ID não encontrado no DB | Staged como nova memória (requer `--confirm`) |
| content_hash igual ao DB | No-op |
| hash diferente, versão igual | Staged como update pendente |
| versão conflitante | Bloqueado — requer `--force-version` |

## Status

Decisão formalizada. Implementação (CLI export/import) é trabalho separado — não bloqueada por esta issue.
