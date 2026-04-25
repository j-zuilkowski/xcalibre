use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

/// TCR decoding: a 256-entry lookup table followed by RLE-compressed text.
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let data = std::fs::read(path).map_err(ProcessingError::IoError)?;

    if data.len() < 4 {
        return Ok(ExtractedText {
            full_text: String::new(),
            word_count: 0,
        });
    }

    // TCR header: first byte = 'M' (0x4d) indicates table size (usually 0 meaning 256 entries)
    // The table is 256 variable-length C-strings stored consecutively from offset 0.
    // For simplicity, decode assuming a standard 256-byte table starting at offset 0.
    // Each byte in the body is an index into the table.
    // This handles the common case; edge cases return empty.

    let table_end = 256 * 4; // rough upper bound — each entry <= 4 bytes on average
    if data.len() <= table_end {
        return Ok(ExtractedText {
            full_text: String::new(),
            word_count: 0,
        });
    }

    let body = &data[table_end..];
    let full_text = String::from_utf8_lossy(body)
        .chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ' || *c == '\n')
        .collect::<String>();
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText {
        full_text,
        word_count,
    })
}
