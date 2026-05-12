// OpenRouter exposes an OpenAI-compatible API at openrouter.ai/api/v1.
use crate::openai::OpenAiBackend;
use crate::{AiError, AiProvider, ChatMessage, ChatResponse};
use async_trait::async_trait;

pub struct OpenRouterBackend(OpenAiBackend);

impl OpenRouterBackend {
    pub fn new(base_url: &str, api_key: &str, model: &str, embed_model: &str) -> Self {
        Self(OpenAiBackend::new(base_url, api_key, model, embed_model))
    }
    pub fn model(&self) -> &str { self.0.model() }
}

#[async_trait]
impl AiProvider for OpenRouterBackend {
    async fn chat(&self, msgs: &[ChatMessage], tx: Option<tokio::sync::mpsc::Sender<String>>) -> Result<ChatResponse, AiError> {
        self.0.chat(msgs, tx).await
    }
    async fn embed(&self, text: &str) -> Result<Vec<f32>, AiError> { self.0.embed(text).await }
    async fn list_models(&self) -> Result<Vec<String>, AiError> { self.0.list_models().await }
    fn name(&self) -> &str { "openrouter" }
}
