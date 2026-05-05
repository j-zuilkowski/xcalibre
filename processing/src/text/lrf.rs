use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

const LRF_MAGIC: &[u8] = b"LRF\x00";

/// LRF text extraction: scan for printable ASCII runs (length > 20 chars)
/// after skipping the binary header. LRF text blocks are not easily decompressed
/// without LZX support, but embedded strings in text objects are recoverable.
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let data = std::fs::read(path).map_err(ProcessingError::IoError)?;

    if data.len() < 8 || &data[..4] != LRF_MAGIC {
        return Ok(ExtractedText { full_text: String::new(), word_count: 0 });
    }

    // Skip the 12-byte fixed header.
    let body = &data[12.min(data.len())..];

    // Collect runs of printable Latin characters (length >= 8).
    let mut full_text = String::new();
    let mut run = String::new();
    for &b in body {
        if (0x20..0x7F).contains(&b) {
            run.push(b as char);
        } else {
            if run.len() >= 8 {
                if !full_text.is_empty() {
                    full_text.push(' ');
                }
                full_text.push_str(run.trim());
            }
            run.clear();
        }
    }
    if run.len() >= 8 {
        if !full_text.is_empty() { full_text.push(' '); }
        full_text.push_str(run.trim());
    }

    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}
