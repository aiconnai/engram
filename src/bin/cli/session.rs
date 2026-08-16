use clap::{Args, Subcommand};
use engram::error::Result;
use engram::intelligence::{build_session_handoff, SessionHandoffRequest};
use engram::storage::Storage;

#[derive(Subcommand)]
pub(crate) enum SessionAction {
    /// Generate a copy-ready packet for continuing work in a new AI session
    Handoff(Box<SessionHandoffArgs>),
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

    /// Files touched during the session. Repeat the flag for multiple files.
    #[arg(long = "file")]
    pub(crate) files_touched: Vec<String>,

    /// Decisions made during the session. Repeat the flag for multiple decisions.
    #[arg(long = "decision")]
    pub(crate) decisions_made: Vec<String>,

    /// Tests or verifications run. Repeat the flag for multiple tests.
    #[arg(long = "test-run")]
    pub(crate) tests_run: Vec<String>,

    /// Tests not run. Repeat the flag for multiple items.
    #[arg(long = "test-not-run")]
    pub(crate) tests_not_run: Vec<String>,

    /// Known risks for the next agent. Repeat the flag for multiple risks.
    #[arg(long = "risk")]
    pub(crate) known_risks: Vec<String>,

    /// Active blockers. Repeat the flag for multiple blockers.
    #[arg(long = "blocker")]
    pub(crate) blockers: Vec<String>,

    /// Next step. Repeat the flag for multiple steps.
    #[arg(long = "next")]
    pub(crate) next_steps: Vec<String>,

    /// Verification evidence summary
    #[arg(long = "evidence")]
    pub(crate) verification_evidence: Option<String>,

    /// Do not persist a checkpoint memory
    #[arg(long)]
    pub(crate) no_persist: bool,

    /// Disable attaching operational context
    #[arg(long)]
    pub(crate) no_operational_context: bool,

    /// Disable topic digest memory retrieval
    #[arg(long)]
    pub(crate) no_digest: bool,

    /// Print structured JSON instead of copy-ready Markdown
    #[arg(long)]
    pub(crate) json: bool,
}

pub(crate) fn handle(storage: &Storage, action: SessionAction) -> Result<()> {
    match action {
        SessionAction::Handoff(args) => handoff(storage, *args),
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
            files_touched: args.files_touched,
            decisions_made: args.decisions_made,
            tests_run: args.tests_run,
            tests_not_run: args.tests_not_run,
            known_risks: args.known_risks,
            blockers: args.blockers,
            next_steps: args.next_steps,
            verification_evidence: args.verification_evidence,
            persist: !args.no_persist,
            include_operational_context: !args.no_operational_context,
            include_digest: !args.no_digest,
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
