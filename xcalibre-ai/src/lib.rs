pub mod provider;
pub mod ollama;
pub mod chunk;
pub mod error;
pub mod openai;
pub mod gemini;
pub mod lmstudio;
pub mod openrouter;
pub mod factory;
pub mod citations;
pub mod reasoning;

pub use error::AiError;
pub use provider::{AiProvider, ChatMessage, ChatResponse, MessageRole};
pub use chunk::{chunk_text, ChunkConfig};
pub use factory::{make_provider, ProviderConfig};
pub use citations::{extract_citations, Citation, CitedResponse};
pub use reasoning::{ReasoningBudget, apply_reasoning_budget};
