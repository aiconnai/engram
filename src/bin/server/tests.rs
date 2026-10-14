#[allow(unused_macros)]
macro_rules! engram_server_only {
    ($($tt:tt)*) => {};
}

#[allow(unused_imports)]
pub(crate) use engram_server_only;

#[allow(dead_code)]
fn main() {}

crate::engram_server_only! {
    use crate::*;

    fn test_handler() -> EngramHandler {
        let storage = Storage::open_in_memory().unwrap();
        let embedder = create_embedder(&EmbeddingConfig::default()).unwrap();
        let (progress_tx, _progress_rx) = std::sync::mpsc::channel();
        EngramHandler {
            storage: storage.clone(),
            search_cache: Arc::new(engram::search::result_cache::SearchResultCache::new(
                Default::default(),
            )),
            embedder: embedder.clone(),
            fuzzy_engine: Arc::new(Mutex::new(FuzzyEngine::new())),
            search_config: SearchConfig::default(),
            realtime: None,
            embedding_cache: Arc::new(engram::embedding::EmbeddingCache::default()),
            hnsw_index: Arc::new(parking_lot::RwLock::new(engram::search::HnswIndex::new(
                engram::search::HnswConfig::new(
                    embedder.dimensions(),
                    engram::search::VectorMetric::Cosine,
                ),
            ))),
            #[cfg(feature = "langfuse")]
            langfuse_runtime: Arc::new(
                tokio::runtime::Runtime::new().expect("Failed to create Langfuse runtime"),
            ),
            #[cfg(feature = "meilisearch")]
            meili: None,
            #[cfg(feature = "meilisearch")]
            meili_indexer: None,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval: 300,
            #[cfg(feature = "hooks")]
            hook_manager: None,
            progress_tx,
            subscriptions: Arc::new(engram::mcp::ResourceSubscriptionManager::new()),
        }
    }

    #[test]
    fn test_tool_ingest_document_idempotent() {
        use tempfile::tempdir;

        let dir = tempdir().unwrap();
        let file_path = dir.path().join("doc.md");
        std::fs::write(&file_path, "# Title\n\nHello world.\n").unwrap();

        let handler = test_handler();

        let first = handler.handle_tool_call(
            "memory_ingest_document",
            json!({
                "path": file_path.to_string_lossy(),
                "format": "md"
            }),
            None,
            None,
        );
        assert!(first.get("error").is_none(), "first ingest error: {first}");
        assert!(
            first
                .get("chunks_created")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 0
        );

        let second = handler.handle_tool_call(
            "memory_ingest_document",
            json!({
                "path": file_path.to_string_lossy(),
                "format": "md"
            }),
            None,
            None,
        );
        assert!(
            second.get("error").is_none(),
            "second ingest error: {second}"
        );
        assert_eq!(
            second
                .get("chunks_created")
                .and_then(|v| v.as_u64())
                .unwrap_or(1),
            0
        );
    }

    /// Verifies that `enable_hooks()` populates the manager and that
    /// `trigger_hook(PostToolUse, …)` succeeds with hooks enabled and is a
    /// silent no-op when disabled. Indirectly validates the wiring used by
    /// the `CALL_TOOL` dispatch path.
    #[cfg(feature = "hooks")]
    #[test]
    fn test_hook_wiring() {
        use engram::hooks::{HookContext, HookResult, LifecycleHook};

        let mut handler = test_handler();
        assert!(
            handler.hook_manager.is_none(),
            "hooks should start disabled"
        );

        handler.enable_hooks();
        assert!(
            handler.hook_manager.is_some(),
            "enable_hooks should populate the manager"
        );

        let mut ctx = HookContext::new(Some("test-session".into()), Some("default".into()));
        ctx.metadata
            .insert("tool_name".into(), json!("memory_create"));
        handler.trigger_hook(LifecycleHook::PostToolUse, ctx);

        let stop_context = HookContext::new(Some("test-session".into()), Some("default".into()));
        let stop_results = handler
            .hook_manager
            .as_ref()
            .expect("hook manager should be enabled")
            .trigger(LifecycleHook::Stop, &stop_context)
            .expect("Stop hook should run without error");
        assert!(
            matches!(stop_results.as_slice(), [HookResult::Continue]),
            "Stop hook should be registered and continue, got {stop_results:?}"
        );
    }

    #[cfg(feature = "hooks")]
    #[test]
    fn test_initialize_drains_session_start_pending_injections_per_workspace() {
        use engram::storage::pending_injections;

        let mut handler = test_handler();
        handler.enable_hooks();

        handler
            .storage
            .with_connection(|conn| {
                pending_injections::enqueue(
                    conn,
                    "ws-alpha",
                    r#"{"source_session_id":"s-1","ended_at":"2026-01-01T00:00:00Z","notes":"alpha handoff","notes_truncated":false}"#,
                    Some("s-1"),
                    None,
                )?;
                pending_injections::enqueue(
                    conn,
                    "ws-beta",
                    r#"{"source_session_id":"s-2","ended_at":"2026-01-01T00:00:00Z","notes":"beta handoff","notes_truncated":false}"#,
                    Some("s-2"),
                    None,
                )?;
                Ok(())
            })
            .unwrap();

        let init_req = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: methods::INITIALIZE.to_string(),
            params: json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "workspace": "ws-alpha",
                "session_id": "s-next"
            }),
        };

        let resp = handler.handle_request_as(init_req, None);
        let result = resp.result.expect("initialize result");
        let injection = result
            .get("pending_injections")
            .expect("pending_injections field should be attached on SessionStart");
        assert_eq!(injection.get("workspace"), Some(&json!("ws-alpha")));
        assert_eq!(injection.get("count"), Some(&json!(1)));

        let (alpha_left, beta_left) = handler
            .storage
            .with_connection(|conn| {
                Ok((
                    pending_injections::pending_count(conn, "ws-alpha")?,
                    pending_injections::pending_count(conn, "ws-beta")?,
                ))
            })
            .unwrap();
        assert_eq!(alpha_left, 0, "ws-alpha queue should be drained");
        assert_eq!(beta_left, 1, "ws-beta queue must remain untouched");
    }

    #[cfg(feature = "hooks")]
    #[test]
    fn test_call_tool_post_tool_use_enforces_workspace_scoping() {
        use engram::auth::{PermissionSet, TokenClaims, TransportPrincipal, UserId};
        use engram::storage::queries::{create_memory, get_policy_record};
        use std::collections::HashMap;

        let mut handler = test_handler();
        handler.enable_hooks();

        let (alpha_mem, beta_mem) = handler
            .storage
            .with_connection(|conn| {
                let alpha = create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: "alpha scoped note".to_string(),
                        memory_type: MemoryType::Note,
                        tags: vec![],
                        metadata: HashMap::new(),
                        importance: None,
                        scope: Default::default(),
                        workspace: Some("ws-alpha".to_string()),
                        tier: Default::default(),
                        defer_embedding: true,
                        ttl_seconds: None,
                        dedup_mode: Default::default(),
                        dedup_threshold: None,
                        event_time: None,
                        event_duration_seconds: None,
                        trigger_pattern: None,
                        summary_of_id: None,
                        media_url: None,
                    },
                )?;
                let beta = create_memory(
                    conn,
                    &CreateMemoryInput {
                        content: "beta scoped note".to_string(),
                        memory_type: MemoryType::Note,
                        tags: vec![],
                        metadata: HashMap::new(),
                        importance: None,
                        scope: Default::default(),
                        workspace: Some("ws-beta".to_string()),
                        tier: Default::default(),
                        defer_embedding: true,
                        ttl_seconds: None,
                        dedup_mode: Default::default(),
                        dedup_threshold: None,
                        event_time: None,
                        event_duration_seconds: None,
                        trigger_pattern: None,
                        summary_of_id: None,
                        media_url: None,
                    },
                )?;
                Ok((alpha, beta))
            })
            .unwrap();

        // Even when memory_expand returns memories from both workspaces,
        // PostToolUse must scope reinforcement strictly to the active workspace
        // ("ws-alpha") rather than reinforcing foreign workspace memories.
        let call_req = McpRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(2)),
            method: methods::CALL_TOOL.to_string(),
            params: json!({
                "name": "memory_expand",
                "arguments": {
                    "ids": [alpha_mem.id, beta_mem.id],
                    "workspace": "ws-alpha",
                    "session_id": "sess-1"
                }
            }),
        };

        let _resp = handler.handle_request_as(call_req, None);

        // Also verify that a namespace-bound principal always overrides any
        // caller-supplied workspace parameter during hook workspace resolution.
        let principal = TransportPrincipal::from_token_claims(TokenClaims {
            user_id: UserId::from_string("user-alpha"),
            key_id: "key-alpha".to_string(),
            permissions: PermissionSet::admin(),
            namespace: Some("ws-alpha".to_string()),
            issued_at: chrono::Utc::now(),
            expires_at: None,
        })
        .unwrap();
        assert_eq!(
            resolve_hook_workspace(&json!({"workspace": "ws-beta"}), Some(&principal)),
            "ws-alpha"
        );

        let (alpha_policy, beta_policy) = handler
            .storage
            .with_connection(|conn| {
                Ok((
                    get_policy_record(conn, alpha_mem.id)?.expect("alpha policy record"),
                    get_policy_record(conn, beta_mem.id)?.expect("beta policy record"),
                ))
            })
            .unwrap();

        assert_eq!(
            alpha_policy.reinforcement_count, 1,
            "memory in ws-alpha should be reinforced by PostToolUse"
        );
        assert_eq!(
            beta_policy.reinforcement_count, 0,
            "memory in ws-beta must NOT be reinforced when PostToolUse is scoped to ws-alpha"
        );
    }
}
