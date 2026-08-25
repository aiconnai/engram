//! Deterministic route resolver and catalog for RFC 0011 Model Routing Contract.

use std::collections::HashMap;

use super::types::{
    CostClass, FallbackPolicy, LatencyClass, ModelCapability, ModelPurpose, ModelRoute,
    OfflinePolicy, RouteResolution, RouteStatus,
};

/// Deterministic model routing engine.
#[derive(Debug, Clone)]
pub struct ModelRouter {
    routes: HashMap<(ModelPurpose, String), ModelRoute>,
    default_providers: HashMap<ModelPurpose, String>,
}

impl Default for ModelRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelRouter {
    /// Create a new `ModelRouter` initialized with the standard system route table.
    pub fn new() -> Self {
        let mut router = Self {
            routes: HashMap::new(),
            default_providers: HashMap::new(),
        };
        router.register_defaults();
        router
    }

    /// Register default system routes matching RFC 0011.
    fn register_defaults(&mut self) {
        // --- 1. Text Embedding Routes ---
        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::EmbeddingText,
                provider_id: "tfidf".to_string(),
                model_id: "tfidf-128".to_string(),
                capability: ModelCapability::LocalRuleBased,
                cost_class: CostClass::FreeLocal,
                latency_class: LatencyClass::Inline,
                offline_policy: OfflinePolicy::WorksOffline,
                fallback_policy: FallbackPolicy::None,
                feature_flag: None,
                requires_secret: false,
                required_secret_name: None,
                fallback_route_provider: None,
            },
            true,
        );

        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::EmbeddingText,
                provider_id: "openai".to_string(),
                model_id: "text-embedding-3-small".to_string(),
                capability: ModelCapability::RemoteEmbedding,
                cost_class: CostClass::MeteredRemote,
                latency_class: LatencyClass::Inline,
                offline_policy: OfflinePolicy::RequiresNetwork,
                fallback_policy: FallbackPolicy::ExplicitLocalFallback,
                feature_flag: Some("openai".to_string()),
                requires_secret: true,
                required_secret_name: Some("OPENAI_API_KEY".to_string()),
                fallback_route_provider: Some("tfidf".to_string()),
            },
            false,
        );

        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::EmbeddingText,
                provider_id: "voyage".to_string(),
                model_id: "voyage-3-lite".to_string(),
                capability: ModelCapability::RemoteEmbedding,
                cost_class: CostClass::MeteredRemote,
                latency_class: LatencyClass::Inline,
                offline_policy: OfflinePolicy::RequiresNetwork,
                fallback_policy: FallbackPolicy::ExplicitLocalFallback,
                feature_flag: Some("voyage".to_string()),
                requires_secret: true,
                required_secret_name: Some("VOYAGE_API_KEY".to_string()),
                fallback_route_provider: Some("tfidf".to_string()),
            },
            false,
        );

        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::EmbeddingText,
                provider_id: "cohere".to_string(),
                model_id: "embed-english-v3.0".to_string(),
                capability: ModelCapability::RemoteEmbedding,
                cost_class: CostClass::MeteredRemote,
                latency_class: LatencyClass::Inline,
                offline_policy: OfflinePolicy::RequiresNetwork,
                fallback_policy: FallbackPolicy::ExplicitLocalFallback,
                feature_flag: Some("cohere".to_string()),
                requires_secret: true,
                required_secret_name: Some("COHERE_API_KEY".to_string()),
                fallback_route_provider: Some("tfidf".to_string()),
            },
            false,
        );

        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::EmbeddingText,
                provider_id: "onnx".to_string(),
                model_id: "all-MiniLM-L6-v2".to_string(),
                capability: ModelCapability::LocalModel,
                cost_class: CostClass::FreeLocal,
                latency_class: LatencyClass::Inline,
                offline_policy: OfflinePolicy::RequiresLocalModel,
                fallback_policy: FallbackPolicy::ExplicitLocalFallback,
                feature_flag: Some("onnx-embed".to_string()),
                requires_secret: false,
                required_secret_name: None,
                fallback_route_provider: Some("tfidf".to_string()),
            },
            false,
        );

        // --- 2. Image Embedding Routes ---
        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::EmbeddingImage,
                provider_id: "clip".to_string(),
                model_id: "clip-vit-base-patch32".to_string(),
                capability: ModelCapability::LocalModel,
                cost_class: CostClass::FreeLocal,
                latency_class: LatencyClass::Inline,
                offline_policy: OfflinePolicy::RequiresLocalModel,
                fallback_policy: FallbackPolicy::DegradeWithoutSubstitution,
                feature_flag: Some("multimodal".to_string()),
                requires_secret: false,
                required_secret_name: None,
                fallback_route_provider: None,
            },
            true,
        );

        // --- 3. Rerank Routes ---
        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::Rerank,
                provider_id: "rule_based".to_string(),
                model_id: "bm25-linear".to_string(),
                capability: ModelCapability::LocalRuleBased,
                cost_class: CostClass::FreeLocal,
                latency_class: LatencyClass::Inline,
                offline_policy: OfflinePolicy::WorksOffline,
                fallback_policy: FallbackPolicy::None,
                feature_flag: None,
                requires_secret: false,
                required_secret_name: None,
                fallback_route_provider: None,
            },
            true,
        );

        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::Rerank,
                provider_id: "cohere".to_string(),
                model_id: "rerank-english-v3.0".to_string(),
                capability: ModelCapability::RemoteLlm,
                cost_class: CostClass::MeteredRemote,
                latency_class: LatencyClass::Inline,
                offline_policy: OfflinePolicy::RequiresNetwork,
                fallback_policy: FallbackPolicy::ExplicitLocalFallback,
                feature_flag: Some("cohere".to_string()),
                requires_secret: true,
                required_secret_name: Some("COHERE_API_KEY".to_string()),
                fallback_route_provider: Some("rule_based".to_string()),
            },
            false,
        );

        // --- 4. Vision Describe Image Routes ---
        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::VisionDescribeImage,
                provider_id: "gemini_vision".to_string(),
                model_id: "gemini-1.5-flash".to_string(),
                capability: ModelCapability::RemoteVision,
                cost_class: CostClass::MeteredRemote,
                latency_class: LatencyClass::Inline,
                offline_policy: OfflinePolicy::RequiresNetwork,
                fallback_policy: FallbackPolicy::None,
                feature_flag: Some("multimodal".to_string()),
                requires_secret: true,
                required_secret_name: Some("GEMINI_API_KEY".to_string()),
                fallback_route_provider: None,
            },
            true,
        );

        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::VisionDescribeImage,
                provider_id: "openai_vision".to_string(),
                model_id: "gpt-4o-mini".to_string(),
                capability: ModelCapability::RemoteVision,
                cost_class: CostClass::MeteredRemote,
                latency_class: LatencyClass::Inline,
                offline_policy: OfflinePolicy::RequiresNetwork,
                fallback_policy: FallbackPolicy::None,
                feature_flag: Some("openai".to_string()),
                requires_secret: true,
                required_secret_name: Some("OPENAI_API_KEY".to_string()),
                fallback_route_provider: None,
            },
            false,
        );

        // --- 5. Audio Transcribe Routes ---
        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::AudioTranscribe,
                provider_id: "whisper_local".to_string(),
                model_id: "whisper-tiny".to_string(),
                capability: ModelCapability::LocalModel,
                cost_class: CostClass::FreeLocal,
                latency_class: LatencyClass::Background,
                offline_policy: OfflinePolicy::RequiresLocalModel,
                fallback_policy: FallbackPolicy::None,
                feature_flag: Some("audio".to_string()),
                requires_secret: false,
                required_secret_name: None,
                fallback_route_provider: None,
            },
            true,
        );

        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::AudioTranscribe,
                provider_id: "openai_whisper".to_string(),
                model_id: "whisper-1".to_string(),
                capability: ModelCapability::RemoteAudio,
                cost_class: CostClass::MeteredRemote,
                latency_class: LatencyClass::Background,
                offline_policy: OfflinePolicy::RequiresNetwork,
                fallback_policy: FallbackPolicy::None,
                feature_flag: Some("openai".to_string()),
                requires_secret: true,
                required_secret_name: Some("OPENAI_API_KEY".to_string()),
                fallback_route_provider: None,
            },
            false,
        );

        // --- 6. LLM Council Routes ---
        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::LlmCouncil,
                provider_id: "llm_council_remote".to_string(),
                model_id: "council-v1".to_string(),
                capability: ModelCapability::RemoteLlm,
                cost_class: CostClass::MeteredRemote,
                latency_class: LatencyClass::Background,
                offline_policy: OfflinePolicy::RequiresNetwork,
                fallback_policy: FallbackPolicy::None,
                feature_flag: Some("http-client".to_string()),
                requires_secret: true,
                required_secret_name: Some("COUNCIL_API_KEY".to_string()),
                fallback_route_provider: None,
            },
            true,
        );

        // --- 7. Token Count Routes (Deterministic Rule-Based) ---
        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::TokenCount,
                provider_id: "tiktoken_local".to_string(),
                model_id: "cl100k_base".to_string(),
                capability: ModelCapability::LocalRuleBased,
                cost_class: CostClass::FreeLocal,
                latency_class: LatencyClass::Inline,
                offline_policy: OfflinePolicy::WorksOffline,
                fallback_policy: FallbackPolicy::None,
                feature_flag: None,
                requires_secret: false,
                required_secret_name: None,
                fallback_route_provider: None,
            },
            true,
        );

        // --- 8. Deterministic Evals Routes ---
        self.add_route(
            ModelRoute {
                purpose: ModelPurpose::DeterministicEval,
                provider_id: "eval_engine".to_string(),
                model_id: "deterministic-metrics-v1".to_string(),
                capability: ModelCapability::LocalRuleBased,
                cost_class: CostClass::FreeLocal,
                latency_class: LatencyClass::Inline,
                offline_policy: OfflinePolicy::WorksOffline,
                fallback_policy: FallbackPolicy::None,
                feature_flag: None,
                requires_secret: false,
                required_secret_name: None,
                fallback_route_provider: None,
            },
            true,
        );
    }

    /// Add a route to the catalog.
    pub fn add_route(&mut self, route: ModelRoute, is_default: bool) {
        let purpose = route.purpose;
        let provider_id = route.provider_id.clone();
        if is_default {
            self.default_providers.insert(purpose, provider_id.clone());
        }
        self.routes.insert((purpose, provider_id), route);
    }

    /// List all registered routes.
    pub fn list_routes(&self) -> Vec<ModelRoute> {
        let mut list: Vec<_> = self.routes.values().cloned().collect();
        list.sort_by(|a, b| {
            a.purpose
                .as_str()
                .cmp(b.purpose.as_str())
                .then_with(|| a.provider_id.cmp(&b.provider_id))
        });
        list
    }

    /// Deterministically resolve a model route for a given purpose and optional preferred provider.
    ///
    /// This method evaluates local environment variables and static configuration without
    /// ever making outbound network requests.
    pub fn resolve(
        &self,
        purpose: ModelPurpose,
        preferred_provider: Option<&str>,
    ) -> RouteResolution {
        let target_provider = preferred_provider
            .map(|p| p.to_lowercase())
            .or_else(|| self.get_env_provider_override(purpose))
            .or_else(|| self.default_providers.get(&purpose).cloned())
            .unwrap_or_else(|| "default".to_string());

        let target_key = (purpose, target_provider.clone());
        let route = match self.routes.get(&target_key) {
            Some(r) => r,
            None => {
                return RouteResolution {
                    purpose,
                    status: RouteStatus::Unavailable,
                    provider_id: target_provider,
                    model_id: "unknown".to_string(),
                    offline_policy: OfflinePolicy::DisabledWithoutFeature,
                    fallback_used: false,
                    fallback_available: None,
                    reason: Some(
                        "No matching model route registered for requested purpose and provider"
                            .to_string(),
                    ),
                    required_secret: None,
                    warnings: Vec::new(),
                };
            }
        };

        // 1. Check feature flag requirements
        if let Some(ref feat) = route.feature_flag {
            if !self.is_feature_compiled(feat) {
                return RouteResolution {
                    purpose,
                    status: RouteStatus::FeatureDisabled,
                    provider_id: route.provider_id.clone(),
                    model_id: route.model_id.clone(),
                    offline_policy: route.offline_policy,
                    fallback_used: false,
                    fallback_available: route.fallback_route_provider.clone(),
                    reason: Some(format!(
                        "Cargo feature '{}' is not enabled in this build",
                        feat
                    )),
                    required_secret: None,
                    warnings: Vec::new(),
                };
            }
        }

        // 2. Check secret / API key requirements
        if route.requires_secret {
            if let Some(ref secret_name) = route.required_secret_name {
                let has_secret = std::env::var(secret_name)
                    .map(|val| !val.trim().is_empty())
                    .unwrap_or(false);

                if !has_secret {
                    return RouteResolution {
                        purpose,
                        status: RouteStatus::MissingSecret,
                        provider_id: route.provider_id.clone(),
                        model_id: route.model_id.clone(),
                        offline_policy: route.offline_policy,
                        fallback_used: false,
                        fallback_available: route.fallback_route_provider.clone(),
                        reason: Some(format!(
                            "Required environment variable '{}' is not set",
                            secret_name
                        )),
                        required_secret: Some(secret_name.clone()),
                        warnings: Vec::new(),
                    };
                }
            }
        }

        // 3. Fully available route
        RouteResolution {
            purpose,
            status: RouteStatus::Ok,
            provider_id: route.provider_id.clone(),
            model_id: route.model_id.clone(),
            offline_policy: route.offline_policy,
            fallback_used: false,
            fallback_available: route.fallback_route_provider.clone(),
            reason: None,
            required_secret: None,
            warnings: Vec::new(),
        }
    }

    /// Check compiled features dynamically.
    fn is_feature_compiled(&self, feature: &str) -> bool {
        match feature {
            "openai" if cfg!(feature = "openai") => true,
            "voyage" if cfg!(feature = "voyage") => true,
            "cohere" if cfg!(feature = "cohere") => true,
            "onnx-embed" if cfg!(feature = "onnx-embed") => true,
            "multimodal" if cfg!(feature = "multimodal") => true,
            "http-client" if cfg!(feature = "http-client") => true,
            "meilisearch" if cfg!(feature = "meilisearch") => true,
            "langfuse" if cfg!(feature = "langfuse") => true,
            _ => false,
        }
    }

    /// Inspect environment variables for provider preference overrides.
    fn get_env_provider_override(&self, purpose: ModelPurpose) -> Option<String> {
        match purpose {
            ModelPurpose::EmbeddingText => std::env::var("ENGRAM_EMBEDDING_PROVIDER")
                .ok()
                .filter(|s| !s.trim().is_empty()),
            ModelPurpose::Rerank => std::env::var("ENGRAM_RERANK_PROVIDER")
                .ok()
                .filter(|s| !s.trim().is_empty()),
            ModelPurpose::VisionDescribeImage => std::env::var("ENGRAM_VISION_PROVIDER")
                .ok()
                .filter(|s| !s.trim().is_empty()),
            ModelPurpose::AudioTranscribe => std::env::var("ENGRAM_AUDIO_PROVIDER")
                .ok()
                .filter(|s| !s.trim().is_empty()),
            _ => None,
        }
    }
}
