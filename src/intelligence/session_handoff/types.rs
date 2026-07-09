use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct SessionHandoffRequest {
    pub session_id: Option<String>,
    pub workspace: Option<String>,
    pub summary: Option<String>,
    pub current_goal: Option<String>,
    pub next_session_hints: Vec<String>,
    pub files_touched: Vec<String>,
    pub decisions_made: Vec<String>,
    pub tests_run: Vec<String>,
    pub tests_not_run: Vec<String>,
    pub known_risks: Vec<String>,
    pub blockers: Vec<String>,
    pub next_steps: Vec<String>,
    pub verification_evidence: Option<String>,
    pub issue_numbers: Vec<i64>,
    pub plan_doc_paths: Vec<String>,
    pub persist: bool,
    pub include_operational_context: bool,
    pub include_digest: bool,
}

impl Default for SessionHandoffRequest {
    fn default() -> Self {
        Self {
            session_id: None,
            workspace: None,
            summary: None,
            current_goal: None,
            next_session_hints: Vec::new(),
            files_touched: Vec::new(),
            decisions_made: Vec::new(),
            tests_run: Vec::new(),
            tests_not_run: Vec::new(),
            known_risks: Vec::new(),
            blockers: Vec::new(),
            next_steps: Vec::new(),
            verification_evidence: None,
            issue_numbers: Vec::new(),
            plan_doc_paths: Vec::new(),
            persist: true,
            include_operational_context: true,
            include_digest: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HandoffItem {
    pub title: String,
    pub detail: Option<String>,
    pub source_memory_id: Option<i64>,
    pub source_context_event_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionHandoffPacket {
    pub session_id: Option<String>,
    pub workspace: String,
    pub created_at: String,
    pub summary: String,
    pub current_goal: Option<String>,
    pub open_items: Vec<HandoffItem>,
    pub decisions: Vec<HandoffItem>,
    pub verification: Vec<HandoffItem>,
    pub risks: Vec<HandoffItem>,
    pub blockers: Vec<HandoffItem>,
    pub files_touched: Vec<String>,
    pub tests_run: Vec<String>,
    pub tests_not_run: Vec<String>,
    pub next_steps: Vec<String>,
    pub source_memory_ids: Vec<i64>,
    pub source_context_event_ids: Vec<i64>,
    pub warnings: Vec<String>,
    pub checkpoint_id: Option<i64>,
    pub copy_block: String,
}
