use xcalibre_processing::convert::rtf::epub_to_rtf;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_rtf_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.rtf");
    epub_to_rtf(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "RTF output must be created");
}

#[test]
fn test_rtf_starts_with_rtf_header() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.rtf");
    epub_to_rtf(&epub_fixture(), &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert!(bytes.starts_with(b"{\\rtf"),
            "RTF must start with {{\\rtf: {:?}", &bytes[..8.min(bytes.len())]);
}

#[test]
fn test_rtf_contains_text_content() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.rtf");
    epub_to_rtf(&epub_fixture(), &out).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    let ascii_count = content.chars().filter(|c| c.is_ascii_alphabetic()).count();
    assert!(ascii_count > 10, "RTF must contain readable text, got {} ascii chars", ascii_count);
}

#[test]
fn test_rtf_invalid_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.rtf");
    assert!(epub_to_rtf(std::path::Path::new("/nonexistent.epub"), &out).is_err());
}
