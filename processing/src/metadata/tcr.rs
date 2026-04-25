use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

/// TCR — simple run-length encoded text.
/// No embedded metadata; use filename as title.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    Ok(BookMetadata {
        title: path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.replace(['_', '-'], " ")),
        ..BookMetadata::default()
    })
}
