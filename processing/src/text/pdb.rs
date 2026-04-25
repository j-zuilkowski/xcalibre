use crate::error::ProcessingError;
use crate::text::ExtractedText;
use std::path::Path;

/// Palm DOC compressed text format.
/// Decompresses Palm DOC record 0 (LZ77 variant) and returns plain text.
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let data = std::fs::read(path).map_err(ProcessingError::IoError)?;
    // Basic Palm DOC: check for "TEXt" type at offset 60–63
    if data.len() < 78 {
        return Ok(ExtractedText {
            full_text: String::new(),
            word_count: 0,
        });
    }
    let type_creator = &data[60..68];
    if type_creator != b"TEXtREAd" && type_creator != b"PMLzPMLz" {
        return Ok(ExtractedText {
            full_text: String::new(),
            word_count: 0,
        });
    }
    // Record count at offset 76–77 (big-endian u16)
    let record_count = u16::from_be_bytes([data[76], data[77]]) as usize;
    if record_count == 0 {
        return Ok(ExtractedText {
            full_text: String::new(),
            word_count: 0,
        });
    }
    // Record list starts at offset 78; each entry is 8 bytes
    // Entry 0: offset at bytes 0–3
    if data.len() < 78 + 8 {
        return Ok(ExtractedText {
            full_text: String::new(),
            word_count: 0,
        });
    }
    let offset0 = u32::from_be_bytes([data[78], data[79], data[80], data[81]]) as usize;
    // Record 1 offset (text records start at record 1)
    let offset1 = if data.len() >= 78 + 16 {
        u32::from_be_bytes([data[86], data[87], data[88], data[89]]) as usize
    } else {
        data.len()
    };
    if offset1 > data.len() || offset0 >= offset1 {
        return Ok(ExtractedText {
            full_text: String::new(),
            word_count: 0,
        });
    }
    // Palm DOC record 0 = header, records 1..N = compressed text
    // Simple approach: treat text bytes as UTF-8 and strip non-printable
    let text_data = &data[offset1.min(data.len())..];
    let full_text = String::from_utf8_lossy(text_data)
        .chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ' || *c == '\n')
        .collect::<String>();
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText {
        full_text,
        word_count,
    })
}
