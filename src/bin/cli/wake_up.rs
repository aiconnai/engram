//! Compact agent wake-up digest generator.
//! Generates a sub-200 token bootstrap prompt with palace orientation, active wings, recent decisions, and tools.

use engram::error::Result;
use engram::storage::Storage;

pub fn handle_wake_up(storage: &Storage, workspace: &str, format: &str) -> Result<()> {
    storage.with_connection(|conn| {
        // 1. Total memories & wings
        let mut stmt = conn.prepare(
            "SELECT scope_path, COUNT(*) FROM memories
             WHERE workspace = ? AND lifecycle_state != 'archived'
             GROUP BY scope_path",
        )?;

        let mut wings = std::collections::BTreeSet::new();
        let mut total_drawers = 0;
        let rows = stmt.query_map([workspace], |row| {
            let path: Option<String> = row.get(0)?;
            let count: i64 = row.get(1)?;
            Ok((path.unwrap_or_else(|| "global".to_string()), count))
        })?;

        for r in rows {
            let (path, count) = r?;
            total_drawers += count;
            let clean = path.trim_start_matches("wing:").trim_start_matches('/');
            let wing = clean.split('/').next().unwrap_or("general");
            wings.insert(wing.to_string());
        }

        // 2. Recent decisions (ADRs)
        let mut stmt_decisions = conn.prepare(
            "SELECT content FROM memories
             WHERE workspace = ? AND memory_type = 'decision' AND lifecycle_state != 'archived'
             ORDER BY updated_at DESC LIMIT 3",
        )?;
        let decisions: Vec<String> = stmt_decisions
            .query_map([workspace], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        // 3. Pending todos
        let mut stmt_todos = conn.prepare(
            "SELECT content FROM memories
             WHERE workspace = ? AND memory_type = 'todo' AND lifecycle_state != 'archived'
             ORDER BY updated_at DESC LIMIT 3",
        )?;
        let todos: Vec<String> = stmt_todos
            .query_map([workspace], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        if format == "json" {
            let payload = serde_json::json!({
                "palace": workspace,
                "total_drawers": total_drawers,
                "wings": wings.into_iter().collect::<Vec<_>>(),
                "recent_decisions": decisions,
                "pending_todos": todos,
                "essential_tools": ["palace_navigate", "room_search", "drawer_open", "memory_smart_retrieve", "session_land"]
            });
            println!("{}", serde_json::to_string_pretty(&payload)?);
            return Ok(());
        }

        // Print compact Markdown digest (~150 tokens)
        println!("## 🏛️ Engram Memory Wake-Up (Palace: {})", workspace);
        println!(
            "- **Status**: {} active drawers across {} wings: [{}]",
            total_drawers,
            wings.len(),
            wings.into_iter().collect::<Vec<_>>().join(", ")
        );

        if !decisions.is_empty() {
            println!("- **Recent Decisions / ADRs**:");
            for d in &decisions {
                let first_line = d.lines().next().unwrap_or(d);
                println!("  • {}", first_line);
            }
        }

        if !todos.is_empty() {
            println!("- **Pending Tasks**:");
            for t in &todos {
                let first_line = t.lines().next().unwrap_or(t);
                println!("  • {}", first_line);
            }
        }

        println!("- **Query Syntax**: Use `palace_navigate` / `room_search` for spatial navigation, `memory_smart_retrieve` for intent retrieval, and `session_land` on handoff.");

        Ok(())
    })
}
