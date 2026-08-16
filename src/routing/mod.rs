//! Model routing and capability introspection layer (RFC 0011).

pub mod model;

pub use model::{inspect_model_routing, ModelCapability, ModelRoutingReport, ProviderStatus};
