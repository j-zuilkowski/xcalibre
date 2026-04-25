use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use std::io::Read;
use std::path::Path;

/// DjVu — magic bytes: "AT&T" followed by "FORM" (IFF-based container).
/// The DjVu metadata chunk (INFO) contains image dimensions but not bibliographic metadata.
/// Best-effort: scan for an Annot chunk with key-value pairs.
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let mut file = std::fs::File::open(path).map_err(ProcessingError::IoError)?;
    let mut buf = vec![0u8; 2048];
    let n = file.read(&mut buf).map_err(ProcessingError::IoError)?;

    let mut title = None;
    let mut authors = Vec::new();

    if let Ok(s) = std::str::from_utf8(&buf[..n]) {
        if let Some(pos) = s.find("(title ") {
            let after = &s[pos + 7..];
            if let Some(end) = after.find(')') {
                let value = after[..end]
                    .trim_matches(|c| c == '"' || c == '\'')
                    .trim()
                    .to_string();
                if !value.is_empty() {
                    title = Some(value);
                }
            }
        }
        if let Some(pos) = s.find("(author ") {
            let after = &s[pos + 8..];
            if let Some(end) = after.find(')') {
                let value = after[..end]
                    .trim_matches(|c| c == '"' || c == '\'')
                    .trim()
                    .to_string();
                if !value.is_empty() {
                    authors.push(value);
                }
            }
        }
    }

    if title.is_none() {
        title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.replace(['_', '-'], " "));
    }

    Ok(BookMetadata {
        title,
        authors,
        ..BookMetadata::default()
    })
}
