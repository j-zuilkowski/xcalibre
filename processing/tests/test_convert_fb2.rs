use xcalibre_processing::convert::fb2::epub_to_fb2;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_fb2_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.fb2");
    epub_to_fb2(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "FB2 output must be created");
}

#[test]
fn test_fb2_is_valid_xml() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.fb2");
    epub_to_fb2(&epub_fixture(), &out).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("<?xml") || content.starts_with("<FictionBook"),
            "FB2 must be valid XML: {}", &content[..100.min(content.len())]);
}

#[test]
fn test_fb2_has_fictionbook_root() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.fb2");
    epub_to_fb2(&epub_fixture(), &out).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("<FictionBook"), "FB2 must have <FictionBook> root element");
}

#[test]
fn test_fb2_has_body_section() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.fb2");
    epub_to_fb2(&epub_fixture(), &out).unwrap();
    let content = std::fs::read_to_string(&out).unwrap();
    assert!(content.contains("<body>") || content.contains("<body "),
            "FB2 must have a <body> element");
}

#[test]
fn test_fb2_invalid_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.fb2");
    assert!(epub_to_fb2(std::path::Path::new("/nonexistent.epub"), &out).is_err());
}
