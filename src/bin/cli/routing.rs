//! CLI handler for Model Routing commands (`engram routing`).

use clap::Subcommand;
use engram::error::Result;
use engram::routing::{ModelPurpose, ModelRouter};
use std::str::FromStr;

#[derive(Subcommand, Debug)]
pub(crate) enum RoutingAction {
    /// List all registered model routes across AI capabilities
    List {
        /// Optional purpose filter (embedding_text, rerank, vision_describe_image, audio_transcribe, llm_council, token_count, deterministic_eval)
        #[arg(short = 'p', long)]
        purpose: Option<String>,
        /// Output format: text, json
        #[arg(short = 'f', long, default_value = "text")]
        format: String,
    },
    /// Resolve the active model route for a given purpose
    Resolve {
        /// Model purpose to resolve (e.g. embedding_text, rerank, vision_describe_image, audio_transcribe, llm_council, token_count, deterministic_eval)
        #[arg(short = 'p', long)]
        purpose: String,
        /// Optional preferred provider (e.g. openai, voyage, cohere, tfidf, clip)
        #[arg(short = 'P', long)]
        provider: Option<String>,
        /// Output format: text, json
        #[arg(short = 'f', long, default_value = "text")]
        format: String,
    },
}

pub(crate) fn handle(action: RoutingAction) -> Result<()> {
    let router = ModelRouter::new();

    match action {
        RoutingAction::List { purpose, format } => {
            let filter = purpose
                .as_deref()
                .and_then(|s| ModelPurpose::from_str(s).ok());
            let mut routes = router.list_routes();
            if let Some(target) = filter {
                routes.retain(|r| r.purpose == target);
            }

            if format.eq_ignore_ascii_case("json") {
                let json_val = serde_json::json!({
                    "routes_count": routes.len(),
                    "routes": routes,
                });
                println!("{}", serde_json::to_string_pretty(&json_val)?);
            } else {
                println!("🌐 [ENGRAM MODEL ROUTING TABLE (RFC 0011)]");
                println!("{}", "═".repeat(95));
                println!(
                    "{:<22} {:<16} {:<24} {:<16} {:<12}",
                    "PURPOSE", "PROVIDER", "MODEL ID", "OFFLINE POLICY", "COST"
                );
                println!("{}", "─".repeat(95));
                for r in &routes {
                    println!(
                        "{:<22} {:<16} {:<24} {:<16} {:<12}",
                        r.purpose.as_str(),
                        r.provider_id,
                        r.model_id,
                        format!("{:?}", r.offline_policy).to_lowercase(),
                        format!("{:?}", r.cost_class).to_lowercase()
                    );
                }
                println!("{}", "═".repeat(95));
                println!("Total registered routes: {}", routes.len());
            }
            Ok(())
        }
        RoutingAction::Resolve {
            purpose,
            provider,
            format,
        } => {
            let model_purpose = ModelPurpose::from_str(&purpose)?;
            let resolution = router.resolve(model_purpose, provider.as_deref());

            if format.eq_ignore_ascii_case("json") {
                println!("{}", serde_json::to_string_pretty(&resolution)?);
            } else {
                let status_icon = match resolution.status {
                    engram::routing::RouteStatus::Ok => "✅",
                    engram::routing::RouteStatus::FallbackUsed => "⚠️",
                    engram::routing::RouteStatus::MissingSecret => "🔑",
                    engram::routing::RouteStatus::FeatureDisabled => "📦",
                    _ => "❌",
                };

                println!("🧭 [MODEL ROUTE RESOLUTION]");
                println!("{}", "═".repeat(60));
                println!("Purpose:        {}", resolution.purpose.as_str());
                println!(
                    "Status:         {} {}",
                    status_icon,
                    resolution.status.as_str()
                );
                println!("Provider:       {}", resolution.provider_id);
                println!("Model ID:       {}", resolution.model_id);
                println!("Offline Policy: {:?}", resolution.offline_policy);
                println!("Fallback Used:  {}", resolution.fallback_used);

                if let Some(ref fb) = resolution.fallback_available {
                    println!("Fallback Avail: {}", fb);
                }
                if let Some(ref sec) = resolution.required_secret {
                    println!("Req. Secret:    {}", sec);
                }
                if let Some(ref reason) = resolution.reason {
                    println!("Diagnostics:    {}", reason);
                }
                if !resolution.warnings.is_empty() {
                    println!("Warnings:       {}", resolution.warnings.join("; "));
                }
                println!("{}", "═".repeat(60));
            }
            Ok(())
        }
    }
}
