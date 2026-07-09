//! Database queries for memory operations.

use crate::intelligence::{extract_features, score_policy, PolicyFeatureInput};
use crate::storage::queries::sync::{record_event, MemoryEventType};
use crate::storage::queries::{
    emit_policy_event, record_reinforcement, upsert_policy_record, PolicyRecordInput,
};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Row};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::error::{EngramError, Result};
use crate::storage::filter::{parse_filter, SqlBuilder};
use crate::types::*;

mod compact;
mod crossref;
mod dedup;
mod dream;
mod duplicate_embedding;
mod duplicate_types;
mod duplicate_workspace;
mod expiration;
mod hash;
mod identity;
mod list;
mod memory_create;
mod memory_delete;
mod memory_read;
mod memory_update;
mod metadata;
mod procedure;
mod row;
mod session;
mod stats;
mod tag_links;
mod tier;
mod versions;
mod workspace;

pub use compact::{list_memories_compact, CompactMemoryRow};
pub use crossref::{create_crossref, delete_crossref, get_crossref, get_related};
pub use dedup::{find_by_content_hash, find_similar_by_embedding};
#[cfg(feature = "dream-phase")]
pub use dream::insert_dream_run;
pub use dream::{acquire_dream_lock, release_dream_lock};
pub use duplicate_embedding::find_duplicates_by_embedding;
pub use duplicate_types::{DuplicateMatchType, DuplicatePair};
pub use duplicate_workspace::{find_duplicates, find_duplicates_in_workspace};
pub use expiration::{cleanup_expired_memories, count_expired_memories, set_memory_expiration};
pub use hash::{compute_content_hash, compute_content_hash_raw, compute_dedup_hash};
pub use identity::search_by_identity;
pub use list::list_memories;
pub use memory_create::create_memory;
pub use memory_delete::delete_memory;
pub use memory_read::{get_memory, load_tags};
pub use memory_update::update_memory;
pub(crate) use metadata::metadata_value_to_param;
pub use procedure::{get_episodic_timeline, get_procedural_memories, record_procedure_outcome};
pub use row::memory_from_row;
pub use session::search_sessions;
pub use stats::get_stats;
use tag_links::ensure_tag;
pub use tier::promote_to_permanent;
pub use versions::get_memory_versions;
pub use workspace::{delete_workspace, get_workspace_stats, list_workspaces, move_to_workspace};

use memory_read::get_memory_internal;

#[cfg(test)]
mod policy_integration_tests {
    use super::*;
    use crate::storage::queries::get_policy_record;
    use crate::storage::Storage;

    fn policy_test_memory_input(content: &str) -> CreateMemoryInput {
        CreateMemoryInput {
            content: content.to_string(),
            memory_type: MemoryType::Note,
            tags: vec![],
            metadata: HashMap::new(),
            importance: None,
            scope: Default::default(),
            workspace: None,
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
        }
    }

    #[test]
    fn create_memory_initializes_policy_record() {
        let storage = Storage::open_in_memory().unwrap();

        storage
            .with_connection(|conn| {
                let memory = create_memory(conn, &policy_test_memory_input("policy init"))?;
                let policy = get_policy_record(conn, memory.id)?
                    .expect("create_memory should initialize memory_policy");

                assert_eq!(policy.memory_id, memory.id);
                assert_eq!(policy.reinforcement_count, 0);
                assert_eq!(policy.policy_version, "heuristic-v1");

                let events: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM enrichment_events
                     WHERE memory_id = ?1
                       AND event_type = 'memory_policy'
                       AND triggered_by = 'create_memory'",
                    params![memory.id],
                    |row| row.get(0),
                )?;
                assert_eq!(events, 1);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn promote_to_permanent_records_policy_reinforcement() {
        let storage = Storage::open_in_memory().unwrap();

        storage
            .with_connection(|conn| {
                let mut input = policy_test_memory_input("policy promotion");
                input.tier = MemoryTier::Daily;
                input.ttl_seconds = Some(3600);
                let memory = create_memory(conn, &input)?;

                let promoted = promote_to_permanent(conn, memory.id)?;
                assert_eq!(promoted.tier, MemoryTier::Permanent);

                let policy = get_policy_record(conn, memory.id)?
                    .expect("promotion should keep policy record");
                assert_eq!(policy.reinforcement_count, 1);
                assert!(policy.last_reinforced_at.is_some());
                Ok(())
            })
            .unwrap();
    }
}
