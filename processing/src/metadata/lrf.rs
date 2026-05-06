use crate::error::ProcessingError;
use crate::metadata::BookMetadata;
use crate::utils::recover::recover_title;
use std::path::Path;

const LRF_MAGIC: &[u8] = b"LRF\x00";

/// LRF / LRX (Sony BroadBand eBook).
/// Scans the binary for `Title=` and `Author=` key=value pairs embedded
/// in the file's metadata objects. Falls back to heuristic title recovery
/// via `recover_title()` when magic bytes do not match; rejects recovered
/// values that contain no alphabetic characters (e.g. pure binary noise).
pub fn extract(path: &Path) -> Result<BookMetadata, ProcessingError> {
    let data = std::fs::read(path).map_err(ProcessingError::IoError)?;

    // Validate magic bytes — fall back to heuristic recovery for non-standard files.
    if data.len() < 8 || &data[..4] != LRF_MAGIC {
        let title = recover_title(path)?
            .filter(|t| t.chars().any(|c| c.is_alphabetic()));
        return Ok(BookMetadata { title, ..BookMetadata::default() });
    }

    let text = String::from_utf8_lossy(&data);

    let title = scan_kv(&text, "Title");
    let author = scan_kv(&text, "Author");
    let authors = author.map(|a| vec![a]).unwrap_or_default();

    Ok(BookMetadata {
        title,
        authors,
        ..BookMetadata::default()
    })
}

/// Find `Key=value\0` or `Key=value\n` pattern in text.
fn scan_kv(text: &str, key: &str) -> Option<String> {
    let prefix = format!("{}=", key);
    let pos = text.find(prefix.as_str())?;
    let after = &text[pos + prefix.len()..];
    let end = after.find(|c: char| ['\0', '\n', '\r'].contains(&c))
        .unwrap_or(after.len().min(256));
    let value = after[..end].trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}
