//! MCP tool catalog: domain definitions, feature gating, and categorizations.

pub mod admin;
pub mod context;
pub mod graph;
pub mod memory_crud;
pub mod misc;
pub mod multimodal;
pub mod policy;
pub mod search;

pub(crate) fn required_features(name: &str) -> &'static [&'static str] {
    match name {
        "langfuse_connect"
        | "langfuse_sync"
        | "langfuse_sync_status"
        | "langfuse_extract_patterns"
        | "memory_from_trace" => &["langfuse"],
        "meilisearch_search"
        | "meilisearch_reindex"
        | "meilisearch_status"
        | "meilisearch_config" => &["meilisearch"],
        "memory_auto_link"
        | "memory_list_auto_links"
        | "memory_auto_link_stats"
        | "memory_cluster"
        | "memory_get_cluster"
        | "memory_list_clusters" => &["emergent-graph"],
        "memory_sync_media" => &["multimodal", "cloud"],
        "memory_describe_image"
        | "memory_transcribe_audio"
        | "memory_capture_screenshot"
        | "memory_process_video"
        | "memory_list_media"
        | "memory_search_by_image"
        | "memory_ingest_media" => &["multimodal"],
        "memory_graph_path" | "memory_temporal_snapshot" | "memory_scope_snapshot" => {
            &["duckdb-graph"]
        }
        "dream_run_now"
        | "dream_create"
        | "dream_get"
        | "dream_list"
        | "dream_cancel"
        | "dream_archive"
        | "dream_candidates_list"
        | "dream_candidate_get"
        | "dream_candidate_review"
        | "dream_candidate_apply"
        | "memory_agent_writeback"
        | "dream_eval_run" => &["dream-phase"],
        "attestation_log"
        | "attestation_verify"
        | "attestation_chain_verify"
        | "attestation_list" => &["attestation"],
        "snapshot_create" | "snapshot_load" | "snapshot_inspect" => &["snapshot"],
        _ => &[],
    }
}

pub(crate) fn required_feature(name: &str) -> Option<&'static str> {
    required_features(name).first().copied()
}

pub(crate) fn required_feature_summary(name: &str) -> Option<String> {
    match required_features(name) {
        [] => None,
        [feature] => Some((*feature).to_string()),
        features => Some(features.join(",")),
    }
}

pub(crate) fn tool_group(name: &str) -> &'static str {
    if let Some(feature) = required_feature(name) {
        return match feature {
            "langfuse" => "feature.langfuse",
            "meilisearch" => "feature.meilisearch",
            "emergent-graph" => "feature.emergent_graph",
            "multimodal" => "feature.multimodal",
            "duckdb-graph" => "feature.duckdb_graph",
            "dream-phase" => "feature.dream",
            "attestation" => "feature.attestation",
            "snapshot" => "feature.snapshot",
            _ => "feature.other",
        };
    }

    match name {
        "discover_tools" | "recent_activity" | "memory_agent_contract" => return "core",
        "context_seed"
        | "context_record"
        | "context_record_artifact"
        | "context_get_artifact"
        | "context_search"
        | "context_build_bundle"
        | "context_budget_check" => return "context",
        _ => {}
    }

    let prefix = name.split('_').next().unwrap_or("");
    match prefix {
        "identity" => "identity",
        "session" => "session",
        "workspace" => "workspace",
        "quality" | "salience" => "quality",
        "scope" => "scope",
        "temporal" => "temporal",
        "sync" => "sync",
        "agent" => "agent",
        "graph" => "memory.graph",
        "harness" => "harness",
        "lifecycle" | "retention" => "lifecycle",
        "attestation" | "snapshot" => "portability",
        "embedding" => "embedding",
        "search" => "search",
        "pending" => "admin",
        "memory" => memory_subgroup(name),
        _ => "misc",
    }
}

fn memory_subgroup(name: &str) -> &'static str {
    let has = |needle: &str| name.contains(needle);
    if has("search")
        || has("retrieve")
        || has("digest")
        || has("expand")
        || has("related")
        || has("traverse")
        || has("find_path")
        || has("smart")
        || has("injection")
    {
        "memory.search"
    } else if has("identity") {
        "identity"
    } else if has("block") {
        "memory.block"
    } else if has("quality") || has("conflict") || has("duplicate") || has("reconcile") {
        "memory.quality"
    } else if has("lifecycle")
        || has("archive")
        || has("decay")
        || has("promote")
        || has("cleanup")
        || has("expir")
        || has("consolidat")
        || has("garden")
        || has("score")
        || has("policy")
    {
        "memory.lifecycle"
    } else if has("entity")
        || has("link")
        || has("cluster")
        || has("coactivation")
        || has("fact")
        || has("triplet")
        || has("knowledge")
        || has("reflect")
    {
        "memory.graph"
    } else if has("session")
        || has("working_memory")
        || has("checkpoint")
        || has("observe_tool")
        || has("archived_output")
    {
        "memory.session"
    } else if has("enrichment")
        || has("replay")
        || has("events")
        || has("stats")
        || has("versions")
        || has("cache")
        || has("embedding")
        || has("share")
        || has("import")
        || has("export")
        || has("migrate")
        || has("rebuild")
        || has("tag")
        || has("validate")
        || has("upload")
        || has("compress")
        || has("sentiment")
        || has("feedback")
        || has("utility")
        || has("synthesis")
        || has("detect")
        || has("suggest")
    {
        "memory.admin"
    } else {
        "memory.core"
    }
}

#[cfg(test)]
mod tests {
    use super::super::TOOL_DEFINITIONS;
    use super::tool_group;

    #[test]
    fn every_tool_resolves_to_a_named_group() {
        for def in TOOL_DEFINITIONS.iter() {
            let group = tool_group(def.name);
            assert!(!group.is_empty(), "empty group for {}", def.name);
            assert_ne!(group, "unknown", "unknown group for {}", def.name);
        }
    }

    #[test]
    fn feature_tools_resolve_to_feature_group() {
        assert_eq!(tool_group("memory_graph_path"), "feature.duckdb_graph");
        assert_eq!(tool_group("attestation_log"), "feature.attestation");
        assert_eq!(tool_group("memory_describe_image"), "feature.multimodal");
    }
}
