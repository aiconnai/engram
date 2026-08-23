# Engram Harness — External Reviewer Prompt

**Task**: markdown-obsidian-vault-adapter
**Mode**: pre
**Date (UTC)**: 2026-08-23

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

## Diff Under Review

```diff
diff --git a/docs/harness/.sensors-last b/docs/harness/.sensors-last
index 698c34a..58f7914 100644
--- a/docs/harness/.sensors-last
+++ b/docs/harness/.sensors-last
@@ -4,4 +4,4 @@ doctor_status=pass
-mode=quick
-timestamp=2026-08-22T13:58:53Z
-duration_sec=5
-ci=cargo fmt + cargo check + pr-title-policy + harness doctor
+mode=full
+timestamp=2026-08-22T14:09:42Z
+duration_sec=138
+ci=fmt + clippy + test_lib + test_integration + test_integration_watch + wasm_target + wasm_all_targets + wasm_wasm_target + doc + ref_check + pr-title-policy + harness doctor
diff --git a/docs/harness/.sensors-log b/docs/harness/.sensors-log
index 5701905..84580a3 100644
--- a/docs/harness/.sensors-log
+++ b/docs/harness/.sensors-log
@@ -85,0 +86 @@
+{"schema_version":"sensors-log-v1","timestamp":"2026-08-22T14:09:42Z","tool":"sensors","mode":"full","status":"pass","duration_sec":138,"ci_status":"pass","doctor_status":"pass","ci_command":"fmt + clippy + test_lib + test_integration + test_integration_watch + wasm_target + wasm_all_targets + wasm_wasm_target + doc + ref_check + pr-title-policy + harness doctor","ci_steps":{"fmt":"pass","clippy":"pass","test_lib":"pass","test_integration":"pass","test_integration_watch":"pass","wasm_target":"pass","wasm_all_targets":"pass","wasm_wasm_target":"pass","doc":"pass","ref_check":"pass"},"exclusion":null,"artifacts":[{"path":"docs/harness/.sensors-last","kind":"sensors_last","format":"key_value"}]}
diff --git a/sdks/python/engram_client/client.py b/sdks/python/engram_client/client.py
index fe109ba..fd5c64f 100644
--- a/sdks/python/engram_client/client.py
+++ b/sdks/python/engram_client/client.py
@@ -21,0 +22 @@ from .resources.spatial import SpatialMixin
+from .resources.vault import VaultMixin
@@ -37,0 +39 @@ class EngramClient(
+    VaultMixin,
diff --git a/sdks/python/engram_client/resources/__init__.py b/sdks/python/engram_client/resources/__init__.py
index a679e17..15bd58a 100644
--- a/sdks/python/engram_client/resources/__init__.py
+++ b/sdks/python/engram_client/resources/__init__.py
@@ -13,0 +14 @@ from .spatial import SpatialMixin
+from .vault import VaultMixin
@@ -26,0 +28 @@ __all__ = [
+    "VaultMixin",
diff --git a/sdks/python/tests/test_resources.py b/sdks/python/tests/test_resources.py
index 83ae077..8dbe06c 100644
--- a/sdks/python/tests/test_resources.py
+++ b/sdks/python/tests/test_resources.py
@@ -17,0 +18 @@ from engram_client.resources import (
+    VaultMixin,
@@ -25,0 +27 @@ from engram_client.resources.spatial import SpatialMixin as DirectSpatialMixin
+from engram_client.resources.vault import VaultMixin as DirectVaultMixin
@@ -472,0 +475,57 @@ async def test_spatial_mixin(mock_client):
+@pytest.mark.asyncio
+async def test_vault_mixin(mock_client):
+    """Test VaultMixin Markdown & Obsidian export/import operations."""
+    assert VaultMixin is DirectVaultMixin
+    mock_client._mcp_call = AsyncMock(
+        return_value={"files_written": 10, "output_dir": "./vault", "workspace": "default"}
+    )
+
+    await mock_client.vault_export(
+        output_dir="./vault",
+        workspace="default",
+        group="workspace",
+        include_links=True,
+    )
+    mock_client._mcp_call.assert_awaited_with(
+        "memory_export_markdown",
+        {
+            "output_dir": "./vault",
+            "workspace": "default",
+            "group": "workspace",
+            "include_links": True,
+        },
+    )
+
+    mock_client._mcp_call = AsyncMock(
+        return_value={"scanned": 5, "in_sync": 3, "new": 1, "pending": 1, "conflict": 0, "applied": 2}
+    )
+    await mock_client.vault_import(
+        input_dir="./vault",
+        workspace="default",
+        confirm=True,
+    )
+    mock_client._mcp_call.assert_awaited_with(
+        "memory_import_markdown",
+        {
+            "input_dir": "./vault",
+            "workspace": "default",
+            "confirm": True,
+            "force_version": False,
+        },
+    )
+
+    await mock_client.vault_preview(
+        input_dir="./vault",
+        workspace="default",
+    )
+    mock_client._mcp_call.assert_awaited_with(
+        "memory_import_markdown",
+        {
+            "input_dir": "./vault",
+            "workspace": "default",
+            "confirm": False,
+            "force_version": False,
+        },
+    )
+
+
diff --git a/sdks/typescript/src/client.ts b/sdks/typescript/src/client.ts
index 8701d82..973b0ce 100644
--- a/sdks/typescript/src/client.ts
+++ b/sdks/typescript/src/client.ts
@@ -13,0 +14 @@ import {
+  VaultResource,
@@ -90,0 +92,4 @@ import type {
+  VaultExportOptions,
+  VaultExportReport,
+  VaultImportOptions,
+  VaultImportReport,
@@ -109,0 +115 @@ export class EngramClient implements McpCaller {
+  public readonly vault: VaultResource;
@@ -131,0 +138 @@ export class EngramClient implements McpCaller {
+    this.vault = new VaultResource(this);
@@ -869,0 +877,23 @@ export class EngramClient implements McpCaller {
+
+  /**
+   * Export memories to Markdown files (Obsidian vault compatible).
+   */
+  vaultExport(options?: VaultExportOptions): Promise<VaultExportReport> {
+    return this.vault.export(options);
+  }
+
+  /**
+   * Import memories from Markdown files into Engram with drift detection.
+   */
+  vaultImport(options: VaultImportOptions): Promise<VaultImportReport> {
+    return this.vault.import(options);
+  }
+
+  /**
+   * Preview Markdown import without mutating the database (dry-run review mode).
+   */
+  vaultPreview(
+    options: Omit<VaultImportOptions, "confirm" | "dryRun">
+  ): Promise<VaultImportReport> {
+    return this.vault.preview(options);
+  }
@@ -871,0 +902 @@ export class EngramClient implements McpCaller {
+
diff --git a/sdks/typescript/src/index.test.ts b/sdks/typescript/src/index.test.ts
index 85a2887..c97e3d4 100644
--- a/sdks/typescript/src/index.test.ts
+++ b/sdks/typescript/src/index.test.ts
@@ -783,0 +784,71 @@ describe("EngramClient", () => {
+
+  describe("VaultResource", () => {
+    it("should export memories to vault markdown", async () => {
+      mockFetch.mockResolvedValueOnce(
+        okResponse({ files_written: 10, output_dir: "./vault", workspace: "default" })
+      );
+      const res = await client.vault.export({
+        outputDir: "./vault",
+        workspace: "default",
+        group: "workspace",
+        includeLinks: true,
+      });
+      expect(requestMethod(0)).toBe("memory_export_markdown");
+      expect(requestArguments(0)).toEqual({
+        workspace: "default",
+        output_dir: "./vault",
+        group: "workspace",
+        include_links: true,
+      });
+      expect(res.files_written).toBe(10);
+    });
+
+    it("should import memories from vault markdown", async () => {
+      mockFetch.mockResolvedValueOnce(
+        okResponse({
+          scanned: 5,
+          in_sync: 3,
+          new: 1,
+          pending: 1,
+          conflict: 0,
+          applied: 2,
+          dry_run: false,
+        })
+      );
+      const res = await client.vault.import({
+        inputDir: "./vault",
+        workspace: "default",
+        confirm: true,
+      });
+      expect(requestMethod(0)).toBe("memory_import_markdown");
+      expect(requestArguments(0)).toEqual({
+        input_dir: "./vault",
+        workspace: "default",
+        confirm: true,
+      });
+      expect(res.applied).toBe(2);
+    });
+
+    it("should preview vault import with dryRun", async () => {
+      mockFetch.mockResolvedValueOnce(
+        okResponse({
+          scanned: 5,
+          in_sync: 3,
+          new: 1,
+          pending: 1,
+          conflict: 0,
+          applied: 0,
+          dry_run: true,
+        })
+      );
+      const res = await client.vaultPreview({
+        inputDir: "./vault",
+      });
+      expect(requestMethod(0)).toBe("memory_import_markdown");
+      expect(requestArguments(0)).toEqual({
+        input_dir: "./vault",
+        confirm: false,
+      });
+      expect(res.dry_run).toBe(true);
+    });
+  });
diff --git a/sdks/typescript/src/index.ts b/sdks/typescript/src/index.ts
index a2ba8c1..931d51e 100644
--- a/sdks/typescript/src/index.ts
+++ b/sdks/typescript/src/index.ts
@@ -41,0 +42 @@ export {
+  VaultResource,
@@ -121,0 +123,6 @@ export type {
+  VaultExportOptions,
+  VaultExportReport,
+  VaultFileDetail,
+  VaultGrouping,
+  VaultImportOptions,
+  VaultImportReport,
diff --git a/sdks/typescript/src/resources/index.ts b/sdks/typescript/src/resources/index.ts
index da5546e..a653f08 100644
--- a/sdks/typescript/src/resources/index.ts
+++ b/sdks/typescript/src/resources/index.ts
@@ -20,0 +21 @@ export { SpatialResource } from "./spatial.js";
+export { VaultResource } from "./vault.js";
diff --git a/sdks/typescript/src/types.ts b/sdks/typescript/src/types.ts
index efbafc0..9f45ac0 100644
--- a/sdks/typescript/src/types.ts
+++ b/sdks/typescript/src/types.ts
@@ -561,0 +562 @@ export interface ClusterOptions {
+export type VaultGrouping = 'flat' | 'day' | 'workspace' | 'type' | 'entity';
@@ -562,0 +564,41 @@ export interface ClusterOptions {
+export interface VaultExportOptions {
+  outputDir?: string;
+  workspace?: string;
+  group?: VaultGrouping;
+  includeLinks?: boolean;
+}
+
+export interface VaultExportReport {
+  files_written: number;
+  output_dir: string;
+  workspace: string;
+  error?: string;
+}
+
+export interface VaultImportOptions {
+  inputDir: string;
+  workspace?: string;
+  confirm?: boolean;
+  dryRun?: boolean;
+  forceVersion?: boolean;
+}
+
+export interface VaultFileDetail {
+  file: string;
+  engram_id?: number | null;
+  status: string;
+  applied?: boolean;
+  reason?: string;
+}
+
+export interface VaultImportReport {
+  scanned: number;
+  in_sync: number;
+  new: number;
+  pending: number;
+  conflict: number;
+  applied: number;
+  dry_run: boolean;
+  files?: VaultFileDetail[];
+  error?: string;
+}
diff --git a/src/attestation/chain.rs b/src/attestation/chain.rs
index 524ba9f..9fd9612 100644
--- a/src/attestation/chain.rs
+++ b/src/attestation/chain.rs
@@ -28,0 +29,6 @@ const GENESIS_HASH: &str = "genesis";
+/// Sentinel for hash scheme version 1 (legacy pipe-delimited format).
+pub const LEGACY_HASH_VERSION: i32 = 1;
+
+/// Current hash scheme version for newly created attestation records (v2 length-prefixed canonical format).
+pub const CURRENT_HASH_VERSION: i32 = 2;
+
@@ -157,0 +164 @@ impl AttestationChain {
+                hash_version: CURRENT_HASH_VERSION,
@@ -160 +167 @@ impl AttestationChain {
-            record.record_hash = Self::compute_record_hash(&record);
+            record.record_hash = Self::compute_record_hash_v2(&record);
@@ -193,2 +200,2 @@ impl AttestationChain {
-                     agent_id, memory_ids, previous_hash, record_hash, signature, metadata)
-                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
+                     agent_id, memory_ids, previous_hash, record_hash, signature, metadata, hash_version)
+                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
@@ -205,0 +213 @@ impl AttestationChain {
+                    record.hash_version,
@@ -236 +244 @@ impl AttestationChain {
-                        metadata, created_at
+                        metadata, created_at, hash_version
@@ -254 +262 @@ impl AttestationChain {
-    /// - Each `record_hash` is correctly computed from the record's fields
+    /// - Each `record_hash` is correctly computed from the record's fields (supporting both v1 and v2 schemes)
@@ -265 +273 @@ impl AttestationChain {
-                        metadata, created_at
+                        metadata, created_at, hash_version
@@ -301,2 +309,18 @@ impl AttestationChain {
-                // 2. Recompute record_hash and compare (constant-time)
-                let recomputed = Self::compute_record_hash(&record);
+                // 2. Recompute record_hash and compare (constant-time).
+                // Supports v1 legacy delimiter format and v2 canonical format.
+                let recomputed = match record.hash_version {
+                    1 => {
+                        let v1_hash = Self::compute_record_hash_v1(&record);
+                        if bool::from(v1_hash.as_bytes().ct_eq(record.record_hash.as_bytes())) {
+                            v1_hash
+                        } else {
+                            let v1_meta = Self::compute_record_hash_v1_with_meta(&record);
+                            if bool::from(v1_meta.as_bytes().ct_eq(record.record_hash.as_bytes())) {
+                                v1_meta
+                            } else {
+                                v1_hash
+                            }
+                        }
+                    }
+                    _ => Self::compute_record_hash_v2(&record),
+                };
@@ -394 +418 @@ impl AttestationChain {
-                        metadata, created_at
+                        metadata, created_at, hash_version
@@ -423,4 +447 @@ impl AttestationChain {
-    /// Compute the canonical `record_hash` for a record.
-    ///
-    /// Hash = SHA-256 of:
-    /// `document_hash|document_name|document_size|ingested_at|agent_id|memory_ids|previous_hash|metadata`
+    /// Compute the record hash for a record according to its `hash_version`.
@@ -428,2 +449,50 @@ impl AttestationChain {
-        // v2: Length-prefixed canonical encoding prevents delimiter injection.
-        // Includes metadata (omitted in v1).
+        match record.hash_version {
+            1 => Self::compute_record_hash_v1(record),
+            _ => Self::compute_record_hash_v2(record),
+        }
+    }
+
+    /// Compute v1 legacy record hash (delimiter-separated).
+    ///
+    /// Hash = SHA-256 of `document_hash|document_name|document_size|ingested_at|agent_id|memory_ids|previous_hash`
+    pub fn compute_record_hash_v1(record: &AttestationRecord) -> String {
+        let canonical = format!(
+            "{}|{}|{}|{}|{}|{}|{}",
+            record.document_hash,
+            record.document_name,
+            record.document_size,
+            record.ingested_at.to_rfc3339(),
+            record.agent_id.as_deref().unwrap_or(""),
+            serde_json::to_string(&record.memory_ids).unwrap_or_default(),
+            record.previous_hash,
+        );
+        let mut hasher = Sha256::new();
+        hasher.update(canonical.as_bytes());
+        format!("sha256:{}", hex::encode(hasher.finalize()))
+    }
+
+    /// Compute intermediate v1 record hash with metadata appended.
+    pub fn compute_record_hash_v1_with_meta(record: &AttestationRecord) -> String {
+        let metadata_str = if record.metadata.is_null() {
+            String::new()
+        } else {
+            serde_json::to_string(&record.metadata).unwrap_or_default()
+        };
+        let canonical = format!(
+            "{}|{}|{}|{}|{}|{}|{}|{}",
+            record.document_hash,
+            record.document_name,
+            record.document_size,
+            record.ingested_at.to_rfc3339(),
+            record.agent_id.as_deref().unwrap_or(""),
+            serde_json::to_string(&record.memory_ids).unwrap_or_default(),
+            record.previous_hash,
+            metadata_str,
+        );
+        let mut hasher = Sha256::new();
+        hasher.update(canonical.as_bytes());
+        format!("sha256:{}", hex::encode(hasher.finalize()))
+    }
+
+    /// Compute v2 canonical record hash (length-prefixed fields with metadata).
+    pub fn compute_record_hash_v2(record: &AttestationRecord) -> String {
@@ -512,0 +582 @@ fn row_to_record(row: &rusqlite::Row<'_>) -> Result<AttestationRecord> {
+    let hash_version: i32 = row.get(12).unwrap_or(1);
@@ -547,0 +618 @@ fn row_to_record(row: &rusqlite::Row<'_>) -> Result<AttestationRecord> {
+        hash_version,
@@ -1101,0 +1173 @@ mod tests {
+            hash_version: CURRENT_HASH_VERSION,
@@ -1163,0 +1236,71 @@ mod tests {
+
+    #[test]
+    fn test_legacy_v1_record_verification_and_mixed_chain() {
+        let storage = Storage::open_in_memory().unwrap();
+        let chain = AttestationChain::new(storage.clone());
+
+        // Construct legacy v1 record manually and insert into attestation_log
+        let now = chrono::Utc::now();
+        let mut rec1 = AttestationRecord {
+            id: None,
+            document_hash: "sha256:abc123doc".to_string(),
+            document_name: "legacy_v1.txt".to_string(),
+            document_size: 128,
+            ingested_at: now,
+            agent_id: Some("agent-v1".to_string()),
+            memory_ids: vec![1, 2],
+            previous_hash: GENESIS_HASH.to_string(),
+            record_hash: String::new(),
+            signature: None,
+            metadata: serde_json::json!({}),
+            created_at: Some(now),
+            hash_version: LEGACY_HASH_VERSION,
+        };
+        rec1.record_hash = AttestationChain::compute_record_hash_v1(&rec1);
+
+        storage
+            .with_transaction(|conn| {
+                conn.execute(
+                    "INSERT INTO attestation_log
+                        (document_hash, document_name, document_size, ingested_at,
+                         agent_id, memory_ids, previous_hash, record_hash, signature, metadata, hash_version)
+                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
+                    rusqlite::params![
+                        rec1.document_hash,
+                        rec1.document_name,
+                        rec1.document_size as i64,
+                        rec1.ingested_at.to_rfc3339(),
+                        rec1.agent_id,
+                        serde_json::to_string(&rec1.memory_ids).unwrap(),
+                        rec1.previous_hash,
+                        rec1.record_hash,
+                        rec1.signature,
+                        serde_json::to_string(&rec1.metadata).unwrap(),
+                        rec1.hash_version,
+                    ],
+                )?;
+                Ok(())
+            })
+            .unwrap();
+
+        // Now append a v2 record using standard log_document
+        let r2 = chain
+            .log_document(
+                b"new v2 content",
+                "modern_v2.txt",
+                Some("agent-v2"),
+                &[3],
+                None,
+            )
+            .expect("append v2 record");
+        assert_eq!(r2.previous_hash, rec1.record_hash);
+        assert_eq!(r2.hash_version, CURRENT_HASH_VERSION);
+
+        // Verify the entire mixed chain (v1 followed by v2)
+        match chain.verify_chain(None).unwrap() {
+            ChainStatus::Valid { record_count } => {
+                assert_eq!(record_count, 2);
+            }
+            other => panic!("expected Valid chain for mixed v1/v2 records, got {other:?}"),
+        }
+    }
diff --git a/src/attestation/export.rs b/src/attestation/export.rs
index b96b008..9b26e51 100644
--- a/src/attestation/export.rs
+++ b/src/attestation/export.rs
@@ -86,0 +87 @@ mod tests {
+            hash_version: 1,
diff --git a/src/attestation/merkle.rs b/src/attestation/merkle.rs
index 33185d0..d556ba2 100644
--- a/src/attestation/merkle.rs
+++ b/src/attestation/merkle.rs
@@ -191,0 +192 @@ mod tests {
+            hash_version: 1,
diff --git a/src/attestation/types.rs b/src/attestation/types.rs
index 4df6a0e..3234756 100644
--- a/src/attestation/types.rs
+++ b/src/attestation/types.rs
@@ -21,0 +22,7 @@ pub struct AttestationRecord {
+    /// Hash scheme version (1 = legacy delimiter, 2 = length-prefixed canonical)
+    #[serde(default = "default_attestation_hash_version")]
+    pub hash_version: i32,
+}
+
+fn default_attestation_hash_version() -> i32 {
+    1
diff --git a/src/bin/cli/args.rs b/src/bin/cli/args.rs
index 302a415..6226acd 100644
--- a/src/bin/cli/args.rs
+++ b/src/bin/cli/args.rs
@@ -95,0 +96,6 @@ pub(crate) enum Commands {
+        /// Continuously watch directory and auto-mine new/modified transcripts in real-time
+        #[arg(long)]
+        watch: bool,
+        /// Debounce interval in milliseconds for watch mode (default: 1000)
+        #[arg(long, default_value = "1000")]
+        debounce_ms: u64,
diff --git a/src/bin/cli/main.rs b/src/bin/cli/main.rs
index f7d8fcb..641c642 100644
--- a/src/bin/cli/main.rs
+++ b/src/bin/cli/main.rs
@@ -67 +67,14 @@ fn main() -> Result<()> {
-        } => mine::handle_mine(&storage, &path, &mode, wing, room, &workspace)?,
+            watch,
+            debounce_ms,
+        } => mine::handle_mine(
+            &storage,
+            mine::MineOptions {
+                path_str: &path,
+                mode: &mode,
+                wing,
+                room,
+                workspace: &workspace,
+                watch,
+                debounce_ms,
+            },
+        )?,
diff --git a/src/bin/cli/mine.rs b/src/bin/cli/mine.rs
index 0b540d4..65e5a6e 100644
--- a/src/bin/cli/mine.rs
+++ b/src/bin/cli/mine.rs
@@ -2,0 +3 @@
+//! Supports real-time auto-mining daemon mode (`--watch`) via filesystem event notifications.
@@ -4,2 +5,10 @@
-use std::fs;
-use std::path::Path;
+use std::collections::HashMap;
+use std::fs::{self, File};
+use std::io::{Read, Seek, SeekFrom};
+use std::path::{Path, PathBuf};
+#[cfg(feature = "watcher")]
+use std::sync::atomic::{AtomicBool, Ordering};
+#[cfg(feature = "watcher")]
+use std::sync::Arc;
+#[cfg(feature = "watcher")]
+use std::time::Duration;
@@ -11 +20 @@ use engram::storage::Storage;
-use engram::types::{CreateMemoryInput, MemoryTier, MemoryType};
+use engram::types::{CreateMemoryInput, DedupMode, MemoryTier, MemoryType};
@@ -13,8 +22,22 @@ use engram::types::{CreateMemoryInput, MemoryTier, MemoryType};
-pub fn handle_mine(
-    storage: &Storage,
-    path_str: &str,
-    mode: &str,
-    wing: Option<String>,
-    room: Option<String>,
-    workspace: &str,
-) -> Result<()> {
+/// Configuration options for the mining engine.
+#[derive(Debug, Clone)]
+pub struct MineOptions<'a> {
+    pub path_str: &'a str,
+    pub mode: &'a str,
+    pub wing: Option<String>,
+    pub room: Option<String>,
+    pub workspace: &'a str,
+    pub watch: bool,
+    pub debounce_ms: u64,
+}
+
+struct SpatialTarget<'a> {
+    default_wing: String,
+    default_room: String,
+    scope_path: String,
+    workspace: &'a str,
+    #[allow(dead_code)]
+    mode: &'a str,
+}
+
+pub fn handle_mine(storage: &Storage, opts: MineOptions) -> Result<()> {
@@ -22 +45 @@ pub fn handle_mine(
-    let expanded = shellexpand::tilde(path_str).to_string();
+    let expanded = shellexpand::tilde(opts.path_str).to_string();
@@ -32 +55 @@ pub fn handle_mine(
-    let default_wing = wing.unwrap_or_else(|| {
+    let default_wing = opts.wing.unwrap_or_else(|| {
@@ -38 +61 @@ pub fn handle_mine(
-    let default_room = room.unwrap_or_else(|| "general".to_string());
+    let default_room = opts.room.unwrap_or_else(|| "general".to_string());
@@ -40,0 +64,8 @@ pub fn handle_mine(
+    let target = SpatialTarget {
+        default_wing,
+        default_room,
+        scope_path,
+        workspace: opts.workspace,
+        mode: opts.mode,
+    };
+
@@ -45 +76,81 @@ pub fn handle_mine(
-        collect_files(path, mode, &mut files_to_process)?;
+        collect_files(path, opts.mode, &mut files_to_process)?;
+    }
+
+    let mut file_offsets: HashMap<PathBuf, u64> = HashMap::new();
+
+    if !files_to_process.is_empty() {
+        println!(
+            "⛏️  Mining {} file(s) in '{}' mode into [Palace: {}, Wing: {}, Room: {}]...",
+            files_to_process.len(),
+            opts.mode,
+            target.workspace,
+            target.default_wing,
+            target.default_room
+        );
+
+        let mut total_created = 0;
+        let mut total_bytes = 0;
+
+        storage.with_transaction(|conn| {
+            for file in &files_to_process {
+                let content = match fs::read_to_string(file) {
+                    Ok(c) => c,
+                    Err(_) => continue,
+                };
+                let len = content.len() as u64;
+                file_offsets.insert(file.clone(), len);
+                total_bytes += content.len();
+
+                let chunks = extract_chunks(&content, opts.mode, file);
+                for chunk in chunks {
+                    if chunk.trim().is_empty() {
+                        continue;
+                    }
+
+                    let mut tags = vec![
+                        format!("wing:{}", target.default_wing),
+                        format!("room:{}", target.default_room),
+                        format!("source:{}", opts.mode),
+                    ];
+                    if let Some(fname) = file.file_name().and_then(|f| f.to_str()) {
+                        tags.push(format!("file:{}", fname));
+                    }
+
+                    let input = CreateMemoryInput {
+                        content: chunk,
+                        memory_type: MemoryType::Verbatim,
+                        tags,
+                        workspace: Some(target.workspace.to_string()),
+                        tier: MemoryTier::Permanent,
+                        dedup_mode: DedupMode::Skip,
+                        ..Default::default()
+                    };
+
+                    let memory = create_memory(conn, &input)?;
+                    conn.execute(
+                        "UPDATE memories SET scope_path = ? WHERE id = ?",
+                        rusqlite::params![target.scope_path, memory.id],
+                    )?;
+                    total_created += 1;
+                }
+            }
+            Ok(())
+        })?;
+
+        let elapsed = start.elapsed();
+        println!(
+            "✅ Initial baseline: mined {} drawers ({:.2} KB) in {:.2}ms (avg {:.2} µs/record)",
+            total_created,
+            total_bytes as f64 / 1024.0,
+            elapsed.as_secs_f64() * 1000.0,
+            if total_created > 0 {
+                (elapsed.as_micros() as f64) / (total_created as f64)
+            } else {
+                0.0
+            }
+        );
+    } else {
+        println!(
+            "No existing files found in: {}. Ready for new files.",
+            expanded
+        );
@@ -48,2 +159 @@ pub fn handle_mine(
-    if files_to_process.is_empty() {
-        println!("No matching files found to mine in: {}", expanded);
+    if !opts.watch {
@@ -52,0 +163,16 @@ pub fn handle_mine(
+    run_watcher_loop(storage, path, &target, file_offsets, opts.debounce_ms)
+}
+
+#[cfg(feature = "watcher")]
+fn run_watcher_loop(
+    storage: &Storage,
+    path: &Path,
+    target: &SpatialTarget,
+    mut file_offsets: HashMap<PathBuf, u64>,
+    debounce_ms: u64,
+) -> Result<()> {
+    use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
+    use std::sync::mpsc::channel;
+
+    println!("\n👀 Auto-mining daemon active in real-time mode.");
+    println!("📁 Monitoring: {}", path.display());
@@ -54,6 +180,2 @@ pub fn handle_mine(
-        "⛏️  Mining {} file(s) in '{}' mode into [Palace: {}, Wing: {}, Room: {}]...",
-        files_to_process.len(),
-        mode,
-        workspace,
-        default_wing,
-        default_room
+        "🏛️ Palace: {} | Wing: {} | Room: {}",
+        target.workspace, target.default_wing, target.default_room
@@ -60,0 +183 @@ pub fn handle_mine(
+    println!("Press Ctrl+C to stop.\n");
@@ -62,2 +185,2 @@ pub fn handle_mine(
-    let mut total_created = 0;
-    let mut total_bytes = 0;
+    let running = Arc::new(AtomicBool::new(true));
+    let r = running.clone();
@@ -65,7 +188,13 @@ pub fn handle_mine(
-    storage.with_transaction(|conn| {
-        for file in &files_to_process {
-            let content = match fs::read_to_string(file) {
-                Ok(c) => c,
-                Err(_) => continue,
-            };
-            total_bytes += content.len();
+    // Catch SIGINT / Ctrl+C
+    ctrlc_handler(r);
+
+    let (tx, rx) = channel();
+    let mut watcher = RecommendedWatcher::new(
+        move |res: notify::Result<Event>| {
+            if let Ok(event) = res {
+                let _ = tx.send(event);
+            }
+        },
+        Config::default(),
+    )
+    .map_err(|e| EngramError::Storage(format!("Failed to initialize file watcher: {}", e)))?;
@@ -73,3 +202,21 @@ pub fn handle_mine(
-            let chunks = extract_chunks(&content, mode, file);
-            for chunk in chunks {
-                if chunk.trim().is_empty() {
+    let watch_mode = if path.is_dir() {
+        RecursiveMode::Recursive
+    } else {
+        RecursiveMode::NonRecursive
+    };
+
+    watcher
+        .watch(path, watch_mode)
+        .map_err(|e| EngramError::Storage(format!("Failed to watch path: {}", e)))?;
+
+    let mut last_processed = Instant::now();
+    let debounce_duration = Duration::from_millis(debounce_ms);
+
+    while running.load(Ordering::SeqCst) {
+        if let Ok(event) = rx.recv_timeout(Duration::from_millis(200)) {
+            if last_processed.elapsed() < debounce_duration {
+                continue;
+            }
+
+            for event_path in event.paths {
+                if !is_matching_file(&event_path, target.mode) {
@@ -79,7 +226,17 @@ pub fn handle_mine(
-                let mut tags = vec![
-                    format!("wing:{}", default_wing),
-                    format!("room:{}", default_room),
-                    format!("source:{}", mode),
-                ];
-                if let Some(fname) = file.file_name().and_then(|f| f.to_str()) {
-                    tags.push(format!("file:{}", fname));
+                std::thread::sleep(Duration::from_millis(50)); // Small settle time
+
+                let newly_mined =
+                    process_file_update(storage, &event_path, target, &mut file_offsets);
+
+                if let Ok(count) = newly_mined {
+                    if count > 0 {
+                        let now = chrono::Local::now().format("%H:%M:%S");
+                        let fname = event_path
+                            .file_name()
+                            .and_then(|f| f.to_str())
+                            .unwrap_or("file");
+                        println!(
+                            "[{}] ⚡ Auto-mined {} new drawer(s) from '{}' -> [Wing: {}, Room: {}]",
+                            now, count, fname, target.default_wing, target.default_room
+                        );
+                    }
@@ -86,0 +244,4 @@ pub fn handle_mine(
+            }
+            last_processed = Instant::now();
+        }
+    }
@@ -88,8 +249,60 @@ pub fn handle_mine(
-                let input = CreateMemoryInput {
-                    content: chunk,
-                    memory_type: MemoryType::Verbatim,
-                    tags,
-                    workspace: Some(workspace.to_string()),
-                    tier: MemoryTier::Permanent,
-                    ..Default::default()
-                };
+    println!("\n🛑 Auto-mining daemon stopped cleanly.");
+    Ok(())
+}
+
+#[cfg(not(feature = "watcher"))]
+fn run_watcher_loop(
+    _storage: &Storage,
+    _path: &Path,
+    _target: &SpatialTarget,
+    _file_offsets: HashMap<PathBuf, u64>,
+    _debounce_ms: u64,
+) -> Result<()> {
+    Err(EngramError::InvalidInput(
+        "Watch mode requires the 'watcher' feature. Rebuild with: cargo build --features watcher"
+            .to_string(),
+    ))
+}
+
+#[cfg(feature = "watcher")]
+fn process_file_update(
+    storage: &Storage,
+    file_path: &Path,
+    target: &SpatialTarget,
+    file_offsets: &mut HashMap<PathBuf, u64>,
+) -> Result<usize> {
+    if !file_path.exists() || !file_path.is_file() {
+        return Ok(0);
+    }
+
+    let current_size = match fs::metadata(file_path) {
+        Ok(m) => m.len(),
+        Err(_) => return Ok(0),
+    };
+
+    let prev_offset = *file_offsets.get(file_path).unwrap_or(&0);
+
+    // If file was truncated or recreated, reset to 0
+    let offset = if current_size < prev_offset {
+        0
+    } else {
+        prev_offset
+    };
+
+    if current_size == offset {
+        return Ok(0);
+    }
+
+    let mut file = match File::open(file_path) {
+        Ok(f) => f,
+        Err(_) => return Ok(0),
+    };
+
+    if offset > 0 && file.seek(SeekFrom::Start(offset)).is_err() {
+        return Ok(0);
+    }
+
+    let mut new_bytes = Vec::new();
+    if file.read_to_end(&mut new_bytes).is_err() {
+        return Ok(0);
+    }
@@ -97,7 +310,19 @@ pub fn handle_mine(
-                let memory = create_memory(conn, &input)?;
-                // Update scope_path in SQLite
-                conn.execute(
-                    "UPDATE memories SET scope_path = ? WHERE id = ?",
-                    rusqlite::params![scope_path, memory.id],
-                )?;
-                total_created += 1;
+    let new_content = String::from_utf8_lossy(&new_bytes);
+    if new_content.trim().is_empty() {
+        file_offsets.insert(file_path.to_path_buf(), current_size);
+        return Ok(0);
+    }
+
+    let chunks = extract_chunks(&new_content, target.mode, file_path);
+    if chunks.is_empty() {
+        file_offsets.insert(file_path.to_path_buf(), current_size);
+        return Ok(0);
+    }
+
+    let mut created_count = 0;
+    let fname = file_path.file_name().and_then(|f| f.to_str()).unwrap_or("");
+
+    storage.with_transaction(|conn| {
+        for chunk in chunks {
+            if chunk.trim().is_empty() {
+                continue;
@@ -104,0 +330,24 @@ pub fn handle_mine(
+
+            let tags = vec![
+                format!("wing:{}", target.default_wing),
+                format!("room:{}", target.default_room),
+                format!("source:{}", target.mode),
+                format!("file:{}", fname),
+            ];
+
+            let input = CreateMemoryInput {
+                content: chunk,
+                memory_type: MemoryType::Verbatim,
+                tags,
+                workspace: Some(target.workspace.to_string()),
+                tier: MemoryTier::Permanent,
+                dedup_mode: DedupMode::Skip,
+                ..Default::default()
+            };
+
+            let memory = create_memory(conn, &input)?;
+            conn.execute(
+                "UPDATE memories SET scope_path = ? WHERE id = ?",
+                rusqlite::params![target.scope_path, memory.id],
+            )?;
+            created_count += 1;
@@ -109,10 +358,8 @@ pub fn handle_mine(
-    let elapsed = start.elapsed();
-    println!(
-        "✅ Mined {} drawers ({:.2} KB) in {:.2}ms (avg {:.2} µs/record)",
-        total_created,
-        total_bytes as f64 / 1024.0,
-        elapsed.as_secs_f64() * 1000.0,
-        if total_created > 0 {
-            (elapsed.as_micros() as f64) / (total_created as f64)
-        } else {
-            0.0
+    file_offsets.insert(file_path.to_path_buf(), current_size);
+    Ok(created_count)
+}
+
+fn is_matching_file(path: &Path, mode: &str) -> bool {
+    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
+        if name.starts_with('.') || name.ends_with('~') || name.ends_with(".tmp") {
+            return false;
@@ -120 +367 @@ pub fn handle_mine(
-    );
+    }
@@ -122 +369,8 @@ pub fn handle_mine(
-    Ok(())
+    match mode {
+        "convos" => path
+            .extension()
+            .map(|ext| ext == "jsonl" || ext == "json" || ext == "log" || ext == "txt")
+            .unwrap_or(false),
+        "markdown" => path.extension().map(|ext| ext == "md").unwrap_or(false),
+        _ => true,
+    }
@@ -125 +379 @@ pub fn handle_mine(
-fn collect_files(dir: &Path, mode: &str, files: &mut Vec<std::path::PathBuf>) -> Result<()> {
+fn collect_files(dir: &Path, mode: &str, files: &mut Vec<PathBuf>) -> Result<()> {
@@ -130 +383,0 @@ fn collect_files(dir: &Path, mode: &str, files: &mut Vec<std::path::PathBuf>) ->
-                // Ignore hidden directories and common ignore folders
@@ -137,12 +390,2 @@ fn collect_files(dir: &Path, mode: &str, files: &mut Vec<std::path::PathBuf>) ->
-            } else if path.is_file() {
-                let match_file = match mode {
-                    "convos" => path
-                        .extension()
-                        .map(|ext| ext == "jsonl" || ext == "json" || ext == "log" || ext == "txt")
-                        .unwrap_or(false),
-                    "markdown" => path.extension().map(|ext| ext == "md").unwrap_or(false),
-                    _ => true,
-                };
-                if match_file {
-                    files.push(path);
-                }
+            } else if path.is_file() && is_matching_file(&path, mode) {
+                files.push(path);
@@ -158 +400,0 @@ fn extract_chunks(content: &str, mode: &str, file: &Path) -> Vec<String> {
-            // Process JSONL or lines
@@ -166 +407,0 @@ fn extract_chunks(content: &str, mode: &str, file: &Path) -> Vec<String> {
-                    // Check standard JSONL fields (role + content)
@@ -187 +427,0 @@ fn extract_chunks(content: &str, mode: &str, file: &Path) -> Vec<String> {
-            // Split by markdown sections (## or paragraphs)
@@ -208 +447,0 @@ fn extract_chunks(content: &str, mode: &str, file: &Path) -> Vec<String> {
-            // Default 800-char window chunking with overlap
@@ -227,0 +467,21 @@ fn extract_chunks(content: &str, mode: &str, file: &Path) -> Vec<String> {
+
+#[cfg(feature = "watcher")]
+fn ctrlc_handler(running: Arc<AtomicBool>) {
+    ctrlc_helper(move || {
+        running.store(false, Ordering::SeqCst);
+    });
+}
+
+#[cfg(feature = "watcher")]
+fn ctrlc_helper<F: FnOnce() + Send + 'static>(f: F) {
+    let cell = std::sync::Mutex::new(Some(f));
+    tokio::spawn(async move {
+        if let Ok(()) = tokio::signal::ctrl_c().await {
+            if let Ok(mut lock) = cell.lock() {
+                if let Some(handler) = lock.take() {
+                    handler();
+                }
+            }
+        }
+    });
+}
diff --git a/src/bin/cli/portability.rs b/src/bin/cli/portability.rs
index 2a4acea..be3a305 100644
--- a/src/bin/cli/portability.rs
+++ b/src/bin/cli/portability.rs
@@ -15 +15,2 @@ pub(crate) enum ExportAction {
-    /// Export memories to Markdown files
+    /// Export memories to Markdown files (Obsidian vault compatible)
+    #[command(alias = "vault")]
@@ -25,0 +27,3 @@ pub(crate) enum ExportAction {
+        /// Include related memory wikilinks
+        #[arg(long, default_value_t = true)]
+        include_links: bool,
@@ -31 +35,2 @@ pub(crate) enum ImportAction {
-    /// Import memories from Markdown files
+    /// Import memories from Markdown files (Obsidian vault compatible)
+    #[command(alias = "vault")]
@@ -41,0 +47,3 @@ pub(crate) enum ImportAction {
+        /// Force overwrite even on version conflicts
+        #[arg(long)]
+        force_version: bool,
@@ -50,0 +59 @@ pub(crate) fn handle_export(storage: &Storage, action: ExportAction) -> Result<(
+            include_links,
@@ -58,0 +68 @@ pub(crate) fn handle_export(storage: &Storage, action: ExportAction) -> Result<(
+                    include_links,
@@ -75,0 +86 @@ pub(crate) fn handle_import(storage: &Storage, action: ImportAction) -> Result<(
+            force_version,
@@ -82,0 +94 @@ pub(crate) fn handle_import(storage: &Storage, action: ImportAction) -> Result<(
+                    force_version,
diff --git a/src/mcp/handlers/markdown_export/export/query.rs b/src/mcp/handlers/markdown_export/export/query.rs
index f6fc134..bd8945e 100644
--- a/src/mcp/handlers/markdown_export/export/query.rs
+++ b/src/mcp/handlers/markdown_export/export/query.rs
@@ -73 +73 @@ pub(super) fn build_related_map(
-        "SELECT from_id, to_id, relation_type FROM cross_references
+        "SELECT from_id, to_id, edge_type FROM crossrefs
@@ -79,6 +78,0 @@ pub(super) fn build_related_map(
-        // Build params: each id appears twice (once for from_id IN, once for to_id IN).
-        let doubled: Vec<i64> = memory_ids
-            .iter()
-            .chain(memory_ids.iter())
-            .copied()
-            .collect();
@@ -86 +80 @@ pub(super) fn build_related_map(
-            .query_map(rusqlite::params_from_iter(doubled.iter()), |row| {
+            .query_map(rusqlite::params_from_iter(memory_ids.iter()), |row| {
diff --git a/src/mcp/handlers/markdown_export/frontmatter.rs b/src/mcp/handlers/markdown_export/frontmatter.rs
index 6bcb893..9277835 100644
--- a/src/mcp/handlers/markdown_export/frontmatter.rs
+++ b/src/mcp/handlers/markdown_export/frontmatter.rs
@@ -135,15 +135 @@ pub(super) fn extract_body(content: &str) -> &str {
-    if !content.starts_with("---") {
-        return content;
-    }
-    // Skip past first ---\n
-    let pos = content.find('\n').map(|p| p + 1).unwrap_or(content.len());
-    // Find closing ---
-    if let Some(rel) = content[pos..].find("\n---\n") {
-        let body_start = pos + rel + 5; // skip \n---\n
-        &content[body_start..]
-    } else if let Some(rel) = content[pos..].find("\n---") {
-        let after = pos + rel + 4;
-        // after == content.len() → closing marker is at EOF, no body
-        // after < content.len() → body follows immediately after "---"
-        &content[after.min(content.len())..]
-    } else {
+    let body = if !content.starts_with("---") {
@@ -150,0 +137,24 @@ pub(super) fn extract_body(content: &str) -> &str {
+    } else {
+        // Skip past first ---\n
+        let pos = content.find('\n').map(|p| p + 1).unwrap_or(content.len());
+        // Find closing ---
+        if let Some(rel) = content[pos..].find("\n---\n") {
+            let body_start = pos + rel + 5; // skip \n---\n
+            &content[body_start..]
+        } else if let Some(rel) = content[pos..].find("\n---") {
+            let after = pos + rel + 4;
+            // after == content.len() → closing marker is at EOF, no body
+            // after < content.len() → body follows immediately after "---"
+            &content[after.min(content.len())..]
+        } else {
+            content
+        }
+    };
+
+    // Strip auto-generated `## Related` or `## Related Memories` footer if present
+    if let Some(pos) = body.find("\n## Related\n") {
+        &body[..pos]
+    } else if let Some(pos) = body.find("\n## Related Memories\n") {
+        &body[..pos]
+    } else {
+        body
diff --git a/src/mcp/handlers/markdown_export/import/files.rs b/src/mcp/handlers/markdown_export/import/files.rs
index 0d0d4a8..e4aec4f 100644
--- a/src/mcp/handlers/markdown_export/import/files.rs
+++ b/src/mcp/handlers/markdown_export/import/files.rs
@@ -20 +20,12 @@ pub(super) fn collect_md_files_inner(
-            out.push(path);
+            let file_name = path
+                .file_name()
+                .and_then(|f| f.to_str())
+                .unwrap_or("")
+                .to_lowercase();
+            if !file_name.starts_with('.')
+                && !file_name.starts_with('_')
+                && file_name != "index.md"
+                && file_name != "readme.md"
+            {
+                out.push(path);
+            }
diff --git a/src/mcp/handlers/markdown_export/import/handler.rs b/src/mcp/handlers/markdown_export/import/handler.rs
index de1519b..68f6241 100644
--- a/src/mcp/handlers/markdown_export/import/handler.rs
+++ b/src/mcp/handlers/markdown_export/import/handler.rs
@@ -91,6 +91,28 @@ pub fn memory_import_markdown(ctx: &HandlerContext, params: Value) -> Value {
-            files_detail.push(json!({
-                "file": filename,
-                "engram_id": null,
-                "status": "skipped",
-                "reason": "no valid engram_id in frontmatter"
-            }));
+            count_new += 1;
+            if confirm {
+                match create_memory_from_import(ctx, &fm, &body, workspace_override.as_deref()) {
+                    Ok(inserted_id) => {
+                        applied += 1;
+                        files_detail.push(json!({
+                            "file": filename,
+                            "engram_id": inserted_id,
+                            "status": "new",
+                            "applied": true
+                        }));
+                    }
+                    Err(e) => {
+                        files_detail.push(json!({
+                            "file": filename,
+                            "engram_id": null,
+                            "status": "error",
+                            "reason": format!("insert error: {}", e)
+                        }));
+                    }
+                }
+            } else {
+                files_detail.push(json!({
+                    "file": filename,
+                    "engram_id": null,
+                    "status": "new"
+                }));
+            }
@@ -228,0 +251 @@ pub fn memory_import_markdown(ctx: &HandlerContext, params: Value) -> Value {
+        "pending": count_pending,
@@ -229,0 +253 @@ pub fn memory_import_markdown(ctx: &HandlerContext, params: Value) -> Value {
+        "conflict": count_conflict,
diff --git a/src/mcp/handlers/markdown_export/import/integration_tests.rs b/src/mcp/handlers/markdown_export/import/integration_tests.rs
index 2fd2511..eb90239 100644
--- a/src/mcp/handlers/markdown_export/import/integration_tests.rs
+++ b/src/mcp/handlers/markdown_export/import/integration_tests.rs
@@ -222 +222 @@ fn test_import_ignores_obsidian_keys() {
-fn test_import_skips_file_without_engram_id() {
+fn test_import_stages_file_without_engram_id_as_new() {
@@ -237 +237 @@ fn test_import_skips_file_without_engram_id() {
-    let skipped = r["files"]
+    let is_new = r["files"]
@@ -241 +241 @@ fn test_import_skips_file_without_engram_id() {
-        .any(|f| f["status"] == "skipped");
+        .any(|f| f["status"] == "new");
@@ -243,2 +243,2 @@ fn test_import_skips_file_without_engram_id() {
-        skipped,
-        "file without engram_id must be skipped; result={}",
+        is_new,
+        "file without engram_id must be staged as new; result={}",
@@ -247,0 +248,7 @@ fn test_import_skips_file_without_engram_id() {
+
+    // When confirm is true, it should be applied to storage
+    let r_confirmed = memory_import_markdown(
+        &c,
+        json!({"input_dir": dir.path().to_str().unwrap(), "confirm": true}),
+    );
+    assert_eq!(r_confirmed["applied"].as_i64(), Some(1));
diff --git a/src/portability/markdown.rs b/src/portability/markdown.rs
index 13625bd..6f96de9 100644
--- a/src/portability/markdown.rs
+++ b/src/portability/markdown.rs
@@ -53,0 +54,12 @@ pub struct ExportOptions {
+    pub include_links: bool,
+}
+
+impl Default for ExportOptions {
+    fn default() -> Self {
+        Self {
+            output_dir: PathBuf::from("./memories-export"),
+            grouping: ExportGrouping::Flat,
+            workspace: None,
+            include_links: true,
+        }
+    }
@@ -69,0 +82,12 @@ pub struct ImportOptions {
+    pub force_version: bool,
+}
+
+impl Default for ImportOptions {
+    fn default() -> Self {
+        Self {
+            input_dir: PathBuf::from("./memories-export"),
+            dry_run: false,
+            target_workspace: None,
+            force_version: false,
+        }
+    }
@@ -131 +155 @@ pub fn export_markdown(storage: &Storage, opts: &ExportOptions) -> Result<Export
-            "include_links": true
+            "include_links": opts.include_links
@@ -167,0 +192 @@ pub fn import_markdown(storage: &Storage, opts: &ImportOptions) -> Result<Import
+            "force_version": opts.force_version,
@@ -178,2 +203,10 @@ pub fn import_markdown(storage: &Storage, opts: &ImportOptions) -> Result<Import
-    let pending = val.get("pending").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
-    let conflict = val.get("conflict").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
+    let pending = val
+        .get("pending")
+        .or_else(|| val.get("pending_updates"))
+        .and_then(|v| v.as_u64())
+        .unwrap_or(0) as usize;
+    let conflict = val
+        .get("conflict")
+        .or_else(|| val.get("conflicts"))
+        .and_then(|v| v.as_u64())
+        .unwrap_or(0) as usize;
@@ -191,0 +225,17 @@ pub fn import_markdown(storage: &Storage, opts: &ImportOptions) -> Result<Import
+
+/// Preview Markdown files import without mutating storage (dry-run mode).
+pub fn preview_markdown(
+    storage: &Storage,
+    input_dir: PathBuf,
+    target_workspace: Option<String>,
+) -> Result<ImportReport> {
+    import_markdown(
+        storage,
+        &ImportOptions {
+            input_dir,
+            dry_run: true,
+            target_workspace,
+            force_version: false,
+        },
+    )
+}
diff --git a/src/portability/mod.rs b/src/portability/mod.rs
index 8f0c125..c73c873 100644
--- a/src/portability/mod.rs
+++ b/src/portability/mod.rs
@@ -6,2 +6,2 @@ pub use markdown::{
-    export_markdown, import_markdown, ExportGrouping, ExportOptions, ExportReport, ImportOptions,
-    ImportReport,
+    export_markdown, import_markdown, preview_markdown, ExportGrouping, ExportOptions,
+    ExportReport, ImportOptions, ImportReport,
diff --git a/src/storage/migrations/mod.rs b/src/storage/migrations/mod.rs
index 02a2ebf..5d416aa 100644
--- a/src/storage/migrations/mod.rs
+++ b/src/storage/migrations/mod.rs
@@ -7,0 +8 @@ mod v47;
+mod v48;
@@ -19,0 +21 @@ use v47::*;
+use v48::*;
@@ -22 +24 @@ use v47::*;
-pub const SCHEMA_VERSION: i32 = 47;
+pub const SCHEMA_VERSION: i32 = 48;
@@ -237,0 +240,4 @@ pub fn run_migrations(conn: &Connection) -> Result<()> {
+    if current_version < 48 {
+        migrate_v48(conn)?;
+    }
+
diff --git a/src/storage/migrations/tests.rs b/src/storage/migrations/tests.rs
index e9f5048..fd79109 100644
--- a/src/storage/migrations/tests.rs
+++ b/src/storage/migrations/tests.rs
@@ -20 +20 @@ fn test_fresh_db_reaches_current_version() {
-    assert_eq!(version, 47);
+    assert_eq!(version, 48);
@@ -25 +25,21 @@ fn test_schema_version_constant() {
-    assert_eq!(SCHEMA_VERSION, 47);
+    assert_eq!(SCHEMA_VERSION, 48);
+}
+
+#[test]
+fn test_attestation_hash_version_column_exists() {
+    let conn = in_memory_conn();
+    conn.execute(
+        "INSERT INTO attestation_log (document_hash, document_name, document_size, ingested_at, memory_ids, previous_hash, record_hash)
+         VALUES ('h1', 'doc.txt', 10, '2026-08-23T00:00:00Z', '[]', 'genesis', 'rechash1')",
+        [],
+    )
+    .expect("insert attestation record");
+
+    let hash_ver: i32 = conn
+        .query_row(
+            "SELECT hash_version FROM attestation_log LIMIT 1",
+            [],
+            |row| row.get(0),
+        )
+        .expect("query hash_version");
+    assert_eq!(hash_ver, 1);
diff --git a/src/types/config.rs b/src/types/config.rs
index e52f40e..642e6b0 100644
--- a/src/types/config.rs
+++ b/src/types/config.rs
@@ -155 +155 @@ pub struct CreateMemoryInput {
-#[derive(Debug, Clone, Serialize, Deserialize)]
+#[derive(Debug, Clone, Default, Serialize, Deserialize)]
diff --git a/tests/portability_permissions_routing_tests.rs b/tests/portability_permissions_routing_tests.rs
index 8aa6236..3735947 100644
--- a/tests/portability_permissions_routing_tests.rs
+++ b/tests/portability_permissions_routing_tests.rs
@@ -103,0 +104 @@ fn test_markdown_portability_export_and_import_roundtrip() {
+            include_links: true,
@@ -116,0 +118 @@ fn test_markdown_portability_export_and_import_roundtrip() {
+            include_links: true,
@@ -128,0 +131 @@ fn test_markdown_portability_export_and_import_roundtrip() {
+            include_links: true,
@@ -140,0 +144 @@ fn test_markdown_portability_export_and_import_roundtrip() {
+            force_version: false,
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
This line is required for hard post-gate enforcement.
