//! Fact extraction/retrieval tools (SPO triples, subject graphs).
use serde_json::{json, Value};

use super::super::HandlerContext;
use crate::storage::enrichment_events::{emit_best_effort, EnrichmentEvent};

/// Extract SPO facts from a memory's content and persist them.
///
/// Params:
/// - `memory_id` (i64, required) — source memory to extract from
pub fn memory_extract_facts(ctx: &HandlerContext, params: Value) -> Value {
    use crate::intelligence::fact_extraction::{
        create_fact, ConversationProcessor, RuleBasedExtractor,
    };

    let memory_id = match params.get("memory_id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return json!({"error": "memory_id is required"}),
    };

    ctx.storage
        .with_connection(|conn| {
            // Fetch the memory content.
            let content: Option<String> = conn
                .query_row(
                    "SELECT content FROM memories WHERE id = ?1",
                    rusqlite::params![memory_id],
                    |row| row.get(0),
                )
                .ok();

            let content = match content {
                Some(c) => c,
                None => {
                    return Ok(json!({"error": format!("memory {} not found", memory_id)}));
                }
            };

            // Extract facts.
            let processor = ConversationProcessor::new(Box::new(RuleBasedExtractor::new()));
            let extracted = processor.process_text(&content, Some(memory_id));

            // Persist each fact.
            let mut stored = Vec::new();
            for fact in &extracted {
                if let Ok(f) = create_fact(conn, fact, Some(memory_id)) {
                    stored.push(json!({
                        "id": f.id,
                        "subject": f.subject,
                        "predicate": f.predicate,
                        "object": f.object,
                        "confidence": f.confidence
                    }));
                }
            }

            let facts_stored = stored.len();
            let result = json!({
                "memory_id": memory_id,
                "facts_extracted": extracted.len(),
                "facts_stored": facts_stored,
                "facts": stored
            });

            if facts_stored > 0 {
                let op_id = uuid::Uuid::new_v4().to_string();
                emit_best_effort(
                    conn,
                    &EnrichmentEvent {
                        operation_id: &op_id,
                        event_type: "fact_ingest",
                        memory_id: Some(memory_id),
                        version_id: None,
                        triggered_by: "memory_extract_facts",
                        agent_id: None,
                        workspace: None,
                        params: json!({}),
                        outcome: json!({"facts_stored": facts_stored}),
                        status: "completed",
                        dry_run: false,
                    },
                );
            }

            Ok(result)
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

/// List facts, optionally filtered by source memory.
///
/// Params:
/// - `memory_id` (i64, optional) — filter to facts from this memory
/// - `limit` (u64, optional) — max rows to return (0 = unlimited)
pub fn memory_list_facts(ctx: &HandlerContext, params: Value) -> Value {
    use crate::intelligence::fact_extraction::list_facts;

    let source_id = params.get("memory_id").and_then(|v| v.as_i64());
    let limit = params.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;

    ctx.storage
        .with_connection(|conn| {
            let facts = list_facts(conn, source_id, limit)?;
            let items: Vec<Value> = facts
                .iter()
                .map(|f| {
                    json!({
                        "id": f.id,
                        "subject": f.subject,
                        "predicate": f.predicate,
                        "object": f.object,
                        "confidence": f.confidence,
                        "source_memory_id": f.source_memory_id,
                        "created_at": f.created_at
                    })
                })
                .collect();
            Ok(json!({"facts": items, "count": items.len()}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}

/// Return all facts for a given subject.
///
/// Params:
/// - `subject` (string, required) — the entity to look up
pub fn memory_fact_graph(ctx: &HandlerContext, params: Value) -> Value {
    use crate::intelligence::fact_extraction::get_fact_graph;

    let subject = match params.get("subject").and_then(|v| v.as_str()) {
        Some(s) => s.to_string(),
        None => return json!({"error": "subject is required"}),
    };

    ctx.storage
        .with_connection(|conn| {
            let facts = get_fact_graph(conn, &subject)?;
            let items: Vec<Value> = facts
                .iter()
                .map(|f| {
                    json!({
                        "id": f.id,
                        "subject": f.subject,
                        "predicate": f.predicate,
                        "object": f.object,
                        "confidence": f.confidence,
                        "source_memory_id": f.source_memory_id
                    })
                })
                .collect();
            Ok(json!({"subject": subject, "facts": items, "count": items.len()}))
        })
        .unwrap_or_else(|e| json!({"error": e.to_string()}))
}
