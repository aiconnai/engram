# Engram Harness — External Reviewer Prompt

**Task**: harness-bootstrap
**Mode**: pre
**Date (UTC)**: 2026-08-16

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
diff --git a/src/bin/server.rs b/src/bin/server.rs
index b0f2179..ad38865 100644
--- a/src/bin/server.rs
+++ b/src/bin/server.rs
@@ -457 +457 @@ impl McpHandler for EngramHandler {
-                let tool_result = ToolCallResult::json(&result);
+                let tool_result = ToolCallResult::from_tool_output(&result);
diff --git a/src/mcp/handlers/agent.rs b/src/mcp/handlers/agent.rs
index f2d2d27..786c270 100644
--- a/src/mcp/handlers/agent.rs
+++ b/src/mcp/handlers/agent.rs
@@ -8,0 +9 @@ use super::HandlerContext;
+use crate::mcp::error::ToolError;
@@ -15 +16 @@ pub fn agent_register(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "agent_id is required"}),
+        None => return ToolError::missing_argument("agent_id").into_value(),
@@ -62 +63 @@ pub fn agent_register(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -70 +71 @@ pub fn agent_deregister(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "agent_id is required"}),
+        None => return ToolError::missing_argument("agent_id").into_value(),
@@ -78 +79 @@ pub fn agent_deregister(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -86 +87 @@ pub fn agent_heartbeat(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "agent_id is required"}),
+        None => return ToolError::missing_argument("agent_id").into_value(),
@@ -94 +95 @@ pub fn agent_heartbeat(ctx: &HandlerContext, params: Value) -> Value {
-                None => Ok(json!({"error": "agent not found", "agent_id": agent_id})),
+                None => Ok(ToolError::not_found("agent", agent_id).into_value()),
@@ -97 +98 @@ pub fn agent_heartbeat(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -101 +102 @@ pub fn agent_list(ctx: &HandlerContext, params: Value) -> Value {
-    use crate::storage::agent_registry::list_agents;
+    use crate::storage::agent_registry::{get_agents_in_namespace, list_agents};
@@ -108,2 +108,0 @@ pub fn agent_list(ctx: &HandlerContext, params: Value) -> Value {
-            // When namespace is provided, get agents in that namespace then
-            // apply status filter client-side (storage query only returns active).
@@ -111 +109,0 @@ pub fn agent_list(ctx: &HandlerContext, params: Value) -> Value {
-                use crate::storage::agent_registry::get_agents_in_namespace;
@@ -113,2 +110,0 @@ pub fn agent_list(ctx: &HandlerContext, params: Value) -> Value {
-                    // get_agents_in_namespace hard-codes active, so for inactive
-                    // we fetch all via list_agents and filter by namespace.
@@ -131 +127 @@ pub fn agent_list(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -139 +135 @@ pub fn agent_get(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "agent_id is required"}),
+        None => return ToolError::missing_argument("agent_id").into_value(),
@@ -147 +143 @@ pub fn agent_get(ctx: &HandlerContext, params: Value) -> Value {
-                None => Ok(json!({"error": "agent not found", "agent_id": agent_id})),
+                None => Ok(ToolError::not_found("agent", agent_id).into_value()),
@@ -150 +146 @@ pub fn agent_get(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -158 +154 @@ pub fn agent_capabilities(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "agent_id is required"}),
+        None => return ToolError::missing_argument("agent_id").into_value(),
@@ -166 +162 @@ pub fn agent_capabilities(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "capabilities array is required"}),
+        None => return ToolError::missing_argument("capabilities").into_value(),
@@ -174 +170 @@ pub fn agent_capabilities(ctx: &HandlerContext, params: Value) -> Value {
-                None => Ok(json!({"error": "agent not found", "agent_id": agent_id})),
+                None => Ok(ToolError::not_found("agent", agent_id).into_value()),
@@ -177 +173 @@ pub fn agent_capabilities(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -186 +182 @@ pub fn memory_grant_access(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "agent_id is required"}),
+        None => return ToolError::missing_argument("agent_id").into_value(),
@@ -191 +187 @@ pub fn memory_grant_access(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "scope_path is required"}),
+        None => return ToolError::missing_argument("scope_path").into_value(),
@@ -216 +212 @@ pub fn memory_grant_access(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -225 +221 @@ pub fn memory_revoke_access(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "agent_id is required"}),
+        None => return ToolError::missing_argument("agent_id").into_value(),
@@ -230 +226 @@ pub fn memory_revoke_access(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "scope_path is required"}),
+        None => return ToolError::missing_argument("scope_path").into_value(),
@@ -238 +234 @@ pub fn memory_revoke_access(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -247 +243 @@ pub fn memory_list_grants(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "agent_id is required"}),
+        None => return ToolError::missing_argument("agent_id").into_value(),
@@ -255 +251 @@ pub fn memory_list_grants(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -264 +260 @@ pub fn memory_check_access(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "agent_id is required"}),
+        None => return ToolError::missing_argument("agent_id").into_value(),
@@ -269 +265 @@ pub fn memory_check_access(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "scope_path is required"}),
+        None => return ToolError::missing_argument("scope_path").into_value(),
@@ -287 +283 @@ pub fn memory_check_access(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
diff --git a/src/mcp/handlers/memory_crud/create.rs b/src/mcp/handlers/memory_crud/create.rs
index 2f5c656..670d4dd 100644
--- a/src/mcp/handlers/memory_crud/create.rs
+++ b/src/mcp/handlers/memory_crud/create.rs
@@ -4,0 +5 @@ use super::super::HandlerContext;
+use crate::mcp::error::ToolError;
@@ -15 +16 @@ pub fn memory_create(ctx: &HandlerContext, params: Value) -> Value {
-        Err(e) => return json!({"error": e.to_string()}),
+        Err(e) => return ToolError::invalid_params(e.to_string()).into_value(),
@@ -36,5 +37,5 @@ pub fn memory_create(ctx: &HandlerContext, params: Value) -> Value {
-                            return json!({
-                                "error": format!(
-                                    "Similar memory detected (id={}, similarity={:.3}). Use dedup_mode='allow' to create anyway.",
-                                    existing.id, similarity
-                                ),
+                            return ToolError::conflict(format!(
+                                "Similar memory detected (id={}, similarity={:.3}). Use dedup_mode='allow' to create anyway.",
+                                existing.id, similarity
+                            ))
+                            .with_details(json!({
@@ -43 +44,2 @@ pub fn memory_create(ctx: &HandlerContext, params: Value) -> Value {
-                            });
+                            }))
+                            .into_value();
@@ -284 +286,2 @@ pub fn context_seed(ctx: &HandlerContext, params: Value) -> Value {
-        return json!({"error": "facts must contain at least one non-empty content"});
+        return ToolError::invalid_params("facts must contain at least one non-empty content")
+            .into_value();
@@ -321 +324 @@ pub fn context_seed(ctx: &HandlerContext, params: Value) -> Value {
-        Err(e) => json!({"error": e.to_string()}),
+        Err(e) => ToolError::from(e).into_value(),
@@ -330 +333 @@ pub fn memory_create_daily(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "content is required"}),
+        None => return ToolError::missing_argument("content").into_value(),
@@ -394 +397 @@ pub fn memory_create_daily(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -403 +406 @@ pub fn memory_create_episodic(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "content is required"}),
+        None => return ToolError::missing_argument("content").into_value(),
@@ -409 +412 @@ pub fn memory_create_episodic(ctx: &HandlerContext, params: Value) -> Value {
-            Err(e) => return json!({"error": format!("Invalid event_time format: {}", e)}),
+            Err(e) => return ToolError::invalid_params(format!("Invalid event_time format: {}", e)).into_value(),
@@ -411 +414 @@ pub fn memory_create_episodic(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "event_time is required for episodic memories"}),
+        None => return ToolError::missing_argument("event_time").into_value(),
@@ -465 +468 @@ pub fn memory_create_episodic(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -473 +476 @@ pub fn memory_create_procedural(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "content is required"}),
+        None => return ToolError::missing_argument("content").into_value(),
@@ -478 +481 @@ pub fn memory_create_procedural(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "trigger_pattern is required for procedural memories"}),
+        None => return ToolError::missing_argument("trigger_pattern").into_value(),
@@ -529 +532 @@ pub fn memory_create_procedural(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -537 +540 @@ pub fn memory_create_section(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "title is required"}),
+        None => return ToolError::missing_argument("title").into_value(),
@@ -550 +553 @@ pub fn memory_create_section(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -558 +561 @@ pub fn memory_create_batch(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "memories array is required"}),
+        None => return ToolError::missing_argument("memories").into_value(),
@@ -567 +570 @@ pub fn memory_create_batch(ctx: &HandlerContext, params: Value) -> Value {
-        return json!({"error": "No valid memory inputs provided"});
+        return ToolError::invalid_params("No valid memory inputs provided").into_value();
@@ -575 +578 @@ pub fn memory_create_batch(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
diff --git a/src/mcp/handlers/memory_crud/read_update_delete.rs b/src/mcp/handlers/memory_crud/read_update_delete.rs
index c2d0872..3ca9344 100644
--- a/src/mcp/handlers/memory_crud/read_update_delete.rs
+++ b/src/mcp/handlers/memory_crud/read_update_delete.rs
@@ -5,0 +6 @@ use super::strip_private_content;
+use crate::mcp::error::ToolError;
@@ -12,0 +14,3 @@ pub fn memory_get(ctx: &HandlerContext, params: Value) -> Value {
+    if id <= 0 {
+        return ToolError::missing_argument("id").into_value();
+    }
@@ -30 +34 @@ pub fn memory_get(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -37,0 +42,3 @@ pub fn memory_get_public(ctx: &HandlerContext, params: Value) -> Value {
+    if id <= 0 {
+        return ToolError::missing_argument("id").into_value();
+    }
@@ -49 +56 @@ pub fn memory_get_public(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -53,0 +61,3 @@ pub fn memory_update(ctx: &HandlerContext, params: Value) -> Value {
+    if id <= 0 {
+        return ToolError::missing_argument("id").into_value();
+    }
@@ -56 +66 @@ pub fn memory_update(ctx: &HandlerContext, params: Value) -> Value {
-        Err(e) => return json!({"error": e.to_string()}),
+        Err(e) => return ToolError::invalid_params(e.to_string()).into_value(),
@@ -110 +120 @@ pub fn memory_update(ctx: &HandlerContext, params: Value) -> Value {
-        Err(e) => json!({"error": e.to_string()}),
+        Err(e) => ToolError::from(e).into_value(),
@@ -117,0 +128,3 @@ pub fn memory_delete(ctx: &HandlerContext, params: Value) -> Value {
+    if id <= 0 {
+        return ToolError::missing_argument("id").into_value();
+    }
@@ -165 +178 @@ pub fn memory_delete(ctx: &HandlerContext, params: Value) -> Value {
-            Err(e) => json!({"error": e.to_string()}),
+            Err(e) => ToolError::from(e).into_value(),
@@ -200 +213 @@ pub fn memory_delete(ctx: &HandlerContext, params: Value) -> Value {
-            Err(e) => json!({"error": e.to_string()}),
+            Err(e) => ToolError::from(e).into_value(),
@@ -212 +225 @@ pub fn memory_list(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -221 +234 @@ pub fn memory_delete_batch(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "ids array is required"}),
+        None => return ToolError::missing_argument("ids").into_value(),
@@ -225 +238 @@ pub fn memory_delete_batch(ctx: &HandlerContext, params: Value) -> Value {
-        return json!({"error": "No valid IDs provided"});
+        return ToolError::invalid_params("No valid IDs provided").into_value();
@@ -249 +262 @@ pub fn memory_delete_batch(ctx: &HandlerContext, params: Value) -> Value {
-            .unwrap_or_else(|e| json!({"error": e.to_string()}))
+            .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -256 +269 @@ pub fn memory_delete_batch(ctx: &HandlerContext, params: Value) -> Value {
-            .unwrap_or_else(|e| json!({"error": e.to_string()}))
+            .unwrap_or_else(|e| ToolError::from(e).into_value())
diff --git a/src/mcp/handlers/mod.rs b/src/mcp/handlers/mod.rs
index 8083aa8..35db93f 100644
--- a/src/mcp/handlers/mod.rs
+++ b/src/mcp/handlers/mod.rs
@@ -531 +531 @@ pub fn dispatch(ctx: &HandlerContext, tool_name: &str, params: Value) -> Value {
-        _ => json!({"error": format!("Unknown tool: {}", tool_name)}),
+        _ => crate::mcp::error::ToolError::tool_not_found(tool_name).into_value(),
diff --git a/src/mcp/handlers/search.rs b/src/mcp/handlers/search.rs
index 0befcb0..5c75195 100644
--- a/src/mcp/handlers/search.rs
+++ b/src/mcp/handlers/search.rs
@@ -1,2 +0,0 @@
-//! Search tool handlers.
-
@@ -9,0 +8 @@ use crate::intelligence::memory_policy::{
+use crate::mcp::error::ToolError;
@@ -341 +340 @@ pub fn memory_search(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -356 +355 @@ pub fn memory_search_by_identity(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "identity is required"}),
+        None => return ToolError::missing_argument("identity").into_value(),
@@ -370 +369 @@ pub fn memory_search_by_identity(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -378 +377 @@ pub fn memory_session_search(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "query is required"}),
+        None => return ToolError::missing_argument("query").into_value(),
@@ -393 +392 @@ pub fn memory_session_search(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -413 +412 @@ pub fn find_duplicates(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -436 +435 @@ pub fn find_semantic_duplicates(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -444 +443 @@ pub fn search_cache_feedback(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "query is required"}),
+        None => return ToolError::missing_argument("query").into_value(),
@@ -449 +448 @@ pub fn search_cache_feedback(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "positive is required"}),
+        None => return ToolError::missing_argument("positive").into_value(),
@@ -497 +496 @@ pub fn memory_explain_search(_ctx: &HandlerContext, params: Value) -> Value {
-            return json!({"error": "results array is required (each with memory_id, bm25, vector, fuzzy, recency, importance, final_score, and optional rerank_score)"})
+            return ToolError::invalid_params("results array is required (each with memory_id, bm25, vector, fuzzy, recency, importance, final_score, and optional rerank_score)").into_value();
@@ -551 +550 @@ pub fn memory_feedback(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "query is required"}),
+        None => return ToolError::missing_argument("query").into_value(),
@@ -556 +555 @@ pub fn memory_feedback(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "memory_id is required"}),
+        None => return ToolError::missing_argument("memory_id").into_value(),
@@ -571 +570 @@ pub fn memory_feedback(ctx: &HandlerContext, params: Value) -> Value {
-            return json!({"error": "signal must be 'helpful'/'useful', 'not_helpful'/'irrelevant', 'outdated', or 'conflict'"});
+            return ToolError::invalid_params("signal must be 'helpful'/'useful', 'not_helpful'/'irrelevant', 'outdated', or 'conflict'").into_value();
@@ -627 +626 @@ pub fn memory_feedback(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -640 +639 @@ pub fn memory_feedback_stats(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -648 +647 @@ pub fn memory_explain_utility(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "memory_id is required"}),
+        None => return ToolError::missing_argument("memory_id").into_value(),
@@ -657 +656 @@ pub fn memory_explain_utility(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -674 +673 @@ pub fn memory_search_compact(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "query is required"}),
+        None => return ToolError::missing_argument("query").into_value(),
@@ -743 +742 @@ pub fn memory_search_compact(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -862 +861 @@ pub fn recent_activity(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
@@ -878 +877 @@ pub fn memory_expand(ctx: &HandlerContext, params: Value) -> Value {
-        None => return json!({"error": "ids array is required"}),
+        None => return ToolError::missing_argument("ids").into_value(),
@@ -902 +901 @@ pub fn memory_expand(ctx: &HandlerContext, params: Value) -> Value {
-        .unwrap_or_else(|e| json!({"error": e.to_string()}))
+        .unwrap_or_else(|e| ToolError::from(e).into_value())
diff --git a/src/mcp/mod.rs b/src/mcp/mod.rs
index 7400dae..9a69deb 100644
--- a/src/mcp/mod.rs
+++ b/src/mcp/mod.rs
@@ -4,0 +5 @@
+pub mod error;
@@ -16,0 +18 @@ pub mod tools;
+pub use error::{HandlerResult, ToolError, ToolErrorCode, ToolResult};
diff --git a/src/mcp/permission.rs b/src/mcp/permission.rs
index 621d249..1f91dda 100644
--- a/src/mcp/permission.rs
+++ b/src/mcp/permission.rs
@@ -119,10 +119,2 @@ fn permission_denied(tool_name: &str, current: PermissionMode, required: Permiss
-    json!({
-        "error": {
-            "code": "permission_denied",
-            "tool": tool_name,
-            "current_mode": current.as_str(),
-            "required_mode": required.as_str(),
-            "message": format!("{tool_name} requires {} mode", required.as_str()),
-            "audit_id": null
-        }
-    })
+    crate::mcp::error::ToolError::permission_denied(tool_name, current.as_str(), required.as_str())
+        .into_value()
diff --git a/src/mcp/protocol.rs b/src/mcp/protocol.rs
index e8ef79e..07bf454 100644
--- a/src/mcp/protocol.rs
+++ b/src/mcp/protocol.rs
@@ -357,0 +358,11 @@ impl ToolCallResult {
+    /// Create a result from tool output value, automatically tagging error responses.
+    pub fn from_tool_output(value: &Value) -> Self {
+        let is_err = value.get("error").is_some()
+            || value.get("status").and_then(|v| v.as_str()) == Some("error");
+        let text = serde_json::to_string_pretty(value).unwrap_or_default();
+        Self {
+            content: vec![ToolContent::Text { text }],
+            is_error: if is_err { Some(true) } else { None },
+        }
+    }
+
diff --git a/src/mcp/tools/mod.rs b/src/mcp/tools/mod.rs
index 734c740..0e16a7a 100644
--- a/src/mcp/tools/mod.rs
+++ b/src/mcp/tools/mod.rs
@@ -373 +373,2 @@ mod tests {
-            .find(r#"_ => json!({"error": format!("Unknown tool"#)
+            .find("tool_not_found")
+            .or_else(|| mod_src[d_start..].find(r#"_ => json!({"error": format!("Unknown tool"#))
diff --git a/tests/mcp_protocol_tests.rs b/tests/mcp_protocol_tests.rs
index 894dc97..ab0a9ca 100644
--- a/tests/mcp_protocol_tests.rs
+++ b/tests/mcp_protocol_tests.rs
@@ -525,0 +526,4 @@ fn mcp_mock_parity_scenarios_match_fixture_contract() {
+                let err_msg = result["error"]
+                    .as_str()
+                    .or_else(|| result["error"]["message"].as_str())
+                    .unwrap_or("");
@@ -528 +532 @@ fn mcp_mock_parity_scenarios_match_fixture_contract() {
-                    "error": result["error"].as_str().unwrap_or("")
+                    "error": err_msg
diff --git a/tests/v070_integration_tests.rs b/tests/v070_integration_tests.rs
index 2993148..a6eb60d 100644
--- a/tests/v070_integration_tests.rs
+++ b/tests/v070_integration_tests.rs
@@ -334 +334,5 @@ fn test_mcp_dispatch_unknown_tool_returns_error() {
-    assert!(result["error"].as_str().unwrap().contains("Unknown tool"));
+    let err_msg = result["error"]
+        .as_str()
+        .or_else(|| result["error"]["message"].as_str())
+        .expect("error message");
+    assert!(err_msg.contains("Unknown tool"));
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
