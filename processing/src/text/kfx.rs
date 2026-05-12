use crate::error::ProcessingError;
use crate::text::ExtractedText;
use rusqlite::Connection;
use std::path::Path;

/// Extract text from KFX content fragments.
/// KFX content is stored in fragments with ftype '$608' (text content).
/// The payload is a binary blob; we extract readable UTF-8 sequences from it.
pub fn extract(path: &Path) -> Result<ExtractedText, ProcessingError> {
    let conn = Connection::open(path)
        .map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let mut stmt = conn.prepare(
        "SELECT payload FROM fragments WHERE ftype = '$608' AND payload IS NOT NULL
         ORDER BY rowid"
    ).map_err(|e| ProcessingError::TextError(e.to_string()))?;

    let blobs: Vec<Vec<u8>> = stmt.query_map([], |row| {
        let b: Vec<u8> = row.get(0)?;
        Ok(b)
    })
    .map_err(|e| ProcessingError::TextError(e.to_string()))?
    .filter_map(|r| r.ok())
    .collect();

    let mut parts = Vec::new();
    for blob in &blobs {
        // KFX content fragments may be UTF-8 text, or binary with embedded text.
        // Try direct UTF-8 first, then extract printable ASCII sequences.
        let text = if let Ok(s) = std::str::from_utf8(blob) {
            s.to_string()
        } else {
            extract_utf8_sequences(blob)
        };
        let cleaned = clean_kfx_text(&text);
        if !cleaned.is_empty() { parts.push(cleaned); }
    }

    let full_text  = parts.join("\n\n");
    let word_count = full_text.split_whitespace().count();
    Ok(ExtractedText { full_text, word_count })
}

fn extract_utf8_sequences(data: &[u8]) -> String {
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        if data[i] >= 0x20 && data[i] < 0x7F {
            let start = i;
            while i < data.len() && data[i] >= 0x20 && data[i] < 0x7F { i += 1; }
            if i - start >= 4 { // minimum meaningful sequence
                if let Ok(s) = std::str::from_utf8(&data[start..i]) {
                    result.push_str(s);
                    result.push(' ');
                }
            }
        } else {
            i += 1;
        }
    }
    result
}

fn clean_kfx_text(text: &str) -> String {
    // Strip KFX markup tags (similar to HTML stripping)
    let tag_re = regex::Regex::new(r"<[^>]+>").unwrap();
    let cleaned = tag_re.replace_all(text, " ");
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}
