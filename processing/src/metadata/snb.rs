use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

/// SNB (Shanda Bambook) — proprietary Chinese ebook format.
/// Magic bytes: "SNBP" at offset 0.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    Ok(BookMetadata {
        title: path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.replace(['_', '-'], " ")),
        ..BookMetadata::default()
    })
}
