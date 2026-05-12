// LM Studio exposes an OpenAI-compatible REST API on localhost:1234.
// This is a thin wrapper around OpenAiBackend with provider name "lmstudio".
use crate::openai::OpenAiBackend;
use crate::{AiError, AiProvider, ChatMessage, ChatResponse};
use async_trait::async_trait;

pub struct LmStudioBackend(OpenAiBackend);

impl LmStudioBackend {
    pub fn new(base_url: &str, model: &str, embed_model: &str) -> Self {
        Self(OpenAiBackend::new(base_url, "", model, embed_model))
    }
    pub fn model(&self) -> &str { self.0.model() }
}

#[async_trait]
impl AiProvider for LmStudioBackend {
    async fn chat(&self, msgs: &[ChatMessage], tx: Option<tokio::sync::mpsc::Sender<String>>) -> Result<ChatResponse, AiError> {
        self.0.chat(msgs, tx).await
    }
    async fn embed(&self, text: &str) -> Result<Vec<f32>, AiError> { self.0.embed(text).await }
    async fn list_models(&self) -> Result<Vec<String>, AiError> { self.0.list_models().await }
    fn name(&self) -> &str { "lmstudio" }
}
