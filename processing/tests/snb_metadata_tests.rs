use std::io::Write;
use std::path::PathBuf;
use xcalibre_processing::{metadata, text};

/// Build a minimal SNB-like file with embedded book.snbf XML metadata.
/// SNB magic = "SNBP" at offset 0. The file table points to a book.snbf
/// entry containing XML metadata with title, author, and publisher fields.
fn write_snb(path: &PathBuf, book_snbf_xml: &[u8]) {
    let xml_bytes = book_snbf_xml.to_vec();
    let xml_len = xml_bytes.len();

    let mut file = std::fs::File::create(path).unwrap();

    // Calculate layout:
    // Offset 0-3: "SNBP" magic
    // Offset 4-7: section count (LE u32) = 1
    // Offset 8-23: section header (16 bytes)
    //   - type (4 bytes LE u32) = 0x02 (file table)
    //   - offset (4 bytes LE u32) = 24 (start of file table)
    //   - length (4 bytes LE u32) = file table length
    //   - flags (4 bytes) = 0
    // Offset 24+: file table
    //   - "book.snbf" (9 bytes)
    //   - file offset (4 bytes LE u32) = offset of XML data
    //   - file length (4 bytes LE u32) = xml_len
    // Then: XML data

    // Layout:
    // 0-3: "SNBP"
    // 4-7: section_count = 1
    // 8-23: section header
    // 24-32: "book.snbf" (9 bytes)
    // 33-36: file_offset (4 bytes) = 37
    // 37-40: file_length (4 bytes) = xml_len
    // 41+: XML data

    let file_offset: u32 = 41;
    let ft_start: u32 = 24;
    let ft_len: u32 = (file_offset - ft_start) as u32 + xml_len as u32;

    // Magic
    file.write_all(b"SNBP").unwrap();
    // Section count = 1
    file.write_all(&(1u32.to_le_bytes())).unwrap();
    // Section header: type=0x02, offset=24, length=ft_len, flags=0
    file.write_all(&(0x02u32.to_le_bytes())).unwrap(); // type
    file.write_all(&ft_start.to_le_bytes()).unwrap(); // offset
    file.write_all(&ft_len.to_le_bytes()).unwrap();   // length
    file.write_all(&(0u32.to_le_bytes())).unwrap();   // flags

    // File table: "book.snbf" + file_offset + file_length
    file.write_all(b"book.snbf").unwrap();
    file.write_all(&file_offset.to_le_bytes()).unwrap();
    file.write_all(&(xml_len as u32).to_le_bytes()).unwrap();

    // XML data
    file.write_all(&xml_bytes).unwrap();
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
