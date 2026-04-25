use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessingError {
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    #[error("integrity check failed: {0}")]
    IntegrityError(String),

    #[error("metadata extraction failed: {0}")]
    MetadataError(String),

    #[error("text extraction failed: {0}")]
    TextError(String),

    #[error("cover extraction failed: {0}")]
    CoverError(String),

    #[error("duplicate file already imported: job_id={0}")]
    Duplicate(String),
}