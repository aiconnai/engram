use clap::{Args, Subcommand};
use engram::error::Result;
use engram::intelligence::{build_session_handoff, SessionHandoffRequest};
use engram::storage::Storage;

#[derive(Subcommand)]
pub(crate) enum SessionAction {
    /// Generate a copy-ready packet for continuing work in a new AI session
    Handoff(SessionHandoffArgs),
}

#[derive(Args)]
pub(crate) struct SessionHandoffArgs {
    /// Session identifier. When omitted, Engram uses the latest session in the workspace.
    #[arg(long)]
    pub(crate) session: Option<String>,

    /// Workspace scope
    #[arg(long, default_value = "default")]
    pub(crate) workspace: String,

    /// Human-provided summary for the handoff packet
    #[arg(long)]
    pub(crate) summary: Option<String>,

    /// Current goal for the next AI session
    #[arg(long)]
    pub(crate) current_goal: Option<String>,

    /// Next step. Repeat the flag for multiple steps.
    #[arg(long = "next")]
    pub(crate) next_steps: Vec<String>,

    /// Do not persist a checkpoint memory
    #[arg(long)]
    pub(crate) no_persist: bool,

    /// Print structured JSON instead of copy-ready Markdown
    #[arg(long)]
    pub(crate) json: bool,
}

pub(crate) fn handle(storage: &Storage, action: SessionAction) -> Result<()> {
    match action {
        SessionAction::Handoff(args) => handoff(storage, args),
    }
}

fn handoff(storage: &Storage, args: SessionHandoffArgs) -> Result<()> {
    let packet = build_session_handoff(
        storage,
        SessionHandoffRequest {
            session_id: args.session,
            workspace: Some(args.workspace),
            summary: args.summary,
            current_goal: args.current_goal,
            next_steps: args.next_steps,
            persist: !args.no_persist,
            ..Default::default()
        },
    )?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&packet)?);
    } else {
        println!("{}", packet.copy_block);
    }
    Ok(())
}
