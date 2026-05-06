use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

/// Plain text extraction: read the file as UTF-8, replacing invalid sequences.
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let raw = std::fs::read(path).map_err(ProcessingError::IoError)?;
    let full_text = String::from_utf8_lossy(&raw).into_owned();
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}
