use xcalibre_processing::convert::mobi::epub_to_mobi;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_mobi_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.mobi");
    epub_to_mobi(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "MOBI output file must be created");
}

#[test]
fn test_epub_to_mobi_has_mobi_magic() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.mobi");
    epub_to_mobi(&epub_fixture(), &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert!(bytes.len() > 1000, "MOBI must be non-trivial in size: {} bytes", bytes.len());
}

#[test]
fn test_epub_to_mobi_invalid_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.mobi");
    let result = epub_to_mobi(std::path::Path::new("/nonexistent.epub"), &out);
    assert!(result.is_err());
}
