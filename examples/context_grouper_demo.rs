//! Demo do ContextGrouper inspirado no RTK
//!
//! Este binário demonstra como o ContextGrouper agrupa memórias
//! por tópico para otimizar a injeção de contexto em LLMs.

use chrono::Utc;
use engram::intelligence::context_grouper::{ContextGrouper, MemoryGroup};
use engram::types::{Memory, MemoryType};
use std::collections::HashMap;

fn main() {
    println!("=== ContextGrouper Demo (RTK-inspired) ===\n");

    let grouper = ContextGrouper::new();

    // Simula memórias de uma sessão
    let memories = vec![
        Memory {
            id: 1,
            content: "User prefers dark mode in UI settings".to_string(),
            memory_type: MemoryType::Note,
            tags: vec!["preference".to_string(), "ui".to_string()],
            metadata: HashMap::new(),
            importance: 0.8,
            access_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed_at: None,
            owner_id: None,
            visibility: Default::default(),
            scope: Default::default(),
            workspace: "default".to_string(),
            tier: Default::default(),
            version: 1,
            has_embedding: false,
            expires_at: None,
            content_hash: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            procedure_success_count: 0,
            procedure_failure_count: 0,
            summary_of_id: None,
            lifecycle_state: Default::default(),
            stability: 1.0,
            media_url: None,
        },
        Memory {
            id: 2,
            content: "User likes coffee with oat milk".to_string(),
            memory_type: MemoryType::Note,
            tags: vec!["preference".to_string(), "food".to_string()],
            metadata: HashMap::new(),
            importance: 0.7,
            access_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed_at: None,
            owner_id: None,
            visibility: Default::default(),
            scope: Default::default(),
            workspace: "default".to_string(),
            tier: Default::default(),
            version: 1,
            has_embedding: false,
            expires_at: None,
            content_hash: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            procedure_success_count: 0,
            procedure_failure_count: 0,
            summary_of_id: None,
            lifecycle_state: Default::default(),
            stability: 1.0,
            media_url: None,
        },
        Memory {
            id: 3,
            content: "UI color scheme set to dark".to_string(),
            memory_type: MemoryType::Note,
            tags: vec!["ui".to_string(), "settings".to_string()],
            metadata: HashMap::new(),
            importance: 0.9,
            access_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed_at: None,
            owner_id: None,
            visibility: Default::default(),
            scope: Default::default(),
            workspace: "default".to_string(),
            tier: Default::default(),
            version: 1,
            has_embedding: false,
            expires_at: None,
            content_hash: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            procedure_success_count: 0,
            procedure_failure_count: 0,
            summary_of_id: None,
            lifecycle_state: Default::default(),
            stability: 1.0,
            media_url: None,
        },
        Memory {
            id: 4,
            content: "Coffee preference: oat milk only".to_string(),
            memory_type: MemoryType::Note,
            tags: vec!["preference".to_string(), "food".to_string()],
            metadata: HashMap::new(),
            importance: 0.7,
            access_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed_at: None,
            owner_id: None,
            visibility: Default::default(),
            scope: Default::default(),
            workspace: "default".to_string(),
            tier: Default::default(),
            version: 1,
            has_embedding: false,
            expires_at: None,
            content_hash: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            procedure_success_count: 0,
            procedure_failure_count: 0,
            summary_of_id: None,
            lifecycle_state: Default::default(),
            stability: 1.0,
            media_url: None,
        },
        Memory {
            id: 5,
            content: "User works on Engram project".to_string(),
            memory_type: MemoryType::Note,
            tags: vec!["project".to_string()],
            metadata: HashMap::new(),
            importance: 0.9,
            access_count: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_accessed_at: None,
            owner_id: None,
            visibility: Default::default(),
            scope: Default::default(),
            workspace: "default".to_string(),
            tier: Default::default(),
            version: 1,
            has_embedding: false,
            expires_at: None,
            content_hash: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            procedure_success_count: 0,
            procedure_failure_count: 0,
            summary_of_id: None,
            lifecycle_state: Default::default(),
            stability: 1.0,
            media_url: None,
        },
    ];

    // Agrupa por tópico
    let groups: Vec<MemoryGroup> = grouper.group_for_context(&memories);

    println!("Memory Groups ({} groups found):\n", groups.len());

    for (i, group) in groups.iter().enumerate() {
        println!("--- Group {} ---", i + 1);
        println!("Topic: {}", group.topic);
        println!("Summary: {}", group.summary);
        println!("Count: {}", group.count);
        println!("Memory IDs: {:?}", group.memory_ids);
        println!();
    }

    // Demonstra busca por tópico específico
    println!("\n=== Finding memories by topic ===\n");
    let topic = "User prefers dark";
    let similar = grouper.find_similar_by_topic(topic, &memories);
    println!("Memories with topic '{}': {} found", topic, similar.len());

    for m in &similar {
        println!("  - [{}] {}", m.id, m.content);
    }
}
