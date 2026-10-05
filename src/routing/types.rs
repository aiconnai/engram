//! Core data structures and enums for RFC 0011 Model Routing Contract.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::EngramError;

/// Purpose / intent of the model route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelPurpose {
    EmbeddingText,
    EmbeddingImage,
    Rerank,
    VisionDescribeImage,
    AudioTranscribe,
    LlmCouncil,
    TokenCount,
    DeterministicEval,
}

impl ModelPurpose {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EmbeddingText => "embedding_text",
            Self::EmbeddingImage => "embedding_image",
            Self::Rerank => "rerank",
            Self::VisionDescribeImage => "vision_describe_image",
            Self::AudioTranscribe => "audio_transcribe",
            Self::LlmCouncil => "llm_council",
            Self::TokenCount => "token_count",
            Self::DeterministicEval => "deterministic_eval",
        }
    }
}

impl fmt::Display for ModelPurpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for ModelPurpose {
    type Err = EngramError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "embedding_text" | "text_embedding" | "embedding" => Ok(Self::EmbeddingText),
            "embedding_image" | "image_embedding" => Ok(Self::EmbeddingImage),
            "rerank" | "reranking" => Ok(Self::Rerank),
            "vision_describe_image" | "vision" | "image_description" => {
                Ok(Self::VisionDescribeImage)
            }
            "audio_transcribe" | "audio" | "transcribe" | "stt" => Ok(Self::AudioTranscribe),
            "llm_council" | "council" => Ok(Self::LlmCouncil),
            "token_count" | "token_counter" | "tokenize" => Ok(Self::TokenCount),
            "deterministic_eval" | "eval" | "evals" => Ok(Self::DeterministicEval),
            other => Err(EngramError::InvalidInput(format!(
                "Unknown model purpose '{}'. Supported: embedding_text, embedding_image, rerank, vision_describe_image, audio_transcribe, llm_council, token_count, deterministic_eval",
                other
            ))),
        }
    }
}

/// Execution capability category of the underlying provider/model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCapability {
    LocalRuleBased,
    LocalModel,
    RemoteEmbedding,
    RemoteLlm,
    RemoteVision,
    RemoteAudio,
}

impl ModelCapability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LocalRuleBased => "local_rule_based",
            Self::LocalModel => "local_model",
            Self::RemoteEmbedding => "remote_embedding",
            Self::RemoteLlm => "remote_llm",
            Self::RemoteVision => "remote_vision",
            Self::RemoteAudio => "remote_audio",
        }
    }
}

/// Cost tier of invoking this route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CostClass {
    FreeLocal,
    MeteredRemote,
    Unknown,
}

/// Latency profile of the route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LatencyClass {
    Inline,
    Background,
    Batch,
}

/// Offline capability policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfflinePolicy {
    WorksOffline,
    RequiresLocalModel,
    RequiresNetwork,
    DisabledWithoutFeature,
}

/// Fallback substitution policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FallbackPolicy {
    None,
    ExplicitLocalFallback,
    ExplicitRemoteFallback,
    DegradeWithoutSubstitution,
}

/// Resolution status of a requested model route.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteStatus {
    Ok,
    MissingSecret,
    FeatureDisabled,
    ModelMissing,
    ProviderUnavailable,
    FallbackUsed,
    Unavailable,
}

impl RouteStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::MissingSecret => "missing_secret",
            Self::FeatureDisabled => "feature_disabled",
            Self::ModelMissing => "model_missing",
            Self::ProviderUnavailable => "provider_unavailable",
            Self::FallbackUsed => "fallback_used",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Declarative metadata descriptor for a model route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelRoute {
    pub purpose: ModelPurpose,
    pub provider_id: String,
    pub model_id: String,
    pub capability: ModelCapability,
    pub cost_class: CostClass,
    pub latency_class: LatencyClass,
    pub offline_policy: OfflinePolicy,
    pub fallback_policy: FallbackPolicy,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_flag: Option<String>,
    pub requires_secret: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_secret_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_route_provider: Option<String>,
}

/// Structured resolution outcome for a requested model route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteResolution {
    pub purpose: ModelPurpose,
    pub status: RouteStatus,
    pub provider_id: String,
    pub model_id: String,
    pub offline_policy: OfflinePolicy,
    pub fallback_used: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_available: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
}
