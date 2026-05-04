use std::io::Write;
use std::path::PathBuf;
use xcalibre_processing::{metadata, text};

/// Build a minimal SNB-like file with embedded book.snbf XML metadata.
/// SNB magic = "SNBP" at offset 0. The file table points to a book.snbf
/// entry containing XML metadata with title, author, and publisher fields.
fn write_snb(path: &PathBuf, book_snbf_xml: &[u8]) {
    let mut file = std::fs::File::create(path).unwrap();
    // SNB header magic
    file.write_all(b"SNBP").unwrap();
    // Pad with zeros to simulate container structure
    file.write_all(&[0u8; 64]).unwrap();
    // Write the book.snbf marker followed by the XML metadata
    file.write_all(b"book.snbf").unwrap();
    file.write_all(book_snbf_xml).unwrap();
    file.write_all(&[0u8; 10]).unwrap();
}

#[test]
fn test_snb_extracts_title_from_book_snbf() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_title.snb");

    let xml = "<?xml version=\"1.0\"?><book><title>南山南</title></book>";
    write_snb(&path, xml.as_bytes());

    let meta = metadata::snb::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("南山南"));
}

#[test]
fn test_snb_extracts_author_from_book_snbf() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_author.snb");

    let xml = "<?xml version=\"1.0\"?><book><author>张三</author><title>Test</title></book>";
    write_snb(&path, xml.as_bytes());

    let result = metadata::snb::extract(&path).unwrap();
    assert!(result.authors.contains(&"张三".to_string()));
}

#[test]
fn test_snb_extracts_publisher() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_publisher.snb");

    let xml = "<?xml version=\"1.0\"?><book><publisher>盛大文学</publisher><title>Test</title></book>";
    write_snb(&path, xml.as_bytes());

    let result = metadata::snb::extract(&path).unwrap();
    assert_eq!(result.publisher.as_deref(), Some("盛大文学"));
}

#[test]
fn test_snb_falls_back_to_filename_when_no_book_snbf() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("My_Great_Book.snb");

    // SNB header but NO book.snbf marker
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(b"SNBP").unwrap();
    file.write_all(&[0u8; 200]).unwrap();

    let meta = metadata::snb::extract(&path).unwrap();
    // Should fall back to filename stem with _/- replaced by spaces.
    assert_eq!(meta.title.as_deref(), Some("My Great Book"));
}

#[test]
fn test_snb_text_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_text.snb");

    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(b"SNBP").unwrap();
    // Embed some readable text for the heuristic to find.
    file.write_all(b"Hello world chapter one. This is the story.").unwrap();
    file.write_all(&[0u8; 100]).unwrap();

    let result = text::snb::extract(&path).unwrap();
    // Text extraction via recover_readable_text should still work.
    assert!(result.word_count > 0);
}
