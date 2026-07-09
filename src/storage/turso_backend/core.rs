//! Turso/libSQL implementation of the StorageBackend trait (Phase 6 - ENG-54)
//!
//! This module provides a Turso/libSQL-based storage backend that implements
//! the `StorageBackend` trait, enabling distributed SQLite with edge replicas.
//!
//! # Features
//!
//! - **Embedded replicas**: Local SQLite with sync to Turso cloud
//! - **Edge-native**: Sub-millisecond reads from local replica
//! - **Sync on demand**: Push/pull changes to cloud
//! - **Compatible schema**: Same migrations as SQLite backend
//!
//! # Usage
//!
//! ```rust,ignore
//! use engram::storage::TursoBackend;
//!
//! // Connect to Turso cloud with embedded replica
//! let backend = TursoBackend::new(
//!     "libsql://your-db.turso.io",
//!     "your-auth-token",
//!     Some("/path/to/local/replica.db"),
//! ).await?;
//!
//! // Or use local-only mode (no cloud sync)
//! let backend = TursoBackend::local_only("/path/to/db.sqlite").await?;
//! ```

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use libsql::{Builder, Connection, Database};
use tokio::sync::RwLock;

use crate::error::{EngramError, Result};
use crate::storage::migrations::SCHEMA_VERSION;
use crate::types::{
    LifecycleState, Memory, MemoryId, MemoryScope, MemoryTier, MemoryType, Visibility,
};

use crate::storage::backend::SyncResult;

pub(crate) const MEMORY_COLUMNS: &str = "id, content, memory_type, importance, access_count, created_at, updated_at, last_accessed_at, owner_id, visibility, version, has_embedding, metadata, scope_type, scope_id, workspace, tier, expires_at, content_hash, event_time, event_duration_seconds, trigger_pattern, procedure_success_count, procedure_failure_count, summary_of_id, lifecycle_state";

/// Turso/libSQL storage backend configuration
#[derive(Debug, Clone)]
pub struct TursoConfig {
    /// Turso database URL (e.g., "libsql://your-db.turso.io")
    pub url: String,
    /// Authentication token for Turso cloud
    pub auth_token: Option<String>,
    /// Path to local embedded replica (for offline support)
    pub local_replica_path: Option<String>,
    /// Sync interval in seconds (0 = manual sync only)
    pub sync_interval_secs: u64,
    /// Whether to sync on startup
    pub sync_on_startup: bool,
}

impl Default for TursoConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            auth_token: None,
            local_replica_path: None,
            sync_interval_secs: 60,
            sync_on_startup: true,
        }
    }
}

/// Turso/libSQL-based storage backend
///
/// Implements the `StorageBackend` trait using libSQL (Turso's fork of SQLite)
/// with support for embedded replicas and cloud sync.
pub struct TursoBackend {
    pub(crate) db: Database,
    pub(crate) conn: Arc<RwLock<Connection>>,
    pub(crate) config: TursoConfig,
    pub(crate) schema_initialized: bool,
}

impl TursoBackend {
    /// Create a new Turso backend connected to Turso cloud
    ///
    /// # Arguments
    /// * `url` - Turso database URL
    /// * `auth_token` - Authentication token
    /// * `local_replica_path` - Optional path for embedded replica
    pub async fn new(
        url: &str,
        auth_token: &str,
        local_replica_path: Option<&str>,
    ) -> Result<Self> {
        let config = TursoConfig {
            url: url.to_string(),
            auth_token: Some(auth_token.to_string()),
            local_replica_path: local_replica_path.map(|s| s.to_string()),
            ..Default::default()
        };

        Self::with_config(config).await
    }

    /// Create a new Turso backend with custom configuration
    pub async fn with_config(config: TursoConfig) -> Result<Self> {
        let db = if let Some(ref replica_path) = config.local_replica_path {
            // Embedded replica mode: local SQLite with sync to cloud
            Builder::new_remote_replica(
                replica_path,
                config.url.clone(),
                config.auth_token.clone().unwrap_or_default(),
            )
            .build()
            .await
            .map_err(|e| EngramError::Storage(format!("Failed to create Turso replica: {}", e)))?
        } else if config.url.starts_with("libsql://") || config.url.starts_with("https://") {
            // Remote-only mode: direct connection to Turso cloud
            Builder::new_remote(
                config.url.clone(),
                config.auth_token.clone().unwrap_or_default(),
            )
            .build()
            .await
            .map_err(|e| EngramError::Storage(format!("Failed to connect to Turso: {}", e)))?
        } else {
            // Local-only mode: pure SQLite via libSQL
            Builder::new_local(&config.url).build().await.map_err(|e| {
                EngramError::Storage(format!("Failed to open local database: {}", e))
            })?
        };

        let conn = db
            .connect()
            .map_err(|e| EngramError::Storage(format!("Failed to get connection: {}", e)))?;

        let mut backend = Self {
            db,
            conn: Arc::new(RwLock::new(conn)),
            config,
            schema_initialized: false,
        };

        // Initialize schema
        backend.init_schema().await?;

        // Sync on startup if configured
        if backend.config.sync_on_startup && backend.config.local_replica_path.is_some() {
            let _ = backend.sync().await;
        }

        Ok(backend)
    }

    /// Create a local-only Turso backend (no cloud sync)
    pub async fn local_only(path: &str) -> Result<Self> {
        let config = TursoConfig {
            url: path.to_string(),
            auth_token: None,
            local_replica_path: None,
            sync_interval_secs: 0,
            sync_on_startup: false,
        };
        Self::with_config(config).await
    }

    /// Create an in-memory Turso backend (useful for testing)
    pub async fn in_memory() -> Result<Self> {
        Self::local_only(":memory:").await
    }

    /// Initialize the database schema
    async fn init_schema(&mut self) -> Result<()> {
        if self.schema_initialized {
            return Ok(());
        }

        let conn = self.conn.write().await;

        // Create schema version table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            (),
        )
        .await
        .map_err(|e| EngramError::Storage(format!("Schema init failed: {}", e)))?;

        // Check current version
        let version: i32 = conn
            .query("SELECT COALESCE(MAX(version), 0) FROM schema_version", ())
            .await
            .map_err(|e| EngramError::Storage(e.to_string()))?
            .next()
            .await
            .map_err(|e| EngramError::Storage(e.to_string()))?
            .map(|row| row.get::<i32>(0).unwrap_or(0))
            .unwrap_or(0);

        // Apply migrations
        if version < SCHEMA_VERSION {
            self.apply_migration_v1(&conn).await?;
        }

        self.schema_initialized = true;
        Ok(())
    }

    /// Apply migration v1 - base schema
    async fn apply_migration_v1(&self, conn: &Connection) -> Result<()> {
        // Memories table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                content TEXT NOT NULL,
                memory_type TEXT NOT NULL DEFAULT 'note',
                importance REAL NOT NULL DEFAULT 0.5,
                access_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                last_accessed_at TEXT,
                owner_id TEXT,
                visibility TEXT NOT NULL DEFAULT 'private',
                version INTEGER NOT NULL DEFAULT 1,
                has_embedding INTEGER NOT NULL DEFAULT 0,
                embedding_queued_at TEXT,
                valid_from TEXT NOT NULL DEFAULT (datetime('now')),
                valid_to TEXT,
                metadata TEXT NOT NULL DEFAULT '{}',
                scope_type TEXT NOT NULL DEFAULT 'global',
                scope_id TEXT,
                expires_at TEXT,
                content_hash TEXT,
                workspace TEXT NOT NULL DEFAULT 'default',
                tier TEXT NOT NULL DEFAULT 'permanent',
                event_time TEXT,
                event_duration_seconds INTEGER,
                trigger_pattern TEXT,
                procedure_success_count INTEGER DEFAULT 0,
                procedure_failure_count INTEGER DEFAULT 0,
                summary_of_id INTEGER REFERENCES memories(id) ON DELETE SET NULL,
                lifecycle_state TEXT DEFAULT 'active'
            )",
            (),
        )
        .await
        .map_err(|e| EngramError::Storage(e.to_string()))?;

        // Tags table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            (),
        )
        .await
        .map_err(|e| EngramError::Storage(e.to_string()))?;

        // Memory-Tags junction table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS memory_tags (
                memory_id INTEGER NOT NULL,
                tag_id INTEGER NOT NULL,
                PRIMARY KEY (memory_id, tag_id),
                FOREIGN KEY (memory_id) REFERENCES memories(id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
            )",
            (),
        )
        .await
        .map_err(|e| EngramError::Storage(e.to_string()))?;

        // Cross-references table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS crossrefs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                from_id INTEGER NOT NULL,
                to_id INTEGER NOT NULL,
                edge_type TEXT NOT NULL DEFAULT 'related_to',
                score REAL NOT NULL,
                confidence REAL NOT NULL DEFAULT 1.0,
                strength REAL NOT NULL DEFAULT 1.0,
                source TEXT NOT NULL DEFAULT 'auto',
                source_context TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                valid_from TEXT NOT NULL DEFAULT (datetime('now')),
                valid_to TEXT,
                pinned INTEGER NOT NULL DEFAULT 0,
                metadata TEXT NOT NULL DEFAULT '{}',
                FOREIGN KEY (from_id) REFERENCES memories(id) ON DELETE CASCADE,
                FOREIGN KEY (to_id) REFERENCES memories(id) ON DELETE CASCADE,
                UNIQUE(from_id, to_id, edge_type)
            )",
            (),
        )
        .await
        .map_err(|e| EngramError::Storage(e.to_string()))?;

        // Identities table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS identities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                canonical_id TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                identity_type TEXT DEFAULT 'unknown',
                metadata TEXT DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            )",
            (),
        )
        .await
        .map_err(|e| EngramError::Storage(e.to_string()))?;

        // Entities table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS entities (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                metadata TEXT DEFAULT '{}',
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(name, entity_type)
            )",
            (),
        )
        .await
        .map_err(|e| EngramError::Storage(e.to_string()))?;

        // Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_memories_workspace ON memories(workspace)",
            (),
        )
        .await
        .ok();
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_memories_type ON memories(memory_type)",
            (),
        )
        .await
        .ok();
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_memories_tier ON memories(tier)",
            (),
        )
        .await
        .ok();
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_memories_lifecycle ON memories(lifecycle_state)",
            (),
        )
        .await
        .ok();
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_memories_created ON memories(created_at)",
            (),
        )
        .await
        .ok();
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_crossrefs_from ON crossrefs(from_id)",
            (),
        )
        .await
        .ok();
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_crossrefs_to ON crossrefs(to_id)",
            (),
        )
        .await
        .ok();

        // Record migration
        conn.execute(
            "INSERT INTO schema_version (version) VALUES (?)",
            libsql::params![SCHEMA_VERSION],
        )
        .await
        .map_err(|e| EngramError::Storage(e.to_string()))?;

        Ok(())
    }

    /// Sync with Turso cloud (if using embedded replica)
    pub async fn sync(&self) -> Result<SyncResult> {
        if self.config.local_replica_path.is_none() {
            return Ok(SyncResult {
                success: true,
                pushed_count: 0,
                pulled_count: 0,
                conflicts_resolved: 0,
                error: Some("No local replica configured".to_string()),
                new_version: 0,
            });
        }

        self.db
            .sync()
            .await
            .map_err(|e| EngramError::Sync(format!("Turso sync failed: {}", e)))?;

        Ok(SyncResult {
            success: true,
            pushed_count: 0,
            pulled_count: 0,
            conflicts_resolved: 0,
            error: None,
            new_version: 0,
        })
    }

    /// Execute a query and return results
    pub(crate) async fn query_memories(
        &self,
        sql: &str,
        params: Vec<libsql::Value>,
    ) -> Result<Vec<Memory>> {
        let conn = self.conn.read().await;
        let stmt = conn
            .prepare(sql)
            .await
            .map_err(|e| EngramError::Storage(e.to_string()))?;

        let rows = stmt
            .query(params)
            .await
            .map_err(|e| EngramError::Storage(e.to_string()))?;

        let mut memories = Vec::new();
        let mut rows = rows;

        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| EngramError::Storage(e.to_string()))?
        {
            let mut memory = self.row_to_memory(&row)?;
            memory.tags = self.load_tags_with_conn(&conn, memory.id).await?;
            memories.push(memory);
        }

        Ok(memories)
    }

    /// Convert a database row to a Memory struct
    fn row_to_memory(&self, row: &libsql::Row) -> Result<Memory> {
        let id: i64 = row
            .get(0)
            .map_err(|e| EngramError::Storage(e.to_string()))?;
        let content: String = row
            .get(1)
            .map_err(|e| EngramError::Storage(e.to_string()))?;
        let memory_type_str: String = row
            .get(2)
            .map_err(|e| EngramError::Storage(e.to_string()))?;
        let importance: f32 = row
            .get::<f64>(3)
            .map_err(|e| EngramError::Storage(e.to_string()))? as f32;
        let access_count: i32 =
            row.get::<i64>(4)
                .map_err(|e| EngramError::Storage(e.to_string()))? as i32;
        let created_at: String = row
            .get(5)
            .map_err(|e| EngramError::Storage(e.to_string()))?;
        let updated_at: String = row
            .get(6)
            .map_err(|e| EngramError::Storage(e.to_string()))?;
        let last_accessed_at: Option<String> = row
            .get(7)
            .map_err(|e| EngramError::Storage(e.to_string()))?;
        let owner_id: Option<String> = row
            .get(8)
            .map_err(|e| EngramError::Storage(e.to_string()))?;
        let visibility_str: String = row
            .get(9)
            .map_err(|e| EngramError::Storage(e.to_string()))?;
        let version: i32 = row
            .get::<i64>(10)
            .map_err(|e| EngramError::Storage(e.to_string()))? as i32;
        let has_embedding: i32 =
            row.get::<i64>(11)
                .map_err(|e| EngramError::Storage(e.to_string()))? as i32;
        let metadata_str: String = row
            .get(12)
            .map_err(|e| EngramError::Storage(e.to_string()))?;
        let scope_type: String = row.get(13).unwrap_or_else(|_| "global".to_string());
        let scope_id: Option<String> = row.get(14).unwrap_or(None);
        let workspace: String = row.get(15).unwrap_or_else(|_| "default".to_string());
        let tier_str: String = row.get(16).unwrap_or_else(|_| "permanent".to_string());
        let expires_at: Option<String> = row.get(17).unwrap_or(None);
        let content_hash: Option<String> = row.get(18).unwrap_or(None);
        let event_time: Option<String> = row.get(19).unwrap_or(None);
        let event_duration_seconds: Option<i64> = row.get(20).unwrap_or(None);
        let trigger_pattern: Option<String> = row.get(21).unwrap_or(None);
        let procedure_success_count: i32 = row.get(22).unwrap_or(0);
        let procedure_failure_count: i32 = row.get(23).unwrap_or(0);
        let summary_of_id: Option<i64> = row.get(24).unwrap_or(None);
        let lifecycle_state_str: Option<String> = row.get(25).unwrap_or(None);

        let memory_type = memory_type_str.parse().unwrap_or(MemoryType::Note);
        let visibility = match visibility_str.as_str() {
            "shared" => Visibility::Shared,
            "public" => Visibility::Public,
            _ => Visibility::Private,
        };

        let scope = match (scope_type.as_str(), scope_id) {
            ("user", Some(id)) => MemoryScope::User { user_id: id },
            ("session", Some(id)) => MemoryScope::Session { session_id: id },
            ("agent", Some(id)) => MemoryScope::Agent { agent_id: id },
            _ => MemoryScope::Global,
        };

        let metadata: HashMap<String, serde_json::Value> =
            serde_json::from_str(&metadata_str).unwrap_or_default();
        let tier = tier_str.parse().unwrap_or(MemoryTier::Permanent);
        let lifecycle_state = lifecycle_state_str
            .and_then(|s| s.parse().ok())
            .unwrap_or(LifecycleState::Active);

        Ok(Memory {
            id,
            content,
            memory_type,
            tags: Vec::new(),
            metadata,
            importance,
            access_count,
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_at)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            last_accessed_at: Self::parse_datetime(last_accessed_at),
            owner_id,
            visibility,
            scope,
            workspace,
            tier,
            version,
            has_embedding: has_embedding != 0,
            expires_at: Self::parse_datetime(expires_at),
            content_hash,
            event_time: Self::parse_datetime(event_time),
            event_duration_seconds,
            trigger_pattern,
            procedure_success_count,
            procedure_failure_count,
            summary_of_id,
            lifecycle_state,
            media_url: None,
        })
    }

    fn parse_datetime(value: Option<String>) -> Option<DateTime<Utc>> {
        value.and_then(|s| {
            DateTime::parse_from_rfc3339(&s)
                .map(|dt| dt.with_timezone(&Utc))
                .ok()
        })
    }

    async fn load_tags_with_conn(
        &self,
        conn: &Connection,
        memory_id: MemoryId,
    ) -> Result<Vec<String>> {
        let stmt = conn
            .prepare(
                "SELECT t.name
                 FROM tags t
                 INNER JOIN memory_tags mt ON mt.tag_id = t.id
                 WHERE mt.memory_id = ?
                 ORDER BY t.name",
            )
            .await
            .map_err(|e| EngramError::Storage(e.to_string()))?;

        let rows = stmt
            .query(libsql::params![memory_id])
            .await
            .map_err(|e| EngramError::Storage(e.to_string()))?;

        let mut tags = Vec::new();
        let mut rows = rows;
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| EngramError::Storage(e.to_string()))?
        {
            let name: String = row.get(0).unwrap_or_default();
            tags.push(name);
        }

        Ok(tags)
    }
}
