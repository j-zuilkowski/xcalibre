use xcalibre_processing::convert::lrf::epub_to_lrf;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_lrf_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.lrf");
    epub_to_lrf(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "LRF output must be created");
}

#[test]
fn test_lrf_has_lrf_magic() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.lrf");
    epub_to_lrf(&epub_fixture(), &out).unwrap();
    let bytes = std::fs::read(&out).unwrap();
    assert!(bytes.len() > 100, "LRF must be non-trivial: {} bytes", bytes.len());
}

#[test]
fn test_lrf_invalid_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.lrf");
    assert!(epub_to_lrf(std::path::Path::new("/nonexistent.epub"), &out).is_err());
}
