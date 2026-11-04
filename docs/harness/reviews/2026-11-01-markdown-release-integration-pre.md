# Engram Harness — External Reviewer Prompt

**Task**: markdown-release-integration
**Mode**: pre
**Date (UTC)**: 2026-11-01
**Scope**: prepare  base=646636f64ec97ff42996130bb2df919cdf8029a7  head=646636f64ec97ff42996130bb2df919cdf8029a7  tree=n/a
**Diff sha256**: 935d2ab2751deb4c0f139043f7eeb48e50284de5dc3906aa98cd30b532468009

## Instructions for the Reviewer

You are acting as an independent senior engineer reviewing a diff for the engram project.
You were NOT the implementer. Your job is to find real problems introduced by the change.

Read the following documents (they are the source of truth for this review):

- docs/harness/SPEC.md
- docs/harness/INVARIANTS.md (process invariants — canonical)
- docs/harness/WHAT_WE_DONT_DO.md (negative scope — no hidden expansion)
- docs/harness/GATES.md (especially the fake-success patterns section)
- docs/harness/CODE_REVIEW_POLICY.md (this policy)
- docs/harness/security/anthropic-reference-harness.md (security boundary)
- .claude/scan-extras.txt and .claude/fp-rules.txt (org-specific scan/triage tuning)
- docs/harness/README.md (workflow)
- Root INVARIANTS.md (data layer invariants for the memory system)

Then review the diff below.

Additional harness-specific requirements:
- Compare scope against docs/harness/WHAT_WE_DONT_DO.md. Flag hidden scope creep, gate weakening, or product changes bundled into harness work.
- Security boundary: flag autonomous Engram execution, implied sandboxing, credential mounts, network/egress expansion, or C/C++/ASAN pipeline import unless an ADR and explicit target contract are present.
- Tuning files: ensure .claude/scan-extras.txt and .claude/fp-rules.txt augment scan/triage behavior without weakening core INVARIANTS/GATES/POLICY or adding blanket suppressions.
- Review Canvas: if the diff is complex, verify that a matching docs/harness/canvas/YYYY-MM-DD-<task-id>.md exists and includes approaches considered, hot-path complexity, at least two edge cases, and a breakage-risk table.
- Harness script changes under docs/harness/bin/* are process-critical. Inspect shell safety, path handling, parseability, read-only guarantees, and whether the script weakens any existing gate.

## Key Fake-Success Patterns (hunt these actively)

1. Tests green only because local-embeddings feature was used; CI Linux parity fails.
2. MCP protocol / golden tests or generated reference (docs/MCP_TOOLS.md) is stale after tool changes.
3. SCHEMA_VERSION bumped in migrations.rs but hardcoded test versions not updated.
4. Clippy clean but unwrap/expect in hot MCP handler, storage, or hook paths.
5. Snapshot/attestation tests pass but Merkle or crypto behavior changed.
6. Hooks (session_end, post_tool_use, etc.) or intelligence modules changed without integration coverage.
7. Harness doctor or sensors would have caught this but were not run.
8. Progress docs (harness or active plan) not updated for a domain change.
9. Cross-SDK (python/typescript) contract drift not reflected.
10. Reviewer is being shown a self-referential or incomplete prompt (call it out).
11. Security boundary drift: static/read-only default weakened, autonomous execution implied, missing ADR/sandbox/egress/target contract, credential mounts allowed, or Anthropic C/C++/ASAN pipeline imported as default.

## Files In Scope

CHANGED_PATH=CHANGELOG.md
CHANGED_PATH=Cargo.lock
CHANGED_PATH=Cargo.toml
CHANGED_PATH=docs/MCP_TOOLS.md
CHANGED_PATH=docs/contracts/advertised-surfaces.toml
CHANGED_PATH=docs/harness/.sensors-last
CHANGED_PATH=docs/harness/.sensors-log
CHANGED_PATH=docs/harness/SPEC.md
CHANGED_PATH=docs/harness/canvas/2026-11-01-markdown-export-safety.md
CHANGED_PATH=docs/harness/canvas/2026-11-01-markdown-import-safety.md
CHANGED_PATH=docs/harness/progress.md
CHANGED_PATH=docs/harness/risk-register.yaml
CHANGED_PATH=docs/migrations/v0.24.0-markdown-export-security.md
CHANGED_PATH=docs/migrations/v0.24.0-markdown-import-security.md
CHANGED_PATH=src/mcp/handlers/markdown_export/export.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/export/filesystem.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/export/filesystem/acl.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/export/filesystem/reader.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/export/format.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/export/format_tests.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/export/handler.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/export/handler_tests.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/export/naming_tests.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/export/query.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/export/security_tests.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/frontmatter.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/import.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/import/apply.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/import/files.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/import/handler.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/import/integration_tests.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/import/lookup.rs
CHANGED_PATH=src/mcp/handlers/markdown_export/import/security_tests.rs
CHANGED_PATH=src/mcp/permission.rs
CHANGED_PATH=src/mcp/tools/catalog/admin.rs
CHANGED_PATH=src/portability/markdown.rs
CHANGED_PATH=tests/portability_permissions_routing_tests.rs
CHANGED_PATH=tests/storage_posix_lock_regression_tests.rs
CHANGED_PATH=docs/harness/canvas/2026-11-01-markdown-release-integration.md
CHANGED_PATH=sdks/typescript/node_modules
CHANGED_PATH=tests/cli_markdown_import_tests.rs

Paths changed but excluded from this diff (harness bookkeeping, not reviewed here):
EXCLUDED_PATH=docs/harness/progress/2026-10-26-remediation.md
EXCLUDED_PATH=docs/harness/progress/2026-11-01-markdown-export-safety.md
EXCLUDED_PATH=docs/harness/progress/2026-11-01-markdown-import-safety.md
EXCLUDED_PATH=docs/harness/progress/2026-11-01-markdown-portability-followup.md
EXCLUDED_PATH=docs/harness/reviews/2026-11-01-markdown-contract-addendum.txt
EXCLUDED_PATH=docs/harness/reviews/2026-11-01-markdown-export-followup-static.md
EXCLUDED_PATH=docs/harness/reviews/2026-11-01-markdown-export-followup-v1.txt
EXCLUDED_PATH=docs/harness/reviews/2026-11-01-markdown-export-followup-v2.txt
EXCLUDED_PATH=docs/harness/reviews/2026-11-01-markdown-export-safety-pre.md
EXCLUDED_PATH=docs/harness/reviews/2026-11-01-markdown-export-safety-static-v1.txt
EXCLUDED_PATH=docs/harness/reviews/2026-11-01-markdown-export-safety-static-v2.txt
EXCLUDED_PATH=docs/harness/reviews/2026-11-01-markdown-import-safety-static.md
EXCLUDED_PATH=docs/harness/reviews/2026-11-01-markdown-import-v1.txt
EXCLUDED_PATH=docs/harness/reviews/2026-11-01-markdown-import-v2.txt

## Diff Under Review

```diff
diff --git a/CHANGELOG.md b/CHANGELOG.md
index ebbbb7e342e628302b40d33d772b986a19fae358..73139144bf51b420d705fbaf4e3fb2653caebbd0 100644
--- a/CHANGELOG.md
+++ b/CHANGELOG.md
@@ -20,0 +21,2 @@ Release candidate prepared for review; tag and package publication are pending.
+- **Markdown filesystem import (RISK-0011)** — scoped writers require an administrator-configured root; unconfined reads require maintenance even during review. Bounded directory-relative reads reject links, special files and SQLite artifacts before opening regular files. Existing IDs are authorized by their stored workspace, and confirmed batches authorize/classify/apply within one transaction. Pre-commit failures roll back; completion uncertainty is explicit. See [compatibility notes](docs/migrations/v0.24.0-markdown-import-security.md).
+- **Markdown filesystem export** — `memory_export_markdown` is a write operation: scoped writers need an administrator-configured `ENGRAM_EXPORT_BASE_DIR`; unconfined destinations require maintenance privileges. Linux/macOS directory-relative writes reject unsafe components, symlinks, hard links, active SQLite artifacts and directory ACLs, and publish private temporary files by atomic rename. Other platforms fail closed. See [compatibility notes](docs/migrations/v0.24.0-markdown-export-security.md).
@@ -70,0 +73,2 @@ Release candidate prepared for review; tag and package publication are pending.
+- Markdown entity grouping safely encodes internal tags (`session:`, `category:`, `file:`, etc.) and reserved names instead of rejecting the workspace. Export uses POSIX fsync with one final strong flush per device, avoiding repeated macOS device flushes; public Markdown wrappers propagate structured errors.
+- Markdown export excludes expired memories and ineligible references, reads data/relations in one snapshot, propagates decode/query errors, sorts ties deterministically and marks incoming relations explicitly. No success is reported for an incomplete batch; atomicity is per file.
diff --git a/Cargo.lock b/Cargo.lock
index 59b9565a3bca71b235aa4b2b5615ed33b417817e..ba1fbb0425b170d5f744b3b2c2d67dd22ac57eaf 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1859,0 +1860 @@ dependencies = [
+ "rustix 1.1.3",
diff --git a/Cargo.toml b/Cargo.toml
index 417170fae509fa5dbb4bbb386c20f80195248764..916cb8add9dae8ad54da2e9182bdbd75d7d39877 100644
--- a/Cargo.toml
+++ b/Cargo.toml
@@ -303,0 +304,4 @@ duckdb = { version = "1.10506.1", features = ["bundled", "chrono"], optional = t
+[target.'cfg(any(target_os = "linux", target_os = "macos"))'.dependencies]
+# Explicit POSIX fsync (macOS std::File::sync_all uses a stronger device flush).
+rustix = { version = "1.1.3", features = ["fs"] }
+
diff --git a/docs/MCP_TOOLS.md b/docs/MCP_TOOLS.md
index 69caa0815a1b5189627784ad249c9ad61653f490..09904a83d5e896b92624eaab6e0faa881be65dae 100644
--- a/docs/MCP_TOOLS.md
+++ b/docs/MCP_TOOLS.md
@@ -210 +210 @@ Total tools: **301**
-| `memory_export_markdown` | advanced | memory.admin | always | readOnlyHint | `workspace` |
+| `memory_export_markdown` | advanced | memory.admin | always | mutating (no MCP hints) | `workspace` |
@@ -3539 +3539 @@ Create a memory from a specific Langfuse trace ID.
-Export a workspace as human-readable Markdown files with YAML frontmatter and wiki-style [[links]]. Creates one .md file per memory, organized by type in subdirectories, with an index.md overview.
+Write unexpired workspace memories as Markdown files with YAML frontmatter and directional [[links]]. Requires scoped_write within ENGRAM_EXPORT_BASE_DIR, otherwise maintenance. Output directories must be owned by the server user, not group/world writable and have no extended/default ACLs. Safe filesystem writes require Linux/macOS.
@@ -3544 +3544 @@ Export a workspace as human-readable Markdown files with YAML frontmatter and wi
-- Annotations: readOnlyHint
+- Annotations: mutating (no MCP hints)
@@ -3556 +3556 @@ Export a workspace as human-readable Markdown files with YAML frontmatter and wi
-Import memories from Markdown files with engram_ frontmatter (RFC 0004). Review mode by default (confirm: false) — returns a staged list without writing. Detects drift via content_hash and version conflicts via engram_version. Ignores non-engram_ frontmatter keys (Obsidian-safe).
+Import bounded Markdown files with engram_ frontmatter on Linux/macOS. Requires scoped_write within ENGRAM_EXPORT_BASE_DIR, otherwise maintenance, including review mode. Input directories must be server-owned, not group/world writable and have no ACLs; links and SQLite artifacts are refused. Authorizes existing IDs by their persisted workspace. Review is default; confirm uses one database transaction. Pre-commit failures roll back; commit/completion failures report an unknown outcome requiring reconciliation. Detects drift/version conflicts and ignores non-engram_ keys (Obsidian-safe). Skips hidden entries beneath the input root. Missing IDs are rejected for workspace-restricted principals; unrestricted local owners treat them as new.
diff --git a/docs/contracts/advertised-surfaces.toml b/docs/contracts/advertised-surfaces.toml
index 815ebb512f52a487408efaff9c4e2d18aa329eb0..5e108f85e477b0dbd02cc003de47f39871538dd9 100644
--- a/docs/contracts/advertised-surfaces.toml
+++ b/docs/contracts/advertised-surfaces.toml
@@ -4,2 +4,2 @@ schema_version = "advertised-surfaces-v1"
-base_sha = "4ea8eedc7f4a272bf827c243df321fd5e2fe8872"
-captured_date = "2026-10-26"
+base_sha = "9bca4efaeb07d9deff722ab50c407dfe77f02f63"
+captured_date = "2026-11-01"
@@ -57 +57 @@ http_endpoints = ["DELETE /api-keys/:id", "DELETE /v1/images/:key", "DELETE /v1/
-env_vars = ["AWS_ENDPOINT_URL", "AWS_PROFILE", "ENGRAM_CLEANUP_INTERVAL", "ENGRAM_CLOUD_ENCRYPT", "ENGRAM_CORS_ORIGINS", "ENGRAM_DB_PATH", "ENGRAM_EMBEDDING_MODEL", "ENGRAM_GRPC_API_KEY", "ENGRAM_GRPC_BIND_ADDRESS", "ENGRAM_GRPC_PORT", "ENGRAM_HNSW_ENABLED", "ENGRAM_HTTP_API_KEY", "ENGRAM_HTTP_BIND_ADDRESS", "ENGRAM_HTTP_MAX_BODY_BYTES", "ENGRAM_HTTP_PORT", "ENGRAM_HTTP_RATE_LIMIT_BURST", "ENGRAM_HTTP_RATE_LIMIT_KEY", "ENGRAM_HTTP_RATE_LIMIT_RPS", "ENGRAM_HTTP_REQUEST_TIMEOUT_MS", "ENGRAM_HTTP_TRUSTED_PROXIES", "ENGRAM_MEDIA_PUBLIC_DOMAIN", "ENGRAM_ONNX_MODEL_DIR", "ENGRAM_S3_BUCKET", "ENGRAM_STORAGE_URI", "ENGRAM_TOOL_TIER", "ENGRAM_WS_ALLOWED_ORIGINS", "ENGRAM_WS_AUTH_KEY", "ENGRAM_WS_BIND_ADDRESS", "ENGRAM_WS_MAX_CONNECTIONS", "ENGRAM_WS_MAX_MESSAGE_BYTES", "ENGRAM_WS_PORT", "ENGRAM_WS_READ_IDLE_TIMEOUT_SECONDS", "LANGFUSE_PUBLIC_KEY", "LANGFUSE_SECRET_KEY", "MEILISEARCH_API_KEY", "MEILISEARCH_INDEXER", "MEILISEARCH_SYNC_INTERVAL", "MEILISEARCH_URL", "OPENAI_API_KEY", "R2_ACCESS_KEY_ID", "R2_SECRET_ACCESS_KEY", "VISION_PROVIDER"]
+env_vars = ["AWS_ENDPOINT_URL", "AWS_PROFILE", "ENGRAM_CLEANUP_INTERVAL", "ENGRAM_CLOUD_ENCRYPT", "ENGRAM_CORS_ORIGINS", "ENGRAM_DB_PATH", "ENGRAM_EMBEDDING_MODEL", "ENGRAM_EXPORT_BASE_DIR", "ENGRAM_GRPC_API_KEY", "ENGRAM_GRPC_BIND_ADDRESS", "ENGRAM_GRPC_PORT", "ENGRAM_HNSW_ENABLED", "ENGRAM_HTTP_API_KEY", "ENGRAM_HTTP_BIND_ADDRESS", "ENGRAM_HTTP_MAX_BODY_BYTES", "ENGRAM_HTTP_PORT", "ENGRAM_HTTP_RATE_LIMIT_BURST", "ENGRAM_HTTP_RATE_LIMIT_KEY", "ENGRAM_HTTP_RATE_LIMIT_RPS", "ENGRAM_HTTP_REQUEST_TIMEOUT_MS", "ENGRAM_HTTP_TRUSTED_PROXIES", "ENGRAM_MEDIA_PUBLIC_DOMAIN", "ENGRAM_ONNX_MODEL_DIR", "ENGRAM_S3_BUCKET", "ENGRAM_STORAGE_URI", "ENGRAM_TOOL_TIER", "ENGRAM_WS_ALLOWED_ORIGINS", "ENGRAM_WS_AUTH_KEY", "ENGRAM_WS_BIND_ADDRESS", "ENGRAM_WS_MAX_CONNECTIONS", "ENGRAM_WS_MAX_MESSAGE_BYTES", "ENGRAM_WS_PORT", "ENGRAM_WS_READ_IDLE_TIMEOUT_SECONDS", "LANGFUSE_PUBLIC_KEY", "LANGFUSE_SECRET_KEY", "MEILISEARCH_API_KEY", "MEILISEARCH_INDEXER", "MEILISEARCH_SYNC_INTERVAL", "MEILISEARCH_URL", "OPENAI_API_KEY", "R2_ACCESS_KEY_ID", "R2_SECRET_ACCESS_KEY", "VISION_PROVIDER"]
diff --git a/docs/harness/.sensors-last b/docs/harness/.sensors-last
index 01be9331b81d23c4ca5778d9cdab781665170dec..5672adb841f94e4376cbbdb1f948ed0fe99c0c9b 100644
--- a/docs/harness/.sensors-last
+++ b/docs/harness/.sensors-last
@@ -5,2 +5,2 @@ mode=full
-timestamp=2026-10-23T20:10:00Z
-duration_sec=310
+timestamp=2026-11-01T16:41:04Z
+duration_sec=422
diff --git a/docs/harness/.sensors-log b/docs/harness/.sensors-log
index 8facd53fe141ce4370360ed49f598d6437ebd023..32eab7a8ae86792bf439bfc0da6738ed0242ba60 100644
--- a/docs/harness/.sensors-log
+++ b/docs/harness/.sensors-log
@@ -106,0 +107,3 @@
+{"schema_version":"sensors-log-v1","timestamp":"2026-11-01T08:45:43Z","tool":"sensors","mode":"full","status":"pass","duration_sec":467,"ci_status":"pass","doctor_status":"pass","ci_command":"fmt + clippy + test_lib + test_integration + test_integration_watch + wasm_target + wasm_all_targets + wasm_wasm_target + doc + ref_check + offline_lane + pr-title-policy + harness doctor","ci_steps":{"fmt":"pass","clippy":"pass","test_lib":"pass","test_integration":"pass","test_integration_watch":"pass","wasm_target":"pass","wasm_all_targets":"pass","wasm_wasm_target":"pass","doc":"pass","ref_check":"pass","offline_lane":"pass"},"exclusion":null,"artifacts":[{"path":"docs/harness/.sensors-last","kind":"sensors_last","format":"key_value"}]}
+{"schema_version":"sensors-log-v1","timestamp":"2026-11-01T09:02:00Z","tool":"sensors","mode":"full","status":"pass","duration_sec":515,"ci_status":"pass","doctor_status":"pass","ci_command":"fmt + clippy + test_lib + test_integration + test_integration_watch + wasm_target + wasm_all_targets + wasm_wasm_target + doc + ref_check + offline_lane + pr-title-policy + harness doctor","ci_steps":{"fmt":"pass","clippy":"pass","test_lib":"pass","test_integration":"pass","test_integration_watch":"pass","wasm_target":"pass","wasm_all_targets":"pass","wasm_wasm_target":"pass","doc":"pass","ref_check":"pass","offline_lane":"pass"},"exclusion":null,"artifacts":[{"path":"docs/harness/.sensors-last","kind":"sensors_last","format":"key_value"}]}
+{"schema_version":"sensors-log-v1","timestamp":"2026-11-01T16:41:04Z","tool":"sensors","mode":"full","status":"pass","duration_sec":422,"ci_status":"pass","doctor_status":"pass","ci_command":"fmt + clippy + test_lib + test_integration + test_integration_watch + wasm_target + wasm_all_targets + wasm_wasm_target + doc + ref_check + offline_lane + pr-title-policy + harness doctor","ci_steps":{"fmt":"pass","clippy":"pass","test_lib":"pass","test_integration":"pass","test_integration_watch":"pass","wasm_target":"pass","wasm_all_targets":"pass","wasm_wasm_target":"pass","doc":"pass","ref_check":"pass","offline_lane":"pass"},"exclusion":null,"artifacts":[{"path":"docs/harness/.sensors-last","kind":"sensors_last","format":"key_value"}]}
diff --git a/docs/harness/SPEC.md b/docs/harness/SPEC.md
index e9c9989df0bd0a645a7281c21bd5b4968ab1768c..94980d7800021ea421be68ba8cd3dce909a47d85 100644
--- a/docs/harness/SPEC.md
+++ b/docs/harness/SPEC.md
@@ -50 +49,0 @@
-## Follow-up autorizado — 2026-10-27
@@ -52,9 +50,0 @@
-A pilha #244–#262 está mesclada no main `2e84a594`. O GO antigo encerrou às 09:35Z.
-Novo GO do owner (chat): diagnosticar Nightly, corrigir defeitos confirmados e lacunas
-relevantes de testes, resolver a proposta de verify-tag e preparar release/aviso de migração;
-entregar evidências e recomendação antes de publicar ou executar mudanças operacionais.
-O [plano do follow-up](plans/2026-10-27-nightly-release-readiness.md) detalha paths e aceite.
-Permite alterações e verificações locais; não autoriza push/PR/merge, tags de release,
-publicação, envio de avisos, deleções ou settings. Os limites históricos acima referem-se
-à frente WT5 original; este follow-up inclui os testes, o fix de scoping e os defeitos confirmados do gerador
-de metadados de SDK ali delimitados.
@@ -62 +52 @@ de metadados de SDK ali delimitados.
-## Tarefa anterior: Turso/libSQL sunset v0.24.0
+## Follow-up autorizado — 2026-11-01, P0 export Markdown
@@ -64,5 +54,5 @@ de metadados de SDK ali delimitados.
-- **Status**: implementação concluída e commitada em `08cb17a4` (2026-10-23); sensores completos PASS
-  registrados em `.sensors-last` (310 s). **Post-gate `PENDING`**: não há receipt humano; o parecer Sonnet
-  `v2` é opinião, não aprovação. Release, tag e publicação **não** foram feitos.
-- **Decisão existente**: [ADR-STORAGE-20260816-1](../adr/2026-08-16-turso-backend-deprecation-and-sunset.md), aceito.
-- **Plano e evidência**: [turso-sunset](./progress/2026-10-23-turso-sunset.md).
+Owner autorizou seguir no P0 independente após o plano v3. Correção local sobre main revalidada,
+testes RED/GREEN, paths/permissões/expiração/integridade/atomicidade e catálogo/reference, com
+verificação no candidato. [Log](progress/2026-11-01-markdown-export-safety.md),
+[canvas](canvas/2026-11-01-markdown-export-safety.md). Tag 0.24.0 aguarda resultado; sem autorização
+de push/PR/merge/tag/publicação, configs reais, RFC0004a ou programa inteiro de evolução.
@@ -70 +60,3 @@ de metadados de SDK ali delimitados.
-## Tarefa anterior: Engram comprehensive improvement — ondas 0-6 (concluída em 2026-10-14)
+Follow-up da revisão: corrigir compatibilidade das tags e custo de sincronização na branch export,
+revalidar em Linux/macOS e registrar RISK-0011 para import em candidato separado. Sem ampliar para
+projeção Git. [Log retomável](progress/2026-11-01-markdown-portability-followup.md).
@@ -72,7 +64,4 @@ de metadados de SDK ali delimitados.
-- **Plano histórico**: [`plans/2026-10-02-engram-comprehensive-improvement-plan.md`](./plans/2026-10-02-engram-comprehensive-improvement-plan.md) (encerrado; sem as ondas 5-6).
-- **Progress log (Lane P — harness/governança/CI docs)**: [`progress/2026-10-05-improvement-lane-p.md`](./progress/2026-10-05-improvement-lane-p.md)
-- **Progress log (Lane R — Rust/produto)**: [`progress/2026-10-05-improvement-lane-r.md`](./progress/2026-10-05-improvement-lane-r.md)
-- **Baseline de execução**: `1952f3b461fe7a8f8a6b5c106ac8c4f5929bae91`. Inventário: [`audits/2026-10-02-improvement-baseline.md`](./audits/2026-10-02-improvement-baseline.md)
-- **Decisões do owner (2026-10-05, chat, históricas)**: ADR `agent-harness-hardening-v1` **aceito**; runners da Onda 4 só com *fake writer*, offline; budget WAL 64 GiB configurável; `defer_embedding=true` enfileira em background. A autorização de execução daquele programa (commits locais, ondas 0-4) **expirou com o programa** e não vale para outras sprints.
-- **Status**: completed (2026-10-14) — as 25 tarefas do plano e as tarefas G1, P1 e Q2F concluídas com revisão SDD independente; lanes integradas (INT); revisão final do branch (Opus): *Ready with fixes*, 0 Critical, corrigida na onda FINALFIX. Ondas 5 e 6 pós-merge (`661b140`..`b1fd869`): `quinn-proto` e deadlock Bash 5.3 em `doctor.sh`, `LazyLock`, `CloudStorage::upload` Invariante #27, `v49` `media_assets` (`#235`), leituras de mídia limitadas (`#236`), wiring `SessionStart`/`PostToolUse`/`SyncWorker` (`#237`), retry `-shm` macOS (`#238`), `aws-sdk-s3` + `rustls 0.23.46` (`#239`), SDKs `161 -> 0` (`G-2`, `G-4`, `G-5`), normalizador de erros MCP (`G-6`, `G-7`, `G-10`), `P1`/`G1`, RFCs 0009–0011, `O2` (73 `.raw`), preparação `v0.24.0` Core / `v0.7.0` SDKs. Registro: [`reviews/2026-10-05-comprehensive-improvement/README.md`](./reviews/2026-10-05-comprehensive-improvement/README.md).
-- **Proveniência**: o veredito [`2026-10-14`](./reviews/2026-10-14-engram-comprehensive-improvement-post.md) não registra reviewer, intervalo nem receipt; ver a [nota de proveniência](./reviews/2026-10-14-engram-comprehensive-improvement-post-provenance.md). Review Canvas do programa: **ausente** (desvio registrado em [`audits/2026-10-02-improvement-baseline.md` §9](./audits/2026-10-02-improvement-baseline.md)).
+RISK-0011: branch local separada `fix/markdown-import-safety`, baseada no export `9bca4efa`.
+Leitura ancorada limitada, modo conforme raiz e autorização/classificação/aplicação transacionais;
+fixtures temporárias com principals reais antes da correção. [Canvas](canvas/2026-11-01-markdown-import-safety.md)
+e [log](progress/2026-11-01-markdown-import-safety.md). Mesmos limites operacionais do P0.
@@ -80 +68,0 @@ de metadados de SDK ali delimitados.
-## Tarefa anterior: Agent harness hardening v1 — Wave 0 governance
@@ -82,5 +70 @@ de metadados de SDK ali delimitados.
-- **Branch**: `feat/harness-hardening-v1`
-- **Progress log**: [`progress/2026-07-21-agent-harness-hardening-v1.md`](./progress/2026-07-21-agent-harness-hardening-v1.md)
-- **Status**: superseded 2026-10-05 — governance proposal landed; the ADR was
-  **accepted by the owner on 2026-10-05** (see the ADR acceptance record). The
-  executable waves now run as part of the comprehensive improvement program above.
+## Follow-up local — candidato integrado Markdown, 2026-11-01
@@ -88,129 +72,4 @@ de metadados de SDK ali delimitados.
-### Em escopo
-
-- Proposed trust-boundary ADR, active plan, Review Canvas, and external-reference intake.
-- Synchronized `SPEC.md` and `progress.md` live-state routing.
-- Independent review under the existing pre-change gates.
-
-### Fora de escopo
-
-- Executable runner, schemas, validators, sandbox adapter, review-parser
-  migration, merge gate, CI/workflow changes, model routing, deployment, and
-  production access.
-- Rust, MCP, storage, SDK, dependency, or product behavior changes.
-
-### Gates esperados
-
-- `bash docs/harness/bin/bootstrap.sh`
-- `bash docs/harness/bin/doctor.sh`
-- `bash docs/harness/bin/sensors.sh`
-- `git diff --check`
-- `bash docs/harness/bin/review-gate.sh post agent-harness-hardening-v1`
-- Human acceptance of the ADR
-
-## Tarefa anterior: Harness maintenance — live-state closeout
-
-- **Branch**: `chore/harness-live-state`
-- **Progress log**: [`progress/2026-06-27-harness-live-state-closeout.md`](./progress/2026-06-27-harness-live-state-closeout.md)
-- **Status**: completed (2026-07-10) — fechamento do estado vivo; review PASS em
-  [`reviews/2026-07-10-engram-10-of-10-live-state-v4-post.md`](./reviews/2026-07-10-engram-10-of-10-live-state-v4-post.md).
-
-### Em escopo
-
-- Atualizar `docs/harness/progress.md` para deixar de apresentar
-  `harness-bootstrap` como tarefa ativa.
-- Atualizar este `SPEC.md` para manter o contrato `SPEC.md` ↔ `progress.md`
-  exigido pelo `doctor.sh`.
-- Registrar um novo log de progresso curto para esta tarefa de housekeeping.
-- Atualizar o metadata de último commit/review/sensores com evidência atual.
-- Adicionar `docs/harness/bin/check-live-state.sh` e `docs/harness/bin/test-check-live-state.sh` como exceção escopada para tornar o live state self-checking.
-- Registrar `docs/harness/canvas/2026-07-09-engram-10-of-10-live-state.md` para a mudança de script do harness.
-
-### Fora de escopo
-
-- Mudanças em scripts do harness além da exceção explícita `check-live-state.sh`/`test-check-live-state.sh`; gates, invariants, policy, workflows, MCP,
-  storage, SDKs ou código Rust.
-- Escolher ou iniciar um follow-up de produto; este closeout só deixa o estado
-  canônico pronto para a próxima branch real.
-
-### Gates esperados
-
-- `bash docs/harness/bin/bootstrap.sh`
-- `bash docs/harness/bin/doctor.sh`
-- `bash docs/harness/bin/sensors.sh quick`
-- `bash docs/harness/bin/review-gate.sh post engram-10-of-10-live-state`
-
-## Sprint encerrada: Harness Engineering v0 — bootstrap & core gates
-
-- **Branch**: (branch atual do trabalho)
-- **Progress log**: [`progress/2026-05-30-harness-bootstrap.md`](./progress/2026-05-30-harness-bootstrap.md)
-- **Status**: completed — implementação inicial do harness operacional
-  inspirado no modelo mbras-backend, adaptado para Rust + MCP + multi-SDK +
-  dual CLI (Claude Code + Claude Code Sonnet), já integrada em `main`.
-
-### Em escopo (v0)
-
-- Criar estrutura `docs/harness/` completa (README, SPEC, INVARIANTS, GATES, CODE_REVIEW_POLICY, progress.md, bin/ scripts, reviews/, known-issues/, progress/).
-- Implementar `bootstrap.sh` — orientação rápida, read-only, <50 linhas, imprime estado + ordem de leitura.
-- Implementar `doctor.sh` — validação de consistência do harness (drift SPEC/progress, executáveis, referências à policy, bootstrap size, etc.).
-- Implementar `sensors.sh` — wrapper determinístico com as etapas de `scripts/ci.sh` (alvo de `make ci`; fmt + clippy -D + testes paridade Linux + docs + MCP ref) + harness doctor + checks específicos de engram.
-- Implementar `review-gate.sh` — generalizado para múltiplos reviewers (claude-sonnet, codex, local). Suporte a pre/post, range, continuity em FAILs, versionamento de artefatos, exclusão de paths do harness, timeout, prompt rico com fake-success patterns de Rust/MCP.
-- Implementar `check-commit-msg.sh` — validador de Conventional Commits com scopes engram/harness.
-- Criar `CODE_REVIEW_POLICY.md` adaptada para Rust, engram (MCP tools, hooks, embeddings, storage invariants, cross-SDK), e o cenário dual-CLI atual.
-- Criar `GATES.md` com thresholds, fake-success patterns específicos (ex.: tests passando só com features locais mas falhando em CI Linux, schema version drift, MCP protocol breakage, embedding cache bounds violados, etc.).
-- Criar `WHAT_WE_DONT_DO.md` como política explícita de escopo negativo para evitar expansão silenciosa de mudanças de harness.
-- Criar `docs/harness/canvas/` com template de Review Canvas para mudanças complexas.
-- Adicionar `baseline.sh`, `quarterly-audit.sh`, lanes opcionais em `sensors.sh` e guard de review para `docs/harness/bin/*`.
-- Atualizar `AGENTS.md` e `Claude.md` para exigir `bootstrap.sh` no início de toda sessão.
-- Atualizar pre-commit hook e/ou justfile para reforçar (sem quebrar fluxo atual).
-- Seed de progresso para esta sprint + registro de decisões.
-- Rodar o loop completo (bootstrap → pre → sensors → post) nesta própria implementação.
-- Documentar como o harness se relaciona com o RFC 0001 (Harness Memory product boundary) e dogfooding futuro.
-
-### Gates esperados para v0
-
-- `bash docs/harness/bin/bootstrap.sh`
-- `bash docs/harness/bin/doctor.sh`
-- `bash docs/harness/bin/sensors.sh`
-- `bash docs/harness/bin/review-gate.sh pre harness-bootstrap`
-- `bash docs/harness/bin/review-gate.sh post harness-bootstrap`
-- `make ci` (`scripts/ci.sh`; paridade mantida; `just ci` é alias opcional quando `just` está instalado)
-
-### Fora de escopo (v0)
-
-- Implementação completa de ingestão automática de eventos de harness no próprio engram (deixa para ENGRA-22+ seguindo RFC 0001).
-- Mudanças em storage schema, MCP tools novos, ou intelligence/consolidation.
-- Suporte nativo non-interactive exec para todos os CLIs (foco em prompt files + paste workflow para o cenário atual Claude Code + Claude Code Sonnet).
-- Substituição total de `.githooks` ou CI GitHub workflows (o harness complementa, não substitui).
-- Reabrir ou alterar RFC 0001 neste escopo.
-- Mudanças em `INVARIANTS.md` (raiz) ou `STANDARDS.md` — apenas docs de processo do harness.
-
-## Próximas iterações (v1+)
-
-- Dogfooding: usar engram MCP + hooks para registrar sessões de harness, reviews, gate results como memórias com provenance forte.
-- `memory_harness_*` tools ou seção dedicada.
-- Agentes especializados em harness (planner, verifier, context-engine) como MCP tools ou personas em `docs/harness/agents/`.
-- Integração mais profunda com Claude Code Sonnet como reviewer externo non-interactive.
-- Suporte a Linear/GitHub sync de tasks no harness (se aplicável ao fluxo de engram).
-
-## Emenda 2026-06-05 — cross-harness improvements
-
-O plano `docs/harness/plans/2026-06-05-engram-harness-improvement-execution-plan.md` adiciona melhorias inspiradas no harness mbras sem importar comportamento de domínio externo:
-
-- Escopo negativo explícito em `WHAT_WE_DONT_DO.md`.
-- Review Canvas para evidência em mudanças complexas.
-- Guard para mudanças em `docs/harness/bin/*`.
-- Baseline snapshot barato.
-- Lanes opcionais em `sensors.sh` que não substituem o gate completo.
-- Auditoria periódica evidence-only.
-
-## Critérios de Saída da Sprint v0
-
-- Estrutura completa + scripts executáveis + doctor.sh verde.
-- Esta própria task passou pelo loop completo (pre + sensors all-green + post PASS).
-- AGENTS.md e Claude.md atualizados e bootstrap rodado com sucesso.
-- README do harness explica o posicionamento de engram como Memory Manager ideal para harnesses de outros projetos.
-- Nenhum drift entre SPEC.md e progress.md; doctor.sh passa limpo.
-
----
-
-**Nota**: Este SPEC é mutável durante a sprint. Atualizações de escopo vão para o log de progresso e (quando relevante) para um novo version do SPEC com nota de data. Invariants do harness não mudam sem ADR + gates anteriores.
+Combinar remediação646636f6 com import1faa5c6f em worktree/branch nova; reter79889460 e hardening,
+validar CLI/SDKs/MCP review default, omitir entradas ocultas do vault, rodar gates no candidato.
+[Log](progress/2026-11-01-markdown-release-integration.md), [canvas](canvas/2026-11-01-markdown-release-integration.md).
+Sem publicar, alterar branch principal ou renovar mandato remoto da branch anterior.
diff --git a/docs/harness/canvas/2026-11-01-markdown-export-safety.md b/docs/harness/canvas/2026-11-01-markdown-export-safety.md
new file mode 100644
index 0000000000000000000000000000000000000000..c4d9a138dcca1fea6fd7c996eca8b6b2664ffd84
--- /dev/null
+++ b/docs/harness/canvas/2026-11-01-markdown-export-safety.md
@@ -0,0 +1,47 @@
+# Review Canvas: markdown-export-safety
+
+Data: 2026-11-01. Escopo autorizado: P0 local sobre origin/main `2e84a594`.
+
+## Approaches Considered
+
+| Abordagem | Decisão | Motivo |
+|---|---|---|
+| Canonicalize da raiz seguido de fs::write | Rejeitada | Tag escapa, symlink redireciona e check/open tem corrida |
+| Incorporar projeção P1/manifest | Rejeitada | Hotfix do produto existente, sem política/RFC nova |
+| Validar lote e escrever por diretórios ancorados sem seguir links | Selecionada | Conter destinos/efeitos; filesystem não é workspace |
+| Ignorar row/relação inválida | Rejeitada | Oculta perda de dados |
+
+## Hot Path Complexity
+
+Consultar/renderizar antes de escrever. Ordenar O(n log n + e log e); buffers proporcionais ao lote.
+Sem rede/LLM/busca vetorial ou transação SQLite durante I/O de arquivos. Atomicidade por arquivo.
+
+## Edge Cases
+
+Traversal/separadores de outra plataforma; symlink de diretório/arquivo; alias SQLite; read-only e
+workspace alheio; expiradas referenciadas; row/relacionamento inválido; empate no índice; erro de
+escrita/rename; corrida em entradas de diretório e limites da raiz administrativa confiável.
+
+## Breakage Risk
+
+| Risco | Impacto | Mitigação/verificação |
+|---|---|---|
+| Permissão sobe de read-only | Cliente antes permissivo é recusado | Nota/dispatcher negativo antes dos efeitos e referência coerente |
+| Plataforma sem primitiva segura | Fallback reabre escape | Falhar fechado sem escrita insegura |
+| Falha parcial de lote | Falso export completo | Preflight e erro explícito; atomicidade só por arquivo |
+| Link falso/fora de escopo/TTL | Histórico incorreto ou vazamento | IDs elegíveis do lote, direção explícita e negativos |
+| Review/gate de outro SHA | Falsa aprovação | Candidato identificado; revisão/receipt próprios, risco não fecha por baseline |
+
+## Follow-up após revisão do owner
+
+- Codificar rótulos como componentes seguros preserva tags internas e nomes reservados; prefixo
+  reservado e hash do rótulo original evitam colisões por substituição simples. Validador FS continua
+  recusando componentes crus inseguros.
+- POSIX fsync por arquivo, diretório por grupo e flush forte final por dispositivo mantêm a garantia
+  de conclusão do lote, retirando o flush forte repetido por arquivo. Teste injeta falha no flush final.
+- Regressões novas falharam antes da correção (77 PASS/3 FAIL); medição de 300 memórias registra
+  mediana 2.388540→0.055461 s. Não é extrapolação de escala nem substitui os gates.
+- ACL deny-only continua recusada por política conservadora, agora explicada ao usuário. Linux
+  ganha fixture de default ACL que não altera os bits de acesso.
+- Import fica em candidato separado (RISK-0011); mesmo nome/mesma descrição de RISK-0010 reconciliados
+  com o checkout de planejamento. O wrapper público deve propagar erro estruturado, nunca sucesso zero.
diff --git a/docs/harness/canvas/2026-11-01-markdown-import-safety.md b/docs/harness/canvas/2026-11-01-markdown-import-safety.md
new file mode 100644
index 0000000000000000000000000000000000000000..43d046b9459518ab87ef5e6d833a9c874979c781
--- /dev/null
+++ b/docs/harness/canvas/2026-11-01-markdown-import-safety.md
@@ -0,0 +1,39 @@
+# Review Canvas: markdown-import-safety
+
+RISK-0011, candidato local separado, empilhado em export `9bca4efa`. Sem publicação/dados reais.
+
+## Approaches Considered
+
+| Abordagem | Decisão | Motivo |
+|---|---|---|
+| Canonicalize seguido de read_to_string | Rejeitada | Descendentes seguem links e há check/open |
+| Autorizar só workspace do argumento | Rejeitada | ID persistido pode pertencer a outro workspace |
+| Leitura ancorada/limitada e aplicação DB transacional | Selecionada | Valida origem antes do open e workspace/versão no mesmo snapshot |
+
+## Hot Path Complexity
+
+Walk ordenado e limitado: profundidade 32, 10000 Markdown, 20000 entradas, 4 MiB por arquivo e
+64 MiB no total. Buffers limitados; nenhum IO de arquivo/rede/embedding dentro da transação DB.
+Descendentes privados segundo a política ACL/ownership do export; mesma identidade OS fora do sandbox.
+
+## Edge Cases
+
+Symlink/ciclo/hardlink/FIFO, artefatos SQLite antes de abrir descritor regular, UTF-8 inválido,
+ID explícito inválido, ID estrangeiro/inexistente indistinguíveis para principal restrito, criação
+por workspace efetivo, versão concorrente, force_version, tags/Obsidian e dry-run legados.
+
+## Breakage Risk
+
+Com raiz administrada, scoped_write; sem raiz, maintenance. Negar chamadas diretas também. Pré-ler
+arquivos e validar lote antes de aplicar. Confirm usa uma transação SQLite para autorizar/classificar
+antes de escrever e aplicar o lote; erro antes do commit desfaz o lote DB; erro após callback concluído tem outcome unknown/applied null, sem afirmar rollback. Dry-run usa snapshot de leitura. Erros
+não viram sucesso zero no wrapper. Limites/ACL e política de IDs restritos exigem nota de compatibilidade.
+
+
+| Risco | Impacto | Mitigação e evidência |
+|---|---|---|
+| Mode/raiz/workspace mais estritos | Clientes permissivos passam a falhar | Nota de migração, testes handler+dispatch com principals e controle positivo |
+| Erro após callback/commit | Retry duplica ou sobrescreve contexto | outcome unknown/applied null, teste real pós-commit, reconciliação antes de retry |
+| Erro antes do commit | Parte do lote persistida | Transação única, preflight, trigger ABORT confirma rollback com tags/versões |
+| Walker abre alias SQLite | Perda de locks e commits WAL | Metadata antes do open, no-follow, G1 por processo independente |
+| YAML escalar/quotas | Campos viram ausência ou consumo sem limite | Testes tab/quoted/empty ID, reader com limites e erros explícitos |
diff --git a/docs/harness/progress.md b/docs/harness/progress.md
index 3a3a0993cdf3e72177b3e2cc7280fdeba24bab4a..03d4554fa1b5bca8455dfb2ef5f79a521d6fb1cd 100644
--- a/docs/harness/progress.md
+++ b/docs/harness/progress.md
@@ -10,3 +10,3 @@
-| Last sensors | `2026-10-23 status=pass mode=full timestamp=2026-10-23T20:10:00Z duration_sec=310, sem exclusões (receipts commitados em 08cb17a4; rodado no candidato do sunset, run anterior falhou em offline_lane). Sem receipt commitado de 2026-09-03 a 2026-10-23; não reexecutado depois. Detalhe em progress/2026-10-26-remediation.md.` |
-| Last commit | `82f79bed` |
-| Last live-state check | `2026-10-30 — Antigravity, em fix/resolve-all-2026-10-29: Last commit = 82f79bed (pai do refresh documental/live-state pós-provisionamento de allowed-signers). bash docs/harness/bin/check-live-state.sh --progress docs/harness/progress.md. Revisão independente R2 PASS (não substitui post-gate humano).` |
+| Last sensors | `2026-11-01 — receipts das branches são históricos; nenhum gate do candidato integrado executado ainda. Último import isolado full422s e export459s não atestam esta combinação.` |
+| Last commit | `646636f6` |
+| Last live-state check | `2026-11-01 — integração local em fix/markdown-release-integration:646636f6 +1faa5c6f, em preparação. Invocação: bash docs/harness/bin/check-live-state.sh --progress docs/harness/progress.md. Gate/receipt do novo candidato pendentes.` |
@@ -39,0 +40,2 @@
+- **Candidato integrado local (2026-11-01)**: remediação646636f6 + import1faa5c6f (inclui export9bca4efa), mantendo79889460; [log](progress/2026-11-01-markdown-release-integration.md). Trabalho local; nenhuma autorização remota ampliada.
+
diff --git a/docs/harness/risk-register.yaml b/docs/harness/risk-register.yaml
index b939fed5c3eaddd007455e31588e10bb47cd4d72..c0082e41e1428232687ae36595e64120e27f28d8 100644
--- a/docs/harness/risk-register.yaml
+++ b/docs/harness/risk-register.yaml
@@ -128,0 +129,30 @@ entries:
+
+  - id: RISK-0010
+    date: 2026-10-31
+    status: open
+    description: "Export Markdown publicado permite escape por tag e escrita read_only; inclui expiradas, omite erros, gera indice instavel e relacoes com direcao incorreta."
+    probability: 3
+    impact: 5
+    score: 15
+    owner: "Ronaldo / integrador P0"
+    mitigation: "Candidato local fix/markdown-export-safety sobre main 2e84a594 (0cd2d2d5/4bf79f85). Full historico PASS/515 s nao cobre o follow-up atual: codec de tags internas, fsync por arquivo/diretorio com flush final por dispositivo e decoder de erros. Follow-up commitado em 9bca4efa: full PASS/459 s, Linux 81+22 PASS (1 probe manual ignorado), review estatico v2 sem novo achado; formal post-gate e integracao pendentes. P1 permanece separado. Log: docs/harness/progress/2026-11-01-markdown-portability-followup.md."
+    contingency: "ENGRAM_EXPORT_BASE_DIR sozinho nao mitiga o codigo publicado. Tag 0.24.0 aguarda os bloqueantes do export e a avaliacao do import RISK-0011; sem publicacao ou alteracao operacional."
+    monitoring:
+      - "tags session:/category:/confidence:/file:/trace:, aux e colisoes no codec"
+      - "traversal, links/SQLite, autorizacao, expiracao, erros e determinismo"
+      - "custo de export e erro de flush final; validacao Linux e SHA da integracao"
+
+  - id: RISK-0011
+    date: 2026-11-01
+    status: open
+    description: "Import Markdown aceita filesystem sem raiz administrada no modo scoped_write, segue links e usa engram_id de arquivo sem autorizar o workspace persistido. Classificacao anterior a transacao permite corrida de versao."
+    probability: 3
+    impact: 5
+    score: 15
+    owner: "Ronaldo / integrador import"
+    mitigation: "Candidato local fix/markdown-import-safety, commit ffb51597, sobre export9bca4efa. RED2PASS/12FAIL reproduziu IDOR/links/raiz; correcao com guard antes de IO, reader ancorado limitado e autorizacao/classificacao/aplicacao em TX. Review encontrou dois achados adicionais, corrigidos com RED/GREEN: tabs no parser e resultado pos-commit unknown. Mac fullPASS/422s, complementoCI13PASS, Linux100+25PASS (probe/helper declarados); formal post-gate/receipt e integracao pendentes. Log docs/harness/progress/2026-11-01-markdown-import-safety.md."
+    contingency: "Manter tag 0.24.0 aguardando avaliacao/correcao; nenhuma tool foi desabilitada em ambiente real. Nao alegar bypass de criacao via frontmatter no dispatcher sem evidencia: override autorizado prevalece."
+    monitoring:
+      - "principal alpha com engram_id de beta e inexistente: mesma negacao, sem mutacao"
+      - "symlink/hardlink/ciclo/FIFO, limites de leitura e aliases SQLite antes do open"
+      - "mudanca de versao/workspace entre preview e apply, erros e estado parcial"
diff --git a/docs/migrations/v0.24.0-markdown-export-security.md b/docs/migrations/v0.24.0-markdown-export-security.md
new file mode 100644
index 0000000000000000000000000000000000000000..c4af4de2898771c48e20ecc2d2bd663824632a6e
--- /dev/null
+++ b/docs/migrations/v0.24.0-markdown-export-security.md
@@ -0,0 +1,75 @@
+# Markdown export security and compatibility
+
+Status: local P0 candidate for 0.24.0; not a published release. No database migration or SDK parameter
+change. `memory_export_markdown` still writes one Markdown file per memory plus an index.
+
+## Authorization and destination
+
+Filesystem export is a mutation, even though it reads the canonical database. `read_only` principals
+and per-call/process restrictions cannot invoke it. With a nonempty administrator-configured
+`ENGRAM_EXPORT_BASE_DIR`, it requires at least `scoped_write`, an authorized workspace, and a destination
+contained in that root. Without that configured root, it requires `maintenance` (system-admin permission
+for authenticated principals). A caller's mode argument cannot elevate its transport principal.
+`permission_mode_status` reports the concrete requirement for the current configuration.
+
+Trusted local CLI/stdio calls with no principal or permission restriction retain their unconstrained
+local behavior. Network deployments should use the existing authenticated principal boundary; this
+change does not supply credentials or alter user configurations.
+
+Output directories must belong to the server OS user, must not be group/world writable, and must not
+have extended/access/default ACLs. macOS ACLs and Linux ACL attributes are inspected on the directory
+descriptor; unavailable/failed inspection is an error. The policy conservatively refuses any such ACL
+rather than inferring whether particular entries grant writing. This includes deny-only ACLs commonly
+present on macOS Desktop/Documents/Downloads; use a private subdirectory without an ACL. Engram does
+not remove ACLs automatically. This also prevents inheriting ACL grants
+into temporary files. New
+directories/files are private (0700/0600); existing directory permissions are not changed automatically.
+The administrator must protect the root and its ancestors from replacement by other processes with
+the server's OS identity. Directory descriptors pin the authorized directory while generated components
+are traversed without following links. This is not process isolation against that OS user.
+
+Grouping labels containing unsafe characters or reserved names are encoded as a bounded slug plus
+96 bits of SHA-256 of the original label. The escape prefix is reserved, so an ordinary tag cannot
+impersonate an encoded one. Tags such as `session:abc`, `category:decision`, `file:path` and `aux`
+therefore remain exportable; names can differ from older exports. Symlink destinations, hard links,
+non-regular files and active SQLite files are refused before publication. The root variable alone did not contain generated paths in older code;
+the correction checks every destination and uses directory-relative filesystem operations. Safe writes
+are currently implemented on Linux/macOS. On other platforms filesystem export returns an explicit unsupported
+error before creating output, rather than using an unsafe fallback.
+
+## Contents and errors
+
+A single read snapshot and cutoff select only current, unarchived memories satisfying
+`expires_at IS NULL OR expires_at > cutoff`, comparing parsed RFC3339 instants with nanosecond
+precision rather than text. Equivalent `Z`/`+00:00`/offset forms compare equally; equality is expired and
+invalid timestamps are an explicit error. Links include only active edges whose
+two endpoints are eligible in the exported workspace; no placeholder reveals an omitted endpoint.
+An incoming A→B `supersedes` relation in B is labeled `incoming supersedes`, preserving the original
+direction. This is not a new graph roundtrip format or RFC0004a implementation.
+
+Rows, metadata or relations that cannot be decoded/read fail the export; they are not silently dropped.
+All data are rendered before files are written. Type-count ties, row ties, tags and relations have total
+ordering, so the same snapshot creates the same bytes in separate processes.
+
+Each file is written to a new private temporary file, synchronized with POSIX `fsync`, then atomically
+renamed. Each destination directory is synchronized after its group; creation also synchronizes its
+parent. A final strong flush per filesystem device completes the batch (`F_FULLFSYNC` on macOS),
+covering the previously synchronized data and metadata on that device. Failure of this final flush
+is an error, even when complete files are already visible. Write/rename errors preserve explicit failure and temporary cleanup is checked.
+There is no transaction across the whole tree: an I/O failure can leave earlier complete files, and a
+post-rename synchronization failure has an uncertain durability outcome. No successful batch result is
+returned in either case. Inspect/retry the export deliberately after resolving the error.
+
+The valid Daily/transcript selection and content-derived filenames of the existing export remain.
+This hotfix does not supply a managed Git projection, publication approval/scanning, manifest, pruning
+or human-edit reconciliation. Those require the separately proposed P1 policy and RFC; do not treat an
+export directory as an automatically publishable wiki.
+
+## Measured follow-up cost
+
+On the same macOS machine, the manual synthetic probe exported 300 memories as 301 files (227562
+bytes), three times per candidate with the CI profile. Median elapsed time changed from 2.388540 s
+(per-file/per-directory strong flush in the first P0) to 0.055461 s (POSIX fsync plus final device
+flush), about 43 times faster. This is a small local measurement, not a p95 or a 100K scaling claim.
+The ignored timing probe is run explicitly; correctness checks never use a wall-clock threshold.
+`docs/harness/progress/2026-11-01-markdown-portability-followup.md` records commands/evidence.
diff --git a/docs/migrations/v0.24.0-markdown-import-security.md b/docs/migrations/v0.24.0-markdown-import-security.md
new file mode 100644
index 0000000000000000000000000000000000000000..70a24d597fb3213c8c77a3d688f636c1bc03b542
--- /dev/null
+++ b/docs/migrations/v0.24.0-markdown-import-security.md
@@ -0,0 +1,72 @@
+# Markdown import security and compatibility
+
+Local RISK-0011 candidate, stacked on export `9bca4efa`; no tag/release or database migration.
+SQLite + WAL remains canonical. This is hardening of the existing import, without a Git projection.
+
+## Access and filesystem
+
+`memory_import_markdown` requires at least scoped_write within the nonempty administrator-configured
+`ENGRAM_EXPORT_BASE_DIR`. Without that root it requires maintenance, including dry-run/review: reading
+server files is an effect. Direct Rust/CLI calls enforce the same restrictions. Unconstrained local
+owner calls without a principal or permission restriction retain their local behavior.
+
+Linux/macOS input roots and descendant directories use pinned descriptors and no-follow operations.
+They must be owned by the server OS user, not group/world writable and have no extended/default ACLs.
+As in export, the conservative policy also refuses deny-only ACLs; use a private subdirectory without
+an ACL. Engram does not change ACLs. The administrator protects the root/ancestors against replacement
+by processes sharing the server OS identity; this is not a sandbox against that identity. Unsupported
+platforms fail explicitly before input reads instead of using an unsafe fallback.
+
+The reader refuses symlinks (including cycles), hardlinked Markdown, non-regular Markdown and active
+SQLite database/WAL/SHM/journal aliases. Metadata/identity checks precede regular-file open to preserve
+SQLite process/inode locks; directories are opened with O_DIRECTORY. Input is sorted and limited to:
+
+- 32 descendant directory levels and 20000 directory entries, including non-Markdown entries;
+- 10000 Markdown files, 4 MiB per file and 64 MiB total content read;
+- UTF-8 names and content; inspection/read/limit failures are errors.
+
+Index/README and underscore-prefixed Markdown files remain excluded. Hidden entries below the
+selected root are skipped without stat/open/traversal, including `.git`, `.obsidian` and `.trash`.
+Each encountered hidden name still counts once against the entry limit; its descendants do not.
+An explicitly selected hidden input root (for example `.engram/wiki`) is still usable. Visible links
+remain errors. Input filesystem IO and parsing finish before database transactions; failures in this
+input phase apply no database writes.
+
+## IDs, versions and batch application
+
+An existing `engram_id` is authorized against its stored workspace inside the same snapshot used for
+classification/application. A file's claimed workspace is never authorization for that ID. For a
+restricted principal, missing and foreign IDs produce the same not_found error, without exposing
+hash, version, workspace or contents. For unrestricted local owners the legacy missing-ID→new
+behavior remains. Absent IDs allow creation; explicitly invalid/zero/negative IDs and duplicate IDs
+within an input batch are rejected. Explicit versions must be positive integers. The scalar-key parser recognizes spaces/tabs after the
+colon and quoted/unquoted scalar keys; it is not a general YAML object/alias deserializer.
+
+Creation normalizes and authorizes its effective workspace (explicit override, then frontmatter,
+then default). An authorized override still takes precedence over frontmatter. Non-engram_ keys
+remain ignored; legacy hash comparison, tags/importance updates and force_version remain supported.
+
+Review uses a read snapshot and changes no memories. Confirm uses BEGIN IMMEDIATE: it authorizes and
+classifies the whole batch before its first write, then applies within that transaction. There is no
+classification/update race with another SQLite writer. Parsing/authorization/pre-commit errors fail the batch, and pre-commit database errors roll back
+earlier writes (`outcome: not_applied`, `applied: 0`). Storage can also fail during COMMIT or while
+revalidating permissions after COMMIT. When the transaction callback completed but Storage returns
+an error, the result is conservatively `outcome: unknown`, `applied: null`, and `attempted: N` actions.
+Inspect the canonical database before retrying; a post-commit error may accompany persisted writes. No filesystem IO,
+network call or embedding API occurs inside the transaction. Version conflicts remain staged in the
+report while other eligible files may be applied, preserving the existing conflict workflow.
+
+The MCP result uses structured errors; the public Rust wrapper propagates them. A report with zeros
+is never a substitute for a failed import. Old per-file error/partial-success behavior is intentionally
+replaced by a failed batch. Reconcile duplicate IDs left by old content-derived export names before
+importing. Publication approval, secret scanning, manifests/pruning and roundtrip graph metadata remain
+in the proposed P1/RFC scope.
+
+## Review defaults across clients
+
+The integrated candidate retains `79889460`: CLI `import markdown` and Rust `ImportOptions::default`
+review without applying by default; CLI requires `--confirm` for writes and rejects combining it with
+`--dry-run`. Python defaults to `confirm=False`; TypeScript to `confirm: false` (and rejects conflicting
+flags). MCP omission/false of `confirm` returns `dry_run: true`; confirmed success returns false.
+These defaults do not relax filesystem/workspace authorization. The missing-ID distinction for
+restricted principals and local owners is intentional and described in the IDs section.
diff --git a/src/mcp/handlers/markdown_export/export.rs b/src/mcp/handlers/markdown_export/export.rs
index 919877029d2fe2420d0917ab44b2b83541aa61e1..5c55592c4eef8634deb26054e9b96b8f19f531d3 100644
--- a/src/mcp/handlers/markdown_export/export.rs
+++ b/src/mcp/handlers/markdown_export/export.rs
@@ -0,0 +1 @@
+pub(super) mod filesystem;
@@ -12,0 +14,2 @@ mod naming_tests;
+#[cfg(test)]
+mod security_tests;
diff --git a/src/mcp/handlers/markdown_export/export/filesystem.rs b/src/mcp/handlers/markdown_export/export/filesystem.rs
new file mode 100644
index 0000000000000000000000000000000000000000..5b54702cb0ec566e93771ea8ef7eb0eb9cfdc980
--- /dev/null
+++ b/src/mcp/handlers/markdown_export/export/filesystem.rs
@@ -0,0 +1,609 @@
+//! Export writes are relative to pinned directory descriptors, never a checked
+//! path followed by a regular-file open. The output root and descendants must
+//! be owned by this process user and not writable by other users/groups.
+//! Administrators must protect the root's ancestors from same-user replacement;
+//! this is not a sandbox against another process with the server's OS identity.
+
+use std::collections::BTreeSet;
+use std::io;
+use std::path::{Component, Path, PathBuf};
+
+use crate::storage::Storage;
+
+#[cfg(any(target_os = "linux", target_os = "macos"))]
+mod acl;
+
+pub(in crate::mcp::handlers::markdown_export) struct MarkdownSource {
+    pub relative: PathBuf,
+    pub raw: String,
+}
+
+#[cfg(any(target_os = "linux", target_os = "macos"))]
+mod reader;
+#[cfg(any(target_os = "linux", target_os = "macos"))]
+pub(in crate::mcp::handlers::markdown_export) use reader::read_markdown;
+
+#[cfg(not(any(target_os = "linux", target_os = "macos")))]
+pub(in crate::mcp::handlers::markdown_export) fn read_markdown(
+    _root: &Path,
+    _storage: &Storage,
+) -> io::Result<Vec<MarkdownSource>> {
+    Err(io::Error::new(
+        io::ErrorKind::Unsupported,
+        "safe filesystem import requires Linux/macOS directory-relative and ACL operations",
+    ))
+}
+
+pub(super) fn validate_component(name: &str) -> io::Result<()> {
+    let stem = name.split('.').next().unwrap_or("").to_ascii_uppercase();
+    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
+        || (stem.len() == 4
+            && (stem.starts_with("COM") || stem.starts_with("LPT"))
+            && matches!(stem.as_bytes()[3], b'1'..=b'9'));
+    if name.is_empty()
+        || matches!(name, "." | "..")
+        || name.ends_with(['.', ' '])
+        || name
+            .chars()
+            .any(|c| c.is_control() || "/\\:<>\"|?*".contains(c))
+        || reserved
+    {
+        return Err(io::Error::new(
+            io::ErrorKind::InvalidInput,
+            "unsafe export path component",
+        ));
+    }
+    Ok(())
+}
+
+fn validate_batch(root: &Path, batch: &[(PathBuf, String)], storage: &Storage) -> io::Result<()> {
+    storage
+        .refuse_active_sqlite_artifact(root)
+        .map_err(io::Error::other)?;
+    let mut names = BTreeSet::new();
+    for (relative, _) in batch {
+        if relative.as_os_str().is_empty() || relative.is_absolute() {
+            return Err(io::Error::new(
+                io::ErrorKind::InvalidInput,
+                "export path must be relative",
+            ));
+        }
+        for component in relative.components() {
+            let Component::Normal(name) = component else {
+                return Err(io::Error::new(
+                    io::ErrorKind::InvalidInput,
+                    "invalid export path",
+                ));
+            };
+            validate_component(
+                name.to_str()
+                    .ok_or_else(|| io::Error::other("non-UTF-8 export name"))?,
+            )?;
+        }
+        if !names.insert(relative.clone()) {
+            return Err(io::Error::new(
+                io::ErrorKind::InvalidInput,
+                "duplicate export destination",
+            ));
+        }
+        storage
+            .refuse_active_sqlite_artifact(root.join(relative))
+            .map_err(io::Error::other)?;
+    }
+    for relative in &names {
+        if relative
+            .ancestors()
+            .skip(1)
+            .any(|parent| names.contains(parent))
+        {
+            return Err(io::Error::new(
+                io::ErrorKind::InvalidInput,
+                "export file/directory collision",
+            ));
+        }
+    }
+    Ok(())
+}
+
+#[cfg(not(any(target_os = "linux", target_os = "macos")))]
+pub(super) fn write_batch(
+    root: &Path,
+    batch: &[(PathBuf, String)],
+    storage: &Storage,
+) -> io::Result<()> {
+    validate_batch(root, batch, storage)?;
+    Err(io::Error::new(
+        io::ErrorKind::Unsupported,
+        "safe filesystem export requires Linux/macOS directory-relative and ACL operations",
+    ))
+}
+
+#[cfg(any(target_os = "linux", target_os = "macos"))]
+pub(super) use unix::write_batch;
+
+#[cfg(any(target_os = "linux", target_os = "macos"))]
+mod unix {
+    use super::*;
+    use std::collections::BTreeMap;
+    use std::ffi::{CStr, CString, OsStr};
+    use std::fs::{File, OpenOptions};
+    use std::io::Write;
+    use std::mem::MaybeUninit;
+    use std::os::fd::{AsRawFd, FromRawFd};
+    use std::os::unix::ffi::OsStrExt;
+    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
+
+    fn c_name(name: &OsStr) -> io::Result<CString> {
+        CString::new(name.as_bytes())
+            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "null byte in export path"))
+    }
+
+    fn sync_to_filesystem(file: &File) -> io::Result<()> {
+        rustix::fs::fsync(file).map_err(io::Error::from)
+    }
+
+    pub(super) fn open_child(parent: &File, name: &CStr, create: bool) -> io::Result<Option<File>> {
+        // SAFETY: parent is a live directory fd and name is NUL-terminated.
+        // O_DIRECTORY prevents opening a SQLite regular file even transiently.
+        let mut fd = unsafe {
+            libc::openat(
+                parent.as_raw_fd(),
+                name.as_ptr(),
+                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
+            )
+        };
+        if fd < 0 && io::Error::last_os_error().kind() == io::ErrorKind::NotFound {
+            if !create {
+                return Ok(None);
+            }
+            // SAFETY: same live directory/name; only this one component is created.
+            let made = unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o700) };
+            if made < 0 && io::Error::last_os_error().kind() != io::ErrorKind::AlreadyExists {
+                return Err(io::Error::last_os_error());
+            }
+            sync_to_filesystem(parent)?;
+            // SAFETY: open the just-created directory without following a replacement symlink.
+            fd = unsafe {
+                libc::openat(
+                    parent.as_raw_fd(),
+                    name.as_ptr(),
+                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
+                )
+            };
+        }
+        if fd < 0 {
+            return Err(io::Error::last_os_error());
+        }
+        // SAFETY: a successful openat returned a new uniquely owned descriptor.
+        Ok(Some(unsafe { File::from_raw_fd(fd) }))
+    }
+
+    pub(super) fn trusted_directory(dir: &File) -> io::Result<()> {
+        let metadata = dir.metadata()?;
+        // SAFETY: geteuid has no arguments and does not mutate memory.
+        if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o022 != 0 {
+            return Err(io::Error::new(
+                io::ErrorKind::PermissionDenied,
+                "export directories must be owned by the server user and not group/world writable",
+            ));
+        }
+        super::acl::check_directory(dir)
+    }
+
+    pub(super) fn open_root(path: &Path, create: bool) -> io::Result<Option<File>> {
+        if !path.is_absolute() {
+            return Err(io::Error::other("export root is not absolute"));
+        }
+        let mut dir = OpenOptions::new()
+            .read(true)
+            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
+            .open("/")?;
+        for component in path.components() {
+            match component {
+                Component::RootDir => (),
+                Component::Normal(name) => {
+                    let Some(child) = open_child(&dir, &c_name(name)?, create)? else {
+                        return Ok(None);
+                    };
+                    dir = child;
+                }
+                _ => {
+                    return Err(io::Error::other(
+                        "export root contains non-normal components",
+                    ))
+                }
+            }
+        }
+        trusted_directory(&dir)?;
+        Ok(Some(dir))
+    }
+
+    fn parent_dir(root: &File, relative: &Path, create: bool) -> io::Result<Option<File>> {
+        let mut dir = root.try_clone()?; // Only duplicates a directory, never SQLite.
+        for component in relative
+            .parent()
+            .unwrap_or_else(|| Path::new(""))
+            .components()
+        {
+            let Component::Normal(name) = component else {
+                return Err(io::Error::other("invalid subdirectory"));
+            };
+            let Some(child) = open_child(&dir, &c_name(name)?, create)? else {
+                return Ok(None);
+            };
+            trusted_directory(&child)?;
+            dir = child;
+        }
+        Ok(Some(dir))
+    }
+
+    pub(super) fn protected_inodes(storage: &Storage) -> Vec<(u64, u64)> {
+        if storage.db_path() == ":memory:" {
+            return Vec::new();
+        }
+        ["", "-wal", "-shm", "-journal"]
+            .iter()
+            .filter_map(|suffix| {
+                std::fs::metadata(format!("{}{suffix}", storage.db_path()))
+                    .ok()
+                    .map(|m| (m.dev(), m.ino()))
+            })
+            .collect()
+    }
+
+    #[allow(clippy::unnecessary_cast)] // libc's dev_t/ino_t differ across Unix targets.
+    pub(super) fn check_file(
+        parent: &File,
+        name: &CStr,
+        protected: &[(u64, u64)],
+    ) -> io::Result<()> {
+        let mut stat = MaybeUninit::<libc::stat>::uninit();
+        // SAFETY: stat points to enough writable storage, parent is live and name valid.
+        // This inspects the entry without opening/closing a SQLite-bearing inode.
+        if unsafe {
+            libc::fstatat(
+                parent.as_raw_fd(),
+                name.as_ptr(),
+                stat.as_mut_ptr(),
+                libc::AT_SYMLINK_NOFOLLOW,
+            )
+        } < 0
+        {
+            let err = io::Error::last_os_error();
+            return if err.kind() == io::ErrorKind::NotFound {
+                Ok(())
+            } else {
+                Err(err)
+            };
+        }
+        // SAFETY: successful fstatat initialized the entire stat structure.
+        let stat = unsafe { stat.assume_init() };
+        if stat.st_mode & libc::S_IFMT != libc::S_IFREG
+            || stat.st_nlink > 1
+            || protected.contains(&(stat.st_dev as u64, stat.st_ino as u64))
+        {
+            return Err(io::Error::new(
+                io::ErrorKind::PermissionDenied,
+                "export destination is a link, non-regular file or active SQLite artifact",
+            ));
+        }
+        Ok(())
+    }
+
+    fn atomic_write(
+        parent: &File,
+        name: &CStr,
+        protected: &[(u64, u64)],
+        write: impl FnOnce(&mut File) -> io::Result<()>,
+    ) -> io::Result<()> {
+        atomic_write_with(parent, name, protected, write, |temporary| {
+            // SAFETY: both names are single components under the same pinned directory.
+            // renameat replaces an entry atomically; it never follows a final symlink.
+            if unsafe {
+                libc::renameat(
+                    parent.as_raw_fd(),
+                    temporary.as_ptr(),
+                    parent.as_raw_fd(),
+                    name.as_ptr(),
+                )
+            } < 0
+            {
+                Err(io::Error::last_os_error())
+            } else {
+                Ok(())
+            }
+        })
+    }
+
+    fn atomic_write_with(
+        parent: &File,
+        name: &CStr,
+        protected: &[(u64, u64)],
+        write: impl FnOnce(&mut File) -> io::Result<()>,
+        publish: impl FnOnce(&CStr) -> io::Result<()>,
+    ) -> io::Result<()> {
+        check_file(parent, name, protected)?;
+        let temp = CString::new(format!(".engram-export-{}.tmp", uuid::Uuid::new_v4()))
+            .map_err(io::Error::other)?;
+        // SAFETY: exclusive creation of a new private file relative to the live directory.
+        // No existing inode (including an active SQLite alias) can be opened here.
+        let fd = unsafe {
+            libc::openat(
+                parent.as_raw_fd(),
+                temp.as_ptr(),
+                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
+                0o600 as libc::c_uint,
+            )
+        };
+        if fd < 0 {
+            return Err(io::Error::last_os_error());
+        }
+        // SAFETY: successful exclusive openat returned a new uniquely owned fd.
+        let mut file = unsafe { File::from_raw_fd(fd) };
+        let result = (|| {
+            write(&mut file)?;
+            sync_to_filesystem(&file)?;
+            check_file(parent, name, protected)?;
+            publish(&temp)?;
+            Ok(())
+        })();
+        if let Err(err) = result {
+            // SAFETY: removes only our temporary name in the pinned directory, never a link target.
+            if unsafe { libc::unlinkat(parent.as_raw_fd(), temp.as_ptr(), 0) } < 0 {
+                let cleanup = io::Error::last_os_error();
+                if cleanup.kind() != io::ErrorKind::NotFound {
+                    return Err(io::Error::other(format!(
+                        "{err}; temporary cleanup failed: {cleanup}"
+                    )));
+                }
+            }
+            return Err(err);
+        }
+        Ok(())
+    }
+
+    pub(crate) fn write_batch(
+        root: &Path,
+        batch: &[(PathBuf, String)],
+        storage: &Storage,
+    ) -> io::Result<()> {
+        write_batch_with_barrier(root, batch, storage, |directory| directory.sync_all())
+    }
+
+    fn write_batch_with_barrier(
+        root: &Path,
+        batch: &[(PathBuf, String)],
+        storage: &Storage,
+        mut barrier: impl FnMut(&File) -> io::Result<()>,
+    ) -> io::Result<()> {
+        validate_batch(root, batch, storage)?;
+        let protected = protected_inodes(storage);
+        let existing = open_root(root, false)?;
+        if let Some(dir) = &existing {
+            for (relative, _) in batch {
+                if let Some(parent) = parent_dir(dir, relative, false)? {
+                    check_file(
+                        &parent,
+                        &c_name(
+                            relative
+                                .file_name()
+                                .ok_or_else(|| io::Error::other("missing filename"))?,
+                        )?,
+                        &protected,
+                    )?;
+                }
+            }
+        }
+        let dir = match existing {
+            Some(dir) => dir,
+            None => {
+                open_root(root, true)?.ok_or_else(|| io::Error::other("export root unavailable"))?
+            }
+        };
+        // Group by destination directory so its metadata is synchronized once.
+        // Keep only one pinned representative per device, not one fd per file
+        // or directory: large exports must not exhaust the descriptor limit.
+        let mut groups: BTreeMap<PathBuf, Vec<&(PathBuf, String)>> = BTreeMap::new();
+        for entry in batch {
+            groups
+                .entry(entry.0.parent().unwrap_or_else(|| Path::new("")).to_owned())
+                .or_default()
+                .push(entry);
+        }
+        let root_files = groups.remove(Path::new(""));
+        let mut devices: BTreeMap<u64, File> = BTreeMap::new();
+        // The root group contains index.md, which remains last in the batch.
+        for (_, files) in groups
+            .into_iter()
+            .chain(root_files.into_iter().map(|files| (PathBuf::new(), files)))
+        {
+            let first = files
+                .first()
+                .ok_or_else(|| io::Error::other("empty export group"))?;
+            let parent = parent_dir(&dir, &first.0, true)?
+                .ok_or_else(|| io::Error::other("export parent unavailable"))?;
+            for (relative, bytes) in files {
+                let name = c_name(
+                    relative
+                        .file_name()
+                        .ok_or_else(|| io::Error::other("missing filename"))?,
+                )?;
+                atomic_write(&parent, &name, &protected, |file| {
+                    file.write_all(bytes.as_bytes())
+                })?;
+            }
+            sync_to_filesystem(&parent)?;
+            if let std::collections::btree_map::Entry::Vacant(entry) =
+                devices.entry(parent.metadata()?.dev())
+            {
+                entry.insert(parent.try_clone()?);
+            }
+        }
+        // On macOS this performs F_FULLFSYNC: prior fsync'd data and metadata
+        // on that device are durable when it succeeds (fcntl(2)). Linux has
+        // already fsync'd each file/directory. Errors remain batch failures.
+        for directory in devices.values() {
+            barrier(directory)?;
+        }
+        Ok(())
+    }
+
+    #[cfg(test)]
+    mod tests {
+        use super::*;
+
+        #[test]
+        fn p0_failed_write_preserves_old_file_and_removes_temporary() {
+            let fixture = tempfile::tempdir().unwrap();
+            let root = std::fs::canonicalize(fixture.path()).unwrap();
+            std::fs::write(root.join("note.md"), b"old").unwrap();
+            let dir = open_root(&root, false).unwrap().unwrap();
+            let err = atomic_write(&dir, c"note.md", &[], |file| {
+                file.write_all(b"partial")?;
+                Err(io::Error::other("injected write failure"))
+            })
+            .unwrap_err();
+            assert!(err.to_string().contains("injected"));
+            assert_eq!(std::fs::read(root.join("note.md")).unwrap(), b"old");
+            assert_eq!(std::fs::read_dir(root).unwrap().count(), 1);
+        }
+
+        #[test]
+        fn p0_failed_rename_preserves_old_file_and_removes_temporary() {
+            let fixture = tempfile::tempdir().unwrap();
+            let root = std::fs::canonicalize(fixture.path()).unwrap();
+            std::fs::write(root.join("note.md"), b"old").unwrap();
+            let dir = open_root(&root, false).unwrap().unwrap();
+            let result = atomic_write_with(
+                &dir,
+                c"note.md",
+                &[],
+                |file| file.write_all(b"new"),
+                |_| Err(io::Error::other("injected rename failure")),
+            );
+            assert!(result.unwrap_err().to_string().contains("rename failure"));
+            assert_eq!(std::fs::read(root.join("note.md")).unwrap(), b"old");
+            assert_eq!(std::fs::read_dir(root).unwrap().count(), 1);
+        }
+
+        #[test]
+        fn followup_final_durability_error_is_not_reported_as_success() {
+            let fixture = tempfile::tempdir().unwrap();
+            let root = std::fs::canonicalize(fixture.path()).unwrap();
+            let storage = Storage::open_in_memory().unwrap();
+            let batch = vec![(PathBuf::from("note.md"), "complete file".into())];
+            let result = write_batch_with_barrier(&root, &batch, &storage, |_| {
+                Err(io::Error::other("injected final durability failure"))
+            });
+            assert!(result
+                .unwrap_err()
+                .to_string()
+                .contains("durability failure"));
+            assert_eq!(
+                std::fs::read_to_string(root.join("note.md")).unwrap(),
+                "complete file"
+            );
+        }
+
+        #[test]
+        fn p0_directory_swap_to_symlink_cannot_redirect_write() {
+            let fixture = tempfile::tempdir().unwrap();
+            let root = std::fs::canonicalize(fixture.path()).unwrap();
+            std::fs::create_dir(root.join("owned")).unwrap();
+            std::fs::create_dir(root.join("outside")).unwrap();
+            let dir = open_root(&root.join("owned"), false).unwrap().unwrap();
+            std::fs::rename(root.join("owned"), root.join("pinned")).unwrap();
+            std::os::unix::fs::symlink(root.join("outside"), root.join("owned")).unwrap();
+            atomic_write(&dir, c"note.md", &[], |f| f.write_all(b"safe")).unwrap();
+            assert_eq!(std::fs::read_dir(root.join("outside")).unwrap().count(), 0);
+            assert_eq!(std::fs::read(root.join("pinned/note.md")).unwrap(), b"safe");
+        }
+
+        #[cfg(target_os = "macos")]
+        #[test]
+        fn p0_review_macos_acl_is_rejected_despite_private_mode_bits() {
+            use std::os::unix::fs::PermissionsExt;
+            let fixture = tempfile::tempdir().unwrap();
+            let root = std::fs::canonicalize(fixture.path()).unwrap();
+            let status = std::process::Command::new("/bin/chmod")
+                .args([
+                    "+a",
+                    "everyone allow add_file,delete_child,file_inherit,directory_inherit",
+                ])
+                .arg(&root)
+                .status()
+                .unwrap();
+            assert!(status.success(), "ACL fixture must be created");
+            assert_eq!(
+                std::fs::metadata(&root).unwrap().permissions().mode() & 0o022,
+                0
+            );
+            assert!(
+                open_root(&root, false).is_err(),
+                "mode 0700 does not exclude macOS ACL grants"
+            );
+        }
+
+        #[cfg(target_os = "linux")]
+        #[test]
+        fn followup_linux_default_acl_is_rejected_despite_private_mode_bits() {
+            let fixture = tempfile::tempdir().unwrap();
+            let root = std::fs::canonicalize(fixture.path()).unwrap();
+            let dir = open_root(&root, false).unwrap().unwrap();
+            // Linux POSIX ACL xattr v2, including a named numeric user. A
+            // default ACL does not change the directory's access mode bits.
+            let mut value = 2u32.to_le_bytes().to_vec();
+            for (tag, permissions, id) in [
+                (0x01u16, 7u16, u32::MAX),
+                (0x02, 7, 1_000_001),
+                (0x04, 0, u32::MAX),
+                (0x10, 7, u32::MAX),
+                (0x20, 0, u32::MAX),
+            ] {
+                value.extend(tag.to_le_bytes());
+                value.extend(permissions.to_le_bytes());
+                value.extend(id.to_le_bytes());
+            }
+            rustix::fs::fsetxattr(
+                &dir,
+                "system.posix_acl_default",
+                &value,
+                rustix::fs::XattrFlags::empty(),
+            )
+            .expect("Linux test filesystem must support the ACL fixture");
+            assert_eq!(dir.metadata().unwrap().mode() & 0o022, 0);
+            assert!(open_root(&root, false).is_err());
+        }
+    }
+}
+
+#[cfg(test)]
+mod component_tests {
+    use super::*;
+
+    #[test]
+    fn p0_components_reject_both_platforms_and_control_characters() {
+        for name in [
+            "",
+            ".",
+            "..",
+            "../../x",
+            "/absolute",
+            "a/b",
+            "a\\b",
+            "C:drive",
+            "C:\\root",
+            "a\0b",
+            "a\nb",
+            "NUL",
+            "COM1.txt",
+            "trailing.",
+            "trailing ",
+        ] {
+            assert!(validate_component(name).is_err(), "{name:?}");
+        }
+        for name in ["São-Paulo", "東京", "tag", "2026-11-01"] {
+            assert!(validate_component(name).is_ok(), "{name:?}");
+        }
+    }
+}
diff --git a/src/mcp/handlers/markdown_export/export/filesystem/acl.rs b/src/mcp/handlers/markdown_export/export/filesystem/acl.rs
new file mode 100644
index 0000000000000000000000000000000000000000..51073789c6290372395b916e860464c5c3e371a0
--- /dev/null
+++ b/src/mcp/handlers/markdown_export/export/filesystem/acl.rs
@@ -0,0 +1,125 @@
+//! Refuse extended/default ACLs rather than infer effective access from mode
+//! bits. In particular, macOS ACL grants are independent of group mode bits.
+
+use std::fs::File;
+use std::io;
+#[cfg(any(target_os = "macos", target_os = "linux"))]
+use std::os::fd::AsRawFd;
+
+#[cfg(target_os = "macos")]
+pub(super) fn check_directory(dir: &File) -> io::Result<()> {
+    // Declarations/constants match the macOS SDK's sys/acl.h. ACL objects are
+    // opaque and allocated/freed by libSystem; no Rust layout is assumed.
+    unsafe extern "C" {
+        fn acl_get_fd_np(fd: libc::c_int, kind: libc::c_int) -> *mut libc::c_void;
+        fn acl_get_entry(
+            acl: *mut libc::c_void,
+            entry: libc::c_int,
+            out: *mut *mut libc::c_void,
+        ) -> libc::c_int;
+        fn acl_free(object: *mut libc::c_void) -> libc::c_int;
+    }
+    const ACL_TYPE_EXTENDED: libc::c_int = 0x100;
+    const ACL_FIRST_ENTRY: libc::c_int = 0;
+    // SAFETY: dir is a live directory fd; the SDK defines this ACL type.
+    let acl = unsafe { acl_get_fd_np(dir.as_raw_fd(), ACL_TYPE_EXTENDED) };
+    if acl.is_null() {
+        let error = io::Error::last_os_error();
+        // macOS reports ENOENT when the vnode has no extended ACL. Other
+        // errors, including unavailable inspection, do not grant trust.
+        return if error.raw_os_error() == Some(libc::ENOENT) {
+            Ok(())
+        } else {
+            Err(error)
+        };
+    }
+    let mut entry = std::ptr::null_mut();
+    // SAFETY: acl was returned by libSystem; entry points to writable pointer storage.
+    let first = unsafe { acl_get_entry(acl, ACL_FIRST_ENTRY, &mut entry) };
+    let result = if first == 0 {
+        Err(io::Error::new(
+            io::ErrorKind::PermissionDenied,
+            "export directories must not have ACLs (including deny-only entries); use a private subdirectory without an ACL",
+        ))
+    } else {
+        let error = io::Error::last_os_error();
+        // On macOS an out-of-range first-entry index in an empty ACL is EINVAL
+        // (acl_get_entry(3)); this differs from the POSIX/Linux return convention.
+        if error.raw_os_error() == Some(libc::EINVAL) {
+            Ok(())
+        } else {
+            Err(error)
+        }
+    };
+    // SAFETY: this is the owned ACL pointer returned above, released exactly once.
+    if unsafe { acl_free(acl) } < 0 {
+        return Err(io::Error::last_os_error());
+    }
+    result
+}
+
+#[cfg(target_os = "linux")]
+pub(super) fn check_directory(dir: &File) -> io::Result<()> {
+    // Linux exposes POSIX access/default ACLs and extensions such as NFSv4
+    // ACLs as extended attributes. Inspection errors are explicit, not a fallback.
+    // listxattr may omit inaccessible names, so also probe known ACL namespaces
+    // directly; permission errors must not be mistaken for absence.
+    for name in [
+        c"system.posix_acl_access",
+        c"system.posix_acl_default",
+        c"system.nfs4_acl",
+        c"system.richacl",
+        c"system.cifs_acl",
+        c"security.NTACL",
+    ] {
+        // SAFETY: live directory fd, NUL-terminated name, size-only query.
+        let size =
+            unsafe { libc::fgetxattr(dir.as_raw_fd(), name.as_ptr(), std::ptr::null_mut(), 0) };
+        if size >= 0 {
+            return Err(io::Error::new(
+                io::ErrorKind::PermissionDenied,
+                "export directory has an ACL attribute",
+            ));
+        }
+        let error = io::Error::last_os_error();
+        if !matches!(error.raw_os_error(), Some(libc::ENODATA | libc::ENOTSUP)) {
+            return Err(error);
+        }
+    }
+    // SAFETY: size-only query of a live directory; no destination buffer is supplied.
+    let size = unsafe { libc::flistxattr(dir.as_raw_fd(), std::ptr::null_mut(), 0) };
+    if size < 0 {
+        return Err(io::Error::last_os_error());
+    }
+    if size == 0 {
+        return Ok(());
+    }
+    if size > 65_536 {
+        return Err(io::Error::other(
+            "directory attribute list exceeds inspection limit",
+        ));
+    }
+    let mut names = vec![0u8; size as usize];
+    // SAFETY: names has size writable bytes and dir remains a live descriptor.
+    let read = unsafe { libc::flistxattr(dir.as_raw_fd(), names.as_mut_ptr().cast(), names.len()) };
+    if read < 0 {
+        return Err(io::Error::last_os_error());
+    }
+    if read as usize > names.len() {
+        return Err(io::Error::other("invalid directory attribute list size"));
+    }
+    names.truncate(read as usize);
+    if !names.is_empty() && names.last() != Some(&0) {
+        return Err(io::Error::other("unterminated directory attribute list"));
+    }
+    if names.split(|&b| b == 0).any(|name| {
+        name.windows(3)
+            .any(|part| part.eq_ignore_ascii_case(b"acl"))
+    }) {
+        return Err(io::Error::new(
+            io::ErrorKind::PermissionDenied,
+            "export directories must not have access, default or extended ACL attributes",
+        ));
+    }
+    Ok(())
+}
diff --git a/src/mcp/handlers/markdown_export/export/filesystem/reader.rs b/src/mcp/handlers/markdown_export/export/filesystem/reader.rs
new file mode 100644
index 0000000000000000000000000000000000000000..8fe5298500d42e9224f38d7d1be26ddf898c2693
--- /dev/null
+++ b/src/mcp/handlers/markdown_export/export/filesystem/reader.rs
@@ -0,0 +1,268 @@
+//! Bounded, descriptor-relative Markdown input. Never follow input links or
+//! transiently open an existing SQLite-bearing inode to inspect it.
+
+use std::collections::BTreeSet;
+use std::ffi::{CString, OsString};
+use std::fs::File;
+use std::io::{self, Read};
+use std::os::unix::ffi::OsStringExt;
+use std::os::unix::fs::MetadataExt;
+use std::path::Path;
+
+use rustix::fs::{AtFlags, Dir, FileType, Mode, OFlags};
+
+use super::unix::{check_file, open_child, open_root, protected_inodes, trusted_directory};
+use crate::storage::Storage;
+
+use super::MarkdownSource;
+
+#[derive(Clone, Copy)]
+struct Limits {
+    depth: usize,
+    files: usize,
+    entries: usize,
+    file_bytes: usize,
+    total_bytes: usize,
+}
+
+impl Default for Limits {
+    fn default() -> Self {
+        Self {
+            depth: 32,
+            files: 10_000,
+            entries: 20_000,
+            file_bytes: 4 * 1024 * 1024,
+            total_bytes: 64 * 1024 * 1024,
+        }
+    }
+}
+
+fn invalid(message: &str) -> io::Error {
+    io::Error::new(io::ErrorKind::InvalidInput, message)
+}
+
+struct Walk<'a> {
+    root: &'a Path,
+    storage: &'a Storage,
+    limits: Limits,
+    protected: Vec<(u64, u64)>,
+    visited: BTreeSet<(u64, u64)>,
+    entries: usize,
+    bytes: usize,
+    sources: Vec<MarkdownSource>,
+}
+
+impl Walk<'_> {
+    fn directory(&mut self, dir: &File, relative: &Path, depth: usize) -> io::Result<()> {
+        if depth > self.limits.depth {
+            return Err(invalid("Markdown input exceeds directory depth limit"));
+        }
+        trusted_directory(dir)?;
+        let metadata = dir.metadata()?;
+        if !self.visited.insert((metadata.dev(), metadata.ino())) {
+            return Err(invalid("Markdown input contains a repeated directory"));
+        }
+        let mut names = Vec::new();
+        for entry in Dir::read_from(dir)? {
+            let entry = entry?;
+            let name = entry.file_name();
+            if matches!(name.to_bytes(), b"." | b"..") {
+                continue;
+            }
+            self.entries += 1;
+            if self.entries > self.limits.entries {
+                return Err(invalid("Markdown input exceeds directory entry limit"));
+            }
+            // Count hidden entries, but never stat/open/walk vault metadata,
+            // trash or their links below the explicitly selected input root.
+            if name.to_bytes().starts_with(b".") {
+                continue;
+            }
+            names.push(CString::from(name));
+        }
+        names.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
+        for name in names {
+            let leaf = OsString::from_vec(name.as_bytes().to_vec());
+            let path = relative.join(leaf);
+            let name_str = path
+                .file_name()
+                .and_then(|value| value.to_str())
+                .ok_or_else(|| invalid("Markdown input names must be UTF-8"))?;
+            let stat = rustix::fs::statat(dir, &name, AtFlags::SYMLINK_NOFOLLOW)?;
+            match FileType::from_raw_mode(stat.st_mode) {
+                FileType::Directory => {
+                    let child = open_child(dir, &name, false)?
+                        .ok_or_else(|| invalid("Markdown input directory disappeared"))?;
+                    self.directory(&child, &path, depth + 1)?;
+                }
+                FileType::Symlink => return Err(invalid("Markdown input links are not allowed")),
+                kind => {
+                    let lower = name_str.to_ascii_lowercase();
+                    if path.extension().and_then(|ext| ext.to_str()) != Some("md")
+                        || name_str.starts_with(['.', '_'])
+                        || matches!(lower.as_str(), "index.md" | "readme.md")
+                    {
+                        continue;
+                    }
+                    if kind != FileType::RegularFile {
+                        return Err(invalid("Markdown input must contain regular files"));
+                    }
+                    if self.sources.len() >= self.limits.files {
+                        return Err(invalid("Markdown input exceeds file count limit"));
+                    }
+                    // Metadata-only checks precede regular-file open: closing an
+                    // alias of SQLite could otherwise release process fcntl locks.
+                    self.storage
+                        .refuse_active_sqlite_artifact(self.root.join(&path))
+                        .map_err(io::Error::other)?;
+                    check_file(dir, &name, &self.protected)?;
+                    let remaining = self.limits.total_bytes.saturating_sub(self.bytes);
+                    let limit = self.limits.file_bytes.min(remaining);
+                    if stat.st_size < 0 || stat.st_size as u64 > limit as u64 {
+                        return Err(invalid("Markdown input exceeds byte limit"));
+                    }
+                    let fd = rustix::fs::openat(
+                        dir,
+                        &name,
+                        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
+                        Mode::empty(),
+                    )?;
+                    let file = File::from(fd);
+                    let opened = file.metadata()?;
+                    if !opened.is_file() || opened.nlink() != 1 {
+                        return Err(invalid("Markdown input file changed or has multiple links"));
+                    }
+                    let mut bytes = Vec::new();
+                    file.take(limit as u64 + 1).read_to_end(&mut bytes)?;
+                    if bytes.len() > limit {
+                        return Err(invalid("Markdown input exceeds byte limit"));
+                    }
+                    self.bytes += bytes.len();
+                    let raw = String::from_utf8(bytes)
+                        .map_err(|_| invalid("Markdown input content must be UTF-8"))?;
+                    self.sources.push(MarkdownSource {
+                        relative: path,
+                        raw,
+                    });
+                }
+            }
+        }
+        Ok(())
+    }
+}
+
+pub(in crate::mcp::handlers::markdown_export) fn read_markdown(
+    root: &Path,
+    storage: &Storage,
+) -> io::Result<Vec<MarkdownSource>> {
+    read_with_limits(root, storage, Limits::default())
+}
+
+fn read_with_limits(
+    root: &Path,
+    storage: &Storage,
+    limits: Limits,
+) -> io::Result<Vec<MarkdownSource>> {
+    storage
+        .refuse_active_sqlite_artifact(root)
+        .map_err(io::Error::other)?;
+    let dir = open_root(root, false)?.ok_or_else(|| invalid("Markdown input root not found"))?;
+    let mut walk = Walk {
+        root,
+        storage,
+        limits,
+        protected: protected_inodes(storage),
+        visited: BTreeSet::new(),
+        entries: 0,
+        bytes: 0,
+        sources: Vec::new(),
+    };
+    walk.directory(&dir, Path::new(""), 0)?;
+    Ok(walk.sources)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn hidden_vault_directories_and_their_links_are_not_import_sources() {
+        let fixture = tempfile::tempdir().unwrap();
+        let root = fixture.path().canonicalize().unwrap();
+        let storage = Storage::open_in_memory().unwrap();
+        std::fs::write(root.join("note.md"), "current note").unwrap();
+        for name in [".trash", ".obsidian", ".git"] {
+            std::fs::create_dir(root.join(name)).unwrap();
+        }
+        std::fs::write(root.join(".trash/old.md"), "deleted note").unwrap();
+        std::os::unix::fs::symlink(root.join("note.md"), root.join(".obsidian/link.md")).unwrap();
+        let sources = read_markdown(&root, &storage).unwrap();
+        assert_eq!(sources.len(), 1);
+        assert_eq!(sources[0].relative, Path::new("note.md"));
+        assert_eq!(sources[0].raw, "current note");
+    }
+
+    #[test]
+    fn hidden_descendants_do_not_consume_walk_depth_or_entry_budget() {
+        let fixture = tempfile::tempdir().unwrap();
+        let root = fixture.path().canonicalize().unwrap();
+        let storage = Storage::open_in_memory().unwrap();
+        std::fs::write(root.join("note.md"), "current note").unwrap();
+        std::fs::create_dir_all(root.join(".git/objects")).unwrap();
+        for n in 0..20 {
+            std::fs::write(root.join(format!(".git/objects/{n}.md")), "internal").unwrap();
+        }
+        let sources = read_with_limits(
+            &root,
+            &storage,
+            Limits {
+                depth: 0,
+                entries: 2,
+                ..Limits::default()
+            },
+        )
+        .unwrap();
+        assert_eq!(sources.len(), 1);
+        assert_eq!(sources[0].raw, "current note");
+    }
+
+    #[test]
+    fn input_quotas_bound_files_bytes_entries_and_depth() {
+        let fixture = tempfile::tempdir().unwrap();
+        let root = fixture.path().canonicalize().unwrap();
+        let storage = Storage::open_in_memory().unwrap();
+        std::fs::write(root.join("a.md"), "12345").unwrap();
+        std::fs::write(root.join("b.md"), "67890").unwrap();
+        for limits in [
+            Limits {
+                file_bytes: 4,
+                ..Limits::default()
+            },
+            Limits {
+                total_bytes: 9,
+                ..Limits::default()
+            },
+            Limits {
+                files: 1,
+                ..Limits::default()
+            },
+            Limits {
+                entries: 1,
+                ..Limits::default()
+            },
+        ] {
+            assert!(read_with_limits(&root, &storage, limits).is_err());
+        }
+        std::fs::create_dir(root.join("nested")).unwrap();
+        assert!(read_with_limits(
+            &root,
+            &storage,
+            Limits {
+                depth: 0,
+                ..Limits::default()
+            }
+        )
+        .is_err());
+        assert_eq!(read_markdown(&root, &storage).unwrap().len(), 2);
+    }
+}
diff --git a/src/mcp/handlers/markdown_export/export/format.rs b/src/mcp/handlers/markdown_export/export/format.rs
index a2a484180b3de638bc187b234f915f9bddb2a1c4..a4f2df95ed0afee0a8bb830d7e1a03ab7d840e92 100644
--- a/src/mcp/handlers/markdown_export/export/format.rs
+++ b/src/mcp/handlers/markdown_export/export/format.rs
@@ -8 +8 @@ pub(super) fn compute_file_subdir(group: &str, mem: &Value) -> Option<String> {
-    match group {
+    let label = match group {
@@ -39,0 +40,18 @@ pub(super) fn compute_file_subdir(group: &str, mem: &Value) -> Option<String> {
+    };
+    label.map(|label| encode_group_component(&label))
+}
+
+/// Keep existing safe labels, but encode namespaces, reserved names and long
+/// components. Reserve the escape prefix so a literal label cannot impersonate
+/// an encoded output. Hash the original bytes, not the lossy slug.
+fn encode_group_component(label: &str) -> String {
+    const PREFIX: &str = "~e1-";
+    if label.len() <= 160
+        && !label
+            .as_bytes()
+            .get(..PREFIX.len())
+            .is_some_and(|prefix| prefix.eq_ignore_ascii_case(PREFIX.as_bytes()))
+        && !label.eq_ignore_ascii_case("index.md")
+        && super::filesystem::validate_component(label).is_ok()
+    {
+        return label.to_string();
@@ -40,0 +59,19 @@ pub(super) fn compute_file_subdir(group: &str, mem: &Value) -> Option<String> {
+    use sha2::{Digest, Sha256};
+    let hash = format!("{:x}", Sha256::digest(label.as_bytes()));
+    let suffix: String = hash.chars().take(24).collect();
+    let slug: String = label
+        .chars()
+        .take(24)
+        .map(|c| {
+            if c.is_alphanumeric() || matches!(c, '-' | '_') {
+                c.to_ascii_lowercase()
+            } else {
+                '-'
+            }
+        })
+        .collect();
+    let slug = slug.trim_matches('-');
+    format!(
+        "{PREFIX}{}-{suffix}",
+        if slug.is_empty() { "label" } else { slug }
+    )
@@ -154 +191 @@ pub(super) fn build_index_markdown(
-    sorted_types.sort_by(|a, b| b.1.cmp(a.1));
+    sorted_types.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
diff --git a/src/mcp/handlers/markdown_export/export/format_tests.rs b/src/mcp/handlers/markdown_export/export/format_tests.rs
index 3dc185c03a465dcd70100b659765c60799544d93..dc38b324b13c9b2d8b32ddde9e0001343c62a06d 100644
--- a/src/mcp/handlers/markdown_export/export/format_tests.rs
+++ b/src/mcp/handlers/markdown_export/export/format_tests.rs
@@ -239,0 +240,14 @@ fn test_content_hash_deterministic() {
+
+#[test]
+fn p0_index_tied_counts_have_a_total_order() {
+    let names: Vec<String> = (0..32).map(|n| format!("type{n:02}")).collect();
+    for _ in 0..8 {
+        let counts: HashMap<String, usize> = names.iter().cloned().map(|s| (s, 1)).collect();
+        let index = build_index_markdown("work", &[], &counts, &HashMap::new(), "flat");
+        let positions: Vec<usize> = names
+            .iter()
+            .map(|s| index.find(&format!("**{s}s/**")).unwrap())
+            .collect();
+        assert!(positions.windows(2).all(|p| p[0] < p[1]), "{index}");
+    }
+}
diff --git a/src/mcp/handlers/markdown_export/export/handler.rs b/src/mcp/handlers/markdown_export/export/handler.rs
index 23e64d52772e98ed760cc49f481f14b3351071f0..5cdc2cfa6077ea024f5fcf32a0ae13750b204129 100644
--- a/src/mcp/handlers/markdown_export/export/handler.rs
+++ b/src/mcp/handlers/markdown_export/export/handler.rs
@@ -2 +2 @@ use std::collections::HashMap;
-use std::fs;
+use std::path::PathBuf;
@@ -5,0 +6 @@ use serde_json::{json, Value};
+use super::filesystem::{validate_component, write_batch};
@@ -9 +10 @@ use super::format::{
-use super::query::{build_related_map, query_workspace_memories};
+use super::query::query_export_snapshot;
@@ -14,3 +15,12 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-    let workspace = match params.get("workspace").and_then(|v| v.as_str()) {
-        Some(w) => w.to_string(),
-        None => return json!({"error": "workspace is required"}),
+    // Direct CLI/library callers receive the same authorization as dispatch.
+    if let Some(denial) = crate::mcp::permission::check_tool_authorization(
+        None,
+        "memory_export_markdown",
+        &params,
+        ctx.principal.as_ref(),
+    ) {
+        return denial;
+    }
+    let workspace = match params.get("workspace").and_then(Value::as_str) {
+        Some(w) => w,
+        None => return json!({"error":"workspace is required"}),
@@ -18,2 +28 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-
-    let default_dir = format!("./engram-export/{}", workspace);
+    let default_dir = format!("./engram-export/{workspace}");
@@ -22 +31 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-        .and_then(|v| v.as_str())
+        .and_then(Value::as_str)
@@ -24 +32,0 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-
@@ -27 +35 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-        Err(e) => return json!({"error": format!("Invalid output_dir: {}", e)}),
+        Err(e) => return json!({"error":format!("Invalid output_dir: {e}")}),
@@ -29 +36,0 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-
@@ -32 +39 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-        .and_then(|v| v.as_bool())
+        .and_then(Value::as_bool)
@@ -34 +40,0 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-
@@ -37,15 +43,4 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-        .and_then(|v| v.as_str())
-        .unwrap_or("flat")
-        .to_string();
-
-    // 1. Query all memories in workspace
-    let memories = match query_workspace_memories(ctx, &workspace) {
-        Ok(m) => m,
-        Err(e) => return json!({"error": format!("Failed to query memories: {}", e)}),
-    };
-
-    if memories.is_empty() {
-        return json!({
-            "error": format!("No memories found in workspace '{}'", workspace),
-            "files_written": 0
-        });
+        .and_then(Value::as_str)
+        .unwrap_or("flat");
+    if !matches!(group, "flat" | "day" | "workspace" | "type" | "entity") {
+        return json!({"error":"Invalid export group"});
@@ -53,10 +48,3 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-
-    // 2. If include_links, query cross-references for all memory IDs
-    let related_map: HashMap<i64, Vec<(i64, String)>> = if include_links {
-        let memory_ids: Vec<i64> = memories
-            .iter()
-            .filter_map(|m| m.get("id").and_then(|v| v.as_i64()))
-            .collect();
-        build_related_map(ctx, &memory_ids)
-    } else {
-        HashMap::new()
+    let (memories, related_map) = match query_export_snapshot(ctx, workspace, include_links) {
+        Ok(result) => result,
+        Err(e) => return json!({"error":format!("Failed to read export snapshot: {e}")}),
@@ -64,4 +52,2 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-
-    // 3. Create root output directory
-    if let Err(e) = fs::create_dir_all(&output_path) {
-        return json!({"error": format!("Failed to create output directory: {}", e)});
+    if memories.is_empty() {
+        return json!({"error":format!("No memories found in workspace '{workspace}'"),"files_written":0});
@@ -69,13 +55 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-
-    // First pass: compute filenames (stem only, no subdir)
-    let id_to_filename: HashMap<i64, String> = memories
-        .iter()
-        .filter_map(|mem| {
-            let id = mem.get("id").and_then(|v| v.as_i64())?;
-            let content = mem.get("content").and_then(|v| v.as_str()).unwrap_or("");
-            let title = content.lines().next().unwrap_or("untitled");
-            let sanitized = sanitize_filename(title);
-            Some((id, format!("{}-{}", id, sanitized)))
-        })
-        .collect();
-
+    let mut id_to_filename = HashMap::new();
@@ -82,0 +57 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
+    let mut paths = Vec::with_capacity(memories.len());
@@ -84,24 +59,2 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-        let mem_type = mem
-            .get("memory_type")
-            .and_then(|v| v.as_str())
-            .unwrap_or("note");
-        *type_counts.entry(mem_type.to_string()).or_insert(0) += 1;
-    }
-
-    // Second pass: write files
-    let mut files_written: usize = 0;
-    for mem in &memories {
-        let id = mem.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
-
-        // Determine target directory based on grouping mode
-        let target_dir = match compute_file_subdir(&group, mem) {
-            Some(subdir) => {
-                let d = output_path.join(&subdir);
-                if let Err(e) = fs::create_dir_all(&d) {
-                    return json!({
-                        "error": format!("Failed to create directory {}: {}", d.display(), e)
-                    });
-                }
-                d
-            }
-            None => output_path.clone(),
+        let Some(id) = mem["id"].as_i64() else {
+            return json!({"error":"Invalid exported memory id"});
@@ -109,11 +62,11 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-
-        let filename = id_to_filename
-            .get(&id)
-            .cloned()
-            .unwrap_or_else(|| format!("{}", id));
-        let file_path = target_dir.join(format!("{}.md", filename));
-
-        let md = format_memory_markdown(mem, include_links, &related_map, &id_to_filename);
-
-        if let Err(e) = fs::write(&file_path, &md) {
-            return json!({"error": format!("Failed to write {}: {}", file_path.display(), e)});
+        let content = mem["content"].as_str().unwrap_or("");
+        let filename = format!(
+            "{id}-{}",
+            sanitize_filename(content.lines().next().unwrap_or("untitled"))
+        );
+        let mut relative = PathBuf::new();
+        if let Some(subdir) = compute_file_subdir(group, mem) {
+            if validate_component(&subdir).is_err() {
+                return json!({"error":"Unsafe export grouping component"});
+            }
+            relative.push(subdir);
@@ -121 +74,6 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-        files_written += 1;
+        relative.push(format!("{filename}.md"));
+        paths.push(relative);
+        id_to_filename.insert(id, filename);
+        *type_counts
+            .entry(mem["memory_type"].as_str().unwrap_or("note").to_string())
+            .or_default() += 1;
@@ -123,7 +81,17 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-
-    // 4. Write index.md in root
-    let index_path = output_path.join("index.md");
-    let index = build_index_markdown(&workspace, &memories, &type_counts, &id_to_filename, &group);
-
-    if let Err(e) = fs::write(&index_path, &index) {
-        return json!({"error": format!("Failed to write index: {}", e)});
+    // Validate/read/render the complete batch before any directory or file is created.
+    let mut batch: Vec<_> = memories
+        .iter()
+        .zip(paths)
+        .map(|(mem, path)| {
+            (
+                path,
+                format_memory_markdown(mem, include_links, &related_map, &id_to_filename),
+            )
+        })
+        .collect();
+    batch.push((
+        PathBuf::from("index.md"),
+        build_index_markdown(workspace, &memories, &type_counts, &id_to_filename, group),
+    ));
+    if let Err(e) = write_batch(&output_path, &batch, &ctx.storage) {
+        return json!({"error":format!("Failed to write export (batch may be incomplete): {e}")});
@@ -131 +98,0 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-
@@ -133,6 +100,3 @@ pub fn memory_export_markdown(ctx: &HandlerContext, params: Value) -> Value {
-        "files_written": files_written + 1,
-        "output_dir": output_path.to_string_lossy(),
-        "index_path": index_path.to_string_lossy(),
-        "memories_exported": memories.len(),
-        "type_breakdown": type_counts,
-        "group": group
+        "files_written":batch.len(),"output_dir":output_path.to_string_lossy(),
+        "index_path":output_path.join("index.md").to_string_lossy(),
+        "memories_exported":memories.len(),"type_breakdown":type_counts,"group":group
diff --git a/src/mcp/handlers/markdown_export/export/handler_tests.rs b/src/mcp/handlers/markdown_export/export/handler_tests.rs
index c5c75f8c656f974fe1e9b03317e6b3a3a8d45e7e..cc3004f3a8594d47a5b0a606bd0c0f4625d619a9 100644
--- a/src/mcp/handlers/markdown_export/export/handler_tests.rs
+++ b/src/mcp/handlers/markdown_export/export/handler_tests.rs
@@ -8 +8 @@ use std::sync::Arc;
-fn ctx() -> crate::mcp::handlers::HandlerContext {
+pub(super) fn ctx() -> crate::mcp::handlers::HandlerContext {
@@ -69,0 +70,272 @@ fn export_markdown_uses_current_scope_schema() {
+
+pub(super) fn p0_memory(
+    c: &crate::mcp::handlers::HandlerContext,
+    content: &str,
+    tags: Vec<String>,
+) -> i64 {
+    c.storage
+        .with_transaction(|conn| {
+            Ok(create_memory(
+                conn,
+                &CreateMemoryInput {
+                    content: content.into(),
+                    memory_type: MemoryType::Note,
+                    workspace: Some("markdown_export_schema".into()),
+                    tags,
+                    ..Default::default()
+                },
+            )?
+            .id)
+        })
+        .unwrap()
+}
+
+pub(super) fn p0_export(
+    c: &crate::mcp::handlers::HandlerContext,
+    path: &std::path::Path,
+    group: &str,
+) -> serde_json::Value {
+    memory_export_markdown(
+        c,
+        json!({
+            "workspace":"markdown_export_schema", "output_dir":path, "group":group
+        }),
+    )
+}
+
+#[test]
+#[ignore = "manual local I/O measurement; no timing threshold in CI"]
+fn followup_manual_export_300_probe() {
+    for run in 0..3 {
+        let c = ctx();
+        for n in 0..300 {
+            p0_memory(
+                &c,
+                &format!("Synthetic benchmark {n:03}\n{}", "x".repeat(224)),
+                vec!["benchmark".into()],
+            );
+        }
+        c.storage.with_connection(|conn| {conn.execute("UPDATE memories SET created_at='2026-01-01T00:00:00+00:00',updated_at='2026-01-01T00:00:00+00:00'",[])?;Ok(())}).unwrap();
+        let fixture = tempfile::tempdir().unwrap();
+        let start = std::time::Instant::now();
+        let result = p0_export(&c, fixture.path(), "flat");
+        let elapsed = start.elapsed().as_secs_f64();
+        assert!(result.get("error").is_none(), "{result}");
+        assert_eq!(result["memories_exported"], 300);
+        let files: Vec<_> = std::fs::read_dir(fixture.path())
+            .unwrap()
+            .map(|e| e.unwrap().path())
+            .collect();
+        let bytes: u64 = files
+            .iter()
+            .map(|p| std::fs::metadata(p).unwrap().len())
+            .sum();
+        println!(
+            "ENGRAM_EXPORT_PROBE {}",
+            serde_json::json!({"run":run,"memories":300,"files":files.len(),"bytes":bytes,"seconds":elapsed})
+        );
+    }
+}
+
+#[test]
+fn followup_entity_group_accepts_tags_generated_by_engram() {
+    for tag in [
+        "session:abc",
+        "category:architecture",
+        "confidence:0.95",
+        "file:src/main.rs",
+        "trace:trace-1",
+        "aux",
+        "CON.txt",
+    ] {
+        let c = ctx();
+        let id = p0_memory(&c, "Namespaced note", vec![tag.into()]);
+        let fixture = tempfile::tempdir().unwrap();
+        let r = p0_export(&c, fixture.path(), "entity");
+        assert!(r.get("error").is_none(), "tag={tag}: {r}");
+        let dirs: Vec<_> = std::fs::read_dir(fixture.path())
+            .unwrap()
+            .map(|e| e.unwrap().path())
+            .filter(|p| p.is_dir())
+            .collect();
+        assert_eq!(dirs.len(), 1);
+        assert!(super::filesystem::validate_component(
+            dirs[0].file_name().unwrap().to_str().unwrap()
+        )
+        .is_ok());
+        assert!(dirs[0].join(format!("{id}-namespaced-note.md")).is_file());
+        let index = std::fs::read_to_string(fixture.path().join("index.md")).unwrap();
+        assert!(index.contains(dirs[0].file_name().unwrap().to_str().unwrap()));
+    }
+}
+
+#[test]
+fn p0_traversal_tag_is_encoded_without_writing_outside_root() {
+    let c = ctx();
+    p0_memory(&c, "Private note", vec!["../../escaped".into()]);
+    let fixture = tempfile::tempdir().unwrap();
+    let base = fixture.path().join("base");
+    std::fs::create_dir(&base).unwrap();
+    let out = base.join("export");
+    let r = p0_export(&c, &out, "entity");
+    assert!(r.get("error").is_none(), "{r}");
+    assert!(out.join("index.md").exists());
+    let dirs: Vec<_> = std::fs::read_dir(&out)
+        .unwrap()
+        .map(|entry| entry.unwrap().path())
+        .filter(|path| path.is_dir())
+        .collect();
+    assert_eq!(dirs.len(), 1);
+    assert!(dirs[0]
+        .file_name()
+        .unwrap()
+        .to_str()
+        .unwrap()
+        .starts_with("~e1-"));
+    assert!(!fixture.path().join("escaped").exists());
+}
+
+#[cfg(unix)]
+#[test]
+fn p0_symlink_directory_is_rejected() {
+    let c = ctx();
+    p0_memory(&c, "Private note", vec!["topic".into()]);
+    let f = tempfile::tempdir().unwrap();
+    let out = f.path().join("export");
+    let outside = f.path().join("outside");
+    std::fs::create_dir(&out).unwrap();
+    std::fs::create_dir(&outside).unwrap();
+    std::os::unix::fs::symlink(&outside, out.join("topic")).unwrap();
+    let r = p0_export(&c, &out, "entity");
+    assert!(r.get("error").is_some(), "{r}");
+    assert_eq!(std::fs::read_dir(out).unwrap().count(), 1);
+    assert_eq!(std::fs::read_dir(outside).unwrap().count(), 0);
+}
+
+#[cfg(unix)]
+#[test]
+fn p0_symlink_file_is_not_followed() {
+    let c = ctx();
+    let id = p0_memory(&c, "Private note", vec![]);
+    let f = tempfile::tempdir().unwrap();
+    let out = f.path().join("export");
+    std::fs::create_dir(&out).unwrap();
+    let sentinel = f.path().join("sentinel");
+    std::fs::write(&sentinel, b"keep").unwrap();
+    std::os::unix::fs::symlink(&sentinel, out.join(format!("{id}-private-note.md"))).unwrap();
+    let r = p0_export(&c, &out, "flat");
+    assert!(r.get("error").is_some(), "{r}");
+    assert_eq!(std::fs::read(sentinel).unwrap(), b"keep");
+    assert!(!out.join("index.md").exists());
+}
+
+#[test]
+fn p0_dispatch_read_only_cannot_create_files() {
+    let c = ctx();
+    make_memory(&c);
+    let f = tempfile::tempdir().unwrap();
+    let out = f.path().join("export");
+    let r = crate::mcp::handlers::dispatch(
+        &c,
+        "memory_export_markdown",
+        json!({
+            "workspace":"markdown_export_schema", "output_dir":out,"_permission_mode":"read_only"
+        }),
+    );
+    assert_eq!(r["error"]["code"], "permission_denied", "{r}");
+    assert!(
+        matches!(
+            r["error"]["required_mode"].as_str(),
+            Some("scoped_write" | "maintenance")
+        ),
+        "{r}"
+    );
+    assert!(!out.exists());
+}
+
+#[test]
+fn p0_invalid_row_fails_before_writing_valid_rows() {
+    let c = ctx();
+    make_memory(&c);
+    let id = p0_memory(&c, "Bad row", vec![]);
+    c.storage
+        .with_connection(|conn| {
+            conn.execute("UPDATE memories SET content=x'01' WHERE id=?1", [id])?;
+            Ok(())
+        })
+        .unwrap();
+    let f = tempfile::tempdir().unwrap();
+    let out = f.path().join("export");
+    let r = p0_export(&c, &out, "flat");
+    assert!(r.get("error").is_some(), "{r}");
+    assert!(!out.exists());
+}
+
+#[test]
+fn p0_relation_read_error_is_not_a_successful_export() {
+    let c = ctx();
+    make_memory(&c);
+    c.storage
+        .with_connection(|conn| {
+            conn.execute_batch("DROP TABLE crossrefs")?;
+            Ok(())
+        })
+        .unwrap();
+    let f = tempfile::tempdir().unwrap();
+    let out = f.path().join("export");
+    let r = p0_export(&c, &out, "flat");
+    assert!(r.get("error").is_some(), "{r}");
+    assert!(!out.exists());
+}
+
+#[test]
+fn p0_expired_memory_is_absent_from_files_index_and_links() {
+    let c = ctx();
+    let live = p0_memory(&c, "Live note", vec![]);
+    let expired = p0_memory(&c, "Expired private note", vec![]);
+    let future = p0_memory(&c, "Future Daily note", vec![]);
+    c.storage.with_connection(|conn| {
+        conn.execute("UPDATE memories SET tier='daily',expires_at='2000-01-01T00:00:00+00:00' WHERE id=?1",[expired])?;
+        conn.execute("UPDATE memories SET tier='daily',expires_at='2999-01-01T00:00:00+00:00' WHERE id=?1",[future])?;
+        conn.execute("INSERT INTO crossrefs(from_id,to_id,edge_type,score) VALUES(?1,?2,'supersedes',1.0)",[live,expired])?;
+        Ok(())
+    }).unwrap();
+    let f = tempfile::tempdir().unwrap();
+    let r = p0_export(&c, f.path(), "flat");
+    assert!(r.get("error").is_none(), "{r}");
+    assert_eq!(r["memories_exported"], 2, "{r}");
+    for entry in std::fs::read_dir(f.path()).unwrap() {
+        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap();
+        assert!(!text.contains("Expired private note"), "{text}");
+        assert!(!text.contains(&format!("[[{expired}-")), "{text}");
+        assert!(!text.contains(&format!("[[memory-{expired}")), "{text}");
+    }
+}
+
+#[test]
+fn p0_incoming_edge_is_not_rendered_as_outgoing() {
+    let c = ctx();
+    let a = p0_memory(&c, "New decision", vec![]);
+    let b = p0_memory(&c, "Old decision", vec![]);
+    c.storage
+        .with_connection(|conn| {
+            conn.execute(
+                "INSERT INTO crossrefs(from_id,to_id,edge_type,score) VALUES(?1,?2,'supersedes',1.0)",
+                [a, b],
+            )?;
+            Ok(())
+        })
+        .unwrap();
+    let f = tempfile::tempdir().unwrap();
+    let r = p0_export(&c, f.path(), "flat");
+    assert!(r.get("error").is_none(), "{r}");
+    let incoming = std::fs::read_to_string(f.path().join(format!("{b}-old-decision.md"))).unwrap();
+    assert!(
+        !incoming.contains(&format!("- supersedes [[{a}-new-decision]]")),
+        "{incoming}"
+    );
+    assert!(
+        incoming.contains(&format!("- incoming supersedes [[{a}-new-decision]]")),
+        "{incoming}"
+    );
+}
diff --git a/src/mcp/handlers/markdown_export/export/naming_tests.rs b/src/mcp/handlers/markdown_export/export/naming_tests.rs
index bac50aac803f3c87b8c5c8f8753985cacec9c55a..5ff3299e81093384109c4bbcdd8f1ba895d80a8d 100644
--- a/src/mcp/handlers/markdown_export/export/naming_tests.rs
+++ b/src/mcp/handlers/markdown_export/export/naming_tests.rs
@@ -151,0 +152,31 @@ fn test_grouping_by_entity_no_tags() {
+
+#[test]
+fn followup_encoded_groups_distinguish_replaced_characters_and_literal_outputs() {
+    let group = |tag: &str| {
+        compute_file_subdir(
+            "entity",
+            &json!({"tags":serde_json::to_string(&vec![tag]).unwrap()}),
+        )
+        .unwrap()
+    };
+    let slash = group("session:a/b");
+    let backslash = group("session:a\\b");
+    assert_ne!(slash, backslash);
+    assert_ne!(
+        group(&slash),
+        slash,
+        "literal tag must not impersonate an encoded tag"
+    );
+    assert!(
+        !group(&slash.to_ascii_uppercase()).eq_ignore_ascii_case(&slash),
+        "literal tag must not impersonate an encoded tag on case-insensitive filesystems"
+    );
+    for tag in ["session:abc", "../../escape", "aux", ""] {
+        let encoded = group(tag);
+        assert!(
+            super::filesystem::validate_component(&encoded).is_ok(),
+            "{tag}: {encoded}"
+        );
+    }
+    assert!(group(&"字".repeat(100)).len() <= 160);
+}
diff --git a/src/mcp/handlers/markdown_export/export/query.rs b/src/mcp/handlers/markdown_export/export/query.rs
index bd8945ef3b4b69adef3be9ca0198469830d9a8df..936c208e6cd3cb8112d5f879c3e0f18d58427ec5 100644
--- a/src/mcp/handlers/markdown_export/export/query.rs
+++ b/src/mcp/handlers/markdown_export/export/query.rs
@@ -1 +1 @@
-use std::collections::HashMap;
+use std::collections::{HashMap, HashSet};
@@ -2,0 +3 @@ use std::collections::HashMap;
+use rusqlite::Connection;
@@ -4,0 +6 @@ use serde_json::{json, Value};
+use crate::error::EngramError;
@@ -7 +9,5 @@ use crate::mcp::handlers::HandlerContext;
-pub(super) fn query_workspace_memories(
+pub(super) type RelatedMap = HashMap<i64, Vec<(i64, String)>>;
+
+/// Read memories and their eligible relations from one consistent SQLite snapshot.
+/// The transaction ends before rendering or filesystem I/O.
+pub(super) fn query_export_snapshot(
@@ -10 +16,2 @@ pub(super) fn query_workspace_memories(
-) -> Result<Vec<Value>, crate::error::EngramError> {
+    include_links: bool,
+) -> Result<(Vec<Value>, RelatedMap), EngramError> {
@@ -12,41 +19,18 @@ pub(super) fn query_workspace_memories(
-        let mut stmt = conn.prepare(
-            "SELECT m.id, m.content, m.memory_type, m.importance, m.workspace, m.tier,
-                    m.created_at, m.updated_at,
-                    (SELECT GROUP_CONCAT(t.name, ',')
-                     FROM memory_tags mt
-                     JOIN tags t ON mt.tag_id = t.id
-                     WHERE mt.memory_id = m.id) as tags,
-                    m.scope_type as scope, m.version, m.metadata, m.content_hash
-             FROM memories m
-             WHERE m.workspace = ?1
-               AND COALESCE(m.lifecycle_state, 'active') != 'archived'
-               AND m.valid_to IS NULL
-             ORDER BY m.memory_type, m.created_at",
-        )?;
-        let rows = stmt.query_map(
-            rusqlite::params_from_iter(std::iter::once(workspace)),
-            |row| {
-                let metadata_str: Option<String> = row.get(11)?;
-                let metadata: Value = metadata_str
-                    .as_deref()
-                    .and_then(|s| serde_json::from_str(s).ok())
-                    .unwrap_or(Value::Null);
-                Ok(json!({
-                    "id": row.get::<_, i64>(0)?,
-                    "content": row.get::<_, String>(1)?,
-                    "memory_type": row.get::<_, String>(2)?,
-                    "importance": row.get::<_, Option<f64>>(3)?,
-                    "workspace": row.get::<_, String>(4)?,
-                    "tier": row.get::<_, Option<String>>(5)?,
-                    "created_at": row.get::<_, String>(6)?,
-                    "updated_at": row.get::<_, Option<String>>(7)?,
-                    "tags": row.get::<_, Option<String>>(8)?,
-                    "scope": row.get::<_, Option<String>>(9)?,
-                    "version": row.get::<_, Option<i64>>(10)?,
-                    "metadata": metadata,
-                    "content_hash": row.get::<_, Option<String>>(12)?
-                }))
-            },
-        )?;
-        let memories: Vec<Value> = rows.filter_map(|r| r.ok()).collect();
-        Ok(memories)
+        let tx = conn.unchecked_transaction()?;
+        let cutoff = chrono::Utc::now().to_rfc3339();
+        let memories = query_workspace_memories_at(&tx, workspace, &cutoff)?;
+        let related = if include_links && !memories.is_empty() {
+            let ids: HashSet<i64> = memories
+                .iter()
+                .map(|m| {
+                    m["id"].as_i64().ok_or_else(|| {
+                        EngramError::InvalidInput("invalid exported memory id".into())
+                    })
+                })
+                .collect::<Result<_, _>>()?;
+            build_related_map(&tx, workspace, &ids)?
+        } else {
+            RelatedMap::new()
+        };
+        tx.commit()?;
+        Ok((memories, related))
@@ -56,11 +40,59 @@ pub(super) fn query_workspace_memories(
-/// Build a map of memory_id -> [(related_id, relation_type)].
-///
-/// Issues a single batched query over all `memory_ids` instead of one per ID.
-/// DB errors are logged and result in an empty map rather than being silently
-/// discarded.
-pub(super) fn build_related_map(
-    ctx: &HandlerContext,
-    memory_ids: &[i64],
-) -> HashMap<i64, Vec<(i64, String)>> {
-    if memory_ids.is_empty() {
-        return HashMap::new();
+pub(super) fn query_workspace_memories_at(
+    conn: &Connection,
+    workspace: &str,
+    cutoff: &str,
+) -> Result<Vec<Value>, EngramError> {
+    let cutoff = chrono::DateTime::parse_from_rfc3339(cutoff)
+        .map_err(|_| EngramError::InvalidInput("invalid export cutoff".into()))?;
+    let mut stmt = conn.prepare(
+        "SELECT m.id,m.content,m.memory_type,m.importance,m.workspace,m.tier,
+                m.created_at,m.updated_at,
+                (SELECT json_group_array(name) FROM (
+                    SELECT t.name AS name FROM memory_tags mt
+                    JOIN tags t ON mt.tag_id=t.id
+                    WHERE mt.memory_id=m.id ORDER BY t.name,t.id
+                )) AS tags,
+                m.scope_type,m.version,m.metadata,m.content_hash,m.expires_at
+         FROM memories m WHERE m.workspace=?1
+           AND COALESCE(m.lifecycle_state,'active') != 'archived'
+           AND m.valid_to IS NULL
+         ORDER BY m.memory_type,m.created_at,m.id",
+    )?;
+    let mut rows = stmt.query([workspace])?;
+    let mut memories = Vec::new();
+    while let Some(row) = rows.next()? {
+        let expiration: Option<String> = row.get(13)?;
+        if let Some(raw) = expiration {
+            // Compare instants, not RFC3339 strings: Z, offsets and fractions
+            // have different textual ordering. Keep nanosecond precision.
+            let expiry = chrono::DateTime::parse_from_rfc3339(&raw).map_err(|err| {
+                rusqlite::Error::FromSqlConversionFailure(
+                    13,
+                    rusqlite::types::Type::Text,
+                    Box::new(err),
+                )
+            })?;
+            if expiry <= cutoff {
+                continue;
+            }
+        }
+        let metadata_str: Option<String> = row.get(11)?;
+        let metadata: Value = match metadata_str {
+            Some(raw) => serde_json::from_str(&raw).map_err(|err| {
+                rusqlite::Error::FromSqlConversionFailure(
+                    11,
+                    rusqlite::types::Type::Text,
+                    Box::new(err),
+                )
+            })?,
+            None => Value::Null,
+        };
+        memories.push(json!({
+            "id": row.get::<_,i64>(0)?, "content": row.get::<_,String>(1)?,
+            "memory_type": row.get::<_,String>(2)?, "importance": row.get::<_,Option<f64>>(3)?,
+            "workspace": row.get::<_,String>(4)?, "tier": row.get::<_,Option<String>>(5)?,
+            "created_at": row.get::<_,String>(6)?, "updated_at": row.get::<_,Option<String>>(7)?,
+            "tags": row.get::<_,String>(8)?, "scope": row.get::<_,Option<String>>(9)?,
+            "version": row.get::<_,Option<i64>>(10)?, "metadata": metadata,
+            "content_hash": row.get::<_,Option<String>>(12)?
+        }));
@@ -67,0 +100,2 @@ pub(super) fn build_related_map(
+    Ok(memories)
+}
@@ -69,38 +103,23 @@ pub(super) fn build_related_map(
-    // Build a single IN-list query for all IDs at once.
-    let placeholders: Vec<String> = (1..=memory_ids.len()).map(|i| format!("?{i}")).collect();
-    let in_list = placeholders.join(", ");
-    let sql = format!(
-        "SELECT from_id, to_id, edge_type FROM crossrefs
-          WHERE from_id IN ({in_list}) OR to_id IN ({in_list})"
-    );
-
-    let result = ctx.storage.with_connection(|conn| {
-        let mut stmt = conn.prepare(&sql)?;
-        let rows: Vec<(i64, i64, String)> = stmt
-            .query_map(rusqlite::params_from_iter(memory_ids.iter()), |row| {
-                Ok((
-                    row.get::<_, i64>(0)?,
-                    row.get::<_, i64>(1)?,
-                    row.get::<_, String>(2)?,
-                ))
-            })?
-            .collect::<std::result::Result<Vec<_>, _>>()
-            .map_err(crate::error::EngramError::Database)?;
-        Ok(rows)
-    });
-
-    match result {
-        Ok(rows) => {
-            let id_set: std::collections::HashSet<i64> = memory_ids.iter().copied().collect();
-            let mut map: HashMap<i64, Vec<(i64, String)>> = HashMap::new();
-            for (from_id, to_id, rel_type) in rows {
-                if id_set.contains(&from_id) {
-                    map.entry(from_id)
-                        .or_default()
-                        .push((to_id, rel_type.clone()));
-                }
-                if id_set.contains(&to_id) && to_id != from_id {
-                    map.entry(to_id).or_default().push((from_id, rel_type));
-                }
-            }
-            map
+fn build_related_map(
+    conn: &Connection,
+    workspace: &str,
+    eligible: &HashSet<i64>,
+) -> Result<RelatedMap, EngramError> {
+    let mut stmt = conn.prepare(
+        "SELECT c.from_id,c.to_id,c.edge_type FROM crossrefs c
+         JOIN memories a ON a.id=c.from_id JOIN memories b ON b.id=c.to_id
+         WHERE a.workspace=?1 AND b.workspace=?1 AND c.valid_to IS NULL
+         ORDER BY c.edge_type,c.from_id,c.to_id",
+    )?;
+    let rows = stmt.query_map([workspace], |r| {
+        Ok((
+            r.get::<_, i64>(0)?,
+            r.get::<_, i64>(1)?,
+            r.get::<_, String>(2)?,
+        ))
+    })?;
+    let mut map = RelatedMap::new();
+    for row in rows {
+        let (from, to, kind) = row?;
+        if !eligible.contains(&from) || !eligible.contains(&to) {
+            continue;
@@ -108,3 +127,5 @@ pub(super) fn build_related_map(
-        Err(e) => {
-            eprintln!("[markdown_export] build_related_map DB error: {e}");
-            HashMap::new()
+        map.entry(from).or_default().push((to, kind.clone()));
+        if to != from {
+            map.entry(to)
+                .or_default()
+                .push((from, format!("incoming {kind}")));
@@ -112,0 +134,4 @@ pub(super) fn build_related_map(
+    for links in map.values_mut() {
+        links.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
+    }
+    Ok(map)
diff --git a/src/mcp/handlers/markdown_export/export/security_tests.rs b/src/mcp/handlers/markdown_export/export/security_tests.rs
new file mode 100644
index 0000000000000000000000000000000000000000..295bfbceffeb668d4a7d1ef11904ed20e8869f5c
--- /dev/null
+++ b/src/mcp/handlers/markdown_export/export/security_tests.rs
@@ -0,0 +1,363 @@
+use super::handler_tests::{ctx, p0_export, p0_memory};
+use crate::auth::{PermissionSet, TokenClaims, TransportPrincipal, UserId};
+use crate::mcp::handlers::dispatch;
+use crate::storage::queries::create_memory;
+use crate::types::{CreateMemoryInput, MemoryType};
+use serde_json::json;
+use std::path::{Path, PathBuf};
+
+fn principal(permissions: PermissionSet) -> TransportPrincipal {
+    TransportPrincipal::from_token_claims(TokenClaims {
+        user_id: UserId::from_string("export-test-user"),
+        key_id: "export-test-key".into(),
+        permissions,
+        namespace: Some("markdown_export_schema".into()),
+        issued_at: chrono::Utc::now(),
+        expires_at: None,
+    })
+    .unwrap()
+}
+
+#[test]
+fn p0_expiration_cutoff_excludes_equality_but_preserves_null_and_future() {
+    let c = ctx();
+    let null = p0_memory(&c, "No expiration", vec![]);
+    let past = p0_memory(&c, "Past", vec![]);
+    let equal = p0_memory(&c, "At cutoff", vec![]);
+    let future = p0_memory(&c, "Future", vec![]);
+    c.storage
+        .with_connection(|conn| {
+            for (id, date) in [
+                (past, "2029-01-01T00:00:00+00:00"),
+                (equal, "2030-01-01T00:00:00+00:00"),
+                (future, "2031-01-01T00:00:00+00:00"),
+            ] {
+                conn.execute(
+                    "UPDATE memories SET tier='daily',expires_at=?1 WHERE id=?2",
+                    rusqlite::params![date, id],
+                )?;
+            }
+            let rows = super::query::query_workspace_memories_at(
+                conn,
+                "markdown_export_schema",
+                "2030-01-01T00:00:00+00:00",
+            )?;
+            let ids: std::collections::BTreeSet<_> =
+                rows.iter().map(|r| r["id"].as_i64().unwrap()).collect();
+            assert_eq!(ids, std::collections::BTreeSet::from([null, future]));
+            Ok(())
+        })
+        .unwrap();
+}
+
+#[test]
+fn p0_review_expiration_compares_instants_without_losing_nanoseconds() {
+    let c = ctx();
+    let unexpired = p0_memory(&c, "No expiry", vec![]);
+    let mut expected = std::collections::BTreeSet::from([unexpired]);
+    for (date, keep) in [
+        ("2030-01-01T00:00:00Z", false),
+        ("2030-01-01T00:00:00.000000000Z", false),
+        ("2030-01-01T01:00:00+01:00", false),
+        ("2029-12-31T23:00:00-01:00", false),
+        ("2029-12-31T23:59:59.999999999Z", false),
+        ("2030-01-01T00:00:00.000000001Z", true),
+    ] {
+        let id = p0_memory(&c, date, vec![]);
+        c.storage
+            .with_connection(|conn| {
+                conn.execute(
+                    "UPDATE memories SET tier='daily',expires_at=?1 WHERE id=?2",
+                    rusqlite::params![date, id],
+                )?;
+                Ok(())
+            })
+            .unwrap();
+        if keep {
+            expected.insert(id);
+        }
+    }
+    c.storage
+        .with_connection(|conn| {
+            let rows = super::query::query_workspace_memories_at(
+                conn,
+                "markdown_export_schema",
+                "2030-01-01T00:00:00+00:00",
+            )?;
+            let ids: std::collections::BTreeSet<_> =
+                rows.iter().map(|r| r["id"].as_i64().unwrap()).collect();
+            assert_eq!(ids, expected);
+            Ok(())
+        })
+        .unwrap();
+}
+
+#[test]
+fn p0_review_invalid_expiration_is_an_explicit_error() {
+    let c = ctx();
+    let id = p0_memory(&c, "Bad expiration", vec![]);
+    c.storage
+        .with_connection(|conn| {
+            conn.execute(
+                "UPDATE memories SET tier='daily',expires_at='not-a-timestamp' WHERE id=?1",
+                [id],
+            )?;
+            assert!(super::query::query_workspace_memories_at(
+                conn,
+                "markdown_export_schema",
+                "2030-01-01T00:00:00Z"
+            )
+            .is_err());
+            Ok(())
+        })
+        .unwrap();
+}
+
+#[test]
+fn p0_invalid_metadata_is_an_error_not_silently_null() {
+    let c = ctx();
+    let id = p0_memory(&c, "Invalid metadata", vec![]);
+    c.storage
+        .with_connection(|conn| {
+            conn.execute("UPDATE memories SET metadata='not-json' WHERE id=?1", [id])?;
+            Ok(())
+        })
+        .unwrap();
+    let fixture = tempfile::tempdir().unwrap();
+    let out = fixture.path().join("export");
+    let r = p0_export(&c, &out, "flat");
+    assert!(r.get("error").is_some(), "{r}");
+    assert!(!out.exists());
+}
+
+#[test]
+fn p0_cross_workspace_and_inactive_relations_are_not_exported() {
+    let c = ctx();
+    let a = p0_memory(&c, "Visible A", vec![]);
+    let b = p0_memory(&c, "Visible B", vec![]);
+    let secret = c
+        .storage
+        .with_transaction(|conn| {
+            Ok(create_memory(
+                conn,
+                &CreateMemoryInput {
+                    content: "Other workspace secret".into(),
+                    workspace: Some("foreign".into()),
+                    ..Default::default()
+                },
+            )?
+            .id)
+        })
+        .unwrap();
+    c.storage.with_connection(|conn| {
+        conn.execute("INSERT INTO crossrefs(from_id,to_id,edge_type,score) VALUES(?1,?2,'supersedes',1.0)",[a,secret])?;
+        conn.execute("INSERT INTO crossrefs(from_id,to_id,edge_type,score,valid_to) VALUES(?1,?2,'contradicts',1.0,'2000-01-01')",[a,b])?;
+        Ok(())
+    }).unwrap();
+    let fixture = tempfile::tempdir().unwrap();
+    let r = p0_export(&c, fixture.path(), "flat");
+    assert!(r.get("error").is_none(), "{r}");
+    let a = std::fs::read_to_string(fixture.path().join(format!("{a}-visible-a.md"))).unwrap();
+    assert!(!a.contains("## Related"), "{a}");
+}
+
+#[cfg(unix)]
+#[test]
+fn p0_active_database_and_hardlink_are_refused_without_opening_them() {
+    for alias in [false, true] {
+        let fixture = tempfile::tempdir().unwrap();
+        let db = fixture
+            .path()
+            .join(if alias { "state.db" } else { "index.md" });
+        let mut c = ctx();
+        c.storage = crate::storage::Storage::open(crate::types::StorageConfig {
+            db_path: db.to_string_lossy().into_owned(),
+            storage_mode: crate::types::StorageMode::Local,
+            cloud_uri: None,
+            encrypt_cloud: false,
+            confidence_half_life_days: 30.0,
+            auto_sync: false,
+            sync_debounce_ms: 5000,
+        })
+        .unwrap();
+        let id = p0_memory(&c, "Private note", vec![]);
+        if alias {
+            std::fs::hard_link(&db, fixture.path().join(format!("{id}-private-note.md"))).unwrap();
+        }
+        let r = p0_export(&c, fixture.path(), "flat");
+        assert!(r.get("error").is_some(), "{r}");
+        if !alias {
+            assert!(!fixture
+                .path()
+                .join(format!("{id}-private-note.md"))
+                .exists());
+        }
+        let count = c
+            .storage
+            .with_connection(|conn| {
+                Ok(conn.query_row("SELECT count(*) FROM memories", [], |r| r.get::<_, i64>(0))?)
+            })
+            .unwrap();
+        assert_eq!(count, 1);
+    }
+}
+
+// Re-exec only this test in a bounded child with a clean environment. This
+// isolates server-root/permission configuration from all concurrent unit tests.
+#[cfg(unix)]
+#[test]
+fn p0_export_fixture_child() {
+    let Some(root) = std::env::var_os("ENGRAM_P0_FIXTURE") else {
+        return;
+    };
+    let root = PathBuf::from(root);
+    let mut c = ctx();
+    for (kind, title) in [
+        (MemoryType::Note, "Alpha"),
+        (MemoryType::Decision, "Beta"),
+        (MemoryType::Summary, "Gamma"),
+    ] {
+        c.storage
+            .with_transaction(|conn| {
+                create_memory(
+                    conn,
+                    &CreateMemoryInput {
+                        content: title.into(),
+                        memory_type: kind,
+                        workspace: Some("markdown_export_schema".into()),
+                        tags: vec!["zeta".into(), "alpha".into()],
+                        ..Default::default()
+                    },
+                )?;
+                Ok(())
+            })
+            .unwrap();
+    }
+    c.storage.with_connection(|conn| {conn.execute("UPDATE memories SET created_at='2026-01-01T00:00:00+00:00',updated_at='2026-01-01T00:00:00+00:00'",[])?;Ok(())}).unwrap();
+    c.principal = Some(principal(PermissionSet::standard_user()));
+    let out = root.join("export");
+    let r = dispatch(
+        &c,
+        "memory_export_markdown",
+        json!({"workspace":"markdown_export_schema","output_dir":out,"group":"type","_permission_mode":"scoped_write"}),
+    );
+    assert!(r.get("error").is_none(), "{r}");
+    assert_eq!(r["memories_exported"], 3);
+
+    // The public Rust facade must propagate structured handler denials, not
+    // turn a denied filesystem operation into a successful zero-file report.
+    std::env::set_var("ENGRAM_PERMISSION_MODE", "read_only");
+    let report = crate::portability::markdown::export_markdown(
+        &c.storage,
+        &crate::portability::markdown::ExportOptions {
+            output_dir: root.join("wrapper-denied"),
+            workspace: Some("markdown_export_schema".into()),
+            ..Default::default()
+        },
+    );
+    assert!(
+        report.is_err(),
+        "structured permission error became success: {report:?}"
+    );
+    assert!(!root.join("wrapper-denied").exists());
+    std::env::remove_var("ENGRAM_PERMISSION_MODE");
+
+    let rejected = root.join("foreign-workspace");
+    let r = dispatch(
+        &c,
+        "memory_export_markdown",
+        json!({"workspace":"foreign","output_dir":rejected}),
+    );
+    assert_eq!(r["error"]["code"], "permission_denied", "{r}");
+    assert!(!rejected.exists());
+    let outside = root.parent().unwrap().join("outside-base");
+    let r = p0_export(&c, &outside, "flat");
+    assert!(r.get("error").is_some(), "{r}");
+    assert!(!outside.exists());
+
+    c.principal = Some(principal(PermissionSet::read_only()));
+    let r = p0_export(&c, &root.join("read-only"), "flat");
+    assert_eq!(r["error"]["code"], "permission_denied", "{r}");
+    assert!(!root.join("read-only").exists());
+
+    std::env::remove_var("ENGRAM_EXPORT_BASE_DIR"); // Child is isolated, only one test.
+    c.principal = Some(principal(PermissionSet::standard_user()));
+    let r = p0_export(&c, &root.join("unconfined"), "flat");
+    assert_eq!(r["error"]["required_mode"], "maintenance", "{r}");
+    assert!(!root.join("unconfined").exists());
+    c.principal = Some(principal(PermissionSet::admin()));
+    let r = p0_export(&c, &root.join("maintenance"), "flat");
+    assert!(r.get("error").is_none(), "{r}");
+}
+
+#[cfg(unix)]
+#[test]
+fn p0_separate_processes_produce_byte_identical_trees_and_enforce_roots() {
+    use std::process::{Command, Stdio};
+    use std::time::{Duration, Instant};
+    let fixture = tempfile::tempdir().unwrap();
+    let mut expected = None;
+    for n in 0..4 {
+        let root = fixture.path().join(format!("run{n}"));
+        std::fs::create_dir(&root).unwrap();
+        let mut child = Command::new(std::env::current_exe().unwrap())
+            .args([
+                "--exact",
+                "mcp::handlers::markdown_export::export::security_tests::p0_export_fixture_child",
+                "--nocapture",
+                "--test-threads=1",
+            ])
+            .env_clear()
+            .env("PATH", "/usr/bin:/bin")
+            .env("TMPDIR", fixture.path())
+            .env("HOME", &root)
+            .env("ENGRAM_P0_FIXTURE", &root)
+            .env("ENGRAM_EXPORT_BASE_DIR", &root)
+            .stdin(Stdio::null())
+            .stdout(Stdio::piped())
+            .stderr(Stdio::piped())
+            .spawn()
+            .unwrap();
+        let deadline = Instant::now() + Duration::from_secs(30);
+        while child.try_wait().unwrap().is_none() {
+            if Instant::now() > deadline {
+                child.kill().unwrap();
+                let _ = child.wait();
+                panic!("export fixture subprocess timed out");
+            }
+            std::thread::sleep(Duration::from_millis(10));
+        }
+        let output = child.wait_with_output().unwrap();
+        assert!(
+            output.status.success(),
+            "{}\n{}",
+            String::from_utf8_lossy(&output.stdout),
+            String::from_utf8_lossy(&output.stderr)
+        );
+        fn tree(
+            root: &Path,
+            base: &Path,
+            result: &mut std::collections::BTreeMap<PathBuf, Vec<u8>>,
+        ) {
+            for entry in std::fs::read_dir(root).unwrap() {
+                let path = entry.unwrap().path();
+                if path.is_dir() {
+                    tree(&path, base, result);
+                } else {
+                    result.insert(
+                        path.strip_prefix(base).unwrap().to_owned(),
+                        std::fs::read(path).unwrap(),
+                    );
+                }
+            }
+        }
+        let mut actual = std::collections::BTreeMap::new();
+        tree(&root.join("export"), &root.join("export"), &mut actual);
+        assert_eq!(actual.len(), 4);
+        if let Some(expected) = &expected {
+            assert_eq!(&actual, expected);
+        } else {
+            expected = Some(actual);
+        }
+    }
+}
diff --git a/src/mcp/handlers/markdown_export/frontmatter.rs b/src/mcp/handlers/markdown_export/frontmatter.rs
index 229e83726d75c30bd3984243a7c019e369971631..7ad6ed436eafefc734ef0d8cc93f21ba90ef4ac8 100644
--- a/src/mcp/handlers/markdown_export/frontmatter.rs
+++ b/src/mcp/handlers/markdown_export/frontmatter.rs
@@ -75,24 +75,30 @@ pub(super) fn parse_frontmatter(content: &str) -> HashMap<String, String> {
-        if let Some(colon_pos) = line.find(": ") {
-            let key = line[..colon_pos].trim();
-            let value = line[colon_pos + 2..].trim();
-
-            if !key.starts_with("engram_") {
-                i += 1;
-                continue;
-            }
-
-            if key == "engram_tags" && value.is_empty() {
-                // Sequence mode — collect following `  - item` lines
-                let mut tags: Vec<String> = Vec::new();
-                i += 1;
-                while i < fm_lines.len() {
-                    let next = fm_lines[i];
-                    if let Some(item) = next.strip_prefix("  - ") {
-                        tags.push(item.trim().to_string());
-                        i += 1;
-                    } else {
-                        break;
-                    }
-                }
-                if !tags.is_empty() {
-                    map.insert("engram_tags_list".to_string(), tags.join(","));
+        i += 1;
+        let Some((raw_key, raw_value)) = line.split_once(':') else {
+            continue;
+        };
+        // YAML scalar mapping separators allow space, tab or end-of-line.
+        // Preserve explicitly empty identity/version values for validation.
+        if !raw_value.is_empty() && !raw_value.starts_with([' ', '\t']) {
+            continue;
+        }
+        let key = raw_key.trim();
+        let key = key
+            .strip_prefix('"')
+            .and_then(|key| key.strip_suffix('"'))
+            .or_else(|| {
+                key.strip_prefix('\'')
+                    .and_then(|key| key.strip_suffix('\''))
+            })
+            .unwrap_or(key);
+        if !key.starts_with("engram_") {
+            continue;
+        }
+        let value = raw_value.trim();
+        if key == "engram_tags" && value.is_empty() {
+            let mut tags = Vec::new();
+            while i < fm_lines.len() {
+                if let Some(item) = fm_lines[i].strip_prefix("  - ") {
+                    tags.push(item.trim().to_string());
+                    i += 1;
+                } else {
+                    break;
@@ -100 +105,0 @@ pub(super) fn parse_frontmatter(content: &str) -> HashMap<String, String> {
-                continue;
@@ -102,23 +107,2 @@ pub(super) fn parse_frontmatter(content: &str) -> HashMap<String, String> {
-
-            // Strip surrounding quotes
-            let cleaned = value.trim_matches('"').to_string();
-            map.insert(key.to_string(), cleaned);
-        } else {
-            // Handle `engram_tags:` without trailing space (value is empty on same line)
-            let trimmed = line.trim();
-            if trimmed == "engram_tags:" {
-                let mut tags: Vec<String> = Vec::new();
-                i += 1;
-                while i < fm_lines.len() {
-                    let next = fm_lines[i];
-                    if let Some(item) = next.strip_prefix("  - ") {
-                        tags.push(item.trim().to_string());
-                        i += 1;
-                    } else {
-                        break;
-                    }
-                }
-                if !tags.is_empty() {
-                    map.insert("engram_tags_list".to_string(), tags.join(","));
-                }
-                continue;
+            if !tags.is_empty() {
+                map.insert("engram_tags_list".to_string(), tags.join(","));
@@ -125,0 +110 @@ pub(super) fn parse_frontmatter(content: &str) -> HashMap<String, String> {
+            continue;
@@ -127 +112 @@ pub(super) fn parse_frontmatter(content: &str) -> HashMap<String, String> {
-        i += 1;
+        map.insert(key.to_string(), value.trim_matches('"').to_string());
diff --git a/src/mcp/handlers/markdown_export/import.rs b/src/mcp/handlers/markdown_export/import.rs
index b26cf22ed2468ced7a1c39f779ea5f070e5775d5..d782289b215115d94d66f671102e4750fd4560af 100644
--- a/src/mcp/handlers/markdown_export/import.rs
+++ b/src/mcp/handlers/markdown_export/import.rs
@@ -4 +3,0 @@ mod handler;
-mod lookup;
@@ -12,0 +12,2 @@ mod integration_tests;
+mod security_tests;
+#[cfg(test)]
diff --git a/src/mcp/handlers/markdown_export/import/apply.rs b/src/mcp/handlers/markdown_export/import/apply.rs
index dd460f268c27645d62bfe827217a0359e34173dd..8dfb5c133d3cbf615af50003e617ecf0b8b8d532 100644
--- a/src/mcp/handlers/markdown_export/import/apply.rs
+++ b/src/mcp/handlers/markdown_export/import/apply.rs
@@ -2,0 +3 @@ use std::collections::HashMap;
+use rusqlite::{Connection, OptionalExtension};
@@ -5 +6,6 @@ use serde_json::{json, Value};
-use crate::mcp::handlers::markdown_export::frontmatter::{frontmatter_tags, import_payload};
+use super::files::MarkdownSource;
+use super::status::{classify_import_status, ImportStatus};
+use crate::error::{EngramError, Result};
+use crate::mcp::handlers::markdown_export::frontmatter::{
+    extract_body, frontmatter_tags, import_payload, parse_frontmatter,
+};
@@ -6,0 +13,3 @@ use crate::mcp::handlers::HandlerContext;
+use crate::mcp::workspace_guard::ensure_memory_access;
+use crate::storage::queries::{compute_content_hash, compute_content_hash_raw};
+use crate::types::{CreateMemoryInput, UpdateMemoryInput};
@@ -8 +17,103 @@ use crate::mcp::handlers::HandlerContext;
-pub(super) fn create_memory_from_import(
+#[derive(Debug)]
+pub(super) struct ImportFailure {
+    error: EngramError,
+    // Storage can fail at COMMIT or in permission revalidation after COMMIT.
+    // A completed callback alone cannot prove either rollback or persistence.
+    attempted: Option<u64>,
+}
+
+impl From<EngramError> for ImportFailure {
+    fn from(error: EngramError) -> Self {
+        Self {
+            error,
+            attempted: None,
+        }
+    }
+}
+
+impl ImportFailure {
+    pub(super) fn into_value(self) -> Value {
+        use crate::mcp::error::ToolError;
+        if let Some(attempted) = self.attempted {
+            let mut value = ToolError::internal(
+                "Import completion is uncertain; inspect the canonical database before retrying",
+            )
+            .into_value();
+            value["applied"] = Value::Null;
+            value["attempted"] = json!(attempted);
+            value["outcome"] = json!("unknown");
+            value
+        } else {
+            let mut value = ToolError::from(self.error).into_value();
+            value["applied"] = json!(0);
+            value["outcome"] = json!("not_applied");
+            value
+        }
+    }
+}
+
+struct Input {
+    file: String,
+    id: Option<i64>,
+    version: i64,
+    fm: HashMap<String, String>,
+    body: String,
+}
+
+enum Mutation {
+    None,
+    Create(CreateMemoryInput),
+    Update(i64, UpdateMemoryInput),
+}
+
+struct Plan {
+    file: String,
+    id: Option<i64>,
+    status: ImportStatus,
+    mutation: Mutation,
+}
+
+fn positive_field(fm: &HashMap<String, String>, field: &str) -> Result<Option<i64>> {
+    fm.get(field)
+        .map(|value| {
+            value
+                .parse::<i64>()
+                .ok()
+                .filter(|value| *value > 0)
+                .ok_or_else(|| {
+                    EngramError::InvalidInput(format!("{field} must be a positive integer"))
+                })
+        })
+        .transpose()
+}
+
+fn prepare(sources: Vec<MarkdownSource>) -> Result<Vec<Input>> {
+    let mut ids = std::collections::BTreeSet::new();
+    sources
+        .into_iter()
+        .map(|source| {
+            let fm = parse_frontmatter(&source.raw);
+            let id = positive_field(&fm, "engram_id")?;
+            if id.is_some_and(|id| !ids.insert(id)) {
+                return Err(EngramError::InvalidInput(
+                    "Duplicate engram_id in Markdown input".into(),
+                ));
+            }
+            Ok(Input {
+                file: source
+                    .relative
+                    .to_str()
+                    .ok_or_else(|| {
+                        EngramError::InvalidInput("Markdown input names must be UTF-8".into())
+                    })?
+                    .to_string(),
+                id,
+                version: positive_field(&fm, "engram_version")?.unwrap_or(1),
+                body: extract_body(&source.raw).trim().to_string(),
+                fm,
+            })
+        })
+        .collect()
+}
+
+fn creation(
@@ -10,6 +121,5 @@ pub(super) fn create_memory_from_import(
-    frontmatter: &HashMap<String, String>,
-    body: &str,
-    workspace_override: Option<&str>,
-) -> Result<i64, crate::error::EngramError> {
-    let mut obj = import_payload(frontmatter, body, true);
-    if let Some(workspace) = workspace_override {
+    input: &Input,
+    workspace: Option<&str>,
+) -> Result<CreateMemoryInput> {
+    let mut obj = import_payload(&input.fm, &input.body, true);
+    if let Some(workspace) = workspace {
@@ -18,3 +128,14 @@ pub(super) fn create_memory_from_import(
-    let tags = frontmatter_tags(frontmatter);
-    if !tags.is_empty() {
-        obj.insert("tags".into(), json!(tags));
+    obj.insert("tags".into(), json!(frontmatter_tags(&input.fm)));
+    let mut parsed: CreateMemoryInput = serde_json::from_value(Value::Object(obj))
+        .map_err(|_| EngramError::InvalidInput("Invalid Markdown creation fields".into()))?;
+    let workspace =
+        crate::types::normalize_workspace(parsed.workspace.as_deref().unwrap_or("default"))
+            .map_err(|error| EngramError::InvalidInput(error.to_string()))?;
+    if ctx
+        .principal
+        .as_ref()
+        .is_some_and(|principal| !principal.allows_workspace(Some(&workspace)))
+    {
+        return Err(EngramError::Unauthorized(
+            "Markdown target workspace is not allowed".into(),
+        ));
@@ -21,0 +143,3 @@ pub(super) fn create_memory_from_import(
+    parsed.workspace = Some(workspace);
+    Ok(parsed)
+}
@@ -23,7 +147,7 @@ pub(super) fn create_memory_from_import(
-    serde_json::from_value::<crate::types::CreateMemoryInput>(Value::Object(obj))
-        .map_err(|e| crate::error::EngramError::Internal(format!("bad import payload: {e}")))
-        .and_then(|input| {
-            ctx.storage
-                .with_transaction(|conn| crate::storage::queries::create_memory(conn, &input))
-        })
-        .map(|new_memory| new_memory.id)
+fn update(input: &Input) -> Result<UpdateMemoryInput> {
+    let mut obj = import_payload(&input.fm, &input.body, false);
+    if input.fm.contains_key("engram_tags_list") {
+        obj.insert("tags".into(), json!(frontmatter_tags(&input.fm)));
+    }
+    serde_json::from_value(Value::Object(obj))
+        .map_err(|_| EngramError::InvalidInput("Invalid Markdown update fields".into()))
@@ -32 +156 @@ pub(super) fn create_memory_from_import(
-pub(super) fn update_memory_from_import(
+fn plan(
@@ -34,15 +158,114 @@ pub(super) fn update_memory_from_import(
-    engram_id: i64,
-    frontmatter: &HashMap<String, String>,
-    body: &str,
-    filename: &str,
-) -> Result<(), crate::error::EngramError> {
-    let mut obj = import_payload(frontmatter, body, false);
-    let tags = frontmatter_tags(frontmatter);
-    if !tags.is_empty() {
-        obj.insert("tags".into(), json!(tags));
-    } else if frontmatter.contains_key("engram_tags_list") {
-        eprintln!(
-            "[markdown_export] import: wiping all tags for memory {} (engram_tags_list is empty in file {})",
-            engram_id, filename
-        );
-        obj.insert("tags".into(), json!([]));
+    conn: &Connection,
+    input: &Input,
+    workspace: Option<&str>,
+    force: bool,
+) -> Result<Plan> {
+    let state: Option<(String, i64)> = if let Some(id) = input.id {
+        // The file's claimed workspace never authorizes an existing ID.
+        // Same connection/snapshot as classification and mutation.
+        ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
+        conn.query_row(
+            "SELECT content_hash, version FROM memories WHERE id = ?1",
+            [id],
+            |row| Ok((row.get(0)?, row.get(1)?)),
+        )
+        .optional()?
+    } else {
+        None
+    };
+    let (hash, baseline) = match input.fm.get("engram_content_hash") {
+        Some(baseline) => (compute_content_hash_raw(&input.body), baseline.clone()),
+        None => (
+            compute_content_hash(&input.body),
+            state
+                .as_ref()
+                .map(|(hash, _)| hash.clone())
+                .unwrap_or_default(),
+        ),
+    };
+    let status = classify_import_status(
+        state
+            .as_ref()
+            .map(|(hash, version)| (hash.as_str(), *version)),
+        &hash,
+        &baseline,
+        input.version,
+        force,
+    );
+    let mutation = match status {
+        ImportStatus::New => Mutation::Create(creation(ctx, input, workspace)?),
+        ImportStatus::PendingUpdate => Mutation::Update(
+            input
+                .id
+                .ok_or_else(|| EngramError::Internal("Missing import ID".into()))?,
+            update(input)?,
+        ),
+        _ => Mutation::None,
+    };
+    Ok(Plan {
+        file: input.file.clone(),
+        id: input.id,
+        status,
+        mutation,
+    })
+}
+
+fn run(
+    ctx: &HandlerContext,
+    conn: &Connection,
+    inputs: &[Input],
+    workspace: Option<&str>,
+    force: bool,
+    confirm: bool,
+) -> Result<Value> {
+    // Authorize and classify the entire batch before the first write. With
+    // confirm, BEGIN IMMEDIATE excludes concurrent writers through application.
+    let plans = inputs
+        .iter()
+        .map(|input| plan(ctx, conn, input, workspace, force))
+        .collect::<Result<Vec<_>>>()?;
+    let (mut synced, mut new, mut pending, mut conflicts, mut applied) = (0, 0, 0, 0, 0);
+    let mut files = Vec::new();
+    for plan in plans {
+        let mut id = plan.id;
+        let mut changed = false;
+        if confirm {
+            match plan.mutation {
+                Mutation::Create(input) => {
+                    id = Some(crate::storage::queries::create_memory(conn, &input)?.id);
+                    changed = true;
+                }
+                Mutation::Update(id, input) => {
+                    crate::storage::queries::update_memory(conn, id, &input)?;
+                    changed = true;
+                }
+                Mutation::None => (),
+            }
+        }
+        applied += usize::from(changed);
+        let (status, reason) = match plan.status {
+            ImportStatus::New => {
+                new += 1;
+                ("new", None)
+            }
+            ImportStatus::InSync => {
+                synced += 1;
+                ("in_sync", None)
+            }
+            ImportStatus::PendingUpdate => {
+                pending += 1;
+                ("pending_update", None)
+            }
+            ImportStatus::Conflict(reason) => {
+                conflicts += 1;
+                ("conflict", Some(reason))
+            }
+        };
+        let mut detail = json!({"file":plan.file,"engram_id":id,"status":status});
+        if confirm && changed {
+            detail["applied"] = json!(true);
+        }
+        if let Some(reason) = reason {
+            detail["reason"] = json!(reason);
+        }
+        files.push(detail);
@@ -49,0 +273,4 @@ pub(super) fn update_memory_from_import(
+    Ok(
+        json!({"scanned":inputs.len(),"in_sync":synced,"new":new,"pending":pending,"pending_updates":pending,"conflict":conflicts,"conflicts":conflicts,"applied":applied,"dry_run":!confirm,"files":files}),
+    )
+}
@@ -51,6 +278,25 @@ pub(super) fn update_memory_from_import(
-    serde_json::from_value::<crate::types::UpdateMemoryInput>(Value::Object(obj))
-        .map_err(|e| crate::error::EngramError::Internal(format!("bad import payload: {e}")))
-        .and_then(|input| {
-            ctx.storage.with_transaction(|conn| {
-                crate::storage::queries::update_memory(conn, engram_id, &input)
-            })
+pub(super) fn import_batch(
+    ctx: &HandlerContext,
+    sources: Vec<MarkdownSource>,
+    workspace: Option<&str>,
+    force: bool,
+    confirm: bool,
+) -> std::result::Result<Value, ImportFailure> {
+    let inputs = prepare(sources)?; // Parsing and filesystem IO happen before DB transactions.
+    let mut attempted = None;
+    let result = if confirm {
+        ctx.storage.with_transaction(|conn| {
+            let report = run(ctx, conn, &inputs, workspace, force, true)?;
+            attempted = Some(
+                report["applied"]
+                    .as_u64()
+                    .ok_or_else(|| EngramError::Internal("Missing import action count".into()))?,
+            );
+            Ok(report)
+        })
+    } else {
+        ctx.storage.with_connection(|conn| {
+            let tx = conn.unchecked_transaction()?;
+            let result = run(ctx, &tx, &inputs, workspace, force, false)?;
+            tx.commit()?;
+            Ok(result)
@@ -58 +304,2 @@ pub(super) fn update_memory_from_import(
-        .map(|_| ())
+    };
+    result.map_err(|error| ImportFailure { error, attempted })
diff --git a/src/mcp/handlers/markdown_export/import/files.rs b/src/mcp/handlers/markdown_export/import/files.rs
index e4aec4f318540de5a9c6a42d656bca95452ff342..a8fae29a0cb6ad59544c0533515a45108be8d525 100644
--- a/src/mcp/handlers/markdown_export/import/files.rs
+++ b/src/mcp/handlers/markdown_export/import/files.rs
@@ -1,35 +1,3 @@
-use std::fs;
-use std::path::PathBuf;
-
-pub(super) fn collect_md_files(dir: &str) -> Result<Vec<PathBuf>, std::io::Error> {
-    let mut result = Vec::new();
-    collect_md_files_inner(&PathBuf::from(dir), &mut result)?;
-    Ok(result)
-}
-
-pub(super) fn collect_md_files_inner(
-    dir: &PathBuf,
-    out: &mut Vec<PathBuf>,
-) -> Result<(), std::io::Error> {
-    for entry in fs::read_dir(dir)? {
-        let entry = entry?;
-        let path = entry.path();
-        if path.is_dir() {
-            collect_md_files_inner(&path, out)?;
-        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
-            let file_name = path
-                .file_name()
-                .and_then(|f| f.to_str())
-                .unwrap_or("")
-                .to_lowercase();
-            if !file_name.starts_with('.')
-                && !file_name.starts_with('_')
-                && file_name != "index.md"
-                && file_name != "readme.md"
-            {
-                out.push(path);
-            }
-        }
-    }
-    Ok(())
-}
+pub(super) use crate::mcp::handlers::markdown_export::export::filesystem::{
+    read_markdown, MarkdownSource,
+};
diff --git a/src/mcp/handlers/markdown_export/import/handler.rs b/src/mcp/handlers/markdown_export/import/handler.rs
index 8c39ecb84c68fa4992d6caa8f3ab990c97133a87..54744e9ad64c948dd3bf05ac2b8236a9eed379fc 100644
--- a/src/mcp/handlers/markdown_export/import/handler.rs
+++ b/src/mcp/handlers/markdown_export/import/handler.rs
@@ -1 +1 @@
-use std::fs;
+use serde_json::Value;
@@ -3,7 +3,3 @@ use std::fs;
-use serde_json::{json, Value};
-
-use super::apply::{create_memory_from_import, update_memory_from_import};
-use super::files::collect_md_files;
-use super::lookup::db_state_for_memory;
-use super::status::{classify_import_status, ImportStatus};
-use crate::mcp::handlers::markdown_export::frontmatter::{extract_body, parse_frontmatter};
+use super::apply::{import_batch, ImportFailure};
+use super::files::read_markdown;
+use crate::error::EngramError;
@@ -12 +7,0 @@ use crate::mcp::handlers::HandlerContext;
-use crate::storage::queries::{compute_content_hash, compute_content_hash_raw};
@@ -15,240 +10,34 @@ pub fn memory_import_markdown(ctx: &HandlerContext, params: Value) -> Value {
-    let input_dir_raw = match params.get("input_dir").and_then(|v| v.as_str()) {
-        Some(d) => d.to_string(),
-        None => return json!({"error": "input_dir is required"}),
-    };
-
-    let validated_input = match validate_export_dir(&input_dir_raw) {
-        Ok(p) => p,
-        Err(e) => return json!({"error": format!("Invalid input_dir: {}", e)}),
-    };
-
-    let input_dir = if validated_input.is_dir() {
-        validated_input.to_string_lossy().to_string()
-    } else {
-        return json!({"error": "input_dir is not a directory"});
-    };
-
-    let workspace_override = params
-        .get("workspace")
-        .and_then(|v| v.as_str())
-        .map(|s| s.to_string());
-
-    let confirm = params
-        .get("confirm")
-        .and_then(|v| v.as_bool())
-        .unwrap_or(false);
-
-    let force_version = params
-        .get("force_version")
-        .and_then(|v| v.as_bool())
-        .unwrap_or(false);
-
-    // Walk input_dir recursively for *.md files
-    let md_files = match collect_md_files(&input_dir) {
-        Ok(f) => f,
-        Err(e) => return json!({"error": format!("Failed to walk input_dir: {}", e)}),
-    };
-
-    let scanned = md_files.len();
-    let mut count_in_sync: usize = 0;
-    let mut count_new: usize = 0;
-    let mut count_pending: usize = 0;
-    let mut count_conflict: usize = 0;
-    let mut applied: usize = 0;
-    let mut files_detail: Vec<Value> = Vec::new();
-
-    for file_path in &md_files {
-        if file_path.to_str().is_none() {
-            continue;
-        }
-        let filename = file_path
-            .file_name()
-            .and_then(|f| f.to_str())
-            .unwrap_or("")
-            .to_string();
-
-        // A `*.md` entry may be a symlink to the active database (G1).
-        if let Err(e) = ctx.storage.refuse_active_sqlite_artifact(file_path) {
-            files_detail.push(json!({
-                "file": filename,
-                "status": "error",
-                "reason": e.to_string()
-            }));
-            continue;
-        }
-
-        let raw = match fs::read_to_string(file_path) {
-            Ok(s) => s,
-            Err(e) => {
-                files_detail.push(json!({
-                    "file": filename,
-                    "status": "error",
-                    "reason": format!("read error: {}", e)
-                }));
-                continue;
-            }
-        };
-
-        let fm = parse_frontmatter(&raw);
-        let body = extract_body(&raw).to_string();
-
-        let engram_id: i64 = fm
-            .get("engram_id")
-            .and_then(|s| s.parse().ok())
-            .unwrap_or(-1);
-
-        if engram_id == -1 {
-            count_new += 1;
-            if confirm {
-                match create_memory_from_import(ctx, &fm, &body, workspace_override.as_deref()) {
-                    Ok(inserted_id) => {
-                        applied += 1;
-                        files_detail.push(json!({
-                            "file": filename,
-                            "engram_id": inserted_id,
-                            "status": "new",
-                            "applied": true
-                        }));
-                    }
-                    Err(e) => {
-                        files_detail.push(json!({
-                            "file": filename,
-                            "engram_id": null,
-                            "status": "error",
-                            "reason": format!("insert error: {}", e)
-                        }));
-                    }
-                }
-            } else {
-                files_detail.push(json!({
-                    "file": filename,
-                    "engram_id": null,
-                    "status": "new"
-                }));
-            }
-            continue;
-        }
-
-        let file_version: i64 = fm
-            .get("engram_version")
-            .and_then(|s| s.parse().ok())
-            .unwrap_or(1);
-
-        let db_state = match db_state_for_memory(ctx, engram_id) {
-            Ok(s) => s,
-            Err(e) => {
-                files_detail.push(json!({
-                    "file": filename,
-                    "engram_id": engram_id,
-                    "status": "error",
-                    "reason": format!("DB lookup error: {}", e)
-                }));
-                continue;
-            }
-        };
-
-        // When the file has an engram_content_hash frontmatter field (written by
-        // memory_export_markdown since this fix), use case-sensitive (raw) hashes so
-        // case-only edits are detected as PendingUpdate.
-        // For files exported before this fix (no frontmatter hash), fall back to
-        // normalized hash comparison against the DB hash for backward compat.
-        let (current_hash, sync_baseline) = match fm.get("engram_content_hash") {
-            Some(fm_hash) => (compute_content_hash_raw(body.trim()), fm_hash.clone()),
-            None => {
-                let norm = compute_content_hash(body.trim());
-                let db_hash = db_state
-                    .as_ref()
-                    .map(|(h, _)| h.clone())
-                    .unwrap_or_default();
-                (norm, db_hash)
-            }
-        };
-
-        let status = classify_import_status(
-            db_state.as_ref().map(|(h, v)| (h.as_str(), *v)),
-            &current_hash,
-            &sync_baseline,
-            file_version,
-            force_version,
-        );
-
-        match &status {
-            ImportStatus::InSync => {
-                count_in_sync += 1;
-                files_detail.push(json!({
-                    "file": filename,
-                    "engram_id": engram_id,
-                    "status": "in_sync"
-                }));
-            }
-            ImportStatus::New => {
-                count_new += 1;
-                if confirm {
-                    match create_memory_from_import(ctx, &fm, &body, workspace_override.as_deref())
-                    {
-                        Ok(inserted_id) => {
-                            applied += 1;
-                            files_detail.push(json!({
-                                "file": filename,
-                                "engram_id": inserted_id,
-                                "status": "new",
-                                "applied": true
-                            }));
-                        }
-                        Err(e) => {
-                            files_detail.push(json!({
-                                "file": filename,
-                                "engram_id": engram_id,
-                                "status": "error",
-                                "reason": format!("insert error: {}", e)
-                            }));
-                        }
-                    }
-                } else {
-                    files_detail.push(json!({
-                        "file": filename,
-                        "engram_id": engram_id,
-                        "status": "new"
-                    }));
-                }
-            }
-            ImportStatus::PendingUpdate => {
-                count_pending += 1;
-                if confirm {
-                    match update_memory_from_import(ctx, engram_id, &fm, &body, &filename) {
-                        Ok(_) => {
-                            applied += 1;
-                            files_detail.push(json!({
-                                "file": filename,
-                                "engram_id": engram_id,
-                                "status": "pending_update",
-                                "applied": true
-                            }));
-                        }
-                        Err(e) => {
-                            files_detail.push(json!({
-                                "file": filename,
-                                "engram_id": engram_id,
-                                "status": "error",
-                                "reason": format!("update error: {}", e)
-                            }));
-                        }
-                    }
-                } else {
-                    files_detail.push(json!({
-                        "file": filename,
-                        "engram_id": engram_id,
-                        "status": "pending_update"
-                    }));
-                }
-            }
-            ImportStatus::Conflict(reason) => {
-                count_conflict += 1;
-                files_detail.push(json!({
-                    "file": filename,
-                    "engram_id": engram_id,
-                    "status": "conflict",
-                    "reason": reason
-                }));
-            }
-        }
+    if let Some(denial) = crate::mcp::permission::check_tool_authorization(
+        None,
+        "memory_import_markdown",
+        &params,
+        ctx.principal.as_ref(),
+    ) {
+        return denial;
+    }
+    let outcome: std::result::Result<Value, ImportFailure> = (|| {
+        let input = params
+            .get("input_dir")
+            .and_then(Value::as_str)
+            .ok_or_else(|| EngramError::InvalidInput("input_dir is required".into()))?;
+        let root = validate_export_dir(input).map_err(EngramError::InvalidInput)?;
+        let sources = read_markdown(&root, &ctx.storage).map_err(|error| {
+            EngramError::InvalidInput(format!("Markdown input refused: {error}"))
+        })?;
+        import_batch(
+            ctx,
+            sources,
+            params.get("workspace").and_then(Value::as_str),
+            params
+                .get("force_version")
+                .and_then(Value::as_bool)
+                .unwrap_or(false),
+            params
+                .get("confirm")
+                .and_then(Value::as_bool)
+                .unwrap_or(false),
+        )
+    })();
+    match outcome {
+        Ok(value) => value,
+        Err(error) => error.into_value(),
@@ -256,13 +44,0 @@ pub fn memory_import_markdown(ctx: &HandlerContext, params: Value) -> Value {
-
-    json!({
-        "scanned": scanned,
-        "in_sync": count_in_sync,
-        "new": count_new,
-        "pending": count_pending,
-        "pending_updates": count_pending,
-        "conflict": count_conflict,
-        "conflicts": count_conflict,
-        "applied": applied,
-        "dry_run": !confirm,
-        "files": files_detail
-    })
diff --git a/src/mcp/handlers/markdown_export/import/integration_tests.rs b/src/mcp/handlers/markdown_export/import/integration_tests.rs
index eb90239c7650530c026af4fd136330ab7336dcc1..4b4c2ac3a80876a0bed04753838e48e02967b5ce 100644
--- a/src/mcp/handlers/markdown_export/import/integration_tests.rs
+++ b/src/mcp/handlers/markdown_export/import/integration_tests.rs
@@ -4,0 +5,176 @@ use serde_json::json;
+#[test]
+fn mcp_success_reports_effective_review_mode_on_the_wire() {
+    for dispatch in [false, true] {
+        for confirm in [None, Some(false), Some(true)] {
+            let c = ctx();
+            let fixture = tempfile::tempdir().unwrap();
+            std::fs::write(fixture.path().join("new.md"), "new note").unwrap();
+            let mut params = json!({"input_dir":fixture.path(),"workspace":"default"});
+            if let Some(confirm) = confirm {
+                params["confirm"] = json!(confirm);
+            }
+            let result = if dispatch {
+                crate::mcp::handlers::dispatch(&c, "memory_import_markdown", params)
+            } else {
+                memory_import_markdown(&c, params)
+            };
+            assert!(result.get("error").is_none(), "{result}");
+            let writes = confirm == Some(true);
+            assert_eq!(result["dry_run"], !writes, "wire result: {result}");
+            assert_eq!(result["applied"], usize::from(writes));
+            let count = c
+                .storage
+                .with_connection(|conn| {
+                    Ok(conn.query_row("SELECT COUNT(*) FROM memories", [], |row| {
+                        row.get::<_, i64>(0)
+                    })?)
+                })
+                .unwrap();
+            assert_eq!(count, i64::from(writes));
+        }
+    }
+}
+
+#[cfg(unix)]
+#[test]
+fn import_post_commit_hardening_error_does_not_report_zero_applied() {
+    let fixture = tempfile::tempdir().unwrap();
+    let physical = fixture.path().join("physical");
+    let replacement = fixture.path().join("replacement");
+    std::fs::create_dir(&physical).unwrap();
+    std::fs::create_dir(&replacement).unwrap();
+    let alias = fixture.path().join("configured-parent");
+    std::os::unix::fs::symlink(&physical, &alias).unwrap();
+    let db_path = alias.join("memory.db");
+    let mut c = ctx();
+    c.storage = crate::storage::Storage::open(crate::types::StorageConfig {
+        db_path: db_path.to_string_lossy().into_owned(),
+        storage_mode: crate::types::StorageMode::Local,
+        cloud_uri: None,
+        encrypt_cloud: false,
+        confidence_half_life_days: 30.0,
+        auto_sync: false,
+        sync_debounce_ms: 5000,
+    })
+    .unwrap();
+    let id = make_memory(&c, "original before post-commit error", &[]);
+    let input = fixture.path().join("input");
+    std::fs::create_dir(&input).unwrap();
+    write_md(
+        &input,
+        "edit.md",
+        Some(id),
+        1,
+        &[],
+        0.5,
+        "committed despite hardening error",
+        &[],
+    );
+    let sentinel = fixture.path().join("sentinel");
+    std::fs::write(&sentinel, "private fixture").unwrap();
+    // SQLite opened the canonical physical parent. Fault-inject a replacement
+    // of Storage's configured parent alias: its post-commit hardening fails,
+    // while the live DB/WAL/SHM themselves are never opened, moved or replaced.
+    std::os::unix::fs::symlink(&sentinel, replacement.join("memory.db")).unwrap();
+    std::fs::remove_file(&alias).unwrap();
+    std::os::unix::fs::symlink(&replacement, &alias).unwrap();
+    let result = memory_import_markdown(&c, json!({"input_dir":input,"confirm":true}));
+    std::fs::remove_file(&alias).unwrap();
+    std::os::unix::fs::symlink(&physical, &alias).unwrap();
+    assert!(
+        result.get("error").is_some(),
+        "hardening failure must remain an error"
+    );
+    assert_eq!(db_row(&c, id).0, "committed despite hardening error");
+    assert!(c
+        .storage
+        .with_connection(|conn| Ok(conn.is_autocommit()))
+        .unwrap());
+    assert!(
+        result["applied"].is_null(),
+        "post-commit error cannot claim applied=0: {result}"
+    );
+    assert_eq!(result["outcome"], "unknown");
+    assert_eq!(result["attempted"], 1);
+    assert_eq!(
+        std::fs::read_to_string(sentinel).unwrap(),
+        "private fixture"
+    );
+}
+
+#[test]
+fn import_tab_separated_invalid_identity_and_version_are_not_missing() {
+    for confirm in [true, false] {
+        for key in [
+            "engram_id",
+            "engram_version",
+            "'engram_id'",
+            "\"engram_version\"",
+        ] {
+            let c = ctx();
+            let fixture = tempfile::tempdir().unwrap();
+            std::fs::write(
+                fixture.path().join("invalid.md"),
+                format!("---\n{key}:\t0\n---\nmust not become a note\n"),
+            )
+            .unwrap();
+            let result =
+                memory_import_markdown(&c, json!({"input_dir":fixture.path(),"confirm":confirm}));
+            assert!(
+                result.get("error").is_some(),
+                "{key}, confirm={confirm}: {result}"
+            );
+            assert_eq!(result["applied"], 0);
+            assert_eq!(
+                c.storage
+                    .with_connection(|conn| Ok(conn.query_row(
+                        "SELECT COUNT(*) FROM memories",
+                        [],
+                        |row| row.get::<_, i64>(0)
+                    )?))
+                    .unwrap(),
+                0
+            );
+        }
+    }
+}
+
+#[test]
+fn import_database_error_rolls_back_earlier_updates() {
+    let c = ctx();
+    let first = make_memory(&c, "first original", &["first"]);
+    let second = make_memory(&c, "second original", &["second"]);
+    let before = (db_row(&c, first), db_row(&c, second));
+    let dir = tempfile::tempdir().unwrap();
+    write_md(
+        dir.path(),
+        "00-first.md",
+        Some(first),
+        1,
+        &["changed"],
+        0.9,
+        "first edit",
+        &[],
+    );
+    write_md(
+        dir.path(),
+        "99-second.md",
+        Some(second),
+        1,
+        &["changed"],
+        0.9,
+        "second edit",
+        &[],
+    );
+    c.storage.with_connection(|conn| {
+        conn.execute_batch(&format!("CREATE TRIGGER reject_second_import BEFORE UPDATE OF content ON memories WHEN NEW.id = {second} BEGIN SELECT RAISE(ABORT, 'injected import failure'); END;"))?;
+        Ok(())
+    }).unwrap();
+    let result = memory_import_markdown(&c, json!({"input_dir":dir.path(),"confirm":true}));
+    assert!(result.get("error").is_some(), "{result}");
+    assert_eq!(result["applied"], 0);
+    assert_eq!((db_row(&c, first), db_row(&c, second)), before);
+    assert_eq!(db_tags(&c, first), vec!["first"]);
+    assert_eq!(db_tags(&c, second), vec!["second"]);
+}
+
diff --git a/src/mcp/handlers/markdown_export/import/lookup.rs b/src/mcp/handlers/markdown_export/import/lookup.rs
deleted file mode 100644
index a5d88c73209cd67a469012d72c73fb778d7872ed..0000000000000000000000000000000000000000
--- a/src/mcp/handlers/markdown_export/import/lookup.rs
+++ /dev/null
@@ -1,20 +0,0 @@
-use crate::error::EngramError;
-use crate::mcp::handlers::HandlerContext;
-
-pub(super) fn db_state_for_memory(
-    ctx: &HandlerContext,
-    engram_id: i64,
-) -> Result<Option<(String, i64)>, EngramError> {
-    ctx.storage.with_connection(|conn| {
-        let result = conn.query_row(
-            "SELECT content_hash, version FROM memories WHERE id = ?1",
-            [engram_id],
-            |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
-        );
-        match result {
-            Ok(row) => Ok(Some(row)),
-            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
-            Err(e) => Err(EngramError::Database(e)),
-        }
-    })
-}
diff --git a/src/mcp/handlers/markdown_export/import/security_tests.rs b/src/mcp/handlers/markdown_export/import/security_tests.rs
new file mode 100644
index 0000000000000000000000000000000000000000..b900bbf53b5a084a40faf8f1f99aa45e5c4870fc
--- /dev/null
+++ b/src/mcp/handlers/markdown_export/import/security_tests.rs
@@ -0,0 +1,473 @@
+//! Import security regressions use private synthetic fixtures. Environment
+//! configuration and potentially blocking inputs run only in bounded children.
+#![cfg(unix)]
+
+use super::handler::memory_import_markdown;
+use super::test_support::ctx;
+use crate::auth::{PermissionSet, TokenClaims, TransportPrincipal, UserId};
+use crate::mcp::handlers::{dispatch, HandlerContext};
+use crate::storage::queries::create_memory;
+use crate::types::CreateMemoryInput;
+use serde_json::{json, Value};
+use std::fs;
+use std::os::unix::fs::{symlink, PermissionsExt};
+use std::path::Path;
+
+fn restricted_context() -> HandlerContext {
+    let mut context = ctx();
+    context.principal = Some(
+        TransportPrincipal::from_token_claims(TokenClaims {
+            user_id: UserId::from_string("import-fixture-user"),
+            key_id: "import-fixture-key".into(),
+            permissions: PermissionSet::standard_user(),
+            namespace: Some("alpha".into()),
+            issued_at: chrono::Utc::now(),
+            expires_at: None,
+        })
+        .unwrap(),
+    );
+    context
+}
+
+fn memory(context: &HandlerContext, workspace: &str, content: &str) -> i64 {
+    context
+        .storage
+        .with_transaction(|conn| {
+            Ok(create_memory(
+                conn,
+                &CreateMemoryInput {
+                    content: content.into(),
+                    workspace: Some(workspace.into()),
+                    tags: vec!["original".into()],
+                    importance: Some(0.5),
+                    ..Default::default()
+                },
+            )?
+            .id)
+        })
+        .unwrap()
+}
+
+#[derive(Debug, PartialEq)]
+struct MemoryState {
+    id: i64,
+    content: String,
+    version: i64,
+    importance: Option<f64>,
+    workspace: String,
+}
+
+#[derive(Debug, PartialEq)]
+struct DatabaseState {
+    memories: Vec<MemoryState>,
+    tags: Vec<(i64, String)>,
+}
+
+fn database_state(context: &HandlerContext) -> DatabaseState {
+    context
+        .storage
+        .with_connection(|conn| {
+            let mut statement = conn.prepare(
+                "SELECT id,content,version,importance,workspace FROM memories ORDER BY id",
+            )?;
+            let memories = statement
+                .query_map([], |row| {
+                    Ok(MemoryState {
+                        id: row.get(0)?,
+                        content: row.get(1)?,
+                        version: row.get(2)?,
+                        importance: row.get(3)?,
+                        workspace: row.get(4)?,
+                    })
+                })?
+                .collect::<rusqlite::Result<Vec<_>>>()?;
+            let mut statement = conn.prepare(
+                "SELECT mt.memory_id,t.name FROM memory_tags mt \
+                 JOIN tags t ON t.id=mt.tag_id ORDER BY mt.memory_id,t.name",
+            )?;
+            let tags = statement
+                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
+                .collect::<rusqlite::Result<Vec<_>>>()?;
+            Ok(DatabaseState { memories, tags })
+        })
+        .unwrap()
+}
+
+fn write_note(path: &Path, id: Option<&str>, workspace: &str, content: &str) {
+    let id_line = id.map_or_else(String::new, |id| format!("engram_id: {id}\n"));
+    fs::write(
+        path,
+        format!(
+            "---\n{id_line}engram_workspace: {workspace}\nengram_type: note\n\
+             engram_version: 1\nengram_importance: 0.9\nengram_tags:\n  - changed\n\
+             ---\n{content}\n"
+        ),
+    )
+    .unwrap();
+    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
+}
+
+fn invoke(context: &HandlerContext, params: Value, through_dispatch: bool) -> Value {
+    if through_dispatch {
+        dispatch(context, "memory_import_markdown", params)
+    } else {
+        memory_import_markdown(context, params)
+    }
+}
+
+fn arguments(root: &Path, confirm: bool, force_version: bool) -> Value {
+    json!({
+        "input_dir": root,
+        "workspace": "alpha",
+        "confirm": confirm,
+        "force_version": force_version,
+    })
+}
+
+fn assert_rejected(result: &Value) {
+    assert!(
+        result.get("error").is_some(),
+        "expected a top-level error: {result}"
+    );
+    assert_eq!(
+        result["applied"], 0,
+        "failed import reported applied writes: {result}"
+    );
+}
+
+fn foreign_and_missing(root: &Path, confirm: bool, force_version: bool) {
+    for through_dispatch in [false, true] {
+        let foreign = restricted_context();
+        let id = memory(&foreign, "beta", "beta-only-secret");
+        write_note(
+            &root.join("note.md"),
+            Some(&id.to_string()),
+            "alpha",
+            "attempted foreign edit",
+        );
+        let missing = restricted_context();
+        let foreign_before = database_state(&foreign);
+        let missing_before = database_state(&missing);
+        let params = arguments(root, confirm, force_version);
+        let denied_foreign = invoke(&foreign, params.clone(), through_dispatch);
+        let denied_missing = invoke(&missing, params, through_dispatch);
+
+        assert_rejected(&denied_foreign);
+        assert_rejected(&denied_missing);
+        assert_eq!(
+            denied_foreign["error"]["code"], "not_found",
+            "{denied_foreign}"
+        );
+        assert_eq!(
+            denied_foreign["error"], denied_missing["error"],
+            "the same caller-supplied ID must not reveal whether a foreign row exists"
+        );
+        let rendered = denied_foreign.to_string();
+        for secret in ["beta-only-secret", "content_hash", "db_version", "beta"] {
+            assert!(
+                !rendered.contains(secret),
+                "foreign state leaked: {rendered}"
+            );
+        }
+        assert_eq!(database_state(&foreign), foreign_before);
+        assert_eq!(database_state(&missing), missing_before);
+    }
+}
+
+fn rejected_batch_rolls_back(root: &Path) {
+    for through_dispatch in [false, true] {
+        let context = restricted_context();
+        let own = memory(&context, "alpha", "own original");
+        let foreign = memory(&context, "beta", "foreign original");
+        write_note(
+            &root.join("00-own.md"),
+            Some(&own.to_string()),
+            "alpha",
+            "own edit",
+        );
+        write_note(
+            &root.join("99-foreign.md"),
+            Some(&foreign.to_string()),
+            "alpha",
+            "foreign edit",
+        );
+        let before = database_state(&context);
+        let result = invoke(&context, arguments(root, true, true), through_dispatch);
+        assert_rejected(&result);
+        assert_eq!(result["error"]["code"], "not_found", "{result}");
+        assert_eq!(
+            database_state(&context),
+            before,
+            "earlier authorized writes survived a denied batch"
+        );
+    }
+}
+
+fn authorized_update_and_create(root: &Path) {
+    for through_dispatch in [false, true] {
+        let context = restricted_context();
+        let own = memory(&context, "alpha", "own original");
+        write_note(
+            &root.join("own.md"),
+            Some(&own.to_string()),
+            "alpha",
+            "own edit",
+        );
+        // The explicit authorized override takes precedence over frontmatter.
+        write_note(&root.join("new.md"), None, "beta", "authorized new note");
+        let before = database_state(&context);
+        let preview = invoke(&context, arguments(root, false, false), through_dispatch);
+        assert!(preview.get("error").is_none(), "{preview}");
+        assert_eq!(preview["applied"], 0, "{preview}");
+        assert_eq!(database_state(&context), before, "preview mutated storage");
+
+        let result = invoke(&context, arguments(root, true, false), through_dispatch);
+        assert!(result.get("error").is_none(), "{result}");
+        assert_eq!(result["applied"], 2, "{result}");
+        let after = database_state(&context);
+        assert_eq!(after.memories.len(), 2);
+        assert!(after.memories.iter().all(|row| row.workspace == "alpha"));
+        let updated = after.memories.iter().find(|row| row.id == own).unwrap();
+        assert_eq!(updated.content, "own edit");
+        assert_eq!(updated.version, 2);
+        assert!((updated.importance.unwrap() - 0.9).abs() < 1e-6);
+        assert!(after.tags.contains(&(own, "changed".into())));
+    }
+}
+
+fn unconfined_is_denied_before_read(root: &Path) {
+    let context = restricted_context();
+    let absent = root.join("must-not-be-scanned");
+    assert!(!absent.exists());
+    for through_dispatch in [false, true] {
+        let before = database_state(&context);
+        let result = invoke(&context, arguments(&absent, true, false), through_dispatch);
+        assert_eq!(
+            result["error"]["code"], "permission_denied",
+            "authorization must precede path validation: {result}"
+        );
+        assert_eq!(result["error"]["required_mode"], "maintenance", "{result}");
+        assert!(!absent.exists());
+        assert_eq!(database_state(&context), before);
+    }
+}
+
+fn unsafe_entry(root: &Path, kind: &str) {
+    let context = restricted_context();
+    let own = memory(&context, "alpha", "own original");
+    write_note(
+        &root.join("00-own.md"),
+        Some(&own.to_string()),
+        "alpha",
+        "own edit",
+    );
+    let outside = root.parent().unwrap().join("outside.md");
+    write_note(&outside, None, "alpha", "outside fixture content");
+    match kind {
+        "leaf-link" => symlink(&outside, root.join("99-leaf.md")).unwrap(),
+        "directory-link" => {
+            let directory = root.parent().unwrap().join("outside-directory");
+            fs::create_dir(&directory).unwrap();
+            fs::set_permissions(&directory, fs::Permissions::from_mode(0o700)).unwrap();
+            write_note(
+                &directory.join("outside.md"),
+                None,
+                "alpha",
+                "outside directory content",
+            );
+            symlink(&directory, root.join("99-directory")).unwrap();
+        }
+        "cycle" => symlink(root, root.join("99-cycle")).unwrap(),
+        "hardlink" => fs::hard_link(&outside, root.join("99-hardlink.md")).unwrap(),
+        "fifo" => {
+            use std::os::unix::ffi::OsStrExt;
+            let name =
+                std::ffi::CString::new(root.join("99-fifo.md").as_os_str().as_bytes()).unwrap();
+            // SAFETY: name is NUL-terminated, references only this private test
+            // fixture, and remains alive for the duration of mkfifo.
+            assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
+        }
+        _ => panic!("unknown unsafe-entry fixture: {kind}"),
+    }
+    let before = database_state(&context);
+    for through_dispatch in [false, true] {
+        let result = invoke(&context, arguments(root, true, false), through_dispatch);
+        assert_rejected(&result);
+        assert_eq!(
+            database_state(&context),
+            before,
+            "filesystem preflight allowed partial application"
+        );
+    }
+    assert!(fs::read_to_string(outside)
+        .unwrap()
+        .contains("outside fixture content"));
+}
+
+fn invalid_explicit_ids(root: &Path) {
+    let context = restricted_context();
+    let before = database_state(&context);
+    for invalid in ["", "0", "-1", "-2", "not-an-id", "9223372036854775808"] {
+        write_note(
+            &root.join("invalid.md"),
+            Some(invalid),
+            "alpha",
+            "must not become a new note",
+        );
+        for through_dispatch in [false, true] {
+            let result = invoke(&context, arguments(root, true, true), through_dispatch);
+            assert_rejected(&result);
+            assert_eq!(
+                result["error"]["code"], "invalid_params",
+                "invalid explicit ID {invalid}: {result}"
+            );
+            assert_eq!(database_state(&context), before);
+        }
+    }
+}
+
+#[test]
+fn p0_import_empty_canonical_id_does_not_become_a_new_note() {
+    let context = ctx();
+    let fixture = tempfile::tempdir().unwrap();
+    fs::write(
+        fixture.path().join("empty-id.md"),
+        "---\nengram_id:\n---\nmust not create\n",
+    )
+    .unwrap();
+    let before = database_state(&context);
+    let result =
+        memory_import_markdown(&context, json!({"input_dir":fixture.path(),"confirm":true}));
+    assert_rejected(&result);
+    assert_eq!(result["error"]["code"], "invalid_params");
+    assert_eq!(database_state(&context), before);
+}
+
+#[test]
+fn p0_import_fixture_child() {
+    let Some(case) = std::env::var_os("ENGRAM_IMPORT_SECURITY_CASE") else {
+        return;
+    };
+    let root = std::path::PathBuf::from(std::env::var_os("ENGRAM_IMPORT_SECURITY_ROOT").unwrap());
+    let case = case.to_str().unwrap();
+    match case {
+        "idor-preview" => foreign_and_missing(&root, false, false),
+        "idor-preview-force" => foreign_and_missing(&root, false, true),
+        "idor-confirm" => foreign_and_missing(&root, true, false),
+        "idor-confirm-force" => foreign_and_missing(&root, true, true),
+        "batch-rollback" => rejected_batch_rolls_back(&root),
+        "authorized" => authorized_update_and_create(&root),
+        "unconfined" => unconfined_is_denied_before_read(&root),
+        "invalid-id" => invalid_explicit_ids(&root),
+        _ => unsafe_entry(&root, case),
+    }
+}
+
+fn run_fixture(case: &str) {
+    use std::process::{Command, Stdio};
+    use std::time::{Duration, Instant};
+
+    let fixture = tempfile::tempdir().unwrap();
+    let root = fixture.path().join("base");
+    fs::create_dir(&root).unwrap();
+    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
+    let mut command = Command::new(std::env::current_exe().unwrap());
+    command
+        .args([
+            "--exact",
+            "mcp::handlers::markdown_export::import::security_tests::p0_import_fixture_child",
+            "--nocapture",
+            "--test-threads=1",
+        ])
+        .env_clear()
+        .env("PATH", "/usr/bin:/bin")
+        .env("TMPDIR", fixture.path())
+        .env("ENGRAM_IMPORT_SECURITY_CASE", case)
+        .env("ENGRAM_IMPORT_SECURITY_ROOT", &root)
+        .env("ENGRAM_PERMISSION_MODE", "scoped_write")
+        .current_dir(&root)
+        .stdin(Stdio::null())
+        .stdout(Stdio::piped())
+        .stderr(Stdio::piped());
+    if case != "unconfined" {
+        command.env("ENGRAM_EXPORT_BASE_DIR", &root);
+    }
+    let mut child = command.spawn().unwrap();
+    let deadline = Instant::now() + Duration::from_secs(10);
+    while child.try_wait().unwrap().is_none() {
+        if Instant::now() >= deadline {
+            let _ = child.kill();
+            let output = child.wait_with_output().unwrap();
+            panic!(
+                "import fixture {case} exceeded 10 seconds (unsafe reads must not block):\n{}\n{}",
+                String::from_utf8_lossy(&output.stdout),
+                String::from_utf8_lossy(&output.stderr)
+            );
+        }
+        std::thread::sleep(Duration::from_millis(10));
+    }
+    let output = child.wait_with_output().unwrap();
+    assert!(
+        output.status.success(),
+        "import fixture {case} failed:\n{}\n{}",
+        String::from_utf8_lossy(&output.stdout),
+        String::from_utf8_lossy(&output.stderr)
+    );
+    assert!(
+        String::from_utf8_lossy(&output.stdout).contains("1 passed"),
+        "the exact child-test filter did not execute the fixture"
+    );
+}
+
+macro_rules! fixture_test {
+    ($name:ident, $case:literal) => {
+        #[test]
+        fn $name() {
+            run_fixture($case);
+        }
+    };
+}
+
+fixture_test!(p0_import_foreign_id_preview_matches_missing, "idor-preview");
+fixture_test!(
+    p0_import_foreign_id_forced_preview_matches_missing,
+    "idor-preview-force"
+);
+fixture_test!(p0_import_foreign_id_confirm_matches_missing, "idor-confirm");
+fixture_test!(
+    p0_import_foreign_id_forced_confirm_matches_missing,
+    "idor-confirm-force"
+);
+fixture_test!(
+    p0_import_denied_batch_rolls_back_earlier_own_update,
+    "batch-rollback"
+);
+fixture_test!(
+    p0_import_authorized_update_and_create_remain_available,
+    "authorized"
+);
+fixture_test!(
+    p0_import_without_root_requires_maintenance_before_path_validation,
+    "unconfined"
+);
+fixture_test!(
+    p0_import_rejects_leaf_symlink_without_partial_writes,
+    "leaf-link"
+);
+fixture_test!(
+    p0_import_rejects_directory_symlink_without_partial_writes,
+    "directory-link"
+);
+fixture_test!(
+    p0_import_rejects_directory_cycle_in_bounded_process,
+    "cycle"
+);
+fixture_test!(
+    p0_import_rejects_hardlink_without_partial_writes,
+    "hardlink"
+);
+fixture_test!(p0_import_rejects_fifo_without_blocking, "fifo");
+fixture_test!(
+    p0_import_rejects_explicit_invalid_ids_instead_of_creating,
+    "invalid-id"
+);
diff --git a/src/mcp/permission.rs b/src/mcp/permission.rs
index 920c925908fc831afd3fa1583db8b0f8e56b6c22..6b050819246ffb4856c6bc34f98161ddc1289e91 100644
--- a/src/mcp/permission.rs
+++ b/src/mcp/permission.rs
@@ -149,0 +150,9 @@ pub fn required_mode_for_call(tool_name: &str, params: &Value) -> PermissionMode
+    // Workspace write permission does not grant arbitrary server filesystem
+    // access. Only the administrator's configured export root permits it.
+    if matches!(
+        canonical,
+        "memory_export_markdown" | "memory_import_markdown"
+    ) && !std::env::var("ENGRAM_EXPORT_BASE_DIR").is_ok_and(|root| !root.trim().is_empty())
+    {
+        return PermissionMode::Maintenance;
+    }
@@ -227 +236 @@ fn status_report_for(active: Option<PermissionMode>, target_tool: Option<&str>)
-        let req = required_mode(tool);
+        let req = required_mode(tool).map(|_| required_mode_for_call(tool, &Value::Null));
@@ -258,0 +268 @@ fn status_report_for(active: Option<PermissionMode>, target_tool: Option<&str>)
+                        .map(|_| required_mode_for_call(t.name, &Value::Null))
diff --git a/src/mcp/tools/catalog/admin.rs b/src/mcp/tools/catalog/admin.rs
index a148852b7a9a21647bd77dffd999e5607c14b312..8b05a6996d00d43a32d94885e1a786aa2acc8972 100644
--- a/src/mcp/tools/catalog/admin.rs
+++ b/src/mcp/tools/catalog/admin.rs
@@ -340 +340 @@ pub const TOOLS: &[ToolDef] = &[
-            description: "Export a workspace as human-readable Markdown files with YAML frontmatter and wiki-style [[links]]. Creates one .md file per memory, organized by type in subdirectories, with an index.md overview.",
+            description: "Write unexpired workspace memories as Markdown files with YAML frontmatter and directional [[links]]. Requires scoped_write within ENGRAM_EXPORT_BASE_DIR, otherwise maintenance. Output directories must be owned by the server user, not group/world writable and have no extended/default ACLs. Safe filesystem writes require Linux/macOS.",
@@ -351 +351 @@ pub const TOOLS: &[ToolDef] = &[
-            annotations: ToolAnnotations::read_only(),
+            annotations: ToolAnnotations::mutating(),
@@ -356 +356 @@ pub const TOOLS: &[ToolDef] = &[
-            description: "Import memories from Markdown files with engram_ frontmatter (RFC 0004). Review mode by default (confirm: false) — returns a staged list without writing. Detects drift via content_hash and version conflicts via engram_version. Ignores non-engram_ frontmatter keys (Obsidian-safe).",
+            description: "Import bounded Markdown files with engram_ frontmatter on Linux/macOS. Requires scoped_write within ENGRAM_EXPORT_BASE_DIR, otherwise maintenance, including review mode. Input directories must be server-owned, not group/world writable and have no ACLs; links and SQLite artifacts are refused. Authorizes existing IDs by their persisted workspace. Review is default; confirm uses one database transaction. Pre-commit failures roll back; commit/completion failures report an unknown outcome requiring reconciliation. Detects drift/version conflicts and ignores non-engram_ keys (Obsidian-safe). Skips hidden entries beneath the input root. Missing IDs are rejected for workspace-restricted principals; unrestricted local owners treat them as new.",
diff --git a/src/portability/markdown.rs b/src/portability/markdown.rs
index cbd7a23c769132730e33833ff6ed17c377a25e54..17c47f7e160b0bbebb67a9ac5165ce59072f8f3b 100644
--- a/src/portability/markdown.rs
+++ b/src/portability/markdown.rs
@@ -161,2 +161,2 @@ pub fn export_markdown(storage: &Storage, opts: &ExportOptions) -> Result<Export
-    if let Some(err) = val.get("error").and_then(|v| v.as_str()) {
-        if val.get("files_written").and_then(|v| v.as_u64()) == Some(0) {
+    if let Some(err) = val.get("error") {
+        if err.is_string() && val.get("files_written").and_then(|v| v.as_u64()) == Some(0) {
@@ -169 +169 @@ pub fn export_markdown(storage: &Storage, opts: &ExportOptions) -> Result<Export
-        return Err(EngramError::InvalidInput(err.to_string()));
+        return Err(EngramError::InvalidInput(handler_error_message(err)));
@@ -198,2 +198,12 @@ pub fn import_markdown(storage: &Storage, opts: &ImportOptions) -> Result<Import
-    if let Some(err) = val.get("error").and_then(|v| v.as_str()) {
-        return Err(EngramError::InvalidInput(err.to_string()));
+    if let Some(err) = val.get("error") {
+        return Err(EngramError::InvalidInput(handler_error_message(err)));
+    }
+
+    if val
+        .get("files")
+        .and_then(|v| v.as_array())
+        .is_some_and(|files| files.iter().any(|file| file["status"] == "error"))
+    {
+        return Err(EngramError::InvalidInput(
+            "Markdown import reported file errors; inspect the tool result before retrying".into(),
+        ));
@@ -227,0 +238,8 @@ pub fn import_markdown(storage: &Storage, opts: &ImportOptions) -> Result<Import
+fn handler_error_message(error: &serde_json::Value) -> String {
+    error
+        .as_str()
+        .or_else(|| error.get("message").and_then(|value| value.as_str()))
+        .unwrap_or("Markdown operation failed with a structured error")
+        .to_string()
+}
+
diff --git a/tests/portability_permissions_routing_tests.rs b/tests/portability_permissions_routing_tests.rs
index b8dd83061a571789ef2ed03014f744b768705261..ddd9d3534dca470637e971784fdf7c6c85e88034 100644
--- a/tests/portability_permissions_routing_tests.rs
+++ b/tests/portability_permissions_routing_tests.rs
@@ -191 +191 @@ fn test_mcp_permission_modes_enforcement_and_denial_shapes() {
-        permission_denial_for_mode("memory_export_markdown", PermissionMode::ReadOnly).is_none()
+        permission_denial_for_mode("memory_export_markdown", PermissionMode::ReadOnly).is_some()
diff --git a/tests/storage_posix_lock_regression_tests.rs b/tests/storage_posix_lock_regression_tests.rs
index ab146eada12930a4e074e9b65d4d20ca0ef31f51..22fe0bc0ea9a1ead98e74a8eecbc272d3d54644e 100644
--- a/tests/storage_posix_lock_regression_tests.rs
+++ b/tests/storage_posix_lock_regression_tests.rs
@@ -169,0 +170,28 @@ fn committed_writes_survive_another_process_open_and_close() {
+#[test]
+fn markdown_import_refuses_sqlite_aliases_without_dropping_locks() {
+    let fixture = tempfile::tempdir().unwrap();
+    let db_path = fixture.path().join("memory.db");
+    let storage = open_with_probe_table(&db_path);
+    insert_probe(&storage, "create-wal-and-shm");
+    for (index, suffix) in ["", "-wal", "-shm"].iter().enumerate() {
+        let artifact = std::path::PathBuf::from(format!("{}{suffix}", db_path.display()));
+        assert!(artifact.exists(), "SQLite artifact fixture must exist");
+        for hardlink in [false, true] {
+            let input = fixture.path().join(format!("input-{index}-{hardlink}"));
+            std::fs::create_dir(&input).unwrap();
+            let alias = input.join("artifact.md");
+            if hardlink {
+                std::fs::hard_link(&artifact, &alias).unwrap();
+            } else {
+                std::os::unix::fs::symlink(&artifact, &alias).unwrap();
+            }
+            let result = engram::portability::preview_markdown(&storage, input, None);
+            assert!(
+                result.is_err(),
+                "SQLite alias must be refused before regular-file open"
+            );
+            assert_writes_survive_foreign_open_close(&storage, &db_path);
+        }
+    }
+}
+
# staged (index) content for docs/MCP_TOOLS.md; the working-tree diff above may differ
diff --git a/docs/MCP_TOOLS.md b/docs/MCP_TOOLS.md
index 69caa0815a1b5189627784ad249c9ad61653f490..9c8698757e9d3d95f3a684634bb10c9711ba909b 100644
--- a/docs/MCP_TOOLS.md
+++ b/docs/MCP_TOOLS.md
@@ -210 +210 @@ Total tools: **301**
-| `memory_export_markdown` | advanced | memory.admin | always | readOnlyHint | `workspace` |
+| `memory_export_markdown` | advanced | memory.admin | always | mutating (no MCP hints) | `workspace` |
@@ -3539 +3539 @@ Create a memory from a specific Langfuse trace ID.
-Export a workspace as human-readable Markdown files with YAML frontmatter and wiki-style [[links]]. Creates one .md file per memory, organized by type in subdirectories, with an index.md overview.
+Write unexpired workspace memories as Markdown files with YAML frontmatter and directional [[links]]. Requires scoped_write within ENGRAM_EXPORT_BASE_DIR, otherwise maintenance. Output directories must be owned by the server user, not group/world writable and have no extended/default ACLs. Safe filesystem writes require Linux/macOS.
@@ -3544 +3544 @@ Export a workspace as human-readable Markdown files with YAML frontmatter and wi
-- Annotations: readOnlyHint
+- Annotations: mutating (no MCP hints)
@@ -3556 +3556 @@ Export a workspace as human-readable Markdown files with YAML frontmatter and wi
-Import memories from Markdown files with engram_ frontmatter (RFC 0004). Review mode by default (confirm: false) — returns a staged list without writing. Detects drift via content_hash and version conflicts via engram_version. Ignores non-engram_ frontmatter keys (Obsidian-safe).
+Import bounded Markdown files with engram_ frontmatter on Linux/macOS. Requires scoped_write within ENGRAM_EXPORT_BASE_DIR, otherwise maintenance, including review mode. Input directories must be server-owned, not group/world writable and have no ACLs; links and SQLite artifacts are refused. Authorizes existing IDs by their persisted workspace. Review is default; confirm uses one database transaction. Pre-commit failures roll back; commit/completion failures report an unknown outcome requiring reconciliation. Detects drift/version conflicts and ignores non-engram_ keys (Obsidian-safe).
# staged (index) content for docs/migrations/v0.24.0-markdown-import-security.md; the working-tree diff above may differ
diff --git a/docs/migrations/v0.24.0-markdown-import-security.md b/docs/migrations/v0.24.0-markdown-import-security.md
new file mode 100644
index 0000000000000000000000000000000000000000..e4beb37b2fca2923834b46d15c618329ab31a6ae
--- /dev/null
+++ b/docs/migrations/v0.24.0-markdown-import-security.md
@@ -0,0 +1,59 @@
+# Markdown import security and compatibility
+
+Local RISK-0011 candidate, stacked on export `9bca4efa`; no tag/release or database migration.
+SQLite + WAL remains canonical. This is hardening of the existing import, without a Git projection.
+
+## Access and filesystem
+
+`memory_import_markdown` requires at least scoped_write within the nonempty administrator-configured
+`ENGRAM_EXPORT_BASE_DIR`. Without that root it requires maintenance, including dry-run/review: reading
+server files is an effect. Direct Rust/CLI calls enforce the same restrictions. Unconstrained local
+owner calls without a principal or permission restriction retain their local behavior.
+
+Linux/macOS input roots and descendant directories use pinned descriptors and no-follow operations.
+They must be owned by the server OS user, not group/world writable and have no extended/default ACLs.
+As in export, the conservative policy also refuses deny-only ACLs; use a private subdirectory without
+an ACL. Engram does not change ACLs. The administrator protects the root/ancestors against replacement
+by processes sharing the server OS identity; this is not a sandbox against that identity. Unsupported
+platforms fail explicitly before input reads instead of using an unsafe fallback.
+
+The reader refuses symlinks (including cycles), hardlinked Markdown, non-regular Markdown and active
+SQLite database/WAL/SHM/journal aliases. Metadata/identity checks precede regular-file open to preserve
+SQLite process/inode locks; directories are opened with O_DIRECTORY. Input is sorted and limited to:
+
+- 32 descendant directory levels and 20000 directory entries, including non-Markdown entries;
+- 10000 Markdown files, 4 MiB per file and 64 MiB total content read;
+- UTF-8 names and content; inspection/read/limit failures are errors.
+
+Index/README and dot/underscore-prefixed Markdown files remain excluded as before. Filesystem IO and
+parsing finish before database transactions. A filesystem failure applies no database writes.
+
+## IDs, versions and batch application
+
+An existing `engram_id` is authorized against its stored workspace inside the same snapshot used for
+classification/application. A file's claimed workspace is never authorization for that ID. For a
+restricted principal, missing and foreign IDs produce the same not_found error, without exposing
+hash, version, workspace or contents. For unrestricted local owners the legacy missing-ID→new
+behavior remains. Absent IDs allow creation; explicitly invalid/zero/negative IDs and duplicate IDs
+within an input batch are rejected. Explicit versions must be positive integers. The scalar-key parser recognizes spaces/tabs after the
+colon and quoted/unquoted scalar keys; it is not a general YAML object/alias deserializer.
+
+Creation normalizes and authorizes its effective workspace (explicit override, then frontmatter,
+then default). An authorized override still takes precedence over frontmatter. Non-engram_ keys
+remain ignored; legacy hash comparison, tags/importance updates and force_version remain supported.
+
+Review uses a read snapshot and changes no memories. Confirm uses BEGIN IMMEDIATE: it authorizes and
+classifies the whole batch before its first write, then applies within that transaction. There is no
+classification/update race with another SQLite writer. Parsing/authorization/pre-commit errors fail the batch, and pre-commit database errors roll back
+earlier writes (`outcome: not_applied`, `applied: 0`). Storage can also fail during COMMIT or while
+revalidating permissions after COMMIT. When the transaction callback completed but Storage returns
+an error, the result is conservatively `outcome: unknown`, `applied: null`, and `attempted: N` actions.
+Inspect the canonical database before retrying; a post-commit error may accompany persisted writes. No filesystem IO,
+network call or embedding API occurs inside the transaction. Version conflicts remain staged in the
+report while other eligible files may be applied, preserving the existing conflict workflow.
+
+The MCP result uses structured errors; the public Rust wrapper propagates them. A report with zeros
+is never a substitute for a failed import. Old per-file error/partial-success behavior is intentionally
+replaced by a failed batch. Reconcile duplicate IDs left by old content-derived export names before
+importing. Publication approval, secret scanning, manifests/pruning and roundtrip graph metadata remain
+in the proposed P1/RFC scope.
# staged (index) content for src/mcp/handlers/markdown_export/export/filesystem/reader.rs; the working-tree diff above may differ
diff --git a/src/mcp/handlers/markdown_export/export/filesystem/reader.rs b/src/mcp/handlers/markdown_export/export/filesystem/reader.rs
new file mode 100644
index 0000000000000000000000000000000000000000..3d2a0dd3a6ffe8bc772081839c7f664b099fc844
--- /dev/null
+++ b/src/mcp/handlers/markdown_export/export/filesystem/reader.rs
@@ -0,0 +1,222 @@
+//! Bounded, descriptor-relative Markdown input. Never follow input links or
+//! transiently open an existing SQLite-bearing inode to inspect it.
+
+use std::collections::BTreeSet;
+use std::ffi::{CString, OsString};
+use std::fs::File;
+use std::io::{self, Read};
+use std::os::unix::ffi::OsStringExt;
+use std::os::unix::fs::MetadataExt;
+use std::path::Path;
+
+use rustix::fs::{AtFlags, Dir, FileType, Mode, OFlags};
+
+use super::unix::{check_file, open_child, open_root, protected_inodes, trusted_directory};
+use crate::storage::Storage;
+
+use super::MarkdownSource;
+
+#[derive(Clone, Copy)]
+struct Limits {
+    depth: usize,
+    files: usize,
+    entries: usize,
+    file_bytes: usize,
+    total_bytes: usize,
+}
+
+impl Default for Limits {
+    fn default() -> Self {
+        Self {
+            depth: 32,
+            files: 10_000,
+            entries: 20_000,
+            file_bytes: 4 * 1024 * 1024,
+            total_bytes: 64 * 1024 * 1024,
+        }
+    }
+}
+
+fn invalid(message: &str) -> io::Error {
+    io::Error::new(io::ErrorKind::InvalidInput, message)
+}
+
+struct Walk<'a> {
+    root: &'a Path,
+    storage: &'a Storage,
+    limits: Limits,
+    protected: Vec<(u64, u64)>,
+    visited: BTreeSet<(u64, u64)>,
+    entries: usize,
+    bytes: usize,
+    sources: Vec<MarkdownSource>,
+}
+
+impl Walk<'_> {
+    fn directory(&mut self, dir: &File, relative: &Path, depth: usize) -> io::Result<()> {
+        if depth > self.limits.depth {
+            return Err(invalid("Markdown input exceeds directory depth limit"));
+        }
+        trusted_directory(dir)?;
+        let metadata = dir.metadata()?;
+        if !self.visited.insert((metadata.dev(), metadata.ino())) {
+            return Err(invalid("Markdown input contains a repeated directory"));
+        }
+        let mut names = Vec::new();
+        for entry in Dir::read_from(dir)? {
+            let entry = entry?;
+            let name = entry.file_name();
+            if matches!(name.to_bytes(), b"." | b"..") {
+                continue;
+            }
+            self.entries += 1;
+            if self.entries > self.limits.entries {
+                return Err(invalid("Markdown input exceeds directory entry limit"));
+            }
+            names.push(CString::from(name));
+        }
+        names.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
+        for name in names {
+            let leaf = OsString::from_vec(name.as_bytes().to_vec());
+            let path = relative.join(leaf);
+            let name_str = path
+                .file_name()
+                .and_then(|value| value.to_str())
+                .ok_or_else(|| invalid("Markdown input names must be UTF-8"))?;
+            let stat = rustix::fs::statat(dir, &name, AtFlags::SYMLINK_NOFOLLOW)?;
+            match FileType::from_raw_mode(stat.st_mode) {
+                FileType::Directory => {
+                    let child = open_child(dir, &name, false)?
+                        .ok_or_else(|| invalid("Markdown input directory disappeared"))?;
+                    self.directory(&child, &path, depth + 1)?;
+                }
+                FileType::Symlink => return Err(invalid("Markdown input links are not allowed")),
+                kind => {
+                    let lower = name_str.to_ascii_lowercase();
+                    if path.extension().and_then(|ext| ext.to_str()) != Some("md")
+                        || name_str.starts_with(['.', '_'])
+                        || matches!(lower.as_str(), "index.md" | "readme.md")
+                    {
+                        continue;
+                    }
+                    if kind != FileType::RegularFile {
+                        return Err(invalid("Markdown input must contain regular files"));
+                    }
+                    if self.sources.len() >= self.limits.files {
+                        return Err(invalid("Markdown input exceeds file count limit"));
+                    }
+                    // Metadata-only checks precede regular-file open: closing an
+                    // alias of SQLite could otherwise release process fcntl locks.
+                    self.storage
+                        .refuse_active_sqlite_artifact(self.root.join(&path))
+                        .map_err(io::Error::other)?;
+                    check_file(dir, &name, &self.protected)?;
+                    let remaining = self.limits.total_bytes.saturating_sub(self.bytes);
+                    let limit = self.limits.file_bytes.min(remaining);
+                    if stat.st_size < 0 || stat.st_size as u64 > limit as u64 {
+                        return Err(invalid("Markdown input exceeds byte limit"));
+                    }
+                    let fd = rustix::fs::openat(
+                        dir,
+                        &name,
+                        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK,
+                        Mode::empty(),
+                    )?;
+                    let file = File::from(fd);
+                    let opened = file.metadata()?;
+                    if !opened.is_file() || opened.nlink() != 1 {
+                        return Err(invalid("Markdown input file changed or has multiple links"));
+                    }
+                    let mut bytes = Vec::new();
+                    file.take(limit as u64 + 1).read_to_end(&mut bytes)?;
+                    if bytes.len() > limit {
+                        return Err(invalid("Markdown input exceeds byte limit"));
+                    }
+                    self.bytes += bytes.len();
+                    let raw = String::from_utf8(bytes)
+                        .map_err(|_| invalid("Markdown input content must be UTF-8"))?;
+                    self.sources.push(MarkdownSource {
+                        relative: path,
+                        raw,
+                    });
+                }
+            }
+        }
+        Ok(())
+    }
+}
+
+pub(in crate::mcp::handlers::markdown_export) fn read_markdown(
+    root: &Path,
+    storage: &Storage,
+) -> io::Result<Vec<MarkdownSource>> {
+    read_with_limits(root, storage, Limits::default())
+}
+
+fn read_with_limits(
+    root: &Path,
+    storage: &Storage,
+    limits: Limits,
+) -> io::Result<Vec<MarkdownSource>> {
+    storage
+        .refuse_active_sqlite_artifact(root)
+        .map_err(io::Error::other)?;
+    let dir = open_root(root, false)?.ok_or_else(|| invalid("Markdown input root not found"))?;
+    let mut walk = Walk {
+        root,
+        storage,
+        limits,
+        protected: protected_inodes(storage),
+        visited: BTreeSet::new(),
+        entries: 0,
+        bytes: 0,
+        sources: Vec::new(),
+    };
+    walk.directory(&dir, Path::new(""), 0)?;
+    Ok(walk.sources)
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn input_quotas_bound_files_bytes_entries_and_depth() {
+        let fixture = tempfile::tempdir().unwrap();
+        let root = fixture.path().canonicalize().unwrap();
+        let storage = Storage::open_in_memory().unwrap();
+        std::fs::write(root.join("a.md"), "12345").unwrap();
+        std::fs::write(root.join("b.md"), "67890").unwrap();
+        for limits in [
+            Limits {
+                file_bytes: 4,
+                ..Limits::default()
+            },
+            Limits {
+                total_bytes: 9,
+                ..Limits::default()
+            },
+            Limits {
+                files: 1,
+                ..Limits::default()
+            },
+            Limits {
+                entries: 1,
+                ..Limits::default()
+            },
+        ] {
+            assert!(read_with_limits(&root, &storage, limits).is_err());
+        }
+        std::fs::create_dir(root.join("nested")).unwrap();
+        assert!(read_with_limits(
+            &root,
+            &storage,
+            Limits {
+                depth: 0,
+                ..Limits::default()
+            }
+        )
+        .is_err());
+        assert_eq!(read_markdown(&root, &storage).unwrap().len(), 2);
+    }
+}
# staged (index) content for src/mcp/handlers/markdown_export/import/apply.rs; the working-tree diff above may differ
diff --git a/src/mcp/handlers/markdown_export/import/apply.rs b/src/mcp/handlers/markdown_export/import/apply.rs
index dd460f268c27645d62bfe827217a0359e34173dd..81f4df168527a53ef596ecacd5740430acc9924d 100644
--- a/src/mcp/handlers/markdown_export/import/apply.rs
+++ b/src/mcp/handlers/markdown_export/import/apply.rs
@@ -2,0 +3 @@ use std::collections::HashMap;
+use rusqlite::{Connection, OptionalExtension};
@@ -5 +6,6 @@ use serde_json::{json, Value};
-use crate::mcp::handlers::markdown_export::frontmatter::{frontmatter_tags, import_payload};
+use super::files::MarkdownSource;
+use super::status::{classify_import_status, ImportStatus};
+use crate::error::{EngramError, Result};
+use crate::mcp::handlers::markdown_export::frontmatter::{
+    extract_body, frontmatter_tags, import_payload, parse_frontmatter,
+};
@@ -6,0 +13,3 @@ use crate::mcp::handlers::HandlerContext;
+use crate::mcp::workspace_guard::ensure_memory_access;
+use crate::storage::queries::{compute_content_hash, compute_content_hash_raw};
+use crate::types::{CreateMemoryInput, UpdateMemoryInput};
@@ -8 +17,103 @@ use crate::mcp::handlers::HandlerContext;
-pub(super) fn create_memory_from_import(
+#[derive(Debug)]
+pub(super) struct ImportFailure {
+    error: EngramError,
+    // Storage can fail at COMMIT or in permission revalidation after COMMIT.
+    // A completed callback alone cannot prove either rollback or persistence.
+    attempted: Option<u64>,
+}
+
+impl From<EngramError> for ImportFailure {
+    fn from(error: EngramError) -> Self {
+        Self {
+            error,
+            attempted: None,
+        }
+    }
+}
+
+impl ImportFailure {
+    pub(super) fn into_value(self) -> Value {
+        use crate::mcp::error::ToolError;
+        if let Some(attempted) = self.attempted {
+            let mut value = ToolError::internal(
+                "Import completion is uncertain; inspect the canonical database before retrying",
+            )
+            .into_value();
+            value["applied"] = Value::Null;
+            value["attempted"] = json!(attempted);
+            value["outcome"] = json!("unknown");
+            value
+        } else {
+            let mut value = ToolError::from(self.error).into_value();
+            value["applied"] = json!(0);
+            value["outcome"] = json!("not_applied");
+            value
+        }
+    }
+}
+
+struct Input {
+    file: String,
+    id: Option<i64>,
+    version: i64,
+    fm: HashMap<String, String>,
+    body: String,
+}
+
+enum Mutation {
+    None,
+    Create(CreateMemoryInput),
+    Update(i64, UpdateMemoryInput),
+}
+
+struct Plan {
+    file: String,
+    id: Option<i64>,
+    status: ImportStatus,
+    mutation: Mutation,
+}
+
+fn positive_field(fm: &HashMap<String, String>, field: &str) -> Result<Option<i64>> {
+    fm.get(field)
+        .map(|value| {
+            value
+                .parse::<i64>()
+                .ok()
+                .filter(|value| *value > 0)
+                .ok_or_else(|| {
+                    EngramError::InvalidInput(format!("{field} must be a positive integer"))
+                })
+        })
+        .transpose()
+}
+
+fn prepare(sources: Vec<MarkdownSource>) -> Result<Vec<Input>> {
+    let mut ids = std::collections::BTreeSet::new();
+    sources
+        .into_iter()
+        .map(|source| {
+            let fm = parse_frontmatter(&source.raw);
+            let id = positive_field(&fm, "engram_id")?;
+            if id.is_some_and(|id| !ids.insert(id)) {
+                return Err(EngramError::InvalidInput(
+                    "Duplicate engram_id in Markdown input".into(),
+                ));
+            }
+            Ok(Input {
+                file: source
+                    .relative
+                    .to_str()
+                    .ok_or_else(|| {
+                        EngramError::InvalidInput("Markdown input names must be UTF-8".into())
+                    })?
+                    .to_string(),
+                id,
+                version: positive_field(&fm, "engram_version")?.unwrap_or(1),
+                body: extract_body(&source.raw).trim().to_string(),
+                fm,
+            })
+        })
+        .collect()
+}
+
+fn creation(
@@ -10,6 +121,5 @@ pub(super) fn create_memory_from_import(
-    frontmatter: &HashMap<String, String>,
-    body: &str,
-    workspace_override: Option<&str>,
-) -> Result<i64, crate::error::EngramError> {
-    let mut obj = import_payload(frontmatter, body, true);
-    if let Some(workspace) = workspace_override {
+    input: &Input,
+    workspace: Option<&str>,
+) -> Result<CreateMemoryInput> {
+    let mut obj = import_payload(&input.fm, &input.body, true);
+    if let Some(workspace) = workspace {
@@ -18,3 +128,14 @@ pub(super) fn create_memory_from_import(
-    let tags = frontmatter_tags(frontmatter);
-    if !tags.is_empty() {
-        obj.insert("tags".into(), json!(tags));
+    obj.insert("tags".into(), json!(frontmatter_tags(&input.fm)));
+    let mut parsed: CreateMemoryInput = serde_json::from_value(Value::Object(obj))
+        .map_err(|_| EngramError::InvalidInput("Invalid Markdown creation fields".into()))?;
+    let workspace =
+        crate::types::normalize_workspace(parsed.workspace.as_deref().unwrap_or("default"))
+            .map_err(|error| EngramError::InvalidInput(error.to_string()))?;
+    if ctx
+        .principal
+        .as_ref()
+        .is_some_and(|principal| !principal.allows_workspace(Some(&workspace)))
+    {
+        return Err(EngramError::Unauthorized(
+            "Markdown target workspace is not allowed".into(),
+        ));
@@ -21,0 +143,3 @@ pub(super) fn create_memory_from_import(
+    parsed.workspace = Some(workspace);
+    Ok(parsed)
+}
@@ -23,7 +147,7 @@ pub(super) fn create_memory_from_import(
-    serde_json::from_value::<crate::types::CreateMemoryInput>(Value::Object(obj))
-        .map_err(|e| crate::error::EngramError::Internal(format!("bad import payload: {e}")))
-        .and_then(|input| {
-            ctx.storage
-                .with_transaction(|conn| crate::storage::queries::create_memory(conn, &input))
-        })
-        .map(|new_memory| new_memory.id)
+fn update(input: &Input) -> Result<UpdateMemoryInput> {
+    let mut obj = import_payload(&input.fm, &input.body, false);
+    if input.fm.contains_key("engram_tags_list") {
+        obj.insert("tags".into(), json!(frontmatter_tags(&input.fm)));
+    }
+    serde_json::from_value(Value::Object(obj))
+        .map_err(|_| EngramError::InvalidInput("Invalid Markdown update fields".into()))
@@ -32 +156 @@ pub(super) fn create_memory_from_import(
-pub(super) fn update_memory_from_import(
+fn plan(
@@ -34,15 +158,114 @@ pub(super) fn update_memory_from_import(
-    engram_id: i64,
-    frontmatter: &HashMap<String, String>,
-    body: &str,
-    filename: &str,
-) -> Result<(), crate::error::EngramError> {
-    let mut obj = import_payload(frontmatter, body, false);
-    let tags = frontmatter_tags(frontmatter);
-    if !tags.is_empty() {
-        obj.insert("tags".into(), json!(tags));
-    } else if frontmatter.contains_key("engram_tags_list") {
-        eprintln!(
-            "[markdown_export] import: wiping all tags for memory {} (engram_tags_list is empty in file {})",
-            engram_id, filename
-        );
-        obj.insert("tags".into(), json!([]));
+    conn: &Connection,
+    input: &Input,
+    workspace: Option<&str>,
+    force: bool,
+) -> Result<Plan> {
+    let state: Option<(String, i64)> = if let Some(id) = input.id {
+        // The file's claimed workspace never authorizes an existing ID.
+        // Same connection/snapshot as classification and mutation.
+        ensure_memory_access(conn, ctx.principal.as_ref(), id)?;
+        conn.query_row(
+            "SELECT content_hash, version FROM memories WHERE id = ?1",
+            [id],
+            |row| Ok((row.get(0)?, row.get(1)?)),
+        )
+        .optional()?
+    } else {
+        None
+    };
+    let (hash, baseline) = match input.fm.get("engram_content_hash") {
+        Some(baseline) => (compute_content_hash_raw(&input.body), baseline.clone()),
+        None => (
+            compute_content_hash(&input.body),
+            state
+                .as_ref()
+                .map(|(hash, _)| hash.clone())
+                .unwrap_or_default(),
+        ),
+    };
+    let status = classify_import_status(
+        state
+            .as_ref()
+            .map(|(hash, version)| (hash.as_str(), *version)),
+        &hash,
+        &baseline,
+        input.version,
+        force,
+    );
+    let mutation = match status {
+        ImportStatus::New => Mutation::Create(creation(ctx, input, workspace)?),
+        ImportStatus::PendingUpdate => Mutation::Update(
+            input
+                .id
+                .ok_or_else(|| EngramError::Internal("Missing import ID".into()))?,
+            update(input)?,
+        ),
+        _ => Mutation::None,
+    };
+    Ok(Plan {
+        file: input.file.clone(),
+        id: input.id,
+        status,
+        mutation,
+    })
+}
+
+fn run(
+    ctx: &HandlerContext,
+    conn: &Connection,
+    inputs: &[Input],
+    workspace: Option<&str>,
+    force: bool,
+    confirm: bool,
+) -> Result<Value> {
+    // Authorize and classify the entire batch before the first write. With
+    // confirm, BEGIN IMMEDIATE excludes concurrent writers through application.
+    let plans = inputs
+        .iter()
+        .map(|input| plan(ctx, conn, input, workspace, force))
+        .collect::<Result<Vec<_>>>()?;
+    let (mut synced, mut new, mut pending, mut conflicts, mut applied) = (0, 0, 0, 0, 0);
+    let mut files = Vec::new();
+    for plan in plans {
+        let mut id = plan.id;
+        let mut changed = false;
+        if confirm {
+            match plan.mutation {
+                Mutation::Create(input) => {
+                    id = Some(crate::storage::queries::create_memory(conn, &input)?.id);
+                    changed = true;
+                }
+                Mutation::Update(id, input) => {
+                    crate::storage::queries::update_memory(conn, id, &input)?;
+                    changed = true;
+                }
+                Mutation::None => (),
+            }
+        }
+        applied += usize::from(changed);
+        let (status, reason) = match plan.status {
+            ImportStatus::New => {
+                new += 1;
+                ("new", None)
+            }
+            ImportStatus::InSync => {
+                synced += 1;
+                ("in_sync", None)
+            }
+            ImportStatus::PendingUpdate => {
+                pending += 1;
+                ("pending_update", None)
+            }
+            ImportStatus::Conflict(reason) => {
+                conflicts += 1;
+                ("conflict", Some(reason))
+            }
+        };
+        let mut detail = json!({"file":plan.file,"engram_id":id,"status":status});
+        if confirm && changed {
+            detail["applied"] = json!(true);
+        }
+        if let Some(reason) = reason {
+            detail["reason"] = json!(reason);
+        }
+        files.push(detail);
@@ -49,0 +273,4 @@ pub(super) fn update_memory_from_import(
+    Ok(
+        json!({"scanned":inputs.len(),"in_sync":synced,"new":new,"pending":pending,"pending_updates":pending,"conflict":conflicts,"conflicts":conflicts,"applied":applied,"files":files}),
+    )
+}
@@ -51,6 +278,25 @@ pub(super) fn update_memory_from_import(
-    serde_json::from_value::<crate::types::UpdateMemoryInput>(Value::Object(obj))
-        .map_err(|e| crate::error::EngramError::Internal(format!("bad import payload: {e}")))
-        .and_then(|input| {
-            ctx.storage.with_transaction(|conn| {
-                crate::storage::queries::update_memory(conn, engram_id, &input)
-            })
+pub(super) fn import_batch(
+    ctx: &HandlerContext,
+    sources: Vec<MarkdownSource>,
+    workspace: Option<&str>,
+    force: bool,
+    confirm: bool,
+) -> std::result::Result<Value, ImportFailure> {
+    let inputs = prepare(sources)?; // Parsing and filesystem IO happen before DB transactions.
+    let mut attempted = None;
+    let result = if confirm {
+        ctx.storage.with_transaction(|conn| {
+            let report = run(ctx, conn, &inputs, workspace, force, true)?;
+            attempted = Some(
+                report["applied"]
+                    .as_u64()
+                    .ok_or_else(|| EngramError::Internal("Missing import action count".into()))?,
+            );
+            Ok(report)
+        })
+    } else {
+        ctx.storage.with_connection(|conn| {
+            let tx = conn.unchecked_transaction()?;
+            let result = run(ctx, &tx, &inputs, workspace, force, false)?;
+            tx.commit()?;
+            Ok(result)
@@ -58 +304,2 @@ pub(super) fn update_memory_from_import(
-        .map(|_| ())
+    };
+    result.map_err(|error| ImportFailure { error, attempted })
# staged (index) content for src/mcp/handlers/markdown_export/import/integration_tests.rs; the working-tree diff above may differ
diff --git a/src/mcp/handlers/markdown_export/import/integration_tests.rs b/src/mcp/handlers/markdown_export/import/integration_tests.rs
index eb90239c7650530c026af4fd136330ab7336dcc1..703f133480645e35bcbccc0bed3b84e46fb7be2f 100644
--- a/src/mcp/handlers/markdown_export/import/integration_tests.rs
+++ b/src/mcp/handlers/markdown_export/import/integration_tests.rs
@@ -4,0 +5,143 @@ use serde_json::json;
+#[cfg(unix)]
+#[test]
+fn import_post_commit_hardening_error_does_not_report_zero_applied() {
+    let fixture = tempfile::tempdir().unwrap();
+    let physical = fixture.path().join("physical");
+    let replacement = fixture.path().join("replacement");
+    std::fs::create_dir(&physical).unwrap();
+    std::fs::create_dir(&replacement).unwrap();
+    let alias = fixture.path().join("configured-parent");
+    std::os::unix::fs::symlink(&physical, &alias).unwrap();
+    let db_path = alias.join("memory.db");
+    let mut c = ctx();
+    c.storage = crate::storage::Storage::open(crate::types::StorageConfig {
+        db_path: db_path.to_string_lossy().into_owned(),
+        storage_mode: crate::types::StorageMode::Local,
+        cloud_uri: None,
+        encrypt_cloud: false,
+        confidence_half_life_days: 30.0,
+        auto_sync: false,
+        sync_debounce_ms: 5000,
+    })
+    .unwrap();
+    let id = make_memory(&c, "original before post-commit error", &[]);
+    let input = fixture.path().join("input");
+    std::fs::create_dir(&input).unwrap();
+    write_md(
+        &input,
+        "edit.md",
+        Some(id),
+        1,
+        &[],
+        0.5,
+        "committed despite hardening error",
+        &[],
+    );
+    let sentinel = fixture.path().join("sentinel");
+    std::fs::write(&sentinel, "private fixture").unwrap();
+    // SQLite opened the canonical physical parent. Fault-inject a replacement
+    // of Storage's configured parent alias: its post-commit hardening fails,
+    // while the live DB/WAL/SHM themselves are never opened, moved or replaced.
+    std::os::unix::fs::symlink(&sentinel, replacement.join("memory.db")).unwrap();
+    std::fs::remove_file(&alias).unwrap();
+    std::os::unix::fs::symlink(&replacement, &alias).unwrap();
+    let result = memory_import_markdown(&c, json!({"input_dir":input,"confirm":true}));
+    std::fs::remove_file(&alias).unwrap();
+    std::os::unix::fs::symlink(&physical, &alias).unwrap();
+    assert!(
+        result.get("error").is_some(),
+        "hardening failure must remain an error"
+    );
+    assert_eq!(db_row(&c, id).0, "committed despite hardening error");
+    assert!(c
+        .storage
+        .with_connection(|conn| Ok(conn.is_autocommit()))
+        .unwrap());
+    assert!(
+        result["applied"].is_null(),
+        "post-commit error cannot claim applied=0: {result}"
+    );
+    assert_eq!(result["outcome"], "unknown");
+    assert_eq!(result["attempted"], 1);
+    assert_eq!(
+        std::fs::read_to_string(sentinel).unwrap(),
+        "private fixture"
+    );
+}
+
+#[test]
+fn import_tab_separated_invalid_identity_and_version_are_not_missing() {
+    for confirm in [true, false] {
+        for key in [
+            "engram_id",
+            "engram_version",
+            "'engram_id'",
+            "\"engram_version\"",
+        ] {
+            let c = ctx();
+            let fixture = tempfile::tempdir().unwrap();
+            std::fs::write(
+                fixture.path().join("invalid.md"),
+                format!("---\n{key}:\t0\n---\nmust not become a note\n"),
+            )
+            .unwrap();
+            let result =
+                memory_import_markdown(&c, json!({"input_dir":fixture.path(),"confirm":confirm}));
+            assert!(
+                result.get("error").is_some(),
+                "{key}, confirm={confirm}: {result}"
+            );
+            assert_eq!(result["applied"], 0);
+            assert_eq!(
+                c.storage
+                    .with_connection(|conn| Ok(conn.query_row(
+                        "SELECT COUNT(*) FROM memories",
+                        [],
+                        |row| row.get::<_, i64>(0)
+                    )?))
+                    .unwrap(),
+                0
+            );
+        }
+    }
+}
+
+#[test]
+fn import_database_error_rolls_back_earlier_updates() {
+    let c = ctx();
+    let first = make_memory(&c, "first original", &["first"]);
+    let second = make_memory(&c, "second original", &["second"]);
+    let before = (db_row(&c, first), db_row(&c, second));
+    let dir = tempfile::tempdir().unwrap();
+    write_md(
+        dir.path(),
+        "00-first.md",
+        Some(first),
+        1,
+        &["changed"],
+        0.9,
+        "first edit",
+        &[],
+    );
+    write_md(
+        dir.path(),
+        "99-second.md",
+        Some(second),
+        1,
+        &["changed"],
+        0.9,
+        "second edit",
+        &[],
+    );
+    c.storage.with_connection(|conn| {
+        conn.execute_batch(&format!("CREATE TRIGGER reject_second_import BEFORE UPDATE OF content ON memories WHEN NEW.id = {second} BEGIN SELECT RAISE(ABORT, 'injected import failure'); END;"))?;
+        Ok(())
+    }).unwrap();
+    let result = memory_import_markdown(&c, json!({"input_dir":dir.path(),"confirm":true}));
+    assert!(result.get("error").is_some(), "{result}");
+    assert_eq!(result["applied"], 0);
+    assert_eq!((db_row(&c, first), db_row(&c, second)), before);
+    assert_eq!(db_tags(&c, first), vec!["first"]);
+    assert_eq!(db_tags(&c, second), vec!["second"]);
+}
+
# staged (index) content for src/mcp/tools/catalog/admin.rs; the working-tree diff above may differ
diff --git a/src/mcp/tools/catalog/admin.rs b/src/mcp/tools/catalog/admin.rs
index a148852b7a9a21647bd77dffd999e5607c14b312..e5ab164f5eb95fc505e8edb459a96b71b6a1746d 100644
--- a/src/mcp/tools/catalog/admin.rs
+++ b/src/mcp/tools/catalog/admin.rs
@@ -340 +340 @@ pub const TOOLS: &[ToolDef] = &[
-            description: "Export a workspace as human-readable Markdown files with YAML frontmatter and wiki-style [[links]]. Creates one .md file per memory, organized by type in subdirectories, with an index.md overview.",
+            description: "Write unexpired workspace memories as Markdown files with YAML frontmatter and directional [[links]]. Requires scoped_write within ENGRAM_EXPORT_BASE_DIR, otherwise maintenance. Output directories must be owned by the server user, not group/world writable and have no extended/default ACLs. Safe filesystem writes require Linux/macOS.",
@@ -351 +351 @@ pub const TOOLS: &[ToolDef] = &[
-            annotations: ToolAnnotations::read_only(),
+            annotations: ToolAnnotations::mutating(),
@@ -356 +356 @@ pub const TOOLS: &[ToolDef] = &[
-            description: "Import memories from Markdown files with engram_ frontmatter (RFC 0004). Review mode by default (confirm: false) — returns a staged list without writing. Detects drift via content_hash and version conflicts via engram_version. Ignores non-engram_ frontmatter keys (Obsidian-safe).",
+            description: "Import bounded Markdown files with engram_ frontmatter on Linux/macOS. Requires scoped_write within ENGRAM_EXPORT_BASE_DIR, otherwise maintenance, including review mode. Input directories must be server-owned, not group/world writable and have no ACLs; links and SQLite artifacts are refused. Authorizes existing IDs by their persisted workspace. Review is default; confirm uses one database transaction. Pre-commit failures roll back; commit/completion failures report an unknown outcome requiring reconciliation. Detects drift/version conflicts and ignores non-engram_ keys (Obsidian-safe).",
diff --git a/docs/harness/canvas/2026-11-01-markdown-release-integration.md b/docs/harness/canvas/2026-11-01-markdown-release-integration.md
new file mode 100644
index 0000000000000000000000000000000000000000..2d5f9b77dd592c69e5f4d988b7dfb51142fa4d63
--- /dev/null
+++ b/docs/harness/canvas/2026-11-01-markdown-release-integration.md
@@ -0,0 +1,30 @@
+# Review Canvas: markdown-release-integration
+
+## Approaches Considered
+
+| Opção | Decisão | Razão |
+|---|---|---|
+| Integrar só export/import sobre main | Recusada | Perde remediação79889460 e outras correções |
+| Restaurar handler antigo | Recusada | Reabre autorização/IO/atomicidade |
+| Remediação + branch import + contratos no engine novo | Adotada | Preserva ambos históricos e resultado seguro |
+
+## Hot Path Complexity
+
+Sem alteração de busca/embeddings/schema. Reader mantém limites de entradas visitadas e ordenação;
+pasta oculta conta como uma entrada mas seus descendentes não são lidos. Nenhum IO de input em TX.
+
+## Edge Cases
+
+Import omitindo confirm; flags conflitantes; force_version sem confirm; criação e update; MCP wire
+dry_run; SDKs Python/TS; .trash/.git/.obsidian com symlinks; links visíveis continuam recusados;
+NotFound restrito vs owner local; outcome unknown; conflito de catálogo e notas de release.
+
+## Breakage Risk
+
+| Risco | Efeito | Verificação |
+|---|---|---|
+| Merge perde defaults | CLI/SDK escrevem sem pedido | Binário real e testes SDK existentes |
+| Merge perde dry_run wire | Cliente recebe contrato incompleto | Assert direto handler/dispatch |
+| Pastas ocultas entram na captura | Lixeira vira memória/quotas falham | Testes reader com limites pequenos e symlink oculto |
+| Conteúdo seguro perdido | IDOR ou sucesso zero | Suíte Markdown/G1 e review do delta |
+| Gate de outra árvore | Falsa prontidão de release | Full novo, manifest e commit conferidos |
# untracked directory (nested repository?) not expanded: sdks/typescript/node_modules
diff --git a/tests/cli_markdown_import_tests.rs b/tests/cli_markdown_import_tests.rs
new file mode 100644
index 0000000000000000000000000000000000000000..2cf1122bb65bcd475e5929da5e23abf7987b9041
--- /dev/null
+++ b/tests/cli_markdown_import_tests.rs
@@ -0,0 +1,288 @@
+//! Exercise the real CLI import path against private synthetic fixtures.
+//! These checks compare logical SQLite state, not physical database bytes:
+//! opening SQLite may checkpoint files without applying a Markdown import.
+#![cfg(any(target_os = "linux", target_os = "macos"))]
+
+use std::fs;
+use std::os::unix::fs::PermissionsExt;
+use std::path::{Path, PathBuf};
+use std::process::{Command, Output, Stdio};
+use std::time::{Duration, Instant};
+
+use engram::storage::queries::create_memory;
+use engram::storage::Storage;
+use engram::types::{CreateMemoryInput, StorageConfig, StorageMode};
+
+const WORKSPACE: &str = "cli_markdown_import_fixture";
+const ORIGINAL: &str = "Original synthetic note, unchanged until confirmation.";
+const EDITED: &str = "# Edited synthetic note\n\nUpdated only with confirmation.";
+const NEW: &str = "# New synthetic note\n\nInserted only with confirmation.";
+
+#[derive(Debug, PartialEq)]
+struct MemoryRow {
+    id: i64,
+    content: String,
+    version: i64,
+    workspace: String,
+    importance: Option<f64>,
+    updated_at: Option<String>,
+}
+
+#[derive(Debug, PartialEq)]
+struct DatabaseState {
+    memories: Vec<MemoryRow>,
+    tags: Vec<(i64, String)>,
+}
+
+struct Fixture {
+    directory: tempfile::TempDir,
+    vault: PathBuf,
+    temporary: PathBuf,
+    database: PathBuf,
+    existing_id: i64,
+}
+
+fn private_directory(path: &Path) {
+    fs::create_dir(path).expect("create private fixture directory");
+    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
+        .expect("restrict fixture directory permissions");
+}
+
+fn open_storage(database: &Path) -> Storage {
+    Storage::open(StorageConfig {
+        db_path: database.to_str().expect("UTF-8 fixture path").to_string(),
+        storage_mode: StorageMode::Local,
+        cloud_uri: None,
+        encrypt_cloud: false,
+        confidence_half_life_days: 30.0,
+        auto_sync: false,
+        sync_debounce_ms: 5000,
+    })
+    .expect("open synthetic fixture through official Storage")
+}
+
+fn write_markdown(path: &Path, id: Option<i64>, tag: &str, body: &str) {
+    let identity = id.map_or_else(String::new, |id| format!("engram_id: {id}\n"));
+    fs::write(
+        path,
+        format!(
+            "---\n{identity}engram_workspace: {WORKSPACE}\nengram_type: note\n\
+             engram_version: 1\nengram_importance: 0.75\nengram_tags:\n  - {tag}\n\
+             ---\n{body}\n"
+        ),
+    )
+    .expect("write synthetic Markdown input");
+    fs::set_permissions(path, fs::Permissions::from_mode(0o600))
+        .expect("restrict Markdown fixture permissions");
+}
+
+impl Fixture {
+    fn new() -> Self {
+        let directory = tempfile::tempdir().expect("private CLI fixture");
+        let vault = directory.path().join("vault");
+        let temporary = directory.path().join("temporary");
+        private_directory(&vault);
+        private_directory(&temporary);
+        let database = directory.path().join("synthetic.db");
+        let existing_id = {
+            let storage = open_storage(&database);
+            storage
+                .with_transaction(|conn| {
+                    Ok(create_memory(
+                        conn,
+                        &CreateMemoryInput {
+                            content: ORIGINAL.into(),
+                            workspace: Some(WORKSPACE.into()),
+                            tags: vec!["original-tag".into()],
+                            importance: Some(0.5),
+                            ..Default::default()
+                        },
+                    )?
+                    .id)
+                })
+                .expect("seed one existing memory")
+        }; // Close Storage before any child process opens the same database.
+        write_markdown(
+            &vault.join("00-edit.md"),
+            Some(existing_id),
+            "edited-tag",
+            EDITED,
+        );
+        write_markdown(&vault.join("01-new.md"), None, "new-tag", NEW);
+        Self {
+            directory,
+            vault,
+            temporary,
+            database,
+            existing_id,
+        }
+    }
+
+    fn state(&self) -> DatabaseState {
+        let storage = open_storage(&self.database);
+        storage
+            .with_connection(|conn| {
+                let mut statement = conn.prepare(
+                    "SELECT id,content,version,workspace,importance,updated_at \
+                     FROM memories ORDER BY id",
+                )?;
+                let memories = statement
+                    .query_map([], |row| {
+                        Ok(MemoryRow {
+                            id: row.get(0)?,
+                            content: row.get(1)?,
+                            version: row.get(2)?,
+                            workspace: row.get(3)?,
+                            importance: row.get(4)?,
+                            updated_at: row.get(5)?,
+                        })
+                    })?
+                    .collect::<rusqlite::Result<Vec<_>>>()?;
+                let mut statement = conn.prepare(
+                    "SELECT mt.memory_id,t.name FROM memory_tags mt \
+                     JOIN tags t ON t.id=mt.tag_id ORDER BY mt.memory_id,t.name",
+                )?;
+                let tags = statement
+                    .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
+                    .collect::<rusqlite::Result<Vec<_>>>()?;
+                Ok(DatabaseState { memories, tags })
+            })
+            .expect("snapshot logical fixture state through official Storage")
+    }
+
+    fn import(&self, database: &Path, flags: &[&str]) -> Output {
+        let mut child = Command::new(env!("CARGO_BIN_EXE_engram-cli"))
+            .env_clear()
+            .env("PATH", "/usr/bin:/bin")
+            .env("TMPDIR", &self.temporary)
+            .env("ENGRAM_DB_PATH", database)
+            .env("ENGRAM_EXPORT_BASE_DIR", &self.vault)
+            .env("ENGRAM_PERMISSION_MODE", "scoped_write")
+            .current_dir(self.directory.path())
+            .args(["import", "markdown", "--path"])
+            .arg(&self.vault)
+            .args(["--workspace", WORKSPACE])
+            .args(flags)
+            .stdin(Stdio::null())
+            .stdout(Stdio::piped())
+            .stderr(Stdio::piped())
+            .spawn()
+            .expect("start the real engram-cli test binary");
+        let deadline = Instant::now() + Duration::from_secs(30);
+        while child.try_wait().expect("poll CLI child").is_none() {
+            if Instant::now() >= deadline {
+                let _ = child.kill();
+                let output = child.wait_with_output().expect("reap timed-out CLI child");
+                panic!(
+                    "CLI import exceeded 30 seconds with flags {flags:?}:\n{}\n{}",
+                    String::from_utf8_lossy(&output.stdout),
+                    String::from_utf8_lossy(&output.stderr)
+                );
+            }
+            std::thread::sleep(Duration::from_millis(10));
+        }
+        child
+            .wait_with_output()
+            .expect("collect and reap CLI child")
+    }
+}
+
+fn assert_success(output: &Output) {
+    assert!(
+        output.status.success(),
+        "CLI failed: status={}\nstdout={}\nstderr={}",
+        output.status,
+        String::from_utf8_lossy(&output.stdout),
+        String::from_utf8_lossy(&output.stderr)
+    );
+}
+
+#[test]
+fn cli_markdown_import_without_confirm_never_applies_creates_or_updates() {
+    let fixture = Fixture::new();
+    let before = fixture.state();
+    assert_eq!(before.memories.len(), 1);
+    assert_eq!(before.memories[0].content, ORIGINAL);
+    for flags in [
+        &[][..],
+        &["--dry-run"][..],
+        &["--force-version"][..],
+        &["--dry-run", "--force-version"][..],
+    ] {
+        let output = fixture.import(&fixture.database, flags);
+        assert_success(&output); // A denied import is not evidence of safe review.
+        let stdout = String::from_utf8_lossy(&output.stdout);
+        assert!(stdout.contains("2 files"), "{stdout}");
+        assert!(stdout.contains("0 applied"), "{stdout}");
+        assert!(stdout.contains("dry_run: true"), "{stdout}");
+        assert!(
+            stdout.contains("--confirm"),
+            "review should explain how to apply: {stdout}"
+        );
+        assert_eq!(
+            fixture.state(),
+            before,
+            "review mutated rows, versions or tags with {flags:?}"
+        );
+    }
+}
+
+#[test]
+fn cli_markdown_import_confirm_applies_both_the_update_and_new_note() {
+    let fixture = Fixture::new();
+    let before = fixture.state();
+    let output = fixture.import(&fixture.database, &["--confirm"]);
+    assert_success(&output);
+    let stdout = String::from_utf8_lossy(&output.stdout);
+    assert!(stdout.contains("2 applied"), "{stdout}");
+    assert!(stdout.contains("dry_run: false"), "{stdout}");
+
+    let after = fixture.state();
+    assert_eq!(before.memories.len(), 1);
+    assert_eq!(after.memories.len(), 2);
+    assert!(after.memories.iter().all(|row| row.workspace == WORKSPACE));
+    let edited = after
+        .memories
+        .iter()
+        .find(|row| row.id == fixture.existing_id)
+        .unwrap();
+    assert_eq!(edited.content, EDITED);
+    assert_eq!(edited.version, before.memories[0].version + 1);
+    assert_eq!(edited.importance, Some(0.75));
+    assert!(after.tags.contains(&(edited.id, "edited-tag".into())));
+    assert!(!after.tags.contains(&(edited.id, "original-tag".into())));
+    let created = after
+        .memories
+        .iter()
+        .find(|row| row.id != fixture.existing_id)
+        .unwrap();
+    assert_eq!(created.content, NEW);
+    assert_eq!(created.version, 1);
+    assert!(after.tags.contains(&(created.id, "new-tag".into())));
+}
+
+#[test]
+fn cli_markdown_import_conflicting_flags_fail_before_creating_a_database() {
+    let fixture = Fixture::new();
+    let before = fixture.state();
+    for (index, flags) in [["--confirm", "--dry-run"], ["--dry-run", "--confirm"]]
+        .iter()
+        .enumerate()
+    {
+        let absent_database = fixture
+            .directory
+            .path()
+            .join(format!("rejected-{index}.db"));
+        assert!(!absent_database.exists());
+        let output = fixture.import(&absent_database, flags);
+        assert_eq!(output.status.code(), Some(2), "{output:?}");
+        let stderr = String::from_utf8_lossy(&output.stderr);
+        assert!(stderr.contains("--confirm"), "{stderr}");
+        assert!(stderr.contains("--dry-run"), "{stderr}");
+        assert!(
+            !absent_database.exists(),
+            "argument rejection opened a new database"
+        );
+        assert_eq!(fixture.state(), before);
+    }
+}
```

## Previous Review Context (if any)

(no previous review supplied for continuity)

## Output Contract (strict)

Your entire response must start with exactly one of:

PASS <one-line summary of what was reviewed and why it is safe>

or

FAIL <one-line summary of the most important problem(s)>

Then a short bullet list using [BLOCKER], [HIGH], [MED], [LOW].
At most 3 substantive findings. Evidence and location required for each.
If nothing substantive: exactly one bullet with [LOW] No issues found...

Remember: you are the external reviewer. Be evidence-driven and skeptical.

Machine-parseable verdict (required):
Add exactly one line, anywhere in the response, beginning with:
REVIEW_VERDICT: PASS <one-line summary>
or
REVIEW_VERDICT: FAIL <one-line summary>
This line is required for hard post-gate enforcement. A response with zero or
more than one such line is rejected.
