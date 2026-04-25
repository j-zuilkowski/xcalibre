use crate::error::ProcessingError;
use crate::text::ExtractedText;
use crate::utils::recover::recover_readable_text;
use std::path::Path;

/// DjVu text extraction falls back to readable string recovery here.
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let full_text = recover_readable_text(path)?;
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}
