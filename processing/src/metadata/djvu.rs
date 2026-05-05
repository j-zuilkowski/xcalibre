use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::path::Path;

/// DjVu — IFF-based format (AT&TFORM/DJVU).
/// Scans the full file for `(metadata ...)` Annot blocks containing
/// title and author key-value pairs.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let data = std::fs::read(path).map_err(ProcessingError::IoError)?;
    let text = String::from_utf8_lossy(&data);

    let title = scan_djvu_kv(&text, "title");
    let author = scan_djvu_kv(&text, "author");
    let authors = author.map(|a| vec![a]).unwrap_or_default();

    Ok(BookMetadata { title, authors, ..BookMetadata::default() })
}

/// Scan for `(key "value")` or `(key 'value')` patterns in DjVu annotation syntax.
fn scan_djvu_kv(text: &str, key: &str) -> Option<String> {
    let prefix = format!("({} ", key);
    let pos = text.find(prefix.as_str())?;
    let after = &text[pos + prefix.len()..];
    // Value is quoted with " or '
    let (open, close) = if after.starts_with('"') { ('"', '"') } else { ('\'', '\'') };
    if after.starts_with(open) {
        let inner = &after[1..];
        let end = inner.find(close)?;
        let value = inner[..end].trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    } else {
        // Unquoted value up to closing paren
        let end = after.find(')')?;
        let value = after[..end].trim().to_string();
        if value.is_empty() { None } else { Some(value) }
    }
}
