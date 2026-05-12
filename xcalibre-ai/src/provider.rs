//! Core AI provider abstraction.
//!
//! All AI backends (Ollama, OpenAI, Gemini, LM Studio, OpenRouter) implement
//! the [`AiProvider`] trait. This allows the rest of the codebase to be written
//! against a single interface and switch providers by changing configuration.
//!
//! ## Provider instantiation
//!
//! Do not construct backends directly. Use [`crate::factory::make_provider`] which
//! reads a [`crate::factory::ProviderConfig`] and returns a `Box<dyn AiProvider>`.
//!
//! ## Streaming
//!
//! The `chat` method accepts an optional `stream_tx: Option<Sender<String>>`. When
//! provided, each token delta is sent through the channel as it arrives from the
//! provider's streaming API. The final `ChatResponse` is still returned at the end
//! with the complete assembled content. When `None`, the provider waits for the
//! full response before returning (non-streaming mode).

use crate::AiError;
use serde::{Deserialize, Serialize};

/// The role of a message in a chat conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    /// Instructions given to the model before the conversation begins.
    System,
    /// A message from the human user.
    User,
    /// A message from the AI assistant.
    Assistant,
}

/// A single turn in a conversation, with a role and text content.
///
/// Used both for sending history to the model and for constructing the system
/// prompt with book context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role:    MessageRole,
    pub content: String,
}

/// The result of a completed chat turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// The full text of the assistant's reply.
    pub content: String,

    /// The model name as reported by the provider (may differ from the requested
    /// model name if the provider resolves aliases).
    pub model: String,

    /// `true` when the model has finished generating (always `true` in non-streaming
    /// mode; set to `true` on the final streamed chunk).
    pub done: bool,

    /// The reasoning trace produced by extended-thinking models (e.g. DeepSeek R1,
    /// Claude 3.7 Sonnet). `None` if the model does not support reasoning or if
    /// the reasoning budget is set to `None`.
    pub reasoning: Option<String>,
}

/// Trait implemented by every AI provider backend.
///
/// All methods are async and the trait requires `Send + Sync` so provider objects
/// can be stored in Tauri's `State<Arc<...>>` and called from async command handlers.
///
/// ## Implementation notes
///
/// Each backend module (`ollama.rs`, `openai.rs`, etc.) has its own HTTP client.
/// Backends that use remote APIs (OpenAI, Gemini, OpenRouter) create an ephemeral
/// `reqwest::Client` per invocation to avoid shared connection pool state that
/// can cause stale-connection errors under governors and rate limiters.
#[async_trait::async_trait]
pub trait AiProvider: Send + Sync {
    /// Send a chat request with conversation history and return the assistant reply.
    ///
    /// `messages` must include at least one `User` message. A `System` message
    /// with book context is typically prepended by the caller.
    ///
    /// If `stream_tx` is `Some`, the backend streams token deltas through the
    /// channel and still returns the full response at the end.
    async fn chat(
        &self,
        messages: &[ChatMessage],
        stream_tx: Option<tokio::sync::mpsc::Sender<String>>,
    ) -> Result<ChatResponse, AiError>;

    /// Generate a text embedding vector for `text`.
    ///
    /// Returns a `Vec<f32>` of length determined by the model (typically 384,
    /// 768, or 1536 dimensions). Used by the RAG chunking pipeline to embed book
    /// text for semantic search.
    ///
    /// Returns `AiError::Unsupported` for providers that do not offer an embedding
    /// endpoint (e.g. OpenRouter).
    async fn embed(&self, text: &str) -> Result<Vec<f32>, AiError>;

    /// Return the list of models available on this provider.
    ///
    /// For local providers (Ollama, LM Studio), this queries the running server.
    /// For remote providers (OpenAI, etc.), this queries the `/models` endpoint.
    /// Results are displayed in the datalist dropdown in [`AIProviderSettingsPanel`].
    async fn list_models(&self) -> Result<Vec<String>, AiError>;

    /// The provider's identifier string, matching the values in `PROVIDERS` in
    /// `ui/src/components/AIProviderSettingsPanel.tsx` and stored in `ai_config.provider`.
    fn name(&self) -> &str;
}
