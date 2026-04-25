use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

/// AZW4 — Amazon's PDF-wrapper format (DRM-protected).
/// Structurally similar to MOBI but wraps a PDF stream.
/// Without DRM keys, only filename-based metadata is possible.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    Ok(BookMetadata {
        title: path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.replace(['_', '-'], " ")),
        ..BookMetadata::default()
    })
}
