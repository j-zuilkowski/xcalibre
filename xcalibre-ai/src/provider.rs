use crate::AiError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageRole { System, User, Assistant }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role:    MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    pub content:   String,
    pub model:     String,
    pub done:      bool,
    pub reasoning: Option<String>,
}

/// Trait implemented by each AI provider backend.
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    async fn chat(
        &self,
        messages: &[ChatMessage],
        stream_tx: Option<tokio::sync::mpsc::Sender<String>>,
    ) -> Result<ChatResponse, AiError>;

    async fn embed(&self, text: &str) -> Result<Vec<f32>, AiError>;

    async fn list_models(&self) -> Result<Vec<String>, AiError>;

    fn name(&self) -> &str;
}
