//! Operational Context write API.

mod artifacts;
mod events;
mod helpers;
mod types;

pub use artifacts::record_context_artifact;
pub use events::record_context;
pub use types::*;
