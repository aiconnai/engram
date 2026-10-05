//! Model routing and capability introspection layer (RFC 0011).
//!
//! Provides deterministic metadata, route cataloging, and provider resolution
//! across all AI capabilities in Engram without runtime network side-effects.

pub mod model;
pub mod resolver;
pub mod types;

pub use model::{inspect_model_routing, ModelRoutingReport, ProviderCapability, ProviderStatus};
pub use resolver::ModelRouter;
pub use types::{
    CostClass, FallbackPolicy, LatencyClass, ModelCapability, ModelPurpose, ModelRoute,
    OfflinePolicy, RouteResolution, RouteStatus,
};
