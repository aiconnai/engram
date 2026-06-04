// MCP tool definitions by domain.

    // Cross-references
    ToolDef {
        name: "memory_link",
        description: "Create a cross-reference between two memories",
        schema: r#"{
            "type": "object",
            "properties": {
                "from_id": {"type": "integer"},
                "to_id": {"type": "integer"},
                "edge_type": {"type": "string", "enum": ["related_to", "supersedes", "contradicts", "implements", "extends", "references", "depends_on", "blocks", "follows_up"], "default": "related_to"},
                "strength": {"type": "number", "minimum": 0, "maximum": 1, "description": "Relationship strength"},
                "source_context": {"type": "string", "description": "Why this link exists"},
                "pinned": {"type": "boolean", "default": false, "description": "Exempt from confidence decay"}
            },
            "required": ["from_id", "to_id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Essential,
    },
    ToolDef {
        name: "memory_unlink",
        description: "Remove a cross-reference",
        schema: r#"{
            "type": "object",
            "properties": {
                "from_id": {"type": "integer"},
                "to_id": {"type": "integer"},
                "edge_type": {"type": "string", "default": "related_to"}
            },
            "required": ["from_id", "to_id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_related",
        description: "Get memories related to a given memory (depth>1 or include_entities returns traversal result)",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Starting memory ID"},
                "depth": {"type": "integer", "default": 1, "description": "Traversal depth (1 = direct relations only)"},
                "include_entities": {"type": "boolean", "default": false, "description": "Include connections through shared entities"},
                "edge_type": {"type": "string", "description": "Filter by edge type"},
                "include_decayed": {"type": "boolean", "default": false}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Essential,
    },
    // Entity Extraction (RML-925)
    ToolDef {
        name: "memory_extract_entities",
        description: "Extract named entities (people, organizations, projects, concepts) from a memory and store them",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID to extract entities from"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::idempotent(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_get_entities",
        description: "Get all entities linked to a memory",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Memory ID"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_search_entities",
        description: "Search for entities by name prefix",
        schema: r#"{
            "type": "object",
            "properties": {
                "query": {"type": "string", "description": "Search query (prefix match)"},
                "entity_type": {"type": "string", "description": "Filter by entity type (person, organization, project, concept, etc.)"},
                "limit": {"type": "integer", "default": 20}
            },
            "required": ["query"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_entity_stats",
        description: "Get statistics about extracted entities",
        schema: r#"{
            "type": "object",
            "properties": {}
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    // Graph Traversal (RML-926)
    ToolDef {
        name: "memory_traverse",
        description: "Traverse the knowledge graph from a starting memory with full control over traversal options",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "Starting memory ID"},
                "depth": {"type": "integer", "default": 2, "description": "Maximum traversal depth"},
                "direction": {"type": "string", "enum": ["outgoing", "incoming", "both"], "default": "both"},
                "edge_types": {"type": "array", "items": {"type": "string"}, "description": "Filter by edge types (related_to, depends_on, etc.)"},
                "min_score": {"type": "number", "default": 0, "description": "Minimum edge score threshold"},
                "min_confidence": {"type": "number", "default": 0, "description": "Minimum confidence threshold"},
                "limit_per_hop": {"type": "integer", "default": 50, "description": "Max results per hop"},
                "include_entities": {"type": "boolean", "default": true, "description": "Include entity-based connections"}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Essential,
    },
    ToolDef {
        name: "memory_find_path",
        description: "Find the shortest path between two memories in the knowledge graph",
        schema: r#"{
            "type": "object",
            "properties": {
                "from_id": {"type": "integer", "description": "Starting memory ID"},
                "to_id": {"type": "integer", "description": "Target memory ID"},
                "max_depth": {"type": "integer", "default": 5, "description": "Maximum path length to search"}
            },
            "required": ["from_id", "to_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    #[cfg(feature = "duckdb-graph")]
    // ── DuckDB Graph Tools (duckdb-graph) ──────────────────────────────────────
    ToolDef {
        name: "memory_graph_path",
        description: "Finds how two entities are connected in the knowledge graph via DuckDB OLAP engine. Discovers hidden relationships across multiple hops using recursive path-finding.",
        schema: r#"{
            "type": "object",
            "properties": {
                "scope": {"type": "string", "description": "Tenant scope prefix, e.g., 'global/org/user'"},
                "source_id": {"type": "integer", "description": "Starting node ID"},
                "target_id": {"type": "integer", "description": "Target node ID"},
                "max_depth": {"type": "integer", "description": "Maximum hops to traverse (default: 4, max: 10)", "default": 4}
            },
            "required": ["scope", "source_id", "target_id"]
        }"#,
        annotations: ToolAnnotations {
            read_only_hint: Some(true),
            destructive_hint: None,
            idempotent_hint: Some(true),
            open_world_hint: None,
        },
        tier: ToolTier::Advanced,
    },
    #[cfg(feature = "duckdb-graph")]
    ToolDef {
        name: "memory_temporal_snapshot",
        description: "Retrieves the exact facts and relationships that were true at a specific historical point in time. Uses DuckDB OLAP engine for fast columnar scans over temporal edges.",
        schema: r#"{
            "type": "object",
            "properties": {
                "scope": {"type": "string", "description": "Tenant scope prefix"},
                "timestamp": {"type": "string", "description": "ISO-8601 timestamp for the point-in-time query"}
            },
            "required": ["scope", "timestamp"]
        }"#,
        annotations: ToolAnnotations {
            read_only_hint: Some(true),
            destructive_hint: None,
            idempotent_hint: Some(true),
            open_world_hint: None,
        },
        tier: ToolTier::Advanced,
    },
    #[cfg(feature = "duckdb-graph")]
    ToolDef {
        name: "memory_scope_snapshot",
        description: "Compares the knowledge graph between two timestamps, showing what relationships were added, removed, or changed. Uses DuckDB OLAP engine for efficient temporal diff.",
        schema: r#"{
            "type": "object",
            "properties": {
                "scope": {"type": "string", "description": "Tenant scope prefix"},
                "from_timestamp": {"type": "string", "description": "Start of comparison window (ISO-8601)"},
                "to_timestamp": {"type": "string", "description": "End of comparison window (ISO-8601)"}
            },
            "required": ["scope", "from_timestamp", "to_timestamp"]
        }"#,
        annotations: ToolAnnotations {
            read_only_hint: Some(true),
            destructive_hint: None,
            idempotent_hint: Some(true),
            open_world_hint: None,
        },
        tier: ToolTier::Advanced,
    },
    #[cfg(feature = "emergent-graph")]
    ToolDef {
        name: "memory_auto_link",
        description: "Run semantic and temporal auto-linker on a workspace, creating crossref edges in the database. Mutates the database.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace to auto-link (default: all workspaces)"},
                "similarity_threshold": {"type": "number", "description": "Minimum cosine similarity to create a semantic link (default: 0.75)"},
                "time_window_minutes": {"type": "integer", "description": "Time window in minutes for temporal linking (default: 30)"}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    #[cfg(feature = "emergent-graph")]
    ToolDef {
        name: "memory_auto_link_stats",
        description: "Return aggregate statistics about auto-generated semantic and temporal links.",
        schema: r#"{
            "type": "object",
            "properties": {},
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // -- reconciliation batch B --
    ToolDef {
        name: "memory_cluster",
        description: "Run Louvain community detection on the memory graph and return detected clusters with modularity score.",
        schema: r#"{
            "type": "object",
            "properties": {
                "min_cluster_size": {
                    "type": "integer",
                    "description": "Minimum number of members for a cluster to be reported (default: 2)."
                },
                "resolution": {
                    "type": "number",
                    "description": "Louvain resolution parameter controlling cluster granularity (default: 1.0)."
                },
                "link_types": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Restrict clustering to these edge/link types. Omit to use all link types."
                }
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Advanced,
    },
    ToolDef {
        name: "memory_coactivation_report",
        description: "Return coactivation graph statistics including edge count, average strength, and strongest co-occurring memory pairs.",
        schema: r#"{
            "type": "object",
            "properties": {},
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    ToolDef {
        name: "memory_fact_graph",
        description: "Return all stored subject-predicate-object facts for a given subject entity.",
        schema: r#"{
            "type": "object",
            "properties": {
                "subject": {
                    "type": "string",
                    "description": "Entity name to look up in the facts table."
                }
            },
            "required": ["subject"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 2. memory_garden
    ToolDef {
        name: "memory_garden",
        description: "Run full autonomous garden maintenance on a workspace: prunes stale memories, merges duplicates, archives cold entries, and compresses verbose content.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace to garden (default: \"default\")."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Advanced,
    },
    // 3. memory_garden_preview
    ToolDef {
        name: "memory_garden_preview",
        description: "Dry-run garden maintenance: reports what would be pruned, merged, archived, or compressed without making any changes.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace to preview gardening for (default: \"default\")."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    #[cfg(feature = "emergent-graph")]
    // 4. memory_get_cluster
    ToolDef {
        name: "memory_get_cluster",
        description: "Return the Louvain community cluster that contains a specific memory, including its cluster ID, size, and member IDs.",
        schema: r#"{
            "type": "object",
            "properties": {
                "memory_id": {"type": "integer", "description": "ID of the memory whose cluster to look up."}
            },
            "required": ["memory_id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 5. memory_knowledge_stats
    ToolDef {
        name: "memory_knowledge_stats",
        description: "Return aggregate statistics over the knowledge-graph facts table: total facts, unique subjects/predicates/objects, and top entities.",
        schema: r#"{
            "type": "object",
            "properties": {},
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    #[cfg(feature = "emergent-graph")]
    // 6. memory_list_auto_links
    ToolDef {
        name: "memory_list_auto_links",
        description: "List auto-generated graph links (semantic or temporal) between memories, optionally filtered by link type.",
        schema: r#"{
            "type": "object",
            "properties": {
                "link_type": {"type": "string", "description": "Filter by link type: \"semantic\" or \"temporal\". Omit for all types."},
                "limit": {"type": "integer", "description": "Maximum number of links to return (default: 50)."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    #[cfg(feature = "emergent-graph")]
    // 7. memory_list_clusters
    ToolDef {
        name: "memory_list_clusters",
        description: "List all detected memory clusters from the persistent cluster table, optionally selecting the detection algorithm.",
        schema: r#"{
            "type": "object",
            "properties": {
                "algorithm": {"type": "string", "description": "Clustering algorithm to filter by (default: \"louvain\")."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 8. memory_list_facts
    ToolDef {
        name: "memory_list_facts",
        description: "List extracted subject-predicate-object facts, optionally scoped to a single source memory.",
        schema: r#"{
            "type": "object",
            "properties": {
                "memory_id": {"type": "integer", "description": "Source memory ID to filter facts; omit to list facts from all memories."},
                "limit": {"type": "integer", "description": "Maximum number of facts to return (default: 100)."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 11. memory_query_triplets
    ToolDef {
        name: "memory_query_triplets",
        description: "SPARQL-like pattern query over the knowledge-graph facts table: match any combination of subject, predicate, and object (all optional, acts as wildcard when omitted).",
        schema: r#"{
            "type": "object",
            "properties": {
                "subject": {"type": "string", "description": "Subject entity to match (wildcard if omitted)."},
                "predicate": {"type": "string", "description": "Predicate/relation to match (wildcard if omitted)."},
                "object": {"type": "string", "description": "Object value to match (wildcard if omitted)."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 12. memory_reflect
    ToolDef {
        name: "memory_reflect",
        description: "Generate a reflective synthesis over a set of memories at a configurable analytical depth (surface, analytical, or meta).",
        schema: r#"{
            "type": "object",
            "properties": {
                "ids": {
                    "type": "array",
                    "items": {"type": "integer"},
                    "description": "Array of memory IDs to reflect on (required, must be non-empty)."
                },
                "depth": {"type": "string", "description": "Reflection depth: \"surface\" (default), \"analytical\", or \"meta\"."}
            },
            "required": ["ids"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 13. memory_resolve_conflict
    ToolDef {
        name: "memory_resolve_conflict",
        description: "Resolve a saved knowledge-graph conflict by ID using a chosen strategy, removing or retaining the conflicting edges accordingly.",
        schema: r#"{
            "type": "object",
            "properties": {
                "conflict_id": {"type": "integer", "description": "ID of the conflict record to resolve (required)."},
                "strategy": {"type": "string", "description": "Resolution strategy: \"keep_newer\" (default), \"keep_higher_confidence\", \"merge\", or \"manual\"."}
            },
            "required": ["conflict_id"]
        }"#,
        annotations: ToolAnnotations::mutating(),
        tier: ToolTier::Standard,
    },
    // 14. memory_sentiment_analyze
    ToolDef {
        name: "memory_sentiment_analyze",
        description: "Analyze the sentiment of a single memory's content, returning a score, label (positive/neutral/negative), confidence, and keyword signals.",
        schema: r#"{
            "type": "object",
            "properties": {
                "id": {"type": "integer", "description": "ID of the memory to analyze (required)."}
            },
            "required": ["id"]
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
    // 15. memory_sentiment_timeline
    ToolDef {
        name: "memory_sentiment_timeline",
        description: "Compute a chronological sentiment timeline over memories in a workspace within an optional time range.",
        schema: r#"{
            "type": "object",
            "properties": {
                "workspace": {"type": "string", "description": "Workspace to scan (default: \"default\")."},
                "from": {"type": "string", "description": "ISO-8601 start timestamp (default: epoch)."},
                "to": {"type": "string", "description": "ISO-8601 end timestamp (default: far future)."},
                "limit": {"type": "integer", "description": "Maximum number of timeline entries to return (default: 50)."}
            },
            "required": []
        }"#,
        annotations: ToolAnnotations::read_only(),
        tier: ToolTier::Standard,
    },
