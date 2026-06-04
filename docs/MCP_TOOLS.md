# MCP Tools Reference

<!-- GENERATED: do not edit manually. Run `./scripts/generate-mcp-reference.sh`. -->

This reference is generated from `src/mcp/tools.rs`.

Total tools: **256**

## Summary

| Tool | Tier | Annotations | Required Inputs |
|------|------|-------------|-----------------|
| `memory_create` | essential | mutating (no MCP hints) | `content` |
| `context_seed` | essential | mutating (no MCP hints) | `facts` |
| `memory_seed` | advanced | mutating (no MCP hints) | `facts` |
| `memory_get` | essential | readOnlyHint | `id` |
| `memory_update` | essential | mutating (no MCP hints) | `id` |
| `memory_delete` | essential | destructiveHint | `id` |
| `memory_list` | essential | readOnlyHint | none |
| `memory_search` | essential | readOnlyHint | `query` |
| `memory_smart_retrieve` | essential | readOnlyHint | `query` |
| `memory_council` | standard | mutating (no MCP hints) | `prompt` |
| `memory_search_suggest` | standard | readOnlyHint | `query` |
| `memory_link` | essential | mutating (no MCP hints) | `from_id`, `to_id` |
| `memory_unlink` | standard | mutating (no MCP hints) | `from_id`, `to_id` |
| `memory_related` | essential | readOnlyHint | `id` |
| `memory_create_todo` | standard | mutating (no MCP hints) | `content` |
| `memory_create_issue` | standard | mutating (no MCP hints) | `title` |
| `memory_versions` | advanced | readOnlyHint | `id` |
| `memory_set_expiration` | standard | mutating (no MCP hints) | `id`, `ttl_seconds` |
| `memory_cleanup_expired` | standard | destructiveHint | none |
| `memory_sync_status` | advanced | readOnlyHint | none |
| `memory_sync_media` | advanced | mutating (no MCP hints) | none |
| `memory_search_by_image` | advanced | readOnlyHint | `image_path` |
| `memory_stats` | essential | readOnlyHint | none |
| `memory_export_graph` | standard | readOnlyHint | none |
| `memory_find_duplicates` | standard | readOnlyHint | none |
| `memory_find_semantic_duplicates` | standard | readOnlyHint | none |
| `memory_scan_project` | advanced | mutating (no MCP hints) | none |
| `memory_get_project_context` | advanced | readOnlyHint | none |
| `memory_list_instruction_files` | advanced | readOnlyHint | none |
| `memory_extract_entities` | standard | idempotentHint | `id` |
| `memory_get_entities` | standard | readOnlyHint | `id` |
| `memory_search_entities` | standard | readOnlyHint | `query` |
| `memory_entity_stats` | advanced | readOnlyHint | none |
| `memory_traverse` | essential | readOnlyHint | `id` |
| `memory_find_path` | standard | readOnlyHint | `from_id`, `to_id` |
| `memory_ingest_document` | advanced | mutating (no MCP hints) | `path` |
| `dream_run_now` | advanced | idempotentHint | none |
| `workspace_list` | essential | readOnlyHint | none |
| `workspace_stats` | standard | readOnlyHint | `workspace` |
| `workspace_move` | standard | mutating (no MCP hints) | `id`, `workspace` |
| `workspace_delete` | advanced | destructiveHint | `workspace` |
| `memory_create_daily` | standard | mutating (no MCP hints) | `content` |
| `memory_promote_to_permanent` | standard | mutating (no MCP hints) | `id` |
| `embedding_cache_stats` | advanced | readOnlyHint | none |
| `embedding_cache_clear` | advanced | destructiveHint | none |
| `session_index` | essential | mutating (no MCP hints) | `messages`, `session_id` |
| `session_index_delta` | standard | mutating (no MCP hints) | `messages`, `session_id` |
| `session_get` | standard | readOnlyHint | `session_id` |
| `session_list` | essential | readOnlyHint | none |
| `session_delete` | standard | destructiveHint | `session_id` |
| `identity_create` | essential | mutating (no MCP hints) | `canonical_id`, `display_name` |
| `identity_get` | standard | readOnlyHint | `canonical_id` |
| `identity_update` | standard | mutating (no MCP hints) | `canonical_id` |
| `identity_delete` | standard | destructiveHint | `canonical_id` |
| `identity_add_alias` | standard | mutating (no MCP hints) | `alias`, `canonical_id` |
| `identity_remove_alias` | advanced | mutating (no MCP hints) | `alias` |
| `identity_resolve` | essential | readOnlyHint | `alias` |
| `identity_list` | standard | readOnlyHint | none |
| `identity_search` | standard | readOnlyHint | `query` |
| `identity_link` | standard | mutating (no MCP hints) | `canonical_id`, `memory_id` |
| `identity_unlink` | advanced | mutating (no MCP hints) | `canonical_id`, `memory_id` |
| `memory_get_identities` | advanced | readOnlyHint | `id` |
| `memory_soft_trim` | advanced | readOnlyHint | `id` |
| `memory_list_compact` | standard | readOnlyHint | none |
| `memory_content_stats` | advanced | readOnlyHint | `id` |
| `memory_create_batch` | standard | mutating (no MCP hints) | `memories` |
| `memory_delete_batch` | standard | destructiveHint | `ids` |
| `memory_ingest_fact` | standard | mutating (no MCP hints) | `fact` |
| `memory_ingest_fact_batch` | standard | mutating (no MCP hints) | `facts` |
| `memory_tags` | standard | readOnlyHint | none |
| `memory_tag_hierarchy` | advanced | readOnlyHint | none |
| `memory_validate_tags` | advanced | readOnlyHint | none |
| `memory_export` | advanced | readOnlyHint | none |
| `memory_import` | advanced | mutating (no MCP hints) | `data` |
| `memory_rebuild_embeddings` | advanced | idempotentHint | none |
| `memory_rebuild_crossrefs` | advanced | idempotentHint | none |
| `memory_create_section` | standard | mutating (no MCP hints) | `title` |
| `memory_checkpoint` | standard | mutating (no MCP hints) | `session_id`, `summary` |
| `memory_create_episodic` | standard | mutating (no MCP hints) | `content`, `event_time` |
| `memory_create_procedural` | standard | mutating (no MCP hints) | `content`, `trigger_pattern` |
| `memory_get_timeline` | standard | readOnlyHint | none |
| `memory_get_procedures` | standard | readOnlyHint | none |
| `memory_record_procedure_outcome` | standard | mutating (no MCP hints) | `id`, `success` |
| `memory_boost` | standard | mutating (no MCP hints) | `id` |
| `memory_explain_utility` | standard | readOnlyHint | `memory_id` |
| `memory_summarize` | advanced | mutating (no MCP hints) | `memory_ids` |
| `memory_get_full` | advanced | readOnlyHint | `id` |
| `context_budget_check` | advanced | readOnlyHint | `budget`, `memory_ids`, `model` |
| `memory_auto_consolidate` | advanced | mutating (no MCP hints) | `action` |
| `memory_consolidate_batch` | advanced | destructiveHint | none |
| `memory_consolidation_history` | advanced | readOnlyHint | none |
| `pending_injections_count` | advanced | readOnlyHint | none |
| `pending_injections_cleanup` | advanced | destructiveHint | none |
| `memory_archive_old` | advanced | destructiveHint | none |
| `langfuse_connect` | advanced | mutating (no MCP hints) | none |
| `langfuse_sync` | advanced | mutating (no MCP hints) | none |
| `langfuse_sync_status` | advanced | readOnlyHint | `task_id` |
| `langfuse_extract_patterns` | advanced | readOnlyHint | none |
| `memory_from_trace` | advanced | mutating (no MCP hints) | `trace_id` |
| `search_cache_feedback` | standard | mutating (no MCP hints) | `positive`, `query` |
| `search_cache_stats` | advanced | readOnlyHint | none |
| `search_cache_clear` | advanced | destructiveHint | none |
| `lifecycle_status` | standard | readOnlyHint | none |
| `lifecycle_run` | standard | idempotentHint | none |
| `memory_set_lifecycle` | advanced | mutating (no MCP hints) | `id`, `state` |
| `lifecycle_config` | advanced | readOnlyHint | none |
| `retention_policy_set` | standard | mutating (no MCP hints) | `workspace` |
| `retention_policy_get` | standard | readOnlyHint | `workspace` |
| `retention_policy_list` | standard | readOnlyHint | none |
| `retention_policy_delete` | advanced | destructiveHint | `workspace` |
| `retention_policy_apply` | advanced | idempotentHint | none |
| `memory_events_poll` | advanced | readOnlyHint | none |
| `memory_events_clear` | advanced | destructiveHint | none |
| `sync_version` | advanced | readOnlyHint | none |
| `sync_delta` | advanced | readOnlyHint | `since_version` |
| `sync_state` | advanced | readOnlyHint | `agent_id` |
| `sync_cleanup` | advanced | destructiveHint | none |
| `memory_share` | advanced | mutating (no MCP hints) | `from_agent`, `memory_id`, `to_agent` |
| `memory_shared_poll` | advanced | readOnlyHint | `agent_id` |
| `memory_share_ack` | advanced | mutating (no MCP hints) | `agent_id`, `share_id` |
| `memory_grant_access` | advanced | mutating (no MCP hints) | `agent_id`, `scope_path` |
| `memory_revoke_access` | advanced | destructiveHint | `agent_id`, `scope_path` |
| `memory_list_grants` | advanced | readOnlyHint | `agent_id` |
| `memory_check_access` | advanced | readOnlyHint | `agent_id`, `scope_path` |
| `memory_search_by_identity` | standard | mutating (no MCP hints) | `identity` |
| `memory_session_search` | standard | mutating (no MCP hints) | `query` |
| `memory_upload_image` | advanced | mutating (no MCP hints) | `file_path`, `memory_id` |
| `memory_migrate_images` | advanced | idempotentHint | none |
| `memory_suggest_tags` | advanced | readOnlyHint | none |
| `memory_auto_tag` | advanced | mutating (no MCP hints) | `id` |
| `salience_get` | advanced | readOnlyHint | `id` |
| `salience_set_importance` | advanced | mutating (no MCP hints) | `id`, `importance` |
| `salience_boost` | advanced | mutating (no MCP hints) | `id` |
| `salience_demote` | advanced | mutating (no MCP hints) | `id` |
| `salience_decay_run` | advanced | destructiveHint | none |
| `salience_stats` | advanced | readOnlyHint | none |
| `salience_history` | advanced | readOnlyHint | `id` |
| `salience_top` | advanced | readOnlyHint | none |
| `session_context_create` | standard | mutating (no MCP hints) | `name` |
| `session_context_add_memory` | advanced | mutating (no MCP hints) | `memory_id`, `session_id` |
| `session_context_remove_memory` | advanced | mutating (no MCP hints) | `memory_id`, `session_id` |
| `session_context_get` | standard | readOnlyHint | `session_id` |
| `session_context_list` | standard | readOnlyHint | none |
| `session_context_search` | standard | readOnlyHint | `query`, `session_id` |
| `session_context_update_summary` | advanced | mutating (no MCP hints) | `session_id`, `summary` |
| `session_context_end` | advanced | mutating (no MCP hints) | `session_id` |
| `session_context_export` | advanced | readOnlyHint | `session_id` |
| `quality_score` | standard | readOnlyHint | `id` |
| `quality_report` | standard | readOnlyHint | none |
| `quality_find_duplicates` | advanced | readOnlyHint | none |
| `quality_get_duplicates` | advanced | readOnlyHint | none |
| `quality_find_conflicts` | advanced | readOnlyHint | `id` |
| `quality_get_conflicts` | advanced | readOnlyHint | none |
| `quality_resolve_conflict` | advanced | destructiveHint | `conflict_id`, `resolution` |
| `quality_source_trust` | advanced | readOnlyHint | `source_type` |
| `quality_improve` | advanced | mutating (no MCP hints) | `id` |
| `meilisearch_search` | advanced | readOnlyHint | `query` |
| `meilisearch_reindex` | advanced | idempotentHint | none |
| `meilisearch_status` | advanced | readOnlyHint | none |
| `meilisearch_config` | advanced | readOnlyHint | none |
| `agent_register` | advanced | mutating (no MCP hints) | `agent_id` |
| `agent_deregister` | advanced | destructiveHint | `agent_id` |
| `agent_heartbeat` | advanced | mutating (no MCP hints) | `agent_id` |
| `agent_list` | advanced | readOnlyHint | none |
| `agent_get` | advanced | readOnlyHint | `agent_id` |
| `agent_capabilities` | advanced | mutating (no MCP hints) | `agent_id`, `capabilities` |
| `snapshot_create` | advanced | mutating (no MCP hints) | `output_path` |
| `snapshot_load` | advanced | mutating (no MCP hints) | `path`, `strategy` |
| `snapshot_inspect` | advanced | readOnlyHint | `path` |
| `attestation_log` | advanced | mutating (no MCP hints) | `content`, `document_name` |
| `attestation_verify` | advanced | readOnlyHint | `content` |
| `attestation_chain_verify` | advanced | readOnlyHint | none |
| `attestation_list` | advanced | readOnlyHint | none |
| `memory_graph_path` | advanced | readOnlyHint, idempotentHint | `scope`, `source_id`, `target_id` |
| `memory_temporal_snapshot` | advanced | readOnlyHint, idempotentHint | `scope`, `timestamp` |
| `memory_scope_snapshot` | advanced | readOnlyHint, idempotentHint | `from_timestamp`, `scope`, `to_timestamp` |
| `memory_get_public` | advanced | readOnlyHint | `id` |
| `memory_search_compact` | essential | readOnlyHint | `query` |
| `memory_expand` | essential | readOnlyHint | `ids` |
| `memory_get_injection_prompt` | essential | readOnlyHint | `query` |
| `memory_observe_tool_use` | standard | mutating (no MCP hints) | `tool_input`, `tool_name`, `tool_output` |
| `memory_archive_tool_output` | standard | mutating (no MCP hints) | `raw_output`, `tool_name` |
| `memory_get_archived_output` | standard | readOnlyHint | `archive_id` |
| `memory_get_working_memory` | standard | readOnlyHint | `session_id` |
| `session_land` | essential | mutating (no MCP hints) | `session_id` |
| `memory_build_context` | standard | readOnlyHint | `query` |
| `memory_export_markdown` | advanced | readOnlyHint | `workspace` |
| `recent_activity` | essential | readOnlyHint | none |
| `discover_tools` | essential | readOnlyHint | none |
| `memory_prepare_context` | advanced | readOnlyHint | `query` |
| `harness_record` | advanced | mutating (no MCP hints) | `kind`, `summary` |
| `harness_status` | advanced | readOnlyHint | none |
| `harness_handoff` | advanced | mutating (no MCP hints) | `current_goal`, `next_steps` |
| `harness_verify` | advanced | mutating (no MCP hints) | `command`, `exit_code`, `output_summary` |
| `memory_import_markdown` | advanced | mutating (no MCP hints) | `input_dir` |
| `memory_agent_start` | standard | readOnlyHint | none |
| `memory_agent_stop` | standard | readOnlyHint | none |
| `memory_agent_status` | standard | readOnlyHint | none |
| `memory_agent_metrics` | advanced | mutating (no MCP hints) | none |
| `memory_auto_link` | advanced | mutating (no MCP hints) | none |
| `memory_auto_link_stats` | standard | readOnlyHint | none |
| `memory_block_create` | standard | mutating (no MCP hints) | `name` |
| `memory_block_get` | standard | readOnlyHint | `name` |
| `memory_block_edit` | standard | mutating (no MCP hints) | `content`, `name` |
| `memory_block_list` | standard | readOnlyHint | none |
| `memory_block_archive` | standard | mutating (no MCP hints) | `name` |
| `memory_block_history` | standard | readOnlyHint | `name` |
| `memory_cache_stats` | standard | readOnlyHint | none |
| `memory_cache_clear` | advanced | mutating (no MCP hints) | none |
| `memory_capture_screenshot` | advanced | readOnlyHint | none |
| `memory_cluster` | advanced | readOnlyHint | none |
| `memory_coactivation_report` | standard | readOnlyHint | none |
| `memory_compress` | advanced | readOnlyHint | `id` |
| `memory_compress_for_context` | standard | readOnlyHint | `ids` |
| `memory_consolidate` | advanced | mutating (no MCP hints) | none |
| `memory_decompress` | standard | readOnlyHint | `id` |
| `memory_describe_image` | advanced | readOnlyHint | `image_path` |
| `memory_detect_conflicts` | standard | mutating (no MCP hints) | none |
| `memory_detect_updates` | standard | readOnlyHint | `content` |
| `memory_embedding_migrate` | advanced | mutating (no MCP hints) | none |
| `memory_embedding_providers` | standard | readOnlyHint | none |
| `memory_explain_search` | standard | readOnlyHint | `results` |
| `memory_extract_facts` | standard | mutating (no MCP hints) | `memory_id` |
| `memory_fact_graph` | standard | readOnlyHint | `subject` |
| `memory_feedback` | standard | mutating (no MCP hints) | `memory_id`, `query`, `signal` |
| `memory_feedback_stats` | standard | readOnlyHint | none |
| `memory_garden` | advanced | mutating (no MCP hints) | none |
| `memory_garden_preview` | standard | readOnlyHint | none |
| `memory_get_cluster` | standard | readOnlyHint | `memory_id` |
| `memory_knowledge_stats` | standard | readOnlyHint | none |
| `memory_list_auto_links` | standard | readOnlyHint | none |
| `memory_list_clusters` | standard | readOnlyHint | none |
| `memory_list_facts` | standard | readOnlyHint | none |
| `memory_list_media` | standard | readOnlyHint | none |
| `memory_process_video` | advanced | mutating (no MCP hints) | `video_path` |
| `memory_query_triplets` | standard | readOnlyHint | none |
| `memory_reflect` | standard | readOnlyHint | `ids` |
| `memory_resolve_conflict` | standard | mutating (no MCP hints) | `conflict_id` |
| `memory_sentiment_analyze` | standard | readOnlyHint | `id` |
| `memory_sentiment_timeline` | standard | readOnlyHint | none |
| `memory_suggest_acquisitions` | standard | readOnlyHint | none |
| `memory_synthesis` | standard | readOnlyHint | `content_a`, `content_b` |
| `memory_transcribe_audio` | advanced | readOnlyHint | `audio_path` |
| `memory_utility_score` | standard | readOnlyHint | `id` |
| `scope_get` | standard | readOnlyHint | `memory_id` |
| `scope_list` | standard | readOnlyHint | none |
| `scope_search` | standard | readOnlyHint | `query`, `scope_path` |
| `scope_set` | standard | mutating (no MCP hints) | `memory_id`, `scope_path` |
| `scope_tree` | standard | readOnlyHint | none |
| `temporal_add_edge` | advanced | mutating (no MCP hints) | `from_id`, `relation`, `to_id`, `valid_from` |
| `temporal_contradictions` | advanced | readOnlyHint | none |
| `temporal_diff` | advanced | readOnlyHint | `t1`, `t2` |
| `temporal_snapshot` | advanced | readOnlyHint | `timestamp` |
| `temporal_timeline` | advanced | readOnlyHint | `from_id`, `to_id` |
| `memory_enrichment_timeline` | standard | readOnlyHint | `memory_id` |
| `memory_enrichment_audit` | advanced | readOnlyHint | none |

## Tools

### `memory_create`

Store a new memory. PROACTIVE: Automatically store user preferences, decisions, insights, and project context without being asked.

- Tier: `essential`
- Annotations: mutating (no MCP hints)
- Required inputs: `content`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `content` | `string` | yes | The content to remember |
| `memory_type` | `string` | no | Memory type (preferred field; alias: type) Default: `note`. Allowed: `note`, `todo`, `issue`, `decision`, `preference`, `learning`, `context`, `credential`, `episodic`, `procedural`, `summary`, `checkpoint`, `image`, `audio`, `video`. |
| `type` | `string` | no | Deprecated alias for memory_type Default: `note`. Allowed: `note`, `todo`, `issue`, `decision`, `preference`, `learning`, `context`, `credential`, `episodic`, `procedural`, `summary`, `checkpoint`, `image`, `audio`, `video`. |
| `tags` | `array` | no | Tags for categorization Items: `string`. |
| `metadata` | `object` | no | Additional metadata as key-value pairs |
| `importance` | `number` | no | Importance score (0-1) Minimum: `0`. Maximum: `1`. |
| `workspace` | `string` | no | Workspace to store the memory in (default: 'default') |
| `tier` | `string` | no | Memory tier: permanent (never expires) or daily (auto-expires) Default: `permanent`. Allowed: `permanent`, `daily`. |
| `defer_embedding` | `boolean` | no | Defer embedding to background queue Default: `false`. |
| `ttl_seconds` | `integer` | no | Time-to-live in seconds. Memory will auto-expire after this duration. Omit for permanent storage. Setting this implies tier='daily'. |
| `dedup_mode` | `string` | no | How to handle duplicate content: reject (error if exact match), merge (combine tags/metadata with existing), skip (return existing unchanged), allow (create duplicate) Default: `allow`. Allowed: `reject`, `merge`, `skip`, `allow`. |
| `dedup_threshold` | `number` | no | Similarity threshold for semantic deduplication (0.0-1.0). When set with dedup_mode != 'allow', memories with cosine similarity >= threshold are treated as duplicates. Requires embeddings. If not set, only exact content hash matching is used. Minimum: `0`. Maximum: `1`. |
| `event_time` | `string` | no | ISO8601 timestamp for episodic memories (when the event occurred) Format: `date-time`. |
| `event_duration_seconds` | `integer` | no | Duration of the event in seconds (for episodic memories) |
| `trigger_pattern` | `string` | no | Pattern that triggers this procedure (for procedural memories) |
| `summary_of_id` | `integer` | no | ID of the memory this summarizes (for summary memories) |
| `media_url` | `string` | no | URL or local path to the primary media asset (for Image/Audio/Video memory types). Format: local:///path, https://..., or s3://... |

### `context_seed`

Injects initial context (premises, persona assumptions, or structured facts) about an entity to avoid cold start. Seeded memories are tagged as origin:seed and status:unverified, and should be treated as revisable assumptions.

- Tier: `essential`
- Annotations: mutating (no MCP hints)
- Required inputs: `facts`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `entity_context` | `string` | no | Name or ID of the entity (e.g., 'Client: Roberto', 'Account: ACME', 'Project: Alpha') Max length: `200`. |
| `workspace` | `string` | no | Workspace to store the memories in (default: 'default') |
| `base_tags` | `array` | no | Tags applied to all facts (e.g., ['vip', 'prospect']) Items: `string`. |
| `ttl_seconds` | `integer` | no | Override TTL for all facts in seconds (0 = disable TTL). If omitted, TTL is derived from confidence. |
| `disable_ttl` | `boolean` | no | Disable TTL and keep seeded memories permanent regardless of confidence. Default: `false`. |
| `facts` | `array` | yes | Items: `object`. Min items: `1`. |

### `memory_seed`

Deprecated alias for context_seed. Use context_seed instead.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `facts`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `entity_context` | `string` | no | Name or ID of the entity (e.g., 'Client: Roberto', 'Account: ACME', 'Project: Alpha') Max length: `200`. |
| `workspace` | `string` | no | Workspace to store the memories in (default: 'default') |
| `base_tags` | `array` | no | Tags applied to all facts (e.g., ['vip', 'prospect']) Items: `string`. |
| `ttl_seconds` | `integer` | no | Override TTL for all facts in seconds (0 = disable TTL). If omitted, TTL is derived from confidence. |
| `disable_ttl` | `boolean` | no | Disable TTL and keep seeded memories permanent regardless of confidence. Default: `false`. |
| `facts` | `array` | yes | Items: `object`. Min items: `1`. |

### `memory_get`

Retrieve a memory by its ID

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID |
| `strip_private` | `boolean` | no | When true, removes all <private>...</private> tagged sections from the content before returning (default: false) |

### `memory_update`

Update an existing memory

- Tier: `essential`
- Annotations: mutating (no MCP hints)
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID |
| `content` | `string` | no | New content |
| `memory_type` | `string` | no | Memory type (preferred field; alias: type) Allowed: `note`, `todo`, `issue`, `decision`, `preference`, `learning`, `context`, `credential`, `episodic`, `procedural`, `summary`, `checkpoint`, `image`, `audio`, `video`. |
| `type` | `string` | no | Deprecated alias for memory_type Allowed: `note`, `todo`, `issue`, `decision`, `preference`, `learning`, `context`, `credential`, `episodic`, `procedural`, `summary`, `checkpoint`, `image`, `audio`, `video`. |
| `tags` | `array` | no | Items: `string`. |
| `metadata` | `object` | no | No description. |
| `importance` | `number` | no | Minimum: `0`. Maximum: `1`. |
| `ttl_seconds` | `integer` | no | Time-to-live in seconds (0 = remove expiration, positive = set new expiration) |
| `event_time` | `string \| null` | no | ISO8601 timestamp for episodic memories (null to clear) Format: `date-time`. |
| `trigger_pattern` | `string \| null` | no | Pattern that triggers this procedure (null to clear) |
| `media_url` | `string \| null` | no | URL or local path to the primary media asset (null to clear) |

### `memory_delete`

Delete a memory (soft delete)

- Tier: `essential`
- Annotations: destructiveHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID |
| `cascade_chain` | `boolean` | no | When true, also delete all memories in the supersedes chain (ancestors this memory replaced). Default: `false`. |

### `memory_list`

List memories with filtering and pagination. Supports workspace isolation, tier filtering, and advanced filter syntax with AND/OR and comparison operators.

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `limit` | `integer` | no | Default: `20`. |
| `offset` | `integer` | no | Default: `0`. |
| `tags` | `array` | no | Items: `string`. |
| `memory_type` | `string` | no | Filter by memory type (preferred field; alias: type) |
| `type` | `string` | no | Deprecated alias for memory_type |
| `workspace` | `string` | no | Filter by single workspace |
| `workspaces` | `array` | no | Filter by multiple workspaces Items: `string`. |
| `tier` | `string` | no | Filter by memory tier Allowed: `permanent`, `daily`. |
| `sort_by` | `string` | no | Allowed: `created_at`, `updated_at`, `last_accessed_at`, `importance`, `access_count`. |
| `sort_order` | `string` | no | Default: `desc`. Allowed: `asc`, `desc`. |
| `filter` | `object` | no | Advanced filter with AND/OR logic and comparison operators. Supports workspace, tier, and metadata fields. Example: {"AND": [{"metadata.project": {"eq": "engram"}}, {"importance": {"gte": 0.5}}]}. Supported operators: eq, neq, gt, gte, lt, lte, contains, not_contains, exists. Fields: content, memory_type, importance, tags, workspace, tier, created_at, updated_at, metadata.* |
| `metadata_filter` | `object` | no | Legacy simple key-value filter (deprecated, use 'filter' instead) |

### `memory_search`

Search memories using hybrid search (keyword + semantic). Automatically selects optimal strategy with optional reranking. Supports workspace isolation, tier filtering, and advanced filters.

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: `query`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `query` | `string` | yes | Search query |
| `limit` | `integer` | no | Default: `10`. |
| `min_score` | `number` | no | Default: `0.1`. |
| `tags` | `array` | no | Items: `string`. |
| `memory_type` | `string` | no | Filter by memory type (preferred field; alias: type) |
| `type` | `string` | no | Deprecated alias for memory_type |
| `workspace` | `string` | no | Filter by single workspace |
| `workspaces` | `array` | no | Filter by multiple workspaces Items: `string`. |
| `tier` | `string` | no | Filter by memory tier Allowed: `permanent`, `daily`. |
| `include_transcripts` | `boolean` | no | Include transcript chunk memories (excluded by default) Default: `false`. |
| `strategy` | `string` | no | Force specific strategy (auto selects based on query; keyword/semantic are aliases for keyword_only/semantic_only) Allowed: `auto`, `keyword`, `keyword_only`, `semantic`, `semantic_only`, `hybrid`. |
| `explain` | `boolean` | no | Include match explanations Default: `false`. |
| `rerank` | `boolean` | no | Apply reranking to improve result quality Default: `true`. |
| `rerank_strategy` | `string` | no | Reranking strategy to use Default: `heuristic`. Allowed: `none`, `heuristic`, `multi_signal`. |
| `filter` | `object` | no | Advanced filter with AND/OR logic. Supports workspace, tier, and metadata fields. Example: {"AND": [{"workspace": {"eq": "my-project"}}, {"importance": {"gte": 0.5}}]} |
| `global` | `boolean` | no | Search across all workspaces (default: false). When true, ignores any workspace filter and returns results from all workspaces with a workspace field in each result. Default: `false`. |

### `memory_smart_retrieve`

Intent-aware unified retrieval. Classifies the query (lookup, exploration, context, path) and dispatches to the right combination of internal retrievers, then merges and dedupes results. Returns audit fields `intents_used` and `strategies_called`.

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: `query`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `query` | `string` | yes | Natural-language query |
| `limit` | `integer` | no | Default: `10`. Minimum: `1`. Maximum: `100`. |
| `workspace` | `string` | no | Optional workspace filter |
| `force_intents` | `array` | no | Override the classifier (for testing/debugging) Items: `string`. |

### `memory_council`

Run a question through an llm-council instance (Karpathy council orchestration) and return consolidated stage outputs and final answer. Optionally persist a checkpoint memory.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `prompt`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `prompt` | `string` | yes | Prompt to send to the council |
| `conversation_id` | `string` | no | Optional existing conversation ID to continue |
| `council_url` | `string` | no | Council HTTP base URL Default: `http://127.0.0.1:8001`. |
| `timeout_seconds` | `integer` | no | Request timeout in seconds (1-300) Default: `90`. Minimum: `1`. Maximum: `300`. |
| `include_raw_stages` | `boolean` | no | Whether to include raw stage payloads Default: `true`. |
| `persist` | `boolean` | no | Persist final answer as checkpoint memory Default: `false`. |
| `workspace` | `string` | no | Target workspace when persist=true Default: `default`. |
| `memory_tags` | `array` | no | Extra tags to include when persist=true (default tags: llm-council, consensus) Items: `string`. |

### `memory_search_suggest`

Get search suggestions and typo corrections

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `query`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `query` | `string` | yes | No description. |

### `memory_link`

Create a cross-reference between two memories

- Tier: `essential`
- Annotations: mutating (no MCP hints)
- Required inputs: `from_id`, `to_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `from_id` | `integer` | yes | No description. |
| `to_id` | `integer` | yes | No description. |
| `edge_type` | `string` | no | Default: `related_to`. Allowed: `related_to`, `supersedes`, `contradicts`, `implements`, `extends`, `references`, `depends_on`, `blocks`, `follows_up`. |
| `strength` | `number` | no | Relationship strength Minimum: `0`. Maximum: `1`. |
| `source_context` | `string` | no | Why this link exists |
| `pinned` | `boolean` | no | Exempt from confidence decay Default: `false`. |

### `memory_unlink`

Remove a cross-reference

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `from_id`, `to_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `from_id` | `integer` | yes | No description. |
| `to_id` | `integer` | yes | No description. |
| `edge_type` | `string` | no | Default: `related_to`. |

### `memory_related`

Get memories related to a given memory (depth>1 or include_entities returns traversal result)

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Starting memory ID |
| `depth` | `integer` | no | Traversal depth (1 = direct relations only) Default: `1`. |
| `include_entities` | `boolean` | no | Include connections through shared entities Default: `false`. |
| `edge_type` | `string` | no | Filter by edge type |
| `include_decayed` | `boolean` | no | Default: `false`. |

### `memory_create_todo`

Create a TODO memory with priority

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `content`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `content` | `string` | yes | No description. |
| `priority` | `string` | no | Default: `medium`. Allowed: `low`, `medium`, `high`, `critical`. |
| `due_date` | `string` | no | Format: `date`. |
| `tags` | `array` | no | Items: `string`. |

### `memory_create_issue`

Create an ISSUE memory for tracking problems

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `title`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `title` | `string` | yes | No description. |
| `description` | `string` | no | No description. |
| `severity` | `string` | no | Default: `medium`. Allowed: `low`, `medium`, `high`, `critical`. |
| `tags` | `array` | no | Items: `string`. |

### `memory_versions`

Get version history for a memory

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | No description. |

### `memory_set_expiration`

Set or update the expiration time for a memory

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `id`, `ttl_seconds`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID |
| `ttl_seconds` | `integer` | yes | Time-to-live in seconds from now. Use 0 to remove expiration (make permanent). |

### `memory_cleanup_expired`

Delete all expired memories. Typically called by a background job, but can be invoked manually.

- Tier: `standard`
- Annotations: destructiveHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_sync_status`

Get cloud sync status

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_sync_media`

Sync local media assets (images, audio, video) to cloud S3/R2 storage. Uploads files from media_assets table that have not yet been synced. Returns a report of synced files. Requires both multimodal and cloud features.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `dry_run` | `boolean` | no | If true, report what would be synced without actually uploading Default: `false`. |

### `memory_search_by_image`

Search memories using an image as the query. Uses multimodal embeddings (CLIP-style) or falls back to describing the image via vision model and searching by description. Returns semantically similar memories — text or media — ranked by relevance.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `image_path`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `image_path` | `string` | yes | Path to the local image file to use as the search query |
| `limit` | `integer` | no | Maximum number of results to return Default: `10`. |
| `min_score` | `number` | no | Minimum similarity score (0.0-1.0) Minimum: `0`. Maximum: `1`. |
| `workspace` | `string` | no | Restrict search to a specific workspace |
| `strategy` | `string` | no | Embedding strategy: clip (requires CLIP embedder), description (vision model + text embedding), auto (use CLIP if available, else description) Default: `auto`. Allowed: `clip`, `description`, `auto`. |

### `memory_stats`

Get storage statistics

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_export_graph`

Export knowledge graph visualization

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `format` | `string` | no | Default: `html`. Allowed: `html`, `json`. |
| `max_nodes` | `integer` | no | Default: `500`. |
| `focus_id` | `integer` | no | Center graph on this memory |

### `memory_find_duplicates`

Find potential duplicate memories

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `threshold` | `number` | no | Default: `0.9`. |

### `memory_find_semantic_duplicates`

Find semantically similar memories using embedding cosine similarity (LLM-powered dedup). Goes beyond hash/n-gram to detect paraphrased content.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `threshold` | `number` | no | Cosine similarity threshold (0.92 = very similar) Default: `0.92`. |
| `workspace` | `string` | no | Filter by workspace (optional) |
| `limit` | `integer` | no | Maximum duplicate pairs to return Default: `50`. |

### `memory_scan_project`

Scan current directory for AI instruction files (CLAUDE.md, AGENTS.md, .cursorrules, etc.) and ingest them as memories. Creates parent memory for each file and child memories for sections.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `path` | `string` | no | Directory to scan (defaults to current working directory) |
| `scan_parents` | `boolean` | no | Also scan parent directories (security: disabled by default) Default: `false`. |
| `extract_sections` | `boolean` | no | Create separate memories for each section Default: `true`. |

### `memory_get_project_context`

Get all project context memories for the current working directory. Returns instruction files and their sections.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `path` | `string` | no | Project path (defaults to current working directory) |
| `include_sections` | `boolean` | no | Include section memories Default: `true`. |
| `file_types` | `array` | no | Filter by file type (claude-md, cursorrules, etc.) Items: `string`. |

### `memory_list_instruction_files`

List AI instruction files (CLAUDE.md, AGENTS.md, .cursorrules, etc.) in a directory without ingesting them. Returns file paths, types, and sizes for discovery purposes.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `path` | `string` | no | Directory to scan (defaults to current working directory) |
| `scan_parents` | `boolean` | no | Also scan parent directories for instruction files Default: `false`. |

### `memory_extract_entities`

Extract named entities (people, organizations, projects, concepts) from a memory and store them

- Tier: `standard`
- Annotations: idempotentHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID to extract entities from |

### `memory_get_entities`

Get all entities linked to a memory

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID |

### `memory_search_entities`

Search for entities by name prefix

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `query`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `query` | `string` | yes | Search query (prefix match) |
| `entity_type` | `string` | no | Filter by entity type (person, organization, project, concept, etc.) |
| `limit` | `integer` | no | Default: `20`. |

### `memory_entity_stats`

Get statistics about extracted entities

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_traverse`

Traverse the knowledge graph from a starting memory with full control over traversal options

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Starting memory ID |
| `depth` | `integer` | no | Maximum traversal depth Default: `2`. |
| `direction` | `string` | no | Default: `both`. Allowed: `outgoing`, `incoming`, `both`. |
| `edge_types` | `array` | no | Filter by edge types (related_to, depends_on, etc.) Items: `string`. |
| `min_score` | `number` | no | Minimum edge score threshold Default: `0`. |
| `min_confidence` | `number` | no | Minimum confidence threshold Default: `0`. |
| `limit_per_hop` | `integer` | no | Max results per hop Default: `50`. |
| `include_entities` | `boolean` | no | Include entity-based connections Default: `true`. |

### `memory_find_path`

Find the shortest path between two memories in the knowledge graph

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `from_id`, `to_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `from_id` | `integer` | yes | Starting memory ID |
| `to_id` | `integer` | yes | Target memory ID |
| `max_depth` | `integer` | no | Maximum path length to search Default: `5`. |

### `memory_ingest_document`

Ingest a document (PDF or Markdown) into memory. Extracts text, splits into chunks with overlap, and creates memories with deduplication.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `path`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `path` | `string` | yes | Local file path to the document |
| `format` | `string` | no | Document format (auto-detect from extension if not specified) Default: `auto`. Allowed: `auto`, `md`, `pdf`. |
| `chunk_size` | `integer` | no | Maximum characters per chunk Default: `1200`. |
| `overlap` | `integer` | no | Overlap between chunks in characters Default: `200`. |
| `max_file_size` | `integer` | no | Maximum file size in bytes (default 10MB) Default: `10485760`. |
| `tags` | `array` | no | Additional tags to add to all chunks Items: `string`. |

### `dream_run_now`

Manually trigger the Dream Phase (background consolidation) across all workspaces. This process compresses old memories and identifies patterns while the agent is 'sleeping'.

- Tier: `advanced`
- Annotations: idempotentHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `workspace_list`

List all workspaces with their statistics (memory count, tier breakdown, etc.)

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `workspace_stats`

Get detailed statistics for a specific workspace

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `workspace`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | yes | Workspace name |

### `workspace_move`

Move a memory to a different workspace

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `id`, `workspace`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID to move |
| `workspace` | `string` | yes | Target workspace name |

### `workspace_delete`

Delete a workspace. Can either move all memories to 'default' workspace or hard delete them.

- Tier: `advanced`
- Annotations: destructiveHint
- Required inputs: `workspace`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | yes | Workspace to delete |
| `move_to_default` | `boolean` | no | If true, moves memories to 'default' workspace. If false, deletes all memories in the workspace. Default: `true`. |

### `memory_create_daily`

Create a daily (ephemeral) memory that auto-expires after the specified TTL. Useful for session context and scratch notes.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `content`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `content` | `string` | yes | The content to remember |
| `type` | `string` | no | Default: `note`. Allowed: `note`, `todo`, `issue`, `decision`, `preference`, `learning`, `context`, `credential`. |
| `tags` | `array` | no | Tags for categorization Items: `string`. |
| `metadata` | `object` | no | Additional metadata as key-value pairs |
| `importance` | `number` | no | Importance score (0-1) Minimum: `0`. Maximum: `1`. |
| `ttl_seconds` | `integer` | no | Time-to-live in seconds (default: 24 hours) Default: `86400`. |
| `workspace` | `string` | no | Workspace to store the memory in (default: 'default') |

### `memory_promote_to_permanent`

Promote a daily memory to permanent tier. Clears the expiration and makes the memory permanent.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID to promote |

### `embedding_cache_stats`

Get statistics about the embedding cache (hits, misses, entries, bytes used, hit rate)

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `embedding_cache_clear`

Clear all entries from the embedding cache

- Tier: `advanced`
- Annotations: destructiveHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `session_index`

Index a conversation into searchable memory chunks. Uses dual-limiter chunking (messages + characters) with overlap.

- Tier: `essential`
- Annotations: mutating (no MCP hints)
- Required inputs: `messages`, `session_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `session_id` | `string` | yes | Unique session identifier |
| `messages` | `array` | yes | Array of conversation messages Items: `object`. |
| `title` | `string` | no | Optional session title |
| `workspace` | `string` | no | Workspace to store chunks in (default: 'default') |
| `agent_id` | `string` | no | Optional agent identifier |
| `max_messages` | `integer` | no | Max messages per chunk Default: `10`. |
| `max_chars` | `integer` | no | Max characters per chunk Default: `8000`. |
| `overlap` | `integer` | no | Overlap messages between chunks Default: `2`. |
| `ttl_days` | `integer` | no | TTL for transcript chunks in days Default: `7`. |

### `session_index_delta`

Incrementally index new messages to an existing session. More efficient than full reindex.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `messages`, `session_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `session_id` | `string` | yes | Session to update |
| `messages` | `array` | yes | New messages to add Items: `object`. |

### `session_get`

Get information about an indexed session

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `session_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `session_id` | `string` | yes | Session ID to retrieve |

### `session_list`

List indexed sessions with optional workspace filter

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Filter by workspace |
| `limit` | `integer` | no | Maximum sessions to return Default: `20`. |

### `session_delete`

Delete a session and all its indexed chunks

- Tier: `standard`
- Annotations: destructiveHint
- Required inputs: `session_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `session_id` | `string` | yes | Session to delete |

### `identity_create`

Create a new identity with canonical ID, display name, and optional aliases

- Tier: `essential`
- Annotations: mutating (no MCP hints)
- Required inputs: `canonical_id`, `display_name`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `canonical_id` | `string` | yes | Unique canonical identifier (e.g., 'user:ronaldo', 'org:acme') |
| `display_name` | `string` | yes | Human-readable display name |
| `entity_type` | `string` | no | Default: `person`. Allowed: `person`, `organization`, `project`, `tool`, `concept`, `other`. |
| `description` | `string` | no | Optional description |
| `aliases` | `array` | no | Initial aliases for this identity Items: `string`. |
| `metadata` | `object` | no | Additional metadata |

### `identity_get`

Get an identity by its canonical ID

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `canonical_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `canonical_id` | `string` | yes | Canonical identifier |

### `identity_update`

Update an identity's display name, description, or type

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `canonical_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `canonical_id` | `string` | yes | Canonical identifier |
| `display_name` | `string` | no | New display name |
| `description` | `string` | no | New description |
| `entity_type` | `string` | no | Allowed: `person`, `organization`, `project`, `tool`, `concept`, `other`. |

### `identity_delete`

Delete an identity and all its aliases

- Tier: `standard`
- Annotations: destructiveHint
- Required inputs: `canonical_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `canonical_id` | `string` | yes | Canonical identifier to delete |

### `identity_add_alias`

Add an alias to an identity. Aliases are normalized (lowercase, trimmed). Conflicts with existing aliases are rejected.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `alias`, `canonical_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `canonical_id` | `string` | yes | Canonical identifier |
| `alias` | `string` | yes | Alias to add |
| `source` | `string` | no | Optional source of the alias (e.g., 'manual', 'extracted') |

### `identity_remove_alias`

Remove an alias from any identity

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `alias`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `alias` | `string` | yes | Alias to remove |

### `identity_resolve`

Resolve an alias to its canonical identity. Returns the identity if found, null otherwise.

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: `alias`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `alias` | `string` | yes | Alias to resolve |

### `identity_list`

List all identities with optional type filter

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `entity_type` | `string` | no | Allowed: `person`, `organization`, `project`, `tool`, `concept`, `other`. |
| `limit` | `integer` | no | Default: `50`. |

### `identity_search`

Search identities by alias or display name

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `query`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `query` | `string` | yes | Search query |
| `limit` | `integer` | no | Default: `20`. |

### `identity_link`

Link an identity to a memory (mark that the identity is mentioned in the memory)

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `canonical_id`, `memory_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `memory_id` | `integer` | yes | Memory ID |
| `canonical_id` | `string` | yes | Identity canonical ID |
| `mention_text` | `string` | no | The text that mentions this identity |

### `identity_unlink`

Remove the link between an identity and a memory

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `canonical_id`, `memory_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `memory_id` | `integer` | yes | Memory ID |
| `canonical_id` | `string` | yes | Identity canonical ID |

### `memory_get_identities`

Get all identities (persons, organizations, projects, etc.) linked to a memory. Returns identity details including display name, type, aliases, and mention information.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID |

### `memory_soft_trim`

Intelligently trim memory content while preserving context. Keeps the beginning (head) and end (tail) of content with an ellipsis in the middle. Useful for displaying long content in limited space while keeping important context from both ends.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID to trim |
| `max_chars` | `integer` | no | Maximum characters for trimmed output Default: `500`. |
| `head_percent` | `integer` | no | Percentage of space for the head (0-100) Default: `60`. |
| `tail_percent` | `integer` | no | Percentage of space for the tail (0-100) Default: `30`. |
| `ellipsis` | `string` | no | Text to insert between head and tail Default: ` ... `. |
| `preserve_words` | `boolean` | no | Avoid breaking in the middle of words Default: `true`. |

### `memory_list_compact`

List memories with compact preview instead of full content. More efficient for browsing/listing UIs. Returns only essential fields and a truncated content preview with metadata about original content length.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `limit` | `integer` | no | Maximum memories to return Default: `20`. |
| `offset` | `integer` | no | Pagination offset Default: `0`. |
| `tags` | `array` | no | Filter by tags Items: `string`. |
| `memory_type` | `string` | no | Filter by memory type (preferred field; alias: type) |
| `type` | `string` | no | Deprecated alias for memory_type |
| `workspace` | `string` | no | Filter by workspace |
| `tier` | `string` | no | Filter by tier Allowed: `permanent`, `daily`. |
| `sort_by` | `string` | no | Default: `created_at`. Allowed: `created_at`, `updated_at`, `last_accessed_at`, `importance`, `access_count`. |
| `sort_order` | `string` | no | Default: `desc`. Allowed: `asc`, `desc`. |
| `preview_chars` | `integer` | no | Maximum characters for content preview Default: `100`. |

### `memory_content_stats`

Get content statistics for a memory (character count, word count, line count, sentence count, paragraph count)

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID |

### `memory_create_batch`

Create multiple memories in a single operation. More efficient than individual creates for bulk imports.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `memories`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `memories` | `array` | yes | Array of memories to create Items: `object`. |

### `memory_delete_batch`

Delete multiple memories in a single operation.

- Tier: `standard`
- Annotations: destructiveHint
- Required inputs: `ids`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `ids` | `array` | yes | Array of memory IDs to delete Items: `integer`. |
| `cascade_chain` | `boolean` | no | When true, also delete all memories in the supersedes chain (ancestors this memory replaced). Default: `false`. |

### `memory_ingest_fact`

Append-only fact ingest for high-frequency sources (sessions, file watchers). Always inserts a new memory with memory_type='fact'. No dedup or upsert.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `fact`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `fact` | `string` | yes | The fact text to store |
| `source` | `string` | no | Origin identifier, e.g. 'session:abc' or 'watcher:/path/to/file' |
| `session_id` | `string` | no | Session ID stored in metadata.session_id |
| `workspace` | `string` | no | Workspace name (default: 'default') |
| `tags` | `array` | no | Optional tags Items: `string`. |
| `importance` | `number` | no | Importance score (default: 0.8) Minimum: `0`. Maximum: `1`. |
| `scope` | `string` | no | Memory scope (default: 'global') |

### `memory_ingest_fact_batch`

Batch append-only fact ingest. Inserts all facts in a single transaction. Returns count and ids.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `facts`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `facts` | `array` | yes | Array of fact objects to insert Items: `object`. |
| `workspace` | `string` | no | Default workspace applied to all facts (default: 'default') |
| `scope` | `string` | no | Memory scope applied to all facts (default: 'global') |

### `memory_tags`

List all tags with usage counts and most recent usage timestamps.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_tag_hierarchy`

Get tags organized in a hierarchical tree structure. Tags with slashes are treated as paths (e.g., 'project/engram/core').

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_validate_tags`

Validate tag consistency across memories. Reports orphaned tags, unused tags, and suggested normalizations.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_export`

Export all memories to a JSON-serializable format for backup or migration.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Optional: export only from specific workspace |
| `include_embeddings` | `boolean` | no | Include embedding vectors in export (larger file size) Default: `false`. |

### `memory_import`

Import memories from a previously exported JSON format.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `data`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `data` | `object` | yes | The exported data object |
| `skip_duplicates` | `boolean` | no | Skip memories with matching content hash Default: `true`. |

### `memory_rebuild_embeddings`

Rebuild embeddings for all memories that are missing them. Useful after model changes or data recovery.

- Tier: `advanced`
- Annotations: idempotentHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_rebuild_crossrefs`

Rebuild cross-reference links between memories. Re-analyzes all memories to find and create links.

- Tier: `advanced`
- Annotations: idempotentHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_create_section`

Create a section memory for organizing content hierarchically. Sections can have parent sections for nested organization.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `title`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `title` | `string` | yes | Section title |
| `content` | `string` | no | Section description or content |
| `parent_id` | `integer` | no | Optional parent section ID for nesting |
| `level` | `integer` | no | Heading level (1-6) Default: `1`. |
| `workspace` | `string` | no | Workspace for the section |

### `memory_checkpoint`

Create a checkpoint memory marking a significant point in a session. Useful for session resumption and context restoration.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `session_id`, `summary`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `session_id` | `string` | yes | Session identifier |
| `summary` | `string` | yes | Summary of session state at checkpoint |
| `context` | `object` | no | Additional context data to preserve |
| `workspace` | `string` | no | Workspace for the checkpoint |

### `memory_create_episodic`

Create an episodic memory representing an event with temporal context. Use for tracking when things happened and their duration.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `content`, `event_time`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `content` | `string` | yes | Description of the event |
| `event_time` | `string` | yes | ISO8601 timestamp when the event occurred Format: `date-time`. |
| `event_duration_seconds` | `integer` | no | Duration of the event in seconds |
| `tags` | `array` | no | Tags for categorization Items: `string`. |
| `metadata` | `object` | no | Additional metadata |
| `importance` | `number` | no | Importance score (0-1) Minimum: `0`. Maximum: `1`. |
| `workspace` | `string` | no | Workspace (default: 'default') |

### `memory_create_procedural`

Create a procedural memory representing a learned pattern or workflow. Tracks success/failure to measure effectiveness.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `content`, `trigger_pattern`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `content` | `string` | yes | Description of the procedure/workflow |
| `trigger_pattern` | `string` | yes | Pattern that triggers this procedure (e.g., 'When user asks about auth') |
| `tags` | `array` | no | Tags for categorization Items: `string`. |
| `metadata` | `object` | no | Additional metadata |
| `importance` | `number` | no | Importance score (0-1) Minimum: `0`. Maximum: `1`. |
| `workspace` | `string` | no | Workspace (default: 'default') |

### `memory_get_timeline`

Query episodic memories by time range. Returns events ordered by event_time.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `start_time` | `string` | no | Start of time range (ISO8601) Format: `date-time`. |
| `end_time` | `string` | no | End of time range (ISO8601) Format: `date-time`. |
| `workspace` | `string` | no | Filter by workspace |
| `tags` | `array` | no | Filter by tags Items: `string`. |
| `limit` | `integer` | no | Maximum results to return Default: `50`. |

### `memory_get_procedures`

List procedural memories (learned patterns/workflows). Optionally filter by trigger pattern.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `trigger_pattern` | `string` | no | Filter by trigger pattern (partial match) |
| `workspace` | `string` | no | Filter by workspace |
| `min_success_rate` | `number` | no | Minimum success rate (successes / (successes + failures)) Minimum: `0`. Maximum: `1`. |
| `limit` | `integer` | no | Maximum results to return Default: `50`. |

### `memory_record_procedure_outcome`

Record a success or failure for a procedural memory. Increments the corresponding counter.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `id`, `success`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Procedural memory ID |
| `success` | `boolean` | yes | true = success, false = failure |

### `memory_boost`

Temporarily boost a memory's importance score. The boost can optionally decay over time.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID to boost |
| `boost_amount` | `number` | no | Amount to increase importance (0-1) Default: `0.2`. |
| `duration_seconds` | `integer` | no | Optional: duration before boost decays (omit for permanent boost) |

### `memory_explain_utility`

Explain why a memory has its current utility score. Returns the full feedback history summary (useful vs. not-useful retrievals), how much temporal decay has been applied, and a plain-English narrative. Useful for debugging or auditing memory quality.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `memory_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `memory_id` | `integer` | yes | ID of the memory to explain |

### `memory_summarize`

Create a summary of one or more memories. Returns a new Summary-type memory with summary_of_id set.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `memory_ids`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `memory_ids` | `array` | yes | IDs of memories to summarize Items: `integer`. |
| `summary` | `string` | no | The summary text (provide this or let the system generate one) |
| `max_length` | `integer` | no | Maximum length for auto-generated summary Default: `500`. |
| `workspace` | `string` | no | Workspace for the summary memory |
| `tags` | `array` | no | Tags for the summary memory Items: `string`. |

### `memory_get_full`

Get the full/original content of a memory. If the memory is a Summary, returns the original content from summary_of_id.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID to get full content for |

### `context_budget_check`

Check token usage of memories against a budget. Returns token counts and suggestions if over budget.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `budget`, `memory_ids`, `model`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `memory_ids` | `array` | yes | IDs of memories to check Items: `integer`. |
| `model` | `string` | yes | Model name for tokenization (gpt-4, gpt-4o, gpt-4o-mini, claude-3-opus, etc.) |
| `encoding` | `string` | no | Override encoding (cl100k_base, o200k_base). Optional if model is known. |
| `budget` | `integer` | yes | Token budget to check against |

### `memory_auto_consolidate`

Enable, disable, configure, or inspect the automatic consolidation scheduler. Use action='enable'/'disable' to toggle it, 'set_interval' with interval_seconds to change the period (60–86400), or 'get_status' to inspect current settings.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `action`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `action` | `string` | yes | Allowed: `enable`, `disable`, `set_interval`, `get_status`. |
| `interval_seconds` | `integer` | no | Minimum: `60`. Maximum: `86400`. |

### `memory_consolidate_batch`

Run one auto-consolidation pass over a workspace: detect duplicates, conflicts, and archive-eligible memories. Defaults to dry-run; returns a structured report of actions taken (or that would be taken).

- Tier: `advanced`
- Annotations: destructiveHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Default: `default`. |
| `dry_run` | `boolean` | no | Default: `true`. |
| `policy` | `object` | no | No description. |

### `memory_consolidation_history`

List recent auto-consolidation runs for a workspace (or all workspaces). Newest-first.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | No description. |
| `limit` | `integer` | no | Default: `20`. Minimum: `1`. Maximum: `1000`. |

### `pending_injections_count`

Count of non-expired payloads queued in pending_injections for a workspace, waiting to be consumed by the next SessionStart.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Default: `default`. |

### `pending_injections_cleanup`

Drop every pending_injections row whose expires_at has passed. Idempotent. Returns the count removed.

- Tier: `advanced`
- Annotations: destructiveHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_archive_old`

Archive old, low-importance memories by creating summaries. Moves originals to archived state.

- Tier: `advanced`
- Annotations: destructiveHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `max_age_days` | `integer` | no | Archive memories older than this many days Default: `90`. |
| `max_importance` | `number` | no | Only archive memories with importance below this Default: `0.5`. |
| `min_access_count` | `integer` | no | Skip memories accessed more than this many times Default: `5`. |
| `workspace` | `string` | no | Limit to specific workspace |
| `dry_run` | `boolean` | no | If true, only report what would be archived Default: `true`. |

### `langfuse_connect`

Configure Langfuse connection for observability integration. Stores config in metadata.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `public_key` | `string` | no | Langfuse public key (or use LANGFUSE_PUBLIC_KEY env var) |
| `secret_key` | `string` | no | Langfuse secret key (or use LANGFUSE_SECRET_KEY env var) |
| `base_url` | `string` | no | Langfuse API base URL Default: `https://cloud.langfuse.com`. |

### `langfuse_sync`

Start background sync from Langfuse traces to memories. Returns task_id for status checking.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `since` | `string` | no | Sync traces since this timestamp (default: 24h ago) Format: `date-time`. |
| `limit` | `integer` | no | Maximum traces to sync Default: `100`. |
| `workspace` | `string` | no | Workspace to create memories in |
| `dry_run` | `boolean` | no | Preview what would be synced without creating memories Default: `false`. |

### `langfuse_sync_status`

Check the status of a Langfuse sync task.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `task_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `task_id` | `string` | yes | Task ID returned from langfuse_sync |

### `langfuse_extract_patterns`

Extract patterns from Langfuse traces without saving. Preview mode for pattern discovery.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `since` | `string` | no | Analyze traces since this timestamp Format: `date-time`. |
| `limit` | `integer` | no | Maximum traces to analyze Default: `50`. |
| `min_confidence` | `number` | no | Minimum confidence for patterns Default: `0.7`. |

### `memory_from_trace`

Create a memory from a specific Langfuse trace ID.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `trace_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `trace_id` | `string` | yes | Langfuse trace ID |
| `memory_type` | `string` | no | Type of memory to create Default: `episodic`. Allowed: `note`, `episodic`, `procedural`, `learning`. |
| `workspace` | `string` | no | Workspace for the memory |
| `tags` | `array` | no | Additional tags Items: `string`. |

### `search_cache_feedback`

Report feedback on search results quality. Helps tune the adaptive cache threshold.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `positive`, `query`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `query` | `string` | yes | The search query |
| `positive` | `boolean` | yes | True if results were helpful, false otherwise |
| `workspace` | `string` | no | Workspace filter used (if any) |

### `search_cache_stats`

Get search result cache statistics including hit rate, entry count, and current threshold.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `search_cache_clear`

Clear the search result cache. Useful after bulk operations.

- Tier: `advanced`
- Annotations: destructiveHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Only clear cache for this workspace (optional) |

### `lifecycle_status`

Get lifecycle statistics (active/stale/archived counts by workspace).

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Filter by workspace (optional) |

### `lifecycle_run`

Manually trigger a lifecycle cycle (mark stale, archive old). Dry run by default.

- Tier: `standard`
- Annotations: idempotentHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `dry_run` | `boolean` | no | Preview changes without applying Default: `true`. |
| `workspace` | `string` | no | Limit to specific workspace |
| `stale_days` | `integer` | no | Mark memories older than this as stale Default: `30`. |
| `archive_days` | `integer` | no | Archive memories older than this Default: `90`. |
| `min_importance` | `number` | no | Only process memories below this importance Default: `0.5`. |

### `memory_set_lifecycle`

Manually set the lifecycle state of a memory.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `id`, `state`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID |
| `state` | `string` | yes | New lifecycle state Allowed: `active`, `stale`, `archived`. |

### `lifecycle_config`

Get or set lifecycle configuration (intervals, thresholds).

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `stale_days` | `integer` | no | Days before marking as stale |
| `archive_days` | `integer` | no | Days before auto-archiving |
| `min_importance` | `number` | no | Importance threshold for lifecycle |
| `min_access_count` | `integer` | no | Access count threshold |

### `retention_policy_set`

Set a retention policy for a workspace. Controls auto-compression, max memory count, and auto-deletion.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `workspace`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | yes | Workspace name |
| `max_age_days` | `integer` | no | Hard age limit — auto-delete after this many days |
| `max_memories` | `integer` | no | Maximum active memories in this workspace |
| `compress_after_days` | `integer` | no | Auto-compress memories older than this |
| `compress_max_importance` | `number` | no | Only compress memories with importance <= this (default 0.3) |
| `compress_min_access` | `integer` | no | Skip compression if access_count >= this (default 3) |
| `auto_delete_after_days` | `integer` | no | Auto-delete archived memories older than this |
| `exclude_types` | `array` | no | Memory types exempt from policy (e.g. ["decision", "checkpoint"]) Items: `string`. |

### `retention_policy_get`

Get the retention policy for a workspace.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `workspace`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | yes | Workspace name |

### `retention_policy_list`

List all retention policies across all workspaces.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `retention_policy_delete`

Delete a retention policy for a workspace.

- Tier: `advanced`
- Annotations: destructiveHint
- Required inputs: `workspace`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | yes | Workspace name |

### `retention_policy_apply`

Apply all retention policies now. Compresses, caps, and deletes per workspace rules.

- Tier: `advanced`
- Annotations: idempotentHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `dry_run` | `boolean` | no | Preview what would happen without making changes Default: `false`. |

### `memory_events_poll`

Poll for memory events (create, update, delete, etc.) since a given point. Useful for syncing and monitoring.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `since_id` | `integer` | no | Return events after this event ID |
| `since_time` | `string` | no | Return events after this timestamp (RFC3339) Format: `date-time`. |
| `agent_id` | `string` | no | Filter events for specific agent |
| `limit` | `integer` | no | Maximum events to return Default: `100`. |

### `memory_events_clear`

Clear old events from the event log. Helps manage storage for long-running systems.

- Tier: `advanced`
- Annotations: destructiveHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `before_id` | `integer` | no | Delete events before this ID |
| `before_time` | `string` | no | Delete events before this timestamp Format: `date-time`. |
| `keep_recent` | `integer` | no | Keep only the N most recent events |

### `sync_version`

Get the current sync version and metadata. Used to check if local data is up-to-date.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `sync_delta`

Get changes (delta) since a specific version. Returns created, updated, and deleted memories.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `since_version`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `since_version` | `integer` | yes | Version to get changes from |

### `sync_state`

Get or update sync state for a specific agent. Tracks what each agent has synced.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `agent_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `agent_id` | `string` | yes | Agent identifier |
| `update_version` | `integer` | no | If provided, updates the agent's last synced version |

### `sync_cleanup`

Clean up old sync data (events, etc.) older than specified days.

- Tier: `advanced`
- Annotations: destructiveHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `older_than_days` | `integer` | no | Delete sync data older than this many days Default: `30`. |

### `memory_share`

Share a memory with another agent. The target agent can poll for shared memories.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `from_agent`, `memory_id`, `to_agent`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `memory_id` | `integer` | yes | ID of memory to share |
| `from_agent` | `string` | yes | Sender agent identifier |
| `to_agent` | `string` | yes | Recipient agent identifier |
| `message` | `string` | no | Optional message to include with share |

### `memory_shared_poll`

Poll for memories shared with this agent.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `agent_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `agent_id` | `string` | yes | Agent identifier to check shares for |
| `include_acknowledged` | `boolean` | no | Include already acknowledged shares Default: `false`. |

### `memory_share_ack`

Acknowledge receipt of a shared memory.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `agent_id`, `share_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `share_id` | `integer` | yes | Share ID to acknowledge |
| `agent_id` | `string` | yes | Agent acknowledging the share |

### `memory_grant_access`

Grant an agent access to a scope path. Supports read, write, and admin permissions. Access also applies to all descendant scopes.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `agent_id`, `scope_path`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `agent_id` | `string` | yes | Agent ID to grant access to |
| `scope_path` | `string` | yes | Scope path to grant access to (e.g. 'global/org:acme') |
| `permissions` | `string` | no | Permission level Default: `read`. Allowed: `read`, `write`, `admin`. |
| `granted_by` | `string` | no | Optional: ID of the granting agent |

### `memory_revoke_access`

Revoke an agent's access to a specific scope path.

- Tier: `advanced`
- Annotations: destructiveHint
- Required inputs: `agent_id`, `scope_path`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `agent_id` | `string` | yes | Agent ID to revoke access from |
| `scope_path` | `string` | yes | Scope path to revoke access from |

### `memory_list_grants`

List all scope access grants for a given agent.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `agent_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `agent_id` | `string` | yes | Agent ID to list grants for |

### `memory_check_access`

Check whether an agent has a required permission level on a scope path (including ancestor grants).

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `agent_id`, `scope_path`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `agent_id` | `string` | yes | Agent ID to check |
| `scope_path` | `string` | yes | Scope path to check access for |
| `permissions` | `string` | no | Required permission level Default: `read`. Allowed: `read`, `write`, `admin`. |

### `memory_search_by_identity`

Search memories by identity (person, entity, or alias). Finds all mentions of a specific identity across memories.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `identity`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `identity` | `string` | yes | Identity name or alias to search for |
| `workspace` | `string` | no | Optional: limit search to specific workspace |
| `limit` | `integer` | no | Maximum results to return Default: `50`. |

### `memory_session_search`

Search within session transcript chunks. Useful for finding content from past conversations.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `query`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `query` | `string` | yes | Search query |
| `session_id` | `string` | no | Optional: limit to specific session |
| `workspace` | `string` | no | Optional: limit to specific workspace |
| `limit` | `integer` | no | Maximum results to return Default: `20`. |

### `memory_upload_image`

Upload an image file and attach it to a memory. The image will be stored locally and linked to the memory's metadata.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `file_path`, `memory_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `memory_id` | `integer` | yes | ID of the memory to attach the image to |
| `file_path` | `string` | yes | Path to the image file to upload |
| `image_index` | `integer` | no | Index for ordering multiple images (0-based) Default: `0`. |
| `caption` | `string` | no | Optional caption for the image |

### `memory_migrate_images`

Migrate existing base64-encoded images in memories to file storage. Scans all memories and uploads any embedded data URIs to storage, replacing them with file references.

- Tier: `advanced`
- Annotations: idempotentHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `dry_run` | `boolean` | no | If true, only report what would be migrated without making changes Default: `false`. |

### `memory_suggest_tags`

Suggest tags for a memory based on AI content analysis. Uses pattern matching, keyword extraction, and structure detection to suggest relevant tags with confidence scores.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | no | Memory ID to analyze (alternative to content) |
| `memory_id` | `integer` | no | Memory ID to analyze (alias for id) |
| `content` | `string` | no | Content to analyze (alternative to id/memory_id) |
| `type` | `string` | no | Memory type (used when providing content directly) Allowed: `note`, `todo`, `issue`, `decision`, `preference`, `learning`, `context`, `credential`. |
| `existing_tags` | `array` | no | Tags already on the memory (excluded from suggestions) Items: `string`. |
| `min_confidence` | `number` | no | Minimum confidence threshold for suggestions Default: `0.5`. Minimum: `0`. Maximum: `1`. |
| `max_tags` | `integer` | no | Maximum number of tags to suggest Default: `5`. |
| `enable_patterns` | `boolean` | no | Use pattern-based tagging Default: `true`. |
| `enable_keywords` | `boolean` | no | Use keyword-based tagging Default: `true`. |
| `enable_entities` | `boolean` | no | Use entity-based tagging Default: `true`. |
| `enable_type_tags` | `boolean` | no | Add tags based on memory type Default: `true`. |
| `keyword_mappings` | `object` | no | Custom keyword-to-tag mappings (e.g., {"ibvi": "project/ibvi"}) |

### `memory_auto_tag`

Automatically suggest and optionally apply tags to a memory. Analyzes content using AI heuristics and can merge suggested tags with existing ones.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID to auto-tag |
| `memory_id` | `integer` | no | Memory ID (alias for id) |
| `apply` | `boolean` | no | If true, apply the suggested tags to the memory. If false, only return suggestions. Default: `false`. |
| `merge` | `boolean` | no | If true and apply=true, merge with existing tags. If false, replace existing tags. Default: `true`. |
| `min_confidence` | `number` | no | Minimum confidence threshold Default: `0.5`. Minimum: `0`. Maximum: `1`. |
| `max_tags` | `integer` | no | Maximum tags to suggest/apply Default: `5`. |
| `keyword_mappings` | `object` | no | Custom keyword-to-tag mappings |

### `salience_get`

Get the salience score for a memory. Returns recency, frequency, importance, and feedback components with the combined score and lifecycle state.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID to get salience for |
| `feedback_signal` | `number` | no | Optional feedback signal (-1 to 1) to include in calculation Default: `0`. Minimum: `-1`. Maximum: `1`. |

### `salience_set_importance`

Set the importance score for a memory. This is the static importance component of salience.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `id`, `importance`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID |
| `importance` | `number` | yes | Importance score (0-1) Minimum: `0`. Maximum: `1`. |

### `salience_boost`

Boost a memory's salience score temporarily or permanently. Useful for marking memories as contextually relevant.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID to boost |
| `boost_amount` | `number` | no | Amount to boost (0-1) Default: `0.2`. Minimum: `0`. Maximum: `1`. |
| `reason` | `string` | no | Optional reason for boosting |

### `salience_demote`

Demote a memory's salience score. Useful for marking memories as less relevant.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID to demote |
| `demote_amount` | `number` | no | Amount to demote (0-1) Default: `0.2`. Minimum: `0`. Maximum: `1`. |
| `reason` | `string` | no | Optional reason for demoting |

### `salience_decay_run`

Run temporal decay on all memories. Updates lifecycle states (Active → Stale → Archived) based on salience scores.

- Tier: `advanced`
- Annotations: destructiveHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `dry_run` | `boolean` | no | If true, compute changes without persisting updates Default: `false`. |
| `record_history` | `boolean` | no | Record salience history entries while updating Default: `true`. |
| `workspace` | `string` | no | Limit to specific workspace |
| `stale_threshold_days` | `integer` | no | Days of inactivity before marking stale Minimum: `1`. |
| `archive_threshold_days` | `integer` | no | Days of inactivity before suggesting archive Minimum: `1`. |

### `salience_stats`

Get salience statistics across all memories. Returns distribution, percentiles, and state counts.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Limit to specific workspace |

### `salience_history`

Get salience score history for a memory. Shows how salience has changed over time.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID |
| `limit` | `integer` | no | Maximum history entries to return Default: `50`. |

### `salience_top`

Get top memories by salience score. Useful for context injection.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `limit` | `integer` | no | Maximum memories to return Default: `20`. |
| `workspace` | `string` | no | Limit to specific workspace |
| `min_score` | `number` | no | Minimum salience score Minimum: `0`. Maximum: `1`. |
| `memory_type` | `string` | no | Filter by memory type |

### `session_context_create`

Create a new session context for tracking related memories during a conversation or task.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `name`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `name` | `string` | yes | Session name |
| `description` | `string` | no | Session description |
| `workspace` | `string` | no | Workspace for the session |
| `metadata` | `object` | no | Additional session metadata |

### `session_context_add_memory`

Add a memory to a session context with relevance score and role.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `memory_id`, `session_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `session_id` | `string` | yes | Session ID |
| `memory_id` | `integer` | yes | Memory ID to add |
| `relevance_score` | `number` | no | How relevant this memory is to the session Default: `1.0`. Minimum: `0`. Maximum: `1`. |
| `context_role` | `string` | no | Role of the memory in the session Default: `referenced`. Allowed: `referenced`, `created`, `updated`, `pinned`. |

### `session_context_remove_memory`

Remove a memory from a session context.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `memory_id`, `session_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `session_id` | `string` | yes | Session ID |
| `memory_id` | `integer` | yes | Memory ID to remove |

### `session_context_get`

Get a session context with its linked memories.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `session_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `session_id` | `string` | yes | Session ID |

### `session_context_list`

List all session contexts with optional filtering.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Filter by workspace |
| `active_only` | `boolean` | no | Only return active sessions Default: `false`. |
| `limit` | `integer` | no | Maximum sessions to return Default: `50`. |
| `offset` | `integer` | no | Offset for pagination Default: `0`. |

### `session_context_search`

Search memories within a specific session context.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `query`, `session_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `session_id` | `string` | yes | Session ID to search within |
| `query` | `string` | yes | Search query |
| `limit` | `integer` | no | Maximum results Default: `20`. |

### `session_context_update_summary`

Update the summary of a session context.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `session_id`, `summary`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `session_id` | `string` | yes | Session ID |
| `summary` | `string` | yes | New session summary |

### `session_context_end`

End a session context, marking it as inactive.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `session_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `session_id` | `string` | yes | Session ID to end |
| `summary` | `string` | no | Optional final summary |

### `session_context_export`

Export a session context with all its memories for archival or sharing.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `session_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `session_id` | `string` | yes | Session ID to export |
| `include_content` | `boolean` | no | Include full memory content Default: `true`. |
| `format` | `string` | no | Export format Default: `json`. Allowed: `json`, `markdown`. |

### `quality_score`

Get the quality score for a memory with detailed breakdown of clarity, completeness, freshness, consistency, and source trust components.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID to score |

### `quality_report`

Generate a comprehensive quality report for a workspace. Includes quality distribution, top issues, conflict and duplicate counts.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Workspace to analyze (default: 'default') |

### `quality_find_duplicates`

Find near-duplicate memories using text similarity. Returns pairs of similar memories above the threshold.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `threshold` | `number` | no | Similarity threshold (0-1) Default: `0.85`. Minimum: `0`. Maximum: `1`. |
| `limit` | `integer` | no | Maximum memories to compare Default: `100`. |

### `quality_get_duplicates`

Get pending duplicate candidates that need review.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `limit` | `integer` | no | Maximum duplicates to return Default: `50`. |

### `quality_find_conflicts`

Detect conflicts for a memory against existing memories. Finds contradictions, staleness, and semantic overlaps.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID to check for conflicts |

### `quality_get_conflicts`

Get unresolved conflicts that need attention.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `limit` | `integer` | no | Maximum conflicts to return Default: `50`. |

### `quality_resolve_conflict`

Resolve a conflict between memories. Options: keep_a, keep_b, merge, keep_both, delete_both, false_positive.

- Tier: `advanced`
- Annotations: destructiveHint
- Required inputs: `conflict_id`, `resolution`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `conflict_id` | `integer` | yes | Conflict ID to resolve |
| `resolution` | `string` | yes | How to resolve the conflict Allowed: `keep_a`, `keep_b`, `merge`, `keep_both`, `delete_both`, `false_positive`. |
| `notes` | `string` | no | Optional notes about the resolution |

### `quality_source_trust`

Get or update trust score for a source type. Higher trust means memories from this source are weighted more in quality calculations.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `source_type`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `source_type` | `string` | yes | Source type (user, seed, extraction, inference, external) |
| `source_identifier` | `string` | no | Optional specific source identifier |
| `trust_score` | `number` | no | New trust score (omit to just get current score) Minimum: `0`. Maximum: `1`. |
| `notes` | `string` | no | Notes about this source |

### `quality_improve`

Get suggestions for improving a memory's quality. Returns actionable recommendations.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID to analyze |

### `meilisearch_search`

Search memories using Meilisearch (typo-tolerant, fast full-text). Requires Meilisearch to be configured. Falls back to hybrid search if unavailable.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `query`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `query` | `string` | yes | Search query text |
| `limit` | `integer` | no | Maximum results to return Default: `20`. |
| `offset` | `integer` | no | Number of results to skip Default: `0`. |
| `workspace` | `string` | no | Filter by workspace |
| `tags` | `array` | no | Filter by tags (AND logic) Items: `string`. |
| `memory_type` | `string` | no | Filter by memory type |

### `meilisearch_reindex`

Trigger a full re-sync from SQLite to Meilisearch. Use after bulk imports or if the index is out of sync.

- Tier: `advanced`
- Annotations: idempotentHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `meilisearch_status`

Get Meilisearch index status including document count, indexing state, and health.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `meilisearch_config`

Show current Meilisearch configuration (URL, sync interval, enabled status).

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `agent_register`

Register an AI agent with capabilities and namespace isolation. Upserts if agent_id already exists.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `agent_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `agent_id` | `string` | yes | Unique identifier for the agent |
| `display_name` | `string` | no | Human-readable name (defaults to agent_id) |
| `capabilities` | `array` | no | List of capabilities (e.g., 'search', 'create', 'analyze') Items: `string`. |
| `namespaces` | `array` | no | Namespaces the agent operates in (default: ['default']) Items: `string`. |
| `metadata` | `object` | no | Additional metadata as key-value pairs |

### `agent_deregister`

Deregister an AI agent (soft delete — sets status to 'inactive').

- Tier: `advanced`
- Annotations: destructiveHint
- Required inputs: `agent_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `agent_id` | `string` | yes | ID of the agent to deregister |

### `agent_heartbeat`

Update an agent's heartbeat timestamp to indicate it is still alive.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `agent_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `agent_id` | `string` | yes | ID of the agent sending heartbeat |

### `agent_list`

List registered agents, optionally filtered by status or namespace.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `status` | `string` | no | Filter by agent status Allowed: `active`, `inactive`. |
| `namespace` | `string` | no | Filter by namespace (returns agents that include this namespace) |

### `agent_get`

Get details of a specific registered agent by ID.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `agent_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `agent_id` | `string` | yes | ID of the agent to retrieve |

### `agent_capabilities`

Update the capabilities list of a registered agent.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `agent_id`, `capabilities`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `agent_id` | `string` | yes | ID of the agent to update |
| `capabilities` | `array` | yes | New capabilities list (replaces existing) Items: `string`. |

### `snapshot_create`

Create a portable .egm snapshot of memories filtered by workspace, tags, date range, or importance. Optionally encrypt with AES-256-GCM or sign with Ed25519.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `output_path`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `output_path` | `string` | yes | File path for the .egm snapshot |
| `workspace` | `string` | no | Filter by workspace |
| `tags` | `array` | no | Filter by tags Items: `string`. |
| `importance_min` | `number` | no | Minimum importance score |
| `memory_types` | `array` | no | Filter by memory types Items: `string`. |
| `description` | `string` | no | Human-readable description |
| `creator` | `string` | no | Creator name |
| `encrypt_key` | `string` | no | Hex-encoded 32-byte AES key |
| `sign_key` | `string` | no | Hex-encoded 32-byte Ed25519 secret key |

### `snapshot_load`

Load a .egm snapshot into the memory store. Strategies: merge (skip duplicates), replace (clear workspace first), isolate (new workspace), dry_run (preview only).

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `path`, `strategy`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `path` | `string` | yes | Path to .egm file |
| `strategy` | `string` | yes | Load strategy Allowed: `merge`, `replace`, `isolate`, `dry_run`. |
| `target_workspace` | `string` | no | Target workspace (defaults to snapshot's workspace) |
| `decrypt_key` | `string` | no | Hex-encoded 32-byte AES key for encrypted snapshots |

### `snapshot_inspect`

Inspect a .egm snapshot without loading it. Returns manifest, file list, and size.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `path`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `path` | `string` | yes | Path to .egm file |

### `attestation_log`

Log a document ingestion with cryptographic attestation. Creates a chained record proving the document was processed.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `content`, `document_name`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `content` | `string` | yes | Document content to attest |
| `document_name` | `string` | yes | Name of the document |
| `agent_id` | `string` | no | ID of the attesting agent |
| `memory_ids` | `array` | no | IDs of memories created from this document Items: `integer`. |
| `sign_key` | `string` | no | Hex-encoded 32-byte Ed25519 secret key |

### `attestation_verify`

Verify whether a document has been attested (ingested and recorded).

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `content`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `content` | `string` | yes | Document content to verify |

### `attestation_chain_verify`

Verify the integrity of the entire attestation chain. Returns valid, broken (with location), or empty.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `attestation_list`

List attestation records with optional filters. Supports JSON, CSV, and Merkle proof export formats.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `limit` | `integer` | no | Maximum records to return Default: `50`. |
| `offset` | `integer` | no | Number of records to skip Default: `0`. |
| `agent_id` | `string` | no | Filter by agent ID |
| `document_name` | `string` | no | Filter by document name |
| `export_format` | `string` | no | Export format Allowed: `json`, `csv`, `merkle_proof`. |

### `memory_graph_path`

Finds how two entities are connected in the knowledge graph via DuckDB OLAP engine. Discovers hidden relationships across multiple hops using recursive path-finding.

- Tier: `advanced`
- Annotations: readOnlyHint, idempotentHint
- Required inputs: `scope`, `source_id`, `target_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `scope` | `string` | yes | Tenant scope prefix, e.g., 'global/org/user' |
| `source_id` | `integer` | yes | Starting node ID |
| `target_id` | `integer` | yes | Target node ID |
| `max_depth` | `integer` | no | Maximum hops to traverse (default: 4, max: 10) Default: `4`. |

### `memory_temporal_snapshot`

Retrieves the exact facts and relationships that were true at a specific historical point in time. Uses DuckDB OLAP engine for fast columnar scans over temporal edges.

- Tier: `advanced`
- Annotations: readOnlyHint, idempotentHint
- Required inputs: `scope`, `timestamp`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `scope` | `string` | yes | Tenant scope prefix |
| `timestamp` | `string` | yes | ISO-8601 timestamp for the point-in-time query |

### `memory_scope_snapshot`

Compares the knowledge graph between two timestamps, showing what relationships were added, removed, or changed. Uses DuckDB OLAP engine for efficient temporal diff.

- Tier: `advanced`
- Annotations: readOnlyHint, idempotentHint
- Required inputs: `from_timestamp`, `scope`, `to_timestamp`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `scope` | `string` | yes | Tenant scope prefix |
| `from_timestamp` | `string` | yes | Start of comparison window (ISO-8601) |
| `to_timestamp` | `string` | yes | End of comparison window (ISO-8601) |

### `memory_get_public`

Get a memory with all <private>...</private> tagged sections removed. Safe for sharing in multi-agent contexts.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID |

### `memory_search_compact`

Token-efficient search returning only id, title (first line, max 80 chars), created_at, and tags. Use memory_expand to get full content for specific IDs.

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: `query`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `query` | `string` | yes | Search query |
| `limit` | `integer` | no | Max results (default: 10) |
| `workspace` | `string` | no | Filter to workspace |
| `global` | `boolean` | no | Search across all workspaces (default: false). When true, ignores any workspace filter and includes a workspace field in each result. Default: `false`. |

### `memory_expand`

Fetch full memory content for specific IDs. Used after memory_search_compact to get full content only for memories you need.

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: `ids`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `ids` | `array` | yes | Memory IDs to expand Items: `integer`. |

### `memory_get_injection_prompt`

Assembles the most relevant memories into a ready-to-inject system prompt block. Uses hybrid search to find relevant memories and formats them as markdown, respecting a token budget.

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: `query`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `query` | `string` | yes | Search query to find relevant memories |
| `token_budget` | `integer` | no | Max tokens for output (default: 2000) |
| `workspace` | `string` | no | Filter to specific workspace |
| `include_types` | `array` | no | Filter by memory types Items: `string`. |

### `memory_observe_tool_use`

Store a tool observation as an episodic memory for session continuity. Automatically compresses large inputs/outputs.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `tool_input`, `tool_name`, `tool_output`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `tool_name` | `string` | yes | Name of the tool that was used |
| `tool_input` | `object` | yes | Tool input parameters |
| `tool_output` | `string` | yes | Tool output/result |
| `session_id` | `string` | no | Session identifier for grouping observations |
| `compress` | `boolean` | no | Compress to 200-char previews (default: true) |

### `memory_archive_tool_output`

Archives a tool's full raw output to memory and returns a compressed summary (~500 tokens) for use in the active context. Transforms O(N²) context growth to O(N) by keeping only summaries in the working context while preserving full outputs for on-demand retrieval.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `raw_output`, `tool_name`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `tool_name` | `string` | yes | Name of the tool whose output is being archived |
| `raw_output` | `string` | yes | Full raw output to archive |
| `session_id` | `string` | no | Session identifier for grouping archived outputs (default: 'unknown') |
| `compress_summary` | `boolean` | no | Whether to generate a compressed summary (default: true) |
| `summary_tokens` | `integer` | no | Max tokens for the compressed summary (default: 500) |

### `memory_get_archived_output`

Retrieves the full raw output for an archived tool observation by its archive ID. Use when you need the complete output that was previously compressed for context efficiency.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `archive_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `archive_id` | `integer` | yes | Archive ID returned by memory_archive_tool_output |

### `memory_get_working_memory`

Assembles all compressed tool observations for a session into a token-budgeted working memory block. Includes archive references for retrieving full outputs on demand. This is the core of the Endless Mode context management system.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `session_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `session_id` | `string` | yes | Session identifier to retrieve observations for |
| `token_budget` | `integer` | no | Max tokens for the working memory block (default: 4000) |
| `include_tool_names` | `array` | no | Whitelist of tool names to include (default: all) Items: `string`. |
| `since_minutes` | `integer` | no | Only include observations from the last N minutes (default: all time) |

### `session_land`

Generate a structured session handoff ('land the plane'). Creates a checkpoint memory with session summary, open items, recent decisions, and a bootstrap prompt for the next session. Call this at the end of every work session for seamless continuity.

- Tier: `essential`
- Annotations: mutating (no MCP hints)
- Required inputs: `session_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `session_id` | `string` | yes | Session identifier to hand off |
| `workspace` | `string` | no | Workspace scope (default: 'default') |
| `summary` | `string` | no | Summary of what was accomplished this session |
| `next_session_hints` | `array` | no | Hints for what should be done next session Items: `string`. |

### `memory_build_context`

Build a structured prompt context from relevant memories using hybrid search, with optional graph traversal depth, timeframe filtering, type filtering, and relationship graph inclusion. Inspired by Basic Memory's build_context.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `query`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `query` | `string` | yes | Search query to retrieve relevant memories |
| `total_budget` | `integer` | no | Max tokens for the entire prompt (default: 4096) |
| `strategy` | `string` | no | Context assembly strategy Default: `greedy`. Allowed: `greedy`, `balanced`, `recency`. |
| `workspace` | `string` | no | Workspace to search in |
| `limit` | `integer` | no | Max memories to retrieve (default: 20) |
| `depth` | `integer` | no | Graph traversal depth: 1=search only, 2=search+1 hop of related memories, 3=search+2 hops Default: `1`. Minimum: `1`. Maximum: `3`. |
| `timeframe` | `string` | no | Time window for memory filtering Default: `all`. Allowed: `1h`, `24h`, `7d`, `30d`, `all`. |
| `include_types` | `array` | no | Only include these memory types (e.g., ['note', 'decision']) Items: `string`. |
| `include_graph` | `boolean` | no | Include entity relationship graph in response Default: `false`. |

### `memory_export_markdown`

Export a workspace as human-readable Markdown files with YAML frontmatter and wiki-style [[links]]. Creates one .md file per memory, organized by type in subdirectories, with an index.md overview.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `workspace`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | yes | Workspace to export |
| `output_dir` | `string` | no | Output directory path (default: ./engram-export/{workspace}/) |
| `include_links` | `boolean` | no | Include [[wiki links]] to related memories in each file Default: `true`. |

### `recent_activity`

Discover recently created or updated memories. Returns compact previews sorted by most recent activity. Useful for understanding what has changed recently.

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Filter by workspace (omit for all workspaces) |
| `timeframe` | `string` | no | Time window for activity Default: `24h`. Allowed: `1h`, `24h`, `7d`, `30d`. |
| `limit` | `integer` | no | Max results to return Default: `20`. Minimum: `1`. Maximum: `100`. |
| `include_types` | `array` | no | Only include these memory types Items: `string`. |

### `discover_tools`

List available Engram tools by tier and category. Use this to progressively discover capabilities beyond the essential tool set. Returns tool names, descriptions, and tiers.

- Tier: `essential`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `tier` | `string` | no | Filter by tier: essential (~20 core tools), standard (~57 common tools), advanced (~104 specialized tools), all (everything) Default: `all`. Allowed: `essential`, `standard`, `advanced`, `all`. |
| `category` | `string` | no | Filter by category keyword (e.g., 'search', 'graph', 'session', 'identity', 'quality') |
| `search` | `string` | no | Search tool names and descriptions |

### `memory_prepare_context`

Prepare optimized context for LLM using RTK-inspired pipeline (filter, group, truncate). Reduces token usage by 70-95% through intelligent context preparation.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `query`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `query` | `string` | yes | Query to prepare context for |
| `budget` | `integer` | no | Token budget for prepared context Default: `4000`. |
| `workspace` | `string` | no | Optional workspace filter |

### `harness_record`

Record a durable harness event (decision, handoff, failed_attempt, verification_result, risk, assumption, bug_reproduction, issue_update) with structured metadata for cross-session continuity. Use instead of memory_create when capturing work-state evidence rather than facts.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `kind`, `summary`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `kind` | `string` | yes | The harness event kind Allowed: `decision`, `handoff`, `failed_attempt`, `bug_reproduction`, `verification_result`, `risk`, `assumption`, `issue_update`. |
| `summary` | `string` | yes | Concise summary of the event (1-500 chars) Max length: `500`. |
| `details` | `string` | no | Optional additional context appended to the summary Max length: `8000`. |
| `source_paths` | `array` | no | File paths relevant to this event Items: `string`. |
| `command` | `string` | no | CLI/shell command that produced this evidence |
| `issue_number` | `integer` | no | Related GitHub issue number |
| `commit_sha` | `string` | no | Related git commit SHA |
| `evidence_refs` | `array` | no | Free-form references (URLs, paths, IDs) Items: `string`. |
| `importance` | `number` | no | Importance score (0-1) Default: `0.7`. Minimum: `0`. Maximum: `1`. |
| `workspace` | `string` | no | Workspace scope (default: 'default') |

### `harness_status`

Assemble current project state from harness memory records and optional git state. Returns current objective, active issues, recent decisions, known blockers, last verification, last handoff, and a suggested next action. Token-budget aware; degrades gracefully when git is unavailable.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Workspace scope (default: 'default') |
| `max_records` | `integer` | no | Max recent harness records to include Default: `10`. Minimum: `1`. Maximum: `50`. |
| `token_budget` | `integer` | no | Approximate max tokens for the output (chars/4 heuristic) Default: `2000`. |
| `include_git` | `boolean` | no | Attempt to collect git branch/status/log state Default: `true`. |

### `harness_handoff`

Generate a structured handoff packet for next-agent continuity: current goal, files touched, decisions, tests run/not run, risks, blockers, and next steps. Optionally persists as a harness record. Does NOT claim completion unless verification_evidence is provided.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `current_goal`, `next_steps`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `current_goal` | `string` | yes | What the agent was working toward Max length: `300`. |
| `files_touched` | `array` | no | Paths modified this session Items: `string`. |
| `decisions_made` | `array` | no | Short decision summaries Items: `string`. |
| `tests_run` | `array` | no | Test commands/names that were run Items: `string`. |
| `tests_not_run` | `array` | no | Tests known to be missing or skipped Items: `string`. |
| `known_risks` | `array` | no | Open risks Items: `string`. |
| `blockers` | `array` | no | Things blocking progress Items: `string`. |
| `next_steps` | `array` | yes | Recommended actions for the next agent Items: `string`. Min items: `1`. |
| `issue_numbers` | `array` | no | Related GitHub issue numbers Items: `integer`. |
| `plan_doc_paths` | `array` | no | Paths to relevant plan docs Items: `string`. |
| `verification_evidence` | `string` | no | Evidence that work is complete (test count, command output summary) |
| `persist` | `boolean` | no | Persist the handoff as a harness record Default: `true`. |
| `workspace` | `string` | no | Workspace scope (default: 'default') |

### `harness_verify`

Record a verification command outcome with exit code, output summary, and optional evidence path/hash. Supports negative evidence (failures, skips with reason). Surfaces in harness_status as last_verification and feeds harness_handoff completion gating.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `command`, `exit_code`, `output_summary`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `command` | `string` | yes | The command that was run (e.g. 'cargo test --lib') Max length: `200`. |
| `exit_code` | `integer` | yes | Process exit code (0 = success) |
| `passed` | `boolean` | no | Explicit pass/fail; derived from exit_code == 0 if omitted |
| `output_summary` | `string` | yes | Concise summary (e.g. '873 tests passed, 0 failed') Max length: `500`. |
| `evidence_path` | `string` | no | Path to the full output file or log |
| `evidence_hash` | `string` | no | SHA-256 of the full output for integrity |
| `skipped_reason` | `string` | no | If skipped, why (negative evidence) |
| `issue_numbers` | `array` | no | Linked GitHub issues Items: `integer`. |
| `memory_ids` | `array` | no | Linked memory IDs Items: `integer`. |
| `importance` | `number` | no | Importance score (0-1) Default: `0.8`. Minimum: `0`. Maximum: `1`. |
| `workspace` | `string` | no | Workspace scope (default: 'default') |

### `memory_import_markdown`

Import memories from Markdown files with engram_ frontmatter (RFC 0004). Review mode by default (confirm: false) — returns a staged list without writing. Detects drift via content_hash and version conflicts via engram_version. Ignores non-engram_ frontmatter keys (Obsidian-safe).

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `input_dir`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `input_dir` | `string` | yes | Directory to scan recursively for .md files |
| `workspace` | `string` | no | Override workspace (default: from each file's engram_workspace) |
| `confirm` | `boolean` | no | Apply writes. When false (default), dry-run review only Default: `false`. |
| `force_version` | `boolean` | no | Bypass version conflict checks Default: `false`. |

### `memory_agent_start`

Configure a tick-based memory agent for a workspace and return its initial configuration.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Workspace the agent will operate on (default: "default") |
| `interval_secs` | `integer` | no | Desired check interval in seconds (default: 300) |

### `memory_agent_stop`

Stop a tick-based memory agent (no-op for stateless agents; resets client-side tracking).

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Workspace whose agent should be stopped (default: "default") |

### `memory_agent_status`

Return current status and memory statistics for a workspace agent.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Workspace to report status for (default: "default") |

### `memory_agent_metrics`

Run one full agent cycle (prune/merge/archive) and return the actions taken and aggregate metrics. Mutates the database.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Workspace to run the agent cycle on (default: "default") |
| `max_actions` | `integer` | no | Maximum number of actions to take in this cycle (default: 10) |

### `memory_auto_link`

Run semantic and temporal auto-linker on a workspace, creating crossref edges in the database. Mutates the database.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Workspace to auto-link (default: all workspaces) |
| `similarity_threshold` | `number` | no | Minimum cosine similarity to create a semantic link (default: 0.75) |
| `time_window_minutes` | `integer` | no | Time window in minutes for temporal linking (default: 30) |

### `memory_auto_link_stats`

Return aggregate statistics about auto-generated semantic and temporal links.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_block_create`

Create a named, token-bounded memory block (Letta/MemGPT-style self-editing context slot).

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `name`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `name` | `string` | yes | Unique name for the memory block |
| `content` | `string` | no | Initial content of the block (default: empty string) |
| `max_tokens` | `integer` | no | Maximum token capacity for the block (default: 4096) |

### `memory_block_get`

Retrieve a memory block by name.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `name`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `name` | `string` | yes | Name of the memory block to retrieve |

### `memory_block_edit`

Update the content of an existing memory block, incrementing its version and recording the reason.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `content`, `name`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `name` | `string` | yes | Name of the memory block to edit |
| `content` | `string` | yes | New content for the block |
| `reason` | `string` | no | Human-readable reason for this edit (optional) |

### `memory_block_list`

List all memory blocks with their names, versions, and token usage.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_block_archive`

Permanently delete a memory block and return its final content before deletion. Destructive and irreversible.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `name`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `name` | `string` | yes | Name of the memory block to archive and delete |

### `memory_block_history`

Return the edit history for a named memory block.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `name`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `name` | `string` | yes | Name of the memory block |
| `limit` | `integer` | no | Maximum number of history entries to return (default: 20) |

### `memory_cache_stats`

Return hit/miss statistics and entry count for the in-memory semantic search cache.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_cache_clear`

Evict all entries from the semantic search cache. Mutates in-memory cache state.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_capture_screenshot`

Capture a screenshot of the full screen or a specific application window and save it to a local file.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `app_name` | `string` | no | Name of the application window to capture; omit to capture the full screen |

### `memory_cluster`

Run Louvain community detection on the memory graph and return detected clusters with modularity score.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `min_cluster_size` | `integer` | no | Minimum number of members for a cluster to be reported (default: 2). |
| `resolution` | `number` | no | Louvain resolution parameter controlling cluster granularity (default: 1.0). |
| `link_types` | `array` | no | Restrict clustering to these edge/link types. Omit to use all link types. Items: `string`. |

### `memory_coactivation_report`

Return coactivation graph statistics including edge count, average strength, and strongest co-occurring memory pairs.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_compress`

Apply rule-based semantic compression to a single memory and return the structured result with key entities and facts.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | ID of the memory to compress. |
| `target_ratio` | `number` | no | Target compression ratio as a fraction of original tokens (default: 0.1). |

### `memory_compress_for_context`

Pack a set of memories into a token budget for LLM context, returning compressed entries and diagnostics about skipped memories.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `ids`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `ids` | `array` | yes | Memory IDs to compress and pack (alias: memory_ids). Items: `integer`. |
| `memory_ids` | `array` | no | Alias for ids. Items: `integer`. |
| `token_budget` | `integer` | no | Maximum token budget for the packed context (default: 4096). |

### `memory_consolidate`

Run offline consolidation over a workspace, merging and archiving similar memories; use dry_run to preview without writing.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Workspace to consolidate (default: "default"). |
| `strategy` | `string` | no | Grouping strategy: "content_overlap" (default), "tag_similarity", or "temporal_proximity". |
| `dry_run` | `boolean` | no | If true, report what would be merged/archived without writing changes (default: false). |

### `memory_decompress`

Retrieve the original (uncompressed) content of a memory by ID.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | ID of the memory whose content to retrieve. |

### `memory_describe_image`

Describe the contents of an image file using the configured vision provider (requires VISION_PROVIDER env).

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `image_path`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `image_path` | `string` | yes | Absolute filesystem path to the image file (JPEG, PNG, WebP, etc.). |
| `prompt` | `string` | no | Optional custom prompt to guide the image description. |

### `memory_detect_conflicts`

Detect contradictory or conflicting facts in the knowledge graph; optionally persist detected conflicts for later resolution.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `save` | `boolean` | no | If true, persist detected conflicts to the conflicts table for later resolution (default: false). |

### `memory_detect_updates`

Given new content, identify existing memories in a workspace that may be stale or in need of an update.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `content`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `content` | `string` | yes | New content to compare against stored memories. |
| `workspace` | `string` | no | Workspace to search for update candidates (default: "default"). |

### `memory_embedding_migrate`

Re-embed all memories using the active embedding model; use dry_run to count affected memories without writing.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `dry_run` | `boolean` | no | If true, count memories to migrate without re-embedding them (default: false). |
| `target_model` | `string` | no | Target embedding model name to record in embedding_model column. Defaults to the active embedder's model name. |

### `memory_embedding_providers`

List the active embedding provider including model name and vector dimensions.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_explain_search`

Explain how each result in a scored search batch was ranked, breaking down bm25, vector, fuzzy, recency, importance, and optional rerank contributions.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `results`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `results` | `array` | yes | Array of scored search result objects to explain. Items: `object`. |
| `reranking_active` | `boolean` | no | Whether cross-encoder reranking was active for this result set (default: false). |
| `rrf_k` | `integer` | no | RRF k constant used during retrieval (default: 60). |

### `memory_extract_facts`

Extract subject-predicate-object facts from a memory's content using rule-based NLP and persist them to the facts table.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `memory_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `memory_id` | `integer` | yes | ID of the memory from which to extract and store facts. |

### `memory_fact_graph`

Return all stored subject-predicate-object facts for a given subject entity.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `subject`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `subject` | `string` | yes | Entity name to look up in the facts table. |

### `memory_feedback`

Record relevance feedback for a search result and update the memory's utility score; schedules low-utility memories for consolidation.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `memory_id`, `query`, `signal`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `query` | `string` | yes | The search query that produced the result. |
| `memory_id` | `integer` | yes | ID of the memory being rated. |
| `signal` | `string` | yes | Feedback signal: "useful" (alias "helpful"), "irrelevant" (alias "not_helpful"), "outdated", or "conflict". |
| `rank_position` | `integer` | no | 0-based rank position of the result in the original result list (optional). |
| `original_score` | `number` | no | The final_score from the original search result (optional). |
| `workspace` | `string` | no | Workspace context for the feedback (default: "default"). |

### `memory_feedback_stats`

Return aggregated search-feedback statistics (thumbs-up/down counts, top-rated queries) for a workspace.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Workspace name to filter stats; omit for all workspaces. |

### `memory_garden`

Run full autonomous garden maintenance on a workspace: prunes stale memories, merges duplicates, archives cold entries, and compresses verbose content.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Workspace to garden (default: "default"). |

### `memory_garden_preview`

Dry-run garden maintenance: reports what would be pruned, merged, archived, or compressed without making any changes.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Workspace to preview gardening for (default: "default"). |

### `memory_get_cluster`

Return the Louvain community cluster that contains a specific memory, including its cluster ID, size, and member IDs.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `memory_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `memory_id` | `integer` | yes | ID of the memory whose cluster to look up. |

### `memory_knowledge_stats`

Return aggregate statistics over the knowledge-graph facts table: total facts, unique subjects/predicates/objects, and top entities.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `memory_list_auto_links`

List auto-generated graph links (semantic or temporal) between memories, optionally filtered by link type.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `link_type` | `string` | no | Filter by link type: "semantic" or "temporal". Omit for all types. |
| `limit` | `integer` | no | Maximum number of links to return (default: 50). |

### `memory_list_clusters`

List all detected memory clusters from the persistent cluster table, optionally selecting the detection algorithm.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `algorithm` | `string` | no | Clustering algorithm to filter by (default: "louvain"). |

### `memory_list_facts`

List extracted subject-predicate-object facts, optionally scoped to a single source memory.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `memory_id` | `integer` | no | Source memory ID to filter facts; omit to list facts from all memories. |
| `limit` | `integer` | no | Maximum number of facts to return (default: 100). |

### `memory_list_media`

List media assets stored in the media_assets table, optionally filtered by type (image, audio, video).

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `media_type` | `string` | no | Filter by media type: "image", "audio", or "video". Omit for all types. |
| `limit` | `integer` | no | Maximum number of assets to return (default: 50). |

### `memory_process_video`

Process a video file: extract metadata and keyframe descriptions via the configured vision provider, and create a memory record for the result.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `video_path`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `video_path` | `string` | yes | Absolute path to the video file to process. |

### `memory_query_triplets`

SPARQL-like pattern query over the knowledge-graph facts table: match any combination of subject, predicate, and object (all optional, acts as wildcard when omitted).

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `subject` | `string` | no | Subject entity to match (wildcard if omitted). |
| `predicate` | `string` | no | Predicate/relation to match (wildcard if omitted). |
| `object` | `string` | no | Object value to match (wildcard if omitted). |

### `memory_reflect`

Generate a reflective synthesis over a set of memories at a configurable analytical depth (surface, analytical, or meta).

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `ids`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `ids` | `array` | yes | Array of memory IDs to reflect on (required, must be non-empty). Items: `integer`. |
| `depth` | `string` | no | Reflection depth: "surface" (default), "analytical", or "meta". |

### `memory_resolve_conflict`

Resolve a saved knowledge-graph conflict by ID using a chosen strategy, removing or retaining the conflicting edges accordingly.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `conflict_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `conflict_id` | `integer` | yes | ID of the conflict record to resolve (required). |
| `strategy` | `string` | no | Resolution strategy: "keep_newer" (default), "keep_higher_confidence", "merge", or "manual". |

### `memory_sentiment_analyze`

Analyze the sentiment of a single memory's content, returning a score, label (positive/neutral/negative), confidence, and keyword signals.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | ID of the memory to analyze (required). |

### `memory_sentiment_timeline`

Compute a chronological sentiment timeline over memories in a workspace within an optional time range.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Workspace to scan (default: "default"). |
| `from` | `string` | no | ISO-8601 start timestamp (default: epoch). |
| `to` | `string` | no | ISO-8601 end timestamp (default: far future). |
| `limit` | `integer` | no | Maximum number of timeline entries to return (default: 50). |

### `memory_suggest_acquisitions`

Analyse knowledge gaps in a workspace and suggest new memories to create.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `workspace` | `string` | no | Workspace to analyse (default: "default"). |
| `limit` | `integer` | no | Maximum number of suggestions to return (default: 10). |

### `memory_synthesis`

Check semantic overlap between two content strings and produce a merged synthesis using the chosen strategy.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `content_a`, `content_b`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `content_a` | `string` | yes | First content string to synthesise. |
| `content_b` | `string` | yes | Second content string to synthesise. |
| `id_a` | `integer` | no | Optional memory ID associated with content_a (default: 0). |
| `strategy` | `string` | no | Synthesis strategy: "merge" (default), "replace", or "append". Allowed: `merge`, `replace`, `append`. |

### `memory_transcribe_audio`

Transcribe an audio file to text using the configured audio transcription provider.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `audio_path`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `audio_path` | `string` | yes | Absolute or relative path to the audio file to transcribe. |

### `memory_utility_score`

Compute the Q-value utility score for a memory based on its retrieval feedback history.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `id` | `integer` | yes | Memory ID to score. |

### `scope_get`

Return the current scope path and level for a given memory.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `memory_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `memory_id` | `integer` | yes | ID of the memory whose scope to retrieve. |

### `scope_list`

List all distinct scope paths currently present in the database.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `scope_search`

Search for memories whose content matches a query within a given scope, including ancestor scopes.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `query`, `scope_path`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `query` | `string` | yes | Substring to search for within scoped memories. |
| `scope_path` | `string` | yes | Hierarchical scope path to search within (e.g. "global/org:acme/user:alice"). |

### `scope_set`

Assign or update the hierarchical scope of a memory.

- Tier: `standard`
- Annotations: mutating (no MCP hints)
- Required inputs: `memory_id`, `scope_path`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `memory_id` | `integer` | yes | ID of the memory to re-scope. |
| `scope_path` | `string` | yes | Target scope path (e.g. "global/org:acme/user:alice"). |

### `scope_tree`

Return a hierarchical tree of all scopes in the database.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `temporal_add_edge`

Add a bi-temporal validity edge between two memories in the knowledge graph.

- Tier: `advanced`
- Annotations: mutating (no MCP hints)
- Required inputs: `from_id`, `relation`, `to_id`, `valid_from`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `from_id` | `integer` | yes | Source memory ID. |
| `to_id` | `integer` | yes | Target memory ID. |
| `relation` | `string` | yes | Semantic label for the edge (e.g. "works_at"). |
| `valid_from` | `string` | yes | RFC3339 timestamp marking the start of edge validity. |
| `properties` | `object` | no | Arbitrary JSON metadata to attach to the edge. |
| `confidence` | `number` | no | Edge confidence score between 0.0 and 1.0 (default: 1.0). |
| `source` | `string` | no | Provenance string identifying where this edge originates. |
| `scope_path` | `string` | no | Optional scope path to associate with this edge. |

### `temporal_contradictions`

Detect overlapping or contradictory edge pairs in the temporal knowledge graph.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| _(none)_ |  | no | No input properties declared. |

### `temporal_diff`

Compute the set of added, removed, and changed edges between two RFC3339 timestamps in the temporal graph.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `t1`, `t2`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `t1` | `string` | yes | Earlier RFC3339 timestamp (snapshot baseline). |
| `t2` | `string` | yes | Later RFC3339 timestamp (snapshot target). |
| `scope_path` | `string` | no | Optional scope path to restrict the diff. |

### `temporal_snapshot`

Return all currently-valid temporal graph edges as of a given RFC3339 timestamp.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `timestamp`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `timestamp` | `string` | yes | RFC3339 point-in-time for the snapshot. |
| `scope_path` | `string` | no | Optional scope path to restrict the snapshot. |

### `temporal_timeline`

Return the full edge history between two memory IDs, ordered chronologically.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: `from_id`, `to_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `from_id` | `integer` | yes | Source memory ID. |
| `to_id` | `integer` | yes | Target memory ID. |
| `scope_path` | `string` | no | Optional scope path to restrict the timeline. |

### `memory_enrichment_timeline`

List all enrichment events for a specific memory (lifecycle transitions, consolidation, compression, etc.). Shows what automated operations affected this memory and why.

- Tier: `standard`
- Annotations: readOnlyHint
- Required inputs: `memory_id`

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `memory_id` | `integer` | yes | ID of the memory whose enrichment history to retrieve. |
| `event_type` | `string` | no | Filter to a specific event type (e.g. "consolidation", "lifecycle_transition"). |
| `include_dry_runs` | `boolean` | no | Include events that were executed in dry-run mode (default: true). |
| `include_snapshots` | `boolean` | no | Include snapshot events (default: true). |
| `limit` | `integer` | no | Maximum number of events to return (default: 20, max: 100). |

### `memory_enrichment_audit`

Query enrichment events globally with filters (status, event_type, agent_id, operation_id, workspace, time range). Use for compliance audit and batch tracing.

- Tier: `advanced`
- Annotations: readOnlyHint
- Required inputs: none

| Input | Type | Required | Summary |
|-------|------|----------|---------|
| `event_type` | `string` | no | Filter by event type (e.g. "consolidation", "lifecycle_transition", "compression"). |
| `triggered_by` | `string` | no | Filter by the tool name that triggered the event. |
| `agent_id` | `string` | no | Filter by the agent ID that triggered the event. |
| `status` | `string` | no | Filter by event outcome status. Allowed: `completed`, `failed`, `skipped`. |
| `workspace` | `string` | no | Filter to a specific workspace. |
| `operation_id` | `string` | no | Filter by a specific operation ID (exact match). |
| `memory_id` | `integer` | no | Filter to events that reference a specific memory. |
| `version_id` | `integer` | no | Filter to events that reference a specific memory version. |
| `dry_run` | `boolean` | no | Filter by dry-run flag (true = only dry-run events, false = only real events). |
| `since` | `string` | no | ISO-8601 timestamp: return events created at or after this time. |
| `until` | `string` | no | ISO-8601 timestamp: return events created at or before this time. |
| `order` | `string` | no | Sort order by creation time: "desc" (newest first, default) or "asc". Allowed: `desc`, `asc`. |
| `limit` | `integer` | no | Maximum number of events to return (default: 50, max: 200). |
