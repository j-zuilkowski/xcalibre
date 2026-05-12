#[derive(Debug, thiserror::Error)]
pub enum EpubError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("ZIP error: {0}")]
    Zip(String),
    #[error("XML parse error: {0}")]
    Xml(String),
    #[error("item not found in manifest: {0}")]
    ItemNotFound(String),
    #[error("missing required element: {0}")]
    MissingElement(String),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("unsupported EPUB version: {0}")]
    UnsupportedVersion(String),
}
