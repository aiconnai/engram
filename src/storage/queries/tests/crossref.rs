use super::*;

#[test]
fn test_collect_supersedes_chain_three_nodes() {
    // C supersedes B supersedes A
    // collect_supersedes_chain(C) should return [C, B, A]
    let storage = Storage::open_in_memory().unwrap();
    storage
        .with_connection(|conn| {
            let id_a = make_memory(conn);
            let id_b = make_memory(conn);
            let id_c = make_memory(conn);
            link_supersedes(conn, id_b, id_a); // B supersedes A
            link_supersedes(conn, id_c, id_b); // C supersedes B

            let chain = collect_supersedes_chain(conn, id_c).unwrap();
            assert!(chain.contains(&id_c), "chain must include root");
            assert!(chain.contains(&id_b), "chain must include B");
            assert!(chain.contains(&id_a), "chain must include A");
            assert_eq!(chain.len(), 3);
            Ok(())
        })
        .unwrap();
}

#[test]
fn test_collect_supersedes_chain_no_ancestors() {
    let storage = Storage::open_in_memory().unwrap();
    storage
        .with_connection(|conn| {
            let id_a = make_memory(conn);
            let chain = collect_supersedes_chain(conn, id_a).unwrap();
            assert_eq!(chain, vec![id_a]);
            Ok(())
        })
        .unwrap();
}

#[test]
fn test_delete_memory_cascade_chain_true() {
    // C supersedes B supersedes A; deleting C with cascade_chain=true removes all three
    let storage = Storage::open_in_memory().unwrap();
    storage
        .with_connection(|conn| {
            let id_a = make_memory(conn);
            let id_b = make_memory(conn);
            let id_c = make_memory(conn);
            link_supersedes(conn, id_b, id_a);
            link_supersedes(conn, id_c, id_b);

            let chain = collect_supersedes_chain(conn, id_c).unwrap();
            for &id in &chain {
                delete_memory(conn, id).unwrap();
            }

            // All three should be soft-deleted (valid_to is set)
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM memories WHERE id IN (?1, ?2, ?3) AND valid_to IS NULL",
                    params![id_a, id_b, id_c],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 0, "all three memories should be deleted");
            Ok(())
        })
        .unwrap();
}

#[test]
fn test_delete_memory_cascade_chain_false_leaves_ancestors() {
    // C supersedes B supersedes A; deleting C without cascade leaves B and A intact
    let storage = Storage::open_in_memory().unwrap();
    storage
        .with_connection(|conn| {
            let id_a = make_memory(conn);
            let id_b = make_memory(conn);
            let id_c = make_memory(conn);
            link_supersedes(conn, id_b, id_a);
            link_supersedes(conn, id_c, id_b);

            delete_memory(conn, id_c).unwrap();

            // B and A should still be alive
            let count: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM memories WHERE id IN (?1, ?2) AND valid_to IS NULL",
                    params![id_a, id_b],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 2, "ancestors must survive when cascade_chain=false");
            Ok(())
        })
        .unwrap();
}
