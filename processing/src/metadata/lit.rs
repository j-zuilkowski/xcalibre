use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use crate::utils::recover::recover_title;
use std::path::Path;

/// LIT (Microsoft Reader) — magic bytes: "ITOLITLS" at offset 0.
/// The format is discontinued, so we recover a readable title from the file
/// contents and otherwise fall back to the filename.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    Ok(BookMetadata {
        title: recover_title(path)?,
        ..BookMetadata::default()
    })
}
