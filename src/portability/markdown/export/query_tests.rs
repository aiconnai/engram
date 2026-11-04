use crate::storage::{queries::create_memory, Storage};
use crate::types::CreateMemoryInput;
struct Fixture {
    storage: Storage,
}
fn ctx() -> Fixture {
    Fixture {
        storage: Storage::open_in_memory().unwrap(),
    }
}
fn p0_memory(c: &Fixture, content: &str, tags: Vec<String>) -> i64 {
    c.storage
        .with_transaction(|conn| {
            Ok(create_memory(
                conn,
                &CreateMemoryInput {
                    content: content.into(),
                    workspace: Some("markdown_export_schema".into()),
                    tags,
                    ..Default::default()
                },
            )?
            .id)
        })
        .unwrap()
}
#[test]
fn p0_expiration_cutoff_excludes_equality_but_preserves_null_and_future() {
    let c = ctx();
    let null = p0_memory(&c, "No expiration", vec![]);
    let past = p0_memory(&c, "Past", vec![]);
    let equal = p0_memory(&c, "At cutoff", vec![]);
    let future = p0_memory(&c, "Future", vec![]);
    c.storage
        .with_connection(|conn| {
            for (id, date) in [
                (past, "2029-01-01T00:00:00+00:00"),
                (equal, "2030-01-01T00:00:00+00:00"),
                (future, "2031-01-01T00:00:00+00:00"),
            ] {
                conn.execute(
                    "UPDATE memories SET tier='daily',expires_at=?1 WHERE id=?2",
                    rusqlite::params![date, id],
                )?;
            }
            let rows = super::query::query_workspace_memories_at(
                conn,
                "markdown_export_schema",
                "2030-01-01T00:00:00+00:00",
            )?;
            let ids: std::collections::BTreeSet<_> =
                rows.iter().map(|r| r["id"].as_i64().unwrap()).collect();
            assert_eq!(ids, std::collections::BTreeSet::from([null, future]));
            Ok(())
        })
        .unwrap();
}

#[test]
fn p0_review_expiration_compares_instants_without_losing_nanoseconds() {
    let c = ctx();
    let unexpired = p0_memory(&c, "No expiry", vec![]);
    let mut expected = std::collections::BTreeSet::from([unexpired]);
    for (date, keep) in [
        ("2030-01-01T00:00:00Z", false),
        ("2030-01-01T00:00:00.000000000Z", false),
        ("2030-01-01T01:00:00+01:00", false),
        ("2029-12-31T23:00:00-01:00", false),
        ("2029-12-31T23:59:59.999999999Z", false),
        ("2030-01-01T00:00:00.000000001Z", true),
    ] {
        let id = p0_memory(&c, date, vec![]);
        c.storage
            .with_connection(|conn| {
                conn.execute(
                    "UPDATE memories SET tier='daily',expires_at=?1 WHERE id=?2",
                    rusqlite::params![date, id],
                )?;
                Ok(())
            })
            .unwrap();
        if keep {
            expected.insert(id);
        }
    }
    c.storage
        .with_connection(|conn| {
            let rows = super::query::query_workspace_memories_at(
                conn,
                "markdown_export_schema",
                "2030-01-01T00:00:00+00:00",
            )?;
            let ids: std::collections::BTreeSet<_> =
                rows.iter().map(|r| r["id"].as_i64().unwrap()).collect();
            assert_eq!(ids, expected);
            Ok(())
        })
        .unwrap();
}

#[test]
fn p0_review_invalid_expiration_is_an_explicit_error() {
    let c = ctx();
    let id = p0_memory(&c, "Bad expiration", vec![]);
    c.storage
        .with_connection(|conn| {
            conn.execute(
                "UPDATE memories SET tier='daily',expires_at='not-a-timestamp' WHERE id=?1",
                [id],
            )?;
            assert!(super::query::query_workspace_memories_at(
                conn,
                "markdown_export_schema",
                "2030-01-01T00:00:00Z"
            )
            .is_err());
            Ok(())
        })
        .unwrap();
}
