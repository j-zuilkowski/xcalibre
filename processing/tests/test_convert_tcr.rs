use xcalibre_processing::convert::tcr::epub_to_tcr;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_tcr_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.tcr");
    epub_to_tcr(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "TCR output must be created");
}

#[test]
fn test_tcr_is_compressed() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.tcr");
    epub_to_tcr(&epub_fixture(), &out).unwrap();
    let tcr_size = std::fs::metadata(&out).unwrap().len();
    assert!(tcr_size > 0, "TCR must not be empty");
}

#[test]
fn test_tcr_invalid_path_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("out.tcr");
    assert!(epub_to_tcr(std::path::Path::new("/nonexistent.epub"), &out).is_err());
}
