use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

const DJVU_MAGIC: &[u8] = b"AT&TFORM";

/// DjVu text extraction: parse the IFF chunk structure to locate TXTz chunks
/// (zlib-compressed hidden text layer), decompress and extract the text strings.
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let data = std::fs::read(path).map_err(ProcessingError::IoError)?;

    if data.len() < 16 || &data[..8] != DJVU_MAGIC {
        return Ok(ExtractedText { full_text: String::new(), word_count: 0 });
    }

    // Header: "AT&TFORM" (8) + u32be size (4) + "DJVU" (4) = 16 bytes.
    let mut full_text = String::new();
    let mut pos = 16usize;

    while pos + 8 <= data.len() {
        let id = &data[pos..pos + 4];
        let size = u32::from_be_bytes([
            data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7],
        ]) as usize;
        pos += 8;

        let end = (pos + size).min(data.len());
        let chunk_data = &data[pos..end];

        if id == b"TXTz" {
            if let Ok(text) = decompress_txtz(chunk_data) {
                let extracted = extract_djvu_text_strings(&text);
                if !extracted.is_empty() {
                    if !full_text.is_empty() { full_text.push(' '); }
                    full_text.push_str(&extracted);
                }
            }
        }

        // Advance past chunk (padded to even byte boundary).
        pos += if size.is_multiple_of(2) { size } else { size + 1 };
    }

    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}

fn decompress_txtz(data: &[u8]) -> Result<String, ProcessingError> {
    use std::io::Read;
    let mut decoder = flate2::read::ZlibDecoder::new(data);
    let mut out = String::new();
    decoder.read_to_string(&mut out)
        .map_err(|e: std::io::Error| ProcessingError::TextError(e.to_string()))?;
    Ok(out)
}

/// Extract quoted string content from DjVu text layer S-expressions.
/// Format: `(page ... (line ... "text1") (line ... "text2") ...)`
fn extract_djvu_text_strings(s_expr: &str) -> String {
    let mut result = String::new();
    let mut in_quote = false;
    let mut current = String::new();

    for ch in s_expr.chars() {
        match ch {
            '"' if !in_quote => in_quote = true,
            '"' if in_quote => {
                in_quote = false;
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    if !result.is_empty() { result.push(' '); }
                    result.push_str(&trimmed);
                }
                current.clear();
            }
            _ if in_quote => current.push(ch),
            _ => {}
        }
    }
    result
}
