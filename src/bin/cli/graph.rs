use engram::error::Result;
use engram::graph::KnowledgeGraph;
use engram::storage::queries::{get_related, list_memories};
use engram::storage::Storage;
use engram::types::ListOptions;

pub(crate) fn export(
    storage: &Storage,
    format: String,
    output: String,
    max_nodes: i64,
) -> Result<()> {
    let options = ListOptions {
        limit: Some(max_nodes),
        ..Default::default()
    };

    let (memories, crossrefs) = storage.with_connection(|conn| {
        let memories = list_memories(conn, &options)?;
        let mut all_crossrefs = Vec::new();
        for memory in &memories {
            if let Ok(refs) = get_related(conn, memory.id) {
                all_crossrefs.extend(refs);
            }
        }
        Ok((memories, all_crossrefs))
    })?;

    let graph = KnowledgeGraph::from_data(&memories, &crossrefs);

    let content = match format.as_str() {
        "json" => serde_json::to_string_pretty(&graph.to_visjs_json())?,
        _ => graph.to_html(),
    };

    if output == "-" {
        println!("{}", content);
    } else {
        std::fs::write(&output, content)?;
        println!("Graph exported to {}", output);
    }
    Ok(())
}
