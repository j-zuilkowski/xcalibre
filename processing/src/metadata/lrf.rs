use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use crate::utils::recover::recover_title;
use std::path::Path;

/// LRF / LRX (Sony BroadBand eBook) fall back to recovered readable strings.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    Ok(BookMetadata {
        title: recover_title(path)?,
        ..BookMetadata::default()
    })
}
