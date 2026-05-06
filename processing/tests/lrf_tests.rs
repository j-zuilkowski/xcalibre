use std::io::Write;
use tempfile::NamedTempFile;
use xcalibre_processing::metadata::lrf::extract as meta_extract;
use xcalibre_processing::text::lrf::extract as text_extract;

/// Minimal LRF binary: magic + version + xor_key + object_count + padding,
/// followed by an embedded title/author string in the body for scanning.
fn make_lrf(title: &str, author: &str, text_content: &str) -> Vec<u8> {
    let mut out = Vec::new();
    // LRF magic: "LRF\0"
    out.extend_from_slice(b"LRF\x00");
    // version: 999 as u16le
    out.extend_from_slice(&999u16.to_le_bytes());
    // xor_key: 0 as u16le
    out.extend_from_slice(&0u16.to_le_bytes());
    // object_count placeholder: 0 as u32le
    out.extend_from_slice(&0u32.to_le_bytes());
    // Embed simple key=value metadata that our scanner can find
    out.extend_from_slice(format!("Title={}\0Author={}\0", title, author).as_bytes());
    // Embed text content
    out.extend_from_slice(text_content.as_bytes());
    out
}

#[test]
fn lrf_metadata_extracts_title() {
    let data = make_lrf("My LRF Book", "Jane Author", "");
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(&data).unwrap();
    let meta = meta_extract(f.path()).unwrap();
    assert_eq!(meta.title.as_deref(), Some("My LRF Book"));
}

#[test]
fn lrf_metadata_extracts_author() {
    let data = make_lrf("Title", "John Smith", "");
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(&data).unwrap();
    let meta = meta_extract(f.path()).unwrap();
    assert!(meta.authors.contains(&"John Smith".to_string()));
}

#[test]
fn lrf_text_extracts_content() {
    let data = make_lrf("", "", "The quick brown fox");
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(&data).unwrap();
    let result = text_extract(f.path()).unwrap();
    assert!(result.full_text.contains("quick brown"), "{:?}", result.full_text);
    assert!(result.word_count >= 3);
}

#[test]
fn lrf_wrong_magic_returns_defaults() {
    let mut f = NamedTempFile::new().unwrap();
    // Non-printable bytes: bad magic, no recoverable text content.
    f.write_all(b"\xFF\xFE\xFF\xFE\xFF\xFE\xFF\xFE\xFF\xFE").unwrap();
    // Should not panic; returns empty/default (no readable title to recover)
    let meta = meta_extract(f.path()).unwrap();
    assert!(meta.title.is_none() || meta.title.as_deref() == Some(""));
}
