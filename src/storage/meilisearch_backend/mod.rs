mod backend;
pub mod document;
pub mod filters;
pub mod health;
#[cfg(test)]
mod tests;

pub use backend::MeilisearchBackend;
pub(super) const MEMORIES_INDEX: &str = "memories";
