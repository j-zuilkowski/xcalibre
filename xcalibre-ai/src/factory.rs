//! Provider factory: construct a concrete [`AiProvider`] from configuration.
//!
//! This is the single place where the provider string from the database is mapped
//! to a backend implementation. Adding a new provider requires:
//!
//! 1. Creating `src/newprovider.rs` implementing [`AiProvider`].
//! 2. Adding it to `xcalibre-ai/src/lib.rs` as a `pub mod`.
//! 3. Adding a match arm here.
//! 4. Adding the provider to the `PROVIDERS` array in
//!    `ui/src/components/AIProviderSettingsPanel.tsx`.
//!
//! ## Default URLs
//!
//! | Provider | Default base URL |
//! |----------|-----------------|
//! | ollama | `http://localhost:11434` |
//! | openai | `https://api.openai.com/v1` |
//! | gemini | `https://generativelanguage.googleapis.com/v1beta` |
//! | lmstudio | `http://localhost:1234` |
//! | openrouter | `https://openrouter.ai/api/v1` |

use crate::{AiProvider, AiError};
use crate::ollama::OllamaBackend;
use crate::openai::OpenAiBackend;
use crate::gemini::GeminiBackend;
use crate::lmstudio::LmStudioBackend;
use crate::openrouter::OpenRouterBackend;

/// All fields needed to construct any AI provider backend.
///
/// Populated from [`xcalibre_processing::db::ai_queries::AiConfig`] by the Tauri
/// command layer before calling [`make_provider`].
pub struct ProviderConfig {
    /// One of `"ollama"`, `"openai"`, `"gemini"`, `"lmstudio"`, `"openrouter"`.
    pub provider: String,

    /// Model identifier sent in API requests (e.g. `"llama3.2"`, `"gpt-4o"`).
    pub model: String,

    /// Embedding model identifier. May be empty for providers without embedding
    /// support.
    pub embed_model: String,

    /// Base URL of the provider's API, overridable by the user in Settings.
    pub base_url: String,

    /// Bearer token or API key. `None` for local providers.
    pub api_key: Option<String>,
}

/// Construct a boxed [`AiProvider`] from a [`ProviderConfig`].
///
/// Returns `Err(AiError::Provider)` if `cfg.provider` is not one of the known
/// strings. The error message includes the unrecognised value to aid debugging.
///
/// Backends are cheap to construct — they hold config fields and a reusable HTTP
/// client, but do not open connections at construction time.
pub fn make_provider(cfg: &ProviderConfig) -> Result<Box<dyn AiProvider>, AiError> {
    // Empty string is treated the same as no key for providers that need one.
    let key = cfg.api_key.as_deref().unwrap_or("");
    match cfg.provider.as_str() {
        "ollama"     => Ok(Box::new(OllamaBackend::new(&cfg.base_url, &cfg.model, &cfg.embed_model))),
        "openai"     => Ok(Box::new(OpenAiBackend::new(&cfg.base_url, key, &cfg.model, &cfg.embed_model))),
        "gemini"     => Ok(Box::new(GeminiBackend::new(&cfg.base_url, key, &cfg.model, &cfg.embed_model))),
        "lmstudio"   => Ok(Box::new(LmStudioBackend::new(&cfg.base_url, &cfg.model, &cfg.embed_model))),
        "openrouter" => Ok(Box::new(OpenRouterBackend::new(&cfg.base_url, key, &cfg.model, &cfg.embed_model))),
        other        => Err(AiError::Provider(format!("unknown provider: {other}"))),
    }
}
