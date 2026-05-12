// stub — implemented in rmp08b
use crate::{AiError, AiProvider, ChatMessage, ChatResponse};

pub struct OllamaProvider {
    base_url: String,
    model: String,
}

impl OllamaProvider {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self { base_url: base_url.to_string(), model: model.to_string() }
    }
}

#[async_trait::async_trait]
impl AiProvider for OllamaProvider {
    async fn chat(&self, _messages: &[ChatMessage], _stream_tx: Option<tokio::sync::mpsc::Sender<String>>) -> Result<ChatResponse, AiError> {
        unimplemented!("rmp08b")
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>, AiError> {
        unimplemented!("rmp08b")
    }
    async fn list_models(&self) -> Result<Vec<String>, AiError> {
        unimplemented!("rmp08b")
    }
    fn name(&self) -> &str { "ollama" }
}
