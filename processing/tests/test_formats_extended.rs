use std::path::PathBuf;
use xcalibre_processing::{metadata, text};

#[test]
fn test_docx_metadata() {
    let path = PathBuf::from("tests/fixtures/fixture_docx.docx");
    let meta = metadata::docx::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("DOCX Fixture"));
    assert!(meta.authors.contains(&"Test Author".to_string()));
}

#[test]
fn test_docx_text() {
    let path = PathBuf::from("tests/fixtures/fixture_docx.docx");
    let extracted = text::docx::extract(&path).unwrap();
    assert!(extracted.word_count > 0);
}

#[test]
fn test_odt_metadata() {
    let path = PathBuf::from("tests/fixtures/fixture_odt.odt");
    let meta = metadata::odt::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("ODT Fixture"));
}

#[test]
fn test_odt_text() {
    let path = PathBuf::from("tests/fixtures/fixture_odt.odt");
    let extracted = text::odt::extract(&path).unwrap();
    assert!(extracted.word_count > 0);
}

#[test]
fn test_cbz_metadata_series() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Batman_042.cbz");

    let file = std::fs::File::create(&path).unwrap();
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
    zip.start_file("page001.jpg", opts).unwrap();
    zip.finish().unwrap();

    let meta = metadata::cbz::extract(&path).unwrap();
    assert_eq!(meta.series.as_deref(), Some("Batman"));
    assert_eq!(meta.series_index, Some(42.0));
}

#[test]
fn test_fb2_fixture_detect() {
    let path = PathBuf::from("tests/fixtures/fixture_fb2.fb2");
    let meta = metadata::fb2::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("FB2 Fixture"));
}

#[test]
fn test_html_fixture_detect() {
    let path = PathBuf::from("tests/fixtures/fixture_html.html");
    let meta = metadata::html::extract(&path).unwrap();
    assert_eq!(meta.title.as_deref(), Some("HTML Fixture"));
}
