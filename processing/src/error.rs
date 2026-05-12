//! Unified error type for the `xcalibre-processing` crate.
//!
//! All public functions return `Result<T, ProcessingError>`. Tauri commands in
//! `src-tauri/src/commands.rs` map this to `String` via `.map_err(|e| e.to_string())`,
//! which propagates the display message to the JavaScript frontend.
//!
//! New variants should be added here rather than using ad-hoc string errors so
//! that callers can pattern-match on specific failure modes.

use thiserror::Error;

/// The single error type used throughout `xcalibre-processing`.
#[derive(Debug, Error)]
pub enum ProcessingError {
    /// Filesystem operations: file not found, permission denied, etc.
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// SQLite query failures surfaced by sqlx. Wraps `sqlx::Error` so callers
    /// don't need to depend on sqlx directly.
    #[error("database error: {0}")]
    DbError(#[from] sqlx::Error),

    /// The file's magic bytes or extension did not match any known format.
    /// The payload typically includes the file path or a description of what
    /// was found (e.g. `"SQLite (non-KFX)"`).
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    /// A structural integrity check failed (truncated ZIP, missing PDF `%%EOF`,
    /// MOBI header too small, etc.).
    #[error("integrity check failed: {0}")]
    IntegrityError(String),

    /// Metadata extraction (OPF parsing, FB2 XML parsing, etc.) failed.
    #[error("metadata extraction failed: {0}")]
    MetadataError(String),

    /// Full-text extraction from EPUB spine, PDF, or similar failed.
    #[error("text extraction failed: {0}")]
    TextError(String),

    /// Cover image extraction or resizing failed.
    #[error("cover extraction failed: {0}")]
    CoverError(String),

    /// The file has already been imported (same SHA-256 found in `jobs` with
    /// `status = 'COMPLETED'`). The payload is the existing job ID.
    #[error("duplicate file already imported: job_id={0}")]
    Duplicate(String),

    /// The caller supplied invalid arguments (empty title, malformed query, etc.).
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// A format conversion step failed (e.g. EPUB → PDF pipeline error).
    #[error("conversion error: {0}")]
    ConversionError(String),
}
