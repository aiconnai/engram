// SessionEnd hook handler
// Triggered when a session ends - generates summaries and finalizes session.
// Also handles cross-session context injection for next session.

use std::collections::HashMap;

use super::{HookContext, HookResult};
use crate::storage::queries;
use crate::types::Memory;
use crate::Result;

/// Handler for SessionEnd hook
/// Generates session summaries, persists session data,
/// and prepares context injection for the next session.
pub struct SessionEndHandler {
    /// Whether to generate AI-powered session summaries
    pub generate_summary: bool,
    /// Whether to automatically save context for next session
    pub auto_save_next_session: bool,
    /// Whether to detect topics for consolidation
    pub auto_consolidate_topics: bool,
    /// Optional auto-consolidator for scheduling consolidations
    pub auto_consolidator: Option<crate::intelligence::AutoConsolidator>,
}

impl Default for SessionEndHandler {
    fn default() -> Self {
        Self {
            generate_summary: true,
            auto_save_next_session: true,
            auto_consolidate_topics: true,
            auto_consolidator: None,
        }
    }
}

impl SessionEndHandler {
    pub fn handle(&self, hook: super::LifecycleHook, context: &HookContext) -> Result<HookResult> {
        if hook != super::LifecycleHook::SessionEnd {
            return Ok(HookResult::Continue);
        }

        eprintln!(
            "[Hook] SessionEnd: session_id={:?}, workspace={:?}",
            context.session_id, context.workspace
        );

        let session_id = context.session_id.as_deref().unwrap_or_default();
        let next_session_id = context
            .metadata
            .get("next_session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // 1. Buscar memórias relevantes da sessão que terminou
        let relevant = self.search_session_memories(session_id);

        // 2. Gerar prompt de injeção para PRÓXIMA sessão
        if self.auto_save_next_session && !next_session_id.is_empty() {
            if let Ok(ref memories) = relevant {
                match self.build_injection_prompt(memories, next_session_id) {
                    Ok(injection) => {
                        // 3. Salvar contexto para próxima sessão (automático)
                        if let Err(e) = self.save_for_next_session(next_session_id, save_for_next_session(next_session_id, &injection) {injection) {
                            eprintln!("[Hook] Failed to save context for next session: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("[Hook] Failed to build injection prompt: {}", e);
                    }
                }
            }
        }

        // 4. Detectar tópicos aprendidos e agenda para consolidação
        if self.auto_consolidate_topics {
            if let Ok(ref memories) = relevant {
                let topics = self.extract_topics(memories);
                if let Ok(ref topic_list) = topics {
                    for topic in topic_list {
                        if self.should_consolidate_topic(&topic) {
                            match self.find_memories_by_topic(&topic) {
                                Ok(ids) => {
                                    if let Some(ref ac) = self.auto_consolidator {
                                        if let Err(e) = ac.schedule_consolidation(&ids) {
                                            eprintln!("[Hook] Failed to schedule consolidation for topic '{}': {}", topic, e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!(
                                        "[Hook] Failed to find memories for topic '{}': {}",
                                        topic, e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // 5. Gerar sumário da sessão se habilitado
        if self.generate_summary {
            eprintln!(
                "[Hook] Would generate session summary for session: {:?}",
                context.session_id
            );
            // TODO: Call intelligence layer to generate summary
        }

        Ok(HookResult::Continue)
    }

    /// Busca memórias relevantes da sessão
    fn search_session_memories(&self, session_id: &str) -> Result<Vec<Memory>> {
        // TODO: Implementar busca real usando o storage
        // Por agora, retorna vazio
        eprintln!("[Hook] Would search memories for session: {}", session_id);
        Ok(Vec::new())
    }

    /// Constrói prompt de injeção baseado nas memórias
    fn build_injection_prompt(&self, memories: &[Memory], next_session_id: &str) -> Result<String> {
        if memories.is_empty() {
            return Ok(
                "# Contexto da Sessão Anterior\n\n*(Nenhuma memória relevante encontrada)*"
                    .to_string(),
            );
        }

        let mut prompt = String::from("# Contexto da Sessão Anterior\n\n");
        prompt.push_str(&format!("Preparado para sessão: {}\n\n", next_session_id));

        for (i, memory) in memories.iter().enumerate() {
            prompt.push_str(&format!("## Memória {}\n", i + 1));
            prompt.push_str(&format!("Tipo: {:?}\n", memory.memory_type));
            prompt.push_str(&format!("Conteúdo: {}\n\n", memory.content));
        }

        Ok(prompt)
    }

    /// Salva contexto para próxima sessão (injeção automática)
    pub fn save_for_next_session(&self, session_id: &str, injection: &str) -> Result<()> {
        // TODO: Implementar salvamento real no storage
        // Isso poderia ser salvo como uma memória especial do tipo "Injection"
        // ou em uma tabela separada de contexto de sessão
        eprintln!(
            "[Hook] Would save injection prompt for next session {} ({} chars)",
            session_id,
            injection.len()
        );
        Ok(())
    }

    /// Extrai tópicos das memórias
    pub fn extract_topics(&self, memories: &[Memory]) -> Result<Vec<String>> {
        // TODO: Implementar extração real de tópicos (usando NLP ou tags)
        let mut topics = Vec::new();
        for memory in memories {
            // Por agora, usa as tags como tópicos
            topics.extend(memory.tags.iter().cloned());
        }
        topics.sort();
        topics.dedup();
        Ok(topics)
    }

    /// Decide se um tópico deve ser consolidado
    pub fn should_consolidate_topic(&self, topic: &str) -> bool {
        // TODO: Implementar lógica real (ex: verificar frequência, importância)
        // Por agora, consolida se o tópico tiver pelo menos 3 caracteres
        topic.len() >= 3
    }

    /// Encontra memórias por tópico
    fn find_memories_by_topic(&self, topic: &str) -> Result<Vec<i64>> {
        // TODO: Implementar busca real por tópico/tag
        eprintln!("[Hook] Would find memories for topic: {}", topic);
        Ok(Vec::new())
    }
}

pub fn create_handler(
) -> impl Fn(super::LifecycleHook, &HookContext) -> Result<HookResult> + Send + Sync {
    move |hook, context| {
        let handler = SessionEndHandler::default();
        handler.handle(hook, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_session_end_handler() {
        let handler = SessionEndHandler::default();
        let context = HookContext {
            session_id: Some("test-session-123".to_string()),
            workspace: Some("default".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };

        let result = handler.handle(crate::hooks::LifecycleHook::SessionEnd, &context);
        assert!(result.is_ok());
    }

    #[test]
    fn test_session_end_without_summary() {
        let handler = SessionEndHandler {
            generate_summary: false,
        };
        let context = HookContext {
            session_id: Some("test-session-456".to_string()),
            workspace: Some("default".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
            metadata: HashMap::new(),
        };

        let result = handler.handle(crate::hooks::LifecycleHook::SessionEnd, &context);
        assert!(result.is_ok());
    }
}
