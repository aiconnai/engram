use crate::error::EngramError;
use std::collections::HashMap;

use crate::storage::backend::{DerivedIndexHealth, DerivedIndexStatus, HealthStatus};

use super::{MeilisearchBackend, MEMORIES_INDEX};

pub(super) fn health_check(backend: &MeilisearchBackend) -> Result<HealthStatus, EngramError> {
    backend.rt.block_on(async {
        match backend.client.health().await {
            Ok(_) => {
                let index_health = backend.client.index(MEMORIES_INDEX).get_stats().await;
                let derived_indexes = match index_health {
                    Ok(index_stats) => {
                        let num_docs = index_stats.number_of_documents as i64;
                        let is_indexing_str = index_stats.is_indexing.to_string();
                        let mut meta = HashMap::new();
                        meta.insert("index_name".to_string(), MEMORIES_INDEX.to_string());
                        meta.insert("source_count".to_string(), num_docs.to_string());
                        meta.insert("is_indexing".to_string(), is_indexing_str);
                        vec![DerivedIndexHealth::external(
                            MEMORIES_INDEX,
                            DerivedIndexStatus::Healthy,
                            num_docs,
                            num_docs,
                            meta,
                        )]
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        let mut meta = HashMap::new();
                        meta.insert("index_name".to_string(), MEMORIES_INDEX.to_string());
                        meta.insert("error".to_string(), err_msg);
                        vec![DerivedIndexHealth::external(
                            MEMORIES_INDEX,
                            DerivedIndexStatus::Unavailable,
                            0,
                            0,
                            meta,
                        )]
                    }
                };

                Ok(HealthStatus {
                    healthy: true,
                    latency_ms: 0.0,
                    error: None,
                    details: HashMap::from([("backend".to_string(), "meilisearch".to_string())]),
                    derived_indexes,
                })
            }
            Err(e) => Ok(HealthStatus {
                healthy: false,
                latency_ms: 0.0,
                error: Some(e.to_string()),
                details: HashMap::from([("backend".to_string(), "meilisearch".to_string())]),
                derived_indexes: vec![DerivedIndexHealth::external(
                    MEMORIES_INDEX,
                    DerivedIndexStatus::Unavailable,
                    0,
                    0,
                    HashMap::from([
                        ("index_name".to_string(), MEMORIES_INDEX.to_string()),
                        ("error".to_string(), e.to_string()),
                    ]),
                )],
            }),
        }
    })
}
