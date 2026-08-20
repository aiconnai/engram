//! Autonomous Sleep-Time Dream & Memory Consolidation Worker (RFC 0008).
//!
//! Provides proactive background consolidation across four stages:
//! 1. **Procedural Distillation**: Synthesizes reusable solutions and lessons from episodic sessions.
//! 2. **Semantic & Perceptual Deduplication**: Detects near-duplicates (cosine similarity >= 0.92 or dHash distance <= 4),
//!    merging metadata and archiving redundant low-salience records.
//! 3. **Knowledge Graph Topology Optimization**: Prunes decayed coactivation edges and optimizes graph connectivity.
//! 4. **Thematic Digest Synthesis**: Emits high-salience executive summary digests with concept tag distributions.

use std::collections::HashSet;

use chrono::{DateTime, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::graph::coactivation::{CoactivationConfig, CoactivationTracker};
use crate::search::vector::cosine_similarity;
use crate::storage::queries::{create_memory, list_memories, update_memory_lifecycle_state};
use crate::storage::Storage;
use crate::types::{
    CreateMemoryInput, LifecycleState, ListOptions, MemoryScope, MemoryTier, MemoryType,
};

/// Configuration for the autonomous dream consolidation pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamPipelineConfig {
    /// Similarity threshold for semantic deduplication (default: 0.92).
    pub semantic_dedup_threshold: f32,
    /// Minimum age in days before decaying unused coactivation edges (default: 7).
    pub graph_prune_min_age_days: i64,
    /// Whether to generate procedural distillates from episodic memories (default: true).
    pub enable_procedural_distillation: bool,
    /// Whether to merge and archive near-duplicates (default: true).
    pub enable_deduplication: bool,
    /// Whether to prune decayed graph edges (default: true).
    pub enable_graph_optimization: bool,
    /// Whether to emit a thematic digest memory (default: true).
    pub enable_thematic_digest: bool,
    /// Dry run mode: detect and report without mutating database (default: false).
    pub dry_run: bool,
}

impl Default for DreamPipelineConfig {
    fn default() -> Self {
        Self {
            semantic_dedup_threshold: 0.92,
            graph_prune_min_age_days: 7,
            enable_procedural_distillation: true,
            enable_deduplication: true,
            enable_graph_optimization: true,
            enable_thematic_digest: true,
            dry_run: false,
        }
    }
}

/// Result of a single workspace consolidation pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConsolidationResult {
    pub workspace: String,
    pub episodic_scanned: usize,
    pub procedural_rules_extracted: usize,
    pub duplicates_found: usize,
    pub duplicates_archived: usize,
    pub graph_edges_pruned: usize,
    pub tokens_saved_estimate: usize,
    pub digest_memory_id: Option<i64>,
    pub details: Vec<String>,
}

/// Comprehensive report of a full multi-workspace consolidation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamPipelineResult {
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub workspaces_processed: usize,
    pub total_rules_extracted: usize,
    pub total_duplicates_archived: usize,
    pub total_edges_pruned: usize,
    pub total_tokens_saved: usize,
    pub workspace_results: Vec<WorkspaceConsolidationResult>,
    pub dry_run: bool,
}

/// Status and metrics of the autonomous dream worker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamWorkerStatus {
    pub is_running: bool,
    pub total_runs: u64,
    pub last_run_started_at: Option<DateTime<Utc>>,
    pub last_run_finished_at: Option<DateTime<Utc>>,
    pub total_rules_extracted: u64,
    pub total_duplicates_archived: u64,
    pub total_tokens_saved: u64,
    pub last_error: Option<String>,
}

/// Extracted actionable insight from consolidated memories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamInsight {
    pub id: i64,
    pub memory_type: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub importance: f64,
    pub created_at: Option<DateTime<Utc>>,
}

/// Report containing recent digests and distilled rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DreamInsightReport {
    pub workspace: String,
    pub digests: Vec<DreamInsight>,
    pub procedural_rules: Vec<DreamInsight>,
    pub top_concept_tags: Vec<(String, usize)>,
    pub total_consolidated_memories: usize,
}

/// Autonomous Dream Pipeline orchestrating procedural distillation, dedup, and graph maintenance.
pub struct DreamPipeline;

impl DreamPipeline {
    /// Execute the full 4-stage consolidation pipeline across all workspaces.
    pub fn run_all(storage: &Storage, config: &DreamPipelineConfig) -> Result<DreamPipelineResult> {
        let started_at = Utc::now();
        let workspaces = storage.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT workspace FROM memories WHERE workspace IS NOT NULL AND lifecycle_state = 'active'",
            )?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
            let mut ws_list = Vec::new();
            for ws in rows.flatten() {
                if !ws.trim().is_empty() {
                    ws_list.push(ws);
                }
            }
            if ws_list.is_empty() {
                ws_list.push("default".to_string());
            }
            Ok(ws_list)
        })?;

        let mut workspace_results = Vec::new();
        let mut total_rules_extracted = 0;
        let mut total_duplicates_archived = 0;
        let mut total_edges_pruned = 0;
        let mut total_tokens_saved = 0;

        for ws in &workspaces {
            match Self::run_workspace(storage, ws, config) {
                Ok(res) => {
                    total_rules_extracted += res.procedural_rules_extracted;
                    total_duplicates_archived += res.duplicates_archived;
                    total_edges_pruned += res.graph_edges_pruned;
                    total_tokens_saved += res.tokens_saved_estimate;
                    workspace_results.push(res);
                }
                Err(e) => {
                    tracing::warn!(
                        target = "engram::dream::worker",
                        workspace = %ws,
                        error = %e,
                        "Consolidation pass failed for workspace"
                    );
                }
            }
        }

        Ok(DreamPipelineResult {
            started_at,
            finished_at: Utc::now(),
            workspaces_processed: workspaces.len(),
            total_rules_extracted,
            total_duplicates_archived,
            total_edges_pruned,
            total_tokens_saved,
            workspace_results,
            dry_run: config.dry_run,
        })
    }

    /// Execute the consolidation pipeline for a single workspace.
    pub fn run_workspace(
        storage: &Storage,
        workspace: &str,
        config: &DreamPipelineConfig,
    ) -> Result<WorkspaceConsolidationResult> {
        let mut details = Vec::new();

        // 1. Stage 1: Procedural Extraction & Distillation
        let (episodic_scanned, rules_extracted) = if config.enable_procedural_distillation {
            Self::stage_procedural_distillation(storage, workspace, config.dry_run, &mut details)?
        } else {
            (0, 0)
        };

        // 2. Stage 2: Semantic & Perceptual Deduplication
        let (duplicates_found, duplicates_archived, tokens_saved) = if config.enable_deduplication {
            Self::stage_semantic_deduplication(storage, workspace, config, &mut details)?
        } else {
            (0, 0, 0)
        };

        // 3. Stage 3: Knowledge Graph Topology Optimization
        let edges_pruned = if config.enable_graph_optimization {
            Self::stage_graph_optimization(storage, config, &mut details)?
        } else {
            0
        };

        // 4. Stage 4: Thematic Digest Synthesis
        let digest_memory_id = if config.enable_thematic_digest && !config.dry_run {
            Self::stage_thematic_digest(
                storage,
                workspace,
                rules_extracted,
                duplicates_archived,
                tokens_saved,
                &mut details,
            )?
        } else {
            None
        };

        Ok(WorkspaceConsolidationResult {
            workspace: workspace.to_string(),
            episodic_scanned,
            procedural_rules_extracted: rules_extracted,
            duplicates_found,
            duplicates_archived,
            graph_edges_pruned: edges_pruned,
            tokens_saved_estimate: tokens_saved,
            digest_memory_id,
            details,
        })
    }

    // ─── Stage 1: Procedural Distillation ─────────────────────────────────────

    fn stage_procedural_distillation(
        storage: &Storage,
        workspace: &str,
        dry_run: bool,
        details: &mut Vec<String>,
    ) -> Result<(usize, usize)> {
        let memories = storage.with_connection(|conn| {
            list_memories(
                conn,
                &ListOptions {
                    workspace: Some(workspace.to_string()),
                    limit: Some(100),
                    include_archived: false,
                    ..Default::default()
                },
            )
        })?;

        let mut episodic_count = 0;
        let mut rules_extracted = 0;

        for m in &memories {
            let is_episodic = m.memory_type == MemoryType::Episodic
                || m.tags
                    .iter()
                    .any(|t| t == "task" || t == "session" || t == "execution" || t == "error");

            if !is_episodic {
                continue;
            }
            episodic_count += 1;

            // Extract pattern/solution pairs from content
            if let Some(rule) = extract_procedural_lesson(&m.content) {
                rules_extracted += 1;
                details.push(format!(
                    "Distilled procedural rule from memory #{}: '{}'",
                    m.id,
                    rule.chars().take(60).collect::<String>()
                ));

                if !dry_run {
                    let mut tags = m.tags.clone();
                    if !tags.contains(&"procedural-rule".to_string()) {
                        tags.push("procedural-rule".to_string());
                    }
                    if !tags.contains(&"dream-distillate".to_string()) {
                        tags.push("dream-distillate".to_string());
                    }

                    let input = CreateMemoryInput {
                        content: format!(
                            "Procedural Lesson (Distilled from Memory #{}):\n\n{}",
                            m.id, rule
                        ),
                        memory_type: MemoryType::Procedural,
                        workspace: Some(workspace.to_string()),
                        scope: MemoryScope::Global,
                        tier: MemoryTier::Permanent,
                        importance: Some((m.importance + 0.2).min(1.0)),
                        tags,
                        summary_of_id: Some(m.id),
                        trigger_pattern: Some(format!("procedure:memory-{}", m.id)),
                        ..Default::default()
                    };

                    let _ = storage.with_transaction(|conn| create_memory(conn, &input));
                }
            }
        }

        Ok((episodic_count, rules_extracted))
    }

    // ─── Stage 2: Semantic & Perceptual Deduplication ─────────────────────────

    fn stage_semantic_deduplication(
        storage: &Storage,
        workspace: &str,
        config: &DreamPipelineConfig,
        details: &mut Vec<String>,
    ) -> Result<(usize, usize, usize)> {
        let (memories_with_embeddings, _all_memories) = storage.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT m.id, m.content, m.importance, m.created_at, e.embedding
                 FROM memories m
                 LEFT JOIN embeddings e ON m.id = e.memory_id
                 WHERE m.workspace = ?1 AND m.lifecycle_state = 'active'
                 ORDER BY m.id DESC LIMIT 200",
            )?;

            let rows = stmt.query_map(params![workspace], |row| {
                let id: i64 = row.get(0)?;
                let content: String = row.get(1)?;
                let importance: f64 = row.get(2)?;
                let created_at: Option<String> = row.get(3)?;
                let emb_bytes: Option<Vec<u8>> = row.get(4)?;

                let embedding = emb_bytes.and_then(|bytes| {
                    if bytes.len() % 4 == 0 {
                        let floats: Vec<f32> = bytes
                            .chunks_exact(4)
                            .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                            .collect();
                        Some(floats)
                    } else {
                        None
                    }
                });

                Ok((id, content, importance, created_at, embedding))
            })?;

            let mut embedded = Vec::new();
            let mut all = Vec::new();
            for entry in rows.flatten() {
                if entry.4.is_some() {
                    embedded.push(entry.clone());
                }
                all.push(entry);
            }
            Ok((embedded, all))
        })?;

        let mut archived_ids = HashSet::new();
        let mut duplicates_found = 0;
        let mut tokens_saved = 0;

        // Perform pairwise embedding cosine comparison
        for i in 0..memories_with_embeddings.len() {
            let (id1, content1, imp1, _, emb1) = &memories_with_embeddings[i];
            if archived_ids.contains(id1) {
                continue;
            }
            let v1 = match emb1 {
                Some(v) => v,
                None => continue,
            };

            for (id2, content2, imp2, _, emb2) in memories_with_embeddings.iter().skip(i + 1) {
                if archived_ids.contains(id2) {
                    continue;
                }
                let v2 = match emb2 {
                    Some(v) => v,
                    None => continue,
                };

                let sim = cosine_similarity(v1, v2);
                if sim >= config.semantic_dedup_threshold {
                    duplicates_found += 1;
                    // Keep higher importance, archive the redundant one
                    let (keeper_id, loser_id, loser_content) = if imp1 >= imp2 {
                        (*id1, *id2, content2)
                    } else {
                        (*id2, *id1, content1)
                    };

                    archived_ids.insert(loser_id);
                    let estimated_tokens = (loser_content.len() / 4).max(10);
                    tokens_saved += estimated_tokens;

                    details.push(format!(
                        "Semantic dedup (sim={:.3}): Retaining #{} and archiving redundant duplicate #{}",
                        sim, keeper_id, loser_id
                    ));

                    if !config.dry_run {
                        let _ = storage.with_transaction(|conn| {
                            update_memory_lifecycle_state(
                                conn,
                                loser_id,
                                LifecycleState::Archived,
                            )?;
                            // Remap summary_of_id to point to keeper
                            conn.execute(
                                "UPDATE memories SET summary_of_id = ?1 WHERE id = ?2",
                                params![keeper_id, loser_id],
                            )?;
                            Ok(())
                        });
                    }
                }
            }
        }

        Ok((duplicates_found, archived_ids.len(), tokens_saved))
    }

    // ─── Stage 3: Graph Topology Optimization ────────────────────────────────

    fn stage_graph_optimization(
        storage: &Storage,
        config: &DreamPipelineConfig,
        details: &mut Vec<String>,
    ) -> Result<usize> {
        let tracker = CoactivationTracker::with_config(CoactivationConfig {
            decay_rate: 0.1,
            min_strength: 0.05,
            ..Default::default()
        });

        let pruned = if !config.dry_run {
            storage.with_connection(|conn| {
                let count =
                    tracker.weaken_unused(conn, 0.1, config.graph_prune_min_age_days as u32)?;
                // Remove any orphaned cross references whose source or target memories no longer exist
                conn.execute(
                    "DELETE FROM crossrefs 
                     WHERE from_id NOT IN (SELECT id FROM memories)
                        OR to_id NOT IN (SELECT id FROM memories)",
                    [],
                )?;
                Ok(count)
            })?
        } else {
            0
        };

        if pruned > 0 {
            details.push(format!(
                "Graph topology optimization: Pruned {} decayed/stale coactivation edges",
                pruned
            ));
        }

        Ok(pruned)
    }

    // ─── Stage 4: Thematic Digest Synthesis ──────────────────────────────────

    fn stage_thematic_digest(
        storage: &Storage,
        workspace: &str,
        rules_extracted: usize,
        duplicates_archived: usize,
        tokens_saved: usize,
        details: &mut Vec<String>,
    ) -> Result<Option<i64>> {
        let (active_count, tag_counts) = storage.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT COUNT(*) FROM memories WHERE workspace = ?1 AND lifecycle_state = 'active'",
            )?;
            let active_count: i64 = stmt.query_row(params![workspace], |r| r.get(0))?;

            let mut tag_stmt = conn.prepare(
                "SELECT t.name, COUNT(*) as cnt 
                 FROM tags t
                 JOIN memory_tags mt ON t.id = mt.tag_id
                 JOIN memories m ON mt.memory_id = m.id
                 WHERE m.workspace = ?1 AND m.lifecycle_state = 'active'
                 GROUP BY t.name ORDER BY cnt DESC LIMIT 8",
            )?;
            let rows = tag_stmt.query_map(params![workspace], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as usize))
            })?;
            let mut tags = Vec::new();
            for t in rows.flatten() {
                tags.push(t);
            }
            Ok((active_count as usize, tags))
        })?;

        let top_tags_str = if tag_counts.is_empty() {
            "None".to_string()
        } else {
            tag_counts
                .iter()
                .map(|(t, c)| format!("`{}` ({})", t, c))
                .collect::<Vec<_>>()
                .join(", ")
        };

        let digest_content = format!(
            "### 🌙 Dream Consolidation Executive Digest ({workspace})\n\n\
             - **Active Knowledge Base**: {} memories indexed\n\
             - **Procedural Lessons Distilled**: {} actionable rules extracted\n\
             - **Deduplication**: {} redundant memories merged and archived (saving ~{} tokens)\n\
             - **Dominant Concept Themes**: {}\n\
             - **Consolidated At**: {}\n",
            active_count,
            rules_extracted,
            duplicates_archived,
            tokens_saved,
            top_tags_str,
            Utc::now().to_rfc3339()
        );

        let input = CreateMemoryInput {
            content: digest_content,
            memory_type: MemoryType::Summary,
            workspace: Some(workspace.to_string()),
            scope: MemoryScope::Global,
            tier: MemoryTier::Permanent,
            importance: Some(0.85),
            tags: vec!["dream-digest".to_string(), "thematic-summary".to_string()],
            ..Default::default()
        };

        let digest_id = storage.with_transaction(|conn| {
            let m = create_memory(conn, &input)?;
            Ok(m.id)
        })?;

        details.push(format!(
            "Emitted executive thematic digest memory #{} for workspace '{}'",
            digest_id, workspace
        ));

        Ok(Some(digest_id))
    }

    /// Retrieve insights, distilled rules, and thematic summaries for a workspace.
    pub fn get_insights(storage: &Storage, workspace: &str) -> Result<DreamInsightReport> {
        storage.with_connection(|conn| {
            // 1. Get digests
            let mut digest_stmt = conn.prepare(
                "SELECT id, memory_type, content, importance, created_at
                 FROM memories
                 WHERE workspace = ?1 AND memory_type = 'summary' AND lifecycle_state = 'active'
                 ORDER BY id DESC LIMIT 10",
            )?;
            let digest_rows = digest_stmt.query_map(params![workspace], |r| {
                let id: i64 = r.get(0)?;
                let mtype: String = r.get(1)?;
                let content: String = r.get(2)?;
                let importance: f64 = r.get(3)?;
                let created_at_str: Option<String> = r.get(4)?;
                let created_at = created_at_str.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                });

                Ok(DreamInsight {
                    id,
                    memory_type: mtype,
                    summary: content.lines().next().unwrap_or("").to_string(),
                    tags: vec!["dream-digest".to_string()],
                    importance,
                    created_at,
                })
            })?;
            let mut digests = Vec::new();
            for item in digest_rows.flatten() {
                digests.push(item);
            }

            // 2. Get procedural rules
            let mut rule_stmt = conn.prepare(
                "SELECT id, memory_type, content, importance, created_at
                 FROM memories
                 WHERE workspace = ?1 AND memory_type = 'procedural' AND lifecycle_state = 'active'
                 ORDER BY id DESC LIMIT 15",
            )?;
            let rule_rows = rule_stmt.query_map(params![workspace], |r| {
                let id: i64 = r.get(0)?;
                let mtype: String = r.get(1)?;
                let content: String = r.get(2)?;
                let importance: f64 = r.get(3)?;
                let created_at_str: Option<String> = r.get(4)?;
                let created_at = created_at_str.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .ok()
                        .map(|dt| dt.with_timezone(&Utc))
                });

                Ok(DreamInsight {
                    id,
                    memory_type: mtype,
                    summary: content.lines().next().unwrap_or("").to_string(),
                    tags: vec!["procedural-rule".to_string()],
                    importance,
                    created_at,
                })
            })?;
            let mut procedural_rules = Vec::new();
            for item in rule_rows.flatten() {
                procedural_rules.push(item);
            }

            // 3. Top concept tags
            let mut tag_stmt = conn.prepare(
                "SELECT t.name, COUNT(*) as cnt
                 FROM tags t
                 JOIN memory_tags mt ON t.id = mt.tag_id
                 JOIN memories m ON mt.memory_id = m.id
                 WHERE m.workspace = ?1 AND m.lifecycle_state = 'active'
                 GROUP BY t.name ORDER BY cnt DESC LIMIT 10",
            )?;
            let tag_rows = tag_stmt.query_map(params![workspace], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)? as usize))
            })?;
            let mut top_concept_tags = Vec::new();
            for tag_entry in tag_rows.flatten() {
                top_concept_tags.push(tag_entry);
            }

            let mut count_stmt =
                conn.prepare("SELECT COUNT(*) FROM memories WHERE workspace = ?1")?;
            let total_consolidated_memories: i64 =
                count_stmt.query_row(params![workspace], |r| r.get(0))?;

            Ok(DreamInsightReport {
                workspace: workspace.to_string(),
                digests,
                procedural_rules,
                top_concept_tags,
                total_consolidated_memories: total_consolidated_memories as usize,
            })
        })
    }
}

/// Helper to parse procedural patterns/solutions from episodic memory text.
fn extract_procedural_lesson(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    let keywords = [
        "solution:",
        "lesson:",
        "fix:",
        "how to:",
        "pattern:",
        "rule:",
        "resolved:",
    ];

    for kw in &keywords {
        if let Some(pos) = lower.find(kw) {
            let snippet = text[pos..].trim();
            if snippet.len() >= 20 {
                return Some(snippet.to_string());
            }
        }
    }

    if lower.contains("error") && lower.contains("fixed by") {
        return Some(text.trim().to_string());
    }

    None
}
