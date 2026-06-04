// MCP tool definitions by domain.

    // ── Harness Memory (cross-session continuity) ────────────────────────────
    ToolDef {
        name: "harness_record",
        description: "Record a durable harness event (decision, handoff, failed_attempt, verification_result, risk, assumption, bug_reproduction, issue_update) with structured metadata for cross-session continuity. Use instead of memory_create when capturing work-state evidence rather than facts.",
        schema: r#"{
            "type": "object",
            "properties": {
                "kind": {"type": "string", "enum": ["decision", "handoff", "failed_attempt", "bug_reproduction", "verification_result", "risk", "assumption", "issue_update"], "description": "The harness event kind"},
                "summary": {"type": "string", "maxLength": 500, "description": "Concise summary of the event (1-500 chars)"},
                "details": {"type": "string", "maxLength": 8000, "description": "Optional additional context appended to the summary"},
                "source_paths": {"type": "array", "items": {"type": "string"}, "description": "File paths relevant to this event"},
                "command": {"type": "string", "description": "CLI/shell command that produced this evidence"},
                "issue_number": {"type": "integer", "description": "Related GitHub issue number"},
                "commit_sha": {"type": "string", "description": "Related git commit SHA"},
                "evidence_refs": {"type": "array", "items": {"type": "string"}, "description": "Free-form references (URLs, paths, IDs)"},
                "importance": {"type": "number", "minimum": 0, "maximum": 1, "default": 0.7, "description": "Importance score (0-1)"},
                "workspace": {"type": "string", "description": "Workspace scope (default: 'default')"}
            },
            "required": ["kind", "summary"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "harness_status",
        description: "Assemble current project state from harness memory records and optional git state. Returns current objective, active issues, recent decisions, known blockers, last verification, last handoff, and a suggested next action. Token-budget aware; degrades gracefully when git is unavailable.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace scope (default: 'default')"},
                "max_records": {"type": "integer", "minimum": 1, "maximum": 50, "default": 10, "description": "Max recent harness records to include"},
                "token_budget": {"type": "integer", "default": 2000, "description": "Approximate max tokens for the output (chars/4 heuristic)"},
                "include_git": {"type": "boolean", "default": true, "description": "Attempt to collect git branch/status/log state"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "harness_handoff",
        description: "Generate a structured handoff packet for next-agent continuity: current goal, files touched, decisions, tests run/not run, risks, blockers, and next steps. Optionally persists as a harness record. Does NOT claim completion unless verification_evidence is provided.",
        schema: r#"{
            "type": "object",
            "properties": {
                "current_goal": {"type": "string", "maxLength": 300, "description": "What the agent was working toward"},
                "files_touched": {"type": "array", "items": {"type": "string"}, "description": "Paths modified this session"},
                "decisions_made": {"type": "array", "items": {"type": "string"}, "description": "Short decision summaries"},
                "tests_run": {"type": "array", "items": {"type": "string"}, "description": "Test commands/names that were run"},
                "tests_not_run": {"type": "array", "items": {"type": "string"}, "description": "Tests known to be missing or skipped"},
                "known_risks": {"type": "array", "items": {"type": "string"}, "description": "Open risks"},
                "blockers": {"type": "array", "items": {"type": "string"}, "description": "Things blocking progress"},
                "next_steps": {"type": "array", "items": {"type": "string"}, "minItems": 1, "description": "Recommended actions for the next agent"},
                "issue_numbers": {"type": "array", "items": {"type": "integer"}, "description": "Related GitHub issue numbers"},
                "plan_doc_paths": {"type": "array", "items": {"type": "string"}, "description": "Paths to relevant plan docs"},
                "verification_evidence": {"type": "string", "description": "Evidence that work is complete (test count, command output summary)"},
                "persist": {"type": "boolean", "default": true, "description": "Persist the handoff as a harness record"},
                "workspace": {"type": "string", "description": "Workspace scope (default: 'default')"}
            },
            "required": ["current_goal", "next_steps"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "harness_verify",
        description: "Record a verification command outcome with exit code, output summary, and optional evidence path/hash. Supports negative evidence (failures, skips with reason). Surfaces in harness_status as last_verification and feeds harness_handoff completion gating.",
        schema: r#"{
            "type": "object",
            "properties": {
                "command": {"type": "string", "maxLength": 200, "description": "The command that was run (e.g. 'cargo test --lib')"},
                "exit_code": {"type": "integer", "description": "Process exit code (0 = success)"},
                "passed": {"type": "boolean", "description": "Explicit pass/fail; derived from exit_code == 0 if omitted"},
                "output_summary": {"type": "string", "maxLength": 500, "description": "Concise summary (e.g. '873 tests passed, 0 failed')"},
                "evidence_path": {"type": "string", "description": "Path to the full output file or log"},
                "evidence_hash": {"type": "string", "description": "SHA-256 of the full output for integrity"},
                "skipped_reason": {"type": "string", "description": "If skipped, why (negative evidence)"},
                "issue_numbers": {"type": "array", "items": {"type": "integer"}, "description": "Linked GitHub issues"},
                "memory_ids": {"type": "array", "items": {"type": "integer"}, "description": "Linked memory IDs"},
                "importance": {"type": "number", "minimum": 0, "maximum": 1, "default": 0.8, "description": "Importance score (0-1)"},
                "workspace": {"type": "string", "description": "Workspace scope (default: 'default')"}
            },
            "required": ["command", "exit_code", "output_summary"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
