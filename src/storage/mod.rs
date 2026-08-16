//! Storage engine for Engram
//!
//! Handles SQLite database operations, WAL mode, and schema management.
//!
//! # Architecture (ENG-14)
//!
//! The storage layer is built around the `StorageBackend` trait which defines
//! the interface for all storage operations. This allows for multiple backend
//! implementations:
//!
//! - `SqliteBackend` - Current default, uses rusqlite with WAL mode
//! - `TursoBackend` - Planned for Phase 6, distributed SQLite
//! - `MeilisearchBackend` - Planned for Phase 7, full-text search focused
//!
//! ## Extension Traits
//!
//! - `TransactionalBackend` - For backends that support ACID transactions
//! - `CloudSyncBackend` - For backends with cloud synchronization

pub mod agent_registry;
mod audit;
pub mod auto_linker;
pub mod backend;
#[cfg(feature = "emergent-graph")]
pub mod clustering;
mod confidence;
mod connection;
pub mod dream_snapshots;
pub mod enrichment_events;
pub mod entity_queries;
pub mod feedback;
pub mod filter;
pub mod graph_queries;
pub mod identity_links;
pub mod image_storage;
mod lock;
pub mod memory_blocks;
mod migrations;
pub mod operational_context;
pub mod pending_injections;
pub mod queries;
pub mod scope_grants;
pub mod scoping;
pub mod sqlite_backend;
pub mod temporal;

#[cfg(feature = "meilisearch")]
pub mod meilisearch_backend;
#[cfg(feature = "meilisearch")]
pub mod meilisearch_indexer;

#[cfg(feature = "turso")]
pub mod turso_backend;

pub use crate::context::{
    ArtifactAccessPolicy, ArtifactRedactionStatus, ArtifactRetentionPolicy,
    ArtifactRetrievalRequest, ContextArtifact, NewContextArtifact, RetrievedContextArtifact,
};
pub use agent_registry::{
    deregister_agent, get_agent, get_agents_in_namespace, heartbeat_agent, list_agents,
    register_agent, update_agent_capabilities, Agent, RegisterAgentInput,
};
pub use audit::*;
pub use auto_linker::{
    auto_link_stats, insert_auto_link, list_auto_links, run_semantic_linker, run_temporal_linker,
    AutoLink, AutoLinkResult, SemanticLinkOptions, TemporalLinkOptions,
};
pub use backend::{
    BatchCreateResult as BackendBatchCreateResult, BatchDeleteResult as BackendBatchDeleteResult,
    CloudSyncBackend, DerivedIndexHealth, DerivedIndexKind, DerivedIndexStatus, HealthStatus,
    StorageBackend, StorageStats, SyncDelta as BackendSyncDelta, SyncResult, SyncState,
    TransactionalBackend,
};
#[cfg(feature = "emergent-graph")]
pub use clustering::{
    get_cluster, list_clusters, run_louvain_clustering, Cluster, ClusteringResult, LouvainOptions,
};
pub use confidence::*;
pub use connection::{Storage, StoragePool};
pub use dream_snapshots::{
    add_dream_candidate_source, create_dream_candidate, create_dream_job, get_dream_candidate,
    get_dream_candidate_with_sources, get_dream_job, list_dream_candidate_sources,
    list_dream_candidates, list_dream_jobs, record_dream_candidate_application,
    review_dream_candidate, transition_dream_job, DreamCandidate, DreamCandidateApplication,
    DreamCandidateSource, DreamCandidateWithSources, DreamJob, NewDreamCandidate,
    NewDreamCandidateSource, NewDreamJob,
};
pub use entity_queries::{
    delete_entity, find_entity, get_entities_for_memory, get_entity, get_entity_stats,
    get_memories_for_entity, link_entity_to_memory, list_entities, search_entities,
    unlink_entity_from_memory, upsert_entity, EntityStats,
};
pub use graph_queries::{
    find_path, get_neighborhood, get_related_multi_hop, ConnectionType, TraversalDirection,
    TraversalNode, TraversalOptions, TraversalResult, TraversalStats,
};
pub use identity_links::{
    add_alias, create_identity, delete_identity, get_aliases, get_identity, get_identity_memories,
    get_memory_identities, link_identity_to_memory, list_identities, normalize_alias, remove_alias,
    resolve_alias, search_identities_by_alias, unlink_identity_from_memory, update_identity,
    CreateIdentityInput, Identity, IdentityAlias, IdentityType, MemoryIdentityLink,
};
pub use image_storage::{
    migrate_images, parse_data_uri, upload_image, ImageRef, ImageStorageConfig, LocalImageStorage,
    MigrationResult, UploadedImage,
};
pub use lock::{LockInfo, StorageLock};
#[cfg(feature = "meilisearch")]
pub use meilisearch_backend::MeilisearchBackend;
#[cfg(feature = "meilisearch")]
pub use meilisearch_indexer::MeilisearchIndexer;
pub(crate) use migrations::SCHEMA_VERSION;
pub use operational_context::{
    create_context_artifact, create_context_event, create_context_summary, get_context_artifact,
    get_context_event, get_context_summary, list_context_artifacts_for_event,
    list_context_events_for_session, list_context_summaries_for_event,
    retrieve_context_artifact_raw, ContextEvent, ContextSummary, NewContextEvent,
    NewContextSummary,
};
pub use queries::{
    acknowledge_share,
    boost_memory,
    cleanup_sync_data,
    clear_events,
    collect_supersedes_chain,
    compute_content_hash,
    compute_dedup_hash,
    create_checkpoint,
    create_memory,
    // Batch operations
    create_memory_batch,
    // Special types
    create_section_memory,
    delete_memory_batch,
    // Import/export
    export_memories,
    get_agent_sync_state,
    get_sync_delta,
    // Advanced sync
    get_sync_version,
    get_tag_hierarchy,
    import_memories,
    // Existing exports
    list_memories_compact,
    // Tag utilities
    list_tags,
    poll_events,
    poll_shared_memories,
    rebuild_crossrefs,
    // Maintenance
    rebuild_embeddings,
    // Event system
    record_event,
    // Search variants
    search_by_identity,
    search_sessions,
    // Multi-agent sharing
    share_memory,
    update_agent_sync_state,
    validate_tags,
    AgentSyncState,
    BatchCreateResult,
    BatchDeleteResult,
    CompactMemoryRow,
    ExportData,
    ImportResult,
    MemoryEvent,
    MemoryEventType,
    SharedMemory,
    SyncDelta,
    SyncVersion,
    TagHierarchyNode,
    TagInfo,
    TagValidationResult,
};
pub use scope_grants::{
    check_scope_access, grant_scope_access, list_grants_for_agent, revoke_scope_access, ScopeGrant,
};
pub use sqlite_backend::{health_check_storage, SqliteBackend};
pub use temporal::{
    MemorySnapshot, StateDiff, TemporalMemory, TemporalQueryEngine, TemporalQueryOptions,
};
#[cfg(feature = "turso")]
#[allow(deprecated)]
pub use turso_backend::{TursoBackend, TursoConfig};
