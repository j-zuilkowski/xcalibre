use std::io::Write;
use std::path::PathBuf;
use xcalibre_processing::{metadata, text};

/// Build a minimal LIT-like file with embedded OPF metadata.
/// LIT magic = "ITOLITLS" at offset 0. The `/meta` entry contains an OPF
/// binary-tagged metadata block with title and author fields.
fn write_lit(path: &PathBuf, meta_block: &[u8]) {
    let mut file = std::fs::File::create(path).unwrap();
    // LIT header magic
    file.write_all(b"ITOLITLS").unwrap();
    // Pad with zeros to simulate container structure
    file.write_all(&[0u8; 64]).unwrap();
    // Write the /meta marker followed by the OPF metadata block
    file.write_all(b"/meta").unwrap();
    file.write_all(meta_block).unwrap();
    file.write_all(&[0u8; 10]).unwrap();
}

#[test]
fn test_lit_extracts_title_from_opf_meta() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_title.lit");

    // OPF binary-tagged metadata: "TITLE=" followed by null-terminated title text.
    let mut meta = Vec::new();
    meta.extend_from_slice(b"TITLE=LIT Test Book\x00");
    meta.extend_from_slice(b"AUTHOR=Mark Author\x00");

    write_lit(&path, &meta);

    let meta = metadata::lit::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("LIT Test Book"));
}

#[test]
fn test_lit_extracts_author_from_opf() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_author.lit");

    let mut meta = Vec::new();
    meta.extend_from_slice(b"AUTHOR=Mark Author\x00");
    meta.extend_from_slice(b"TITLE=Some Book\x00");

    write_lit(&path, &meta);

    let result = metadata::lit::extract(&path).unwrap();
    assert!(result.authors.contains(&"Mark Author".to_string()));
}

#[test]
fn test_lit_falls_back_to_recover_title_when_no_meta_entry() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("No_Meta_Book.lit");

    // LIT header but NO /meta marker
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(b"ITOLITLS").unwrap();
    file.write_all(&[0u8; 200]).unwrap();

    let meta = metadata::lit::extract(&path).unwrap();
    // Should fall back to recover_title from filename.
    assert!(meta.title.is_some());
    assert!(!meta.title.as_deref().unwrap_or("").is_empty());
}

#[test]
fn test_lit_text_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_text.lit");

    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(b"ITOLITLS").unwrap();
    // Embed some readable text for the heuristic to find.
    file.write_all(b"Hello world chapter one. This is the story.").unwrap();
    file.write_all(&[0u8; 100]).unwrap();

    let result = text::lit::extract(&path).unwrap();
    // Text extraction via recover_readable_text should still work.
    assert!(result.word_count > 0);
}
