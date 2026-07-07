use std::io::{self, Write};

use engram::embedding::create_embedder;
use engram::error::Result;
use engram::search::{hybrid_search, SearchConfig};
use engram::storage::queries::{create_memory, get_memory, get_stats, list_memories};
use engram::storage::Storage;
use engram::types::{CreateMemoryInput, EmbeddingConfig, ListOptions, MemoryType, SearchOptions};

use crate::util::truncate;

pub(crate) fn run(storage: &Storage) -> Result<()> {
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
            "help" => print_help(),
            "stats" => print_stats(storage)?,
            "list" => print_recent_memories(storage)?,
            _ if line.starts_with("get ") => get_by_id(storage, line)?,
            _ if line.starts_with("create ") => create_note(storage, line)?,
            _ if line.starts_with("search ") => search_memories(storage, line)?,
            _ => println!("Unknown command. Type 'help' for available commands."),
        }
    }

    println!("Goodbye!");
    Ok(())
}

fn print_help() {
    println!("Commands:");
    println!("  create <content>  - Create a memory");
    println!("  get <id>          - Get memory by ID");
    println!("  list              - List recent memories");
    println!("  search <query>    - Search memories");
    println!("  stats             - Show statistics");
    println!("  quit              - Exit");
}

fn print_stats(storage: &Storage) -> Result<()> {
    let stats = storage.with_connection(get_stats)?;
    println!("Memories: {}", stats.total_memories);
    println!("Tags: {}", stats.total_tags);
    println!("Cross-refs: {}", stats.total_crossrefs);
    Ok(())
}

fn print_recent_memories(storage: &Storage) -> Result<()> {
    let options = ListOptions {
        limit: Some(10),
        ..Default::default()
    };
    let memories = storage.with_connection(|conn| list_memories(conn, &options))?;
    for memory in memories {
        println!("#{}: {}", memory.id, truncate(&memory.content, 60));
    }
    Ok(())
}

fn get_by_id(storage: &Storage, line: &str) -> Result<()> {
    if let Ok(id) = line[4..].trim().parse::<i64>() {
        match storage.with_connection(|conn| get_memory(conn, id)) {
            Ok(memory) => println!("{}", serde_json::to_string_pretty(&memory)?),
            Err(e) => println!("Error: {}", e),
        }
    } else {
        println!("Invalid ID");
    }
    Ok(())
}

fn create_note(storage: &Storage, line: &str) -> Result<()> {
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
    Ok(())
}

fn search_memories(storage: &Storage, line: &str) -> Result<()> {
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
        hybrid_search(conn, query, query_embedding.as_deref(), &options, &config)
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
    Ok(())
}
