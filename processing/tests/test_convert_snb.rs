use xcalibre_processing::convert::snb::epub_to_snb_text;
use std::path::PathBuf;

fn epub_fixture() -> PathBuf { PathBuf::from("tests/fixtures/fixture_epub.epub") }

#[test]
fn test_epub_to_snb_creates_file() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.snb");
    epub_to_snb_text(&epub_fixture(), &out).expect("conversion");
    assert!(out.exists(), "SNB output must be created");
}

#[test]
fn test_snb_output_has_content() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("output.snb");
    epub_to_snb_text(&epub_fixture(), &out).unwrap();
    let size = std::fs::metadata(&out).unwrap().len();
    assert!(size > 100, "SNB output must be non-trivial: {size} bytes");
}
