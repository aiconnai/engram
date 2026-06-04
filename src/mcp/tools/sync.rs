// MCP tool definitions by domain.

    // Sync
    ToolDef {
        name: "memory_sync_status",
        description: "Get cloud sync status",
        schema: r#"{"type": "object", "properties": {}}"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_sync_media",
        description: "Sync local media assets (images, audio, video) to cloud S3/R2 storage. Uploads files from media_assets table that have not yet been synced. Returns a report of synced files. Requires both multimodal and cloud features.",
        schema: r#"{
            "type": "object",
            "properties": {
                "dry_run": {"type": "boolean", "default": false, "description": "If true, report what would be synced without actually uploading"}
            }
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    // Event System
    ToolDef {
        name: "memory_events_poll",
        description: "Poll for memory events (create, update, delete, etc.) since a given point. Useful for syncing and monitoring.",
        schema: r#"{
            "type": "object",
            "properties": {
                "since_id": {"type": "integer", "description": "Return events after this event ID"},
                "since_time": {"type": "string", "format": "date-time", "description": "Return events after this timestamp (RFC3339)"},
                "agent_id": {"type": "string", "description": "Filter events for specific agent"},
                "limit": {"type": "integer", "default": 100, "description": "Maximum events to return"}
            }
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_events_clear",
        description: "Clear old events from the event log. Helps manage storage for long-running systems.",
        schema: r#"{
            "type": "object",
            "properties": {
                "before_id": {"type": "integer", "description": "Delete events before this ID"},
                "before_time": {"type": "string", "format": "date-time", "description": "Delete events before this timestamp"},
                "keep_recent": {"type": "integer", "description": "Keep only the N most recent events"}
            }
        }"#,
        annotations: ToolAnnotations::destructive(),
        tier: ToolTier::Advanced,
    },
    // Advanced Sync
    ToolDef {
        name: "sync_version",
        description: "Get the current sync version and metadata. Used to check if local data is up-to-date.",
        schema: r#"{
            "type": "object",
            "properties": {}
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "sync_delta",
        description: "Get changes (delta) since a specific version. Returns created, updated, and deleted memories.",
        schema: r#"{
            "type": "object",
            "properties": {
                "since_version": {"type": "integer", "description": "Version to get changes from"}
            },
            "required": ["since_version"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "sync_state",
        description: "Get or update sync state for a specific agent. Tracks what each agent has synced.",
        schema: r#"{
            "type": "object",
            "properties": {
                "agent_id": {"type": "string", "description": "Agent identifier"},
                "update_version": {"type": "integer", "description": "If provided, updates the agent's last synced version"}
            },
            "required": ["agent_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "sync_cleanup",
        description: "Clean up old sync data (events, etc.) older than specified days.",
        schema: r#"{
            "type": "object",
            "properties": {
                "older_than_days": {"type": "integer", "default": 30, "description": "Delete sync data older than this many days"}
            }
        }"#,
        annotations: ToolAnnotations::destructive(),
        tier: ToolTier::Advanced,
    },
    // Multi-Agent Sharing
    ToolDef {
        name: "memory_share",
        description: "Share a memory with another agent. The target agent can poll for shared memories.",
        schema: r#"{
            "type": "object",
            "properties": {
                "memory_id": {"type": "integer", "description": "ID of memory to share"},
                "from_agent": {"type": "string", "description": "Sender agent identifier"},
                "to_agent": {"type": "string", "description": "Recipient agent identifier"},
                "message": {"type": "string", "description": "Optional message to include with share"}
            },
            "required": ["memory_id", "from_agent", "to_agent"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_shared_poll",
        description: "Poll for memories shared with this agent.",
        schema: r#"{
            "type": "object",
            "properties": {
                "agent_id": {"type": "string", "description": "Agent identifier to check shares for"},
                "include_acknowledged": {"type": "boolean", "default": false, "description": "Include already acknowledged shares"}
            },
            "required": ["agent_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_share_ack",
        description: "Acknowledge receipt of a shared memory.",
        schema: r#"{
            "type": "object",
            "properties": {
                "share_id": {"type": "integer", "description": "Share ID to acknowledge"},
                "agent_id": {"type": "string", "description": "Agent acknowledging the share"}
            },
            "required": ["share_id", "agent_id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    // Scope-based access grants for multi-agent memory sharing
    ToolDef {
        name: "memory_grant_access",
        description: "Grant an agent access to a scope path. Supports read, write, and admin permissions. Access also applies to all descendant scopes.",
        schema: r#"{
            "type": "object",
            "properties": {
                "agent_id": {"type": "string", "description": "Agent ID to grant access to"},
                "scope_path": {"type": "string", "description": "Scope path to grant access to (e.g. 'global/org:acme')"},
                "permissions": {"type": "string", "enum": ["read", "write", "admin"], "default": "read", "description": "Permission level"},
                "granted_by": {"type": "string", "description": "Optional: ID of the granting agent"}
            },
            "required": ["agent_id", "scope_path"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_revoke_access",
        description: "Revoke an agent's access to a specific scope path.",
        schema: r#"{
            "type": "object",
            "properties": {
                "agent_id": {"type": "string", "description": "Agent ID to revoke access from"},
                "scope_path": {"type": "string", "description": "Scope path to revoke access from"}
            },
            "required": ["agent_id", "scope_path"]
        }"#,
        annotations: ToolAnnotations::destructive(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_list_grants",
        description: "List all scope access grants for a given agent.",
        schema: r#"{
            "type": "object",
            "properties": {
                "agent_id": {"type": "string", "description": "Agent ID to list grants for"}
            },
            "required": ["agent_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_check_access",
        description: "Check whether an agent has a required permission level on a scope path (including ancestor grants).",
        schema: r#"{
            "type": "object",
            "properties": {
                "agent_id": {"type": "string", "description": "Agent ID to check"},
                "scope_path": {"type": "string", "description": "Scope path to check access for"},
                "permissions": {"type": "string", "enum": ["read", "write", "admin"], "default": "read", "description": "Required permission level"}
            },
            "required": ["agent_id", "scope_path"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
