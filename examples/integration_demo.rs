//! Demo da integração completa inspirada no RTK
//!
//! Este binário demonstra o uso do IntegrationOrchestrator para preparar
//! contexto otimizado para LLMs, reduzindo o uso de tokens em 70-95%.

use engram::intelligence::integration_orchestrator::IntegrationOrchestrator;
use engram::types::{Memory, MemoryType};
use std::collections::HashMap;

fn main() {
    println!("=== Engram RTK-Inspired Integration Demo ===\n");

    let orchestrator = IntegrationOrchestrator::new();

    // Simula consulta do usuário
    let query = "What are the user preferences?";

    // Simula memórias recuperadas (em produção, viriam do banco de dados)
    let memories = vec![
        Memory {
            id: 1,
            content: "User prefers dark mode".to_string(),
            memory_type: MemoryType::Preference,
            tags: vec!["ui".to_string(), "preference".to_string()],
            metadata: HashMap::new(),
            importance: 0.8,
            access_count: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_accessed_at: None,
            owner_id: None,
            visibility: engram::types::Visibility::default(),
            scope: engram::types::MemoryScope::default(),
            workspace: "default".to_string(),
            tier: engram::types::MemoryTier::default(),
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
            lifecycle_state: engram::types::LifecycleState::default(),
            media_url: None,
        },
        Memory {
            id: 2,
            content: "User likes coffee with oat milk".to_string(),
            memory_type: MemoryType::Preference,
            tags: vec!["food".to_string(), "preference".to_string()],
            metadata: HashMap::new(),
            importance: 0.7,
            access_count: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_accessed_at: None,
            owner_id: None,
            visibility: engram::types::Visibility::default(),
            scope: engram::types::MemoryScope::default(),
            workspace: "default".to_string(),
            tier: engram::types::MemoryTier::default(),
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
            lifecycle_state: engram::types::LifecycleState::default(),
            media_url: None,
        },
        Memory {
            id: 3,
            content: "User is learning Rust programming".to_string(),
            memory_type: MemoryType::Learning,
            tags: vec!["programming".to_string(), "rust".to_string()],
            metadata: HashMap::new(),
            importance: 0.9,
            access_count: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_accessed_at: None,
            owner_id: None,
            visibility: engram::types::Visibility::default(),
            scope: engram::types::MemoryScope::default(),
            workspace: "default".to_string(),
            tier: engram::types::MemoryTier::default(),
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
            lifecycle_state: engram::types::LifecycleState::default(),
            media_url: None,
        },
        Memory {
            id: 4,
            content: "User prefers VS Code as editor".to_string(),
            memory_type: MemoryType::Preference,
            tags: vec!["tools".to_string(), "editor".to_string()],
            metadata: HashMap::new(),
            importance: 0.6,
            access_count: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_accessed_at: None,
            owner_id: None,
            visibility: engram::types::Visibility::default(),
            scope: engram::types::MemoryScope::default(),
            workspace: "default".to_string(),
            tier: engram::types::MemoryTier::default(),
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
            lifecycle_state: engram::types::LifecycleState::default(),
            media_url: None,
        },
        Memory {
            id: 5,
            content: "User works on AI projects".to_string(),
            memory_type: MemoryType::Context,
            tags: vec!["ai".to_string(), "work".to_string()],
            metadata: HashMap::new(),
            importance: 0.85,
            access_count: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            last_accessed_at: None,
            owner_id: None,
            visibility: engram::types::Visibility::default(),
            scope: engram::types::MemoryScope::default(),
            workspace: "default".to_string(),
            tier: engram::types::MemoryTier::default(),
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
            lifecycle_state: engram::types::LifecycleState::default(),
            media_url: None,
        },
    ];

    println!("Query: {}", query);
    println!("Memories retrieved: {}\n", memories.len());

    // Prepara contexto otimizado
    match orchestrator.prepare_context_for_llm(query, &memories, 4000) {
        Ok(prepared) => {
            println!("=== Prepared Context ===");
            println!("Token count: {}", prepared.token_count);
            println!("Groups created: {}", prepared.groups_count);
            println!("\n--- Context Content ---\n{}", prepared.context);
            println!("\n=== Token Reduction Summary ===");
            println!(
                "Original memories: {} characters",
                memories.iter().map(|m| m.content.len()).sum::<usize>()
            );
            println!("Optimized context: {} characters", prepared.context.len());
        }
        Err(e) => {
            eprintln!("Error preparing context: {}", e);
            std::process::exit(1);
        }
    }
}
