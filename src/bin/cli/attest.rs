use clap::Subcommand;
use engram::attestation::{AttestationChain, AttestationFilter};
use engram::error::Result;
use engram::storage::Storage;

#[derive(Subcommand)]
pub(crate) enum AttestAction {
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

pub(crate) fn handle(storage: &Storage, action: AttestAction) -> Result<()> {
    match action {
        AttestAction::Log {
            path,
            name,
            agent_id,
        } => log(storage, path, name, agent_id),
        AttestAction::Verify { path } => verify(storage, path),
        AttestAction::ChainVerify => verify_chain(storage),
        AttestAction::List { limit, format } => list(storage, limit, format),
    }
}

fn log(
    storage: &Storage,
    path: String,
    name: Option<String>,
    agent_id: Option<String>,
) -> Result<()> {
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
    Ok(())
}

fn verify(storage: &Storage, path: String) -> Result<()> {
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
    Ok(())
}

fn verify_chain(storage: &Storage) -> Result<()> {
    let chain = AttestationChain::new(storage.clone());
    match chain.verify_chain(None) {
        Ok(status) => println!("{}", serde_json::to_string_pretty(&status)?),
        Err(e) => {
            eprintln!("Error verifying chain: {}", e);
            std::process::exit(1);
        }
    }
    Ok(())
}

fn list(storage: &Storage, limit: usize, format: Option<String>) -> Result<()> {
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
    Ok(())
}
