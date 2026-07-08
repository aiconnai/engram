use super::types::SessionHandoffPacket;
use crate::error::Result;
use crate::storage::queries::create_memory;
use crate::types::{CreateMemoryInput, MemoryTier, MemoryType};
use crate::Storage;

pub(super) fn persist_checkpoint(storage: &Storage, packet: &SessionHandoffPacket) -> Result<i64> {
    let content =
        serde_json::to_string_pretty(packet).unwrap_or_else(|_| packet.copy_block.clone());
    let mut tags = vec!["session-handoff".to_string()];
    if let Some(session_id) = &packet.session_id {
        tags.push(format!("session:{session_id}"));
    }

    storage.with_transaction(|conn| {
        let memory = create_memory(
            conn,
            &CreateMemoryInput {
                content,
                memory_type: MemoryType::Checkpoint,
                tags,
                workspace: Some(packet.workspace.clone()),
                importance: Some(0.9),
                tier: MemoryTier::Permanent,
                ..Default::default()
            },
        )?;
        Ok(memory.id)
    })
}
