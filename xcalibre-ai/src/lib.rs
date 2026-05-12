pub mod provider;
pub mod ollama;
pub mod chunk;
pub mod error;

pub use error::AiError;
pub use provider::{AiProvider, ChatMessage, ChatResponse, MessageRole};
pub use chunk::{chunk_text, ChunkConfig};
