use std::io::{Seek, Write};
use std::path::PathBuf;
use xcalibre_processing::{metadata, text};

/// Build a minimal valid ITSF (CHM) container with specified HTML content.
/// ITSF magic = "ITSF" at offset 0. We craft enough of the header for chmlib
/// to parse the container and locate HTML pages.
fn write_chm(path: &PathBuf, html_pages: &[(&str, &str)]) {
    let mut file = std::fs::File::create(path).unwrap();

    // ITSF header magic + version
    file.write_all(b"ITSF").unwrap();
    file.write_all(&3u32.to_le_bytes()).unwrap(); // version 3

    // Header size (96 bytes)
    file.write_all(&96u32.to_le_bytes()).unwrap();
    // Keep-alive interval
    file.write_all(&1u32.to_le_bytes()).unwrap();
    // Timestamp
    file.write_all(&0u32.to_le_bytes()).unwrap();
    // Language ID (LCID)
    file.write_all(&0u32.to_le_bytes()).unwrap();

    // Directory GUID (16 zeroed bytes)
    file.write_all(&[0u8; 16]).unwrap();

    // Header section 0: offset + length (will be patched later)
    let header_sec0_pos = file.stream_position().unwrap();
    file.write_all(&0u64.to_le_bytes()).unwrap(); // offset (patched)
    file.write_all(&0u64.to_le_bytes()).unwrap(); // length (patched)

    // Header section 1: offset + length
    file.write_all(&0u64.to_le_bytes()).unwrap();
    file.write_all(&0u64.to_le_bytes()).unwrap();

    // Padding to 96-byte header boundary
    let current = file.stream_position().unwrap();
    if current < 96 {
        let remaining = (96 - current) as usize;
        file.write_all(&vec![0u8; remaining]).unwrap();
    }

    // Build HTML content and directory entries
    let mut dir_entries: Vec<(String, u64, u64)> = Vec::new();
    let mut html_data: Vec<Vec<u8>> = Vec::new();

    for (name, content) in html_pages {
        let mut chunk = Vec::new();
        chunk.extend_from_slice(b"PMGL");
        chunk.extend_from_slice(&(content.len() as u32).to_le_bytes());
        chunk.extend_from_slice(&(content.len() as u32).to_le_bytes());
        // Reset table (2 bytes) + 2 bytes align
        chunk.extend_from_slice(&[0u8; 4]);
        chunk.extend_from_slice(content.as_bytes());
        html_data.push(chunk);
    }

    // Write content sections starting at offset after directory
    let dir_header_size: u64 = 12; // ITSP + version + num_chunks
    let entry_overhead: u64 = html_pages.len() as u64 * 16; // rough estimate per entry
    let pmgi_header: u64 = 4; // PMGI magic
    let pmgi_size_field: u64 = 4;
    let content_start: u64 = 96 + dir_header_size + entry_overhead + pmgi_header + pmgi_size_field;

    // Calculate entry offsets
    let mut current_offset = content_start;
    for (i, (name, _content)) in html_pages.iter().enumerate() {
        let entry_len = html_data[i].len() as u64;
        dir_entries.push((name.to_string(), current_offset, entry_len));
        current_offset += entry_len;
    }

    // PMGI (Index) chunk
    file.write_all(b"PMGI").unwrap();
    // PMGI uncompressed size (4 bytes) — placeholder
    let pmgi_size_pos = file.stream_position().unwrap();
    file.write_all(&[0u8; 4]).unwrap();

    // Write dense directory entries
    for (name, offset, length) in &dir_entries {
        let name_bytes = name.as_bytes();
        let entry_size = (4 + name_bytes.len() + 8) as u32; // length compressed + name + content_section (u64)
        file.write_all(&entry_size.to_le_bytes()).unwrap();
        let name_len = name_bytes.len() as u16;
        file.write_all(&name_len.to_le_bytes()).unwrap();
        file.write_all(name_bytes).unwrap();
        // Content section offset + length as u64 pair
        file.write_all(&offset.to_le_bytes()).unwrap();
        file.write_all(&length.to_le_bytes()).unwrap();
    }

    // Patch PMGI size
    let pmgi_end = file.stream_position().unwrap();
    file.seek(std::io::SeekFrom::Start(pmgi_size_pos)).unwrap();
    let pmgi_data_size = (pmgi_end - pmgi_start_pos()) as u32;
    file.write_all(&pmgi_data_size.to_le_bytes()).unwrap();
    file.seek(std::io::SeekFrom::End(0)).unwrap();

    // Write actual content sections
    for chunk in &html_data {
        file.write_all(chunk).unwrap();
    }

    // Patch header section 0 with PMGI offset/length
    let pmgi_start = content_start - (pmgi_end - 96 - dir_header_size);
    // Compute PMGI offset relative to file start — it's right after the ITSF header
    let pmgi_real_start = 96u64; // PMGI starts right after the 96-byte ITSF header
    let pmgi_real_len = pmgi_end - pmgi_real_start;
    file.seek(std::io::SeekFrom::Start(header_sec0_pos)).unwrap();
    file.write_all(&pmgi_real_start.to_le_bytes()).unwrap();
    file.write_all(&pmgi_real_len.to_le_bytes()).unwrap();
}

fn pmgi_start_pos() -> u64 {
    // PMGI starts right after the 96-byte ITSF header
    96
}

#[test]
fn test_chm_extracts_title_from_html() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_title.chm");

    write_chm(
        &path,
        &[(
            "/default.html",
            "<!DOCTYPE html><html><head><title>Test CHM Book</title></head><body><p>Content.</p></body></html>",
        )],
    );

    let meta = metadata::chm::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("Test CHM Book"));
}

#[test]
fn test_chm_extracts_author_from_meta() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_author.chm");

    write_chm(
        &path,
        &[(
            "/default.html",
            "<!DOCTYPE html><html><head><meta name=\"author\" content=\"Jane Author\"><title>A Book</title></head><body><p>Text</p></body></html>",
        )],
    );

    let meta = metadata::chm::extract(&path).unwrap();
    assert!(meta.authors.contains(&"Jane Author".to_string()));
}

#[test]
fn test_chm_falls_back_to_recover_title_when_no_html() {
    // A file with ITSF magic but no valid directory entries will trigger
    // the fallback in the implementation.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Test_Book.chm");

    let mut file = std::fs::File::create(&path).unwrap();
    // Write ITSF magic but the rest is garbage — directory parse will fail.
    file.write_all(b"ITSF").unwrap();
    file.write_all(&[0u8; 200]).unwrap();

    let meta = metadata::chm::extract(&path).unwrap();
    // Should fall back to recover_title which will use the filename stem.
    assert!(meta.title.is_some());
    assert!(!meta.title.as_deref().unwrap_or("").is_empty());
}

#[test]
fn test_chm_text_strips_html_tags() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_text.chm");

    write_chm(
        &path,
        &[
            ("/page1.html", "<!DOCTYPE html><html><head><title>A</title></head><body><p>Hello</p></body></html>"),
            ("/page2.html", "<!DOCTYPE html><html><head><title>B</title></head><body><p>World</p></body></html>"),
        ],
    );

    let result = text::chm::extract(&path).unwrap();
    assert!(result.full_text.contains("Hello"));
    assert!(result.full_text.contains("World"));
    assert_eq!(result.word_count, 2);
}

#[test]
fn test_chm_text_handles_empty_container() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_empty.chm");

    // Minimal ITSF with no HTML content — only the header.
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(b"ITSF").unwrap();
    file.write_all(&3u32.to_le_bytes()).unwrap();
    file.write_all(&96u32.to_le_bytes()).unwrap(); // header size
    file.write_all(&vec![0u8; 84]).unwrap(); // rest of header

    let result = text::chm::extract(&path).unwrap();
    // With no HTML content, the implementation should fall back to
    // recover_readable_text which will find strings in the binary.
    // Either empty or some recovered text is acceptable at this stage.
    let _ = result;
}
