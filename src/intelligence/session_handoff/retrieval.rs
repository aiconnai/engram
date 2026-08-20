use super::privacy::strip_private_content;
use super::types::{HandoffItem, SessionHandoffPacket};
use crate::error::Result;
use crate::Storage;

pub(super) fn collect_open_items(storage: &Storage, workspace: &str) -> Result<Vec<HandoffItem>> {
    storage.with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, content, memory_type \
             FROM memories \
             WHERE workspace = ?1 \
               AND memory_type IN ('todo', 'issue') \
               AND (lifecycle_state IS NULL OR lifecycle_state != 'archived') \
             ORDER BY importance DESC, created_at DESC \
             LIMIT 50",
        )?;
        let items = stmt
            .query_map([workspace], |row| {
                let id: i64 = row.get(0)?;
                let content: String = row.get(1)?;
                let memory_type: String = row.get(2)?;
                Ok(memory_item(id, content, &memory_type))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(items)
    })
}

pub(super) fn collect_recent_decisions(
    storage: &Storage,
    workspace: &str,
) -> Result<Vec<HandoffItem>> {
    storage.with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, content \
             FROM memories \
             WHERE workspace = ?1 \
               AND memory_type = 'decision' \
             ORDER BY created_at DESC \
             LIMIT 20",
        )?;
        let items = stmt
            .query_map([workspace], |row| {
                let id: i64 = row.get(0)?;
                let content: String = row.get(1)?;
                Ok(memory_item(id, content, "memory_decision"))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(items)
    })
}

pub(super) fn collect_topic_digest_items(
    storage: &Storage,
    workspace: &str,
    query: &str,
) -> Result<Vec<HandoffItem>> {
    let clean_query = query.trim();
    if clean_query.is_empty() {
        return Ok(Vec::new());
    }

    let pattern = format!("%{clean_query}%");
    storage.with_connection(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, content, memory_type \
             FROM memories \
             WHERE workspace = ?1 \
               AND content LIKE ?2 \
               AND (lifecycle_state IS NULL OR lifecycle_state != 'archived') \
             ORDER BY importance DESC, created_at DESC \
             LIMIT 15",
        )?;
        let items = stmt
            .query_map(rusqlite::params![workspace, pattern], |row| {
                let id: i64 = row.get(0)?;
                let content: String = row.get(1)?;
                let memory_type: String = row.get(2)?;
                Ok(memory_item(id, content, &format!("digest_{memory_type}")))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(items)
    })
}

pub(super) fn push_source_ids(packet: &mut SessionHandoffPacket) {
    let mut ids = packet
        .open_items
        .iter()
        .chain(packet.decisions.iter())
        .chain(packet.verification.iter())
        .chain(packet.risks.iter())
        .chain(packet.blockers.iter())
        .filter_map(|item| item.source_memory_id)
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids.dedup();
    packet.source_memory_ids = ids;
}

fn memory_item(id: i64, content: String, detail: &str) -> HandoffItem {
    HandoffItem {
        title: truncate_preview(&strip_private_content(&content), 200),
        detail: Some(detail.to_string()),
        source_memory_id: Some(id),
        source_context_event_id: None,
    }
}

fn truncate_preview(content: &str, max_chars: usize) -> String {
    if content.chars().count() <= max_chars {
        return content.to_string();
    }

    let mut preview = content
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    preview.push('…');
    preview
}
