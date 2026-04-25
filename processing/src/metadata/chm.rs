use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use crate::utils::recover::recover_title;
use std::path::Path;

/// CHM (Microsoft HTML Help) — magic bytes: ITSF at offset 0.
/// This still falls back to filename recovery, but the shared text heuristics
/// let CHM titles come from embedded strings when they exist.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    Ok(BookMetadata {
        title: recover_title(path)?,
        ..BookMetadata::default()
    })
}
