use crate::{AiProvider, AiError};
use crate::ollama::OllamaBackend;
use crate::openai::OpenAiBackend;
use crate::gemini::GeminiBackend;
use crate::lmstudio::LmStudioBackend;
use crate::openrouter::OpenRouterBackend;

pub struct ProviderConfig {
    pub provider:    String,
    pub model:       String,
    pub embed_model: String,
    pub base_url:    String,
    pub api_key:     Option<String>,
}

pub fn make_provider(cfg: &ProviderConfig) -> Result<Box<dyn AiProvider>, AiError> {
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
