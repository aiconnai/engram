//! Application state for the Engram MCP server.
//!
//! Wraps storage, embedder, search engine, realtime broadcaster, and
//! (when the `hooks` feature is enabled) the lifecycle `HookManager`.

use crate::embedding::{Embedder, EmbeddingCache};
#[cfg(feature = "hooks")]
use crate::hooks::{HookContext, HookManager, HookResult, LifecycleHook};
use crate::realtime::RealtimeManager;
#[cfg(feature = "meilisearch")]
use crate::search::MeilisearchIndexer;
use crate::search::{FuzzyEngine, SearchConfig};
#[cfg(feature = "meilisearch")]
use crate::storage::MeilisearchBackend;
use crate::storage::Storage;
use std::sync::Arc;

/// Application state shared across MCP handlers.
pub struct AppState {
    pub storage: Arc<Storage>,
    pub embedder: Arc<dyn Embedder + Send + Sync>,
    pub fuzzy_engine: Arc<parking_lot::Mutex<FuzzyEngine>>,
    pub search_config: SearchConfig,
    pub realtime: Option<Arc<RealtimeManager>>,
    pub embedding_cache: Arc<EmbeddingCache>,
    #[cfg(feature = "meilisearch")]
    pub meili: Option<Arc<MeilisearchBackend>>,
    #[cfg(feature = "meilisearch")]
    pub meili_indexer: Option<Arc<MeilisearchIndexer>>,
    #[cfg(feature = "meilisearch")]
    pub meili_sync_interval: Option<u64>,
    /// Hook manager for lifecycle hooks (Phase L - ENG-78). See issue #11
    /// for the in-progress wiring of `trigger_hook` into MCP handlers.
    #[cfg(feature = "hooks")]
    pub hook_manager: Option<Arc<HookManager>>,
}

impl AppState {
    pub fn new(
        storage: Arc<Storage>,
        embedder: Arc<dyn Embedder + Send + Sync>,
        fuzzy_engine: Arc<parking_lot::Mutex<FuzzyEngine>>,
        search_config: SearchConfig,
        realtime: Option<Arc<RealtimeManager>>,
        embedding_cache: Arc<EmbeddingCache>,
        #[cfg(feature = "meilisearch")] meili: Option<Arc<MeilisearchBackend>>,
        #[cfg(feature = "meilisearch")] meili_indexer: Option<Arc<MeilisearchIndexer>>,
        #[cfg(feature = "meilisearch")] meili_sync_interval: Option<u64>,
    ) -> Self {
        Self {
            storage,
            embedder,
            fuzzy_engine,
            search_config,
            realtime,
            embedding_cache,
            #[cfg(feature = "meilisearch")]
            meili,
            #[cfg(feature = "meilisearch")]
            meili_indexer,
            #[cfg(feature = "meilisearch")]
            meili_sync_interval,
            #[cfg(feature = "hooks")]
            hook_manager: None,
        }
    }

    /// Enable lifecycle hooks (call during startup if hooks feature is enabled).
    #[cfg(feature = "hooks")]
    pub fn enable_hooks(&mut self) {
        let mut hook_manager = HookManager::new();

        hook_manager.register(LifecycleHook::SessionStart, |_hook, context| {
            eprintln!(
                "[Hook] SessionStart: session_id={:?}, workspace={:?}",
                context.session_id, context.workspace
            );
            Ok(HookResult::Continue)
        });

        self.hook_manager = Some(Arc::new(hook_manager));
        tracing::info!("Lifecycle hooks enabled");
    }

    /// Trigger a lifecycle hook. Returns an empty vec if hooks are not enabled.
    #[cfg(feature = "hooks")]
    pub fn trigger_hook(
        &self,
        hook: LifecycleHook,
        context: HookContext,
    ) -> crate::Result<Vec<HookResult>> {
        if let Some(ref manager) = self.hook_manager {
            manager.trigger(hook, &context)
        } else {
            Ok(vec![])
        }
    }
}
