use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

/// AZW4 wraps an embedded PDF. Scan the raw bytes for `%PDF-` magic,
/// write the embedded PDF to a temp file, and delegate to the PDF extractor.
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let data = std::fs::read(path).map_err(ProcessingError::IoError)?;

    // Locate the embedded PDF by searching for the %PDF- marker.
    let marker = b"%PDF-";
    let Some(offset) = data.windows(marker.len()).position(|w| w == marker) else {
        return Ok(ExtractedText { full_text: String::new(), word_count: 0 });
    };

    let pdf_data = &data[offset..];

    // Write to a temp file so the PDF extractor (which takes a &Path) can use it.
    let mut tmp = tempfile::NamedTempFile::new()
        .map_err(ProcessingError::IoError)?;
    use std::io::Write;
    tmp.write_all(pdf_data).map_err(ProcessingError::IoError)?;

    // Delegate to the PDF extractor. If it fails (stub/malformed PDF), return empty.
    crate::text::pdf::extract(tmp.path()).or_else(|_| {
        Ok(ExtractedText { full_text: String::new(), word_count: 0 })
    })
}
