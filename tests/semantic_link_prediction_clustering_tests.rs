//! Integration tests for Semantic Link Prediction & Concept Clustering.

use engram::graph::concept_clustering::{cluster_concepts, ConceptClusterOptions};
use engram::graph::link_prediction::{predict_links, PredictLinksOptions};
use engram::storage::queries::{create_crossref, create_memory};
use engram::storage::Storage;
use engram::types::{CreateCrossRefInput, CreateMemoryInput, EdgeType};

#[test]
fn test_link_prediction_transitive_and_topological() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            // Memory A
            let mem_a = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Authentication service uses OAuth2 and JWT tokens".to_string(),
                    tags: vec!["auth".to_string(), "security".to_string()],
                    workspace: Some("core".to_string()),
                    ..Default::default()
                },
            )?;

            // Memory B
            let mem_b = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Session management verifies JWT signatures and expiration"
                        .to_string(),
                    tags: vec!["auth".to_string(), "session".to_string()],
                    workspace: Some("core".to_string()),
                    ..Default::default()
                },
            )?;

            // Memory C
            let mem_c = create_memory(
                conn,
                &CreateMemoryInput {
                    content:
                        "User profile service validates session before returning sensitive claims"
                            .to_string(),
                    tags: vec!["session".to_string(), "user".to_string()],
                    workspace: Some("core".to_string()),
                    ..Default::default()
                },
            )?;

            // Link A -> B
            create_crossref(
                conn,
                &CreateCrossRefInput {
                    from_id: mem_a.id,
                    to_id: mem_b.id,
                    edge_type: EdgeType::RelatedTo,
                    strength: Some(1.0),
                    pinned: false,
                    source_context: None,
                },
            )?;

            // Link B -> C
            create_crossref(
                conn,
                &CreateCrossRefInput {
                    from_id: mem_b.id,
                    to_id: mem_c.id,
                    edge_type: EdgeType::RelatedTo,
                    strength: Some(1.0),
                    pinned: false,
                    source_context: None,
                },
            )?;

            // Predict links
            let opts = PredictLinksOptions {
                memory_id: Some(mem_a.id),
                workspace: Some("core".to_string()),
                min_confidence: 0.2,
                top_k: 5,
                algorithm: "transitive".to_string(),
                auto_apply: false,
            };

            let res = predict_links(conn, None, &opts)?;
            assert!(
                !res.predictions.is_empty(),
                "Should predict 2-hop link A -> C"
            );
            let target_pred = res.predictions.iter().find(|p| {
                (p.from_id == mem_a.id && p.to_id == mem_c.id)
                    || (p.from_id == mem_c.id && p.to_id == mem_a.id)
            });
            assert!(
                target_pred.is_some(),
                "Link between A and C should be predicted"
            );
            assert_eq!(res.applied_count, 0);

            Ok(())
        })
        .unwrap();
}

#[test]
fn test_link_prediction_auto_apply() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            let m1 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Redis caching layer for query performance".to_string(),
                    tags: vec!["database".to_string(), "cache".to_string()],
                    workspace: Some("infra".to_string()),
                    ..Default::default()
                },
            )?;
            let m2 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "Cache invalidation hooks on write operations".to_string(),
                    tags: vec!["database".to_string(), "cache".to_string()],
                    workspace: Some("infra".to_string()),
                    ..Default::default()
                },
            )?;

            let opts = PredictLinksOptions {
                memory_id: Some(m1.id),
                workspace: Some("infra".to_string()),
                min_confidence: 0.1,
                top_k: 5,
                algorithm: "hybrid".to_string(),
                auto_apply: true,
            };

            let res = predict_links(conn, None, &opts)?;
            assert_eq!(res.count, 1);
            assert_eq!(res.applied_count, 1);

            // Verify edge was created
            let count: i64 = conn.query_row(
                "SELECT COUNT(*) FROM crossrefs WHERE from_id = ?1 AND to_id = ?2",
                [m1.id, m2.id],
                |r| r.get(0),
            )?;
            assert_eq!(count, 1);

            Ok(())
        })
        .unwrap();
}

#[test]
fn test_concept_clustering_synthesis() {
    let storage = Storage::open_in_memory().unwrap();

    storage
        .with_transaction(|conn| {
            // Auth cluster
            let a1 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "OAuth2 Provider Setup".to_string(),
                    tags: vec!["security".to_string(), "auth".to_string()],
                    workspace: Some("app".to_string()),
                    ..Default::default()
                },
            )?;
            let a2 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "JWT validation interceptor".to_string(),
                    tags: vec!["security".to_string(), "auth".to_string()],
                    workspace: Some("app".to_string()),
                    ..Default::default()
                },
            )?;
            create_crossref(
                conn,
                &CreateCrossRefInput {
                    from_id: a1.id,
                    to_id: a2.id,
                    edge_type: EdgeType::RelatedTo,
                    strength: Some(1.0),
                    pinned: false,
                    source_context: None,
                },
            )?;

            // Database cluster
            let d1 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "PostgreSQL Migration V1".to_string(),
                    tags: vec!["database".to_string(), "postgres".to_string()],
                    workspace: Some("app".to_string()),
                    ..Default::default()
                },
            )?;
            let d2 = create_memory(
                conn,
                &CreateMemoryInput {
                    content: "PostgreSQL Connection Pooling".to_string(),
                    tags: vec!["database".to_string(), "postgres".to_string()],
                    workspace: Some("app".to_string()),
                    ..Default::default()
                },
            )?;
            create_crossref(
                conn,
                &CreateCrossRefInput {
                    from_id: d1.id,
                    to_id: d2.id,
                    edge_type: EdgeType::RelatedTo,
                    strength: Some(1.0),
                    pinned: false,
                    source_context: None,
                },
            )?;

            let opts = ConceptClusterOptions {
                workspace: Some("app".to_string()),
                min_cluster_size: 2,
                max_clusters: 5,
            };

            let clusters = cluster_concepts(conn, &opts)?;
            assert_eq!(
                clusters.len(),
                2,
                "Should identify 2 separate concept clusters"
            );
            assert!(clusters
                .iter()
                .any(|c| c.key_tags.contains(&"auth".to_string())));
            assert!(clusters
                .iter()
                .any(|c| c.key_tags.contains(&"database".to_string())));

            Ok(())
        })
        .unwrap();
}
