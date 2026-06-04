// MCP tool definitions by domain.

    // ── Agent Registry ────────────────────────────────────────────────────
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
    #[cfg(feature = "agent-portability")]
    // ── Snapshot Tools (agent-portability) ────────────────────────────────────
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
    #[cfg(feature = "agent-portability")]
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
    #[cfg(feature = "agent-portability")]
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
    #[cfg(feature = "agent-portability")]
    // ── Attestation Tools (agent-portability) ──────────────────────────────────
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
    #[cfg(feature = "agent-portability")]
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
    #[cfg(feature = "agent-portability")]
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
    #[cfg(feature = "agent-portability")]
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
    // === Reconciliation: previously-unadvertised tools (dispatch/registry parity) ===
    // -- reconciliation batch A --
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
