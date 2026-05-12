#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("not connected: {0}")]
    NotConnected(String),
}
