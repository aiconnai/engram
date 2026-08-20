//! MCP tool definitions: Miscellaneous & Autonomous.
//! Dream phase, agent lifecycle, snapshots, attestations, harness memory, and routing.

use crate::mcp::protocol::ToolAnnotations;
use crate::mcp::tools::{ToolDef, ToolTier};

pub const TOOLS: &[ToolDef] = &[
    ToolDef {
            name: "dream_run_now",
            description: "Manually trigger the Dream Phase (background consolidation) across all workspaces or a specific workspace. Distills procedural rules, merges semantic duplicates, and emits thematic digests.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "description": "Optional workspace name to consolidate. Omit to run across all workspaces."},
                    "semantic_dedup_threshold": {"type": "number", "default": 0.92, "description": "Cosine similarity threshold for near-duplicate deduplication."},
                    "dry_run": {"type": "boolean", "default": false, "description": "When true, scans and reports without mutating database."}
                }
            }"#,
            annotations: ToolAnnotations::idempotent(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "dream_consolidation_status",
            description: "Retrieve memory consolidation status, counts of distilled procedural rules, archived duplicates, and tokens saved.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "default": "default", "description": "Workspace to inspect"}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "dream_insights",
            description: "Retrieve actionable distilled insights, procedural rules, and thematic summaries generated during consolidation passes.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "default": "default", "description": "Workspace to query insights for"}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "dream_create",
            description: "Create a reviewable dream snapshot job and optionally run deterministic candidate generation. Generated candidates are proposals, not canonical memories.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "default": "default"},
                    "job_id": {"type": "string", "description": "Optional stable job id. Omit to generate one."},
                    "instructions": {"type": "string"},
                    "run": {"type": "boolean", "default": true, "description": "When true, run deterministic generation immediately."},
                    "max_memories": {"type": "integer", "default": 50, "minimum": 1},
                    "max_candidates": {"type": "integer", "default": 25, "minimum": 1},
                    "summary_min_memories": {"type": "integer", "default": 2, "minimum": 1}
                }
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "dream_get",
            description: "Inspect one dream snapshot job.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "string", "description": "Dream job id"}
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "dream_list",
            description: "List dream snapshot jobs by workspace and status.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string"},
                    "status": {"type": "string", "enum": ["pending", "running", "completed", "failed", "canceled", "archived"]},
                    "limit": {"type": "integer", "default": 100, "minimum": 1, "maximum": 1000}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "dream_cancel",
            description: "Cancel a pending or running dream snapshot job idempotently.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "string", "description": "Dream job id"}
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::idempotent(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "dream_archive",
            description: "Archive a terminal dream snapshot job.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "string", "description": "Dream job id"}
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::idempotent(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "dream_candidates_list",
            description: "List review candidates emitted by dream snapshot jobs. Results are proposals and are not canonical memory facts.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string"},
                    "job_id": {"type": "string"},
                    "review_state": {"type": "string", "enum": ["pending", "accepted", "edited", "rejected", "applied", "archived"]},
                    "limit": {"type": "integer", "default": 100, "minimum": 1, "maximum": 1000}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "dream_candidate_get",
            description: "Inspect one dream candidate and its evidence sources.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "string", "description": "Dream candidate id"}
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "dream_candidate_review",
            description: "Review a dream candidate by accepting, editing, rejecting, or archiving it. This does not mutate canonical memory.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "string", "description": "Dream candidate id"},
                    "review_state": {"type": "string", "enum": ["accepted", "edited", "rejected", "archived"]},
                    "edited_content": {"type": "string", "description": "Reviewed replacement content when review_state is edited."},
                    "metadata_patch": {"type": "object", "description": "Optional review metadata merged into candidate metadata."}
                },
                "required": ["id", "review_state"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "dream_candidate_apply",
            description: "Apply an accepted or edited dream candidate to canonical memory. Requires confirm=true unless dry_run=true; repeated apply is idempotent.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "id": {"type": "string", "description": "Dream candidate id"},
                    "confirm": {"type": "boolean", "default": false, "description": "Must be true for canonical mutation."},
                    "dry_run": {"type": "boolean", "default": false, "description": "Preview planned canonical mutation without applying."}
                },
                "required": ["id"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_agent_writeback",
            description: "Create a pending agent-generated memory proposal as an agent_writeback dream candidate. Defaults to dry_run=true and never mutates canonical memory; review/apply still happens through dream_candidate_get/review/apply.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "proposed_content": {"type": "string", "minLength": 1, "description": "Agent-generated memory content to propose for human/agent review."},
                    "workspace": {"type": "string", "default": "default", "description": "Workspace for the pending candidate."},
                    "job_id": {"type": "string", "description": "Optional dream job id to group writeback candidates. Omit to generate one."},
                    "candidate_id": {"type": "string", "description": "Optional stable candidate id. Omit to generate one."},
                    "confidence": {"type": "number", "minimum": 0, "maximum": 1, "default": 0.5, "description": "Confidence in the proposed writeback."},
                    "reason_codes": {"type": "array", "items": {"type": "string"}, "description": "Reason codes for the pending candidate. Defaults to agent_writeback."},
                    "metadata": {"type": "object", "description": "Additional candidate metadata. Governance markers are added by Engram."},
                    "source_memory_ids": {
                        "type": "array",
                        "items": {"type": "integer", "minimum": 1},
                        "description": "Canonical memory ids that support this proposal."
                    },
                    "evidence": {
                        "type": "array",
                        "description": "Additional non-memory evidence sources. At least one source_memory_ids entry or evidence item is required.",
                        "items": {
                            "type": "object",
                            "properties": {
                                "source_type": {"type": "string", "minLength": 1},
                                "source_id": {"type": "string", "minLength": 1},
                                "source_ref": {"type": "string"},
                                "evidence": {"type": "object"}
                            },
                            "required": ["source_type", "source_id"]
                        }
                    },
                    "dry_run": {"type": "boolean", "default": true, "description": "Preview the pending candidate without writing dream_candidates."},
                    "confirm": {"type": "boolean", "default": false, "description": "Required with dry_run=false to create the pending candidate."}
                },
                "required": ["proposed_content"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "dream_eval_run",
            description: "Run deterministic local dream snapshot evaluation fixtures and return parseable CI-safe metrics. Does not require network, credentials, or model access.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "fixtures": {
                        "type": "array",
                        "items": {"type": "string", "enum": ["carry_forward_context", "preferences_constraints", "freshness_temporal", "provenance_correctness", "unsafe_raw_log_rejection", "no_canonical_mutation_before_apply"]},
                        "description": "Optional subset of fixed fixture names. Omit to run all fixtures."
                    },
                    "include_details": {"type": "boolean", "default": true, "description": "Include per-fixture candidate details."}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "agent_register",
            description: "Register an AI agent with capabilities and namespace isolation. Upserts if agent_id already exists.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string", "description": "Unique identifier for the agent"},
                    "display_name": {"type": "string", "description": "Human-readable name (defaults to agent_id)"},
                    "capabilities": {"type": "array", "items": {"type": "string"}, "description": "List of capabilities (e.g., 'search', 'create', 'analyze')"},
                    "namespaces": {"type": "array", "items": {"type": "string"}, "description": "Namespaces the agent operates in (default: ['default'])"},
                    "metadata": {"type": "object", "description": "Additional metadata as key-value pairs"}
                },
                "required": ["agent_id"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "agent_deregister",
            description: "Deregister an AI agent (soft delete — sets status to 'inactive').",
            schema: r#"{
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string", "description": "ID of the agent to deregister"}
                },
                "required": ["agent_id"]
            }"#,
            annotations: ToolAnnotations::destructive(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "agent_heartbeat",
            description: "Update an agent's heartbeat timestamp to indicate it is still alive.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string", "description": "ID of the agent sending heartbeat"}
                },
                "required": ["agent_id"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "agent_list",
            description: "List registered agents, optionally filtered by status or namespace.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "status": {"type": "string", "enum": ["active", "inactive"], "description": "Filter by agent status"},
                    "namespace": {"type": "string", "description": "Filter by namespace (returns agents that include this namespace)"}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "agent_get",
            description: "Get details of a specific registered agent by ID.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string", "description": "ID of the agent to retrieve"}
                },
                "required": ["agent_id"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "agent_capabilities",
            description: "Update the capabilities list of a registered agent.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "agent_id": {"type": "string", "description": "ID of the agent to update"},
                    "capabilities": {"type": "array", "items": {"type": "string"}, "description": "New capabilities list (replaces existing)"}
                },
                "required": ["agent_id", "capabilities"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "snapshot_create",
            description: "Create a portable .egm snapshot of memories filtered by workspace, tags, date range, or importance. Optionally encrypt with AES-256-GCM or sign with Ed25519.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "output_path": {"type": "string", "description": "File path for the .egm snapshot"},
                    "workspace": {"type": "string", "description": "Filter by workspace"},
                    "tags": {"type": "array", "items": {"type": "string"}, "description": "Filter by tags"},
                    "importance_min": {"type": "number", "description": "Minimum importance score"},
                    "memory_types": {"type": "array", "items": {"type": "string"}, "description": "Filter by memory types"},
                    "description": {"type": "string", "description": "Human-readable description"},
                    "creator": {"type": "string", "description": "Creator name"},
                    "encrypt_key": {"type": "string", "description": "Hex-encoded 32-byte AES key"},
                    "sign_key": {"type": "string", "description": "Hex-encoded 32-byte Ed25519 secret key"}
                },
                "required": ["output_path"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "snapshot_load",
            description: "Load a .egm snapshot into the memory store. Strategies: merge (skip duplicates), replace (clear workspace first), isolate (new workspace), dry_run (preview only).",
            schema: r#"{
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to .egm file"},
                    "strategy": {"type": "string", "enum": ["merge", "replace", "isolate", "dry_run"], "description": "Load strategy"},
                    "target_workspace": {"type": "string", "description": "Target workspace (defaults to snapshot's workspace)"},
                    "decrypt_key": {"type": "string", "description": "Hex-encoded 32-byte AES key for encrypted snapshots"}
                },
                "required": ["path", "strategy"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "snapshot_inspect",
            description: "Inspect a .egm snapshot without loading it. Returns manifest, file list, and size.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Path to .egm file"}
                },
                "required": ["path"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "attestation_log",
            description: "Log a document ingestion with cryptographic attestation. Creates a chained record proving the document was processed.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "content": {"type": "string", "description": "Document content to attest"},
                    "document_name": {"type": "string", "description": "Name of the document"},
                    "agent_id": {"type": "string", "description": "ID of the attesting agent"},
                    "memory_ids": {"type": "array", "items": {"type": "integer"}, "description": "IDs of memories created from this document"},
                    "sign_key": {"type": "string", "description": "Hex-encoded 32-byte Ed25519 secret key"}
                },
                "required": ["content", "document_name"]
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "attestation_verify",
            description: "Verify whether a document has been attested (ingested and recorded).",
            schema: r#"{
                "type": "object",
                "properties": {
                    "content": {"type": "string", "description": "Document content to verify"}
                },
                "required": ["content"]
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "attestation_chain_verify",
            description: "Verify the integrity of the entire attestation chain. Returns valid, broken (with location), or empty.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "verifying_key": {
                        "type": "string",
                        "description": "Hex-encoded 32-byte Ed25519 verifying key. When provided, every record must carry a valid signature; missing or invalid signatures cause the chain to report as Broken. When omitted, only hash-chain integrity is verified."
                    }
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "attestation_list",
            description: "List attestation records with optional filters. Supports JSON, CSV, and Merkle proof export formats.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "limit": {"type": "integer", "description": "Maximum records to return", "default": 50},
                    "offset": {"type": "integer", "description": "Number of records to skip", "default": 0},
                    "agent_id": {"type": "string", "description": "Filter by agent ID"},
                    "document_name": {"type": "string", "description": "Filter by document name"},
                    "export_format": {"type": "string", "enum": ["json", "csv", "merkle_proof"], "description": "Export format"}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Advanced,
        },
    ToolDef {
            name: "memory_agent_start",
            description: "Configure a tick-based memory agent for a workspace and return its initial configuration.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "description": "Workspace the agent will operate on (default: \"default\")"},
                    "interval_secs": {"type": "integer", "description": "Desired check interval in seconds (default: 300)"}
                },
                "required": []
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_agent_stop",
            description: "Stop a tick-based memory agent (no-op for stateless agents; resets client-side tracking).",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "description": "Workspace whose agent should be stopped (default: \"default\")"}
                },
                "required": []
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_agent_status",
            description: "Return current status and memory statistics for a workspace agent.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "description": "Workspace to report status for (default: \"default\")"}
                },
                "required": []
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
    ToolDef {
            name: "memory_agent_metrics",
            description: "Run one full agent cycle (prune/merge/archive) and return the actions taken and aggregate metrics. Mutates the database.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "description": "Workspace to run the agent cycle on (default: \"default\")"},
                    "max_actions": {"type": "integer", "description": "Maximum number of actions to take in this cycle (default: 10)"}
                },
                "required": []
            }"#,
            annotations: ToolAnnotations::mutating(),
            tier: ToolTier::Advanced,
        },
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
    ToolDef {
            name: "recent_activity",
            description: "Discover recently created or updated memories. Returns compact previews sorted by most recent activity. Useful for understanding what has changed recently.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "workspace": {"type": "string", "description": "Filter by workspace (omit for all workspaces)"},
                    "timeframe": {"type": "string", "enum": ["1h", "24h", "7d", "30d"], "default": "24h", "description": "Time window for activity"},
                    "limit": {"type": "integer", "minimum": 1, "maximum": 100, "default": 20, "description": "Max results to return"},
                    "include_types": {"type": "array", "items": {"type": "string"}, "description": "Only include these memory types"}
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Essential,
        },
    ToolDef {
            name: "discover_tools",
            description: "List Engram tools by tier, group, or search query. Includes feature-disabled tools with enablement hints so agents can progressively discover capabilities beyond the small default tools/list surface.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "tier": {"type": "string", "enum": ["essential", "standard", "advanced", "all"], "default": "all", "description": "Filter by tier: essential (small first-connect surface), standard (common workflows), advanced (specialized tools), all (everything)"},
                    "group": {"type": "string", "description": "Filter by structured group (e.g., 'memory.search', 'context', 'identity', 'feature.attestation')"},
                    "category": {"type": "string", "description": "Deprecated alias for group/search-style filtering."},
                    "search": {"type": "string", "description": "Search tool names and descriptions"},
                    "detail": {"type": "string", "enum": ["names", "summary", "schema"], "default": "summary", "description": "Per-tool detail level: 'names' (name only, cheapest), 'summary' (name + description + tier + group + availability + feature hints, the default), or 'schema' (summary plus the full input schema as a JSON object, so the tool can be called without a separate tools/list round-trip)."}
                }
            }"#,
            tier: ToolTier::Essential,
            annotations: ToolAnnotations::read_only(),
        },
    ToolDef {
            name: "model_routing_status",
            description: "Inspect active model provider availability, embedding dimensions, reranker health, and local vs cloud routing status.",
            schema: r#"{
                "type": "object",
                "properties": {
                    "model": {
                        "type": "string",
                        "description": "Optional provider name to inspect (e.g. tfidf, onnx, openai)."
                    },
                    "embedding_model": {
                        "type": "string",
                        "description": "Optional specific model ID."
                    },
                    "dimensions": {
                        "type": "integer",
                        "description": "Optional dimension configuration."
                    }
                }
            }"#,
            annotations: ToolAnnotations::read_only(),
            tier: ToolTier::Standard,
        },
];
