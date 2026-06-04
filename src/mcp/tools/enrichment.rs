// MCP tool definitions by domain.

    ToolDef {
        name: "memory_replay_at_time",
        description: "Replay one memory as it existed at a given RFC3339 timestamp and optionally return enrichment events affecting it up to that time.",
        schema: r#"{
            "type": "object",
            "properties": {
                "memory_id": {
                    "type": "integer",
                    "description": "ID of the memory to replay."
                },
                "timestamp": {
                    "type": "string",
                    "description": "RFC3339 timestamp to replay state at."
                },
                "event_type": {
                    "type": "string",
                    "description": "Optional event type filter for replayed event list (e.g. \"consolidation\")."
                },
                "include_events": {
                    "type": "boolean",
                    "default": true,
                    "description": "Whether to include enrichment events in the response."
                },
                "include_failed": {
                    "type": "boolean",
                    "default": false,
                    "description": "Whether to include failed enrichment events."
                },
                "include_dry_runs": {
                    "type": "boolean",
                    "default": false,
                    "description": "Whether to include dry-run events."
                },
                "event_limit": {
                    "type": "integer",
                    "description": "Max number of events to include in replay trail (default 50, max 200)."
                }
            },
            "required": ["memory_id", "timestamp"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
