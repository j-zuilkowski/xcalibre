use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

/// CHM (Microsoft HTML Help) metadata extraction via filename heuristic.
///
/// Full ITSF container parsing requires chmlib-sys, which has a build-script
/// bug on Windows. We use the filename-based title recovery that works on all
/// platforms. In practice, CHM files rarely embed author metadata in a
/// machine-readable location anyway.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let title = crate::utils::recover::recover_title(path)?;
    Ok(BookMetadata {
        title,
        ..BookMetadata::default()
    })
}
