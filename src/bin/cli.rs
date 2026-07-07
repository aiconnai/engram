//! Engram CLI
//!
//! Command-line interface for memory management.

use std::io::{self, Write};
#[cfg(feature = "onnx-embed")]
use std::path::Path;

use clap::{Parser, Subcommand};

#[cfg(feature = "attestation")]
use engram::attestation::{AttestationChain, AttestationFilter};
use engram::embedding::{
    create_embedder, run_embedding_queue_hygiene, EmbeddingQueueHygieneConfig,
    EmbeddingQueueHygieneReport, DEFAULT_COMPLETE_RETENTION,
};
use engram::error::Result;
use engram::graph::KnowledgeGraph;
use engram::search::{hybrid_search, SearchConfig};
#[cfg(feature = "snapshot")]
use engram::snapshot::{LoadStrategy, SnapshotBuilder, SnapshotLoader};
use engram::storage::queries::*;
use engram::storage::{health_check_storage, HealthStatus, Storage};
use engram::types::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "onnx-embed")]
use sha2::{Digest, Sha256};
#[cfg(feature = "snapshot")]
use std::str::FromStr as _;

#[derive(Parser)]
#[command(name = "engram")]
#[command(about = "AI Memory Infrastructure CLI")]
#[command(version)]
struct Cli {
    /// Database path
    #[arg(
        long,
        env = "ENGRAM_DB_PATH",
        default_value = "~/.local/share/engram/memories.db"
    )]
    db_path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new memory
    Create {
        /// Content to remember
        content: String,
        /// Memory type
        #[arg(short, long, default_value = "note")]
        r#type: String,
        /// Tags (comma-separated)
        #[arg(short = 'T', long)]
        tags: Option<String>,
        /// Importance (0-1)
        #[arg(short, long)]
        importance: Option<f32>,
    },
    /// Get a memory by ID
    Get {
        /// Memory ID
        id: i64,
    },
    /// List memories
    List {
        /// Maximum number to return
        #[arg(short, long, default_value = "20")]
        limit: i64,
        /// Filter by tags (comma-separated)
        #[arg(short = 'T', long)]
        tags: Option<String>,
        /// Filter by type
        #[arg(short, long)]
        r#type: Option<String>,
    },
    /// Search memories
    Search {
        /// Search query
        query: String,
        /// Maximum results
        #[arg(short, long, default_value = "10")]
        limit: i64,
        /// Show match explanations
        #[arg(short, long)]
        explain: bool,
    },
    /// Delete a memory
    Delete {
        /// Memory ID
        id: i64,
    },
    /// Show statistics
    Stats,
    /// Read-only maintenance and storage diagnostics
    Maintenance {
        #[command(subcommand)]
        action: MaintenanceAction,
    },
    /// Export knowledge graph
    Graph {
        /// Output format (html, json)
        #[arg(short, long, default_value = "html")]
        format: String,
        /// Output file (- for stdout)
        #[arg(short, long, default_value = "-")]
        output: String,
        /// Maximum nodes
        #[arg(short, long, default_value = "500")]
        max_nodes: i64,
    },
    /// Link two memories
    Link {
        /// Source memory ID
        from: i64,
        /// Target memory ID
        to: i64,
        /// Relationship type
        #[arg(short, long, default_value = "related_to")]
        edge_type: String,
    },
    /// Show version history
    Versions {
        /// Memory ID
        id: i64,
    },
    /// Interactive mode
    Interactive,
    /// Manage local embedding models
    #[cfg(feature = "onnx-embed")]
    Model {
        #[command(subcommand)]
        action: ModelAction,
    },
    /// Create, load, or inspect .egm snapshots
    #[cfg(feature = "snapshot")]
    Snapshot {
        #[command(subcommand)]
        action: SnapshotAction,
    },
    /// Log and verify document attestations
    #[cfg(feature = "attestation")]
    Attest {
        #[command(subcommand)]
        action: AttestAction,
    },
}

#[derive(Subcommand)]
enum MaintenanceAction {
    /// Show storage health and maintenance status without mutating the database
    Status {
        /// Emit JSON instead of human-readable output
        #[arg(long)]
        json: bool,
    },
    /// Report (dry-run) or apply compaction: prune queue, checkpoint WAL, and
    /// VACUUM when there is enough free disk space
    Compact {
        /// Perform the operations (default is a read-only dry-run)
        #[arg(long)]
        apply: bool,
        /// Emit JSON instead of human-readable text
        #[arg(long)]
        json: bool,
    },
    /// Report (dry-run) or apply a rebuild of derived indexes — the FTS index
    /// and embedding requeue. Canonical memories are never touched
    Rebuild {
        /// Rebuild the FTS index (if neither flag is set, both are rebuilt)
        #[arg(long)]
        fts: bool,
        /// Requeue memories missing embeddings (if neither flag is set, both)
        #[arg(long)]
        embeddings: bool,
        /// Perform the rebuild (default is a read-only dry-run)
        #[arg(long)]
        apply: bool,
        /// Emit JSON instead of human-readable text
        #[arg(long)]
        json: bool,
    },
    /// Run explicit queue hygiene: stale repair, optional failed retries, and complete-row retention pruning
    QueueHygiene {
        /// Explicitly requeue failed rows with retry budget left
        #[arg(long)]
        requeue_failed: bool,
        /// Perform writes (default is a read-only dry-run)
        #[arg(long)]
        apply: bool,
        /// Keep dry-run explicit when you do not want to mutate
        #[arg(long)]
        dry_run: bool,
        /// Emit JSON instead of human-readable output
        #[arg(long)]
        json: bool,
    },
}

#[cfg(feature = "snapshot")]
#[derive(Subcommand)]
enum SnapshotAction {
    /// Create a snapshot
    Create {
        /// Output path for the .egm file
        #[arg(short, long)]
        output: String,
        /// Workspace to snapshot
        #[arg(short, long)]
        workspace: Option<String>,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Load a snapshot
    Load {
        /// Path to .egm file
        path: String,
        /// Load strategy: merge, replace, isolate, dry_run
        #[arg(short, long, default_value = "merge")]
        strategy: String,
        /// Target workspace
        #[arg(short = 'w', long)]
        target_workspace: Option<String>,
    },
    /// Inspect a snapshot
    Inspect {
        /// Path to .egm file
        path: String,
    },
}

#[cfg(feature = "attestation")]
#[derive(Subcommand)]
enum AttestAction {
    /// Log document attestation
    Log {
        /// Path to document file
        path: String,
        /// Document name
        #[arg(short, long)]
        name: Option<String>,
        /// Agent ID
        #[arg(short, long)]
        agent_id: Option<String>,
    },
    /// Verify a document was attested
    Verify {
        /// Path to document file
        path: String,
    },
    /// Verify the attestation chain
    ChainVerify,
    /// List attestation records
    List {
        /// Maximum records
        #[arg(short, long, default_value = "50")]
        limit: usize,
        /// Export format: json, csv
        #[arg(short, long)]
        format: Option<String>,
    },
}

#[cfg(feature = "onnx-embed")]
#[derive(Subcommand)]
enum ModelAction {
    /// Download a local embedding model into the cache
    Download {
        /// Model registry name
        #[arg(default_value = "minilm-l6-v2")]
        name: String,
    },
    /// List local embedding models
    List,
    /// Print the local cache path for a model
    Path {
        /// Model registry name
        #[arg(default_value = "minilm-l6-v2")]
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Expand ~ in path
    let db_path = shellexpand::tilde(&cli.db_path).to_string();

    let config = StorageConfig {
        db_path,
        storage_mode: StorageMode::Local,
        cloud_uri: None,
        encrypt_cloud: false,
        confidence_half_life_days: 30.0,
        auto_sync: false,
        sync_debounce_ms: 5000,
    };

    #[cfg(feature = "onnx-embed")]
    if let Commands::Model { action } = &cli.command {
        return handle_model_action(action);
    }

    let storage = Storage::open(config)?;

    match cli.command {
        Commands::Create {
            content,
            r#type,
            tags,
            importance,
        } => {
            let memory_type: MemoryType = r#type.parse().unwrap_or(MemoryType::Note);
            let tags: Vec<String> = tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            let input = CreateMemoryInput {
                content,
                memory_type,
                tags,
                metadata: Default::default(),
                importance,
                scope: Default::default(),
                workspace: None,
                tier: Default::default(),
                defer_embedding: true,
                ttl_seconds: None,
                dedup_mode: Default::default(),
                dedup_threshold: None,
                event_time: None,
                event_duration_seconds: None,
                trigger_pattern: None,
                summary_of_id: None,
                media_url: None,
            };

            let memory = storage.with_transaction(|conn| create_memory(conn, &input))?;
            println!("Created memory #{}", memory.id);
            println!("{}", serde_json::to_string_pretty(&memory)?);
        }

        Commands::Get { id } => {
            let memory = storage.with_connection(|conn| get_memory(conn, id))?;
            println!("{}", serde_json::to_string_pretty(&memory)?);
        }

        Commands::List {
            limit,
            tags,
            r#type,
        } => {
            let tags: Option<Vec<String>> =
                tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());
            let memory_type = r#type.and_then(|t| t.parse().ok());

            let options = ListOptions {
                limit: Some(limit),
                tags,
                memory_type,
                ..Default::default()
            };

            let memories = storage.with_connection(|conn| list_memories(conn, &options))?;
            for memory in memories {
                println!(
                    "#{} [{}] {} - {}",
                    memory.id,
                    memory.memory_type.as_str(),
                    memory.tags.join(", "),
                    truncate(&memory.content, 60)
                );
            }
        }

        Commands::Search {
            query,
            limit,
            explain,
        } => {
            let embedding_config = EmbeddingConfig::default();
            let embedder = create_embedder(&embedding_config)?;
            let query_embedding = embedder.embed(&query).ok();

            let options = SearchOptions {
                limit: Some(limit),
                explain,
                ..Default::default()
            };

            let config = SearchConfig::default();
            let results = storage.with_connection(|conn| {
                hybrid_search(conn, &query, query_embedding.as_deref(), &options, &config)
            })?;

            for result in results {
                println!(
                    "#{} (score: {:.3}) - {}",
                    result.memory.id,
                    result.score,
                    truncate(&result.memory.content, 60)
                );
                if explain {
                    println!(
                        "  Strategy: {:?}, Matched: {:?}",
                        result.match_info.strategy, result.match_info.matched_terms
                    );
                }
            }
        }

        Commands::Delete { id } => {
            storage.with_transaction(|conn| delete_memory(conn, id))?;
            println!("Deleted memory #{}", id);
        }

        Commands::Stats => {
            let stats = storage.with_connection(get_stats)?;
            println!("{}", serde_json::to_string_pretty(&stats)?);
        }

        Commands::Maintenance { action } => match action {
            MaintenanceAction::Status { json } => {
                let status = maintenance_status(&storage)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&status)?);
                } else {
                    print_maintenance_status(&status);
                }
            }
            MaintenanceAction::QueueHygiene {
                requeue_failed,
                apply,
                dry_run,
                json,
            } => {
                if apply && dry_run {
                    eprintln!("--apply and --dry-run are mutually exclusive");
                    std::process::exit(1);
                }
                if dry_run {
                    eprintln!("WARNING: --dry-run requested explicitly; mutation will be skipped.");
                }
                let r = run_embedding_queue_maintenance(&storage, requeue_failed, apply)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&r)?);
                } else {
                    print_maintenance_queue_hygiene(&r, apply);
                }
            }
            MaintenanceAction::Compact { apply, json } => {
                let r = storage.compact(apply)?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&r)?);
                } else {
                    println!(
                        "Storage compaction ({})",
                        if r.applied { "APPLIED" } else { "dry-run" }
                    );
                    println!("  database size:        {} bytes", r.db_size_bytes);
                    println!(
                        "  WAL / SHM:            {} / {} bytes",
                        r.wal_bytes, r.shm_bytes
                    );
                    println!(
                        "  reclaimable (VACUUM): {} bytes ({} free page(s))",
                        r.reclaimable_bytes, r.freelist_count
                    );
                    println!(
                        "  queue prunable:       {} complete, {} failed",
                        r.queue_complete_prunable, r.queue_failed_prunable
                    );
                    println!("  orphan embeddings:    {}", r.orphan_embeddings);
                    let free = if r.free_space_bytes < 0 {
                        "unknown".to_string()
                    } else {
                        format!("{} bytes", r.free_space_bytes)
                    };
                    println!(
                        "  free space:           {} (vacuum safe: {})",
                        free, r.vacuum_safe
                    );
                    println!("  operations:");
                    for op in &r.operations {
                        let status = if op.applied {
                            "applied".to_string()
                        } else if let Some(reason) = &op.skipped_reason {
                            format!("skipped ({reason})")
                        } else {
                            "dry-run".to_string()
                        };
                        println!(
                            "    - {:<22} candidates={} [{}]",
                            op.name, op.candidates, status
                        );
                    }
                    if !r.applied {
                        println!("  (dry-run; re-run with --apply to execute)");
                    }
                }
            }
            MaintenanceAction::Rebuild {
                fts,
                embeddings,
                apply,
                json,
            } => {
                // If neither target is named, rebuild both.
                let (do_fts, do_embeddings) = if !fts && !embeddings {
                    (true, true)
                } else {
                    (fts, embeddings)
                };
                let r = storage.with_transaction(|conn| {
                    rebuild_derived_indexes(conn, do_fts, do_embeddings, apply)
                })?;
                if json {
                    println!("{}", serde_json::to_string_pretty(&r)?);
                } else {
                    println!(
                        "Derived-index rebuild ({})",
                        if r.applied { "APPLIED" } else { "dry-run" }
                    );
                    println!("  live memories (preserved): {}", r.memories);
                    if r.fts_targeted {
                        println!(
                            "  FTS: indexed {} -> {}, drift {} -> {} (rebuilt: {})",
                            r.fts_indexed_before,
                            r.fts_indexed_after,
                            r.fts_drift_before,
                            r.fts_drift_after,
                            r.fts_rebuilt
                        );
                    }
                    if r.embeddings_targeted {
                        println!(
                            "  embeddings: {} present, {} missing, {} requeued",
                            r.embeddings_present, r.embeddings_missing, r.embeddings_requeued
                        );
                    }
                    if !r.applied {
                        println!("  (dry-run; re-run with --apply to execute)");
                    }
                }
            }
        },

        Commands::Graph {
            format,
            output,
            max_nodes,
        } => {
            let options = ListOptions {
                limit: Some(max_nodes),
                ..Default::default()
            };

            let (memories, crossrefs) = storage.with_connection(|conn| {
                let memories = list_memories(conn, &options)?;
                let mut all_crossrefs = Vec::new();
                for memory in &memories {
                    if let Ok(refs) = get_related(conn, memory.id) {
                        all_crossrefs.extend(refs);
                    }
                }
                Ok((memories, all_crossrefs))
            })?;

            let graph = KnowledgeGraph::from_data(&memories, &crossrefs);

            let content = match format.as_str() {
                "json" => serde_json::to_string_pretty(&graph.to_visjs_json())?,
                _ => graph.to_html(),
            };

            if output == "-" {
                println!("{}", content);
            } else {
                std::fs::write(&output, content)?;
                println!("Graph exported to {}", output);
            }
        }

        Commands::Link {
            from,
            to,
            edge_type,
        } => {
            let edge_type: EdgeType = edge_type.parse().unwrap_or(EdgeType::RelatedTo);
            let input = CreateCrossRefInput {
                from_id: from,
                to_id: to,
                edge_type,
                strength: None,
                source_context: None,
                pinned: false,
            };

            storage.with_transaction(|conn| create_crossref(conn, &input))?;
            println!("Linked #{} -> #{} ({})", from, to, edge_type.as_str());
        }

        Commands::Versions { id } => {
            let versions = storage.with_connection(|conn| get_memory_versions(conn, id))?;
            for version in versions {
                println!(
                    "v{} ({}) - {}",
                    version.version,
                    version.created_at.format("%Y-%m-%d %H:%M"),
                    truncate(&version.content, 50)
                );
            }
        }

        #[cfg(feature = "snapshot")]
        Commands::Snapshot { action } => match action {
            SnapshotAction::Create {
                output,
                workspace,
                description,
            } => {
                let mut builder = SnapshotBuilder::new(storage.clone());
                if let Some(ws) = workspace {
                    builder = builder.workspace(&ws);
                }
                if let Some(desc) = description {
                    builder = builder.description(&desc);
                }
                let path = std::path::Path::new(&output);
                match builder.build(path) {
                    Ok(manifest) => {
                        println!(
                            "Snapshot created: {} ({} memories)",
                            output, manifest.memory_count
                        );
                        println!("{}", serde_json::to_string_pretty(&manifest)?);
                    }
                    Err(e) => {
                        eprintln!("Error creating snapshot: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            SnapshotAction::Load {
                path,
                strategy,
                target_workspace,
            } => {
                let load_strategy = match LoadStrategy::from_str(&strategy) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Invalid strategy '{}': {}", strategy, e);
                        std::process::exit(1);
                    }
                };
                let p = std::path::Path::new(&path);
                match SnapshotLoader::load(
                    &storage,
                    p,
                    load_strategy,
                    target_workspace.as_deref(),
                    None,
                ) {
                    Ok(result) => {
                        println!(
                            "Loaded {} memories, {} skipped",
                            result.memories_loaded, result.memories_skipped
                        );
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    }
                    Err(e) => {
                        eprintln!("Error loading snapshot: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            SnapshotAction::Inspect { path } => {
                let p = std::path::Path::new(&path);
                match SnapshotLoader::inspect(p) {
                    Ok(info) => {
                        println!("Snapshot: {}", path);
                        println!("  File size: {} bytes", info.file_size_bytes);
                        println!("  Memories:  {}", info.manifest.memory_count);
                        println!("  Entities:  {}", info.manifest.entity_count);
                        println!("  Edges:     {}", info.manifest.edge_count);
                        println!("  Created:   {}", info.manifest.created_at.to_rfc3339());
                        if let Some(desc) = &info.manifest.description {
                            println!("  Desc:      {}", desc);
                        }
                        println!("  Encrypted: {}", info.manifest.encrypted);
                        println!("  Signed:    {}", info.manifest.signed);
                    }
                    Err(e) => {
                        eprintln!("Error inspecting snapshot: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        },

        #[cfg(feature = "attestation")]
        Commands::Attest { action } => match action {
            AttestAction::Log {
                path,
                name,
                agent_id,
            } => {
                let content = std::fs::read(&path)?;
                let doc_name = name.unwrap_or_else(|| path.clone());
                let chain = AttestationChain::new(storage.clone());
                match chain.log_document(&content, &doc_name, agent_id.as_deref(), &[], None) {
                    Ok(record) => {
                        println!("Attested: {}", doc_name);
                        println!("{}", serde_json::to_string_pretty(&record)?);
                    }
                    Err(e) => {
                        eprintln!("Error logging attestation: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            AttestAction::Verify { path } => {
                let content = std::fs::read(&path)?;
                let chain = AttestationChain::new(storage.clone());
                match chain.verify_document(&content) {
                    Ok(Some(record)) => {
                        println!("Attested: YES");
                        println!("{}", serde_json::to_string_pretty(&record)?);
                    }
                    Ok(None) => {
                        println!("Attested: NO — document not found in attestation chain");
                    }
                    Err(e) => {
                        eprintln!("Error verifying attestation: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            AttestAction::ChainVerify => {
                let chain = AttestationChain::new(storage.clone());
                match chain.verify_chain(None) {
                    Ok(status) => {
                        println!("{}", serde_json::to_string_pretty(&status)?);
                    }
                    Err(e) => {
                        eprintln!("Error verifying chain: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            AttestAction::List { limit, format } => {
                let filter = AttestationFilter {
                    limit: Some(limit),
                    offset: Some(0),
                    agent_id: None,
                    document_name: None,
                };
                let chain = AttestationChain::new(storage.clone());
                match chain.list(&filter) {
                    Ok(records) => {
                        if let Some("csv") = format.as_deref() {
                            match engram::attestation::export_csv(&records) {
                                Ok(csv) => println!("{}", csv),
                                Err(e) => {
                                    eprintln!("Export error: {}", e);
                                    std::process::exit(1);
                                }
                            }
                        } else {
                            println!("{}", serde_json::to_string_pretty(&records)?);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error listing attestations: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        },

        Commands::Interactive => {
            println!("Engram Interactive Mode");
            println!("Type 'help' for commands, 'quit' to exit\n");

            let stdin = io::stdin();
            let mut stdout = io::stdout();

            loop {
                print!("engram> ");
                stdout.flush()?;

                let mut line = String::new();
                stdin.read_line(&mut line)?;
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                match line {
                    "quit" | "exit" => break,
                    "help" => {
                        println!("Commands:");
                        println!("  create <content>  - Create a memory");
                        println!("  get <id>          - Get memory by ID");
                        println!("  list              - List recent memories");
                        println!("  search <query>    - Search memories");
                        println!("  stats             - Show statistics");
                        println!("  quit              - Exit");
                    }
                    "stats" => {
                        let stats = storage.with_connection(get_stats)?;
                        println!("Memories: {}", stats.total_memories);
                        println!("Tags: {}", stats.total_tags);
                        println!("Cross-refs: {}", stats.total_crossrefs);
                    }
                    "list" => {
                        let options = ListOptions {
                            limit: Some(10),
                            ..Default::default()
                        };
                        let memories =
                            storage.with_connection(|conn| list_memories(conn, &options))?;
                        for memory in memories {
                            println!("#{}: {}", memory.id, truncate(&memory.content, 60));
                        }
                    }
                    _ if line.starts_with("get ") => {
                        if let Ok(id) = line[4..].trim().parse::<i64>() {
                            match storage.with_connection(|conn| get_memory(conn, id)) {
                                Ok(memory) => {
                                    println!("{}", serde_json::to_string_pretty(&memory)?);
                                }
                                Err(e) => println!("Error: {}", e),
                            }
                        } else {
                            println!("Invalid ID");
                        }
                    }
                    _ if line.starts_with("create ") => {
                        let content = line[7..].trim();
                        let input = CreateMemoryInput {
                            content: content.to_string(),
                            memory_type: MemoryType::Note,
                            tags: vec![],
                            metadata: Default::default(),
                            importance: None,
                            scope: Default::default(),
                            workspace: None,
                            tier: Default::default(),
                            defer_embedding: true,
                            ttl_seconds: None,
                            dedup_mode: Default::default(),
                            dedup_threshold: None,
                            event_time: None,
                            event_duration_seconds: None,
                            trigger_pattern: None,
                            summary_of_id: None,
                            media_url: None,
                        };
                        match storage.with_transaction(|conn| create_memory(conn, &input)) {
                            Ok(memory) => println!("Created #{}", memory.id),
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                    _ if line.starts_with("search ") => {
                        let query = line[7..].trim();
                        let embedding_config = EmbeddingConfig::default();
                        let embedder = create_embedder(&embedding_config)?;
                        let query_embedding = embedder.embed(query).ok();

                        let options = SearchOptions {
                            limit: Some(5),
                            ..Default::default()
                        };
                        let config = SearchConfig::default();

                        match storage.with_connection(|conn| {
                            hybrid_search(
                                conn,
                                query,
                                query_embedding.as_deref(),
                                &options,
                                &config,
                            )
                        }) {
                            Ok(results) => {
                                for result in results {
                                    println!(
                                        "#{} ({:.2}): {}",
                                        result.memory.id,
                                        result.score,
                                        truncate(&result.memory.content, 50)
                                    );
                                }
                            }
                            Err(e) => println!("Error: {}", e),
                        }
                    }
                    _ => println!("Unknown command. Type 'help' for available commands."),
                }
            }

            println!("Goodbye!");
        }

        #[cfg(feature = "onnx-embed")]
        Commands::Model { .. } => unreachable!("model commands are handled before storage opens"),
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MaintenanceStatus {
    #[serde(flatten)]
    health: HealthStatus,
    stats: MaintenanceStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MaintenanceStats {
    total_memories: i64,
    total_tags: i64,
    total_crossrefs: i64,
    total_versions: i64,
    memories_with_embeddings: i64,
    memories_pending_embedding: i64,
    sync_pending: bool,
    storage_mode: String,
    schema_version: i32,
    db_size_bytes: i64,
}

fn maintenance_status(storage: &Storage) -> Result<MaintenanceStatus> {
    let health = storage_health(storage)?;
    let stats = storage.with_connection(get_stats)?;

    Ok(MaintenanceStatus {
        health,
        stats: MaintenanceStats {
            total_memories: stats.total_memories,
            total_tags: stats.total_tags,
            total_crossrefs: stats.total_crossrefs,
            total_versions: stats.total_versions,
            memories_with_embeddings: stats.memories_with_embeddings,
            memories_pending_embedding: stats.memories_pending_embedding,
            sync_pending: stats.sync_pending,
            storage_mode: stats.storage_mode,
            schema_version: stats.schema_version,
            db_size_bytes: stats.db_size_bytes,
        },
    })
}

fn run_embedding_queue_maintenance(
    storage: &Storage,
    requeue_failed: bool,
    apply: bool,
) -> Result<EmbeddingQueueHygieneReport> {
    let config = EmbeddingQueueHygieneConfig {
        complete_retention: DEFAULT_COMPLETE_RETENTION,
        ..Default::default()
    };
    storage.with_connection(|conn| {
        run_embedding_queue_hygiene(conn, &config, requeue_failed, apply, true)
    })
}

fn storage_health(storage: &Storage) -> Result<HealthStatus> {
    health_check_storage(storage)
}

fn json_enum_name<T: Serialize + std::fmt::Debug>(value: T) -> String {
    serde_json::to_string(&value)
        .unwrap_or_else(|_| format!("{value:?}"))
        .trim_matches('"')
        .to_string()
}

fn write_maintenance_status<W: std::io::Write>(
    mut writer: W,
    status: &MaintenanceStatus,
) -> Result<()> {
    writeln!(
        writer,
        "Storage: {}",
        if status.health.healthy {
            "healthy"
        } else {
            "unhealthy"
        }
    )?;
    writeln!(writer, "Latency: {:.2} ms", status.health.latency_ms)?;
    writeln!(writer, "Database: {}", status.health.details["db_path"])?;
    writeln!(writer, "Storage mode: {}", status.stats.storage_mode)?;
    writeln!(writer, "Schema version: {}", status.stats.schema_version)?;
    if let Some(quick_check) = status.health.details.get("quick_check") {
        writeln!(writer, "PRAGMA quick_check: {}", quick_check)?;
    }
    if let (
        Some(page_size),
        Some(page_count),
        Some(db_size_bytes),
        Some(freelist_count),
        Some(reclaimable_bytes),
    ) = (
        status.health.details.get("page_size"),
        status.health.details.get("page_count"),
        status.health.details.get("db_size_bytes"),
        status.health.details.get("freelist_count"),
        status.health.details.get("reclaimable_bytes"),
    ) {
        writeln!(
            writer,
            "Database pages: {} pages @ {} bytes ({} free, {} reclaimable bytes)",
            page_count, page_size, freelist_count, reclaimable_bytes
        )?;
        writeln!(writer, "Database size: {} bytes", db_size_bytes)?;
    } else {
        writeln!(
            writer,
            "Database size: {} bytes",
            status.stats.db_size_bytes
        )?;
    }
    if let Some(warning) = status.health.details.get("warning") {
        writeln!(writer, "Warning: {}", warning)?;
    }
    writeln!(writer, "Memories: {}", status.stats.total_memories)?;
    writeln!(
        writer,
        "Embeddings: {} ready, {} pending",
        status.stats.memories_with_embeddings, status.stats.memories_pending_embedding
    )?;
    writeln!(writer, "Tags: {}", status.stats.total_tags)?;
    writeln!(writer, "Cross-refs: {}", status.stats.total_crossrefs)?;
    writeln!(writer, "Versions: {}", status.stats.total_versions)?;
    writeln!(writer, "Sync pending: {}", status.stats.sync_pending)?;

    if !status.health.derived_indexes.is_empty() {
        writeln!(writer, "Derived indexes:")?;
        for index in &status.health.derived_indexes {
            writeln!(
                writer,
                "  {} ({}): {} source={} indexed={} pending={} stale={} failed={} orphaned={}",
                index.name,
                json_enum_name(index.kind),
                json_enum_name(index.status),
                index.source_count,
                index.indexed_count,
                index.pending_count,
                index.stale_count,
                index.failed_count,
                index.orphaned_count
            )?;

            if index.name == "embeddings" {
                let pending = index
                    .details
                    .get("pending")
                    .map(String::as_str)
                    .unwrap_or("0");
                let processing = index
                    .details
                    .get("processing")
                    .map(String::as_str)
                    .unwrap_or("0");
                let stale_processing = index
                    .details
                    .get("stale_processing")
                    .map(String::as_str)
                    .unwrap_or("0");
                let failed = index
                    .details
                    .get("failed")
                    .map(String::as_str)
                    .unwrap_or("0");
                let zero_retry_failed = index
                    .details
                    .get("zero_retry_failed")
                    .map(String::as_str)
                    .unwrap_or("0");
                let retryable_failed = index
                    .details
                    .get("retryable_failed")
                    .map(String::as_str)
                    .unwrap_or("0");
                let exhausted_failed = index
                    .details
                    .get("exhausted_failed")
                    .map(String::as_str)
                    .unwrap_or("0");
                let max_retry_count = index
                    .details
                    .get("max_retry_count")
                    .map(String::as_str)
                    .unwrap_or("0");
                let oldest_pending_age = index
                    .details
                    .get("oldest_pending_age")
                    .or_else(|| index.details.get("oldest_pending_age_seconds"))
                    .map(String::as_str)
                    .unwrap_or("none");
                let oldest_processing_age = index
                    .details
                    .get("oldest_processing_age")
                    .or_else(|| index.details.get("oldest_processing_age_seconds"))
                    .map(String::as_str)
                    .unwrap_or("none");
                let oldest_failed_age = index
                    .details
                    .get("oldest_failed_age")
                    .or_else(|| index.details.get("oldest_failed_age_seconds"))
                    .map(String::as_str)
                    .unwrap_or("none");
                let retry_count_0 = index
                    .details
                    .get("retry_count_0")
                    .map(String::as_str)
                    .unwrap_or("0");
                let retry_count_1 = index
                    .details
                    .get("retry_count_1")
                    .map(String::as_str)
                    .unwrap_or("0");
                let retry_count_2 = index
                    .details
                    .get("retry_count_2")
                    .map(String::as_str)
                    .unwrap_or("0");
                let retry_count_3_plus = index
                    .details
                    .get("retry_count_3_plus")
                    .map(String::as_str)
                    .unwrap_or("0");

                writeln!(
                    writer,
                    "    queue-state: pending={} processing={} stale_processing={} failed={} zero_retry_failed={} retryable_failed={} exhausted_failed={} max_retry_count={} oldest_pending_age={} oldest_processing_age={} oldest_failed_age={} retry_count_0={} retry_count_1={} retry_count_2={} retry_count_3+={}",
                    pending,
                    processing,
                    stale_processing,
                    failed,
                    zero_retry_failed,
                    retryable_failed,
                    exhausted_failed,
                    max_retry_count,
                    oldest_pending_age,
                    oldest_processing_age,
                    oldest_failed_age,
                    retry_count_0,
                    retry_count_1,
                    retry_count_2,
                    retry_count_3_plus,
                )?;

                let embedding_profile_rows = index
                    .details
                    .get("embedding_profile_rows")
                    .map(String::as_str)
                    .unwrap_or("0");
                let embedding_profile_bytes_total = index
                    .details
                    .get("embedding_profile_bytes_total")
                    .map(String::as_str)
                    .unwrap_or("0");
                let embedding_profile_bytes_avg = index
                    .details
                    .get("embedding_profile_bytes_avg")
                    .map(String::as_str)
                    .unwrap_or("0");
                let embedding_profile_bytes_min = index
                    .details
                    .get("embedding_profile_bytes_min")
                    .map(String::as_str)
                    .unwrap_or("0");
                let embedding_profile_bytes_max = index
                    .details
                    .get("embedding_profile_bytes_max")
                    .map(String::as_str)
                    .unwrap_or("0");
                writeln!(
                    writer,
                    "    embedding profile: rows={} total_bytes={} avg_bytes={} min_bytes={} max_bytes={}",
                    embedding_profile_rows,
                    embedding_profile_bytes_total,
                    embedding_profile_bytes_avg,
                    embedding_profile_bytes_min,
                    embedding_profile_bytes_max
                )?;
            }

            if index.name == "memories_fts" {
                let drift = index
                    .details
                    .get("drift_rows")
                    .or_else(|| index.details.get("missing_rows"))
                    .map(String::as_str)
                    .unwrap_or("0");
                writeln!(writer, "    drift: {}", drift)?;
            }
        }
    }

    if let Some(error) = &status.health.error {
        writeln!(writer, "Error: {}", error)?;
    }

    Ok(())
}

fn print_maintenance_queue_hygiene(r: &EmbeddingQueueHygieneReport, apply: bool) {
    println!(
        "Embedding queue hygiene ({})",
        if apply { "APPLIED" } else { "dry-run" }
    );
    println!("  stale rows requeued:   {}", r.requeued_stale);
    println!("  stale rows failed:     {}", r.failed_exhausted);
    println!("  failed rows requeued:  {}", r.requeued_failed);
    println!("  complete rows pruned:  {}", r.pruned_complete);
    if !apply {
        println!("  (dry-run; re-run with --apply to execute)");
    }
}

fn print_maintenance_status(status: &MaintenanceStatus) {
    if let Err(e) = write_maintenance_status(std::io::stdout(), status) {
        eprintln!("Failed to write maintenance status: {}", e);
    }
}

#[cfg(feature = "onnx-embed")]
fn handle_model_action(action: &ModelAction) -> Result<()> {
    use engram::embedding::onnx_registry::{default_model_dir, find_model, REGISTRY};
    use engram::error::EngramError;

    match action {
        ModelAction::List => {
            for entry in REGISTRY {
                let dir = default_model_dir();
                let installed = model_files_present(&dir);
                let status = if installed {
                    "installed"
                } else {
                    "not installed"
                };
                println!(
                    "{}\t{}\t{} dims\tmax_seq_len={}",
                    entry.name, status, entry.dimensions, entry.max_seq_len
                );
            }
        }
        ModelAction::Path { name } => {
            let entry = find_model(name).ok_or_else(|| {
                EngramError::InvalidInput(format!("Unknown local embedding model: {name}"))
            })?;
            let dir = default_model_dir_for(entry.name);
            println!("{}", dir.display());
        }
        ModelAction::Download { name } => {
            let entry = find_model(name).ok_or_else(|| {
                EngramError::InvalidInput(format!("Unknown local embedding model: {name}"))
            })?;
            let dir = default_model_dir_for(entry.name);
            std::fs::create_dir_all(&dir)?;

            download_file(entry.model_url, entry.model_sha256, &dir.join("model.onnx"))?;
            download_file(
                entry.tokenizer_url,
                entry.tokenizer_sha256,
                &dir.join("tokenizer.json"),
            )?;

            println!("Downloaded {} to {}", entry.name, dir.display());
        }
    }

    Ok(())
}

#[cfg(feature = "onnx-embed")]
fn default_model_dir_for(name: &str) -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("engram")
        .join("models")
        .join(name)
}

#[cfg(feature = "onnx-embed")]
fn model_files_present(dir: &Path) -> bool {
    dir.join("model.onnx").is_file() && dir.join("tokenizer.json").is_file()
}

#[cfg(feature = "onnx-embed")]
fn download_file(url: &str, expected_sha256: &str, target: &Path) -> Result<()> {
    use engram::error::EngramError;

    if target.is_file()
        && sha256_file(target)? == expected_sha256_or_current(expected_sha256, target)?
    {
        println!("{} already present", target.display());
        return Ok(());
    }

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| EngramError::Config(format!("Failed to create download runtime: {e}")))?;
    let bytes = runtime.block_on(async {
        let response = reqwest::Client::new().get(url).send().await?;
        let response = response.error_for_status()?;
        response.bytes().await
    })?;

    let actual_hash = sha256_bytes(&bytes);
    if !expected_sha256.is_empty() && actual_hash != expected_sha256 {
        return Err(EngramError::Config(format!(
            "SHA-256 mismatch for {url}: expected {expected_sha256}, got {actual_hash}"
        )));
    }

    let tmp = target.with_extension(format!("tmp.{}", std::process::id()));
    std::fs::write(&tmp, &bytes)?;
    std::fs::rename(&tmp, target)?;
    println!("Downloaded {}", target.display());

    Ok(())
}

#[cfg(feature = "onnx-embed")]
fn expected_sha256_or_current(expected: &str, target: &Path) -> Result<String> {
    if expected.is_empty() {
        sha256_file(target)
    } else {
        Ok(expected.to_string())
    }
}

#[cfg(feature = "onnx-embed")]
fn sha256_file(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path)?;
    Ok(sha256_bytes(&bytes))
}

#[cfg(feature = "onnx-embed")]
fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn truncate(s: &str, max: usize) -> String {
    let first_line = s.lines().next().unwrap_or(s);
    if first_line.len() <= max {
        first_line.to_string()
    } else {
        format!("{}...", &first_line[..max - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::{params, Connection};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn memory_input(content: &str) -> CreateMemoryInput {
        CreateMemoryInput {
            content: content.to_string(),
            memory_type: MemoryType::Note,
            tags: vec![],
            metadata: HashMap::new(),
            importance: None,
            scope: Default::default(),
            workspace: None,
            tier: Default::default(),
            defer_embedding: false,
            ttl_seconds: None,
            dedup_mode: DedupMode::Allow,
            dedup_threshold: None,
            event_time: None,
            event_duration_seconds: None,
            trigger_pattern: None,
            summary_of_id: None,
            media_url: None,
        }
    }

    fn test_storage() -> (TempDir, Storage) {
        let dir = tempfile::tempdir().expect("temporary directory should be created");
        let db_path = dir.path().join("memories.db").to_string_lossy().to_string();
        let config = StorageConfig {
            db_path,
            storage_mode: StorageMode::Local,
            cloud_uri: None,
            encrypt_cloud: false,
            confidence_half_life_days: 30.0,
            auto_sync: false,
            sync_debounce_ms: 5000,
        };
        let storage = Storage::open(config).expect("file storage should open");
        (dir, storage)
    }

    fn table_counts(conn: &Connection) -> Result<(i64, i64, i64)> {
        Ok((
            conn.query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))?,
            conn.query_row("SELECT COUNT(*) FROM memory_versions", [], |row| row.get(0))?,
            conn.query_row("SELECT COUNT(*) FROM embedding_queue", [], |row| row.get(0))?,
        ))
    }

    #[test]
    fn maintenance_status_matches_storage_health_shape() {
        let (_dir, storage) = test_storage();

        let status = maintenance_status(&storage).expect("status should be collected");
        let json = serde_json::to_value(status).expect("status should serialize");

        assert!(json["healthy"].is_boolean());
        assert!(json["latency_ms"].is_number());
        assert!(json["error"].is_null() || json["error"].is_string());
        assert!(json["details"]["db_path"]
            .as_str()
            .expect("db_path should be a string")
            .ends_with("memories.db"));
        if let Some(storage_mode) = json["details"]["storage_mode"].as_str() {
            assert_eq!(storage_mode, "Local");
        }
        assert_eq!(json["details"]["quick_check"].as_str(), Some("ok"));
        assert!(json["details"]["page_size"].is_string());
        assert!(json["details"]["page_count"].is_string());
        assert!(json["details"]["db_size_bytes"].is_string());
        assert!(json["details"]["freelist_count"].is_string());
        assert!(json["details"]["reclaimable_bytes"].is_string());
        assert!(json["derived_indexes"].is_array());
        assert_eq!(json["stats"]["total_memories"], 0);
        assert!(json["stats"]["schema_version"].is_number());

        let embedding_index = json["derived_indexes"]
            .as_array()
            .and_then(|indexes| {
                indexes
                    .iter()
                    .find(|index| index["name"].as_str() == Some("embeddings"))
            })
            .expect("embedding derived index should be present in health payload");

        let details = embedding_index["details"]
            .as_object()
            .expect("details should be object");
        for key in [
            "pending",
            "processing",
            "stale_processing",
            "failed",
            "zero_retry_failed",
            "retryable_failed",
            "exhausted_failed",
            "max_retry_count",
            "oldest_pending_age",
            "oldest_pending_age_seconds",
            "oldest_processing_age",
            "oldest_failed_age",
            "retry_count_0",
            "retry_count_1",
            "retry_count_2",
            "retry_count_3_plus",
            "embedding_profile_rows",
            "embedding_profile_bytes_total",
            "embedding_profile_bytes_avg",
            "embedding_profile_bytes_min",
            "embedding_profile_bytes_max",
        ] {
            assert!(
                details.contains_key(key),
                "details missing queue state key: {key}"
            );
        }
    }

    #[test]
    fn maintenance_status_is_read_only_for_storage_tables() {
        let (_dir, storage) = test_storage();

        let before = storage
            .with_connection(table_counts)
            .expect("initial counts should be readable");
        let _ = maintenance_status(&storage).expect("status should be collected");
        let after = storage
            .with_connection(table_counts)
            .expect("final counts should be readable");

        assert_eq!(before, after);
    }

    #[test]
    fn maintenance_status_includes_sqlite_health_contract() {
        let (_dir, storage) = test_storage();
        let status = maintenance_status(&storage).expect("status should be collected");
        let mut output = Vec::new();

        write_maintenance_status(&mut output, &status).expect("status should render");
        let text = String::from_utf8(output).expect("output should be utf8");

        assert!(text.contains("PRAGMA quick_check: ok"));
        assert!(text.contains("Database pages:"));
        assert!(text.contains("embedding profile: rows="));
        assert!(text.contains("drift:"));
    }

    #[test]
    fn maintenance_status_reports_warning_for_cloud_path() {
        let dir = tempfile::tempdir().expect("temporary directory should be created");
        let db_path = dir
            .path()
            .join("my_dropbox_backup")
            .join("memories.db")
            .to_string_lossy()
            .to_string();
        let config = StorageConfig {
            db_path,
            storage_mode: StorageMode::Local,
            cloud_uri: None,
            encrypt_cloud: false,
            confidence_half_life_days: 30.0,
            auto_sync: false,
            sync_debounce_ms: 5000,
        };
        let storage = Storage::open(config).expect("file storage should open");
        let status = maintenance_status(&storage).expect("status should be collected");

        let warning = status
            .health
            .details
            .get("warning")
            .expect("warning should be present for cloud-like path");
        assert!(warning.contains("WAL mode"));
    }

    #[test]
    fn maintenance_status_human_output_includes_derived_indexes() {
        let (_dir, storage) = test_storage();
        let status = maintenance_status(&storage).expect("status should be collected");
        let mut output = Vec::new();

        write_maintenance_status(&mut output, &status).expect("status should render");
        let text = String::from_utf8(output).expect("output should be utf8");

        assert!(text.contains("Derived indexes:"));
        assert!(text.contains("embeddings (embedding):"));
        assert!(text.contains("memories_fts (full_text):"));
        assert!(text.contains("crossrefs (graph):"));
        assert!(text.contains("source="));
        assert!(text.contains("indexed="));
        assert!(text.contains("orphaned="));
    }

    #[test]
    fn maintenance_status_human_output_includes_embedding_queue_state_counters() {
        let (_dir, storage) = test_storage();
        let stale_time = (chrono::Utc::now() - chrono::Duration::minutes(30)).to_rfc3339();
        let old_pending_time = (chrono::Utc::now() - chrono::Duration::minutes(20)).to_rfc3339();

        storage
            .with_connection(|conn| {
                let pending = create_memory(conn, &memory_input("state counter pending"))?;
                let processing =
                    create_memory(conn, &memory_input("state counter processing"))?;
                let retryable_failed =
                    create_memory(conn, &memory_input("state counter retryable"))?;
                let exhausted_failed =
                    create_memory(conn, &memory_input("state counter exhausted"))?;
                let zero_retry_failed =
                    create_memory(conn, &memory_input("state counter zero retry"))?;

                conn.execute(
                    "UPDATE embedding_queue SET status = 'processing', started_at = ?, retry_count = 0 WHERE memory_id = ?",
                    params![stale_time, processing.id],
                )?;
                conn.execute(
                    "UPDATE embedding_queue SET queued_at = ?, status = 'pending' WHERE memory_id = ?",
                    params![old_pending_time, pending.id],
                )?;
                conn.execute(
                    "UPDATE embedding_queue SET status = 'failed', retry_count = 1 WHERE memory_id = ?",
                    params![retryable_failed.id],
                )?;
                conn.execute(
                    "UPDATE embedding_queue SET status = 'failed', retry_count = 4 WHERE memory_id = ?",
                    params![exhausted_failed.id],
                )?;
                conn.execute(
                    "UPDATE embedding_queue SET status = 'failed', retry_count = 0 WHERE memory_id = ?",
                    params![zero_retry_failed.id],
                )?;
                Ok(())
            })
            .unwrap();

        let status = maintenance_status(&storage).expect("status should be collected");
        let mut output = Vec::new();

        write_maintenance_status(&mut output, &status).expect("status should render");
        let text = String::from_utf8(output).expect("output should be utf8");

        assert!(text.contains(
            "queue-state: pending=1 processing=1 stale_processing=1 failed=3 zero_retry_failed=1 retryable_failed=2 exhausted_failed=1 max_retry_count=4 oldest_pending_age="
        ));
        assert!(text.contains("retry_count_0=1"));
        assert!(text.contains("retry_count_1=1"));
        assert!(text.contains("retry_count_2=0"));
        assert!(text.contains("retry_count_3+=1"));
    }

    #[test]
    fn maintenance_queue_hygiene_dry_run_does_not_mutate_and_apply_updates() {
        let (_dir, storage) = test_storage();
        let (stale_retryable, stale_exhausted, stale_fresh, failed_retryable, complete_recent, complete_old) =
            storage.with_connection(|conn| {
                let stale_retryable = create_memory(conn, &memory_input("queue-hygiene stale retryable"))?;
                let stale_exhausted = create_memory(conn, &memory_input("queue-hygiene stale exhausted"))?;
                let stale_fresh = create_memory(conn, &memory_input("queue-hygiene stale fresh"))?;
                let failed_retryable = create_memory(conn, &memory_input("queue-hygiene failed retryable"))?;
                let complete_recent = create_memory(conn, &memory_input("queue-hygiene complete recent"))?;
                let complete_old = create_memory(conn, &memory_input("queue-hygiene complete old"))?;

                let old_started_at = (chrono::Utc::now() - chrono::Duration::minutes(30)).to_rfc3339();
                let fresh_started_at = chrono::Utc::now().to_rfc3339();
                let old_completed = (chrono::Utc::now() - chrono::Duration::days(30)).to_rfc3339();
                let new_completed = (chrono::Utc::now() - chrono::Duration::minutes(10)).to_rfc3339();

                conn.execute(
                    "UPDATE embedding_queue SET status = 'processing', started_at = ?, retry_count = 1 WHERE memory_id = ?",
                    params![old_started_at, stale_retryable.id],
                )?;
                conn.execute(
                    "UPDATE embedding_queue SET status = 'processing', started_at = ?, retry_count = 3 WHERE memory_id = ?",
                    params![old_started_at, stale_exhausted.id],
                )?;
                conn.execute(
                    "UPDATE embedding_queue SET status = 'processing', started_at = ?, retry_count = 0 WHERE memory_id = ?",
                    params![fresh_started_at, stale_fresh.id],
                )?;
                conn.execute(
                    "UPDATE embedding_queue SET status = 'failed', retry_count = 1 WHERE memory_id = ?",
                    params![failed_retryable.id],
                )?;
                conn.execute(
                    "UPDATE embedding_queue SET status = 'complete', queued_at = ?, completed_at = ? WHERE memory_id = ?",
                    params![old_completed, old_completed, complete_old.id],
                )?;
                conn.execute(
                    "UPDATE embedding_queue SET status = 'complete', queued_at = ?, completed_at = ? WHERE memory_id = ?",
                    params![new_completed, new_completed, complete_recent.id],
                )?;
                Ok((
                    stale_retryable.id,
                    stale_exhausted.id,
                    stale_fresh.id,
                    failed_retryable.id,
                    complete_recent.id,
                    complete_old.id,
                ))
            })
            .unwrap();

        let dry_run = run_embedding_queue_maintenance(&storage, true, false).unwrap();
        assert_eq!(dry_run.requeued_stale, 1);
        assert_eq!(dry_run.failed_exhausted, 1);
        assert_eq!(dry_run.requeued_failed, 1);
        assert_eq!(dry_run.pruned_complete, 1);

        let before = storage
            .with_connection(|conn| {
                let stale_retryable_state = conn.query_row(
                    "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                    params![stale_retryable],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
                )?;
                let stale_exhausted_state = conn.query_row(
                    "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                    params![stale_exhausted],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
                )?;
                let stale_fresh_state = conn.query_row(
                    "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                    params![stale_fresh],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
                )?;
                let failed_retryable_state = conn.query_row(
                    "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                    params![failed_retryable],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
                )?;
                let old_complete = conn.query_row(
                "SELECT COUNT(*) FROM embedding_queue WHERE status = 'complete' AND memory_id = ?",
                params![complete_old],
                |row| row.get::<_, i64>(0),
            )?;
                Ok((
                    stale_retryable_state,
                    stale_exhausted_state,
                    stale_fresh_state,
                    failed_retryable_state,
                    old_complete,
                ))
            })
            .unwrap();
        assert_eq!(before.0, ("processing".to_string(), 1));
        assert_eq!(before.1, ("processing".to_string(), 3));
        assert_eq!(before.2, ("processing".to_string(), 0));
        assert_eq!(before.3, ("failed".to_string(), 1));
        assert_eq!(before.4, 1);

        let applied = run_embedding_queue_maintenance(&storage, true, true).unwrap();
        assert_eq!(applied.requeued_stale, 1);
        assert_eq!(applied.failed_exhausted, 1);
        assert_eq!(applied.requeued_failed, 1);
        assert_eq!(applied.pruned_complete, 1);

        let after = storage.with_connection(|conn| {
            let stale_retryable_state = conn
                .query_row(
                    "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                    params![stale_retryable],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
                )?;
            let stale_exhausted_state = conn
                .query_row(
                    "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                    params![stale_exhausted],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
                )?;
            let stale_fresh_state = conn
                .query_row(
                    "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                    params![stale_fresh],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
                )?;
            let failed_retryable_state = conn
                .query_row(
                    "SELECT status, retry_count FROM embedding_queue WHERE memory_id = ?",
                    params![failed_retryable],
                    |row| Ok((row.get::<_, String>(0)?, row.get::<_, i32>(1)?)),
                )?;
            let complete_count = conn.query_row(
                "SELECT COUNT(*) FROM embedding_queue WHERE status = 'complete' AND memory_id IN (?, ?)",
                params![complete_recent, complete_old],
                |row| row.get::<_, i64>(0),
            )?;
            Ok((
                stale_retryable_state,
                stale_exhausted_state,
                stale_fresh_state,
                failed_retryable_state,
                complete_count,
            ))
        })
        .unwrap();

        assert_eq!(after.0, ("pending".to_string(), 2));
        assert_eq!(after.1, ("failed".to_string(), 3));
        assert_eq!(after.2, ("processing".to_string(), 0));
        assert_eq!(after.3, ("pending".to_string(), 2));
        assert_eq!(after.4, 1);
    }
}
