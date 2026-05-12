pub mod provider;
pub mod ollama;
pub mod chunk;
pub mod error;
pub mod openai;
pub mod gemini;
pub mod lmstudio;
pub mod openrouter;
pub mod factory;

pub use error::AiError;
pub use provider::{AiProvider, ChatMessage, ChatResponse, MessageRole};
pub use chunk::{chunk_text, ChunkConfig};
pub use factory::{make_provider, ProviderConfig};
